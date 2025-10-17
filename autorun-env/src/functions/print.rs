use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	if !crate::global::is_inside_env(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(());
	}

	let nargs = lua.get_top(state);
	let mut args = Vec::with_capacity(nargs as usize);
	for i in 1..=nargs {
		let arg = lua.to::<String>(state, i);
		args.push(arg);
	}

	let msg = args.join("\t");
	println!("[Lua] {msg}");

	Ok(())
}
