pub fn steam_install_dir() -> Option<std::path::PathBuf> {
	let home = std::env::home_dir()?;

	let native_dir = home.join(".steam").join("steam");
	if native_dir.exists() {
		return Some(native_dir);
	}

	None
}
