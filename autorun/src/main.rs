mod backend;
mod frontend;

use backend::Autorun;

#[derive(Debug, thiserror::Error)]
enum TotalError {
	#[error("Error when loading app: {0}")]
	App(#[from] frontend::error::AppError),

	#[error("Error when loading Autorun state: {0}")]
	Autorun(#[from] backend::error::AutorunError),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(target_os = "linux")]
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../libautorun_lib.so"));
	std::fs::write( std::env::current_dir().unwrap().join("payload.so"), lib )?;

	// println!("Included lib size {} kb", lib.len() / 1024);

	let mut autorun = Autorun::new();
	autorun.launch_attached()?;

	// frontend::run(autorun);

	/*let gmod_exe = locator::gmod_dir().unwrap().join("hl2.exe");
	let cmd = std::process::Command::new(gmod_exe).spawn().unwrap();

	std::thread::sleep_ms(50);

	let mut injector = injector::Injector::new();
	println!("{:#?}", injector.inject(cmd.id(), "D:\\Files\\Github\\Autorun-next\\target\\i686-pc-windows-msvc\\debug\\autorun_lib.dll"));*/

	Ok(())
}
