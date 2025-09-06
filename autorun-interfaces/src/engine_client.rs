use std::{ffi::c_void, os::raw::c_int};

use crate::net::INetChannelInfo;

#[repr(C)]
pub struct IEngineClient {
	pub vtable: *const IEngineClientVTable,
}

#[repr(C)]
pub struct IEngineClientVTable {
	#[cfg(target_os = "linux")]
	rtti: *const c_void,
	_padding: [*const c_void; 4],
	get_screen_size: extern "C" fn(this: *const c_void, width: *mut c_int, height: *mut c_int),
	_padding2: [*const c_void; 68],
	get_net_channel_info: extern "C" fn(this: *const c_void) -> *mut INetChannelInfo,
}

// 47 away from origin
// 8b 88 0c 10 00 00 85 c9 0f 95

#[derive(Debug)]
pub struct EngineClientApi {
	pub client: *mut IEngineClient,
}

// I pinky promise this won't disappear out of nowhere :)
unsafe impl Send for EngineClientApi {}
unsafe impl Sync for EngineClientApi {}

impl EngineClientApi {
	pub fn new(engine_client: *mut IEngineClient) -> Result<Self, libloading::Error> {
		Ok(Self { client: engine_client })
	}

	pub fn get_net_channel_info(&self) -> Option<*mut INetChannelInfo> {
		let iengineclient = unsafe { self.client.as_mut().unwrap() };
		let vtable = unsafe { iengineclient.vtable.as_ref().unwrap() };

		let net_channel_info = (vtable.get_net_channel_info)(iengineclient as *const _ as _);
		if net_channel_info.is_null() {
			return None;
		}

		Some(net_channel_info)
	}

	pub fn get_screen_size(&self) -> (usize, usize) {
		let mut width: i32 = 0;
		let mut height: i32 = 0;

		let iengineclient = unsafe { self.client.as_mut().expect("Engine client pointer is null") };
		let vtable = unsafe { iengineclient.vtable.as_ref().expect("VTable pointer is null") };

		(vtable.get_screen_size)(iengineclient as *const _ as _, &mut width, &mut height);

		(width as usize, height as usize)
	}
}

static ENGINE_CLIENT_API: std::sync::OnceLock<EngineClientApi> = std::sync::OnceLock::new();

pub fn get_api() -> Result<&'static EngineClientApi, crate::util::GetInterfaceError> {
	if let Some(api) = ENGINE_CLIENT_API.get() {
		return Ok(api);
	}

	let engine_client = crate::util::get_interface("engine_client.so", c"VEngineClient015")? as *mut IEngineClient;
	let engine_client = EngineClientApi::new(engine_client)?;

	ENGINE_CLIENT_API
		.set(engine_client)
		.expect("Should never already be initialized");

	Ok(ENGINE_CLIENT_API.get().expect("Should be initialized"))
}
