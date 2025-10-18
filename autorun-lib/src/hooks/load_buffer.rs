use std::ffi::{c_char, c_int};

use autorun_types::{LuaState, Realm};

type LoadBufferFn = extern "C-unwind" fn(*mut LuaState, *const c_char, usize, *const c_char, *const c_char) -> c_int;

static LOAD_BUFFER_H: std::sync::OnceLock<retour::GenericDetour<LoadBufferFn>> = std::sync::OnceLock::new();
static WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

extern "C-unwind" fn load_buffer_h(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	let is_client_state = Some(state) == autorun_interfaces::lua::get_state(Realm::Client).unwrap();
	if !is_client_state {
		return call_original(state, buff, size, name, mode);
	}

	let engine = autorun_interfaces::engine_client::get_api().unwrap();
	let is_drawing_loading_image = engine.is_drawing_loading_image();
	let previously_was_drawing_loading_image = *WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE.lock().unwrap();

	if is_drawing_loading_image && !previously_was_drawing_loading_image {
		unsafe { LOAD_BUFFER_H.get().unwrap().disable().unwrap() }
		if let Err(why) = crate::events::init::run(state) {
			let name = unsafe { std::ffi::CStr::from_ptr(name) };
			autorun_log::error!("Failed to run init for {}: {why}", name.to_string_lossy());
		}
		unsafe { LOAD_BUFFER_H.get().unwrap().enable().unwrap() }
	} else if Some(state) == autorun_interfaces::lua::get_state(Realm::Client).unwrap() {
		// let name = unsafe { std::ffi::CStr::from_ptr(name) };
		// let buff = unsafe { std::ffi::CStr::from_ptr(buff).to_bytes() };
		// let mode = unsafe { std::ffi::CStr::from_ptr(mode).to_bytes() };

		// unsafe { LOAD_BUFFER_H.get().unwrap().disable().unwrap() }
		// if let Err(why) = crate::events::hook::run(state, buff, name.to_bytes(), mode) {
		// 	autorun_log::error!("Failed to run hook for {}: {why}", name.to_string_lossy());
		// }
		// unsafe { LOAD_BUFFER_H.get().unwrap().enable().unwrap() }
	}

	*WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE.lock().unwrap() = is_drawing_loading_image;

	call_original(state, buff, size, name, mode)
}

pub fn init() -> anyhow::Result<()> {
	let lua = autorun_lua::get_api()?;
	let target_fn = lua._load_buffer_x;

	let detour = unsafe {
		let detour = retour::GenericDetour::new(target_fn, load_buffer_h)?;
		detour.enable()?;
		detour
	};

	LOAD_BUFFER_H.set(detour).unwrap();

	Ok(())
}

#[inline]
pub fn call_original(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	LOAD_BUFFER_H.get().unwrap().call(state, buff, size, name, mode)
}
