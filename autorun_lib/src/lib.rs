mod exports;

use std::os::raw::c_char;

pub fn main() -> anyhow::Result<()> {
	println!("Hello main");

	std::thread::spawn(|| loop {
		std::thread::sleep(std::time::Duration::from_secs(1));

		unsafe {
			if let Ok(tier0) = libloading::Library::new("libtier0_client.so") {
				if let Ok(msg) = tier0.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0") {
					msg(c"Hello, autorun!".as_ptr());
				}
			}
		}
	});

	Ok(())
}

#[ctor::ctor]
fn on_library_load() {
	match std::env::current_exe() {
		Err(why) => {
			eprintln!("Failed to get current exe path: {:?}", why);
		}
		Ok(exe) => {
			if exe.file_name() == Some(std::ffi::OsStr::new("gmod")) {
				std::env::remove_var("LD_PRELOAD");
				main();
			}
		}
	}
}
