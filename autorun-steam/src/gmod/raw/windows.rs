pub fn launch(lib_path: impl AsRef<std::path::Path>) -> anyhow::Result<std::process::Child> {
	let gmod_dir = crate::locate::gmod_dir().ok_or_else(|| anyhow::anyhow!("Failed to locate gmod dir"))?;

	let gmod_exe = gmod_dir.join("bin").join("win64").join("gmod.exe");

	let child = std::process::Command::new(&gmod_exe).spawn()?;

	let pid = child.id();

	let owned_process = dll_syringe::process::OwnedProcess::from_pid(pid)?;
	let syringe = dll_syringe::Syringe::for_process(owned_process);
	let injected_payload = syringe.inject(lib_path.as_ref())?;

	let remote_entrypoint =
		unsafe { syringe.get_payload_procedure::<fn() -> ()>(injected_payload, "autorun_entrypoint") }?.unwrap();

	remote_entrypoint.call()?;

	Ok(child)
}
