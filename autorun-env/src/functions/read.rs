use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn read(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<core::ffi::c_int> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(0);
	}

	let target_path = lua.check_string(state, 1);
	let plugin = crate::global::get_running_plugin(lua, state).ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro"))?;

	let content = plugin.dir().read(target_path.to_string())?;
	lua.push_lstring(state, content.as_ptr() as _, content.len());

	Ok(1)
}
