use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	let msg = lua.check_string(state, 1);
	println!("[Lua] {msg}");

	Ok(())
}
