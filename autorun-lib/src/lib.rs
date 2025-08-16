mod exports;

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

	Ok(())
}

fn start_ipc_server() -> anyhow::Result<()> {
	let server = Server::start()?;
	println!("IPC server started");

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

fn handle_client(messenger: &mut autorun_ipc::Messenger) -> anyhow::Result<()> {
	loop {
		match messenger.receive() {
			Ok(message) => {
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
						unsafe {
							if let Ok(tier0) = libloading::Library::new("libtier0_client.so") {
								if let Ok(msg_func) =
									tier0.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
								{
									let output = format!("Executing: {}", code);
									if let Ok(c_text) = std::ffi::CString::new(output) {
										msg_func(c_text.as_ptr());
									}
								}
							}
						}
					}
					Message::Shutdown => {
						println!("Received shutdown request");
						break;
					}
				}
			}
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
