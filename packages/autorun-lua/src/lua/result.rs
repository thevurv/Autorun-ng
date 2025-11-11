// todo: completely redo these
#[derive(Debug)]
#[non_exhaustive]
pub enum LuaError {
	Runtime(String),
	TypeMismatch {
		expected: crate::LuaTypeId,
		found: crate::LuaTypeId,
	},
	InvalidReference,
	GenericFailure,
}

impl LuaError {
	pub fn mismatch(expected: crate::LuaTypeId, found: crate::LuaTypeId) -> Self {
		LuaError::TypeMismatch { expected, found }
	}
}

impl core::fmt::Display for LuaError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			LuaError::InvalidReference => write!(f, "Invalid Reference"),
			LuaError::Runtime(msg) => write!(f, "Runtime Error: {}", msg),
			LuaError::GenericFailure => write!(f, "Generic Failure"),

			LuaError::TypeMismatch { expected, found } => {
				write!(f, "Type Mismatch: expected {expected}, found {found}")
			}
		}
	}
}

impl core::error::Error for LuaError {}

pub type LuaResult<T> = Result<T, LuaError>;
