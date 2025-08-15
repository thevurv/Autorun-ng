mod attach;
mod exec;

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum AutorunStatus {
	Starting,
	Injected,
	Attached,
}

impl Default for AutorunStatus {
	fn default() -> Self {
		Self::Starting
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
}
