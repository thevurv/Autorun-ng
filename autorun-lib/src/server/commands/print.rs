use autorun_ipc::Message;

pub fn handle(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::Print(text) = message else {
		return Err(anyhow::anyhow!("Expected Print message"));
	};

	let tier0 = autorun_interfaces::tier0::get_api()?;
	let c_text = std::ffi::CString::new(text.clone())?;

	tier0.msg(c_text.as_ptr());

	Ok(())
}
