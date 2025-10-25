use autorun_ipc::Message;

use crate::lua_queue::LUA_QUEUE;

pub fn handle(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::RunCode(realm, code) = message else {
		return Err(anyhow::anyhow!("Expected RunCode message"));
	};

	if autorun_interfaces::lua::get_state(realm)?.is_none() {
		autorun_log::error!("Lua state for realm {realm:?} is not ready");
		return Ok(());
	}

	LUA_QUEUE.lock().unwrap().push((realm, code));

	Ok(())
}
