mod server;

pub fn main() -> anyhow::Result<()> {
	// Start IPC server in background thread
	std::thread::spawn(|| {
		if let Err(e) = server::start() {
			eprintln!("Failed to start IPC server: {}", e);
		}
	});

	Ok(())
}

#[ctor::ctor]
fn on_library_load() {
	match std::env::current_exe() {
		Err(why) => {
			eprintln!("Failed to get current exe path: {:?}", why);
		}
		Ok(exe) => {
			// Ensure LD_PRELOAD doesn't affect other programs
			// Without this, iirc steam messes up
			if exe.file_name() == Some(std::ffi::OsStr::new("gmod")) {
				unsafe { std::env::remove_var("LD_PRELOAD") };
				main();
			}
		}
	}
}
