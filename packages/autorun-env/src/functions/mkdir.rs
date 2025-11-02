use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn mkdir(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	let target_path = lua.check_string(state, 1);
	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let data_dir = plugin.data_dir();
	let target_path = target_path.to_string();

	if data_dir.is_file(&target_path) {
		anyhow::bail!("Cannot create directory '{target_path}': A file with the same name already exists");
	}

	if !data_dir.exists(&target_path) {
		data_dir.create_dir_all(&target_path)?;

		return Ok(true);
	}

	Ok(false)
}
