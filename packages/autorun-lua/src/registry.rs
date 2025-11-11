use crate::{IntoLua, LuaApi, LuaResult, LuaState, REGISTRY_INDEX, RawLuaApi};

/// A handle to a value in the lua registry.
/// Note this does not have any reference counting, hence can be cloned.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RawHandle(i32);

impl RawHandle {
	/// Pops the value at the top of the stack and stores it in the registry.
	pub fn from_stack(lua: &RawLuaApi, state: *mut LuaState) -> Option<Self> {
		if lua.gettop(state) < 1 {
			return None;
		}

		lua.reference(state).map(Self)
	}

	pub fn from_id(id: i32) -> Self {
		Self(id)
	}

	pub fn free(self, lua: &LuaApi, state: *mut LuaState) -> LuaResult<()> {
		lua.raw.unreference(state, self.0)
	}

	pub fn id(&self) -> i32 {
		self.0
	}
}

impl IntoLua for &RawHandle {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		lua.rawgeti(state, REGISTRY_INDEX, self.0);
	}
}
