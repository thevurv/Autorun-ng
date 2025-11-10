use crate::IntoLua;

pub struct Globals;

impl IntoLua for Globals {
	fn into_lua(self, lua: &crate::LuaApi, state: *mut crate::LuaState) {
		lua.push_globals(state);
	}
}
