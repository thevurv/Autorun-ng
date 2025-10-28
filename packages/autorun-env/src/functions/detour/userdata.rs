use autorun_jit::Function;
use autorun_lua::{IntoLua, LuaApi, LuaFunction, RawHandle};
use autorun_types::LuaState;
use crate::functions::detour::handlers::retour_handler;

pub struct Detour {
	pub detour: Box<retour::GenericDetour<LuaFunction>>,
	pub detour_callback: RawHandle,
	pub detour_trampoline: Function,
	pub retour_trampoline: Function,
	pub original_function_ptr: Box<usize>,
}

impl IntoLua for Detour {
	fn into_lua(self, lua: &LuaApi, state: *mut LuaState) {
		lua.new_userdata(state, self);
	}
}

pub fn detour_enable(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
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

pub fn detour_disable(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
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

pub fn detour_get_original(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<LuaFunction> {
    let detour_userdata = lua.to_userdata(state, 1) as *mut Detour;
    if detour_userdata.is_null() {
        anyhow::bail!("First argument must be a detour userdata.");
    }

    let detour = unsafe { &mut *detour_userdata };

    Ok(unsafe { std::mem::transmute(*detour.original_function_ptr) })
}

pub fn detour_remove(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<()> {
    let detour_userdata = lua.to_userdata(state, 1) as *mut Detour;
    if detour_userdata.is_null() {
        anyhow::bail!("First argument must be a detour userdata.");
    }

    let detour = unsafe { &mut *detour_userdata };
    unsafe {
        detour.detour.disable()?;
        std::ptr::drop_in_place(detour_userdata);
    }

    Ok(())
}