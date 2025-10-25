pub mod global;

use std::ffi::CStr;

use autorun_core::plugins::Plugin;
use autorun_lua::{LuaApi, RawHandle};
use autorun_types::LuaState;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EnvHandle(RawHandle);

impl core::ops::Deref for EnvHandle {
	type Target = RawHandle;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

macro_rules! as_env_lua_function {
	($func:expr) => {
		autorun_lua::as_lua_function!(|lua: &LuaApi, state: *mut LuaState| {
			let realm = crate::global::get_realm(state);
			let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

			if !env.is_active(lua, state) {
				autorun_log::warn!(
					"Attempted to call '{}' outside of authorized environment",
					stringify!($func)
				);

				// todo: potentially add a silenterror type so we can return that and it'll return a nil.
				// right now this would kind of leak the fact that it's an autorun function.
				lua.push(state, c"");
				lua.error(state);
			} else {
				$func(lua, state, env)
			}
		})
	};
}

impl EnvHandle {
	pub fn is_active(&self, lua: &LuaApi, state: *mut LuaState) -> bool {
		if lua.get_info(state, 1, c"f").is_none() {
			// No function info available
			return false;
		}

		lua.get_fenv(state, -1);
		self.push(lua, state);

		let equal = lua.is_raw_equal(state, -1, -2);
		lua.pop(state, 3);

		equal
	}

	pub fn get_active_plugin(&self, lua: &LuaApi, state: *mut LuaState) -> Option<&Plugin> {
		if !self.is_active(lua, state) {
			return None;
		}

		self.push_autorun_table(lua, state);
		lua.get_field(state, -1, c"PLUGIN".as_ptr());

		let dir = lua.to_userdata(state, -1) as *mut Plugin;
		if dir.is_null() {
			lua.pop(state, 3);
			return None;
		}
		lua.pop(state, 3);

		unsafe { dir.as_ref() }
	}

	fn create_autorun_table(lua: &LuaApi, state: *mut LuaState) {
		lua.create_table(state, 0, 6);

		lua.push(state, c"print");
		lua.push(state, as_env_lua_function!(crate::functions::print));
		lua.set_table(state, -3);

		lua.push(state, c"read");
		lua.push(state, as_env_lua_function!(crate::functions::read));
		lua.set_table(state, -3);

		lua.push(state, c"write");
		lua.push(state, as_env_lua_function!(crate::functions::write));
		lua.set_table(state, -3);

		lua.push(state, c"writeAsync");
		lua.push(state, as_env_lua_function!(crate::functions::write_async));
		lua.set_table(state, -3);

		lua.push(state, c"mkdir");
		lua.push(state, as_env_lua_function!(crate::functions::mkdir));
		lua.set_table(state, -3);

		lua.push(state, c"VERSION");
		lua.push(state, env!("CARGO_PKG_VERSION").to_string());
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
		lua.create_table(state, 0, 2);

		lua.push(state, "Autorun");
		Self::create_autorun_table(lua, state);
		lua.set_table(state, -3);

		lua.push(state, "_G");
		lua.push_globals(state);
		lua.set_table(state, -3);

		// Can unwrap since we are sure there is something on the stack
		let handle = RawHandle::from_stack(lua, state).unwrap();

		// Create lua standard library
		let this = Self(handle);
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/builtins.lua"))?;
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/include.lua"))?;
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/require.lua"))?;
		this.execute(lua, state, c"@stdlib", include_bytes!("./lua/event.lua"))?;

		Ok(this)
	}

	fn push_autorun_table(&self, lua: &LuaApi, state: *mut LuaState) {
		self.0.push(lua, state);
		lua.get_field(state, -1, c"Autorun".as_ptr());
		lua.remove(state, -2);
	}

	pub fn set_plugin(&self, lua: &LuaApi, state: *mut LuaState, plugin: &Plugin) -> anyhow::Result<()> {
		self.push_autorun_table(lua, state);

		lua.push(state, c"PLUGIN");
		lua.new_userdata(state, plugin.try_clone()?);
		lua.set_table(state, -3);

		lua.pop(state, 1);
		Ok(())
	}
}
