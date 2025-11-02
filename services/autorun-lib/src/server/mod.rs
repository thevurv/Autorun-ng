//! Autorun's IPC Server.
//! This runs inside of the autorun library which will accept connections from autorun client UIs.
mod commands;

use autorun_ipc::{Message, Server};

pub fn start() -> anyhow::Result<()> {
	let server = Server::start()?;

	loop {
		match server.accept() {
			Ok(mut messenger) => {
				std::thread::spawn(move || {
					if let Err(e) = handle_client(&mut messenger) {
						autorun_log::error!("{e}");
					}
				});
			}

			Err(e) => {
				eprintln!("Failed to accept client: {}", e);
				std::thread::sleep(std::time::Duration::from_millis(100));
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

		Message::RunCode(..) => {
			commands::execute::handle(messenger, message)?;
		}

		Message::SetWorkspacePath(..) => {
			commands::set_workspace_path::handle(messenger, message)?;
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
