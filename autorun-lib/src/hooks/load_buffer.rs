use std::ffi::{c_char, c_int};

use autorun_types::LuaState;

type LoadBufferFn = extern "C" fn(*mut LuaState, *const c_char, usize, *const c_char, *const c_char) -> c_int;

static LOAD_BUFFER_H: std::sync::OnceLock<retour::GenericDetour<LoadBufferFn>> = std::sync::OnceLock::new();

extern "C" fn load_buffer_h(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	let r = LOAD_BUFFER_H.get().unwrap().call(state, buff, size, name, mode);
	autorun_log::info!("Loadbuffer called");
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
