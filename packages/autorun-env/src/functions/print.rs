use autorun_lua::LuaApi;
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<()> {
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

			autorun_lua::LuaTypeId::Userdata => {
				let ptr = lua.to_userdata(state, i);
				args.push(format!("userdata: {:p}", ptr));
			}

			autorun_lua::LuaTypeId::Function => {
				let ptr = lua.to_function(state, i);
				args.push(format!("function: {:p}", ptr));
			}

			autorun_lua::LuaTypeId::Thread => {
				let ptr = lua.to_thread(state, i);
				args.push(format!("thread: {:p}", ptr));
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
