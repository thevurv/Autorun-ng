use std::ffi::{CStr, CString};

use crate::{LuaApi, LuaFunction, LuaState};

pub trait IntoLua {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState);
}

pub trait FromLua: Sized {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self;
}

impl IntoLua for f64 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_number(state, self);
	}
}

impl<'a> FromLua for f64 {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.to_number(state, stack_idx)
	}
}

impl IntoLua for &CStr {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_string(state, self.as_ptr());
	}
}

// Commented because Strings aren't guaranteed to be nul terminated
// impl<'a> FromLua for &'a CStr {
// 	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
// 		let str = lua.to_lstring(state, stack_idx, std::ptr::null_mut());
// 		if str.is_null() {
// 			c"nil"
// 		} else {
// 			unsafe { CStr::from_ptr(str) }
// 		}
// 	}
// }

impl IntoLua for bool {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_bool(state, self);
	}
}

impl FromLua for bool {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.to_bool(state, stack_idx)
	}
}

impl IntoLua for () {
	fn into_lua(self, _lua: &LuaApi, _state: *mut LuaState) {
		// No value to push for unit type
	}
}

impl FromLua for () {
	fn from_lua(_lua: &LuaApi, _state: *mut LuaState, _stack_idx: i32) -> Self {
		()
	}
}

impl IntoLua for i32 {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_integer(state, self);
	}
}

impl FromLua for i32 {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.to_integer(state, stack_idx)
	}
}

impl IntoLua for &[u8] {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl FromLua for &[u8] {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let mut len = 0;
		let str = lua.to_lstring(state, stack_idx, &mut len);
		if str.is_null() {
			&[]
		} else {
			unsafe { std::slice::from_raw_parts(str as *const u8, len as _) }
		}
	}
}

impl IntoLua for String {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as *const i8, self.len());
	}
}

impl FromLua for String {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let bytes = lua.to::<&[u8]>(state, stack_idx);
		String::from_utf8_lossy(bytes).to_string()
	}
}

impl IntoLua for LuaFunction {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_function(state, self);
	}
}

impl FromLua for LuaFunction {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.to_function(state, stack_idx)
	}
}

impl IntoLua for &std::borrow::Cow<'_, str> {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.push_lstring(state, self.as_ptr() as _, self.len());
	}
}

impl FromLua for std::borrow::Cow<'_, str> {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let bytes = lua.to::<&[u8]>(state, stack_idx);
		String::from_utf8_lossy(bytes)
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
