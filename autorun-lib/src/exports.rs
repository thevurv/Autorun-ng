// use std::ffi::{CString, CStr};
// use std::os::raw::{c_char};

#[repr(u8)]
enum Realm {
	Menu,
	Client,
}

/*#[no_mangle]
extern "C" fn run_code(code: *const c_char, len: usize, realm: Realm) {
	println!("Run coce");
}*/
