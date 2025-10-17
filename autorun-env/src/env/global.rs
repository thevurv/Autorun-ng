use autorun_lua::{LuaApi, LuaState};

use super::Environment;

static AUTORUN_ENV: std::sync::OnceLock<Environment> = std::sync::OnceLock::new();

pub fn get_env(lua: &LuaApi, state: *mut LuaState) -> &'static Environment {
	AUTORUN_ENV.get_or_init(|| Environment::create(lua, state))
}

pub fn get_current_path(lua: &LuaApi, state: *mut LuaState) -> std::borrow::Cow<'static, str> {
	let env = get_env(lua, state);

	env.push(lua, state);
	lua.get_field(state, -1, c"Autorun".as_ptr());
	lua.get_field(state, -1, c"PATH".as_ptr());
	let path_str = lua.check_string(state, -1);
	lua.pop(state, 3);

	path_str
}

pub fn is_inside_env(lua: &LuaApi, state: *mut LuaState) -> bool {
	if let Some(env) = AUTORUN_ENV.get() {
		if lua.get_info(state, 1, c"f").is_none() {
			// No function info available
			return false;
		}

		lua.get_fenv(state, -1);
		env.handle.push(lua, state);

		let equal = lua.is_raw_equal(state, -1, -2);
		lua.pop(state, 3);

		equal
	} else {
		false
	}
}
