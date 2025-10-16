static AUTORUN_WORKSPACE: std::sync::OnceLock<autorun_core::Workspace> = std::sync::OnceLock::new();

pub fn get_workspace() -> anyhow::Result<&'static autorun_core::Workspace> {
	if let Some(workspace) = AUTORUN_WORKSPACE.get() {
		return Ok(workspace);
	}

	let workspace = autorun_core::Workspace::from_exe()?;

	AUTORUN_WORKSPACE
		.set(workspace)
		.map_err(|_| anyhow::anyhow!("Failed to set workspace"))?;

	Ok(AUTORUN_WORKSPACE.get().unwrap())
}

pub mod hook;
pub mod init;
