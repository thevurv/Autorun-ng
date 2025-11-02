use crate::{LuaApi, LuaState, LuaTypeId};

pub fn dump_stack(lua: &LuaApi, state: *mut LuaState) {
	let top = lua.get_top(state);
	for i in 1..=top {
		let type_id = lua.type_id(state, i);
		let value = match type_id {
			LuaTypeId::Nil => "nil".to_string(),
			LuaTypeId::Boolean => {
				let b = lua.to_bool(state, i);
				format!("boolean: {}", b)
			}
			LuaTypeId::Number => {
				let n = lua.to_number(state, i);
				format!("number: {}", n)
			}
			LuaTypeId::String => {
				let s = lua.check_string(state, i);
				format!("string: {}", s)
			}
			LuaTypeId::Table => "table".to_string(),
			LuaTypeId::Function => "function".to_string(),
			LuaTypeId::Userdata => "userdata".to_string(),
			LuaTypeId::Thread => "thread".to_string(),
			LuaTypeId::LightUserdata => "lightuserdata".to_string(),
			_ => "unknown".to_string(),
		};

		println!("  [{}] - {}", i, value);
	}
}
