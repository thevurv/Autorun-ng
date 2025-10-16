/// Function that triggers all plugins init (server start) scripts.
pub fn run(state: *mut autorun_types::LuaState) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;
	let env = super::get_env(&lua, state);

	// Set up the hook environment once before running all plugins
	let (plugins, _errors) = workspace.get_plugins()?;
	if plugins.is_empty() {
		return Ok(());
	}

	env.set_name(lua, state, b"__INIT__");
	env.set_code(lua, state, b"");
	env.set_mode(lua, state, b"init");

	for plugin in &plugins {
		run_entrypoint(state, lua, &plugin, env)?;
	}

	Ok(())
}

fn run_entrypoint(
	state: *mut autorun_types::LuaState,
	lua: &autorun_lua::LuaApi,
	plugin: &autorun_core::plugins::Plugin,
	env: &autorun_env::EnvHandle,
) -> anyhow::Result<()> {
	let config = plugin.get_config()?;

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			let Some(init_file) = plugin.get_init_file() else {
				return Ok(());
			};

			// Read the hook file content
			let init_content = std::fs::read(&init_file)?;
			let init_name = init_file.to_string_lossy();

			// Execute the Lua code via the original load_buffer function through the detour
			let result = crate::hooks::load_buffer::call_original(
				state,
				init_content.as_ptr() as *const i8,
				init_content.len(),
				c"init.lua".as_ptr(),
				std::ptr::null(),
			);

			if result != 0 {
				return Err(anyhow::anyhow!("Failed to load Lua hook: {init_name}"));
			}

			env.push(lua, state);
			lua.set_fenv(state, -2);

			if let Err(why) = lua.pcall(state, 0, 0, 0) {
				return Err(anyhow::anyhow!("Failed to execute Lua hook: {why}"));
			}

			Ok(())
		}

		_ => Err(anyhow::anyhow!("Unsupported language: {:?}", config.plugin.language)),
	}
}
