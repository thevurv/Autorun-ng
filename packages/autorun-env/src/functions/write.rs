use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn write(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	let realm = crate::global::get_realm(state);
	let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

	if !env.is_active(lua, state) {
		autorun_log::warn!("Attempted to call 'write' outside of authorized environment");
		return Ok(());
	}

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
