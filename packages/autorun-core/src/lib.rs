pub mod plugins;
pub mod settings;

pub struct Workspace {
	pub unsafe_raw_path: std::path::PathBuf,
	path: cap_std::fs::Dir,
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

	fn plugins(&self) -> std::io::Result<cap_std::fs::Dir> {
		self.path.open_dir(Self::PLUGINS_DIR)
	}

	fn logs(&self) -> std::io::Result<cap_std::fs::Dir> {
		self.path.open_dir(Self::LOGS_DIR)
	}

	/// Creates a workspace from a specific directory.
	/// You should probably use `from_cwd` instead which uses the standard ./autorun directory.
	pub fn from_dir(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
		let path = path.as_ref();

		create_if_dne(path.join(Self::PLUGINS_DIR))?;
		create_if_dne(path.join(Self::LOGS_DIR))?;

		let settings = path.join(Self::SETTINGS_FILE);
		if !settings.exists() {
			std::fs::write(&settings, include_bytes!("../data/default_settings.toml"))?;
		}

		Ok(Self {
			unsafe_raw_path: path.to_path_buf(),
			path: cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())?,
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
