/// Function that triggers all plugins hook (each file) scripts.
pub fn run(state: *mut autorun_types::LuaState, buffer: &[u8], name: &[u8], mode: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
	let lua = autorun_lua::get_api()?;

	let realm = autorun_env::global::get_realm(state);
	let env = autorun_env::global::get_realm_env(realm).expect("env should exist here");

	lua.raw.push(state, name);
	lua.raw.push(state, buffer);
	lua.raw.push(state, mode);
	env.trigger(lua, state, c"loadbuffer", 3)?;

	// let n_returns = lua.get_top(state);
	// match n_returns {
	// 	0 => (),
	// 	1 => match lua.type_id(state, -1) {
	// 		autorun_lua::LuaTypeId::Nil => {
	// 			lua.pop(state, 1);
	// 		}

	// 		autorun_lua::LuaTypeId::String => {
	// 			let str = lua.to::<&[u8]>(state, -1);
	// 			let str = std::str::from_utf8(str).unwrap_or("<invalid utf8>");
	// 			lua.pop(state, 1);

	// 			// Replace buffer
	// 			anyhow::bail!("Replacing buffer in loadbuffer event"));
	// 		}

	// 		autorun_lua::LuaTypeId::Boolean => {
	// 			let bool = lua.to_bool(state, -1);
	// 			lua.pop(state, 1);

	// 			if bool {
	// 				anyhow::bail!("Blocking loadbuffer in loadbuffer event"));
	// 			}
	// 		}

	// 		_ => {
	// 			lua.pop(state, 1);
	// 			anyhow::bail!("loadbuffer event returned invalid type"));
	// 		}
	// 	},

	// 	_ => {
	// 		anyhow::bail!("loadbuffer event returned too many values: {}", n_returns));
	// 	}
	// }

	Ok(None)
}
