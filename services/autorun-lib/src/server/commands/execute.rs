use autorun_ipc::Message;
use autorun_log::*;

pub fn handle(_messenger: &mut autorun_ipc::Messenger, message: Message) -> anyhow::Result<()> {
	let Message::RunCode(realm, code) = message else {
		anyhow::bail!("Expected RunCode message");
	};

	if autorun_interfaces::lua::get_state(realm)?.is_none() {
		error!("Lua state for realm {realm:?} is not ready");
		return Ok(());
	}

	crate::lua_queue::push(move |lua| {
		let state = autorun_interfaces::lua::get_state(realm)?.unwrap();
		let env = autorun_env::global::get_realm_env(realm).ok_or(anyhow::anyhow!("Failed to get env"))?;

		env.execute(lua, state, c"RunString", code.as_bytes())?;

		Ok(())
	});

	Ok(())
}
