use autorun_ipc::Message;

pub fn handle(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::SetWorkspacePath(text) = message else {
		return Err(anyhow::anyhow!("Expected SetWorkspacePath message"));
	};

	let had_workspace_path = crate::events::get_workspace().is_ok();
	crate::events::set_workspace_path(&text)?;

	// if !had_workspace_path {
	// 	crate::menu::start_waiting_for_menu();
	// }

	Ok(())
}
