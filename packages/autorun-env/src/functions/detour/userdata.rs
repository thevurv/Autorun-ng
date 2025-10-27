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