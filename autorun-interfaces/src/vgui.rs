use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void};

use autorun_types::LuaState;

#[repr(C)]
pub struct Panel {
	pub vtable: *const PanelVTable,
}

#[repr(C)]
pub struct PanelVTable {
	#[cfg(target_os = "linux")]
	rtti: *const c_void,

	padding: [*const c_void; 40],
	pub paint_traverse: extern "C" fn(this: *const c_void, panel: u32, force_repaint: bool, allow_force: bool),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetRawApiError {
	#[error("Failed to get interface: {0}")]
	GetInterface(#[from] crate::util::GetInterfaceError),
}

#[derive(Debug)]
pub struct VGUIApi {
	pub vgui: *mut Panel,
}

impl VGUIApi {
	pub fn new(vgui: *mut Panel) -> Self {
		Self { vgui }
	}
}

// I pinky promise the panel won't just vanish out of thin air :D
unsafe impl Send for VGUIApi {}
unsafe impl Sync for VGUIApi {}

static VGUI2_API: std::sync::OnceLock<VGUIApi> = std::sync::OnceLock::new();

pub fn get_api() -> Result<&'static VGUIApi, GetRawApiError> {
	if let Some(api) = VGUI2_API.get() {
		return Ok(api);
	}

	let vgui = crate::util::get_interface("vgui2_client.so", c"VGUI_Panel009")? as *mut Panel;
	let vgui = VGUIApi::new(vgui);

	VGUI2_API.set(vgui).expect("Should never already be initialized");

	Ok(VGUI2_API.get().expect("Should be initialized"))
}
