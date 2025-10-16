/// Function that triggers all plugins init (server start) scripts.
pub fn run(state: *mut autorun_types::LuaState) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;

	// Set up the hook environment once before running all plugins
	setup_env(state, lua)?;

	let (plugins, _errors) = workspace.get_plugins()?;
	for plugin in &plugins {
		autorun_log::info!("Running init for plugin: {}", plugin.get_config()?.plugin.name);
		run_entrypoint(state, lua, &plugin)?;
	}

	lua.pop(state, 1); // Pop the environment

	Ok(())
}

fn setup_env(state: *mut autorun_types::LuaState, lua: &autorun_lua::LuaApi) -> anyhow::Result<()> {
	lua.create_table(state, 0, 0);

	lua.create_table(state, 0, 2);

	lua.push_string(state, c"Autorun".as_ptr());
	lua.push_value(state, -2);
	lua.set_table(state, -4);

	lua.pop(state, 1);

	Ok(())
}

fn run_entrypoint(
	state: *mut autorun_types::LuaState,
	lua: &autorun_lua::LuaApi,
	plugin: &autorun_core::plugins::Plugin,
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

			// Execute the loaded chunk
			if let Err(why) = lua.pcall(state, 0, 0, 0) {
				return Err(anyhow::anyhow!("Failed to execute Lua hook: {why}"));
			}

			Ok(())
		}

		_ => Err(anyhow::anyhow!("Unsupported language: {:?}", config.plugin.language)),
	}
}
