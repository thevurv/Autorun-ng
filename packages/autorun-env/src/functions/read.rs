use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn read(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<Option<String>> {
	let target_path = lua.check_string(state, 1);
	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let content = plugin.dir().read_to_string(target_path.to_string())?;
	Ok(Some(content))
}
