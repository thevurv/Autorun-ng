mod raw;
pub use raw::*;

mod result;
pub use result::*;

mod returns;
pub use returns::*;

use crate::{FromLua, IntoLua, LuaFunction, LuaTable, LuaValue, types::LuaState};
use std::ffi::{CStr, c_char, c_int};

#[derive(Debug)]
pub struct LuaApi {
	pub raw: RawLuaApi,
}

impl LuaApi {
	pub fn new(lib: &libloading::Library) -> Result<Self, libloading::Error> {
		let raw = RawLuaApi::new(lib)?;
		Ok(Self { raw })
	}

	pub fn load(&self, state: *mut LuaState, src: impl AsRef<[u8]>, name: &CStr) -> LuaResult<LuaFunction> {
		let src = src.as_ref();
		self.raw.loadbufferx(state, src, name, c"t")?;
		let func = self.raw.try_to(state, -1)?;
		self.raw.pop(state, -1);

		Ok(func)
	}

	pub fn setfenv(&self, state: *mut LuaState, f: &LuaFunction, env: &LuaTable) -> LuaResult<()> {
		self.raw.push(state, f);
		self.raw.push(state, env);
		self.raw.setfenv(state, -2)?;
		self.raw.pop(state, 1);

		Ok(())
	}

	pub fn getregistry(&self, state: *mut LuaState, key: impl IntoLua) -> LuaValue<'_> {
		key.into_lua(&self.raw, state);
		self.raw.rawget(state, REGISTRY_INDEX);
		let value = self.raw.to(state, -1);
		self.raw.pop(state, 1);
		value
	}

	pub fn setregistry(&self, state: *mut LuaState, key: impl IntoLua, value: impl IntoLua) {
		key.into_lua(&self.raw, state);
		value.into_lua(&self.raw, state);
		self.raw.rawset(state, REGISTRY_INDEX);
	}

	pub fn error(&self, state: *mut LuaState, msg: impl IntoLua) -> ! {
		self.raw.push(state, msg);
		self.raw.error(state);
	}
}

#[macro_export]
macro_rules! as_lua_function {
	($func:expr) => {{
		extern "C-unwind" fn lua_wrapper(state: *mut $crate::LuaState) -> i32 {
			let lua = autorun_lua::get_api().expect("Failed to get Lua API");
			match $func(lua, state) {
				Ok(ret) => $crate::LuaReturn::into_lua_return(ret, &lua.raw, state),
				Err(e) => {
					lua.error(state, e.to_string());
				}
			}
		}
		lua_wrapper as $crate::LuaCFunction
	}};
}
