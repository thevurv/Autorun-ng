use crate::functions::detour::raw;
use autorun_log::*;
use autorun_lua::{LUA_MULTRET, LuaApi, LuaCFunction, RawHandle};
use autorun_types::LuaState;

pub extern "C-unwind" fn retour_handler(state: *mut LuaState, detour: *const retour::GenericDetour<LuaCFunction>) -> i32 {
	unsafe { (*detour).call(state) }
}

pub extern "C-unwind" fn detour_handler(
	state: *mut LuaState,
	metadata: i32,
	lua_api: *const LuaApi,
	original_function: *const LuaCFunction,
) -> i32 {
	let detour_metadata = raw::DetourMetadata::from_packed(metadata);
	let callback_id = detour_metadata.callback_ref();
	let lua = unsafe { &*lua_api };

	let callback_handle = RawHandle::from_id(callback_id);
	callback_handle.push(lua, state);
	lua.raw.insert(state, 1);

	let num_arguments = lua.raw.gettop(state) - 1;

	let original_function_included = if original_function as usize != 0 {
		// add the original function as the first argument
		unsafe {
			lua.raw.pushcfunction(state, *original_function);
			lua.raw.insert(state, 2);
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

	let base = lua.raw.gettop(state) - num_arguments;

	if let Err(why) = lua.raw.pcall(state, num_arguments, LUA_MULTRET, 0) {
		error!("Error calling detour callback: {why}");
		return 0;
	}

	lua.raw.gettop(state) - base + 1
}
