use crate::LuaState;

/// A handle to a value in the lua registry.
/// Note this does not have any reference counting, hence can be cloned.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RawHandle(i32);

impl RawHandle {
	/// Pops the value at the top of the stack and stores it in the registry.
	pub fn from_stack(lua: &crate::LuaApi, state: *mut LuaState) -> Option<Self> {
		if lua.get_top(state) < 1 {
			return None;
		}

		lua.reference(state).map(Self)
	}

	pub fn push(&self, lua: &crate::LuaApi, state: *mut LuaState) {
		lua.get_registry(state, self.0);
	}

	pub fn free(self, lua: &crate::LuaApi, state: *mut LuaState) -> Result<(), crate::RegistryDerefError> {
		lua.dereference(state, self.0)
	}
}
