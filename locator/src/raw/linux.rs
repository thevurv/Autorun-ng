pub fn steam_install_dir() -> Option<std::path::PathBuf> {
    if let Ok(steam_dir) = std::env::var("STEAM_DIR") {
        return Some(std::path::PathBuf::from(steam_dir));
    }

    #[allow(deprecated)]
    let home = std::env::home_dir()?;

    let native_dir = home.join(".steam").join("steam");
    if native_dir.exists() {
        return Some(native_dir);
    }

    None
}