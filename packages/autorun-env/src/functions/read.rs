use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn read(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<Option<String>> {
	let realm = crate::global::get_realm(state);
	let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

	if !env.is_active(lua, state) {
		autorun_log::warn!("Attempted to call 'read' outside of authorized environment");
		return Ok(None);
	}

	let target_path = lua.check_string(state, 1);
	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro."))?;

	let content = plugin.dir().read_to_string(target_path.to_string())?;
	Ok(Some(content))
}
