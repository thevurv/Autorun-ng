use crate::{FromLua, Globals, IntoLua, LuaApi, LuaResult, LuaState, RawHandle, RawLuaApi, TryFromLua};

#[derive(Debug, Clone, Copy)]
pub struct LuaTable {
	handle: RawHandle,
}

impl LuaTable {
	pub(crate) fn from_raw(handle: RawHandle) -> LuaTable {
		LuaTable { handle }
	}
}

impl LuaApi {
	pub fn globals(&self, state: *mut LuaState) -> LuaTable {
		self.raw.push(state, Globals);
		let handle = RawHandle::from_stack(&self.raw, state).expect("Failed to allocate registry value");
		LuaTable { handle }
	}

	pub fn table(&self, state: *mut LuaState) -> LuaTable {
		self.raw.createtable(state, 0, 0);
		let handle = RawHandle::from_stack(&self.raw, state).expect("Failed to allocate registry value");
		LuaTable { handle }
	}

	pub fn set(&self, state: *mut LuaState, table: &LuaTable, key: impl IntoLua, value: impl IntoLua) {
		self.raw.push(state, &table.handle);
		self.raw.push(state, key);
		self.raw.push(state, value);
		self.raw.settable(state, -3);
		self.raw.pop(state, 1);
	}

	pub fn get<T: TryFromLua>(&self, state: *mut LuaState, table: &LuaTable, key: impl IntoLua) -> LuaResult<T> {
		self.raw.push(state, &table.handle);
		self.raw.push(state, key);
		self.raw.gettable(state, -2);
		let value = self.raw.try_to::<T>(state, -1);
		self.raw.pop(state, 2);
		value
	}
}

impl IntoLua for &LuaTable {
	fn into_lua(self, lua: &RawLuaApi, state: *mut LuaState) {
		lua.push(state, &self.handle);
	}
}
