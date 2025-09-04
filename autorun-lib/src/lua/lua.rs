use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};
pub type LuaState = c_void;

const LUA_GLOBALSINDEX: c_int = -10002;

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
                $fn_name: extern "C" fn($($param_name: $param_type),*) $(-> $return_type)?,
            )*
        }

        impl LuaApi {
            fn new(lua_shared: &libloading::Library) -> anyhow::Result<LuaApi> {
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
	#[name = "lua_pushnil"]
	pub fn push_nil(state: *mut LuaState);
	#[name = "lua_pushstring"]
	pub fn push_string(state: *mut LuaState, string: *const c_char);
	#[name = "lua_pushthread"]
	pub fn push_thread(state: *mut LuaState) -> c_int;
	#[name = "lua_pushvalue"]
	pub fn push_value(state: *mut LuaState, index: c_int);

	#[name = "lua_rawequal"]
	pub fn is_raw_equal(state: *mut LuaState, index1: c_int, index2: c_int) -> c_int;
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

	#[name = "luaL_loadstring"]
	pub fn load_string(state: *mut LuaState, str: *const c_char) -> c_int;
	#[name = "luaL_checknumber"]
	pub fn check_number(state: *mut LuaState, index: c_int) -> c_double;
	#[name = "luaL_checklstring"]
	pub fn check_lstring(state: *mut LuaState, index: c_int, len: *mut c_uint) -> *const c_char;

	#[name = "lua_call"]
	pub fn call(state: *mut LuaState, n_args: c_int, n_results: c_int) -> c_int;
	#[name = "lua_pcall"]
	pub fn pcall(state: *mut LuaState, n_args: c_int, n_results: c_int, err_func: c_int) -> c_int;
	#[name = "lua_createtable"]
	pub fn create_table(state: *mut LuaState, narr: c_int, nrec: c_int);
	#[name = "lua_equal"]
	pub fn is_equal(state: *mut LuaState, index1: c_int, index2: c_int) -> c_int;
	#[name = "lua_error"]
	pub fn error(state: *mut LuaState) -> !;
	#[name = "lua_gc"]
	pub fn gc(state: *mut LuaState, what: c_int, data: c_int) -> c_int;
	#[name = "lua_settop"]
	pub fn set_top(state: *mut LuaState, index: c_int);
	#[name = "lua_gettop"]
	pub fn get_top(state: *mut LuaState) -> c_int;
	#[name = "lua_status"]
	pub fn status(state: *mut LuaState) -> c_int;

	#[name = "lua_toboolean"]
	pub fn to_boolean(state: *mut LuaState, index: c_int) -> c_uchar;
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

	#[name = "lua_type"]
	pub fn get_type(state: *mut LuaState, index: c_int) -> c_int;
	#[name = "lua_typename"]
	pub fn get_type_name(state: *mut LuaState, type_: c_int) -> *const c_char;
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
	pub fn get_info(state: *mut LuaState, what: *const c_char, ar: *mut c_void) -> c_int;
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
}

static LUA_API: std::sync::OnceLock<LuaApi> = std::sync::OnceLock::new();

impl LuaApi {
	pub fn get_global(&self, state: *mut LuaState, name: *const c_char) {
		self.get_field(state, LUA_GLOBALSINDEX, name);
	}
}

pub fn get_api() -> anyhow::Result<&'static LuaApi> {
	if let Some(api) = LUA_API.get() {
		return Ok(api);
	}

	let lua_shared = unsafe { libloading::Library::new("lua_shared_client.so") }?;

	// Generated by macro.
	let api = LuaApi::new(&lua_shared)?;

	std::mem::forget(lua_shared);

	LUA_API
		.set(api)
		.expect("Should never already be initialized");

	Ok(LUA_API.get().expect("Should be initialized"))
}
