mod raw;
pub use raw::*;

mod result;
pub use result::*;

use crate::{FromLua, IntoLua, LuaTypeId, LuaValue, types::LuaState};
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

	pub fn loadstring(&self, state: *mut LuaState, s: *const c_char) -> Result<(), std::borrow::Cow<'static, str>> {
		match self.raw.loadstring(state, s) {
			LUA_OK | LUA_YIELD => Ok(()),
			_ => {
				let err_msg = self.raw.tolstring(state, -1, std::ptr::null_mut());
				self.raw.pop(state, 1);

				let err_str = if !err_msg.is_null() {
					unsafe { std::ffi::CStr::from_ptr(err_msg) }.to_string_lossy()
				} else {
					std::borrow::Cow::Borrowed("Unknown error")
				};

				Err(err_str)
			}
		}
	}

	pub fn setfenv(&self, state: *mut LuaState, index: c_int) -> LuaResult<()> {
		if self.raw.setfenv(state, index) != 0 {
			Ok(())
		} else {
			Err(LuaError::GenericFailure)
		}
	}

	pub fn loadbufferx(&self, state: *mut LuaState, buff: &[u8], name: &CStr, mode: &CStr) -> LuaResult<()> {
		match self
			.raw
			.loadbufferx(state, buff.as_ptr() as _, buff.len(), name.as_ptr(), mode.as_ptr())
		{
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err = self.raw.checkstring(state, -1);
				self.raw.pop(state, 1);

				Err(LuaError::Runtime(err.to_string()))
			}
		}
	}

	pub fn pcall(&self, state: *mut LuaState, n_args: c_int, n_results: c_int, err_func: c_int) -> LuaResult<()> {
		match self.raw.pcall(state, n_args, n_results, err_func) {
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err = self.raw.checkstring(state, -1);
				self.raw.pop(state, 1);

				Err(LuaError::Runtime(err.to_string()))
			}
		}
	}

	pub fn push<T: IntoLua>(&self, state: *mut LuaState, value: T) {
		T::into_lua(value, self, state);
	}

	pub fn to<T: FromLua>(&self, state: *mut LuaState, stack_idx: c_int) -> T {
		T::from_lua(self, state, stack_idx)
	}

	pub fn getregistry(&self, state: *mut LuaState, key: impl IntoLua) -> LuaValue {
		key.into_lua(self, state);
		self.raw.rawget(state, REGISTRY_INDEX);
		let value = self.to(state, -1);
		self.raw.pop(state, 1);
		value
	}

	pub fn setregistry(&self, state: *mut LuaState, key: impl IntoLua, value: impl IntoLua) {
		key.into_lua(self, state);
		value.into_lua(self, state);
		self.raw.rawset(state, REGISTRY_INDEX);
	}
}

pub trait LuaReturn {
	fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32;
}

impl LuaReturn for () {
	fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32 {
		0
	}
}

impl<T: IntoLua> LuaReturn for T {
	fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32 {
		self.into_lua(lua, state);
		1
	}
}

#[repr(transparent)]
pub struct RawLuaReturn(pub i32);

impl LuaReturn for RawLuaReturn {
	fn into_lua_return(self, _lua: &LuaApi, _state: *mut LuaState) -> i32 {
		self.0
	}
}

// Macro to implement LuaReturn for tuples
macro_rules! impl_lua_return_tuple {
    ($($T:ident),+) => {
        impl<$($T: IntoLua),+> LuaReturn for ($($T,)+) {
            fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32 {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                let mut count = 0;
                $(
                    $T.into_lua(lua, state);
                    count += 1;
                )+
                count
            }
        }
    };
}

impl_lua_return_tuple!(T1);
impl_lua_return_tuple!(T1, T2);
impl_lua_return_tuple!(T1, T2, T3);
impl_lua_return_tuple!(T1, T2, T3, T4);
impl_lua_return_tuple!(T1, T2, T3, T4, T5);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

#[macro_export]
macro_rules! as_lua_function {
	($func:expr) => {{
		extern "C-unwind" fn lua_wrapper(state: *mut $crate::LuaState) -> i32 {
			let lua = autorun_lua::get_api().expect("Failed to get Lua API");
			match $func(lua, state) {
				Ok(ret) => $crate::LuaReturn::into_lua_return(ret, lua, state),
				Err(e) => {
					lua.push(state, e.to_string());
					lua.raw.error(state);
				}
			}
		}
		lua_wrapper as $crate::LuaCFunction
	}};
}
