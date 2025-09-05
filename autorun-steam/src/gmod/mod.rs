mod raw;

/// Launches gmod with the given library injected.
pub fn launch(lib_path: impl AsRef<std::path::Path>) -> anyhow::Result<std::process::Child> {
	#[cfg(target_os = "linux")]
	{
		return raw::linux::launch(lib_path);
	}

	#[cfg(not(target_os = "linux"))]
	{
		// Should be pretty easy to do with dll_syringe on windows.
		// MacOS sucks.
		todo!("Not implemented on !linux")
	}
}
