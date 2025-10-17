use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn mkdir(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<core::ffi::c_int> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(0);
	}

	let target_path = lua.check_string(state, 1);
	let plugin = crate::global::get_running_plugin(lua, state).ok_or(anyhow::anyhow!(
		"What is wrong with you why did you delete Autorun.PLUGIN. You will pay for this."
	))?;

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

		lua.push(state, true);
		return Ok(1);
	}

	lua.push(state, false);
	Ok(1)
}
