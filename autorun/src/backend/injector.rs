use winapi::{
	shared::{minwindef::{FALSE, MAX_PATH}, ntdef::NULL},
	um::{
		handleapi::CloseHandle,
		libloaderapi::{LoadLibraryA, GetProcAddress},
		memoryapi::{VirtualAllocEx, VirtualFreeEx, WriteProcessMemory},
		processthreadsapi::{CreateRemoteThread, OpenProcess},
		synchapi::WaitForSingleObject,
		winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, PROCESS_ALL_ACCESS},
	},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Failed to open a handle in the process")]
	OpenProcess,

	#[error("Failed to allocate memory in the process (Do you have permissions?)")]
	VirtualAllocEx,

	#[error("Failed to write memory in the process")]
	WriteProcessMemory,

	#[error("Failed to create remote thread in process")]
	CreateRemoteThread
}

pub fn inject(pid: u32, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
	let path = path.as_ref();

	unsafe {
		let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
		if process.is_null() {
			return Err(Error::OpenProcess);
		}

		let addr = VirtualAllocEx(
			process,
			NULL,
			MAX_PATH,
			MEM_RESERVE | MEM_COMMIT,
			PAGE_READWRITE,
		);

		if addr.is_null() {
			CloseHandle(process);
			return Err(Error::VirtualAllocEx);
		}

		let mut bytes_written = 0;
		let result = WriteProcessMemory(
			process,
			addr,
			path as *const _ as _,
			MAX_PATH,
			&mut bytes_written,
		);

		if result == FALSE {
			VirtualFreeEx(process, addr, 0, MEM_RELEASE);
			CloseHandle(process);

			return Err(Error::WriteProcessMemory);
		} else {
			let kernel_addr = GetProcAddress(LoadLibraryA(b"Kernel32\0".as_ptr() as _), b"LoadLibraryA\0".as_ptr() as _);
			let thread = CreateRemoteThread(
				process,
				std::ptr::null_mut(),
				0,
				Some(std::mem::transmute(kernel_addr)),
				addr,
				0,
				std::ptr::null_mut(),
			);

			if thread.is_null() {
				VirtualFreeEx(process, addr, 0, MEM_RELEASE);
				CloseHandle(process);

				Err(Error::CreateRemoteThread)
			} else {
				WaitForSingleObject(thread, 0xFFFFFFFF);
				CloseHandle(thread);

				CloseHandle(process);
				VirtualFreeEx(process, addr, 0, MEM_RELEASE);

				Ok(())
			}
		}
	}
}