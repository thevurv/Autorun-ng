use core::ffi::{c_char, c_double, c_int, c_uint, c_void};
use std::ffi::CStr;

use crate::{IntoLua, LuaCFunction, LuaError, LuaResult, LuaState, LuaTypeId};

#[cfg(feature = "gmod")]
const LUA_IDSIZE: usize = 128;

#[cfg(not(feature = "gmod"))]
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
	pub nups: c_int,                     // u
	pub linedefined: c_int,              // S
	pub lastlinedefined: c_int,          // S
	pub short_src: [c_char; LUA_IDSIZE], // S
	pub i_ci: c_int,
}

pub const GLOBALS_INDEX: c_int = -10002;
pub const ENVIRON_INDEX: c_int = -10001;
pub const REGISTRY_INDEX: c_int = -10000;
pub const LUA_MULTRET: c_int = -1;

pub const LUA_OK: c_int = 0;
pub const LUA_YIELD: c_int = 1;
pub const LUA_ERRRUN: c_int = 2;
pub const LUA_ERRSYNTAX: c_int = 3;
pub const LUA_ERRMEM: c_int = 4;
pub const LUA_ERRGCMM: c_int = 5;
pub const LUA_ERRERR: c_int = 6;

pub const LUA_REFNIL: c_int = -1;
pub const LUA_NOREF: c_int = -2;

