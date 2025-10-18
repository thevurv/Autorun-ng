mod raw;

/// Launches gmod with the given library injected.
pub fn launch(lib_path: impl AsRef<std::path::Path>) -> anyhow::Result<std::process::Child> {
	#[cfg(target_os = "linux")]
	{
		raw::linux::launch(lib_path)
	}

	#[cfg(target_os = "windows")]
	{
		raw::windows::launch(lib_path)
	}

	#[cfg(not(any(target_os = "linux", target_os = "windows")))]
	{
		// macos sucks
		todo!("Not implemented on this platform")
	}
}
