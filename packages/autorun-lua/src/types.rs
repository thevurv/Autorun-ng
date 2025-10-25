use core::ffi::{c_int, c_void};

pub type LuaState = c_void;
pub type LuaFunction = extern "C-unwind" fn(state: *mut LuaState) -> c_int;
pub type LuaLightUserdata = *mut c_void;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaTypeId {
	None = -1,
	Nil = 0,
	Boolean = 1,
	LightUserdata = 2,
	Number = 3,
	String = 4,
	Table = 5,
	Function = 6,
	Userdata = 7,
	Thread = 8,
}
