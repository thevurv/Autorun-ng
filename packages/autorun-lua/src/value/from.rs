use crate::{LuaCFunction, LuaError, LuaResult, LuaState, LuaTypeId, LuaUserdata, LuaValue, RawLuaApi};

pub trait FromLua: Sized {
	fn from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> Self;
}

pub trait TryFromLua: Sized {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self>;
}

impl<T: FromLua> TryFromLua for T {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		Ok(T::from_lua(lua, state, stack_idx))
	}
}

impl TryFromLua for f64 {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to(state, stack_idx) {
			LuaValue::Number(n) => Ok(n),
			other => Err(LuaError::TypeMismatch {
				expected: LuaTypeId::Number,
				found: other.typeid(),
			}),
		}
	}
}

impl TryFromLua for bool {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to(state, stack_idx) {
			LuaValue::Boolean(b) => Ok(b),
			other => Err(LuaError::TypeMismatch {
				expected: LuaTypeId::Boolean,
				found: other.typeid(),
			}),
		}
	}
}

impl TryFromLua for i32 {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to(state, stack_idx) {
			LuaValue::Number(i) => Ok(i as _),
			other => Err(LuaError::mismatch(LuaTypeId::Number, other.typeid())),
		}
	}
}

impl TryFromLua for &[u8] {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to::<LuaValue>(state, stack_idx) {
			LuaValue::String(s) => Ok(s),
			_ => Err(LuaError::mismatch(LuaTypeId::String, lua.typeid(state, stack_idx))),
		}
	}
}

impl TryFromLua for String {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to::<LuaValue>(state, stack_idx) {
			LuaValue::String(s) => Ok(String::from_utf8_lossy(s).into_owned()),
			_ => Err(LuaError::mismatch(LuaTypeId::String, lua.typeid(state, stack_idx))),
		}
	}
}

// Some seemingly "C" functions are actually LuaJIT fast-functions and return NULL despite being valid C functions
impl FromLua for Option<LuaCFunction> {
	fn from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> Self {
		lua.tocfunction(state, stack_idx)
	}
}

impl<T: TryFromLua> TryFromLua for Option<T> {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.typeid(state, stack_idx) {
			LuaTypeId::None | LuaTypeId::Nil => Ok(None),
			_ => Ok(Some(T::try_from_lua(lua, state, stack_idx)?)),
		}
	}
}

impl TryFromLua for std::borrow::Cow<'_, str> {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		let bytes = lua.try_to::<&[u8]>(state, stack_idx)?;
		Ok(String::from_utf8_lossy(bytes))
	}
}

impl<T: LuaUserdata> TryFromLua for *mut T {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, stack_idx: i32) -> LuaResult<Self> {
		match lua.to::<LuaValue>(state, stack_idx) {
			LuaValue::LightUserdata(u) | LuaValue::Userdata(u) => Ok(u.cast()),
			_ => Err(LuaError::mismatch(LuaTypeId::Userdata, lua.typeid(state, stack_idx))),
		}
	}
}
