mod backend;
mod frontend;
mod util;

use backend::Autorun;

fn main() -> anyhow::Result<()> {
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../libautorun_lib.so"));
	std::fs::write(util::get_payload_path()?, lib)?;

	let autorun = Autorun::new()?;
	frontend::run(autorun);

	Ok(())
}
