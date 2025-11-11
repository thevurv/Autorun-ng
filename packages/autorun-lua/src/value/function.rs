use crate::{IntoLua, LuaError, LuaResult, LuaState, LuaTypeId, LuaValue, RawHandle, RawLuaApi, TryFromLua};

#[derive(Debug, Clone, Copy)]
pub struct LuaFunction {
	handle: RawHandle,
}

impl LuaFunction {
	pub(crate) fn from_raw(handle: RawHandle) -> Self {
		Self { handle }
	}
}

impl IntoLua for &LuaFunction {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		lua.push(state, &self.handle);
	}
}

impl TryFromLua for LuaFunction {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, index: i32) -> LuaResult<Self> {
		match lua.try_to(state, index)? {
			LuaValue::Function(f) => Ok(f),
			other => Err(LuaError::mismatch(LuaTypeId::Function, other.typeid())),
		}
	}
}
