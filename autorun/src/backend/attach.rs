use anyhow::anyhow;

use super::{Autorun, AutorunStatus};
use std::{path::Path, process::Child};

#[cfg(unix)]
fn launch_gmod(_gmod_dir: &Path) -> anyhow::Result<Child> {
	let payload = std::env::current_dir()?.join("payload.so");

	println!("payload is {:?}", payload.display());

	let ret = std::process::Command::new("steam")
		.env("LD_PRELOAD", payload)
		.arg("steam://rungameid/4000")
		.spawn()?;

	Ok(ret)
}

impl Autorun {
	/// Launches the game with Autorun injected.
	pub fn launch_attached(&mut self) -> anyhow::Result<()> {
		let gmod_dir = locator::gmod_dir().ok_or(anyhow!("Game not found"))?;

		launch_gmod(&gmod_dir)?;
		self.set_status(AutorunStatus::Injected);

		Ok(())
	}

	/// Detaches from the spawned Garry's Mod process.
	/// # Note
	/// This is called when the ui closes.
	/// Normally I'd use Drop to call this but for some reason it's being called despite egui still running the app.
	/// wtf
	pub fn detach(&mut self) -> anyhow::Result<()> {
		println!("Unloading hooks");

		Ok(())
	}
}
