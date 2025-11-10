use crate::{IntoLua, LuaApi, LuaState};

pub trait LuaArgs {
	fn len(&self) -> usize;
}

impl LuaArgs for () {
	fn len(&self) -> usize {
		0
	}
}

pub trait LuaFunction<R> {
	fn call(&self, lua: &LuaApi, state: *mut LuaState, args: LuaArgs) -> Result<R, Box<dyn std::error::Error>>;
}
