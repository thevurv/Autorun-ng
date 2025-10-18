/// Start a thread waiting for the menu to be ready to then run the menu init event.
/// This is to avoid a race condition where the menu starts before IPC is ready.
pub fn start_waiting_for_menu() {
	// Wait for menu to be ready, then run event
	std::thread::spawn(|| {
		loop {
			std::thread::sleep(std::time::Duration::from_millis(500));

			if let Some(menu) = autorun_interfaces::lua::get_state(autorun_types::Realm::Menu).unwrap() {
				if let Err(why) = crate::events::menu::run(menu) {
					autorun_log::error!("Failed to run menu event: {why}");
				}

				break;
			}
		}
	});
}
