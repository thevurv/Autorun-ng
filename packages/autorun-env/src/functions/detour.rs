mod raw;

use retour::GenericDetour;
use autorun_lua::{IntoLua, LuaApi, LuaFunction, LuaTypeId, RawHandle, LUA_MULTRET};
use autorun_types::LuaState;
use crate::functions::detour::raw::{CallbackTrampoline, RetourLuaTrampoline};

pub struct Detour {
    detour: Box<retour::GenericDetour<LuaFunction>>,
    detour_callback: RawHandle,
    callback_trampoline: CallbackTrampoline,
    retour_trampoline: RetourLuaTrampoline,
}

impl IntoLua for Detour {
    fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
        lua.new_userdata(state, self);
    }
}

extern "C-unwind" fn retour_handler(state: *mut LuaState, detour: *const retour::GenericDetour<LuaFunction>) -> i32 {
    dbg!("retour handler called");

    unsafe {
        (*detour).call(state)
    }
}

extern "C-unwind" fn detour_handler(state: *mut LuaState, metadata: i32, lua_api: *const LuaApi, original_function: *const LuaFunction) -> i32 {
    let detour_metadata = raw::DetourMetadata::from_packed(metadata);
    let callback_id = detour_metadata.callback_ref();
    let num_arguments = detour_metadata.num_arguments();

    dbg!("detour handler called with callback id: {}", callback_id);
    let lua = unsafe { &*lua_api };

    let callback_handle = RawHandle::from_id(callback_id);
    callback_handle.push(lua, state);
    lua.insert(state, 1);

    // add the original function as the first argument
    unsafe {
        lua.push_function(state, *original_function);
        lua.insert(state, 2);
    }

    let base = lua.get_top(state) - num_arguments - 2; // 2 for the callback and original function
    if let Err(why) = lua.pcall(state, num_arguments + 1, LUA_MULTRET, 0) {
        dbg!("Error calling detour callback: {}", why);
        return 0;
    }

    let ret_count = lua.get_top(state) - base;
    dbg!("detour handler returning {} values", ret_count);

    ret_count
}

pub fn detour(
    lua: &LuaApi,
    state: *mut LuaState,
    env: crate::EnvHandle,
) -> anyhow::Result<Detour> {
    let target_function = lua.to_function(state, 1);
    if target_function.is_none() {
        anyhow::bail!("First argument must be a function to detour.");
    }

    let target_function = target_function.unwrap();
    let num_arguments = lua.check_number(state, 2) as i32;

    if lua.type_id(state, 3) != LuaTypeId::Function {
        anyhow::bail!("Third argument must be a function to use as detour.");
    }

    let detour_callback = RawHandle::from_stack(lua, state);
    if detour_callback.is_none() {
        anyhow::bail!("Failed to ref detour callback from stack.");
    }

    let detour_callback = detour_callback.unwrap();

    // create the trampoline
    let callback_trampoline = CallbackTrampoline::allocate();
    if callback_trampoline.is_err() {
        anyhow::bail!("Failed to allocate callback trampoline.");
    }

    let mut callback_trampoline = callback_trampoline.unwrap();
    unsafe {
        callback_trampoline.generate_code(detour_callback.get_id(), lua, num_arguments, detour_handler);
        if callback_trampoline.make_executable().is_err() {
            anyhow::bail!("Failed to make callback trampoline executable.");
        }
    }

    let detour = unsafe {
        Box::new(retour::GenericDetour::new(
            target_function,
            (&callback_trampoline).into()
        )?)
    };

    unsafe {
        if detour.enable().is_err() {
            anyhow::bail!("Failed to enable detour.");
        }
    }

    // create retour trampoline
    let mut retour_trampoline = RetourLuaTrampoline::allocate();
    if retour_trampoline.is_err() {
        anyhow::bail!("Failed to allocate retour trampoline.");
    }

    let mut retour_trampoline = retour_trampoline.unwrap();
    unsafe {
        retour_trampoline.generate_code(detour.as_ref() as *const GenericDetour<LuaFunction>, retour_handler);
        if retour_trampoline.make_executable().is_err() {
            anyhow::bail!("Failed to make retour trampoline executable.");
        }
    }

    callback_trampoline.write_original_function_pointer(retour_trampoline.as_function());

    Ok(Detour {
        detour,
        detour_callback,
        callback_trampoline,
        retour_trampoline,
    })
}