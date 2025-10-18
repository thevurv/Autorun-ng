mod raw;

pub fn steam_install_dir() -> Option<std::path::PathBuf> {
	if let Ok(steam_dir) = std::env::var("STEAM_DIR") {
		return Some(std::path::PathBuf::from(steam_dir));
	}

	#[cfg(target_os = "linux")]
	{
		raw::linux::steam_install_dir()
	}

	#[cfg(target_os = "windows")]
	{
		raw::windows::steam_install_dir()
	}
}

pub fn gmod_dir() -> Option<std::path::PathBuf> {
	if let Ok(env) = std::env::var("GMOD_DIR") {
		return Some(std::path::PathBuf::from(env));
	}

	let library_folders = steam_install_dir()?.join("steamapps").join("libraryfolders.vdf");

	let content = std::fs::read_to_string(&library_folders).ok()?;

	let tokens = crate::vdf::tokenize(content.as_bytes()).unwrap();
	let ast = crate::vdf::parse(&tokens).unwrap();

	if let (_, crate::vdf::Value::Object(folders)) = ast {
		let mut iter = folders.into_iter();
		while let Some((_index, crate::vdf::Value::Object(folder))) = iter.next() {
			if let Some((_, crate::vdf::Value::String(path))) = folder.iter().find(|x| x.0 == b"path") {
				if let Some((_, crate::vdf::Value::Object(apps))) = folder.iter().find(|x| x.0 == b"apps") {
					if apps.iter().any(|x| x.0 == b"4000") {
						// This is the folder that contains gmod
						let gmod_dir = std::path::PathBuf::from(String::from_utf8_lossy(path).to_string())
							.join("steamapps")
							.join("common")
							.join("GarrysMod");

						return Some(gmod_dir);
					}
				} else {
					todo!()
				}
			} else {
				todo!()
			}
		}

		todo!();
	}

	None
}
