use crate::{LuaCFunction, LuaState, LuaTypeId, RawHandle, RawLuaApi};
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

mod function;
pub use function::*;

#[derive(Debug, Clone)]
pub enum LuaValue<'a> {
	Nil,
	Boolean(bool),
	Number(f64),
	String(&'a [u8]),
	Table(LuaTable),
	Function(LuaFunction),
	CFunction(LuaCFunction),
	LightUserdata(*mut c_void),
	Userdata(*mut c_void),
}

impl LuaValue<'_> {
	pub fn typeid(&self) -> LuaTypeId {
		match self {
			LuaValue::Nil => LuaTypeId::Nil,
			LuaValue::Boolean(_) => LuaTypeId::Boolean,
			LuaValue::Number(_) => LuaTypeId::Number,
			LuaValue::String(_) => LuaTypeId::String,
			LuaValue::Table(_) => LuaTypeId::Table,
			LuaValue::Function(_) => LuaTypeId::Function,
			LuaValue::CFunction(_) => LuaTypeId::Function,
			LuaValue::LightUserdata(_) => LuaTypeId::LightUserdata,
			LuaValue::Userdata(_) => LuaTypeId::Userdata,
		}
	}
}

impl IntoLua for LuaValue<'_> {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		match self {
			LuaValue::Nil => lua.pushnil(state),
			LuaValue::Boolean(b) => lua.pushboolean(state, b),
			LuaValue::Number(n) => lua.pushnumber(state, n),
			LuaValue::String(s) => lua.pushlstring(state, s.as_ptr() as *const i8, s.len()),
			LuaValue::Table(t) => t.into_lua(lua, state),
			LuaValue::Function(f) => f.into_lua(lua, state),
			LuaValue::CFunction(f) => lua.pushcfunction(state, f),
			LuaValue::Userdata(u) => lua.pushlightuserdata(state, u),
			LuaValue::LightUserdata(u) => lua.pushlightuserdata(state, u),
		}
	}
}

impl FromLua for LuaValue<'_> {
	fn from_lua(lua: &RawLuaApi, state: *mut LuaState, index: i32) -> Self {
		let lua_type = lua.typeid(state, index);
		match lua_type {
			LuaTypeId::Boolean => LuaValue::Boolean(lua.toboolean(state, index)),
			LuaTypeId::Number => LuaValue::Number(lua.tonumber(state, index)),
			LuaTypeId::String => {
				let mut len = 0;
				let str = lua.tolstring(state, index, &mut len);
				if str.is_null() {
					LuaValue::String(&[])
				} else {
					LuaValue::String(unsafe { std::slice::from_raw_parts(str as *const u8, len as _) })
				}
			}

			LuaTypeId::Table => {
				lua.pushvalue(state, index);
				let handle = RawHandle::from_stack(lua, state).expect("Failed to allocate registry value");
				LuaValue::Table(LuaTable::from_raw(handle))
			}

			LuaTypeId::Function => {
				let func = lua.tocfunction(state, index);
				if let Some(func) = func {
					LuaValue::CFunction(func)
				} else {
					lua.pushvalue(state, index);
					let handle = RawHandle::from_stack(lua, state).expect("Failed to allocate registry value");
					LuaValue::Function(LuaFunction::from_raw(handle))
				}
			}

			LuaTypeId::LightUserdata => LuaValue::LightUserdata(lua.touserdata(state, index)),
			LuaTypeId::Userdata => LuaValue::Userdata(lua.touserdata(state, index)),

			_ => LuaValue::Nil,
		}
	}
}
