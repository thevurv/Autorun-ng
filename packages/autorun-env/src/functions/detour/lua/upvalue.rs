//! Handles upvalue overwriting to support detouring Lua functions.
//! This module provides the ability to overwrite upvalues of Lua functions,
//! which is essential for our detouring mechanism to pull the target function into a register

use anyhow::Context;
use autorun_luajit::{GCUpval, GCfuncL, TValue};

pub fn replace(func: GCfuncL, target_index: u32, replacement_tv: TValue) -> anyhow::Result<()> {
	if target_index >= func.header.nupvalues as u32 {
		anyhow::bail!(
			"Upvalue replacement index out of bounds: target_index {} exceeds number of upvalues {}.",
			target_index,
			func.header.nupvalues
		);
	}

	let upvalue_array_ptr = func.uvptr.as_ptr();
	let target_uv_gcr = unsafe { *upvalue_array_ptr.add(target_index as usize) };
	let target_uv = unsafe {
		target_uv_gcr
			.as_ptr::<GCUpval>()
			.as_ref()
			.context("Failed to deref GCUpval.")?
	};

	// #define uvval(uv_)	(mref((uv_)->v, TValue))
	let tvalue_ptr = unsafe { target_uv.v.as_mut_ptr::<TValue>() };
	// We want to *actually* overwrite the TValue at this location, not just reassign the pointer
	unsafe {
		std::ptr::write(tvalue_ptr, replacement_tv);
	}

	Ok(())
}
