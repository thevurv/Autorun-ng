use autorun_lua::{LuaApi, LuaFunction, RawHandle, LUA_MULTRET};
use autorun_types::LuaState;
use crate::functions::detour::raw;

pub extern "C-unwind" fn retour_handler(state: *mut LuaState, detour: *const retour::GenericDetour<LuaFunction>) -> i32 {
    dbg!("retour handler called");

    unsafe {
        (*detour).call(state)
    }
}

pub extern "C-unwind" fn detour_handler(state: *mut LuaState, metadata: i32, lua_api: *const LuaApi, original_function: *const LuaFunction) -> i32 {
    let detour_metadata = raw::DetourMetadata::from_packed(metadata);
    let callback_id = detour_metadata.callback_ref();

    dbg!("detour handler called with callback id: {}", callback_id);
    let lua = unsafe { &*lua_api };

    let callback_handle = RawHandle::from_id(callback_id);
    callback_handle.push(lua, state);
    lua.insert(state, 1);

    let num_arguments = lua.get_top(state) - 1;
    dbg!("detour handler detected {} arguments", num_arguments);
    // add the original function as the first argument
    unsafe {
        lua.push_function(state, *original_function);
        lua.insert(state, 2);
    }

    let base = lua.get_top(state) - num_arguments - 1;
    if let Err(why) = lua.pcall(state, num_arguments + 1, LUA_MULTRET, 0) {
        dbg!("Error calling detour callback: {}", why);
        return 0;
    }

    let ret_count = lua.get_top(state) - base + 1; // account for the mystery off-by-one
    dbg!("detour handler returning {} values", ret_count);

    ret_count
}