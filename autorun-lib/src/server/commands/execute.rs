use anyhow::anyhow;
use autorun_ipc::Message;

use crate::lua;

pub fn handle_execute(
	messenger: &mut autorun_ipc::Messenger,
	message: Message,
) -> anyhow::Result<()> {
	let Message::RunCode(realm, code) = message else {
		return Err(anyhow::anyhow!("Expected Print message"));
	};

	let lua = lua::get_api()?;
	let state = lua::get_state(realm)?.ok_or_else(|| anyhow!("State isn't ready"))?;
	let c_text = std::ffi::CString::new(code.clone())?;

	if let Err(why) = lua.load_string(state, c_text.as_ptr()) {
		return Err(anyhow!("Failed to load Lua string: {}", why));
	}

	let existing_hook = lua.get_hook(state);
	let existing_hook_info = if existing_hook.is_null() {
		None
	} else {
		Some((
			existing_hook,
			lua.get_hook_mask(state),
			lua.get_hook_count(state),
		))
	};

	if existing_hook_info.is_some() {
		lua.set_hook(state, std::ptr::null(), 0, 0);
	}

	if let Err(why) = lua.pcall(state, 0, 0, 0) {
		return Err(anyhow!("Failed to execute Lua code: {why}"));
	}

	let did_user_set_hook = !lua.get_hook(state).is_null();
	if did_user_set_hook {
		autorun_log::warn!("User set a hook in executed code. This is not recommended.");
	}

	if !did_user_set_hook && let Some((hook, mask, count)) = existing_hook_info {
		lua.set_hook(state, hook, mask, count);
	}

	Ok(())
}
