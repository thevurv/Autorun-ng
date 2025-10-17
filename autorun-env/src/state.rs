use autorun_lua::LuaApi;
use autorun_types::LuaState;

#[repr(transparent)]
pub struct State {}

impl State {
	pub fn create(lua: &LuaApi) -> Self {
		Self {}
	}
}
