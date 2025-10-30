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

/// # Safety
/// `path` must be a valid null-terminated C string.
/// `content` must be a valid pointer to a buffer of `content_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn autorun_write(
	plugin_handle: *mut autorun_core::plugins::Plugin,
	path: *const c_char,
	content: *const u8,
	content_len: usize,
) -> PluginResult {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return PluginResult::ErrNullHandle;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };
	let content = unsafe { std::slice::from_raw_parts(content, content_len) };

	plugin
		.data_dir()
		.write(path.to_str().unwrap_or_default(), content)
		.map(|_| PluginResult::Ok)
		.unwrap_or(PluginResult::ErrWriteFailed)
}

/// # Safety
/// `path` must be a valid null-terminated C string.
/// `buffer` must be a valid pointer to a buffer of at least `buffer_size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn autorun_read(
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

/// # Safety
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn autorun_read_size(plugin_handle: *mut autorun_core::plugins::Plugin, path: *const c_char) -> c_int {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return -1;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };
	match plugin.dir().read(path.to_str().unwrap_or_default()) {
		Ok(data) => data.len() as c_int,
		Err(_) => -2,
	}
}

/// # Safety
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn autorun_mkdir(plugin_handle: *mut autorun_core::plugins::Plugin, path: *const c_char) -> PluginResult {
	let Some(plugin) = (unsafe { plugin_handle.as_ref() }) else {
		return PluginResult::ErrNullHandle;
	};

	let path = unsafe { std::ffi::CStr::from_ptr(path) };

	plugin
		.data_dir()
		.create_dir_all(path.to_str().unwrap_or_default())
		.map(|_| PluginResult::Ok)
		.unwrap_or(PluginResult::ErrWriteFailed)
}

/// # Safety
/// `message` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn autorun_print(message: *const c_char) {
	let message = unsafe { std::ffi::CStr::from_ptr(message) };

	if let Ok(message_str) = message.to_str() {
		println!("[Autorun Plugin] {}", message_str);
	}
}
