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
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../autorun_lib.dll"));
	std::fs::write("payload.dll", lib)?;

	println!("Included lib size {} kb", lib.len() / 1024);

	let mut autorun = Autorun::new();
	autorun.launch_attached()?;

	frontend::run(autorun);

	println!("Ran app?");

	Ok(())
}
