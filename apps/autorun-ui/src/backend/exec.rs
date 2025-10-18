use super::Autorun;
use autorun_ipc::Message;

impl Autorun {
	pub fn run_code(&mut self, realm: autorun_types::Realm, code: impl Into<String>) -> anyhow::Result<()> {
		let code = code.into();

		if self.client.is_none() {
			return Err(anyhow::anyhow!("Not connected to autorun server"));
		}

		self.send_message(Message::RunCode(realm, code))?;
		Ok(())
	}

	#[allow(unused)]
	pub fn print_to_game(&self, text: impl Into<String>) -> anyhow::Result<()> {
		let text = text.into();

		if self.client.is_none() {
			return Err(anyhow::anyhow!("Not connected to autorun server"));
		}

		self.send_message(Message::Print(text))?;
		Ok(())
	}
}
