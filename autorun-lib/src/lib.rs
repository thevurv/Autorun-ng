use autorun_types::Realm;

mod events;
mod hooks;
mod lua_queue;
mod server;

pub fn main() -> anyhow::Result<()> {
	// Start IPC server in background thread
	std::thread::spawn(|| {
		if let Err(e) = server::start() {
			eprintln!("Failed to start IPC server: {}", e);
		}
	});

	// Wait for menu to be ready, then run event
	std::thread::spawn(|| {
		loop {
			std::thread::sleep(std::time::Duration::from_millis(500));

			if let Some(menu) = autorun_interfaces::lua::get_state(Realm::Menu).unwrap() {
				autorun_log::info!("Menu exists?");

				if let Err(why) = events::menu::run(menu) {
					autorun_log::error!("Failed to run menu event: {why}");
				}

				break;
			}
		}
	});

	// hooks::paint_traverse::init()?;
	hooks::load_buffer::init()?;
	// hooks::the_fn::init()?;

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

				if let Err(why) = main() {
					autorun_log::error!("{why}");
				}
			}
		}
	}
}
