fn get_payload_dir() -> anyhow::Result<std::path::PathBuf> {
	let exe_path = std::env::current_exe()?;
	let exe_dir = exe_path
		.parent()
		.ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;

	Ok(exe_dir.to_path_buf())
}

pub fn get_payload_path() -> anyhow::Result<std::path::PathBuf> {
	let payload_dir = get_payload_dir()?;

	#[cfg(target_os = "linux")]
	let payload = payload_dir.join("payload.so");

	#[cfg(target_os = "windows")]
	let payload = payload_dir.join("payload.dll");

	Ok(payload)
}
