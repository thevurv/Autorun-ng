use autorun_core::plugins::Plugin;
use autorun_lua::{LuaApi, LuaState};

mod client {
	use crate::EnvHandle;

	static ENV: std::sync::Mutex<Option<EnvHandle>> = std::sync::Mutex::new(None);

	pub fn set_env(env: EnvHandle) {
		*ENV.lock().unwrap() = Some(env);
	}

	pub fn get_env() -> Option<EnvHandle> {
		ENV.lock().unwrap().clone()
	}
}

mod menu {
	use crate::EnvHandle;

	static ENV: std::sync::Mutex<Option<EnvHandle>> = std::sync::Mutex::new(None);

	pub fn set_env(env: EnvHandle) {
		*ENV.lock().unwrap() = Some(env);
	}

	pub fn get_env() -> Option<EnvHandle> {
		ENV.lock().unwrap().clone()
	}
}

pub fn get_realm(state: *mut LuaState) -> autorun_types::Realm {
	let client_state = autorun_interfaces::lua::get_state(autorun_types::Realm::Client).unwrap();

	if Some(state) == client_state {
		autorun_types::Realm::Client
	} else {
		autorun_types::Realm::Menu
	}
}

pub fn get_realm_env(realm: autorun_types::Realm) -> Option<crate::EnvHandle> {
	match realm {
		autorun_types::Realm::Client => client::get_env(),
		autorun_types::Realm::Menu => menu::get_env(),
	}
}

pub fn set_realm_env(realm: autorun_types::Realm, env: crate::EnvHandle) {
	match realm {
		autorun_types::Realm::Client => client::set_env(env),
		autorun_types::Realm::Menu => menu::set_env(env),
	}
}

pub fn get_running_plugin(env: crate::EnvHandle, lua: &LuaApi, state: *mut LuaState) -> Option<&Plugin> {
	env.push(lua, state);
	lua.get_field(state, -1, c"Autorun".as_ptr());
	lua.get_field(state, -1, c"PLUGIN".as_ptr());

	let dir = lua.to_userdata(state, -1) as *mut Plugin;
	if dir.is_null() {
		lua.pop(state, 3);
		return None;
	}
	lua.pop(state, 3);

	unsafe { dir.as_ref() }
}
