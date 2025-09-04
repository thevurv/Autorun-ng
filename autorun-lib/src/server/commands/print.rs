use autorun_ipc::Message;

use crate::tier0;

pub fn handle_print_command(
	messenger: &mut autorun_ipc::Messenger,
	message: Message,
) -> anyhow::Result<()> {
	let Message::Print(text) = message else {
		return Err(anyhow::anyhow!("Expected Print message"));
	};

	let tier0 = tier0::get_api()?;
	let c_text = std::ffi::CString::new(text.clone())?;

	tier0.msg(c_text.as_ptr());

	Ok(())
}
