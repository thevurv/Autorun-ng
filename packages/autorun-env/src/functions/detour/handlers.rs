use crate::functions::detour::raw;
use autorun_log::*;
use autorun_lua::{LUA_MULTRET, LuaApi, LuaFunction, RawHandle};
use autorun_types::LuaState;

pub extern "C-unwind" fn retour_handler(state: *mut LuaState, detour: *const retour::GenericDetour<LuaFunction>) -> i32 {
	unsafe { (*detour).call(state) }
}

pub extern "C-unwind" fn detour_handler(
	state: *mut LuaState,
	metadata: i32,
	lua_api: *const LuaApi,
	original_function: *const LuaFunction,
) -> i32 {
	let detour_metadata = raw::DetourMetadata::from_packed(metadata);
	let callback_id = detour_metadata.callback_ref();
	let lua = unsafe { &*lua_api };

	let callback_handle = RawHandle::from_id(callback_id);
	callback_handle.push(lua, state);
	lua.insert(state, 1);

	let num_arguments = lua.get_top(state) - 1;

	let original_function_included = if original_function as usize != 0 {
		// add the original function as the first argument
		unsafe {
			lua.push_function(state, *original_function);
			lua.insert(state, 2);
		}

		true
	} else {
		false
	};

	let num_arguments = if original_function_included {
		num_arguments + 1
	} else {
		num_arguments
	};

	let base = lua.get_top(state) - num_arguments;

	if let Err(why) = lua.pcall(state, num_arguments, LUA_MULTRET, 0) {
		error!("Error calling detour callback: {why}");
		return 0;
	}

	lua.get_top(state) - base + 1
}
