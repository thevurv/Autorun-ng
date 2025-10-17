use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn write_async(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<core::ffi::c_int> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(0);
	}

	let target_path = lua.check_string(state, 1);
	let content = lua.check_string(state, 2);

	let plugin = crate::global::get_running_plugin(lua, state).ok_or(anyhow::anyhow!(
		"What is wrong with you why did you delete Autorun.PLUGIN. You will pay for this."
	))?;

	let data_dir = (*plugin.data_dir()).try_clone()?;
	let target_path = target_path.to_string();
	let content = content.to_string();

	std::thread::spawn(move || {
		if !data_dir.exists(&target_path) {
			if let Err(why) = data_dir.create(&target_path) {
				autorun_log::error!("Failed to create file '{target_path}' asynchronously: {why}");
				return;
			}
		}

		if let Err(why) = data_dir.write(&target_path, content) {
			autorun_log::error!("Failed to write to file '{target_path}' asynchronously: {why}");
		}
	});

	Ok(0)
}
