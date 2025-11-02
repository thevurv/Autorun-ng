use autorun_log::*;
use autorun_types::Realm;

/// Function that triggers all plugins init (server start) scripts.
pub fn run(state: *mut autorun_types::LuaState) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;

	let env = autorun_env::EnvHandle::create(lua, state, Realm::Client)?;
	let realm = autorun_env::global::get_realm(state);
	autorun_env::global::set_realm_env(realm, env);

	let (mut plugins, _errors) = workspace.get_plugins()?;
	if plugins.is_empty() {
		return Ok(());
	}

	plugins.sort_by_key(|p| p.config().plugin.ordering.unwrap_or(9999));
	for plugin in &plugins {
		env.set_plugin(lua, state, plugin)?;
		run_entrypoint(lua, state, plugin, &env)?;
	}

	Ok(())
}

fn run_entrypoint(
	lua: &autorun_lua::LuaApi,
	state: *mut autorun_types::LuaState,
	plugin: &autorun_core::plugins::Plugin,
	env: &autorun_env::EnvHandle,
) -> anyhow::Result<()> {
	let config = plugin.config();

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			if let Ok(client_init) = plugin.read_client_init() {
				env.execute(lua, state, c"client/init.lua", &client_init)?;
			};

			if let Ok(shared_init) = plugin.read_shared_init() {
				env.execute(lua, state, c"shared/init.lua", &shared_init)?;
			}
		}

		autorun_core::plugins::ConfigPluginLanguage::Native => {
			#[cfg(target_os = "linux")]
			const PLUGIN_PATH: &str = "plugin.so";

			#[cfg(target_os = "windows")]
			const PLUGIN_PATH: &str = "plugin.dll";

			let dir = plugin.dir();
			let path = autorun_fs::get_path(autorun_fs::ambient_authority(), dir)?;
			let lib_path = path.join(PLUGIN_PATH);
			if !lib_path.exists() {
				warn!(
					"Native init plugin library not found for plugin '{plugin}': {}",
					lib_path.display()
				);

				return Ok(());
			}

			let library = unsafe { libloading::Library::new(path.join("plugin.so"))? };

			if let Ok(autorun_client_init) =
				unsafe { library.get::<extern "C" fn(plugin: *const core::ffi::c_void)>(b"autorun_client_init\0") }
			{
				autorun_client_init(&raw const *plugin as _);
			}
		}

		_ => anyhow::bail!("Unsupported language: {:?}", config.plugin.language),
	}

	Ok(())
}
