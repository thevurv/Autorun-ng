use std::ffi::{c_char, c_int};

use autorun_types::LuaState;

type LoadBufferFn = extern "C" fn(*mut LuaState, *const c_char, usize, *const c_char, *const c_char) -> c_int;

static LOAD_BUFFER_H: std::sync::OnceLock<retour::GenericDetour<LoadBufferFn>> = std::sync::OnceLock::new();
static WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

extern "C" fn load_buffer_h(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	let r = LOAD_BUFFER_H.get().unwrap().call(state, buff, size, name, mode);

	let engine = autorun_interfaces::engine_client::get_api().unwrap();

	let is_drawing_loading_image = engine.is_drawing_loading_image();
	let previously_was_drawing_loading_image = *WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE.lock().unwrap();

	if is_drawing_loading_image && !previously_was_drawing_loading_image {
		let name = unsafe { std::ffi::CStr::from_ptr(name).to_string_lossy() };
		autorun_log::info!("Hi there {name}");
	}

	*WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE.lock().unwrap() = is_drawing_loading_image;

	return r;
}

pub fn init() -> anyhow::Result<()> {
	let lua = autorun_lua::get_api()?;
	let target_fn = lua.load_buffer_x;

	let detour = unsafe {
		let detour = retour::GenericDetour::new(target_fn, load_buffer_h)?;
		detour.enable()?;
		detour
	};

	LOAD_BUFFER_H.set(detour).unwrap();

	Ok(())
}

pub fn call_original(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	LOAD_BUFFER_H.get().unwrap().call(state, buff, size, name, mode)
}
