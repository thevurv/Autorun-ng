use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};

use autorun_types::LuaState;

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
	create_lua_interface: extern "C" fn(this: *const c_void, realm: c_uchar, whatever: bool) -> *mut ILuaInterface,
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetStateError {
	#[error("Failed to get interface: {0}")]
	GetInterface(#[from] crate::util::GetInterfaceError),
}

// todo: maybe make a get_api to get access to other functions which may be useful

pub fn get_state(realm: autorun_types::Realm) -> Result<Option<*mut LuaState>, GetStateError> {
	let lua_shared_003 = crate::util::get_interface("lua_shared_client.so", c"LUASHARED003")? as *mut ILuaShared;
	let lua_shared_003 = unsafe { lua_shared_003.as_mut().unwrap() };

	let vtable = lua_shared_003.vtable;
	let vtable = unsafe { vtable.as_ref().unwrap() };

	let menu = (vtable.get_lua_interface)(
		lua_shared_003 as *const _ as _,
		match realm {
			autorun_types::Realm::Client => STATE_CLIENT,
			autorun_types::Realm::Menu => STATE_MENU,
		},
	);

	if !menu.is_null() {
		let menu = unsafe { menu.as_mut().unwrap() };
		return Ok(Some(menu.state));
	}

	Ok(None)
}
