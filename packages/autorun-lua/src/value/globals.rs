use crate::{GLOBALS_INDEX, IntoLua};

#[derive(Debug, Clone, Copy)]
pub struct Globals;

impl IntoLua for Globals {
	fn into_lua(self, lua: &crate::LuaApi, state: *mut crate::LuaState) {
		lua.raw.pushvalue(state, GLOBALS_INDEX);
	}
}
