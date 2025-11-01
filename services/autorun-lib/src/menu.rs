/// Start a thread waiting for the menu to be ready to then run the menu init event.
/// This is to avoid a race condition where the menu starts before IPC is ready.
pub fn start_waiting_for_menu() {
	// Wait for menu to be ready, then run event
	std::thread::spawn(|| {
		loop {
			std::thread::sleep(std::time::Duration::from_millis(500));

			if let Some(menu) = autorun_interfaces::lua::get_state(autorun_types::Realm::Menu).unwrap() {
				let menu = menu as usize;
				crate::lua_queue::push(move |_| crate::events::menu_init::run(menu as *mut core::ffi::c_void));

				break;
			}
		}
	});
}
