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

pub fn detour(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<Detour> {
	let target_function = lua
		.to_function(state, 1)
		.context("Failed to get target function from stack.")?;

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

pub fn get_function_ffid(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<i32> {
	if lua.is_c_function(state, 1) == 0 {
		anyhow::bail!("First argument must be a C function.");
	}

	unsafe {
		let L = state as *mut lua_State;
		let L_ref = L.as_ref().context("Failed to dereference lua_State pointer.")?;

		let tv = index2adr(L_ref, 1).context("Failed to get TValue for function at index 1.")?;
		let gcfunc = (*tv).as_ref::<GCfunc>();
		let ffid = gcfunc.c.header.ffid;

		Ok(ffid as i32)
	}
}
