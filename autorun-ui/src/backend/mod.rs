mod attach;
mod exec;

use anyhow;

#[non_exhaustive]
#[derive(Clone, Copy)]
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

	pub fn detach(&mut self) -> anyhow::Result<()> {
		// TODO: Implement actual detach logic
		self.status = AutorunStatus::Disconnected;
		Ok(())
	}
}
