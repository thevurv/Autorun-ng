#![allow(unused)]
use core::ffi::{CStr, c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};

use crate::{FromLua, IntoLua, LuaFunction, LuaTypeId, types::LuaState};

const LUA_IDSIZE: usize = 60;

#[repr(C)]
#[derive(Debug)]
pub struct DebugInfo {
	pub event: c_int,
	pub name: *const c_char,             // n
	pub namewhat: *const c_char,         // n
	pub what: *const c_char,             // S
	pub source: *const c_char,           // S
	pub currentline: c_int,              // l
	pub nups: c_uchar,                   // u
	pub linedefined: c_int,              // S
	pub lastlinedefined: c_int,          // S
	pub short_src: [c_char; LUA_IDSIZE], // S
}

pub const GLOBALS_INDEX: c_int = -10002;
pub const ENVIRON_INDEX: c_int = -10001;
pub const REGISTRY_INDEX: c_int = -10000;

const LUA_OK: c_int = 0;
const LUA_YIELD: c_int = 1;
const LUA_ERRRUN: c_int = 2;
const LUA_ERRSYNTAX: c_int = 3;
const LUA_ERRMEM: c_int = 4;
const LUA_ERRGCMM: c_int = 5;
const LUA_ERRERR: c_int = 6;

const LUA_REFNIL: c_int = -1;
const LUA_NOREF: c_int = -2;

macro_rules! define_lua_api {
    (
        $(
            #[name = $name:literal]
            $vis:vis fn $fn_name:ident($($param_name:ident: $param_type:ty),* $(,)?) $(-> $return_type:ty)?;
        )*
    ) => {
        #[derive(Debug)]
        pub struct LuaApi {
            $(
                pub $fn_name: extern "C-unwind" fn($($param_name: $param_type),*) $(-> $return_type)?,
            )*
        }

        impl LuaApi {
            pub fn new(lua_shared: &libloading::Library) -> Result<LuaApi, libloading::Error> {
                let api = LuaApi {
                    $(
                        $fn_name: *unsafe { lua_shared.get(concat!($name, "\0").as_bytes())? },
                    )*
                };
                Ok(api)
            }

            $(
                $vis fn $fn_name(&self, $($param_name: $param_type),*) $(-> $return_type)? {
                    (self.$fn_name)($($param_name),*)
                }
            )*
        }
    };
}

