use crate::LuaApi;

static LUA_API: std::sync::OnceLock<LuaApi> = std::sync::OnceLock::new();

#[cfg(target_os = "linux")]
const LUA_SHARED_PATH: &str = "lua_shared_client.so";

#[cfg(target_os = "windows")]
const LUA_SHARED_PATH: &str = "lua_shared.dll";

pub fn get_api() -> Result<&'static LuaApi, libloading::Error> {
	if let Some(api) = LUA_API.get() {
		return Ok(api);
	}

	let lua_shared = unsafe { libloading::Library::new(LUA_SHARED_PATH) }?;

	let api = LuaApi::new(&lua_shared)?;

	std::mem::forget(lua_shared);

	LUA_API.set(api).expect("Should never already be initialized");

	Ok(LUA_API.get().expect("Should be initialized"))
}
