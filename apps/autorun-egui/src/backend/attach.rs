use crate::util;

use super::Autorun;

impl Autorun {
	pub fn launch_game(&mut self) -> anyhow::Result<()> {
		autorun_steam::gmod::launch(util::get_payload_path()?)?;
		Ok(())
	}
}
