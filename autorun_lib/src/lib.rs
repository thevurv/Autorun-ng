mod exports;

use std::{
	os::raw::{c_char, c_double, c_int, c_void},
	sync::Mutex,
};

// use detour::GenericDetour;

// type NewState = extern "C" fn() -> *mut c_void;
// type LoadBufferX = extern "C" fn(*const c_char, usize, *const c_char, *const c_char) -> i32;

// static NewState: Mutex<Option<GenericDetour<NewState>>> = Mutex::new(None);
// static LoadBufferX: Mutex<Option<GenericDetour<LoadBufferX>>> = Mutex::new(None);

// #[cfg_attr(target_os = "linux", link_section = ".init_array")]
// #[used]
// pub static INITIALIZE: extern "C" fn() = myplugin_initialize;

// constructor function
// #[no_mangle]
// pub extern "C" fn myplugin_initialize() {
// 	// println!("myplugin initialized");

// 	unsafe {
// 		let tier0 = libloading::Library::new("libtier0.so").unwrap();
// 		let msg = tier0
// 			.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
// 			.unwrap();

// 		msg(b"!! Loaded Autorun !!\0".as_ptr() as _);
// 	}
// }

// #[no_mangle]
// pub unsafe extern "system" fn DllMain(_: *const u8, reason: u32, _: *const u8) -> u32 {
// 	match reason {
// 		// attach
// 		1 => {
// 			let tier0 = libloading::Library::new("tier0.dll").unwrap();
// 			let msg = tier0
// 				.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
// 				.unwrap();
// 			msg(b"!! Loaded Autorun !!\0".as_ptr() as _);

// 			std::process::exit(2);

// 			/*
// 			let lua_shared = libloading::Library::new("lua_shared.dll").unwrap();

// 			let luaL_newstate = lua_shared.get::<NewState>(b"luaL_newstate\0").unwrap();
// 			let luaL_newstate = std::mem::transmute::<_, NewState>(luaL_newstate);

// 			let luaL_loadbufferx = lua_shared.get::<NewState>(b"luaL_loadbufferx\0").unwrap();
// 			let luaL_loadbufferx = std::mem::transmute::<_, LoadBufferX>(luaL_loadbufferx);

// 			extern "C" fn h_luaL_newstate() -> *mut c_void {
// 				let detour = NewState.lock().unwrap();
// 				let detour = detour.as_ref().unwrap();

// 				let state = detour.call();

// 				unsafe {
// 					let lua_shared = libloading::Library::new("lua_shared.dll").unwrap();
// 					let lua_pushnumber = lua_shared
// 						.get::<extern "C" fn(*mut c_void, c_double) -> ()>(b"lua_pushnumber\0")
// 						.unwrap();
// 					let lua_setfield = lua_shared
// 						.get::<extern "C" fn(*mut c_void, c_int, *const c_char) -> ()>(
// 							b"lua_setfield\0",
// 						)
// 						.unwrap();

// 					lua_pushnumber(state, 69.0);
// 					lua_setfield(state, -10002, b"test\0".as_ptr() as _);
// 				}

// 				state
// 			}

// 			extern "C" fn h_luaL_loadbufferx(
// 				buf: *const c_char,
// 				size: usize,
// 				name: *const c_char,
// 				mode: *const c_char,
// 			) -> i32 {
// 				let detour = LoadBufferX.lock().unwrap();
// 				let detour = detour.as_ref().unwrap();

// 				detour.call(b"print('hello world!')".as_ptr() as _, 21, name, mode)
// 			}

// 			*NewState.lock().unwrap() =
// 				Some(detour::GenericDetour::new(luaL_newstate, h_luaL_newstate).unwrap());
// 			*LoadBufferX.lock().unwrap() =
// 				Some(detour::GenericDetour::new(luaL_loadbufferx, h_luaL_loadbufferx).unwrap());*/
// 		}

// 		// detach
// 		0 => {}

// 		_ => (),
// 	}

// 	1
// }

#[no_mangle]
extern "C-unwind" fn gmod13_open(_state: *const c_void) -> c_int {
	unsafe {
		let tier0 = libloading::Library::new("libtier0_client.so").unwrap();

		let msg = tier0
			.get::<extern "C" fn(fmt: *const c_char, ...)>(b"Msg\0")
			.unwrap();

		msg(c"Autorun has been loaded\n".as_ptr());

		std::thread::spawn(move || loop {
			msg(c"Hello from thread\n".as_ptr());
			std::thread::sleep(std::time::Duration::from_secs(1));
		});

		// if let Err(why) = std::process::Command::new("/bin/foot").spawn() {
		// 	let err = format!("Failed to start foot: {}\n\0", why);
		// 	msg(err.as_ptr() as _);
		// }
	}

	0
}

#[no_mangle]
extern "C-unwind" fn gmod13_close(_state: *const c_void) -> c_int {
	0
}
