use autorun_core::Workspace;

static AUTORUN_WORKSPACE: std::sync::OnceLock<Workspace> = std::sync::OnceLock::new();

pub fn get_workspace() -> anyhow::Result<&'static Workspace> {
	AUTORUN_WORKSPACE
		.get()
		.ok_or_else(|| anyhow::anyhow!("Workspace not initialized"))
}

pub fn set_workspace_path(path: &str) -> anyhow::Result<()> {
	let workspace = Workspace::from_dir(path)?;
	AUTORUN_WORKSPACE
		.set(workspace)
		.map_err(|_| anyhow::anyhow!("Failed to set workspace"))?;

	Ok(())
}

pub mod client_init;
pub mod hook;
pub mod menu_init;
