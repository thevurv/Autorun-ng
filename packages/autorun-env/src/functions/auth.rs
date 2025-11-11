use anyhow::Context;
use autorun_lua::{DebugInfo, LuaApi};
use autorun_luajit::{Frame, GCProto, LJ_TPROTO, LJState, get_gcobj, index2adr, push_tvalue};
use autorun_types::LuaState;

pub fn is_function_authorized(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	if !matches!(
		lua.raw.typeid(state, 1),
		autorun_lua::LuaTypeId::Function | autorun_lua::LuaTypeId::Number
	) {
		anyhow::bail!("First argument must be a function or stack level.");
	}

	if lua.raw.typeid(state, 1) == autorun_lua::LuaTypeId::Number {
		// attempt to resolve the function at the given stack level
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };
		let stack_level = lua.to::<i32>(state, 1);
		lua.raw.pop(state, 1); // remove the stack level argument

		if lua.raw.getstack(state, stack_level, &raw mut debug_info as _) == 0 {
			anyhow::bail!("Invalid stack level provided.");
		}

		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_mut() }.context("Failed to dereference LJState")?;
		let frame_func = Frame::from_debug_ci(lj_state, debug_info.i_ci).get_func_tv();

		// copy the function to the top of the stack
		unsafe {
			push_tvalue(lj_state, &*frame_func);
		}
	}

	env.is_function_authorized(lua, state, None)
		.context("Failed to check function authorization.")
}

pub fn is_proto_authorized(_lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	// Protos dont play nice with the usual public API types, so we just have to do it manually
	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };
	let proto_tv = index2adr(lj_state, 1).context("Failed to get TValue for given index.")?;
	let proto_tv = unsafe { &*proto_tv };

	// TODO: When the stack spoof PR is merged, replace this with the new type check helper
	if proto_tv.itype() != LJ_TPROTO {
		anyhow::bail!("First argument must be a proto.");
	}

	let proto_gc = get_gcobj::<GCProto>(lj_state, 1).context("Failed to get GCProto for given index.")?;
	let proto_chunk_name = proto_gc.chunk_name_str().context("Failed to get chunk name from proto.")?;
	let proto_chunk_name_cstr =
		std::ffi::CString::new(proto_chunk_name.clone()).context("Failed to convert chunk name to CString.")?;

	Ok(env.is_chunk_name_authorized(&proto_chunk_name_cstr))
}
