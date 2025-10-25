use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn mkdir(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<Option<bool>> {
	let realm = crate::global::get_realm(state);
	let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

	if !env.is_active(lua, state) {
		autorun_log::warn!("Attempted to call 'mkdir' outside of authorized environment");
		return Ok(None);
	}

	let target_path = lua.check_string(state, 1);
	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let data_dir = plugin.data_dir();
	let target_path = target_path.to_string();

	if data_dir.is_file(&target_path) {
		return Err(anyhow::anyhow!(
			"Cannot create directory '{}': A file with the same name already exists",
			target_path
		));
	}

	if !data_dir.exists(&target_path) {
		data_dir.create_dir_all(&target_path)?;

		return Ok(Some(true));
	}

	Ok(Some(false))
}
