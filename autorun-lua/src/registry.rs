use crate::{FromLua, IntoLua, LuaApi, LuaState};

/// A handle to a value in the lua registry.
pub struct RawHandle(pub i32);

impl RawHandle {
	/// Pops the value at the top of the stack and stores it in the registry.
	pub fn from_stack(lua: &crate::LuaApi, state: *mut LuaState) -> Option<Self> {
		if lua.get_top(state) < 1 {
			return None;
		}

		lua.reference(state).map(Self)
	}

	pub fn push(&self, lua: &crate::LuaApi, state: *mut LuaState) {
		println!("About to push reference: {} (ptr: {:p})", self.0, &self.0);
		lua.get_registry(state, self.0);

		let type_id = lua.type_id(state, -1);
		let type_name = lua.type_name(state, type_id);
		println!("Registry[87] contains: {:?} (type_id: {})", type_name, type_id);

		// If it's a number, print it
		if type_id == super::LUA_TNUMBER {
			// Assuming you have a to_number method
			println!("  Value: {}", lua.to_number(state, -1));
		}
	}
}
