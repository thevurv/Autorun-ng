use std::{os::raw::{c_void, c_char, c_int}, sync::Mutex};
use super::{Autorun, Status, error};

type LoadBufferx = extern "C" fn(state: *mut c_void, code: *const c_char, size: usize, name: *const c_char, mode: *const c_char) -> c_int;
pub static LOADBUFFERX_ORIGINAL: Mutex<Option<detour::GenericDetour<LoadBufferx>>> = Mutex::new(None);

#[inline(never)]
extern "C" fn loadbufferx(state: *mut c_void, code: *const c_char, len: usize, name: *const c_char, mode: *const c_char) -> c_int {
	// Awful
	let func = LOADBUFFERX_ORIGINAL.lock();
	let func = func.unwrap();
	let func = func.as_ref().unwrap();

	let new_code = b"print('Hello')\0".as_ptr().cast();

	unsafe { func.call(state, new_code, 15, name, mode) }
}

impl Autorun {
	pub fn load_hooks(&mut self) -> Result<(), error::HookError> {
		use sysinfo::{ProcessExt, SystemExt};

		let mut gmod: Option<&sysinfo::Process> = None;
		for process in self.system.processes_by_name("gmod") {
			gmod = process.parent()
				.map(|x| self.system.process(x))
				.flatten();
			
			if gmod.is_some() {
				break
			}
		}

		if let Some(process) = gmod {
			let bin_folder = process.root();
			
			match bin_folder.file_name() {
				Some(x) if x != "win64" && x != "bin" && x != "linux32" && x != "linux64" => {
					Err(error::HookError::WeirdPath)
				},
				_ => {
					// Necessary so LoadLibrary can find dependency dlls (even from an absolute path)
					std::env::set_current_dir(bin_folder);

					use sysinfo::PidExt;
					let x = process.pid().as_u32();

					

					/*let lua_shared = libloading::library_filename("lua_shared");
					let lua_shared = unsafe { libloading::Library::new(lua_shared) }?;

					let loadbufx = unsafe { lua_shared.get::<LoadBufferx>(b"luaL_loadbufferx") }?;
					let loadbufx = unsafe { std::mem::transmute::<_, LoadBufferx>(loadbufx) };

					let tour = unsafe { detour::GenericDetour::new(loadbufx, loadbufferx) }
						.map_err(|_| error::HookError::DetourLoadbufx)?;
					
					unsafe { tour.enable() }
						.map_err(|_| error::HookError::DetourLoadbufx)?;

					println!("{tour:?}");

					*LOADBUFFERX_ORIGINAL.lock().unwrap() = Some(tour);

					let vgui2 = libloading::library_filename("vgui2");
					let vgui2 = unsafe { libloading::Library::new(vgui2) }?;

					type PaintTraverse = extern "fastcall" fn(&'static c_void, usize, bool, bool);*/

					//println!("Process {process:?}");
					self.set_status(Status::Injected);

					Ok(())
				}
			}
		} else {
			Err(error::HookError::GameNotFound)
		}
	}

	/// This is called when the ui closes.
	/// Normally I'd use Drop to call this but for some reason it's being called despite egui still running the app.
	/// wtf
	pub fn unload_hooks(&mut self) -> Result<(), error::HookError> {
		println!("Unloading hooks");

		let detour = LOADBUFFERX_ORIGINAL.lock().unwrap();
		let detour = detour.as_ref();

		if let Some(detour) = detour {
			unsafe { detour.disable() };
		}

		Ok(())
	}
}