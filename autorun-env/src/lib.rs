mod functions;

use autorun_lua::LuaApi;
use autorun_types::LuaState;

#[repr(transparent)]
pub struct EnvHandle(i32);

impl EnvHandle {
	fn add_print_functions(lua: &LuaApi, state: *mut LuaState) {
		lua.push_string(state, c"print".as_ptr());
		lua.push_function(state, autorun_lua::as_lua_function!(functions::print));
		lua.set_table(state, -3);
	}

	fn create_autorun_table(lua: &LuaApi, state: *mut LuaState) {
		lua.create_table(state, 0, 2);

		lua.push_string(state, c"NAME".as_ptr());
		lua.push_string(state, c"".as_ptr());
		lua.set_table(state, -3);

		lua.push_string(state, c"SRC".as_ptr());
		lua.push_string(state, c"".as_ptr());
		lua.set_table(state, -3);

		Self::add_print_functions(lua, state);
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState) -> Self {
		// // Ensure debug.getregistry isnt a C function.
		// // This should never be the case as its been ages since it was removed
		// // But I'd rather not take ANY chances that the Autorun functions are leaked to the client.
		// lua.get_global(state, c"debug".as_ptr());
		// lua.push_string(state, c"getregistry".as_ptr());
		// lua.get_table(state, -2);
		// if lua.is_c_function(state, -1) != 0 {
		// 	lua.push_string(state, c"getregistry".as_ptr());
		// 	lua.push_nil(state);
		// 	lua.set_table(state, -3);
		// }
		// lua.pop(state, 2);

		// Create env table first
		lua.create_table(state, 0, 1);

		// Create and configure autorun table
		Self::create_autorun_table(lua, state);

		// Store autorun_table in env_table
		lua.push_string(state, c"Autorun".as_ptr());
		lua.push_value(state, -2);
		lua.set_table(state, -4);

		// Pop autorun_table (we're done with it on the stack)
		lua.pop(state, 1);

		// Store env_table in registry and get reference
		let registry_env_idx = lua.reference(state);

		// Pop env_table (we're done with it on the stack)
		lua.pop(state, 1);

		EnvHandle(registry_env_idx)
	}

	pub fn set_name(&self, lua: &LuaApi, state: *mut LuaState, name: &str) {
		lua.get_registry(state, self.0);
		lua.push_string(state, c"Autorun".as_ptr());
		lua.get_table(state, -2);

		lua.push_string(state, c"NAME".as_ptr());
		lua.push_string(state, std::ffi::CString::new(name).unwrap().as_ptr());
		lua.set_table(state, -3);

		lua.pop(state, 2);
	}

	pub fn set_src(&self, lua: &LuaApi, state: *mut LuaState, src: &str) {
		lua.get_registry(state, self.0);
		lua.push_string(state, c"Autorun".as_ptr());
		lua.get_table(state, -2);

		lua.push_string(state, c"SRC".as_ptr());
		lua.push_string(state, std::ffi::CString::new(src).unwrap().as_ptr());
		lua.set_table(state, -3);

		lua.pop(state, 2);
	}

	/// Pushes the environment onto the stack.
	pub fn push(&self, lua: &LuaApi, state: *mut LuaState) {
		lua.get_registry(state, self.0);
	}

	pub fn destroy(self, lua: &LuaApi, state: *mut LuaState) {
		lua.dereference(state, self.0);
	}
}
