use anyhow::Context;
use autorun_lua::{DebugInfo, LuaApi};
use autorun_luajit::{Frame, LJState, push_tvalue};
use autorun_types::LuaState;

pub fn is_function_authorized(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	if !matches!(
		lua.type_id(state, 1),
		autorun_lua::LuaTypeId::Function | autorun_lua::LuaTypeId::Number
	) {
		anyhow::bail!("First argument must be a function or stack level.");
	}

	let frames = Frame::walk_stack(state as *mut LJState);
	dbg!(frames);
	if lua.type_id(state, 1) == autorun_lua::LuaTypeId::Number {
		// attempt to resolve the function at the given stack level
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };
		let stack_level = lua.to::<i32>(state, 1);
		lua.pop(state, 1); // remove the stack level argument

		if lua.get_stack(state, stack_level, &raw mut debug_info as _) == 0 {
			anyhow::bail!("Invalid stack level provided.");
		}

		let lj_state = state as *mut LJState;
		let frame_func = Frame::from_debug_ci(lj_state, debug_info.i_ci).get_func_tv();

		// copy the function to the top of the stack
		unsafe {
			let state = lj_state.as_mut().context("Failed to dereference LJState")?;
			push_tvalue(state, &*frame_func);
		}
	}

	Ok(env
		.is_function_authorized(lua, state, None)
		.context("Failed to check function authorization.")?)
}
