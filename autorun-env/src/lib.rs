mod functions;

use autorun_lua::LuaApi;
use autorun_types::LuaState;

#[repr(transparent)]
pub struct EnvHandle(i32);

impl EnvHandle {
	fn create_autorun_table(lua: &LuaApi, state: *mut LuaState) {
		lua.create_table(state, 0, 3);

		lua.push_string(state, c"NAME".as_ptr());
		lua.push_string(state, c"".as_ptr());
		lua.set_table(state, -3);

		lua.push_string(state, c"CODE".as_ptr());
		lua.push_string(state, c"".as_ptr());
		lua.set_table(state, -3);

		lua.push_string(state, c"CODE_LEN".as_ptr());
		lua.push_integer(state, 0);
		lua.set_table(state, -3);

		lua.push_string(state, c"print".as_ptr());
		lua.push_function(state, autorun_lua::as_lua_function!(functions::print));
		lua.set_table(state, -3);
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState) -> Self {
		// Create autorun environment
		lua.create_table(state, 0, 1);

		lua.push_string(state, c"Autorun".as_ptr());
		Self::create_autorun_table(lua, state);
		lua.set_table(state, -3);

		// Create metatable for environment
		lua.create_table(state, 0, 1);

		lua.push_string(state, c"__index".as_ptr());
		lua.push_globals(state);
		lua.set_table(state, -3);

		lua.set_metatable(state, -2);

		// Pop environment table and store reference
		EnvHandle(lua.reference(state))
	}

	fn push_autorun_table(&self, lua: &LuaApi, state: *mut LuaState) {
		lua.get_registry(state, self.0);
		lua.get_field(state, -1, c"Autorun".as_ptr());
		lua.remove(state, -2);
	}

	pub fn set_name(&self, lua: &LuaApi, state: *mut LuaState, name: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push_string(state, c"NAME".as_ptr());
		lua.push_lstring(state, name.as_ptr() as _, name.len());
		lua.set_table(state, -3);
	}

	pub fn set_code(&self, lua: &LuaApi, state: *mut LuaState, code: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push_string(state, c"CODE".as_ptr());
		lua.push_lstring(state, code.as_ptr() as _, code.len());
		lua.set_table(state, -3);

		lua.push_string(state, c"CODE_LEN".as_ptr());
		lua.push_integer(state, code.len() as _);
		lua.set_table(state, -3);
	}

	pub fn set_mode(&self, lua: &LuaApi, state: *mut LuaState, mode: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push_string(state, c"MODE".as_ptr());
		lua.push_lstring(state, mode.as_ptr() as _, mode.len());
		lua.set_table(state, -3);
	}

	/// Pushes the environment onto the stack.
	pub fn push(&self, lua: &LuaApi, state: *mut LuaState) {
		lua.get_registry(state, self.0);
	}

	pub fn destroy(self, lua: &LuaApi, state: *mut LuaState) {
		lua.dereference(state, self.0);
	}
}
