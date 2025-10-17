use std::ffi::{c_char, c_int, c_void};
type CreateInterfaceFn = extern "C-unwind" fn(name: *const c_char, return_code: *mut c_int) -> *mut c_void;

#[derive(Debug, thiserror::Error)]
pub enum GetInterfaceError {
	#[error("Libloading Error: {0}")]
	Libloading(#[from] libloading::Error),

	#[error("CreateInterface returned error code {0}")]
	Errored(c_int),

	#[error("CreateInterface returned null pointer")]
	Null,
}

pub fn get_interface(
	path: impl AsRef<std::ffi::OsStr>,
	interface: &std::ffi::CStr,
) -> Result<*mut std::ffi::c_void, GetInterfaceError> {
	let library = unsafe { libloading::Library::new(path.as_ref()) }?;
	let factory = unsafe { library.get::<CreateInterfaceFn>(c"CreateInterface".to_bytes_with_nul())? };

	let mut return_code: c_int = 0;
	let interface_ptr = factory(interface.as_ptr(), &mut return_code as *mut c_int);

	if return_code != 0 {
		return Err(GetInterfaceError::Errored(return_code));
	}

	std::mem::forget(library);

	if interface_ptr.is_null() {
		Err(GetInterfaceError::Null)
	} else {
		Ok(interface_ptr)
	}
}
