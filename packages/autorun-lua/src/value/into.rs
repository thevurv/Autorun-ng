use std::ffi::CStr;

use crate::{LuaApi, LuaCFunction, LuaState, LuaUserdata};

pub trait IntoLua {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState);
}

impl IntoLua for f64 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushnumber(state, self);
	}
}

impl IntoLua for &CStr {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushstring(state, self.as_ptr());
	}
}

// NOTE: Do not implement FromLua for &CStr because strings arent guaranteed to be null terminated

impl IntoLua for bool {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushboolean(state, self);
	}
}

impl IntoLua for i32 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushinteger(state, self);
	}
}

impl IntoLua for Vec<u8> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushlstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for &[u8] {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushlstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for String {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushlstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for LuaCFunction {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushcfunction(state, self);
	}
}

impl IntoLua for &std::borrow::Cow<'_, str> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushlstring(state, self.as_ptr() as _, self.len());
	}
}

impl IntoLua for &std::path::PathBuf {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push(state, &self.to_string_lossy());
	}
}

impl IntoLua for &str {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.pushlstring(state, self.as_ptr() as _, self.len());
	}
}

impl<T: IntoLua> IntoLua for Option<T> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		match self {
			Some(val) => val.into_lua(lua, state),
			None => lua.raw.pushnil(state),
		}
	}
}

impl<T: LuaUserdata> IntoLua for T {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.raw.newuserdata(state, self);
	}
}

// todo: implement IntoLua for () when we have specialization
// currently it conflicts with LuaReturn
