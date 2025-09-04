use std::ffi::{c_char, c_int, c_void};

type CreateInterfaceFn = extern "C" fn(name: *const c_char, return_code: *mut c_int) -> *mut c_void;

pub fn get_interface(
	path: impl AsRef<std::ffi::OsStr>,
	interface: &str,
) -> anyhow::Result<*mut std::ffi::c_void> {
	let library =
		unsafe { libloading::Library::new(path.as_ref()) }.expect("Failed to load library");

	let factory = unsafe {
		library
			.get::<CreateInterfaceFn>(c"CreateInterface".to_bytes_with_nul())
			.expect("Failed to get CreateInterface function")
	};

	let interface_cstr = std::ffi::CString::new(interface)?;

	let mut return_code: c_int = 0;
	let interface_ptr = factory(interface_cstr.as_ptr(), &mut return_code as *mut c_int);

	if return_code != 0 {
		return Err(anyhow::anyhow!(
			"CreateInterface returned error code {} for interface {} from {:?}",
			return_code,
			interface,
			path.as_ref()
		));
	}

	std::mem::forget(library);

	if interface_ptr.is_null() {
		Err(anyhow::anyhow!(
			"Failed to get interface: {} from {:?}",
			interface,
			path.as_ref()
		))
	} else {
		Ok(interface_ptr)
	}
}
