use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn exists(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	let target_path = lua.check_string(state, 1);
	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let dir = plugin.dir();
	let target_path = target_path.to_string();

	Ok(dir.exists(target_path))
}
