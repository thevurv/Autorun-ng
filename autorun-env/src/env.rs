pub mod global;

use std::ffi::CStr;

use autorun_core::plugins::Plugin;
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

		lua.push(state, c"read");
		lua.push(state, autorun_lua::as_lua_function!(crate::functions::read));
		lua.set_table(state, -3);

		lua.push(state, c"write");
		lua.push(state, autorun_lua::as_lua_function!(crate::functions::write));
		lua.set_table(state, -3);

		lua.push(state, c"writeAsync");
		lua.push(state, autorun_lua::as_lua_function!(crate::functions::write_async));
		lua.set_table(state, -3);

		lua.push(state, c"mkdir");
		lua.push(state, autorun_lua::as_lua_function!(crate::functions::mkdir));
		lua.set_table(state, -3);
	}

	pub fn execute(&self, lua: &LuaApi, state: *mut LuaState, name: &CStr, src: &[u8]) -> anyhow::Result<()> {
		if let Err(why) = lua.load_buffer_x(state, src, name, c"t") {
			return Err(anyhow::anyhow!("Failed to compile: {why}"));
		}

		self.push(lua, state);
		if lua.set_fenv(state, -2).is_err() {
			return Err(anyhow::anyhow!("Failed to set environment"));
		}

		if let Err(why) = lua.pcall(state, 0, 0, 0) {
			return Err(anyhow::anyhow!("Failed to execute: {}", why));
		}

		Ok(())
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<Self> {
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

		// Create lua standard library
		let this = Self { handle };
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/builtins.lua"))?;
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/include.lua"))?;
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/require.lua"))?;

		Ok(this)
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

	pub fn set_plugin(&self, lua: &LuaApi, state: *mut LuaState, plugin: &Plugin) {
		self.push_autorun_table(lua, state);

		lua.push(state, c"PLUGIN");
		lua.push_lightuserdata(state, &raw const *plugin as _);
		lua.set_table(state, -3);

		lua.pop(state, 1);
	}

	/// Pushes the environment onto the stack.
	pub fn push(&self, lua: &LuaApi, state: *mut LuaState) {
		self.handle.push(lua, state);
	}
}
