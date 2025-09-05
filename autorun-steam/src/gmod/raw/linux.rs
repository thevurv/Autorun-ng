use anyhow::anyhow;

pub fn launch(lib_path: impl AsRef<std::path::Path>) -> anyhow::Result<std::process::Child> {
	let steam_dir = crate::locate::steam_install_dir()
		.ok_or_else(|| anyhow!("Failed to locate steam install dir"))?;

	let gmod_dir = crate::locate::gmod_dir().ok_or_else(|| anyhow!("Failed to locate gmod dir"))?;

	let steam_launch_wrapper = steam_dir.join("ubuntu12_32").join("steam-launch-wrapper");
	if !steam_launch_wrapper.exists() {
		return Err(anyhow!(
			"steam-launch-wrapper not found at {:?}",
			steam_launch_wrapper
		));
	}

	let reaper = steam_dir.join("ubuntu12_32").join("reaper");
	if !reaper.exists() {
		return Err(anyhow!("reaper not found at {:?}", reaper));
	}

	let soldier_entrypoint = steam_dir
		.join("steamapps")
		.join("common")
		.join("SteamLinuxRuntime_soldier")
		.join("_v2-entry-point");

	if !soldier_entrypoint.exists() {
		return Err(anyhow!(
			"SteamLinuxRuntime_soldier entrypoint not found at {:?}",
			soldier_entrypoint
		));
	}

	let scout_on_soldier_entrypoint = steam_dir
		.join("steamapps")
		.join("common")
		.join("SteamLinuxRuntime")
		.join("scout-on-soldier-entry-point-v2");

	if !scout_on_soldier_entrypoint.exists() {
		return Err(anyhow!(
			"scout-on-soldier-entry-point-v2 not found at {:?}",
			scout_on_soldier_entrypoint
		));
	}

	let hl2_sh = gmod_dir.join("hl2.sh");
	if !hl2_sh.exists() {
		return Err(anyhow!("hl2.sh not found at {:?}", hl2_sh));
	}

	let ret = std::process::Command::new(steam_launch_wrapper)
		.env("GMOD_ENABLE_LD_PRELOAD", "1")
		.env("LD_PRELOAD", lib_path.as_ref())
		.arg("--")
		.arg(&reaper)
		.arg("SteamLaunch")
		.arg("AppId=4000")
		.arg("--")
		.arg(&soldier_entrypoint)
		.arg("--verb=waitforexitandrun")
		.arg("--")
		.arg(&scout_on_soldier_entrypoint)
		.arg("--")
		.arg(&hl2_sh)
		.arg("-steam")
		.arg("-game")
		.arg("garrysmod")
		.spawn()?;

	Ok(ret)
}
