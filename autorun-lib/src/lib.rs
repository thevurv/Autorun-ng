use autorun_interfaces::net::{INetChannelInfo, INetChannelInfoVTable};

mod events;
mod hooks;
mod lua_queue;
mod server;

pub fn main() -> anyhow::Result<()> {
	// Start IPC server in background thread
	std::thread::spawn(|| {
		if let Err(e) = server::start() {
			eprintln!("Failed to start IPC server: {}", e);
		}
	});

	hooks::paint_traverse::init()?;
	hooks::load_buffer::init()?;

	let engine = autorun_interfaces::engine_client::get_api().unwrap();

	// let net_chan = engine.get_net_channel_info().unwrap() as *mut INetChannelInfo;
	// let msg = unsafe { *net_chan.add(0x2460 / 8) };
	// let msg = unsafe { std::ffi::CStr::from_ptr(msg) };
	// println!("Message: {}", msg.to_string_lossy());
	// let net_chan = unsafe { net_chan.as_ref().unwrap() };

	// let vtable = net_chan.vtable;
	// let vtable = unsafe { vtable.as_ref().unwrap() };

	// let mut data = [0u8; 0x3000];
	// println!("{:p}", (vtable.get_name)(data.as_ptr() as _));

	// let name = (vtable.get_address)(std::ptr::null());
	// let name = (vtable.get_name)(net_chan as *const _ as _);
	// autorun_log::warn!("good!");
	// let name = unsafe { std::ffi::CStr::from_ptr(name) };
	// let name = name.to_string_lossy();

	Ok(())
}

#[ctor::ctor]
fn on_library_load() {
	match std::env::current_exe() {
		Err(why) => {
			eprintln!("Failed to get current exe path: {:?}", why);
		}
		Ok(exe) => {
			// Ensure LD_PRELOAD doesn't affect other programs
			// Without this, iirc steam messes up
			if exe.file_name() == Some(std::ffi::OsStr::new("gmod")) {
				unsafe { std::env::remove_var("LD_PRELOAD") };

				if let Err(why) = main() {
					autorun_log::error!("{why}");
				}
			}
		}
	}
}
