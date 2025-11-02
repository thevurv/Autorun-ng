use core::ffi::{c_char, c_uint};

#[derive(Debug)]
pub enum HookEventOutcome {
	Replace(*const c_char, c_uint),
	Skip,
	Continue,
}

/// Function that triggers all plugins hook (each file) scripts.
pub fn run(state: *mut autorun_types::LuaState, buffer: &[u8], name: &[u8], mode: &[u8]) -> anyhow::Result<HookEventOutcome> {
	let lua = autorun_lua::get_api()?;

	let realm = autorun_env::global::get_realm(state);
	let env = autorun_env::global::get_realm_env(realm).expect("env should exist here");

	lua.push(state, name);
	lua.push(state, buffer);
	lua.push(state, mode);
	let n_returns = env.trigger(lua, state, c"loadbuffer", 3)?;

	autorun_log::info!("Hi {n_returns} {}", lua.get_top(state));
	for i in 1..=lua.get_top(state) {
		autorun_log::info!("Return {i}: type {:?}", lua.type_id(state, -n_returns + (i - 1)));
		let str = lua.to::<String>(state, -n_returns + (i - 1));
		autorun_log::info!("Return {i}: value {:?}", str);
	}

	match n_returns {
		0 => (),
		1 => match lua.type_id(state, -1) {
			autorun_lua::LuaTypeId::Nil | autorun_lua::LuaTypeId::None => {
				lua.pop(state, 1);
			}

			autorun_lua::LuaTypeId::String => {
				// Get the raw pointer from lua so we don't have to manage the memory ourselves
				let mut len = 0;
				let ptr = lua.to_lstring(state, -1, &mut len);
				let str = lua.check_string(state, -1);
				lua.pop(state, 1);
				println!("got str {str}");

				return Ok(HookEventOutcome::Replace(ptr, len));
			}

			autorun_lua::LuaTypeId::Boolean => {
				let should_skip = lua.to_bool(state, -1);
				lua.pop(state, 1);

				if should_skip {
					return Ok(HookEventOutcome::Skip);
				}
			}

			_ => {
				lua.pop(state, 1);
				anyhow::bail!("loadbuffer event returned invalid type");
			}
		},

		// This actually shouldn't be possible anymore as trigger only returns a single value
		_ => {
			anyhow::bail!("loadbuffer event returned too many values: {}", n_returns);
		}
	}

	Ok(HookEventOutcome::Continue)
}
