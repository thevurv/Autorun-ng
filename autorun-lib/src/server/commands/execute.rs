use anyhow::anyhow;
use autorun_ipc::Message;

use crate::lua;

pub fn handle_execute(
	messenger: &mut autorun_ipc::Messenger,
	message: Message,
) -> anyhow::Result<()> {
	let Message::RunCode(code) = message else {
		return Err(anyhow::anyhow!("Expected Print message"));
	};

	let lua = lua::get_api()?;
	let menu_state = lua::get_menu_state()?.ok_or_else(|| anyhow!("Menu state isn't ready"))?;
	let c_text = std::ffi::CString::new(code.clone())?;

	if lua.load_string(menu_state, c_text.as_ptr()) == 0 {
		if lua.pcall(menu_state, 0, 0, 0) != 0 {
			eprintln!("Failed to execute Lua code");
		}
	} else {
		eprintln!("Failed to load Lua string");
	}

	Ok(())
}
