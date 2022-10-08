use sysinfo::{ProcessRefreshKind, SystemExt};

mod attach;
pub mod error;
mod exec;

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Status {
	/// First created
	Creation,

	/// Loaded configs
	Settings,

	/// Injected into the game
	Injected,
}

impl Default for Status {
	fn default() -> Self {
		Self::Creation
	}
}

impl Status {
	pub fn is_ready(&self) -> bool {
		match *self {
			Status::Creation => false,
			Status::Settings => false,
			Status::Injected => true,
		}
	}

	pub fn as_str(&self) -> &str {
		match *self {
			Status::Creation => "Creation",
			Status::Settings => "Settings",
			Status::Injected => "Injected",
		}
	}
}

/// The Autorun state.
#[derive(Default)]
pub struct Autorun {
	status: Status,
	system: sysinfo::System,
}

impl Autorun {
	pub fn new() -> Self {
		use sysinfo::RefreshKind;
		let sys = sysinfo::System::new_with_specifics(
			RefreshKind::new().with_processes(ProcessRefreshKind::new().with_user()),
		);

		Self {
			system: sys,
			status: Status::Creation,
		}
	}

	pub fn status(&self) -> Status {
		self.status
	}

	pub fn set_status(&mut self, status: Status) {
		self.status = status;
	}

	pub fn load_settings(&mut self) -> Result<(), error::SettingsError> {
		self.set_status(Status::Settings);
		Ok(())
	}
}
