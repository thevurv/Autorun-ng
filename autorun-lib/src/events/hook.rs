/// Function that triggers all plugins hook (each file) scripts.
pub fn run(state: *mut autorun_types::LuaState, buffer: &[u8], name: &[u8], mode: &[u8]) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;
	let env = autorun_env::global::get_env(&lua, state);

	let (plugins, _errors) = workspace.get_plugins()?;
	if plugins.is_empty() {
		return Ok(());
	}

	env.set_name(lua, state, name);
	env.set_code(lua, state, buffer);
	env.set_mode(lua, state, b"hook");

	for plugin in plugins {
		env.set_plugin(lua, state, &plugin);
		run_entrypoint(lua, state, &plugin, env)?;
	}

	Ok(())
}

fn run_entrypoint(
	lua: &autorun_lua::LuaApi,
	state: *mut autorun_types::LuaState,
	plugin: &autorun_core::plugins::Plugin,
	env: &autorun_env::Environment,
) -> anyhow::Result<()> {
	let config = plugin.get_config()?;

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			let Ok(hook_content) = plugin.read_hook_lua() else {
				return Ok(());
			};

			env.execute(lua, state, &hook_content)
		}

		_ => Err(anyhow::anyhow!("Unsupported language: {:?}", config.plugin.language)),
	}
}
