use crate::{FromLua, IntoLua, LuaApi, LuaState};

/// A handle to a value in the lua registry.
pub struct RawHandle(i32);

impl RawHandle {
	/// Pops the value at the top of the stack and stores it in the registry.
	pub fn from_stack(lua: &crate::LuaApi, state: *mut LuaState) -> Option<Self> {
		if lua.get_top(state) < 1 {
			return None;
		}

		Some(Self(lua.reference(state)))
	}

	pub fn push(&self, lua: &crate::LuaApi, state: *mut LuaState) {
		lua.get_registry(state, self.0);
	}
}

struct Handle<T> {
	_marker: core::marker::PhantomData<T>,
}
