use std::ffi::{c_char, c_float, c_void};

#[repr(C)]
#[derive(Debug)]
pub enum NetEnum {
	Generic = 0,
	LocalPlayer,
	OtherPlayers,
	Entities,
	Sounds,
	Events,
	UserMessages,
	EntMessages,
	Voice,
	StringTable,
	Move,
	StringCmd,
	SignOn,
	Total,
}

#[repr(C)]
pub struct INetChannelInfo {
	pub vtable: *const INetChannelInfoVTable,
}

#[repr(C)]
pub struct INetChannelInfoVTable {
	#[cfg(target_os = "linux")]
	rtti: *const c_void,
	// _padding: [*const c_void; 1],
	pub get_name: extern "C" fn(this: *const INetChannelInfo) -> *const c_char,
	pub get_address: extern "C" fn(this: *const INetChannelInfo) -> *const c_char,
	pub get_time: extern "C" fn(this: *const INetChannelInfo) -> c_float,
	pub get_time_connected: extern "C" fn(this: *const INetChannelInfo) -> c_float,
}
