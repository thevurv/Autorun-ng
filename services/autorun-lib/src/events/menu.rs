#[allow(unused)]
pub fn run(state: *mut autorun_types::LuaState) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;

	let env = autorun_env::EnvHandle::create(&lua, state)?;
	autorun_env::global::set_realm_env(autorun_types::Realm::Menu, env);

	let (plugins, _errors) = workspace.get_plugins()?;
	if plugins.is_empty() {
		return Ok(());
	}

	for plugin in &plugins {
		env.set_plugin(lua, state, plugin);
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
	let config = plugin.get_config()?;

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			let Ok(menu_content) = plugin.read_menu_init() else {
				return Ok(());
			};

			env.execute(lua, state, c"menu/init.lua", &menu_content)
		}

		_ => Err(anyhow::anyhow!("Unsupported language: {:?}", config.plugin.language)),
	}
}
