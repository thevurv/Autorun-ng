pub mod lj_debug_funcname;

pub fn install_auth_hooks() -> anyhow::Result<()> {
	lj_debug_funcname::init()?;

	Ok(())
}
