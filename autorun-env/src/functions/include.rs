use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn include(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(());
	}

	let target_path = lua.check_string(state, 1);
	let current_dir = crate::global::get_current_path(lua, state).ok_or(anyhow::anyhow!(
		"What is wrong with you why did you delete Autorun.DIR. You will pay for this."
	));

	Ok(())
}
