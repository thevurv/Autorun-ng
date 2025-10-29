use cap_std::fs::Dir;
use serde::{Deserialize, Serialize};

pub struct Plugin {
	/// Top level directory. Read-only.
	dir: Dir,

	/// Directory for mutable data.
	data_dir: Dir,

	config: Config,
}

impl core::fmt::Display for Plugin {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let cfg = &self.config;
		write!(f, "{} v{} by {}", cfg.plugin.name, cfg.plugin.version, cfg.plugin.author)
	}
}

impl Plugin {
	const PLUGIN_CONFIG: &str = "plugin.toml";

	const PLUGIN_SRC: &str = "src";
	const PLUGIN_DATA: &str = "data";

	const INIT_FILE: &str = "init.lua";

	pub fn dir(&self) -> &Dir {
		&self.dir
	}

	pub fn data_dir(&self) -> &Dir {
		&self.data_dir
	}

	pub fn src(&self) -> std::io::Result<Dir> {
		self.dir.open_dir(Self::PLUGIN_SRC)
	}

	pub fn client(&self) -> std::io::Result<Dir> {
		self.src()?.open_dir("client")
	}

	pub fn menu(&self) -> std::io::Result<Dir> {
		self.src()?.open_dir("menu")
	}

	pub fn shared(&self) -> std::io::Result<Dir> {
		self.src()?.open_dir("shared")
	}

	pub fn try_clone(&self) -> std::io::Result<Self> {
		Ok(Self {
			dir: self.dir.try_clone()?,
			data_dir: self.data_dir.try_clone()?,
			config: self.config.clone(),
		})
	}

	pub fn read_client_init(&self) -> std::io::Result<Vec<u8>> {
		self.client()?.read(Self::INIT_FILE)
	}

	pub fn read_menu_init(&self) -> std::io::Result<Vec<u8>> {
		self.menu()?.read(Self::INIT_FILE)
	}

	pub fn read_shared_init(&self) -> std::io::Result<Vec<u8>> {
		self.shared()?.read(Self::INIT_FILE)
	}

	pub fn from_dir(dir: Dir) -> anyhow::Result<Self> {
		if !dir.exists(Self::PLUGIN_CONFIG) {
			return Err(anyhow::anyhow!("Plugin config not found"));
		}

		let data_dir = match dir.exists(Self::PLUGIN_DATA) {
			true => dir.open_dir(Self::PLUGIN_DATA)?,
			false => {
				dir.create_dir(Self::PLUGIN_DATA)?;
				dir.open_dir(Self::PLUGIN_DATA)?
			}
		};

		let config_data = dir.read_to_string(Self::PLUGIN_CONFIG)?;
		let config: Config = toml::from_str(&config_data)?;

		Ok(Self { dir, data_dir, config })
	}

	pub fn config(&self) -> &Config {
		&self.config
	}
}

nestify::nest! {
	#[derive(Debug, Clone, Serialize, Deserialize)]*
	pub struct Config {
		pub plugin: pub struct ConfigPlugin {
			pub name: String,
			pub author: String,
			pub version: String,
			pub description: String,
			pub ordering: Option<u32>,

			pub language: #[serde(rename_all = "lowercase")] #[non_exhaustive] pub enum ConfigPluginLanguage {
				Lua,
				Native,
			}
		}
	}
}
