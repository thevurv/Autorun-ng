mod backend;
mod frontend;

use backend::Autorun;

fn main() -> anyhow::Result<()> {
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../libautorun_lib.so"));
	std::fs::write(std::env::current_dir().unwrap().join("payload.so"), lib)?;

	let autorun = Autorun::new();
	frontend::run(autorun);

	Ok(())
}
