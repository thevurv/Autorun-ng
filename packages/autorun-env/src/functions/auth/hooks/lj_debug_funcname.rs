use anyhow::Context;
use autorun_luajit::{Frame, LJState, TValue};

#[cfg(target_os = "windows")]
pub const TARGET_MODULE: &str = "lua_shared.dll";

#[cfg(target_os = "linux")]
pub const TARGET_MODULE: &str = "lua_shared_client.so";

#[cfg(target_os = "windows")]
pub const LJ_DEBUG_FUNCNAME_SIG: &str =
	"48 89 5c 24 08 48 89 74 24 10 57 48 83 ec 20 48 8b 41 38 49 8b f0 48 83 c0 08 48 8b d9 48 3b d0";

#[cfg(target_os = "linux")]
pub const LJ_DEBUG_FUNCNAME_SIG: &str = "41 55 41 54 55 48 89 fd 53 48 83 ec 08 48 8b 47 38 48 83 c0 08 48 39 c6 0f ?? ?? ?? ?? ?? 48 8b 06 49 89 d5 48 89 c2 83 e2 07 48 83 fa 03";

/// Number of frames to stitch across for Autorun. This is computed based on the typical
/// frame structure for `Autorun.copyFastFunction(foo, function(...) Autorun.safeCall(...) end)`,
pub const STITCHED_AUTORUN_FRAMES: usize = 4;
pub const MINIMUM_STACK_FRAMES: usize = 4;

type LjDebugFuncnameFn = unsafe extern "C" fn(state: *mut LJState, frame: *mut TValue, name: *const *const u8) -> *const u8;

static LJ_DEBUG_FUNCNAME_HOOK: std::sync::OnceLock<retour::GenericDetour<LjDebugFuncnameFn>> = std::sync::OnceLock::new();

/// Hook for `lj_debug_funcname` to stitch across Autorun frames,
/// enabling LuaJIT to properly identify function names in the call stack.
///
/// This is particularly useful for avoiding detection by anti-cheat systems that monitor the call stack for unauthorized code.
/// Detection can occur if a function name is missing and replaced with `?` when Autorun is in use, as opposed to
/// a proper function name when Autorun is not present.
///
/// This hook attempts to find the correct frame for `lj_debug_funcname` to use by walking the stack, locating the original frame,
/// and stitching across the two Autorun frames that are typically present.
///
/// It only activates in the situation where there are at least `MINIMUM_STACK_FRAMES` frames in the stack, and
/// the original function name was not found (i.e., the first call to the original `lj_debug_funcname` returned null).
///
/// This helps ensure that legitimate function names are returned in any legitimate scenarios, while still providing the stitching functionality
/// for Autorun-involved calls. Ideally, we could be more precise about when to stitch, but somehow we would need to identify
/// which frames belong to Autorun specifically, which is non-trivial at this point because we don't have reliable metadata about the frames.
///
/// We also cannot set a flag or anything because we forward errors which longjmps back to LuaJIT code, which means we can not control the ending state
extern "C" fn lj_debug_funcname_hook(state: *mut LJState, frame: *mut TValue, name: *const *const u8) -> *const u8 {
	let first_ret = unsafe { LJ_DEBUG_FUNCNAME_HOOK.get().unwrap().call(state, frame, name) };
	if first_ret != std::ptr::null() {
		// Never attempt to stitch if we already have a valid name.
		return first_ret;
	}

	let frames = Frame::walk_stack(state);
	if frames.len() < MINIMUM_STACK_FRAMES {
		return first_ret;
	}

	let mut matched_frame_index: Option<usize> = None;

	for (i, f) in frames.iter().enumerate() {
		if f.tvalue.eq(&frame) {
			matched_frame_index = Some(i);
		}
	}

	let mut target_frame = frame;
	if let Some(matched_index) = matched_frame_index {
		let new_index = matched_index + STITCHED_AUTORUN_FRAMES;
		target_frame = frames.get(new_index).map_or(frame, |f| f.tvalue);
	}

	let ret = unsafe { LJ_DEBUG_FUNCNAME_HOOK.get().unwrap().call(state, target_frame, name) };
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

	unsafe {
		let hook = retour::GenericDetour::<LjDebugFuncnameFn>::new(
			std::mem::transmute(lj_debug_funcname_addr as *const ()),
			lj_debug_funcname_hook,
		)
		.context("Failed to create lj_debug_funcname detour")?;

		unsafe {
			hook.enable().context("Failed to enable lj_debug_funcname hook")?;
		}

		LJ_DEBUG_FUNCNAME_HOOK
			.set(hook)
			.map_err(|_| anyhow::anyhow!("Failed to set LJ_DEBUG_FUNCNAME_HOOK"))?;
	}

	Ok(())
}
