mod handlers;
mod raw;
mod userdata;

use crate::functions::detour::handlers::{detour_handler, retour_handler};
use crate::functions::detour::raw::{make_detour_trampoline, make_retour_lua_trampoline};
use crate::functions::detour::userdata::Detour;
use anyhow::Context;
use autorun_lua::{IntoLua, LuaApi, LuaFunction, LuaTypeId, RawHandle};
use autorun_luajit::{GCfunc, index2adr, lua_State};
use autorun_types::LuaState;
use retour::GenericDetour;
pub use userdata::{detour_disable, detour_enable, detour_get_original, detour_remove};

fn get_gcfunc(state: *mut LuaState, index: i32) -> anyhow::Result<GCfunc> {
	unsafe {
		let L = state as *mut lua_State;
		let L_ref = L.as_ref().context("Failed to dereference lua_State pointer.")?;

		let tv = index2adr(L_ref, index).context("Failed to get TValue for function at given index.")?;
		let gcfunc = (*tv).as_ref::<GCfunc>();
		Ok(*gcfunc) // copying the GCfunc is fine as we don't intend to modify it
	}
}

fn get_gcfunc_mut(state: *mut LuaState, index: i32) -> anyhow::Result<*mut GCfunc> {
	unsafe {
		let L = state as *mut lua_State;
		let L_ref = L.as_ref().context("Failed to dereference lua_State pointer.")?;

		let tv = index2adr(L_ref, index).context("Failed to get TValue for function at given index.")?;
		let gcfunc = (*tv).as_mut::<GCfunc>();
		Ok(gcfunc)
	}
}

pub fn detour(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<Detour> {
	if lua.is_c_function(state, 1) == 0 {
		anyhow::bail!("First argument must be a C function to detour.");
	}

	let gcfunc = get_gcfunc(state, 1).context("Failed to get GCfunc for target function.")?;

	// For fast-functions, this will be a pointer to the fast-function handler, which will detour as expected
	// in most cases.
	let target_function: LuaFunction = unsafe { std::mem::transmute(gcfunc.c.c) };
	if target_function as usize == 0 {
		anyhow::bail!("Target function pointer is null.");
	}

	dbg!(target_function);
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
		detour_callback,
		detour_trampoline,
		retour_trampoline,
		original_function_ptr,
	})
}

pub fn copy_fast_function(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<LuaFunction> {
	if lua.is_c_function(state, 1) == 0 {
		anyhow::bail!("First argument must be a C function to copy.");
	}

	if lua.type_id(state, 2) != LuaTypeId::Function {
		anyhow::bail!("Second argument must be a function to use.");
	}

	let gcfunc = get_gcfunc(state, 1).context("Failed to get GCfunc for target function.")?;

	if !gcfunc.is_fast_function() {
		anyhow::bail!("Function is not a fast-function.");
	}

	let original_ffid = gcfunc.as_c().header.ffid;
	let function_handle = RawHandle::from_stack(lua, state).context("Failed to create raw handle for function.")?;

	let trampoline = make_detour_trampoline(lua, function_handle.get_id(), std::ptr::null(), detour_handler)?;
	// Not a detour, but we can reuse the trampoline maker to create a fast-function trampoline

	// Push it as a closure to call
	unsafe {
		lua.push_function(state, std::mem::transmute(trampoline.as_ptr()));
	}

	let new_gcfunc = get_gcfunc_mut(state, -1).context("Failed to get GCfunc for copied function.")?;
	unsafe {
		(*new_gcfunc).as_c_mut().header.ffid = original_ffid;
	}

	Ok(unsafe { std::mem::transmute(*new_gcfunc) })
}
