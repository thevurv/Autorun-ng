use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};

type CreateInterfaceFn = extern "C" fn(name: *const c_char, return_code: *mut c_int) -> *mut c_void;
type LuaState = c_void;

pub fn get_interface(
	path: impl AsRef<std::ffi::OsStr>,
	interface: &str,
) -> anyhow::Result<*mut std::ffi::c_void> {
	let library =
		unsafe { libloading::Library::new(path.as_ref()) }.expect("Failed to load library");

	let factory = unsafe {
		library
			.get::<CreateInterfaceFn>(c"CreateInterface".to_bytes_with_nul())
			.expect("Failed to get CreateInterface function")
	};

	let interface_cstr = std::ffi::CString::new(interface)?;

	let mut return_code: c_int = 0;
	let interface_ptr = factory(interface_cstr.as_ptr(), &mut return_code as *mut c_int);

	if return_code != 0 {
		return Err(anyhow::anyhow!(
			"CreateInterface returned error code {} for interface {} from {:?}",
			return_code,
			interface,
			path.as_ref()
		));
	}

	std::mem::forget(library);

	if interface_ptr.is_null() {
		Err(anyhow::anyhow!(
			"Failed to get interface: {} from {:?}",
			interface,
			path.as_ref()
		))
	} else {
		Ok(interface_ptr)
	}
}

#[repr(C)]
struct ILuaShared {
	vtable: *const ILuaSharedVTable,
}

#[repr(C)]
struct ILuaSharedVTable {
	#[cfg(target_os = "linux")]
	rtti: *const c_void,

	destructor: *const c_void,
	init: *const c_void,
	shutdown: *const c_void,
	dump_stats: *const c_void,
	create_lua_interface:
		extern "C" fn(this: *const c_void, realm: c_uchar, whatever: bool) -> *mut ILuaInterface,
	close_lua_interface: *const c_void,
	get_lua_interface: extern "C" fn(this: *const c_void, realm: c_uchar) -> *mut ILuaInterface,
}

#[repr(C)]
struct ILuaInterface {
	vtable: *const ILuaInterfaceVTable,
	state: *mut LuaState,
}

#[repr(C)]
struct ILuaInterfaceVTable {
	base: ILuaBaseVTable,
	init: *const c_void,
	shutdown: *const c_void,
	cycle: *const c_void,
	global: *const c_void,
}

#[repr(C)]
struct ILuaBase {
	vtable: *const ILuaBaseVTable,
	state: *mut LuaState,
}

#[repr(C)]
struct ILuaBaseVTable {
	top: *const c_void,
	push: *const c_void,
	pop: *const c_void,
	get_table: *const c_void,
	_other: [*const c_void; 51],
}

#[repr(C)]
struct CLuaInterface {
	// Has no virtual methods of its own, so fine to use ILuaInterface directly
	vtable: *const ILuaInterfaceVTable,
	state: *mut c_void,
}

const STATE_CLIENT: c_uchar = 0;
const STATE_SERVER: c_uchar = 1;
const STATE_MENU: c_uchar = 2;

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

pub fn get_menu_state() -> anyhow::Result<Option<*mut LuaState>> {
	let lua_shared_003 = get_interface("lua_shared_client.so", "LUASHARED003")? as *mut ILuaShared;
	let lua_shared_003 = unsafe { lua_shared_003.as_mut().unwrap() };

	let vtable = lua_shared_003.vtable;
	let vtable = unsafe { vtable.as_ref().unwrap() };

	let menu = (vtable.get_lua_interface)(lua_shared_003 as *const _ as _, STATE_MENU);

	if !menu.is_null() {
		let menu = unsafe { menu.as_mut().unwrap() };
		return Ok(Some(menu.state));
	}

	Ok(None)
}
