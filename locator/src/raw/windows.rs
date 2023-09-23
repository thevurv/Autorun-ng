use std::path::PathBuf;

use winreg::{
	enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
	RegKey,
};

const STEAM_PATHS: &[&str] = &[
	"SOFTWARE\\WOW6432Node\\Valve\\Steam",
	"SOFTWARE\\Valve\\Steam",
];

const HKEYS: &[RegKey] = &[
	RegKey::predef(HKEY_LOCAL_MACHINE),
	RegKey::predef(HKEY_CURRENT_USER),
];

pub fn steam_install_dir() -> Option<PathBuf> {
	STEAM_PATHS
		.iter()
		.find_map(|path| {
			HKEYS.iter().find_map(|regkey| {
				regkey
					.open_subkey(path)
					.map(|path| path.get_value::<String, &str>("InstallPath"))
					.map(core::result::Result::ok)
					.ok()
					.flatten()
			})
		})
		.map(PathBuf::from)
}