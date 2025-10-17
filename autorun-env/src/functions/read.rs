use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn read(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<core::ffi::c_int> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(0);
	}

	let target_path = lua.check_string(state, 1);
	let current_dir = crate::global::get_current_path(lua, state).ok_or(anyhow::anyhow!(
		"What is wrong with you why did you delete Autorun.DIR. You will pay for this."
	))?;

	let content = current_dir.read(target_path.to_string())?;
	lua.push_lstring(state, content.as_ptr() as _, content.len());

	Ok(1)
}
