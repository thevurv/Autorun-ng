//! Saves the original state of a detoured function before it was applied.

use crate::functions::detour::lua::upvalue::overwrite_upvalue;
use anyhow::Context;
use autorun_luajit::{BCIns, GCfunc, TValue};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Clone)]
pub struct OriginalDetourState {
	pub original_bytecode: Vec<BCIns>,
	pub original_frame_size: u8,
	pub original_upvalue_0: TValue,
}

pub static ORIGINAL_DETOUR_STATES: LazyLock<Mutex<HashMap<usize, OriginalDetourState>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

impl OriginalDetourState {
	pub fn new() -> Self {
		Self {
			original_bytecode: Vec::new(),
			original_frame_size: 0,
			original_upvalue_0: TValue::nil(),
		}
	}
}

/// Associates the given function with its original detour state.
pub fn save_state(func: *mut GCfunc, state: OriginalDetourState) {
	ORIGINAL_DETOUR_STATES.lock().unwrap().insert(func as usize, state);
}

/// Retrieves the original detour state for the given function, if it exists.
pub fn get_state(func: *mut GCfunc) -> Option<OriginalDetourState> {
	ORIGINAL_DETOUR_STATES.lock().unwrap().get(&(func as usize)).cloned()
}

pub fn restore_func(func: *mut GCfunc) -> anyhow::Result<()> {
	let state = get_state(func).context("Failed to find original detour state for function.")?;
	let gcfunc_l = unsafe { (*func).as_l().context("Function is not a Lua function.")? };

	// Fix upvalues
	autorun_log::debug!("Fixing upvalues...");
	//overwrite_upvalue(gcfunc_l, 0, state.original_upvalue_0)?;
	autorun_log::debug!("Upvalues fixed.");

	// Restore bytecode and frame size
	autorun_log::debug!("Restoring bytecode...");

	// We don't want to instantiate a writer do it one by one, we can just copy directly
	let original_bytecode = state.original_bytecode;
	let bc_ptr = gcfunc_l.get_bc_ins().context("Failed to get bytecode pointer.")?;
	let proto = gcfunc_l.get_proto().context("Failed to get proto.")?;
	let proto = unsafe { proto.as_mut().context("Failed to dereference proto.")? };
	unsafe {
		std::ptr::copy_nonoverlapping(original_bytecode.as_ptr(), bc_ptr, original_bytecode.len());
	}

	autorun_log::debug!("Bytecode restored.");

	autorun_log::debug!("Restoring frame size...");
	proto.framesize = state.original_frame_size;
	autorun_log::debug!("Frame size restored.");

	// We no longer need to keep the state
	ORIGINAL_DETOUR_STATES.lock().unwrap().remove(&(func as usize));

	Ok(())
}
