use anyhow::anyhow;
use autorun_ipc::Message;

use crate::lua_queue::LUA_QUEUE;

pub fn handle_execute(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::RunCode(realm, code) = message else {
		return Err(anyhow::anyhow!("Expected Print message"));
	};

	if autorun_interfaces::lua::get_state(realm)?.is_none() {
		autorun_log::error!("Lua state for realm {realm:?} is not ready");
		return Ok(());
	}

	let c_text = std::ffi::CString::new(code.clone())?;
	LUA_QUEUE.lock().unwrap().push((realm, c_text));

	Ok(())
}
