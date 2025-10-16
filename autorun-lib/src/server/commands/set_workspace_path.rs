use autorun_ipc::Message;

pub fn handle(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::SetWorkspacePath(text) = message else {
		return Err(anyhow::anyhow!("Expected SetWorkspacePath message"));
	};

	crate::events::set_workspace_path(&text)?;

	Ok(())
}
