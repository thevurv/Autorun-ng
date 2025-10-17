use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	let nargs = lua.get_top(state);

	let mut args = Vec::with_capacity(nargs as usize);
	for i in 1..=nargs {
		let arg = lua.to::<&core::ffi::CStr>(state, i);
		args.push(arg.to_string_lossy());
	}

	let msg = args.join("\t");
	println!("[Lua] {msg}");

	Ok(())
}
