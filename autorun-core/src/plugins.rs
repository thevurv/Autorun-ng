use serde::{Deserialize, Serialize};

pub struct Plugin {
	dir: std::path::PathBuf,
	config: std::sync::OnceLock<Config>,
}

impl Plugin {
	const PLUGIN_CONFIG: &str = "plugin.toml";

	const PLUGIN_SRC: &str = "src";

	const PLUGIN_MENU_FILE: &str = "menu.lua";
	const PLUGIN_INIT_FILE: &str = "init.lua";
	const PLUGIN_HOOK_FILE: &str = "hook.lua";

	pub fn from_dir(p: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
		let p = p.as_ref();
		if !p.is_dir() {
			return Err(anyhow::anyhow!("Plugin path is not a directory"));
		}

		Ok(Self {
			dir: p.to_path_buf(),
			config: std::sync::OnceLock::new(),
		})
	}

	pub fn get_menu_file(&self) -> Option<std::path::PathBuf> {
		let file = self.dir.join(Self::PLUGIN_SRC).join(Self::PLUGIN_MENU_FILE);
		if file.exists() { Some(file) } else { None }
	}

	pub fn get_init_file(&self) -> Option<std::path::PathBuf> {
		let file = self.dir.join(Self::PLUGIN_SRC).join(Self::PLUGIN_INIT_FILE);
		if file.exists() { Some(file) } else { None }
	}

	pub fn get_hook_file(&self) -> Option<std::path::PathBuf> {
		let file = self.dir.join(Self::PLUGIN_SRC).join(Self::PLUGIN_HOOK_FILE);
		if file.exists() { Some(file) } else { None }
	}

	pub fn get_config(&self) -> anyhow::Result<&Config> {
		if let Some(cfg) = self.config.get() {
			return Ok(cfg);
		}

		let config_path = self.dir.join(Self::PLUGIN_CONFIG);
		if !config_path.exists() {
			return Err(anyhow::anyhow!("Plugin config not found at {:?}", config_path));
		}

		let config_data = std::fs::read_to_string(&config_path)?;
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
