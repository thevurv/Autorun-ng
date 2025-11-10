use anyhow::anyhow;
use autorun_jit::Function;
use autorun_lua::{LuaApi, LuaFunction, LuaUserdata, RawHandle};
use autorun_types::LuaState;

pub struct Detour {
	pub detour: Box<retour::GenericDetour<LuaFunction>>,
	pub _detour_callback: RawHandle,
	pub _detour_trampoline: Function,
	pub _retour_trampoline: Function,
	pub original_function_ptr: Box<usize>,
}

impl LuaUserdata for Detour {}

pub fn detour_enable(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<()> {
	let detour = lua.to::<*mut Detour>(state, 1);
	let detour = unsafe { detour.as_mut() }.ok_or(anyhow!("First argument must be a detour userdata"))?;

	unsafe {
		detour.detour.enable()?;
	}

	Ok(())
}

pub fn detour_disable(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<()> {
	let detour = lua.to::<*mut Detour>(state, 1);
	let detour = unsafe { detour.as_mut() }.ok_or(anyhow!("First argument must be a detour userdata"))?;

	unsafe {
		detour.detour.disable()?;
	}

	Ok(())
}

pub fn detour_get_original(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<LuaFunction> {
	let detour = lua.to::<*mut Detour>(state, 1);
	let detour = unsafe { detour.as_mut() }.ok_or(anyhow!("First argument must be a detour userdata"))?;

	Ok(unsafe { std::mem::transmute::<usize, LuaFunction>(*detour.original_function_ptr) })
}

pub fn detour_remove(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<()> {
	let detour = lua.to::<*mut Detour>(state, 1);
	let detour = unsafe { detour.as_mut() }.ok_or(anyhow!("First argument must be a detour userdata"))?;

	unsafe {
		detour.detour.disable()?;
		std::ptr::drop_in_place(detour);
	}

	Ok(())
}
