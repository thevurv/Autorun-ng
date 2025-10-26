use core::ffi::{c_char, c_int};

#[repr(C)]
pub enum PluginResult {
	Ok = 0,
	ErrNullHandle = -1,
	ErrWriteFailed = -2,
}

#[unsafe(no_mangle)]
pub extern "C" fn autorun_version() -> *const c_char {
	concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[unsafe(no_mangle)]
pub extern "C" fn autorun_write(
	plugin_handle: *mut autorun_core::plugins::Plugin,
	path: *const c_char,
	content: *const c_char,
) -> PluginResult {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return PluginResult::ErrNullHandle;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };
	let content = unsafe { std::ffi::CStr::from_ptr(content) };

	plugin
		.dir()
		.write(path.to_str().unwrap_or_default(), content.to_bytes())
		.map(|_| PluginResult::Ok)
		.unwrap_or(PluginResult::ErrWriteFailed)
}

#[unsafe(no_mangle)]
pub extern "C" fn autorun_read(
	plugin_handle: *mut autorun_core::plugins::Plugin,
	path: *const c_char,
	buffer: *mut u8,
	buffer_size: usize,
) -> c_int {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return -1;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };
	let data = match plugin.dir().read(path.to_str().unwrap_or_default()) {
		Ok(data) => data,
		Err(_) => return -2,
	};

	let read_size = std::cmp::min(buffer_size, data.len());
	unsafe {
		std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, read_size);
	};

	read_size as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn autorun_read_size(plugin_handle: *mut autorun_core::plugins::Plugin, path: *const c_char) -> c_int {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return -1;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };
	match plugin.dir().read(path.to_str().unwrap_or_default()) {
		Ok(data) => data.len() as c_int,
		Err(_) => -2,
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn autorun_mkdir(plugin_handle: *mut autorun_core::plugins::Plugin, path: *const c_char) -> PluginResult {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return PluginResult::ErrNullHandle;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };

	plugin
		.dir()
		.create_dir_all(path.to_str().unwrap_or_default())
		.map(|_| PluginResult::Ok)
		.unwrap_or(PluginResult::ErrWriteFailed)
}
