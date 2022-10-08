use super::{error::AttachError, Autorun, Status};
use std::{path::Path, process::Child};

#[cfg(unix)]
fn launch_gmod(gmod: &Path) -> Result<Child, AttachError> {
	// todo: maybe also support hl2_darwin

	let exe_path = gmod.join("hl2_linux");

	let payload = std::env::current_dir()?.join("payload.dll");

	std::env::set_var("LD_PRELOAD", payload);

	let ret = std::process::Command::new(exe_path).spawn()?;

	Ok(ret)
}

#[cfg(windows)]
fn launch_gmod(gmod: &Path) -> Result<Child, AttachError> {
	use injector::{Injection, Injector};

	let exe_path = gmod.join("hl2.exe");
	let ret = std::process::Command::new(exe_path).spawn()?;

	let payload = std::env::current_dir()?.join("payload.dll");

	let mut inj = Injector::new();
	if let Err(why) = inj.inject(ret.id(), &payload) {
		println!("Failed to inject: {why:?}");
	}

	Ok(ret)
}

impl Autorun {
	/// Launches the game with Autorun injected.
	pub fn launch_attached(&mut self) -> Result<(), AttachError> {
		let mut steam_dir = steamlocate::SteamDir::locate().ok_or(AttachError::LibraryNotFound)?;

		let gmod = steam_dir.app(&4000).ok_or(AttachError::GameNotFound)?;

		let mut gmod = launch_gmod(&gmod.path)?;
		self.set_status(Status::Injected);

		Ok(())
	}

	/// Detaches from the spawned Garry's Mod process.
	/// # Note
	/// This is called when the ui closes.
	/// Normally I'd use Drop to call this but for some reason it's being called despite egui still running the app.
	/// wtf
	pub fn detach(&mut self) -> Result<(), AttachError> {
		println!("Unloading hooks");

		Ok(())
	}
}
