// todo: completely redo these
#[derive(Debug)]
#[non_exhaustive]
pub enum LuaError {
	Runtime(String),
	InvalidReference,
	GenericFailure,
}

impl core::fmt::Display for LuaError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			LuaError::InvalidReference => write!(f, "Lua Invalid Reference"),
			LuaError::Runtime(msg) => write!(f, "Lua Error: {}", msg),
			LuaError::GenericFailure => write!(f, "Lua Generic Failure"),
		}
	}
}

impl core::error::Error for LuaError {}

pub type LuaResult<T> = Result<T, LuaError>;
