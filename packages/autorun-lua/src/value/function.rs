use crate::{FromLua, IntoLua, LuaApi, LuaState, LuaTypeId, RawHandle};

#[derive(Debug, Clone, Copy)]
pub struct LuaFunction {
	handle: RawHandle,
}

impl IntoLua for &LuaFunction {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		self.handle.push(lua, state);
	}
}

impl FromLua for LuaFunction {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, index: i32) -> Self {
		assert_eq!(
			lua.raw.typeid(state, index),
			LuaTypeId::Function,
			"Value was not a LuaFunction"
		);

		lua.raw.pushvalue(state, index);
		let handle = RawHandle::from_stack(lua, state).expect("Failed to allocate registry value");
		LuaFunction { handle }
	}
}
