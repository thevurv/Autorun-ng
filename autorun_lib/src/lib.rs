mod exports;

use std::os::raw::{c_char, c_int, c_void};

/// Constructor function that runs automatically when the library is loaded via LD_PRELOAD
#[ctor::ctor]
fn on_library_load() {
	unsafe {
		// Try to load tier0 and print a message when the library is injected
		if let Ok(tier0) = libloading::Library::new("libtier0_client.so") {
			if let Ok(msg) = tier0.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0") {
				msg(c"Autorun library injected via LD_PRELOAD\n".as_ptr());
			}
		}

		std::thread::spawn(|| {
			std::thread::sleep(std::time::Duration::from_secs(5));

			if let Ok(tier0) = libloading::Library::new("libtier0_client.so") {
				if let Ok(msg) = tier0.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0") {
					msg(c"Waited and then now im here\n".as_ptr());
				}
			}
		});
	}
}

#[no_mangle]
extern "C-unwind" fn gmod13_open(_state: *const c_void) -> c_int {
	unsafe {
		let tier0 = libloading::Library::new("libtier0_client.so").unwrap();

		let msg = tier0
			.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
			.unwrap();

		msg(c"Autorun has been loaded\n".as_ptr());
	}

	0
}

#[no_mangle]
extern "C-unwind" fn gmod13_close(_state: *const c_void) -> c_int {
	0
}
