use autorun_types::Realm;

mod events;
mod hooks;
mod lua_queue;
mod menu;
mod server;

pub fn main() -> anyhow::Result<()> {
	// Start IPC server in background thread
	std::thread::spawn(|| {
		if let Err(e) = server::start() {
			eprintln!("Failed to start IPC server: {}", e);
		}
	});

	// hooks::paint_traverse::init()?;
	hooks::load_buffer::init()?;
	// hooks::the_fn::init()?;

	Ok(())
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
extern "C" fn autorun_entrypoint() {
	// Redirect stdout to stderr.
	// This is a hack because for some reason stdout isn't intercepted on windows?
	// Might be gmod's fault. I don't care.
	unsafe {
		use windows::Win32::System::Console::*;
		let stderr_handle = GetStdHandle(STD_ERROR_HANDLE).unwrap();
		SetStdHandle(STD_OUTPUT_HANDLE, stderr_handle).ok();
	}

	if let Err(why) = main() {
		autorun_log::error!("{why}");
	}
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

				if let Err(why) = main() {
					autorun_log::error!("{why}");
				}
			}
		}
	}
}
