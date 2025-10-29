use autorun_lua::{LuaApi, RawLuaReturn};
use autorun_types::LuaState;

pub fn load(lua: &LuaApi, state: *mut LuaState, _env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	let source = lua.check_string(state, 1);
	let chunk_name = lua.to::<Option<&[u8]>>(state, 2).unwrap_or(b"loadstring");
	let chunk_name = std::ffi::CString::new(chunk_name)?;

	if let Err(why) = lua.load_buffer_x(state, source.as_bytes(), &chunk_name, c"t") {
		lua.push_nil(state);
		lua.push(state, &why);
		return Ok(RawLuaReturn(2));
	}

	Ok(RawLuaReturn(1))
}
