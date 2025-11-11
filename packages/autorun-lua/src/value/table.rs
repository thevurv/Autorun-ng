use crate::{FromLua, Globals, IntoLua, LuaApi, LuaState, LuaTypeId, RawHandle};

#[derive(Debug, Clone, Copy)]
pub struct LuaTable {
	handle: RawHandle,
}

impl LuaApi {
	pub fn globals(&self, state: *mut LuaState) -> LuaTable {
		self.push(state, Globals);
		let handle = RawHandle::from_stack(self, state).expect("Failed to allocate registry value");
		LuaTable { handle }
	}

	pub fn table(&self, state: *mut LuaState) -> LuaTable {
		self.raw.createtable(state, 0, 0);
		let handle = RawHandle::from_stack(self, state).expect("Failed to allocate registry value");
		LuaTable { handle }
	}

	pub fn set(&self, state: *mut LuaState, table: &LuaTable, key: impl IntoLua, value: impl IntoLua) {
		table.handle.push(self, state);
		key.into_lua(self, state);
		value.into_lua(self, state);
		self.raw.settable(state, -3);
		self.raw.pop(state, 1);
	}

	pub fn get<T: FromLua>(&self, state: *mut LuaState, table: &LuaTable, key: impl IntoLua) -> T {
		table.handle.push(self, state);
		key.into_lua(self, state);
		self.raw.gettable(state, -2);
		let value = T::from_lua(self, state, -1);
		self.raw.pop(state, 2);
		value
	}
}

impl IntoLua for &LuaTable {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		self.handle.push(lua, state);
	}
}

impl FromLua for LuaTable {
	fn from_lua(lua: &LuaApi, state: *mut LuaState, index: i32) -> Self {
		assert_eq!(lua.raw.typeid(state, index), LuaTypeId::Table, "Value was not a LuaTable");

		lua.raw.pushvalue(state, index);
		let handle = RawHandle::from_stack(lua, state).expect("Failed to allocate registry value");
		LuaTable { handle }
	}
}
