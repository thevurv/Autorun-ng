use std::io::Write;

use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn append(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
	let target_path = lua.check_string(state, 1);
	let content = lua.check_string(state, 2);

	let plugin = env
		.get_active_plugin(lua, state)
		.ok_or(anyhow::anyhow!("dont delete autorun.plugin lil bro"))?;

	let data_dir = plugin.data_dir();
	let target_path = target_path.to_string();

	let mut f = data_dir.open_with(target_path, cap_std::fs::OpenOptions::new().append(true))?;
	f.write_all(content.as_bytes())?;

	Ok(())
}
