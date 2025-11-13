//! Handles upvalue overwriting to support detouring Lua functions.
//! This module provides the ability to overwrite upvalues of Lua functions,
//! which is essential for our detouring mechanism to pull the target function into a register

use crate::functions::detour::lua::state::OriginalDetourState;
use anyhow::Context;
use autorun_luajit::{GCRef, GCUpval, GCfuncL, MRef, TValue};

pub fn replace(
	func: &GCfuncL,
	target_index: u32,
	replacement_tv: TValue,
	original_detour_state: &mut OriginalDetourState,
) -> anyhow::Result<()> {
	let old_uv = overwrite_upvalue(func, target_index, replacement_tv)?;
	// TODO: Support multiple upvalues in the future
	original_detour_state.original_upvalue_0 = old_uv;

	Ok(())
}

pub fn overwrite_upvalue(func: &GCfuncL, target_index: u32, replacement_tv: TValue) -> anyhow::Result<TValue> {
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
			.as_mut()
			.context("Failed to deref GCUpval.")?
	};

	// #define uvval(uv_)	(mref((uv_)->v, TValue))
	dbg!(&target_uv);
	autorun_log::debug!("UV pointer: {:p}", target_uv.v.as_ptr::<TValue>());
	autorun_log::debug!("Actual GCUpval pointer: {:p}", target_uv);

	let tvalue_ptr = unsafe { target_uv.v.as_mut_ptr::<TValue>() };
	let original_tv = unsafe { std::ptr::read(tvalue_ptr) };

	// We actually re-assign the pointer. We do not want to do it in-place as it will affect even clones of the function.
	// Unfortunately, it's a tad ugly. And also leaks memory. We can fix that later.
	unsafe {
		let new_tv = Box::new(replacement_tv);
		let new_tv_ptr = Box::into_raw(new_tv);
		target_uv.v.set_ptr(new_tv_ptr);
	}

	Ok(original_tv)
}
