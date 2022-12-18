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

/// Time to wait for gmod to start up after running the process to inject.
#[cfg(windows)]
const GMOD_WAIT_TIME: std::time::Duration = std::time::Duration::from_millis(500);

#[cfg(windows)]
fn launch_gmod(gmod: &Path) -> Result<(), AttachError> {
	use super::injector::inject;

	// hl2.exe if user is on 32 bit branch and hasn't gone on the 64 bit branch before.
	// bin/gmod.exe if user is on 32 bit branch and has gone on the 64 bit branch before.
	// bin/win64/gmod.exe if user is on 64 bit branch

	#[cfg(target_arch = "x86_64")] // 64 bit assumes you're on the 64 bit branch
	let gmod_exe = gmod.join("bin/win64/gmod.exe");

	#[cfg(not(target_arch = "x86_64"))] // 32 bit assumes no branch
	let gmod_exe = gmod.join("hl2.exe");

	let cmd = std::process::Command::new(&gmod_exe).spawn()?;

	std::thread::sleep(GMOD_WAIT_TIME);

	let path = std::env::current_dir()?.join("payload.dll");
	println!("{:#?}", path);

	// This will return an error if compiling on i686-pc-windows-msvc branch yet using 64 bit branch on gmod, as hl2.exe acts as a launcher to another exe.
	println!("{:#?}", inject(cmd.id(), path));

	Ok(())
}

impl Autorun {
	/// Launches the game with Autorun injected.
	pub fn launch_attached(&mut self) -> Result<(), AttachError> {
		let gmod_dir = locator::gmod_dir().ok_or(AttachError::GameNotFound)?;

		launch_gmod(&gmod_dir)?;
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
