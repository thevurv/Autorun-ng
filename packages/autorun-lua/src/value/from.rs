use crate::{LuaApi, LuaCFunction, LuaState, LuaTypeId, LuaUserdata};

pub trait FromLua: Sized {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self;
}

impl FromLua for f64 {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.raw.tonumber(state, stack_idx)
	}
}

impl FromLua for bool {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.raw.toboolean(state, stack_idx)
	}
}

impl FromLua for i32 {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.raw.tointeger(state, stack_idx)
	}
}

impl FromLua for &[u8] {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let mut len = 0;
		let str = lua.raw.tolstring(state, stack_idx, &mut len);
		if str.is_null() {
			&[]
		} else {
			unsafe { std::slice::from_raw_parts(str as *const u8, len as _) }
		}
	}
}

impl FromLua for String {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let bytes = lua.to::<&[u8]>(state, stack_idx);
		String::from_utf8_lossy(bytes).to_string()
	}
}

// Some seemingly "C" functions are actually LuaJIT fast-functions and return NULL despite being valid C functions
impl FromLua for Option<LuaCFunction> {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.raw.tocfunction(state, stack_idx)
	}
}

impl<T: FromLua> FromLua for Option<T> {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		match lua.raw.typeid(state, stack_idx) {
			LuaTypeId::None | LuaTypeId::Nil => None,
			_ => Some(T::from_lua(lua, state, stack_idx)),
		}
	}
}

impl FromLua for std::borrow::Cow<'_, str> {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		let bytes = lua.to::<&[u8]>(state, stack_idx);
		String::from_utf8_lossy(bytes)
	}
}

impl<T: LuaUserdata> FromLua for *mut T {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.raw.touserdata(state, stack_idx) as _
	}
}
