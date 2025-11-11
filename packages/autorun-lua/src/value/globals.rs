use crate::{GLOBALS_INDEX, IntoLua, RawLuaApi};

#[derive(Debug, Clone, Copy)]
pub struct Globals;

impl IntoLua for Globals {
	fn into_lua(self, lua: &RawLuaApi, state: *mut crate::LuaState) {
		lua.pushvalue(state, GLOBALS_INDEX);
	}
}
