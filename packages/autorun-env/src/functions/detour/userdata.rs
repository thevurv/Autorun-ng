use std::ffi::CStr;
use autorun_lua::{IntoLua, LuaApi, LuaFunction, RawHandle};
use autorun_types::LuaState;
use crate::functions::detour::raw::{CallbackTrampoline, RetourLuaTrampoline};


pub struct Detour {
    pub detour: Box<retour::GenericDetour<LuaFunction>>,
    pub detour_callback: RawHandle,
    pub callback_trampoline: CallbackTrampoline,
    pub retour_trampoline: RetourLuaTrampoline,
}

impl IntoLua for Detour {
    fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
        lua.new_userdata(state, self);
    }
}

pub fn detour_enable(
    lua: &LuaApi,
    state: *mut LuaState,
    env: crate::EnvHandle,
) -> anyhow::Result<()> {
    let detour_userdata = lua.to_userdata(state, 1) as *mut Detour;
    if detour_userdata.is_null() {
        anyhow::bail!("First argument must be a detour userdata.");
    }

    let detour = unsafe { &mut *detour_userdata };
    unsafe {
        detour.detour.enable()?;
    }

    Ok(())
}

pub fn detour_disable(
    lua: &LuaApi,
    state: *mut LuaState,
    env: crate::EnvHandle,
) -> anyhow::Result<()> {
    let detour_userdata = lua.to_userdata(state, 1) as *mut Detour;
    if detour_userdata.is_null() {
        anyhow::bail!("First argument must be a detour userdata.");
    }

    let detour = unsafe { &mut *detour_userdata };
    unsafe {
        detour.detour.disable()?;
    }

    Ok(())
}
