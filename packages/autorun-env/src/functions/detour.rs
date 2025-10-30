mod handlers;
mod raw;
mod userdata;

use crate::functions::detour::handlers::{detour_handler, retour_handler};
use crate::functions::detour::raw::{make_detour_trampoline, make_retour_lua_trampoline};
use crate::functions::detour::userdata::Detour;
use anyhow::Context;
use autorun_lua::{LuaApi, LuaFunction, LuaTypeId, RawHandle, RawLuaReturn};
use autorun_luajit::{GCfunc, LJState, get_gcobj, get_gcobj_mut};
use autorun_types::LuaState;
use retour::GenericDetour;
use std::ffi::c_int;
pub use userdata::{detour_disable, detour_enable, detour_get_original, detour_remove};

pub fn detour(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<Detour> {
	if lua.is_c_function(state, 1) == 0 {
		anyhow::bail!("First argument must be a C function to detour.");
	}

	let l = state as *mut LJState;
	let l_ref = unsafe { l.as_ref().context("Failed to dereference lua_State.")? };

	let gcfunc = get_gcobj::<GCfunc>(l_ref, 1).context("Failed to get GCfunc for target function.")?;

	if gcfunc.is_fast_function() {
		anyhow::bail!("Cannot detour a fast-function. Use copyFastFunction instead.");
	}

	let target_function: LuaFunction = unsafe { std::mem::transmute(gcfunc.c.c) };
	if target_function as usize == 0 {
		anyhow::bail!("Target function pointer is null.");
	}

	if lua.type_id(state, 2) != LuaTypeId::Function {
		anyhow::bail!("Second argument must be a function to use as detour.");
	}

	let detour_callback = RawHandle::from_stack(lua, state).context("Failed to create raw handle for detour callback.")?;
	let mut original_function_ptr = Box::new(0usize);

	// create the trampoline
	let detour_trampoline = make_detour_trampoline(
		lua,
		detour_callback.get_id(),
		original_function_ptr.as_ref() as *const usize,
		detour_handler,
	)?;

	let detour = unsafe {
		Box::new(GenericDetour::new(
			target_function,
			std::mem::transmute(detour_trampoline.as_ptr()),
		)?)
	};

	unsafe {
		detour
			.enable()
			.map_err(|e| anyhow::anyhow!("Failed to enable detour: {}", e))?;
	}

	// create retour trampoline
	let retour_trampoline = make_retour_lua_trampoline(detour.as_ref() as *const GenericDetour<LuaFunction>, retour_handler)?;

	// link the original function pointer
	*original_function_ptr = retour_trampoline.as_ptr() as usize;

	Ok(Detour {
		detour,
		_detour_callback: detour_callback,
		_detour_trampoline: detour_trampoline,
		_retour_trampoline: retour_trampoline,
		original_function_ptr,
	})
}

pub fn copy_fast_function(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	if lua.is_c_function(state, 1) == 0 {
		anyhow::bail!("First argument must be a C function to copy.");
	}

	if lua.type_id(state, 2) != LuaTypeId::Function {
		anyhow::bail!("Second argument must be a function to use.");
	}

	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState.")? };
	let gcfunc = get_gcobj::<GCfunc>(lj_state, 1).context("Failed to get GCfunc for target function.")?;

	if !gcfunc.is_fast_function() {
		anyhow::bail!("Function is not a fast-function.");
	}

	let original_ffid = gcfunc.header().ffid;
	let original_upvalues = gcfunc.header().nupvalues;

	let function_handle = RawHandle::from_stack(lua, state).context("Failed to create raw handle for function.")?;

	let trampoline = make_detour_trampoline(lua, function_handle.get_id(), std::ptr::null(), detour_handler)?;
	// Not a detour, but we can reuse the trampoline maker to create a fast-function trampoline

	// Push it as a closure to call
	unsafe {
		lua.push_closure(state, std::mem::transmute(trampoline.as_ptr()), original_upvalues as c_int);
	}

	let new_gcfunc = get_gcobj_mut::<GCfunc>(lj_state, -1).context("Failed to get GCfunc for copied function.")?;
	new_gcfunc.header_mut().ffid = original_ffid;
	new_gcfunc.header_mut().nupvalues = original_upvalues;

	// TODO: Handle garbage collection of the trampoline function?
	std::mem::forget(trampoline);

	Ok(RawLuaReturn(1))
}
