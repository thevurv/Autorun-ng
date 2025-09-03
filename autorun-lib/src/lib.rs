mod lua;

use autorun_ipc::{Message, Server};
use std::os::raw::c_char;
use std::thread;

pub fn main() -> anyhow::Result<()> {
	// Start IPC server in background thread
	thread::spawn(|| {
		if let Err(e) = start_ipc_server() {
			eprintln!("Failed to start IPC server: {}", e);
		}
	});

	thread::spawn(|| loop {
		if let Ok(Some(menu_state)) = lua::get_menu_state() {
			let api = lua::get_api().expect("Failed to get api");

			api.get_global(menu_state, c"print".as_ptr());
			api.push_string(menu_state, c"Hello from Rust, Lua!".as_ptr());

			if api.pcall(menu_state, 1, 0, 0) != 0 {
				eprintln!("Failed to call Lua print function");
			}
		}

		std::thread::sleep(std::time::Duration::from_secs(1));
	});

	Ok(())
}

fn start_ipc_server() -> anyhow::Result<()> {
	let server = Server::start()?;

	loop {
		match server.accept() {
			Ok(mut messenger) => {
				thread::spawn(move || {
					if let Err(e) = handle_client(&mut messenger) {
						eprintln!("Client handler error: {}", e);
					}
				});
			}

			Err(e) => {
				eprintln!("Failed to accept client: {}", e);
				thread::sleep(std::time::Duration::from_millis(100));
			}
		}
	}
}

fn handle_message(messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	match message {
		Message::Pong => (),
		Message::Ping => {
			messenger.send(Message::Pong)?;
		}

		Message::Print(text) => {
			unsafe {
				if let Ok(tier0) = libloading::Library::new("libtier0_client.so") {
					if let Ok(msg_func) =
						tier0.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
					{
						// Create a C string from the text
						if let Ok(c_text) = std::ffi::CString::new(text.clone()) {
							msg_func(c_text.as_ptr());
						}
					}
				}
			}
		}

		Message::RunCode(code) => {
			// TODO: Implement actual Lua code execution here
			// For now, just print the code using the game's Msg function
			let menu_state = lua::get_menu_state()?;
		}

		_ => (),
	}

	Ok(())
}

fn handle_client(messenger: &mut autorun_ipc::Messenger) -> anyhow::Result<()> {
	loop {
		match messenger.receive() {
			Ok(Message::Shutdown) => break,
			Ok(message) => handle_message(messenger, message)?,
			Err(e) => {
				eprintln!("Failed to receive message: {}", e);
				break;
			}
		}
	}

	println!("Client disconnected");
	Ok(())
}

#[ctor::ctor]
fn on_library_load() {
	match std::env::current_exe() {
		Err(why) => {
			eprintln!("Failed to get current exe path: {:?}", why);
		}
		Ok(exe) => {
			if exe.file_name() == Some(std::ffi::OsStr::new("gmod")) {
				std::env::remove_var("LD_PRELOAD");
				main();
			}
		}
	}
}
