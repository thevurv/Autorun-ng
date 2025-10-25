use cap_std::fs::Dir;
use serde::{Deserialize, Serialize};

pub struct Plugin {
	/// Top level directory. Read-only.
	dir: Dir,

	/// Directory for mutable data.
	data_dir: Dir,

	config: std::sync::OnceLock<Config>,
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

	pub fn try_clone(&self) -> std::io::Result<Self> {
		Ok(Self {
			dir: self.dir.try_clone()?,
			data_dir: self.data_dir.try_clone()?,
			config: std::sync::OnceLock::new(),
		})
	}

	pub fn read_client_init(&self) -> std::io::Result<Vec<u8>> {
		self.client()?.read(Self::INIT_FILE)
	}

	pub fn client_exists(&self) -> std::io::Result<bool> {
		self.client()?.try_exists(Self::INIT_FILE)
	}

	pub fn read_menu_init(&self) -> std::io::Result<Vec<u8>> {
		self.menu()?.read("init.lua")
	}

	pub fn menu_exists(&self) -> std::io::Result<bool> {
		self.menu()?.try_exists(Self::INIT_FILE)
	}

	pub fn from_dir(dir: Dir) -> anyhow::Result<Self> {
		if !dir.exists(Self::PLUGIN_CONFIG) {
			return Err(anyhow::anyhow!("Plugin config not found"));
		}

		let Ok(src) = dir.open_dir(Self::PLUGIN_SRC) else {
			return Err(anyhow::anyhow!("Plugin src directory not found"));
		};

		let data_dir = match dir.exists(Self::PLUGIN_DATA) {
			true => dir.open_dir(Self::PLUGIN_DATA)?,
			false => {
				dir.create_dir(Self::PLUGIN_DATA)?;
				dir.open_dir(Self::PLUGIN_DATA)?
			}
		};

		let this = Self {
			dir,
			data_dir,
			config: std::sync::OnceLock::new(),
		};

		if !this.menu_exists().unwrap_or(false) && !this.client_exists().unwrap_or(false) {
			return Err(anyhow::anyhow!("No plugin entrypoint files found"));
		}

		Ok(this)
	}

	pub fn get_config(&self) -> anyhow::Result<&Config> {
		if let Some(cfg) = self.config.get() {
			return Ok(cfg);
		}

		let config_data = self.dir.read_to_string(Self::PLUGIN_CONFIG)?;
		let config: Config = toml::from_str(&config_data)?;
		self.config.set(config).expect("Shouldn't be set");

		Ok(self.config.get().expect("Should be set"))
	}
}

nestify::nest! {
	#[derive(Debug, Serialize, Deserialize)]*
	pub struct Config {
		pub plugin: pub struct ConfigPlugin {
			pub name: String,
			pub author: String,
			pub version: String,
			pub description: String,
			pub language: #[serde(rename_all = "lowercase")] pub enum ConfigPluginLanguage {
				Lua,
				Native,
			}
		}
	}
}
