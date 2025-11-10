use autorun_lua::{LuaApi, RawLuaReturn};
use autorun_types::LuaState;

pub fn load(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	let source = lua.raw.checkstring(state, 1);
	let chunk_name = lua.to::<Option<&[u8]>>(state, 2).unwrap_or(b"loadstring");
	let chunk_name = std::ffi::CString::new(chunk_name)?;
	let chunk_name = env.format_chunk_name(&chunk_name)?;

	if let Err(why) = lua.raw.loadbufferx(state, source.as_bytes(), &chunk_name, c"t") {
		lua.raw.pushnil(state);
		lua.push(state, why.to_string());
		return Ok(RawLuaReturn(2));
	}

	Ok(RawLuaReturn(1))
}
