use core::ffi::{c_int, c_void};

pub type LuaState = c_void;
pub type LuaFunction = extern "C-unwind" fn(state: *mut LuaState) -> c_int;
pub type LuaLightUserdata = *mut c_void;
