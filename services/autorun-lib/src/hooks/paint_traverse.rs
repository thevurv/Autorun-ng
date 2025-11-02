use autorun_log::error;

type PaintTraverseFn = extern "C-unwind" fn(this: *mut std::ffi::c_void, panel: usize, force_repaint: bool, allow_force: bool);

static PAINT_TRAVERSE_H: std::sync::OnceLock<retour::GenericDetour<PaintTraverseFn>> = std::sync::OnceLock::new();

extern "C-unwind" fn paint_traverse_h(this: *mut std::ffi::c_void, panel_id: usize, force_repaint: bool, force_allow: bool) {
	PAINT_TRAVERSE_H
		.get()
		.unwrap()
		.call(this, panel_id, force_repaint, force_allow);

	let Some(callback) = crate::lua_queue::pop() else {
		return;
	};

	crate::hooks::load_buffer::disable();

	let lua = autorun_lua::get_api().unwrap();
	if let Err(why) = callback(lua) {
		error!("Error in PaintTraverse Lua callback: {}", why);
	}

	crate::hooks::load_buffer::enable();
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
