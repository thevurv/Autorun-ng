pub use cap_std::{AmbientAuthority, ambient_authority};

// Methods sourced from Rust's standard library itself
// Windows: https://github.com/rust-lang/rust/blob/c24e166527cd46d3b4033b0c061d02657e3c3cbf/library/std/src/sys/pal/windows/fs.rs#L1300
// Unix: https://github.com/rust-lang/rust/blob/c24e166527cd46d3b4033b0c061d02657e3c3cbf/library/std/src/sys/pal/unix/fs.rs#L1508

pub fn get_path(_ambient_authority: AmbientAuthority, dir: &cap_std::fs::Dir) -> std::io::Result<std::path::PathBuf> {
	#[cfg(unix)]
	{
		use cap_std::io_lifetimes::raw::AsRawFilelike;

		let fd = dir.as_raw_filelike();
		let path = std::fs::read_link(format!("/proc/self/fd/{fd}"))?;

		Ok(path)
	}

	#[cfg(windows)]
	{
		use std::os::windows::io::AsRawHandle;
		let handle = dir.as_raw_handle();

		let mut buffer = vec![0u16; 260]; // MAX_PATH
		let len = unsafe {
			windows::Win32::Storage::FileSystem::GetFinalPathNameByHandleW(
				windows::Win32::Foundation::HANDLE(handle),
				&mut buffer,
				windows::Win32::Storage::FileSystem::GETFINALPATHNAMEBYHANDLE_FLAGS(0),
			)
		};
		if len == 0 {
			return Err(std::io::Error::last_os_error());
		}
		buffer.resize(len as usize, 0);

		let path = String::from_utf16_lossy(&buffer);
		let path = std::path::PathBuf::from(path.trim_start_matches(r"\\?\"));
		Ok(path)
	}
}
