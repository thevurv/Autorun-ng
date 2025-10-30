use anyhow::Context;
use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn is_function_authorized(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	// TODO: Support stack levels
	if lua.type_id(state, 1) != autorun_lua::LuaTypeId::Function {
		anyhow::bail!("First argument must be a function.");
	}

	Ok(env
		.is_function_authorized(lua, state, None)
		.context("Failed to check function authorization.")?)
}
