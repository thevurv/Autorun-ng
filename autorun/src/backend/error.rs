#[derive(Debug, thiserror::Error)]
pub enum SettingsError {}

#[derive(Debug, thiserror::Error)]
pub enum AttachError {
	#[error("Couldn't find Garry's Mod in your steam library.")]
	GameNotFound,

	#[cfg(windows)]
	#[error("Failed to inject: {0}")]
	Injection(#[from] super::injector::Error),

	#[error("{0}")]
	Libloading(#[from] libloading::Error),

	#[error("IO Error: {0}")]
	IO(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AutorunError {
	#[error("Error while loading settings: {0}")]
	Settings(#[from] SettingsError),

	#[error("Error while injecting: {0}")]
	Inject(#[from] AttachError),
}
