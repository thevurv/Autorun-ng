mod attach;
mod exec;

use autorun_ipc::{Client, Message};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[non_exhaustive]
#[derive(Default, Clone, Copy, PartialEq)]
pub enum AutorunStatus {
	#[default]
	Disconnected,
	Connected,
}

/// The Autorun state.
pub struct Autorun {
	status: AutorunStatus,
	workspace: autorun_core::Workspace,
	client: Option<Arc<Mutex<Client>>>,
	last_connection_attempt: Option<Instant>,
	last_ping_time: Option<Instant>,
}

impl Autorun {
	pub fn new() -> anyhow::Result<Self> {
		Ok(Self {
			status: AutorunStatus::Disconnected,
			workspace: autorun_core::Workspace::from_exe()?,
			client: None,
			last_connection_attempt: None,
			last_ping_time: None,
		})
	}

	pub fn workspace(&self) -> &autorun_core::Workspace {
		&self.workspace
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

		// Check if existing connection is still alive (only every 5 seconds)
		if self.status == AutorunStatus::Connected {
			let should_ping = match self.last_ping_time {
				Some(last) => last.elapsed() > Duration::from_secs(5),
				None => true,
			};

			if should_ping {
				self.last_ping_time = Some(Instant::now());

				let should_disconnect = if let Some(ref client) = self.client {
					// Use try_lock to avoid blocking the UI
					if let Ok(mut client) = client.try_lock() {
						// Try to send a ping to check connection
						client.send(Message::Ping).is_err()
					} else {
						// If we can't get the lock, assume connection is busy but alive
						false
					}
				} else {
					true
				};

				if should_disconnect {
					self.set_status(AutorunStatus::Disconnected);
					self.client = None;
				}
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

				// Send initial workspace path
				self.send_message(Message::SetWorkspacePath(
					self.workspace().unsafe_raw_path.to_string_lossy().to_string(),
				))?;

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
			// Use try_lock to avoid blocking the UI thread
			if let Ok(mut client) = client.try_lock() {
				client.send(message)?;
			} else {
				return Err(anyhow::anyhow!("Client is busy"));
			}
		}
		Ok(())
	}

	pub fn detach(&mut self) -> anyhow::Result<()> {
		if let Some(ref client) = self.client {
			// Use try_lock to avoid blocking the UI thread
			if let Ok(mut client) = client.try_lock() {
				let _ = client.send(Message::Shutdown);
			}
		}

		self.client = None;
		self.status = AutorunStatus::Disconnected;
		Ok(())
	}
}
