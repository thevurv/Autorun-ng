use core::ffi::{c_int, c_void};

pub type LuaState = c_void;
pub type LuaCFunction = extern "C-unwind" fn(state: *mut LuaState) -> c_int;
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

impl core::fmt::Display for LuaTypeId {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			LuaTypeId::None => write!(f, "None"),
			LuaTypeId::Nil => write!(f, "Nil"),
			LuaTypeId::Boolean => write!(f, "Boolean"),
			LuaTypeId::LightUserdata => write!(f, "LightUserdata"),
			LuaTypeId::Number => write!(f, "Number"),
			LuaTypeId::String => write!(f, "String"),
			LuaTypeId::Table => write!(f, "Table"),
			LuaTypeId::Function => write!(f, "Function"),
			LuaTypeId::Userdata => write!(f, "Userdata"),
			LuaTypeId::Thread => write!(f, "Thread"),
		}
	}
}
