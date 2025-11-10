use std::ffi::CStr;

use crate::{LuaApi, LuaFunction, LuaState, LuaTypeId, LuaUserdata};

pub trait IntoLua {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState);
}

impl IntoLua for f64 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_number(state, self);
	}
}

impl IntoLua for &CStr {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_string(state, self.as_ptr());
	}
}

// NOTE: Do not implement FromLua for &CStr because strings arent guaranteed to be null terminated

impl IntoLua for bool {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_bool(state, self);
	}
}

impl IntoLua for i32 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_integer(state, self);
	}
}

impl IntoLua for Vec<u8> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for &[u8] {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for String {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl IntoLua for LuaFunction {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_function(state, self);
	}
}

impl IntoLua for &std::borrow::Cow<'_, str> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as _, self.len());
	}
}

impl IntoLua for &std::path::PathBuf {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push(state, &self.to_string_lossy());
	}
}

impl IntoLua for &str {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as _, self.len());
	}
}

impl<T: IntoLua> IntoLua for Option<T> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		match self {
			Some(val) => val.into_lua(lua, state),
			None => lua.push_nil(state),
		}
	}
}

impl<T: LuaUserdata> IntoLua for T {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.new_userdata(state, self);
	}
}