macro_rules! define_lua_api {
    (
        $(
            #[name = $name:literal]
            $vis:vis fn $fn_name:ident($($param_name:ident: $param_type:ty),* $(,)?) $(-> $return_type:ty)?;
        )*
    ) => {
        #[derive(Debug)]
        pub struct RawLuaApi {
            $(
                pub $fn_name: extern "C-unwind" fn($($param_name: $param_type),*) $(-> $return_type)?,
            )*
        }

        impl RawLuaApi {
            pub fn new(lua_shared: &libloading::Library) -> Result<Self, libloading::Error> {
                let api = Self {
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
	pub fn pushnumber(state: *mut LuaState, number: c_double);
	#[name = "lua_pushinteger"]
	pub fn pushinteger(state: *mut LuaState, integer: c_int);
	#[name = "lua_pushnil"]
	pub fn pushnil(state: *mut LuaState);
	#[name = "lua_pushstring"]
	pub fn pushstring(state: *mut LuaState, string: *const c_char);
	#[name = "lua_pushlstring"]
	pub fn pushlstring(state: *mut LuaState, string: *const c_char, len: usize);
	#[name = "lua_pushthread"]
	pub fn pushthread(state: *mut LuaState) -> c_int;
	#[name = "lua_pushvalue"]
	pub fn pushvalue(state: *mut LuaState, index: c_int);
	#[name = "lua_pushcclosure"]
	pub fn pushcclosure(state: *mut LuaState, func: LuaCFunction, nups: c_int);
	#[name = "lua_pushlightuserdata"]
	pub fn pushlightuserdata(state: *mut LuaState, p: *mut c_void);
	#[name = "lua_pushboolean"]
	fn _pushboolean(state: *mut LuaState, b: c_int);

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
	pub fn gettable(state: *mut LuaState, index: c_int);
	#[name = "lua_settable"]
	pub fn settable(state: *mut LuaState, index: c_int);
	#[name = "lua_getfield"]
	pub fn getfield(state: *mut LuaState, index: c_int, k: *const c_char);
	#[name = "lua_insert"]
	pub fn insert(state: *mut LuaState, index: c_int);

	#[name = "luaL_loadbufferx"]
	pub fn _loadbufferx(
		state: *mut LuaState,
		buff: *const c_char,
		size: usize,
		name: *const c_char,
		mode: *const c_char,
	) -> c_int;
	#[name = "luaL_loadstring"]
	pub fn _loadstring(state: *mut LuaState, str: *const c_char) -> c_int;
	#[name = "luaL_checknumber"]
	pub fn checknumber(state: *mut LuaState, index: c_int) -> c_double;
	#[name = "luaL_checklstring"]
	pub fn checklstring(state: *mut LuaState, index: c_int, len: *mut c_uint) -> *const c_char;

	#[name = "lua_call"]
	pub fn call(state: *mut LuaState, n_args: c_int, n_results: c_int) -> c_int;
	#[name = "lua_pcall"]
	pub fn _pcall(state: *mut LuaState, n_args: c_int, n_results: c_int, err_func: c_int) -> c_int;
	#[name = "lua_createtable"]
	pub fn createtable(state: *mut LuaState, narr: c_int, nrec: c_int);
	#[name = "lua_equal"]
	fn _equal(state: *mut LuaState, index1: c_int, index2: c_int) -> c_int;
	#[name = "lua_error"]
	pub fn error(state: *mut LuaState) -> !;
	#[name = "lua_gc"]
	pub fn gc(state: *mut LuaState, what: c_int, data: c_int) -> c_int;
	#[name = "lua_settop"]
	pub fn settop(state: *mut LuaState, index: c_int);
	#[name = "lua_gettop"]
	pub fn gettop(state: *mut LuaState) -> c_int;
	#[name = "lua_remove"]
	pub fn remove(state: *mut LuaState, index: c_int);
	#[name = "lua_status"]
	pub fn status(state: *mut LuaState) -> c_int;
	#[name = "lua_type"]
	pub fn _typeid(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_typename"]
	pub fn _typename(state: *mut LuaState, type_: c_int) -> *const c_char;

	#[name = "lua_toboolean"]
	fn _toboolean(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_tonumber"]
	pub fn tonumber(state: *mut LuaState, index: c_int) -> c_double;
	#[name = "lua_topointer"]
	pub fn topointer(state: *mut LuaState, index: c_int) -> *const c_void;
	#[name = "lua_tolstring"]
	pub fn tolstring(state: *mut LuaState, index: c_int, len: *mut c_uint) -> *const c_char;
	#[name = "lua_tothread"]
	pub fn tothread(state: *mut LuaState, index: c_int) -> *mut LuaState;
	#[name = "lua_touserdata"]
	pub fn touserdata(state: *mut LuaState, index: c_int) -> *mut c_void;
	#[name = "lua_tointeger"]
	pub fn tointeger(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_tocfunction"]
	pub fn tocfunction(state: *mut LuaState, index: c_int) -> Option<LuaCFunction>;

	#[name = "lua_xmove"]
	pub fn xmove(from: *mut LuaState, to: *mut LuaState, n: c_int);
	#[name = "lua_iscfunction"]
	fn _iscfunction(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isnumber"]
	fn _isnumber(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isstring"]
	fn _isstring(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_isuserdata"]
	fn _isuserdata(state: *mut LuaState, index: c_int) -> c_int;

	// Debug library
	#[name = "lua_gethook"]
	pub fn gethook(state: *mut LuaState) -> *const c_void;
	#[name = "lua_gethookcount"]
	pub fn gethookcount(state: *mut LuaState) -> c_int;
	#[name = "lua_gethookmask"]
	pub fn gethookmask(state: *mut LuaState) -> c_int;
	#[name = "lua_getinfo"]
	fn _getinfo(state: *mut LuaState, what: *const c_char, ar: *mut c_void) -> c_int;
	#[name = "lua_getlocal"]
	pub fn getlocal(state: *mut LuaState, ar: *mut c_void, n: c_int) -> *const c_char;
	#[name = "lua_getstack"]
	pub fn getstack(state: *mut LuaState, level: c_int, ar: *mut c_void) -> c_int;
	#[name = "lua_getupvalue"]
	pub fn getupvalue(state: *mut LuaState, funcindex: c_int, n: c_int) -> *const c_char;
	#[name = "lua_sethook"]
	pub fn sethook(state: *mut LuaState, func: *const c_void, mask: c_int, count: c_int);
	#[name = "lua_setlocal"]
	pub fn setlocal(state: *mut LuaState, ar: *mut c_void, n: c_int) -> *const c_char;
	#[name = "lua_setupvalue"]
	pub fn setupvalue(state: *mut LuaState, funcindex: c_int, n: c_int) -> *const c_char;

	#[name = "lua_getfenv"]
	pub fn getfenv(state: *mut LuaState, index: c_int);
	#[name = "lua_setfenv"]
	pub fn _setfenv(state: *mut LuaState, index: c_int) -> c_int;

	#[name = "luaL_ref"]
	pub fn _reference(state: *mut LuaState, t: c_int) -> c_int;
	#[name = "luaL_unref"]
	pub fn _unreference(state: *mut LuaState, t: c_int, r: c_int);

	#[name = "lua_getmetatable"]
	pub fn getmetatable(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_setmetatable"]
	pub fn setmetatable(state: *mut LuaState, index: c_int) -> c_int;

	#[name = "lua_newuserdata"]
	fn _newuserdata(state: *mut LuaState, size: usize) -> *mut c_void;
}

impl RawLuaApi {
	pub fn pushcfunction(&self, state: *mut LuaState, func: LuaCFunction) {
		self.pushcclosure(state, func, 0);
	}

	pub fn pushboolean(&self, state: *mut LuaState, b: bool) {
		self._pushboolean(state, b as _);
	}

	pub fn toboolean(&self, state: *mut LuaState, index: c_int) -> bool {
		self._toboolean(state, index) != 0
	}

	pub fn getinfo(&self, state: *mut LuaState, level: c_int, what: &CStr) -> Option<DebugInfo> {
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };

		if self.getstack(state, level, &raw mut debug_info as _) == 0 {
			return None;
		}

		if self._getinfo(state, what.as_ptr(), &raw mut debug_info as _) == 0 {
			return None;
		}

		Some(debug_info)
	}

	pub fn pop(&self, state: *mut LuaState, n: c_int) {
		self.settop(state, -n - 1);
	}

	pub fn rawequal(&self, state: *mut LuaState, index1: c_int, index2: c_int) -> bool {
		self._rawequal(state, index1, index2) != 0
	}

	pub fn equal(&self, state: *mut LuaState, index1: c_int, index2: c_int) -> bool {
		self._equal(state, index1, index2) != 0
	}

	pub fn checkstring(&self, state: *mut LuaState, index: c_int) -> std::borrow::Cow<'static, str> {
		let c_str = self.checklstring(state, index, std::ptr::null_mut());
		unsafe { std::ffi::CStr::from_ptr(c_str) }.to_string_lossy()
	}

	pub fn newuserdata<T: Sized>(&self, state: *mut LuaState, init: T) -> *mut T {
		let ptr = self._newuserdata(state, core::mem::size_of::<T>()) as *mut T;
		unsafe {
			ptr.write(init);
		}
		ptr
	}

	pub fn tostring(&self, state: *mut LuaState, index: c_int) -> Option<std::borrow::Cow<'static, str>> {
		let mut len: c_uint = 0;
		let c_str = self.tolstring(state, index, &mut len as *mut c_uint);
		if c_str.is_null() {
			None
		} else {
			Some(unsafe { std::ffi::CStr::from_ptr(c_str) }.to_string_lossy())
		}
	}

	pub fn typeid(&self, state: *mut LuaState, index: c_int) -> LuaTypeId {
		let raw_type_id = self._typeid(state, index);
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

	pub fn typename(&self, state: *mut LuaState, type_: c_int) -> Option<&std::ffi::CStr> {
		let c_str = self._typename(state, type_);
		if c_str.is_null() {
			None
		} else {
			Some(unsafe { std::ffi::CStr::from_ptr(c_str) })
		}
	}

	pub fn iscfunction(&self, state: *mut LuaState, index: c_int) -> bool {
		self._iscfunction(state, index) != 0
	}

	pub fn isnumber(&self, state: *mut LuaState, index: c_int) -> bool {
		self._isnumber(state, index) != 0
	}

	pub fn isstring(&self, state: *mut LuaState, index: c_int) -> bool {
		self._isstring(state, index) != 0
	}

	pub fn isuserdata(&self, state: *mut LuaState, index: c_int) -> bool {
		self._isuserdata(state, index) != 0
	}

	pub fn reference(&self, state: *mut LuaState) -> Option<c_int> {
		match self._reference(state, REGISTRY_INDEX) {
			LUA_REFNIL | LUA_NOREF => None,
			ref_id => Some(ref_id),
		}
	}

	pub fn unreference(&self, state: *mut LuaState, reference: c_int) -> LuaResult<()> {
		self.rawgeti(state, REGISTRY_INDEX, reference);

		let ty = self.typeid(state, -1);
		self.pop(state, 1);

		if ty == LuaTypeId::Nil {
			return Err(LuaError::InvalidReference);
		}

		self._unreference(state, REGISTRY_INDEX, reference);
		Ok(())
	}

	pub fn loadstring(&self, state: *mut LuaState, s: *const c_char) -> Result<(), std::borrow::Cow<'static, str>> {
		match self._loadstring(state, s) {
			LUA_OK | LUA_YIELD => Ok(()),
			_ => {
				let err_msg = self.tolstring(state, -1, std::ptr::null_mut());
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

	pub fn loadbufferx(&self, state: *mut LuaState, buff: &[u8], name: &CStr, mode: &CStr) -> LuaResult<()> {
		match self._loadbufferx(state, buff.as_ptr() as _, buff.len(), name.as_ptr(), mode.as_ptr()) {
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err = self.checkstring(state, -1);
				self.pop(state, 1);

				Err(LuaError::Runtime(err.to_string()))
			}
		}
	}

	pub fn pcall(&self, state: *mut LuaState, n_args: c_int, n_results: c_int, err_func: c_int) -> LuaResult<()> {
		match self._pcall(state, n_args, n_results, err_func) {
			LUA_OK | LUA_YIELD => Ok(()),

			_ => {
				let err = self.checkstring(state, -1);
				self.pop(state, 1);

				Err(LuaError::Runtime(err.to_string()))
			}
		}
	}

	pub fn setfenv(&self, state: *mut LuaState, index: c_int) -> LuaResult<()> {
		if self._setfenv(state, index) != 0 {
			Ok(())
		} else {
			Err(LuaError::GenericFailure)
		}
	}
}
