static AUTORUN_WORKSPACE: std::sync::OnceLock<autorun_core::Workspace> = std::sync::OnceLock::new();

pub fn get_workspace() -> anyhow::Result<&'static autorun_core::Workspace> {
	AUTORUN_WORKSPACE
		.get()
		.ok_or_else(|| anyhow::anyhow!("Workspace not initialized"))
}

pub fn set_workspace_path(path: &str) -> anyhow::Result<()> {
	let workspace = autorun_core::Workspace::from_dir(path)?;
	AUTORUN_WORKSPACE
		.set(workspace)
		.map_err(|_| anyhow::anyhow!("Failed to set workspace"))?;

	Ok(())
}

static AUTORUN_ENV: std::sync::OnceLock<autorun_env::EnvHandle> = std::sync::OnceLock::new();

pub fn get_env(api: &autorun_lua::LuaApi, state: *mut autorun_types::LuaState) -> &'static autorun_env::EnvHandle {
	AUTORUN_ENV.get_or_init(|| autorun_env::EnvHandle::create(api, state))
}

pub mod hook;
pub mod init;
