use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};

use crate::lua;

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

#[link(name = "lua_shared_client.so")]
extern "C" {
	fn lua_getglobal(L: *mut LuaState, name: *const c_char) -> c_int;
	fn lua_getfield(L: *mut LuaState, idx: c_int, k: *const c_char) -> c_int;
	fn lua_pcall(L: *mut LuaState, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int;
	fn lua_pushstring(L: *mut LuaState, s: *const c_char);
	fn lua_pushnumber(L: *mut LuaState, n: c_double);
	fn lua_pushboolean(L: *mut LuaState, b: c_int);
	fn lua_tostring(L: *mut LuaState, idx: c_int) -> *const c_char;
	fn lua_tonumber(L: *mut LuaState, idx: c_int) -> c_double;
	fn lua_toboolean(L: *mut LuaState, idx: c_int) -> c_int;
	fn lua_type(L: *mut LuaState, idx: c_int) -> c_int;
	fn lua_gettop(L: *mut LuaState) -> c_int;
	fn lua_settop(L: *mut LuaState, idx: c_int);
	fn lua_isfunction(L: *mut LuaState, idx: c_int) -> c_int;
	fn lua_isnil(L: *mut LuaState, idx: c_int) -> c_int;
}

const LUA_GLOBALSINDEX: c_int = -10002;

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
