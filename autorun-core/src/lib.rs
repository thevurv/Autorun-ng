mod plugins;
mod settings;

pub struct Workspace {
	plugins_path: std::path::PathBuf,
	logs_path: std::path::PathBuf,
	settings_path: std::path::PathBuf,
	settings: std::sync::OnceLock<settings::Settings>,
}

fn create_if_dne(p: std::path::PathBuf) -> std::io::Result<std::path::PathBuf> {
	if !p.exists() {
		std::fs::create_dir_all(&p)?;
	}

	Ok(p)
}

impl Workspace {
	const PLUGINS_DIR: &str = "plugins";
	const LOGS_DIR: &str = "logs";
	const SETTINGS_FILE: &str = "settings.toml";

	/// Creates a workspace from a specific directory.
	/// You should probably use `from_cwd` instead which uses the standard ./autorun directory.
	pub fn from_dir(p: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
		let p = p.as_ref();

		let plugins = create_if_dne(p.join(Self::PLUGINS_DIR))?;
		let logs = create_if_dne(p.join(Self::LOGS_DIR))?;

		let settings = p.join(Self::SETTINGS_FILE);
		if !settings.exists() {
			std::fs::write(&settings, include_str!("../data/default_settings.toml"))?;
		}

		Ok(Self {
			plugins_path: plugins,
			logs_path: logs,
			settings_path: settings,
			settings: std::sync::OnceLock::new(),
		})
	}

	pub fn from_exe() -> std::io::Result<Self> {
		let cwd = std::env::current_exe()?
			.parent()
			.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to get exe parent"))?
			.to_path_buf();

		Self::from_dir(cwd.join("autorun"))
	}
}
