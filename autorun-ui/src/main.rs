#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod backend;
mod frontend;
mod util;

use backend::Autorun;

fn main() -> anyhow::Result<()> {
	#[cfg(target_os = "linux")]
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../libautorun_lib.so"));

	#[cfg(target_os = "windows")]
	let lib = include_bytes!(concat!(env!("OUT_DIR"), "/../../../autorun_lib.dll"));
	std::fs::write(util::get_payload_path()?, lib)?;

	let autorun = Autorun::new()?;
	frontend::run(autorun);

	Ok(())
}
