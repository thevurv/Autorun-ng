mod backend;
mod frontend;

use backend::Autorun;

fn main() -> anyhow::Result<()> {
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../libautorun_lib.so"));
	std::fs::write(std::env::current_dir().unwrap().join("payload.so"), lib)?;

	// println!("Included lib size {} kb", lib.len() / 1024);

	let mut autorun = Autorun::new();
	autorun.launch()?;

	frontend::run(autorun);

	Ok(())
}
