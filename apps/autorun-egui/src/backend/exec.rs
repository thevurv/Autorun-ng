use super::Autorun;
use autorun_ipc::Message;

impl Autorun {
	pub fn run_code(&mut self, realm: autorun_types::Realm, code: impl Into<String>) -> anyhow::Result<()> {
		let code = code.into();

		if self.client.is_none() {
			anyhow::bail!("Not connected to autorun server");
		}

		self.send_message(Message::RunCode(realm, code))?;
		Ok(())
	}
}
