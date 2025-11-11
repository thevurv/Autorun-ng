use autorun_lua::{LuaApi, RawLuaReturn};
use autorun_types::LuaState;

pub fn load(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	let source = lua.raw.checkstring(state, 1);
	let chunk_name = lua.raw.try_to::<Option<&[u8]>>(state, 2)?.unwrap_or(b"loadstring");
	let chunk_name = std::ffi::CString::new(chunk_name)?;
	let chunk_name = env.format_chunk_name(&chunk_name)?;

	match lua.load(state, source.as_bytes(), &chunk_name) {
		Err(why) => {
			lua.raw.pushnil(state);
			lua.raw.push(state, why.to_string());
			Ok(RawLuaReturn(2))
		}
		Ok(chunk) => {
			lua.raw.push(state, &chunk);
			Ok(RawLuaReturn(1))
		}
	}
}
