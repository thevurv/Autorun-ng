/// Function that triggers all plugins hook (each file) scripts.
pub fn run(state: *mut autorun_types::LuaState, buffer: &[u8], name: &[u8], mode: &[u8]) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;
	let lua = autorun_lua::get_api()?;
	let env = super::get_env(&lua, state);

	let (plugins, _errors) = workspace.get_plugins()?;
	if plugins.is_empty() {
		return Ok(());
	}

	env.set_name(lua, state, name);
	env.set_code(lua, state, buffer);
	env.set_mode(lua, state, b"hook");

	for plugin in plugins {
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
