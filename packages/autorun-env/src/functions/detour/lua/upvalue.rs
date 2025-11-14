//! Handles upvalue overwriting to support detouring Lua functions.
//! This module provides the ability to overwrite upvalues of Lua functions,
//! which is essential for our detouring mechanism to pull the target function into a register

use crate::functions::detour::lua::clone::close_upvalue;
use crate::functions::detour::lua::state::OriginalDetourState;
use anyhow::Context;
use autorun_luajit::{GCHeader, GCRef, GCSize, GCUpval, GCfuncL, LJState, MRef, TValue, mem_newgco};

pub fn replace(
	lj_state: &mut LJState,
	func: *mut GCfuncL,
	target_index: u32,
	replacement_tv: TValue,
	original_detour_state: &mut OriginalDetourState,
) -> anyhow::Result<()> {
	let old_uv = overwrite_upvalue(lj_state, func, target_index, replacement_tv)?;
	// TODO: Support multiple upvalues in the future
	original_detour_state.original_upvalue_0 = old_uv;

	Ok(())
}

pub fn overwrite_upvalue(
	lj_state: &mut LJState,
	func: *mut GCfuncL,
	target_index: u32,
	replacement_tv: TValue,
) -> anyhow::Result<TValue> {
	let nupvalues = unsafe { (*func).header.nupvalues };

	if target_index >= nupvalues as u32 {
		anyhow::bail!(
			"Upvalue replacement index out of bounds: target_index {} exceeds number of upvalues {}.",
			target_index,
			nupvalues
		);
	}

	let mut upvalue_array_ptr = unsafe { (*func).uvptr.as_mut_ptr() };

	let target_uv_gcr = unsafe { *upvalue_array_ptr.add(target_index as usize) };
	// We create a new GCupval to replace the existing one.
	// We can't modify it in-place or it will affect any other shared upvalues.
	// This is most prominent in a `hook.Call` detour (very common scenario). Modifiying in place would
	// cause hook.Add to see the modified upvalue as well.

	let target_uv = unsafe { target_uv_gcr.as_direct_ptr::<GCUpval>() };
	let new_target_uv = unsafe { mem_newgco::<GCUpval>(lj_state, size_of::<GCUpval>() as GCSize)? };

	// copy
	unsafe {
		std::ptr::copy_nonoverlapping(
			target_uv.byte_add(size_of::<GCHeader>()) as *const u8,
			new_target_uv.byte_add(size_of::<GCHeader>()) as *mut u8,
			size_of::<GCUpval>() - size_of::<GCHeader>(),
		);
	}

	// close it, and replace the value
	close_upvalue(new_target_uv, Some(replacement_tv))?;

	// update the GCRef to point to the new upvalue
	unsafe {
		upvalue_array_ptr
			.add(target_index as usize)
			.write(GCRef::from_ptr(new_target_uv as *mut ()));
	}

	Ok(TValue::nil())
}
