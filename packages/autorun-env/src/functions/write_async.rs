use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn write_async(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	let realm = crate::global::get_realm(state);
	let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

	if !env.is_active(lua, state) {
		autorun_log::warn!("Attempted to call 'writeAsync' outside of authorized environment");
		return Ok(());
	}

	let target_path = lua.check_string(state, 1);
	let content = lua.check_string(state, 2);

	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let data_dir = (*plugin.data_dir()).try_clone()?;
	let target_path = target_path.to_string();
	let content = content.to_string();

	std::thread::spawn(move || {
		if !data_dir.exists(&target_path)
			&& let Err(why) = data_dir.create(&target_path)
		{
			autorun_log::error!("Failed to create file '{target_path}' asynchronously: {why}");
			return;
		}

		if let Err(why) = data_dir.write(&target_path, content) {
			autorun_log::error!("Failed to write to file '{target_path}' asynchronously: {why}");
		}
	});

	Ok(())
}
