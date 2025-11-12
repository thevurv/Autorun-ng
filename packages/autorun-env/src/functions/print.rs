use autorun_lua::{LuaApi, LuaValue};
use autorun_types::LuaState;

pub fn print(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
	let plugin_name = env.get_active_plugin(lua, state).map(|p| p.config().plugin.name.as_str());

	let nargs = lua.raw.gettop(state);
	let mut args = Vec::with_capacity(nargs as usize);
	for i in 1..=nargs {
		match lua.raw.to::<LuaValue>(state, i) {
			LuaValue::Nil => {
				args.push(String::from("nil"));
			}

			LuaValue::LightUserdata(ptr) => {
				args.push(format!("lightuserdata: {:p}", ptr));
			}

			LuaValue::Userdata(ptr) => {
				args.push(format!("userdata: {:p}", ptr));
			}

			LuaValue::CFunction(func) => {
				args.push(format!("function: {:p}", func));
			}

			LuaValue::Boolean(val) => {
				args.push(String::from(if val { "true" } else { "false" }));
			}

			LuaValue::Function(_func) => {
				let func_str = lua.raw.topointer(state, i);
				args.push(format!("function: {:p}", func_str));
			}

			LuaValue::Table(_table) => {
				let func_str = lua.raw.topointer(state, i);
				args.push(format!("table: {:p}", func_str));
			}

			LuaValue::Number(val) => {
				args.push(val.to_string());
			}

			_ => {
				match lua.raw.tostring(state, i) {
					Some(s) => args.push(s.to_string()),
					None => args.push(String::from("<tostring failed>")),
				}
			}
		}
	}

	let msg = args.join(" ");
	println!("[{}] {msg}", plugin_name.unwrap_or("Lua"));

	Ok(())
}
