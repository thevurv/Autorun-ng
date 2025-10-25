use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn write(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
	let target_path = lua.check_string(state, 1);
	let content = lua.check_string(state, 2);

	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro"))?;

	let data_dir = plugin.data_dir();
	let target_path = target_path.to_string();

	if !data_dir.exists(&target_path) {
		data_dir.create(&target_path)?;
	}

	data_dir.write(target_path, content.to_string())?;

	Ok(())
}
