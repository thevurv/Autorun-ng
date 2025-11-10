use crate::{LuaApi, LuaCFunction, LuaState, LuaTypeId};
use core::ffi::c_void;

mod from;
pub use from::*;

mod into;
pub use into::*;

mod table;
pub use table::*;

mod globals;
pub use globals::*;

mod userdata;
pub use userdata::*;

#[derive(Debug, Clone)]
pub enum LuaValue {
	Nil,
	Boolean(bool),
	Number(f64),
	String(String),
	Table(LuaTable),
	CFunction(LuaCFunction),
	LightUserdata(*mut c_void),
	Userdata(*mut c_void),
}

impl IntoLua for LuaValue {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		match self {
			LuaValue::Nil => lua.raw.pushnil(state),
			LuaValue::Boolean(b) => lua.raw.pushboolean(state, b),
			LuaValue::Number(n) => lua.raw.pushnumber(state, n),
			LuaValue::String(s) => lua.raw.pushlstring(state, s.as_ptr() as *const i8, s.len()),
			LuaValue::Table(t) => t.into_lua(lua, state),
			LuaValue::CFunction(f) => lua.raw.pushcfunction(state, f),
			LuaValue::Userdata(u) => lua.raw.pushlightuserdata(state, u),
			LuaValue::LightUserdata(u) => lua.raw.pushlightuserdata(state, u),
		}
	}
}

impl FromLua for LuaValue {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, index: i32) -> Self {
		let lua_type = lua.raw.typeid(state, index);
		match lua_type {
			LuaTypeId::Boolean => LuaValue::Boolean(lua.to::<bool>(state, index)),
			LuaTypeId::Number => LuaValue::Number(lua.to::<f64>(state, index)),
			LuaTypeId::String => LuaValue::String(lua.to::<String>(state, index)),
			LuaTypeId::Table => LuaValue::Table(lua.to::<LuaTable>(state, index)),
			LuaTypeId::Function => {
				let func = lua.to::<Option<LuaCFunction>>(state, index);
				if let Some(func) = func {
					LuaValue::CFunction(func)
				} else {
					LuaValue::Nil
				}
			}
			LuaTypeId::LightUserdata => LuaValue::LightUserdata(lua.raw.touserdata(state, index)),
			LuaTypeId::Userdata => LuaValue::Userdata(lua.raw.touserdata(state, index)),
			_ => LuaValue::Nil,
		}
	}
}
