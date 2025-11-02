pub mod global;

use anyhow::Context;
use autorun_core::plugins::Plugin;
use autorun_log::*;
use autorun_lua::{LuaApi, RawHandle};
use autorun_luajit::{GCRef, LJState, index2adr};
use autorun_types::{LuaState, Realm};
use std::ffi::{CStr, c_int};

#[derive(Debug, Clone, Copy)]
pub struct EnvHandle {
	realm: Realm,
	env_gcr: GCRef,
	handle: RawHandle,
}

impl core::ops::Deref for EnvHandle {
	type Target = RawHandle;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}

macro_rules! as_env_lua_function {
	($func:expr) => {
		autorun_lua::as_lua_function!(|lua: &LuaApi, state: *mut LuaState| {
			let realm = crate::global::get_realm(state);
			let env = crate::global::get_realm_env(realm).ok_or_else(|| anyhow::anyhow!("env doesn't exist somehow"))?;

			if !env.is_active(lua, state) {
				warn!(
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
	pub fn realm(&self) -> Realm {
		self.realm
	}

	pub fn is_function_authorized(&self, lua: &LuaApi, state: *mut LuaState, func_index: Option<i32>) -> anyhow::Result<bool> {
		let func_index = func_index.unwrap_or(-1);

		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };

		lua.get_fenv(state, func_index);
		let function_env_tvalue = index2adr(lj_state, -1).context("Failed to get TValue for function environment")?;
		let function_env_gcr = unsafe { (*function_env_tvalue).gcr };

		lua.pop(state, 1);
		Ok(function_env_gcr == self.env_gcr)
	}

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
			lua.pop(state, 2);
			return None;
		}
		lua.pop(state, 2);

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

		lua.push(state, c"append");
		lua.push(state, as_env_lua_function!(crate::functions::append));
		lua.set_table(state, -3);

		lua.push(state, c"exists");
		lua.push(state, as_env_lua_function!(crate::functions::exists));
		lua.set_table(state, -3);

		lua.push(state, c"detour");
		lua.push(state, as_env_lua_function!(crate::functions::detour));
		lua.set_table(state, -3);

		lua.push(state, c"enableDetour");
		lua.push(state, as_env_lua_function!(crate::functions::detour_enable));
		lua.set_table(state, -3);

		lua.push(state, c"disableDetour");
		lua.push(state, as_env_lua_function!(crate::functions::detour_disable));
		lua.set_table(state, -3);

		lua.push(state, c"removeDetour");
		lua.push(state, as_env_lua_function!(crate::functions::detour_remove));
		lua.set_table(state, -3);

		lua.push(state, c"getOriginalFunction");
		lua.push(state, as_env_lua_function!(crate::functions::detour_get_original));
		lua.set_table(state, -3);

		lua.push(state, c"copyFastFunction");
		lua.push(state, as_env_lua_function!(crate::functions::copy_fast_function));
		lua.set_table(state, -3);

		lua.push(state, c"load");
		lua.push(state, as_env_lua_function!(crate::functions::load));
		lua.set_table(state, -3);

		lua.push(state, c"triggerRemote");
		lua.push(state, as_env_lua_function!(crate::functions::trigger_remote));
		lua.set_table(state, -3);

		lua.push(state, c"isFunctionAuthorized");
		lua.push(state, as_env_lua_function!(crate::functions::is_function_authorized));
		lua.set_table(state, -3);

		lua.push(state, c"safeCall");
		lua.push(state, as_env_lua_function!(crate::functions::safe_call));
		lua.set_table(state, -3);

		lua.push(state, c"VERSION");
		lua.push(state, env!("CARGO_PKG_VERSION").to_string());
		lua.set_table(state, -3);
	}

	pub fn execute(&self, lua: &LuaApi, state: *mut LuaState, name: &CStr, src: &[u8]) -> anyhow::Result<()> {
		if let Err(why) = lua.load_buffer_x(state, src, name, c"t") {
			anyhow::bail!("Failed to compile: {why}");
		}

		self.push(lua, state);
		if lua.set_fenv(state, -2).is_err() {
			anyhow::bail!("Failed to set environment");
		}

		if let Err(why) = lua.pcall(state, 0, 0, 0) {
			anyhow::bail!("Failed to execute: {}", why);
		}

		Ok(())
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState, realm: Realm) -> anyhow::Result<Self> {
		// Create autorun environment
		lua.create_table(state, 0, 2);

		lua.push(state, "Autorun");
		Self::create_autorun_table(lua, state);
		lua.set_table(state, -3);

		lua.push(state, "_G");
		lua.push_globals(state);
		lua.set_table(state, -3);

		// Can unwrap since we are sure there is something on the stack
		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };
		let env_tvalue = index2adr(lj_state, -1).context("Failed to get TValue for environment")?;
		let env_gcr = unsafe { (*env_tvalue).gcr };

		let handle = RawHandle::from_stack(lua, state).unwrap();
		Ok(Self { realm, env_gcr, handle })
	}

	pub fn push_autorun_table(&self, lua: &LuaApi, state: *mut LuaState) {
		self.push(lua, state);
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

	pub fn trigger(&self, lua: &LuaApi, state: *mut LuaState, event_name: &CStr, n_args: c_int) -> anyhow::Result<()> {
		lua.push(state, event_name);
		lua.insert(state, -(n_args + 1));

		self.push_autorun_table(lua, state);
		lua.get_field(state, -1, c"trigger".as_ptr());
		lua.remove(state, -2); // remove Autorun table

		if lua.type_id(state, -1) != autorun_lua::LuaTypeId::Function {
			lua.pop(state, 1);
			anyhow::bail!("don't remove Autorun.trigger lil bro.");
		}

		lua.insert(state, -(n_args + 2));
		lua.pcall(state, n_args + 1, 0, 0).map_err(|e| anyhow::anyhow!(e))?;

		Ok(())
	}

	pub fn run_remote_callbacks(
		&self,
		lua: &LuaApi,
		state: *mut LuaState,
		event_name: &CStr,
		n_args: c_int,
	) -> anyhow::Result<()> {
		lua.push(state, event_name);
		lua.insert(state, -(n_args + 1));

		self.push_autorun_table(lua, state);
		lua.get_field(state, -1, c"runRemoteCallbacks".as_ptr());
		lua.remove(state, -2); // remove Autorun table

		if lua.type_id(state, -1) != autorun_lua::LuaTypeId::Function {
			lua.pop(state, 1);
			anyhow::bail!("don't remove Autorun.runRemoteCallbacks lil bro.");
		}

		lua.insert(state, -(n_args + 2));
		lua.pcall(state, n_args + 1, 0, 0).map_err(|e| anyhow::anyhow!(e))?;

		Ok(())
	}
}
