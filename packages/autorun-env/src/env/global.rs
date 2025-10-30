use autorun_lua::LuaState;

mod client {
	use crate::EnvHandle;

	static ENV: std::sync::Mutex<Option<EnvHandle>> = std::sync::Mutex::new(None);

	pub fn set_env(env: EnvHandle) {
		*ENV.lock().unwrap() = Some(env);
	}

	pub fn get_env() -> Option<EnvHandle> {
		*ENV.lock().unwrap()
	}
}

mod menu {
	use crate::EnvHandle;

	static ENV: std::sync::Mutex<Option<EnvHandle>> = std::sync::Mutex::new(None);

	pub fn set_env(env: EnvHandle) {
		*ENV.lock().unwrap() = Some(env);
	}

	pub fn get_env() -> Option<EnvHandle> {
		*ENV.lock().unwrap()
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
