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
		run_entrypoint(state, lua, &plugin, env)?;
	}

	Ok(())
}

fn run_entrypoint(
	state: *mut autorun_types::LuaState,
	lua: &autorun_lua::LuaApi,
	plugin: &autorun_core::plugins::Plugin,
	env: &autorun_env::Environment,
) -> anyhow::Result<()> {
	let config = plugin.get_config()?;

	match config.plugin.language {
		autorun_core::plugins::ConfigPluginLanguage::Lua => {
			let Ok(hook_content) = plugin.read_hook_lua() else {
				return Ok(());
			};

			env.set_path(lua, state, "/src/hook.lua");

			// Execute the Lua code via the original load_buffer function through the detour
			let result = crate::hooks::load_buffer::call_original(
				state,
				hook_content.as_ptr() as *const i8,
				hook_content.len(),
				c"hook.lua".as_ptr(),
				std::ptr::null(),
			);

			if result != 0 {
				return Err(anyhow::anyhow!("Failed to load Lua hook"));
			}

			env.push(lua, state);
			if lua.set_fenv(state, -2).is_err() {
				return Err(anyhow::anyhow!("Failed to set fenv for Lua hook"));
			}

			if let Err(why) = lua.pcall(state, 0, 0, 0) {
				return Err(anyhow::anyhow!("Failed to execute Lua hook: {why}"));
			}

			Ok(())
		}

		_ => Err(anyhow::anyhow!("Unsupported language: {:?}", config.plugin.language)),
	}
}
