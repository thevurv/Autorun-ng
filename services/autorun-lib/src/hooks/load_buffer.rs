use core::ffi::{c_char, c_int};

use autorun_types::{LuaState, Realm};

type LoadBufferFn = extern "C-unwind" fn(*mut LuaState, *const c_char, usize, *const c_char, *const c_char) -> c_int;

static LOAD_BUFFER_H: std::sync::OnceLock<retour::GenericDetour<LoadBufferFn>> = std::sync::OnceLock::new();
// static WAS_PREVIOUSLY_DRAWING_LOADING_IMAGE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

// Holds a usize but really holding *mut LuaState
// This stores the previous Lua state acquired upon init's run.
static PREVIOUS_LUA_STATE: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

extern "C-unwind" fn load_buffer_h(
	state: *mut LuaState,
	buff: *const c_char,
	size: usize,
	name: *const c_char,
	mode: *const c_char,
) -> c_int {
	// Ignore menu code
	if autorun_env::global::get_realm(state) != Realm::Client {
		return call_original(state, buff, size, name, mode);
	}

	// Init
	if *PREVIOUS_LUA_STATE.lock().unwrap() != (state as usize) {
		*PREVIOUS_LUA_STATE.lock().unwrap() = state as usize;

		unsafe { LOAD_BUFFER_H.get().unwrap().disable().unwrap() }
		if let Err(why) = crate::events::init::run(state) {
			let name = unsafe { std::ffi::CStr::from_ptr(name) };
			autorun_log::error!("Failed to run init for {}: {why}", name.to_string_lossy());
		}
		unsafe { LOAD_BUFFER_H.get().unwrap().enable().unwrap() }
	} else {
		// Hook
		let name = unsafe { std::ffi::CStr::from_ptr(name) };
		let buff = unsafe { std::ffi::CStr::from_ptr(buff).to_bytes() };
		let mode = unsafe { std::ffi::CStr::from_ptr(mode).to_bytes() };

		unsafe { LOAD_BUFFER_H.get().unwrap().disable().unwrap() }
		if let Err(why) = crate::events::hook::run(state, buff, name.to_bytes(), mode) {
			autorun_log::error!("Failed to run hook for {}: {why}", name.to_string_lossy());
		}
		unsafe { LOAD_BUFFER_H.get().unwrap().enable().unwrap() }
	}

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
