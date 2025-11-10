pub mod global;

use anyhow::Context;
use autorun_core::plugins::Plugin;
use autorun_log::*;
use autorun_lua::{Globals, LuaApi, LuaTable};
use autorun_luajit::{GCRef, LJState, index2adr};
use autorun_types::{LuaState, Realm};
use std::ffi::{CStr, CString, c_int};

use crate::functions;

#[derive(Debug, Clone, Copy)]
pub struct EnvHandle {
	realm: Realm,
	env_gcr: GCRef,
	chunk_nonce: u64,
	env: LuaTable,
	autorun: LuaTable,
}

macro_rules! wrap {
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
				lua.raw.error(state);
			} else {
				$func(lua, state, env)
			}
		})
	};
}

impl EnvHandle {
	pub fn push(&self, lua: &LuaApi, state: *mut LuaState) {
		self.env.push(lua, state);
	}

	pub fn realm(&self) -> Realm {
		self.realm
	}

	pub fn is_function_authorized(&self, lua: &LuaApi, state: *mut LuaState, func_index: Option<i32>) -> anyhow::Result<bool> {
		let func_index = func_index.unwrap_or(-1);

		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };

		lua.raw.getfenv(state, func_index);
		let function_env_tvalue = index2adr(lj_state, -1).context("Failed to get TValue for function environment")?;
		let function_env_gcr = unsafe { (*function_env_tvalue).gcr };

		lua.raw.pop(state, 1);
		Ok(function_env_gcr == self.env_gcr)
	}

	pub fn is_active(&self, lua: &LuaApi, state: *mut LuaState) -> bool {
		if lua.raw.getinfo(state, 1, c"f").is_none() {
			// No function info available
			return false;
		}

		lua.raw.getfenv(state, -1);
		self.push(lua, state);

		let equal = lua.raw.rawequal(state, -1, -2);
		lua.raw.pop(state, 3);

		equal
	}

	pub fn get_active_plugin(&self, lua: &LuaApi, state: *mut LuaState) -> Option<&Plugin> {
		if !self.is_active(lua, state) {
			return None;
		}

		let dir: *mut Plugin = lua.get(state, &self.autorun, "PLUGIN");
		let dir = unsafe { dir.as_ref() }?;

		Some(dir)
	}

	fn create_autorun_table(lua: &LuaApi, state: *mut LuaState) -> LuaTable {
		let t = lua.table(state);
		lua.set(state, &t, "print", wrap!(functions::print));
		lua.set(state, &t, "read", wrap!(functions::read));
		lua.set(state, &t, "write", wrap!(functions::write));
		lua.set(state, &t, "writeAsync", wrap!(functions::write_async));
		lua.set(state, &t, "mkdir", wrap!(functions::mkdir));
		lua.set(state, &t, "append", wrap!(functions::append));
		lua.set(state, &t, "exists", wrap!(functions::exists));
		lua.set(state, &t, "detour", wrap!(functions::detour));
		lua.set(state, &t, "enableDetour", wrap!(functions::detour_enable));
		lua.set(state, &t, "disableDetour", wrap!(functions::detour_disable));
		lua.set(state, &t, "removeDetour", wrap!(functions::detour_remove));
		lua.set(state, &t, "getOriginalFunction", wrap!(functions::detour_get_original));
		lua.set(state, &t, "copyFastFunction", wrap!(functions::copy_fast_function));
		lua.set(state, &t, "load", wrap!(functions::load));
		lua.set(state, &t, "triggerRemote", wrap!(functions::trigger_remote));
		lua.set(state, &t, "isFunctionAuthorized", wrap!(functions::is_function_authorized));
		lua.set(state, &t, "isProtoAuthorized", wrap!(functions::is_proto_authorized));
		lua.set(state, &t, "VERSION", env!("CARGO_PKG_VERSION"));

		return t;
	}

	pub fn execute(&self, lua: &LuaApi, state: *mut LuaState, name: &CStr, src: &[u8]) -> anyhow::Result<()> {
		let name = self.format_chunk_name(name)?;
		if let Err(why) = lua.loadbufferx(state, src, &name, c"t") {
			anyhow::bail!("Failed to compile: {why}");
		}

		self.push(lua, state);
		if lua.setfenv(state, -2).is_err() {
			anyhow::bail!("Failed to set environment");
		}

		if let Err(why) = lua.pcall(state, 0, 0, 0) {
			anyhow::bail!("Failed to execute: {}", why);
		}

		Ok(())
	}

	pub fn create(lua: &LuaApi, state: *mut LuaState, realm: Realm) -> anyhow::Result<Self> {
		let autorun = Self::create_autorun_table(lua, state);

		let env = lua.table(state);
		lua.set(state, &env, "Autorun", &autorun);
		lua.set(state, &env, "_G", Globals);

		// todo: refactor luajit code to not depend on the stack
		env.push(lua, state);

		// Can unwrap since we are sure there is something on the stack
		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };
		let env_tvalue = index2adr(lj_state, -1).context("Failed to get TValue for environment")?;
		let env_gcr = unsafe { (*env_tvalue).gcr };

		// todo: refactor luajit code to not depend on the stack
		lua.raw.pop(state, 1);

		let chunk_nonce = rand::random::<u64>();
		Ok(Self {
			realm,
			env_gcr,
			chunk_nonce,
			env,
			autorun,
		})
	}

	pub fn format_chunk_name(&self, base: &CStr) -> anyhow::Result<CString> {
		let formatted = format!("{}-{}", self.chunk_nonce, base.to_str()?);
		Ok(CString::new(formatted)?)
	}

	pub fn is_chunk_name_authorized(&self, chunk_name: &CStr) -> bool {
		match chunk_name.to_str() {
			Ok(name_str) => name_str.starts_with(&self.chunk_nonce.to_string()),
			Err(_) => false,
		}
	}

	pub fn set_plugin(&self, lua: &LuaApi, state: *mut LuaState, plugin: &Plugin) -> anyhow::Result<()> {
		lua.set(state, &self.autorun, "PLUGIN", plugin.try_clone()?);
		Ok(())
	}

	pub fn trigger(&self, lua: &LuaApi, state: *mut LuaState, event_name: &CStr, n_args: c_int) -> anyhow::Result<()> {
		lua.push(state, event_name);
		lua.raw.insert(state, -(n_args + 1));

		self.autorun.push(lua, state);
		lua.raw.getfield(state, -1, c"trigger".as_ptr());
		lua.raw.remove(state, -2); // remove Autorun table

		if lua.raw.typeid(state, -1) != autorun_lua::LuaTypeId::Function {
			lua.raw.pop(state, 1);
			anyhow::bail!("don't remove Autorun.trigger lil bro.");
		}

		lua.raw.insert(state, -(n_args + 2));
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
		lua.raw.insert(state, -(n_args + 1));

		self.autorun.push(lua, state);
		lua.raw.getfield(state, -1, c"runRemoteCallbacks".as_ptr());
		lua.raw.remove(state, -2); // remove Autorun table

		if lua.raw.typeid(state, -1) != autorun_lua::LuaTypeId::Function {
			lua.raw.pop(state, 1);
			anyhow::bail!("don't remove Autorun.runRemoteCallbacks lil bro.");
		}

		lua.raw.insert(state, -(n_args + 2));
		lua.pcall(state, n_args + 1, 0, 0).map_err(|e| anyhow::anyhow!(e))?;

		Ok(())
	}
}
