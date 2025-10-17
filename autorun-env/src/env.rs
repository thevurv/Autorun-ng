use autorun_lua::{IntoLua, LuaApi, RawHandle};
use autorun_types::LuaState;

pub struct Environment {
	handle: RawHandle,
}

impl Environment {
	fn create_autorun_table(lua: &LuaApi, state: *mut LuaState) {
		lua.create_table(state, 0, 3);

		lua.push(state, c"NAME");
		lua.push(state, c"");
		lua.set_table(state, -3);

		lua.push(state, c"CODE");
		lua.push(state, c"");
		lua.set_table(state, -3);

		lua.push(state, c"CODE_LEN");
		lua.push(state, 0);
		lua.set_table(state, -3);

		lua.push(state, c"print");
		lua.push(state, autorun_lua::as_lua_function!(crate::functions::print));
		lua.set_table(state, -3);
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState) -> Self {
		// Create autorun environment
		lua.create_table(state, 0, 1);

		lua.push(state, c"Autorun");
		Self::create_autorun_table(lua, state);
		lua.set_table(state, -3);

		// Create metatable for environment
		lua.create_table(state, 0, 1);

		lua.push(state, c"__index");
		lua.push_globals(state);
		lua.set_table(state, -3);

		lua.set_metatable(state, -2);

		// Can unwrap since we are sure there is something on the stack
		let handle = RawHandle::from_stack(lua, state).unwrap();

		Self { handle }
	}

	fn push_autorun_table(&self, lua: &LuaApi, state: *mut LuaState) {
		self.handle.push(lua, state);
		lua.get_field(state, -1, c"Autorun".as_ptr());
		lua.remove(state, -2);
	}

	pub fn set_name(&self, lua: &LuaApi, state: *mut LuaState, name: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push(state, c"NAME");
		lua.push_lstring(state, name.as_ptr() as _, name.len());
		lua.set_table(state, -3);

		lua.pop(state, 1);
	}

	pub fn set_code(&self, lua: &LuaApi, state: *mut LuaState, code: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push(state, c"CODE");
		lua.push_lstring(state, code.as_ptr() as _, code.len());
		lua.set_table(state, -3);

		lua.push(state, c"CODE_LEN");
		lua.push(state, code.len() as i32);
		lua.set_table(state, -3);

		lua.pop(state, 1);
	}

	pub fn set_mode(&self, lua: &LuaApi, state: *mut LuaState, mode: &[u8]) {
		self.push_autorun_table(lua, state);

		lua.push(state, c"MODE");
		lua.push_lstring(state, mode.as_ptr() as _, mode.len());
		lua.set_table(state, -3);

		lua.pop(state, 1);
	}

	pub fn set_path(&self, lua: &LuaApi, state: *mut LuaState, path: impl IntoLua) {
		self.push_autorun_table(lua, state);

		lua.push(state, c"PATH");
		lua.push(state, path);
		lua.set_table(state, -3);

		lua.pop(state, 1);
	}

	/// Pushes the environment onto the stack.
	pub fn push(&self, lua: &LuaApi, state: *mut LuaState) {
		self.handle.push(lua, state);
	}
}
