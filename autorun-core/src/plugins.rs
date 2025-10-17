use cap_std::fs::{Dir, File};
use serde::{Deserialize, Serialize};

pub struct Plugin {
	dir: Dir,
	config: std::sync::OnceLock<Config>,
}

impl Plugin {
	const PLUGIN_CONFIG: &str = "plugin.toml";

	const PLUGIN_SRC: &str = "src";

	const PLUGIN_MENU_FILE: &str = "menu.lua";
	const PLUGIN_INIT_FILE: &str = "init.lua";
	const PLUGIN_HOOK_FILE: &str = "hook.lua";

	pub fn dir(&self) -> &Dir {
		&self.dir
	}

	pub fn src(&self) -> std::io::Result<Dir> {
		self.dir.open_dir(Self::PLUGIN_SRC)
	}

	pub fn read_init_lua(&self) -> std::io::Result<Vec<u8>> {
		self.src()?.read(Self::PLUGIN_INIT_FILE)
	}

	pub fn read_hook_lua(&self) -> std::io::Result<Vec<u8>> {
		self.src()?.read(Self::PLUGIN_HOOK_FILE)
	}

	pub fn read_menu_lua(&self) -> std::io::Result<Vec<u8>> {
		self.src()?.read(Self::PLUGIN_MENU_FILE)
	}

	pub fn from_dir(dir: Dir) -> anyhow::Result<Self> {
		if !dir.exists(Self::PLUGIN_CONFIG) {
			return Err(anyhow::anyhow!("Plugin config not found"));
		}

		let Ok(src) = dir.open_dir(Self::PLUGIN_SRC) else {
			return Err(anyhow::anyhow!("Plugin src directory not found"));
		};

		if !src.exists(Self::PLUGIN_MENU_FILE) && !src.exists(Self::PLUGIN_INIT_FILE) && !src.exists(Self::PLUGIN_HOOK_FILE) {
			return Err(anyhow::anyhow!("No plugin entrypoint files found"));
		}

		Ok(Self {
			dir,
			config: std::sync::OnceLock::new(),
		})
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
