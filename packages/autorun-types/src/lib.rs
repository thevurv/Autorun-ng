use nanoserde::{DeBin, SerBin};

pub type LuaState = std::ffi::c_void;

#[derive(Debug, Clone, Copy, DeBin, SerBin, PartialEq)]
pub enum Realm {
	Menu,
	Client,
}

impl std::fmt::Display for Realm {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Realm::Menu => write!(f, "Menu"),
			Realm::Client => write!(f, "Client"),
		}
	}
}
