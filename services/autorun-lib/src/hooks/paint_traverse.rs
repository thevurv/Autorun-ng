use crate::lua_queue::LUA_QUEUE;

type PaintTraverseFn = extern "C-unwind" fn(this: *mut std::ffi::c_void, panel: usize, force_repaint: bool, allow_force: bool);

static PAINT_TRAVERSE_H: std::sync::OnceLock<retour::GenericDetour<PaintTraverseFn>> = std::sync::OnceLock::new();

extern "C-unwind" fn paint_traverse_h(this: *mut std::ffi::c_void, panel_id: usize, force_repaint: bool, force_allow: bool) {
	PAINT_TRAVERSE_H
		.get()
		.unwrap()
		.call(this, panel_id, force_repaint, force_allow);

	let mut queue = LUA_QUEUE.lock().unwrap();
	if queue.is_empty() {
		return;
	}

	let lua = autorun_lua::get_api().expect("it's over");

	let (realm, source) = queue.remove(0);
	let state = match autorun_interfaces::lua::get_state(realm).expect("it's over") {
		Some(s) => s,
		None => {
			autorun_log::error!("Lua state for realm {realm:?} is not ready");
			return;
		}
	};

	crate::hooks::load_buffer::disable();
	if let Err(why) = lua.load_buffer_x(state, source.as_bytes(), c"RunString", c"t") {
		crate::hooks::load_buffer::enable();
		autorun_log::error!("Failed to load Lua string: {why}");
		return;
	}
	crate::hooks::load_buffer::enable();

	let existing_hook = lua.get_hook(state);
	let existing_hook_info = if existing_hook.is_null() {
		None
	} else {
		Some((existing_hook, lua.get_hook_mask(state), lua.get_hook_count(state)))
	};

	if existing_hook_info.is_some() {
		lua.set_hook(state, std::ptr::null(), 0, 0);
	}

	if let Err(why) = lua.pcall(state, 0, 0, 0) {
		autorun_log::error!("Failed to execute Lua code: {why}");
		return;
	}

	let did_user_set_hook = !lua.get_hook(state).is_null();
	if did_user_set_hook {
		autorun_log::warn!("User set a hook in executed code. This is not recommended.");
	}

	if !did_user_set_hook && let Some((hook, mask, count)) = existing_hook_info {
		lua.set_hook(state, hook, mask, count);
	}
}

#[allow(unused)]
pub fn init() -> anyhow::Result<()> {
	let vgui = autorun_interfaces::vgui::get_api()?;

	// + 1 for that rtti pointer
	#[cfg(target_os = "linux")]
	let paint_traverse_offset = 42;

	#[cfg(target_os = "windows")]
	let paint_traverse_offset = 41;

	let original = unsafe {
		std::mem::transmute::<*mut std::ffi::c_void, PaintTraverseFn>(
			((*vgui.vgui).vtable as *mut *mut std::ffi::c_void)
				.offset(paint_traverse_offset)
				.read(),
		)
	};

	let detour = unsafe {
		let detour = retour::GenericDetour::new(original, paint_traverse_h)?;
		detour.enable()?;
		detour
	};

	PAINT_TRAVERSE_H.set(detour).expect("Should never already be initialized");

	Ok(())
}
