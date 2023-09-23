mod raw;
mod vdf;

#[cfg(target_os = "windows")]
pub fn gmod_dir() -> Option<std::path::PathBuf> {
	let library_folders = raw::windows::steam_install_dir()?
		.join("steamapps")
		.join("libraryfolders.vdf");

	let content = std::fs::read_to_string(&library_folders).ok()?;

	let ast = crate::vdf::parse(&content).ok()?;

	if let (_, crate::vdf::Value::Object(folders)) = ast.1 {
		let mut iter = folders.into_iter();
		while let Some((_index, crate::vdf::Value::Object(folder))) = iter.next() {
			if let Some((_, crate::vdf::Value::Str(path))) = folder.iter().find(|x| x.0 == "path") {
				if let Some((_, crate::vdf::Value::Object(apps))) = folder.iter().find(|x| x.0 == "apps") {
					if apps.iter().find(|x| x.0 == "4000").is_some() {
						// This is the folder that contains gmod
						let steam_dir = std::path::PathBuf::from(path)
							.join("steamapps")
							.join("common")
							.join("GarrysMod");

						return Some(steam_dir);
					}
				} else {
					// todo!()
				}
			} else {
				// todo!()
			}
		}

		// todo!();
	}

	None
}

#[test]
fn test() {
	println!("{:#?}", gmod_dir())
}