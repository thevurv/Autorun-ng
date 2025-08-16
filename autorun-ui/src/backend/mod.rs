mod attach;
mod exec;

use anyhow;
use autorun_ipc::{Client, Message};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq)]
pub enum AutorunStatus {
	Disconnected,
	Connected,
}

impl Default for AutorunStatus {
	fn default() -> Self {
		Self::Disconnected
	}
}

/// The Autorun state.
#[derive(Default)]
pub struct Autorun {
	status: AutorunStatus,
	client: Option<Arc<Mutex<Client>>>,
	last_connection_attempt: Option<Instant>,
}

impl Clone for Autorun {
	fn clone(&self) -> Self {
		Self {
			status: self.status,
			client: self.client.clone(),
			last_connection_attempt: self.last_connection_attempt,
		}
	}
}

impl Autorun {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn status(&self) -> AutorunStatus {
		self.status
	}

	pub fn set_status(&mut self, status: AutorunStatus) {
		self.status = status;
	}

	pub fn update(&mut self) {
		// Check if we should try to reconnect
		if self.status == AutorunStatus::Disconnected {
			let should_attempt = match self.last_connection_attempt {
				Some(last) => last.elapsed() > Duration::from_secs(2),
				None => true,
			};

			if should_attempt {
				self.last_connection_attempt = Some(Instant::now());
				let _ = self.try_connect();
			}
		}

		// Check if existing connection is still alive
		if self.status == AutorunStatus::Connected {
			let should_disconnect = if let Some(ref client) = self.client {
				if let Ok(mut client) = client.try_lock() {
					// Try to send a ping to check connection
					client.send(Message::Ping).is_err()
				} else {
					false
				}
			} else {
				true
			};

			if should_disconnect {
				self.status = AutorunStatus::Disconnected;
				self.client = None;
			}
		}
	}

	pub fn try_connect(&mut self) -> anyhow::Result<()> {
		match Client::connect() {
			Ok(client) => {
				self.client = Some(Arc::new(Mutex::new(client)));
				self.status = AutorunStatus::Connected;

				// Send a ping to verify connection
				if let Some(ref client) = self.client {
					let client_clone = Arc::clone(client);
					thread::spawn(move || {
						if let Ok(mut client) = client_clone.lock() {
							let _ = client.send(Message::Ping);
						}
					});
				}

				Ok(())
			}
			Err(e) => {
				self.status = AutorunStatus::Disconnected;
				Err(e)
			}
		}
	}

	pub fn send_message(&self, message: Message) -> anyhow::Result<()> {
		if let Some(ref client) = self.client {
			let mut client = client.lock().unwrap();
			client.send(message)?;
		}
		Ok(())
	}

	pub fn detach(&mut self) -> anyhow::Result<()> {
		if let Some(ref client) = self.client {
			let mut client = client.lock().unwrap();
			let _ = client.send(Message::Shutdown);
		}
		self.client = None;
		self.status = AutorunStatus::Disconnected;
		Ok(())
	}
}
