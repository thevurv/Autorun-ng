use serde::{Deserialize, Serialize};

use crate::{Workspace, plugins};

nestify::nest! {
	#[derive(Debug, Deserialize, Serialize, PartialEq)]*
	pub struct Settings {
		pub autorun: pub struct AutorunSettings {
			pub check_version: bool
		}
	}
}

impl Workspace {
	fn read_settings(&self) -> anyhow::Result<Settings> {
		let content = self.path.read("settings.toml")?;
		let settings = toml::from_slice::<Settings>(&content)?;
		Ok(settings)
	}

	pub fn get_settings(&self) -> anyhow::Result<&Settings> {
		if let Some(settings) = self.settings.get() {
			return Ok(settings);
		}

		let settings = self.read_settings()?;
		self.settings.set(settings).expect("Shouldn't exist");

		Ok(self.settings.get().expect("Should exist"))
	}

	/// Retrieves all plugins (configs lazily loaded).
	/// Returns a tuple of (plugins, errors).
	pub fn get_plugins(&self) -> anyhow::Result<(Vec<plugins::Plugin>, Vec<anyhow::Error>)> {
		let mut plugins = Vec::new();
		let mut errors = Vec::new();

		for entry in self.plugins()?.read_dir(".")? {
			let entry = entry?;
			if entry.file_type()?.is_dir() {
				match plugins::Plugin::from_dir(entry.open_dir()?) {
					Ok(plugin) => plugins.push(plugin),
					Err(e) => errors.push(e),
				}
			}
		}

		Ok((plugins, errors))
	}
}
