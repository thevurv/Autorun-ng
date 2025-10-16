/// Function that triggers all plugins hook (each file) scripts.
pub fn run(state: *mut autorun_types::LuaState, buffer: &[u8], name: &[u8], mode: &[u8]) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;

	setup_env(state, lua, buffer, name)?;

	let (plugins, _errors) = workspace.get_plugins()?;
	for plugin in plugins {
		run_hook_entrypoint(state, lua, &plugin)?;
	}

	lua.pop(state, 1); // Pop the environment

	Ok(())
}

fn setup_env(state: *mut autorun_types::LuaState, lua: &autorun_lua::LuaApi, buffer: &[u8], name: &[u8]) -> anyhow::Result<()> {
	lua.create_table(state, 0, 0);

	lua.create_table(state, 0, 2);

	lua.push_string(state, c"SRC".as_ptr());
	lua.push_lstring(state, buffer.as_ptr() as *const i8, buffer.len());
	lua.set_table(state, -3);

	lua.push_string(state, c"NAME".as_ptr());
	lua.push_lstring(state, name.as_ptr() as *const i8, name.len());
	lua.set_table(state, -3);

	lua.push_string(state, c"Autorun".as_ptr());
	lua.push_value(state, -2);
	lua.set_table(state, -4);

	lua.pop(state, 1);

	Ok(())
}

fn run_hook_entrypoint(
	state: *mut autorun_types::LuaState,
	lua: &autorun_lua::LuaApi,
	plugin: &autorun_core::plugins::Plugin,
) -> anyhow::Result<()> {
	let config = plugin.get_config()?;

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			let Some(hook_file) = plugin.get_hook_file() else {
				return Ok(());
			};

			// Read the hook file content
			let hook_content = std::fs::read(&hook_file)?;
			let hook_name = hook_file.to_string_lossy();
			let hook_name_cstr = std::ffi::CString::new(hook_name.as_ref())?;

			// Execute the Lua code via the original load_buffer function through the detour
			let result = crate::hooks::load_buffer::call_original(
				state,
				hook_content.as_ptr() as *const i8,
				hook_content.len(),
				hook_name_cstr.as_ptr(),
				std::ptr::null(),
			);

			if result != 0 {
				return Err(anyhow::anyhow!("Failed to load Lua hook: {}", hook_name));
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