define_lua_api! {
	#[name = "lua_pushnumber"]
	pub fn push_number(state: *mut LuaState, number: c_double);
	#[name = "lua_pushinteger"]
	pub fn push_integer(state: *mut LuaState, integer: c_int);
	#[name = "lua_pushnil"]
	pub fn push_nil(state: *mut LuaState);
	#[name = "lua_pushstring"]
	pub fn push_string(state: *mut LuaState, string: *const c_char);
	#[name = "lua_pushlstring"]
	pub fn push_lstring(state: *mut LuaState, string: *const c_char, len: usize);
	#[name = "lua_pushthread"]
	pub fn push_thread(state: *mut LuaState) -> c_int;
	#[name = "lua_pushvalue"]
	pub fn push_value(state: *mut LuaState, index: c_int);
	#[name = "lua_pushcclosure"]
	pub fn push_closure(state: *mut LuaState, func: LuaFunction, nups: c_int);
	#[name = "lua_pushlightuserdata"]
	pub fn push_lightuserdata(state: *mut LuaState, p: *mut c_void);
	#[name = "lua_pushboolean"]
	fn _push_boolean(state: *mut LuaState, b: c_int);

	#[name = "lua_rawequal"]
	fn _rawequal(state: *mut LuaState, index1: c_int, index2: c_int) -> c_int;
	#[name = "lua_rawget"]
	pub fn rawget(state: *mut LuaState, index: c_int);
	#[name = "lua_rawgeti"]
	pub fn rawgeti(state: *mut LuaState, index: c_int, n: c_int);
	#[name = "lua_rawset"]
	pub fn rawset(state: *mut LuaState, index: c_int);
	#[name = "lua_rawseti"]
	pub fn rawseti(state: *mut LuaState, index: c_int, n: c_int);

	#[name = "lua_gettable"]
	pub fn get_table(state: *mut LuaState, index: c_int);
	#[name = "lua_settable"]
	pub fn set_table(state: *mut LuaState, index: c_int);
	#[name = "lua_getfield"]
	pub fn get_field(state: *mut LuaState, index: c_int, k: *const c_char);
	#[name = "lua_insert"]
	pub fn insert(state: *mut LuaState, index: c_int);

	#[name = "luaL_loadbufferx"]
	pub fn _load_buffer_x(
		state: *mut LuaState,
		buff: *const c_char,
		size: usize,
		name: *const c_char,
		mode: *const c_char,
	) -> c_int;
	#[name = "luaL_loadstring"]
	pub fn _load_string(state: *mut LuaState, str: *const c_char) -> c_int;
	#[name = "luaL_checknumber"]
	pub fn check_number(state: *mut LuaState, index: c_int) -> c_double;
	#[name = "luaL_checklstring"]
	pub fn check_lstring(state: *mut LuaState, index: c_int, len: *mut c_uint) -> *const c_char;

	#[name = "lua_call"]
	pub fn call(state: *mut LuaState, n_args: c_int, n_results: c_int) -> c_int;
	#[name = "lua_pcall"]
	fn _pcall(state: *mut LuaState, n_args: c_int, n_results: c_int, err_func: c_int) -> c_int;
	#[name = "lua_createtable"]
	pub fn create_table(state: *mut LuaState, narr: c_int, nrec: c_int);
	#[name = "lua_equal"]
	fn _equal(state: *mut LuaState, index1: c_int, index2: c_int) -> c_int;
	#[name = "lua_error"]
	pub fn error(state: *mut LuaState) -> !;
	#[name = "lua_gc"]
	pub fn gc(state: *mut LuaState, what: c_int, data: c_int) -> c_int;
	#[name = "lua_settop"]
	pub fn set_top(state: *mut LuaState, index: c_int);
	#[name = "lua_gettop"]
	pub fn get_top(state: *mut LuaState) -> c_int;
	#[name = "lua_remove"]
	pub fn remove(state: *mut LuaState, index: c_int);
	#[name = "lua_status"]
	pub fn status(state: *mut LuaState) -> c_int;
	#[name = "lua_type"]
	fn _type_id(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_typename"]
	fn _type_name(state: *mut LuaState, type_: c_int) -> *const c_char;

	#[name = "lua_toboolean"]
	fn _to_boolean(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_tonumber"]
	pub fn to_number(state: *mut LuaState, index: c_int) -> c_double;
	#[name = "lua_topointer"]
	pub fn to_pointer(state: *mut LuaState, index: c_int) -> *const c_void;
	#[name = "lua_tolstring"]
	pub fn to_lstring(state: *mut LuaState, index: c_int, len: *mut c_uint) -> *const c_char;
	#[name = "lua_tothread"]
	pub fn to_thread(state: *mut LuaState, index: c_int) -> *mut LuaState;
	#[name = "lua_touserdata"]
	pub fn to_userdata(state: *mut LuaState, index: c_int) -> *mut c_void;
	#[name = "lua_tointeger"]
	pub fn to_integer(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_tocfunction"]
	pub fn to_function(state: *mut LuaState, index: c_int) -> LuaFunction;

	#[name = "lua_xmove"]
	pub fn xmove(from: *mut LuaState, to: *mut LuaState, n: c_int);
	#[name = "lua_iscfunction"]
	pub fn is_c_function(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isnumber"]
	pub fn is_number(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isstring"]
	pub fn is_string(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isuserdata"]
	pub fn is_userdata(state: *mut LuaState, index: c_int) -> c_int;

	// Debug library
	#[name = "lua_gethook"]
	pub fn get_hook(state: *mut LuaState) -> *const c_void;
	#[name = "lua_gethookcount"]
	pub fn get_hook_count(state: *mut LuaState) -> c_int;
	#[name = "lua_gethookmask"]
	pub fn get_hook_mask(state: *mut LuaState) -> c_int;
	#[name = "lua_getinfo"]
	fn _get_info(state: *mut LuaState, what: *const c_char, ar: *mut c_void) -> c_int;
	#[name = "lua_getlocal"]
	pub fn get_local(state: *mut LuaState, ar: *mut c_void, n: c_int) -> *const c_char;
	#[name = "lua_getstack"]
	pub fn get_stack(state: *mut LuaState, level: c_int, ar: *mut c_void) -> c_int;
	#[name = "lua_getupvalue"]
	pub fn get_upvalue(state: *mut LuaState, funcindex: c_int, n: c_int) -> *const c_char;
	#[name = "lua_sethook"]
	pub fn set_hook(state: *mut LuaState, func: *const c_void, mask: c_int, count: c_int);
	#[name = "lua_setlocal"]
	pub fn set_local(state: *mut LuaState, ar: *mut c_void, n: c_int) -> *const c_char;
	#[name = "lua_setupvalue"]
	pub fn set_upvalue(state: *mut LuaState, funcindex: c_int, n: c_int) -> *const c_char;

	#[name = "lua_getfenv"]
	pub fn get_fenv(state: *mut LuaState, index: c_int);
	#[name = "lua_setfenv"]
	fn _set_fenv(state: *mut LuaState, index: c_int) -> c_int;

	#[name = "luaL_ref"]
	fn _reference(state: *mut LuaState, t: c_int) -> c_int;
	#[name = "luaL_unref"]
	fn _dereference(state: *mut LuaState, t: c_int, r: c_int);

	#[name = "lua_getmetatable"]
	pub fn get_metatable(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_setmetatable"]
	pub fn set_metatable(state: *mut LuaState, index: c_int) -> c_int;

	#[name = "lua_newuserdata"]
	fn _new_userdata(state: *mut LuaState, size: usize) -> *mut c_void;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SetfenvError {
	GenericFail,
}

impl core::fmt::Display for SetfenvError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			SetfenvError::GenericFail => write!(f, "Failed to set environment"),
		}
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RegistryDerefError {
	InvalidReference,
}

impl core::fmt::Display for RegistryDerefError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			RegistryDerefError::InvalidReference => write!(f, "Invalid registry reference"),
		}
	}
}

impl core::error::Error for SetfenvError {}

impl LuaApi {
	pub fn get_global(&self, state: *mut LuaState, name: *const c_char) {
		self.get_field(state, GLOBALS_INDEX, name);
	}

	pub fn push_globals(&self, state: *mut LuaState) {
		self.push_value(state, GLOBALS_INDEX);
	}

	pub fn reference(&self, state: *mut LuaState) -> Option<c_int> {
		match self._reference(state, REGISTRY_INDEX) {
			LUA_REFNIL | LUA_NOREF => None,
			ref_id => Some(ref_id),
		}
	}

	pub fn dereference(&self, state: *mut LuaState, reference: c_int) -> Result<(), RegistryDerefError> {
		self.get_registry(state, reference);

		let ty = self.type_id(state, -1);
		self.pop(state, 1);

		if ty == LuaTypeId::Nil {
			return Err(RegistryDerefError::InvalidReference);
		}

		self._dereference(state, REGISTRY_INDEX, reference);
		Ok(())
	}

	pub fn get_registry(&self, state: *mut LuaState, key: impl IntoLua) {
		key.into_lua(self, state);
		self.rawget(state, REGISTRY_INDEX);
	}

	pub fn set_registry(&self, state: *mut LuaState, key: impl IntoLua) {
		key.into_lua(self, state);
		self.push_value(state, -2);
		self.rawset(state, REGISTRY_INDEX);
	}

	pub fn push_function(&self, state: *mut LuaState, func: extern "C-unwind" fn(*mut LuaState) -> c_int) {
		self.push_closure(state, func, 0);
	}

	pub fn push_bool(&self, state: *mut LuaState, b: bool) {
		self._push_boolean(state, b as _);
	}

	pub fn to_bool(&self, state: *mut LuaState, index: c_int) -> bool {
		self._to_boolean(state, index) != 0
	}

	pub fn get_info(&self, state: *mut LuaState, level: c_int, what: &CStr) -> Option<DebugInfo> {
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };

		if self.get_stack(state, level, &raw mut debug_info as _) == 0 {
			return None;
		}

		if self._get_info(state, what.as_ptr(), &raw mut debug_info as _) == 0 {
			return None;
		}

		Some(debug_info)
	}

	pub fn type_id(&self, state: *mut LuaState, index: c_int) -> LuaTypeId {
		let raw_type_id = self._type_id(state, index);
		match raw_type_id {
			-1 => LuaTypeId::None,
			0 => LuaTypeId::Nil,
			1 => LuaTypeId::Boolean,
			2 => LuaTypeId::LightUserdata,
			3 => LuaTypeId::Number,
			4 => LuaTypeId::String,
			5 => LuaTypeId::Table,
			6 => LuaTypeId::Function,
			7 => LuaTypeId::Userdata,
			8 => LuaTypeId::Thread,
			_ => unreachable!("Invalid Lua type id: {}", raw_type_id),
		}
	}

	pub fn type_name(&self, state: *mut LuaState, type_: c_int) -> Option<&std::ffi::CStr> {
		let c_str = self._type_name(state, type_);
		if c_str.is_null() {
			None
		} else {
			Some(unsafe { std::ffi::CStr::from_ptr(c_str) })
		}
	}

	pub fn to_string(&self, state: *mut LuaState, index: c_int) -> Option<std::borrow::Cow<'static, str>> {
		let mut len: c_uint = 0;
		let c_str = self.to_lstring(state, index, &mut len as *mut c_uint);
		if c_str.is_null() {
			None
		} else {
			Some(unsafe { std::ffi::CStr::from_ptr(c_str) }.to_string_lossy())
		}
	}

	pub fn check_string(&self, state: *mut LuaState, index: c_int) -> std::borrow::Cow<'static, str> {
		let c_str = self.check_lstring(state, index, std::ptr::null_mut());
		unsafe { std::ffi::CStr::from_ptr(c_str) }.to_string_lossy()
	}

	pub fn load_string(&self, state: *mut LuaState, s: *const c_char) -> Result<(), std::borrow::Cow<'static, str>> {
		match self._load_string(state, s) {
			LUA_OK | LUA_YIELD => Ok(()),
			_ => {
				let err_msg = self.to_lstring(state, -1, std::ptr::null_mut());
				self.pop(state, 1);

				let err_str = if !err_msg.is_null() {
					unsafe { std::ffi::CStr::from_ptr(err_msg) }.to_string_lossy()
				} else {
					std::borrow::Cow::Borrowed("Unknown error")
				};

				Err(err_str)
			}
		}
	}

	pub fn set_fenv(&self, state: *mut LuaState, index: c_int) -> Result<(), SetfenvError> {
		if self._set_fenv(state, index) != 0 {
			Ok(())
		} else {
			Err(SetfenvError::GenericFail)
		}
	}

	pub fn is_raw_equal(&self, state: *mut LuaState, index1: c_int, index2: c_int) -> bool {
		self._rawequal(state, index1, index2) != 0
	}

	pub fn is_equal(&self, state: *mut LuaState, index1: c_int, index2: c_int) -> bool {
		self._equal(state, index1, index2) != 0
	}

	pub fn load_buffer_x(
		&self,
		state: *mut LuaState,
		buff: &[u8],
		name: &CStr,
		mode: &CStr,
	) -> Result<(), std::borrow::Cow<'static, str>> {
		match self._load_buffer_x(state, buff.as_ptr() as _, buff.len(), name.as_ptr(), mode.as_ptr()) {
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err_msg = self.to_lstring(state, -1, std::ptr::null_mut());
				self.pop(state, 1);

				let err_str = if !err_msg.is_null() {
					unsafe { std::ffi::CStr::from_ptr(err_msg) }.to_string_lossy()
				} else {
					std::borrow::Cow::Borrowed("Unknown error")
				};

				Err(err_str)
			}
		}
	}

	pub fn pcall(
		&self,
		state: *mut LuaState,
		n_args: c_int,
		n_results: c_int,
		err_func: c_int,
	) -> Result<(), std::borrow::Cow<'static, str>> {
		match self._pcall(state, n_args, n_results, err_func) {
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err_msg = self.to_lstring(state, -1, std::ptr::null_mut());
				self.pop(state, 1);

				let err_str = if !err_msg.is_null() {
					unsafe { std::ffi::CStr::from_ptr(err_msg) }.to_string_lossy()
				} else {
					std::borrow::Cow::Borrowed("Unknown error")
				};

				Err(err_str)
			}
		}
	}

	pub fn new_userdata<T: Sized>(&self, state: *mut LuaState, init: T) -> &mut T {
		let ptr = self._new_userdata(state, core::mem::size_of::<T>()) as *mut T;
		unsafe {
			ptr.write(init);
			&mut *ptr
		}
	}

	pub fn pop(&self, state: *mut LuaState, n: c_int) {
		self.set_top(state, -n - 1);
	}

	pub fn push<T: IntoLua>(&self, state: *mut LuaState, value: T) {
		T::into_lua(value, self, state);
	}

	pub fn to<T: FromLua>(&self, state: *mut LuaState, stack_idx: c_int) -> T {
		T::from_lua(self, state, stack_idx)
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
					lua.error(state);
				}
			}
		}
		lua_wrapper as $crate::LuaFunction
	}};
}
