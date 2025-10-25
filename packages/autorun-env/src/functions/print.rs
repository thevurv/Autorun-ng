use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<()> {
	let realm = crate::global::get_realm(state);
	let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

	if !env.is_active(lua, state) {
		autorun_log::warn!("Attempted to call 'print' outside of authorized environment");
		return Ok(());
	}

	let nargs = lua.get_top(state);
	let mut args = Vec::with_capacity(nargs as usize);
	for i in 1..=nargs {
		match lua.type_id(state, i) {
			autorun_lua::LuaTypeId::Nil => {
				args.push(String::from("nil"));
			}

			autorun_lua::LuaTypeId::LightUserdata => {
				let ptr = lua.to_userdata(state, i);
				args.push(format!("lightuserdata: {:p}", ptr));
			}

			_ => {
				let arg = lua.to::<String>(state, i);
				args.push(arg);
			}
		}
	}

	let msg = args.join("\t");
	println!("[Lua] {msg}");

	Ok(())
}
