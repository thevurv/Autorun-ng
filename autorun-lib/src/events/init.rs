/// Function that triggers all plugins init (server start) scripts.
pub fn init(state: *mut autorun_types::LuaState) -> anyhow::Result<()> {
	let workspace = super::get_workspace()?;

	let (plugins, errors) = workspace.get_plugins()?;

	for plugin in plugins {
		plugin.run_init_entrypoint()?;
	}

	Ok(())
}
