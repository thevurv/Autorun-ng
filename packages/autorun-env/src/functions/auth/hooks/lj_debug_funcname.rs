use anyhow::Context;
use autorun_lua::LuaState;
use autorun_luajit::{Frame, LJState, TValue, push_frame_func, push_tvalue};

#[cfg(target_os = "windows")]
pub const TARGET_MODULE: &str = "lua_shared.dll";

#[cfg(target_os = "linux")]
pub const TARGET_MODULE: &str = "lua_shared_client.so";

pub const LJ_DEBUG_FUNCNAME_SIG: &str =
	"48 89 5c 24 08 48 89 74 24 10 57 48 83 ec 20 48 8b 41 38 49 8b f0 48 83 c0 08 48 8b d9 48 3b d0";

type LjDebugFuncnameFn = unsafe extern "C" fn(state: *mut LJState, frame: *mut TValue, name: *const *const u8) -> *const u8;

static LJ_DEBUG_FUNCNAME_HOOK: std::sync::OnceLock<retour::GenericDetour<LjDebugFuncnameFn>> = std::sync::OnceLock::new();

extern "C" fn lj_debug_funcname_hook(state: *mut LJState, frame: *mut TValue, name: *const *const u8) -> *const u8 {
	let first_ret = unsafe { LJ_DEBUG_FUNCNAME_HOOK.get().unwrap().call(state, frame, name) };
	if first_ret != std::ptr::null() {
		autorun_log::debug!("lj_debug_funcname_hook: First call returned a valid name, waiting for next call.");
		return first_ret;
	}

	let frames = Frame::walk_stack(state);
	let mut matched_frame_index: Option<usize> = None;

	autorun_log::debug!("Current call stack frames:");
	for (i, f) in frames.iter().enumerate() {
		autorun_log::debug!(
			"Frame {}: {} - 0x{:x} (value: 0x{:x})",
			i,
			f.get_type(),
			f.tvalue as usize,
			unsafe { (*f.tvalue).ftsz }
		);

		if f.tvalue.eq(&frame) {
			autorun_log::debug!("--> Matched frame at index {}", i);
			matched_frame_index = Some(i);
		}
	}

	let mut target_frame = frame;

	if let Some(matched_index) = matched_frame_index {
		// we need to stitch across one more frame to get past the autorun frame
		let new_index = matched_index + 2;
		autorun_log::debug!("Stitching frames ({} -> {})", matched_index, new_index);
		autorun_log::debug!("Before stitching, target_frame: 0x{:x}", target_frame as usize);
		target_frame = frames.get(new_index).map_or(frame, |f| f.tvalue);
		autorun_log::debug!("After stitching, target_frame: 0x{:x}", target_frame as usize);
	} else {
		autorun_log::warn!("No matching frame found for lj_debug_funcname hook. Could not stitch frames.");
	}

	let ret = unsafe { LJ_DEBUG_FUNCNAME_HOOK.get().unwrap().call(state, target_frame, name) };
	// disable hook, it's one-time use
	let hook = LJ_DEBUG_FUNCNAME_HOOK.get().unwrap();
	unsafe {
		//hook.disable().expect("Failed to disable lj_debug_funcname hook");
	}

	ret
}

pub fn enable() -> anyhow::Result<()> {
	let hook = LJ_DEBUG_FUNCNAME_HOOK
		.get()
		.context("lj_debug_funcname hook is not initialized")?;
	unsafe {
		hook.enable().context("Failed to enable lj_debug_funcname hook")?;
	}

	Ok(())
}

pub fn disable() -> anyhow::Result<()> {
	let hook = LJ_DEBUG_FUNCNAME_HOOK
		.get()
		.context("lj_debug_funcname hook is not initialized")?;
	unsafe {
		hook.disable().context("Failed to disable lj_debug_funcname hook")?;
	}

	Ok(())
}

pub fn init() -> anyhow::Result<()> {
	if LJ_DEBUG_FUNCNAME_HOOK.get().is_some() {
		return Ok(());
	}

	let lj_debug_funcname_addr = autorun_scan::scan(autorun_scan::sig_byte_string(LJ_DEBUG_FUNCNAME_SIG), Some(TARGET_MODULE))
		.context("Failed to find lj_debug_funcname signature")?;
	let lj_debug_funcname_addr = lj_debug_funcname_addr.context("lj_debug_funcname address not found")?;

	autorun_log::info!("Found lj_debug_funcname at address: {:x}", lj_debug_funcname_addr);
	unsafe {
		let hook = retour::GenericDetour::<LjDebugFuncnameFn>::new(
			std::mem::transmute(lj_debug_funcname_addr as *const ()),
			lj_debug_funcname_hook,
		)
		.context("Failed to create lj_debug_funcname detour")?;

		// not enabled by default
		LJ_DEBUG_FUNCNAME_HOOK
			.set(hook)
			.map_err(|_| anyhow::anyhow!("Failed to set LJ_DEBUG_FUNCNAME_HOOK"))?;
		autorun_log::info!("lj_debug_funcname hook installed successfully");
	}

	Ok(())
}
