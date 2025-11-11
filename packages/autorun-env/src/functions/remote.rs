use autorun_lua::{IntoLua, LuaApi, LuaTypeId};
use autorun_types::{LuaState, Realm};

#[derive(Debug)]
enum RemoteValue {
	String(String),
	Number(f64),
	Boolean(bool),
	Nil,
}

impl IntoLua for RemoteValue {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		match self {
			RemoteValue::String(s) => {
				lua.push(state, s.as_str());
			}
			RemoteValue::Number(n) => {
				lua.push(state, n);
			}
			RemoteValue::Boolean(b) => {
				lua.push(state, b);
			}
			RemoteValue::Nil => {
				lua.raw.pushnil(state);
			}
		}
	}
}

fn serialize_value(
	lua: &LuaApi,
	state: *mut LuaState,
	_env: crate::EnvHandle,
	stack_idx: core::ffi::c_int,
) -> anyhow::Result<RemoteValue> {
	match lua.raw.typeid(state, -1) {
		LuaTypeId::LightUserdata | LuaTypeId::Function | LuaTypeId::Userdata | LuaTypeId::Thread => {
			Err(anyhow::anyhow!("Unsupported type for remote value"))
		}

		LuaTypeId::String => Ok(RemoteValue::String(lua.to::<String>(state, stack_idx))),
		LuaTypeId::Number => Ok(RemoteValue::Number(lua.to::<f64>(state, stack_idx))),
		LuaTypeId::Boolean => Ok(RemoteValue::Boolean(lua.to::<bool>(state, stack_idx))),
		LuaTypeId::None | LuaTypeId::Nil => Ok(RemoteValue::Nil),

		LuaTypeId::Table => Err(anyhow::anyhow!("Table serialization not implemented")),
	}
}

pub fn trigger_remote(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
	let event_name = lua.raw.checkstring(state, 1);
	let event_name = std::ffi::CString::new(event_name.as_bytes())?;
	let value = serialize_value(lua, state, env, 2)?;

	let opposite_realm = match env.realm() {
		Realm::Client => Realm::Menu,
		Realm::Menu => Realm::Client,
	};

	let opposite_state =
		autorun_interfaces::lua::get_state(opposite_realm)?.ok_or(anyhow::anyhow!("Opposing state does not exist"))?;

	let opposite_env = crate::global::get_realm_env(opposite_realm).ok_or(anyhow::anyhow!("Opposing env does not exist"))?;

	lua.push(opposite_state, value);
	opposite_env.run_remote_callbacks(lua, opposite_state, &event_name, 1)?;

	Ok(())
}
