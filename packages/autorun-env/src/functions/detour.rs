mod handlers;
mod lua;
mod raw;
mod userdata;

use crate::functions::detour::handlers::{detour_handler, retour_handler};
use crate::functions::detour::lua::state::OriginalDetourState;
use crate::functions::detour::raw::{make_detour_trampoline, make_retour_lua_trampoline};
use crate::functions::detour::userdata::Detour;
use anyhow::Context;
use autorun_lua::{LuaApi, LuaCFunction, LuaTypeId, RawHandle, RawLuaReturn};
use autorun_luajit::bytecode::{BCWriter, Op};
use autorun_luajit::{BCIns, GCfunc, GCfuncL, LJState, get_gcobj, get_gcobj_mut, get_gcobj_ptr, index2adr};
use autorun_types::LuaState;
use retour::GenericDetour;
use std::ffi::c_int;
pub use userdata::{detour_disable, detour_enable, detour_get_original, detour_remove};

pub fn detour(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<Detour> {
	if !lua.raw.iscfunction(state, 1) {
		anyhow::bail!("First argument must be a C function to detour.");
	}

	let l = state as *mut LJState;
	let l_ref = unsafe { l.as_ref().context("Failed to dereference lua_State.")? };

	let gcfunc = get_gcobj::<GCfunc>(l_ref, 1).context("Failed to get GCfunc for target function.")?;

	if gcfunc.is_fast_function() {
		anyhow::bail!("Cannot detour a fast-function. Use copyFastFunction instead.");
	}

	let target_function: LuaCFunction = unsafe { std::mem::transmute(gcfunc.c.c) };
	if target_function as usize == 0 {
		anyhow::bail!("Target function pointer is null.");
	}

	if lua.raw.typeid(state, 2) != LuaTypeId::Function {
		anyhow::bail!("Second argument must be a function to use as detour.");
	}

	let detour_callback = RawHandle::from_stack(&lua.raw, state).context("Failed to create raw handle for detour callback.")?;
	let mut original_function_ptr = Box::new(0usize);

	// create the trampoline
	let detour_trampoline = make_detour_trampoline(
		lua,
		detour_callback.id(),
		original_function_ptr.as_ref() as *const usize,
		detour_handler,
	)?;

	let detour = unsafe {
		Box::new(GenericDetour::new(
			target_function,
			std::mem::transmute::<*const u8, LuaCFunction>(detour_trampoline.as_ptr()),
		)?)
	};

	unsafe {
		detour
			.enable()
			.map_err(|e| anyhow::anyhow!("Failed to enable detour: {}", e))?;
	}

	// create retour trampoline
	let retour_trampoline = make_retour_lua_trampoline(detour.as_ref() as *const GenericDetour<LuaCFunction>, retour_handler)?;

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
	if !lua.raw.iscfunction(state, 1) {
		anyhow::bail!("First argument must be a C function to copy.");
	}

	if lua.raw.typeid(state, 2) != LuaTypeId::Function {
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

	let function_handle = RawHandle::from_stack(&lua.raw, state).context("Failed to create raw handle for function.")?;

	let trampoline = make_detour_trampoline(lua, function_handle.id(), std::ptr::null(), detour_handler)?;
	// Not a detour, but we can reuse the trampoline maker to create a fast-function trampoline

	// Push it as a closure to call
	unsafe {
		lua.raw.pushcclosure(
			state,
			std::mem::transmute::<*const u8, LuaCFunction>(trampoline.as_ptr()),
			original_upvalues as c_int,
		);
	}

	let new_gcfunc = get_gcobj_mut::<GCfunc>(lj_state, -1).context("Failed to get GCfunc for copied function.")?;
	new_gcfunc.header_mut().ffid = original_ffid;
	new_gcfunc.header_mut().nupvalues = original_upvalues;

	// TODO: Handle garbage collection of the trampoline function?
	std::mem::forget(trampoline);

	Ok(RawLuaReturn(1))
}

pub fn detour_lua(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	if lua.raw.typeid(state, 1) != LuaTypeId::Function {
		anyhow::bail!("First argument must be a function.");
	}

	// get function
	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState.")? };
	let gcfunc = get_gcobj::<GCfunc>(lj_state, 1).context("Failed to get GCfunc for target function.")?;
	let gcfunc_ptr = get_gcobj_ptr::<GCfunc>(lj_state, 1).context("Failed to get GCfunc pointer for target function.")?;
	let gcfunc_l_ptr = unsafe { std::mem::transmute::<*mut GCfunc, *mut GCfuncL>(gcfunc_ptr) };

	let gcfunc_l = gcfunc.as_l().context("Target function must be a Lua function.")?;
	let proto = gcfunc_l.get_proto()?;
	let proto = unsafe { proto.as_mut().context("Failed to get proto for target function.")? };

	let replacement_tv = unsafe {
		index2adr(lj_state, 2)
			.context("Failed to get TValue for replacement upvalue.")?
			.read()
	};

	let detour_gcfunc = get_gcobj::<GCfunc>(lj_state, 2).context("Failed to get GCfunc for detour function.")?;
	let detour_gcfunc_l = detour_gcfunc.as_l().context("Detour function must be a Lua function.")?;
	let detour_proto = detour_gcfunc_l.get_proto()?;
	let detour_proto = unsafe { detour_proto.as_mut().context("Failed to get proto for detour function.")? };

	// Copy over debug information from detour to target
	detour_proto.chunkname = proto.chunkname;
	detour_proto.firstline = proto.firstline;
	detour_proto.lineinfo = proto.lineinfo;
	detour_proto.uvinfo = proto.uvinfo;
	detour_proto.varinfo = proto.varinfo;

	let gcfunc_l = gcfunc.as_l().context("Must be a Lua function.")?;
	autorun_log::debug!("Patching upvalue...");
	let mut original_detour_state = OriginalDetourState::new();
	lua::upvalue::replace(lj_state, gcfunc_l_ptr, 0, replacement_tv, &mut original_detour_state)?;
	lua::trampoline::overwrite_with_trampoline(gcfunc_l, &mut original_detour_state)?;

	let original_function_ptr = index2adr(lj_state, 1).context("Failed to get TValue for target function.")?;
	let original_function_ptr =
		unsafe { (*original_function_ptr).as_ptr::<GCfunc>() }.context("Failed to get GCfunc pointer.")?;

	lua::state::save_state(original_function_ptr, original_detour_state);

	Ok(RawLuaReturn(0))
}

pub fn restore_lua(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	if lua.raw.typeid(state, 1) != LuaTypeId::Function {
		anyhow::bail!("First argument must be a function.");
	}

	// get function
	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState.")? };
	let original_function_ptr = index2adr(lj_state, 1).context("Failed to get TValue for target function.")?;
	let original_function_ptr =
		unsafe { (*original_function_ptr).as_ptr::<GCfunc>() }.context("Failed to get GCfunc pointer.")?;

	lua::state::restore_func(original_function_ptr)?;

	Ok(RawLuaReturn(0))
}

pub fn clone_lua_function(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	if lua.raw.typeid(state, 1) != LuaTypeId::Function {
		anyhow::bail!("First argument must be a function.");
	}

	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState.")? };

	let gcfunc = get_gcobj_ptr::<GCfunc>(lj_state, 1).context("Failed to get GCfunc for target function.")?;
	let gcfunc_l = unsafe { std::mem::transmute::<*mut GCfunc, *mut GCfuncL>(gcfunc) };
	lua::clone(lj_state, gcfunc_l)?;
	Ok(RawLuaReturn(1))
}
