use core::ffi::{c_char, c_int};

use autorun_log::error;
use autorun_types::{LuaState, Realm};

type LoadBufferFn = extern "C-unwind" fn(*mut LuaState, *const c_char, usize, *const c_char, *const c_char) -> c_int;

static LOAD_BUFFER_H: std::sync::OnceLock<retour::GenericDetour<LoadBufferFn>> = std::sync::OnceLock::new();

// Holds a usize but really holding *mut LuaState
// This stores the previous Lua state acquired upon init's run.
static PREVIOUS_LUA_STATE: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

extern "C-unwind" fn load_buffer_h(
	state: *mut LuaState,
	mut buff: *const c_char,
	mut size: usize,
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

		disable();
		if let Err(why) = crate::events::client_init::run(state) {
			let name = unsafe { std::ffi::CStr::from_ptr(name) };
			let name = name.to_string_lossy();
			error!("Failed to run init for {name}: {why}");
		}
		enable();
	} else {
		// Hook
		let name_cstr = unsafe { std::ffi::CStr::from_ptr(name) };
		let buff_bytes = unsafe { std::ffi::CStr::from_ptr(buff).to_bytes() };
		let mode_bytes = unsafe { std::ffi::CStr::from_ptr(mode).to_bytes() };

		disable();
		match crate::events::hook::run(state, buff_bytes, name_cstr.to_bytes(), mode_bytes) {
			Ok(Some(x)) => {
				buff = x.as_ptr().cast::<c_char>();
				size = x.len();
			}
			Err(why) => {
				let name_cstr = name_cstr.to_string_lossy();
				error!("Failed to run hook for {name_cstr}: {why}");
			}
			_ => (),
		}
		enable();
	}

	call_original(state, buff, size, name, mode)
}

pub fn init() -> anyhow::Result<()> {
	let lua = autorun_lua::get_api()?;
	let target_fn = lua.raw.loadbufferx;

	let detour = unsafe {
		let detour = retour::GenericDetour::new(target_fn, load_buffer_h)?;
		detour.enable()?;
		detour
	};

	LOAD_BUFFER_H.set(detour).unwrap();

	Ok(())
}

pub fn disable() {
	if let Some(detour) = LOAD_BUFFER_H.get() {
		unsafe {
			detour.disable().unwrap();
		}
	}
}

pub fn enable() {
	if let Some(detour) = LOAD_BUFFER_H.get() {
		unsafe {
			detour.enable().unwrap();
		}
	}
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
