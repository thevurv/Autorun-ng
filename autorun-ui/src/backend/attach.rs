use crate::util;

use super::Autorun;

impl Autorun {
	pub fn launch_game(&mut self) -> anyhow::Result<()> {
		autorun_steam::gmod::launch(util::get_payload_path()?)?;
		Ok(())
	}

	/// Attempts to connect to the game's IPC server
	pub fn try_connect_to_game(&mut self) -> anyhow::Result<()> {
		self.try_connect()
	}
}
