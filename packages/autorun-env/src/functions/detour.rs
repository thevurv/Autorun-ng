mod raw;
mod handlers;
mod userdata;

use retour::GenericDetour;
use autorun_lua::{IntoLua, LuaApi, LuaFunction, LuaTypeId, RawHandle, LUA_MULTRET};
use autorun_types::LuaState;
use crate::functions::detour::userdata::Detour;

pub use userdata::{detour_enable, detour_disable};
use crate::functions::detour::handlers::{detour_handler, retour_handler};
use crate::functions::detour::raw::{make_detour_trampoline, make_retour_lua_trampoline};

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

    if lua.type_id(state, 2) != LuaTypeId::Function {
        anyhow::bail!("Second argument must be a function to use as detour.");
    }

    let detour_callback = RawHandle::from_stack(lua, state);
    if detour_callback.is_none() {
        anyhow::bail!("Failed to ref detour callback from stack.");
    }

    let detour_callback = detour_callback.unwrap();
    let mut original_function_ptr = Box::new(0usize);

    // create the trampoline
    let detour_trampoline = make_detour_trampoline(
        lua,
        detour_callback.get_id(),
        original_function_ptr.as_ref() as *const usize,
        detour_handler
    )?;

    let detour = unsafe {
        Box::new(GenericDetour::new(
            target_function,
            std::mem::transmute(detour_trampoline.as_ptr()),
        )?)
    };

    unsafe {
        detour.enable().map_err(|e| anyhow::anyhow!("Failed to enable detour: {}", e))?;
    }

    // create retour trampoline
    let retour_trampoline = make_retour_lua_trampoline(detour.as_ref() as *const GenericDetour<LuaFunction>, retour_handler)?;

    // link the original function pointer
    *original_function_ptr = retour_trampoline.as_ptr() as usize;

    Ok(Detour {
        detour,
        detour_callback,
        detour_trampoline,
        retour_trampoline,
        original_function_ptr,
    })
}