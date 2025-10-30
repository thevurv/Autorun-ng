use core::ffi::{c_char, c_int, c_void};

macro_rules! define_autorun_api {
    (
        $(
            #[name = $name:literal]
            $vis:vis fn $fn_name:ident($($param_name:ident: $param_type:ty),* $(,)?) $(-> $return_type:ty)?;
        )*
    ) => {
        #[derive(Debug)]
        pub struct AutorunApi {
            $(
                pub $fn_name: extern "C-unwind" fn($($param_name: $param_type),*) $(-> $return_type)?,
            )*
            plugin_handle: *mut c_void,
        }

        impl AutorunApi {
            pub fn new(lua_shared: &libloading::Library, plugin_handle: *mut c_void) -> Result<AutorunApi, libloading::Error> {
                let api = AutorunApi {
                    $(
                        $fn_name: *unsafe { lua_shared.get(concat!($name, "\0").as_bytes())? },
                    )*
                    plugin_handle,
                };

                Ok(api)
            }

            $(
                $vis fn $fn_name(&self, $($param_name: $param_type),*) $(-> $return_type)? {
                    (self.$fn_name)($($param_name),*)
                }
            )*
        }
    };
}

define_autorun_api! {
	#[name = "autorun_version"]
	fn _version() -> *const c_char;

	#[name = "autorun_write"]
	fn _write(
		plugin_handle: *mut c_void,
		path: *const c_char,
		content: *const c_char,
		content_len: usize,
	) -> c_int;

	#[name = "autorun_read"]
	fn _read(
		plugin_handle: *mut c_void,
		path: *const c_char,
		buffer: *mut u8,
		buffer_size: usize,
	) -> c_int;

	#[name = "autorun_read_size"]
	fn _read_size(
		plugin_handle: *mut c_void,
		path: *const c_char,
	) -> c_int;

	#[name = "autorun_mkdir"]
	fn _mkdir(
		plugin_handle: *mut c_void,
		path: *const c_char,
	) -> c_int;
}

pub enum AutorunError {
	NullHandle,
	NulError,
	WriteFailed,
	ReadFailed,
	MkdirFailed,
}

impl From<std::ffi::NulError> for AutorunError {
	fn from(_: std::ffi::NulError) -> Self {
		AutorunError::NulError
	}
}

pub type AutorunResult<T> = Result<T, AutorunError>;

impl AutorunApi {
	pub fn version(&self) -> &core::ffi::CStr {
		unsafe { core::ffi::CStr::from_ptr(self._version()) }
	}

	pub fn write(&self, path: impl AsRef<std::path::Path>, content: &[u8]) -> AutorunResult<()> {
		let c_path = std::ffi::CString::new(path.as_ref().to_string_lossy().as_bytes())?;

		let result = (self._write)(self.plugin_handle, c_path.as_ptr(), content.as_ptr() as _, content.len());

		match result {
			0 => Ok(()),
			1 => Err(AutorunError::NullHandle),
			_ => Err(AutorunError::WriteFailed),
		}
	}

	pub fn read(&self, path: impl AsRef<std::path::Path>) -> AutorunResult<Vec<u8>> {
		let c_path = std::ffi::CString::new(path.as_ref().to_string_lossy().as_bytes()).unwrap();

		let size = (self._read_size)(self.plugin_handle, c_path.as_ptr());
		if size < 0 {
			return Err(AutorunError::ReadFailed);
		}

		let mut buffer = Vec::with_capacity(size as usize);

		let read_size = (self._read)(self.plugin_handle, c_path.as_ptr(), buffer.as_mut_ptr(), size as usize);

		if read_size < 0 {
			return Err(AutorunError::ReadFailed);
		}

		Ok(buffer)
	}
}

#[macro_export]
macro_rules! autorun_entrypoint {
	($init_fn:expr, $name:ident) => {
		#[unsafe(no_mangle)]
		pub extern "C-unwind" fn $name(plugin_handle: *mut c_void) -> c_int {
			if plugin_handle.is_null() {
				return 1;
			}

			#[cfg(target_os = "linux")]
			let lib = libloading::os::unix::Library::this();

			#[cfg(target_os = "windows")]
			let Ok(lib) = libloading::os::windows::Library::this() else {
				return 2;
			};

			let api = match AutorunApi::new(&lib.into(), plugin_handle) {
				Ok(api) => api,
				Err(_) => return 2,
			};

			match $init_fn(&api) {
				Ok(_) => 0,
				Err(_) => 3,
			}
		}
	};
}

#[macro_export]
macro_rules! autorun_client_entrypoint {
	($init_fn:expr) => {
		autorun_entrypoint!($init_fn, autorun_client_init);
	};
}

#[macro_export]
macro_rules! autorun_menu_entrypoint {
	($init_fn:expr) => {
		autorun_entrypoint!($init_fn, autorun_menu_init);
	};
}

pub mod prelude {
	pub use crate::{AutorunApi, AutorunError, AutorunResult, autorun_client_entrypoint, autorun_menu_entrypoint};
}
