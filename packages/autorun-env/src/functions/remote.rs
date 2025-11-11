use autorun_lua::{IntoLua, LuaApi, LuaValue, RawLuaApi};
use autorun_types::{LuaState, Realm};

#[derive(Debug)]
enum RemoteValue<'a> {
	String(&'a [u8]),
	Number(f64),
	Boolean(bool),
	Nil,
}

impl IntoLua for RemoteValue<'_> {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		match self {
			RemoteValue::String(s) => lua.push(state, s),
			RemoteValue::Number(n) => lua.push(state, n),
			RemoteValue::Boolean(b) => lua.push(state, b),
			RemoteValue::Nil => lua.pushnil(state),
		}
	}
}

fn serialize_value(
	lua: &LuaApi,
	state: *mut LuaState,
	_env: crate::EnvHandle,
	stack_idx: core::ffi::c_int,
) -> anyhow::Result<RemoteValue> {
	match lua.raw.to(state, stack_idx) {
		LuaValue::String(s) => Ok(RemoteValue::String(s)),
		LuaValue::Number(n) => Ok(RemoteValue::Number(n)),
		LuaValue::Boolean(b) => Ok(RemoteValue::Boolean(b)),
		LuaValue::Nil => Ok(RemoteValue::Nil),
		LuaValue::Table(_) => Err(anyhow::anyhow!("Table serialization not implemented")),
		_ => Err(anyhow::anyhow!("Unsupported type for remote value")),
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

	opposite_env.run_remote_callbacks(lua, opposite_state, (event_name.as_c_str(), value))?;

	Ok(())
}
