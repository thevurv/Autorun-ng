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

		lua.push_string(state, c"CODE".as_ptr());
		lua.push_string(state, c"".as_ptr());
		lua.set_table(state, -3);

		lua.push_string(state, c"CODE_LEN".as_ptr());
		lua.push_integer(state, 0);
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

		// Create metatable for env table
		lua.create_table(state, 0, 1);

		// Set __index to _G
		lua.push_string(state, c"__index".as_ptr());
		lua.push_globals(state);
		lua.set_table(state, -3);

		// Set metatable for env table
		lua.set_metatable(state, -2);

		// Store env_table in registry and get reference
		let registry_env_idx = lua.reference(state);

		// Pop env_table (we're done with it on the stack)
		lua.pop(state, 1);

		EnvHandle(registry_env_idx)
	}

	pub fn set_name(&self, lua: &LuaApi, state: *mut LuaState, name: &[u8]) {
		lua.get_registry(state, self.0);
		lua.push_string(state, c"Autorun".as_ptr());
		lua.get_table(state, -2);

		lua.push_string(state, c"NAME".as_ptr());
		lua.push_lstring(state, name.as_ptr() as _, name.len());
		lua.set_table(state, -3);

		lua.pop(state, 2);
	}

	pub fn set_code(&self, lua: &LuaApi, state: *mut LuaState, code: &[u8]) {
		lua.get_registry(state, self.0);
		lua.push_string(state, c"Autorun".as_ptr());
		lua.get_table(state, -2);

		lua.push_string(state, c"CODE".as_ptr());
		lua.push_lstring(state, code.as_ptr() as _, code.len());
		lua.set_table(state, -3);

		lua.push_string(state, c"CODE_LEN".as_ptr());
		lua.push_integer(state, code.len() as _);
		lua.set_table(state, -3);

		lua.pop(state, 2);
	}

	pub fn set_mode(&self, lua: &LuaApi, state: *mut LuaState, mode: &[u8]) {
		lua.get_registry(state, self.0);
		lua.push_string(state, c"Autorun".as_ptr());
		lua.get_table(state, -2);

		lua.push_string(state, c"MODE".as_ptr());
		lua.push_lstring(state, mode.as_ptr() as _, mode.len());
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
