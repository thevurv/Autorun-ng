use std::{borrow::Cow, ffi::c_void, path::Path};

use windows::{
	core::PCSTR,
	Win32::{
		Foundation::CloseHandle,
		System::{
			Diagnostics::Debug::WriteProcessMemory,
			LibraryLoader::LoadLibraryA,
			Memory::{
				VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
			},
			Threading::{CreateRemoteThread, OpenProcess, WaitForSingleObject, PROCESS_ALL_ACCESS},
		},
	},
};

use crate::InjectorError;

use super::{Injection, InjectorResponse};

pub struct Injector {
	injected: bool,
}

macro_rules! injerr {
	($msg:literal) => {
		Err(InjectorError::Generic(Cow::Borrowed($msg)))
	};
}

impl Injection for Injector {
	fn new() -> Self {
		Self { injected: false }
	}

	fn inject(&mut self, pid: u32, path: impl AsRef<Path>) -> InjectorResponse<()> {
		let path = path.as_ref();
		let len = path.as_os_str().len();

		unsafe {
			let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
			if process.is_invalid() {
				return injerr!("process.is_invalid");
			}

			let addr = VirtualAllocEx(
				process,
				std::ptr::null_mut(),
				len,
				MEM_RESERVE | MEM_COMMIT,
				PAGE_READWRITE,
			);

			if addr.is_null() {
				CloseHandle(process);
				return injerr!("Failed to allocate memory.");
			}

			let mut bytes_written = 0;
			let result = WriteProcessMemory(
				process,
				addr,
				path as *const Path as _,
				len + 1,
				&mut bytes_written,
			)
			.as_bool();

			if !result {
				VirtualFreeEx(process, addr, 0, MEM_RELEASE);
				CloseHandle(process);

				injerr!("Failed to write memory in process")
			} else {
				println!("Wrote {bytes_written} {}", path.display());
				let load_lib = LoadLibraryA::<PCSTR> as *const ()
					as *const unsafe extern "system" fn(*mut c_void) -> u32;

				let thread = CreateRemoteThread(
					process,
					std::ptr::null_mut(),
					0,
					Some(std::mem::transmute(load_lib)),
					addr,
					0,
					std::ptr::null_mut(),
				);

				match thread {
					Err(why) => {
						VirtualFreeEx(process, addr, 0, MEM_RELEASE);
						CloseHandle(process);

						Err(why.into())
					}
					Ok(thread) => {
						WaitForSingleObject(thread, 0xFFFFFFFF);
						CloseHandle(thread);

						VirtualFreeEx(process, addr, 0, MEM_RELEASE);
						CloseHandle(process);

						Ok(())
					}
				}
			}
		}
	}

	fn uninject(&mut self) -> InjectorResponse<()> {
		// silly
		Ok(())
	}
}

impl Drop for Injector {
	fn drop(&mut self) {
		if self.injected {
			self.uninject();
		}
	}
}
