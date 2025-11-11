use crate::{IntoLua, LuaApi, LuaError, LuaResult, LuaState, LuaTypeId, LuaValue, RawHandle, RawLuaApi, TryFromLua};

#[derive(Debug, Clone, Copy)]
pub struct LuaFunction {
	handle: RawHandle,
}

impl LuaFunction {
	pub(crate) fn from_raw(handle: RawHandle) -> Self {
		Self { handle }
	}
}

impl LuaApi {
	pub fn call<A: IntoLuaArgs>(&self, state: *mut LuaState, f: &LuaFunction, args: A) -> Vec<LuaValue<'_>> {
		let top_before = self.raw.gettop(state);

		self.raw.push(state, &f.handle);

		let nargs = args.push_args(&self.raw, state);

		self.raw.call(state, nargs, crate::LUA_MULTRET);

		let top_after = self.raw.gettop(state);
		let nresults = top_after - top_before;

		let mut results = Vec::with_capacity(nresults as usize);
		for i in 0..nresults {
			results.push(self.raw.to(state, top_before + 1 + i));
		}

		self.raw.settop(state, top_before);

		results
	}

	pub fn pcall(&self, state: *mut LuaState, f: &LuaFunction, args: impl IntoLuaArgs) -> LuaResult<Vec<LuaValue<'_>>> {
		let top_before = self.raw.gettop(state);

		self.raw.push(state, &f.handle);

		let nargs = args.push_args(&self.raw, state);

		self.raw.pcall(state, nargs, crate::LUA_MULTRET, 0)?;

		let top_after = self.raw.gettop(state);
		let nresults = top_after - top_before;

		let mut results = Vec::with_capacity(nresults as usize);
		for i in 0..nresults {
			results.push(self.raw.to(state, top_before + 1 + i));
		}

		self.raw.settop(state, top_before);

		Ok(results)
	}
}

pub trait IntoLuaArgs {
	fn push_args(self, lua: &RawLuaApi, state: *mut LuaState) -> i32;
}

impl IntoLuaArgs for () {
	fn push_args(self, _lua: &RawLuaApi, _state: *mut LuaState) -> i32 {
		0
	}
}

macro_rules! impl_into_lua_args {
	($($T:ident),*) => {
		impl<$($T: IntoLua),*> IntoLuaArgs for ($($T,)*) {
			#[allow(non_snake_case)]
			fn push_args(self, lua: &RawLuaApi, state: *mut LuaState) -> i32 {
				let ($($T,)*) = self;
				let mut count = 0;
				$(
					lua.push(state, $T);
					count += 1;
				)*
				count
			}
		}
	};
}

impl_into_lua_args!(T1);
impl_into_lua_args!(T1, T2);
impl_into_lua_args!(T1, T2, T3);
impl_into_lua_args!(T1, T2, T3, T4);
impl_into_lua_args!(T1, T2, T3, T4, T5);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_into_lua_args!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

impl IntoLua for &LuaFunction {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		lua.push(state, &self.handle);
	}
}

impl TryFromLua for LuaFunction {
	fn try_from_lua(lua: &RawLuaApi, state: *mut LuaState, index: i32) -> LuaResult<Self> {
		match lua.try_to(state, index)? {
			LuaValue::Function(f) => Ok(f),
			other => Err(LuaError::mismatch(LuaTypeId::Function, other.typeid())),
		}
	}
}
