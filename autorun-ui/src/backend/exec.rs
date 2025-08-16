use super::Autorun;

impl Autorun {
	pub fn run_code(&mut self, code: impl Into<String>) {
		let code = code.into();

		/*let func = hooks::LOADBUFFERX_ORIGINAL.lock();
		let func = func.unwrap();
		let func = func.as_ref().unwrap();*/

		// let ident = b"bruh\0".as_ptr() as _;
		// let code = b"print('hello world')".as_ptr() as _;
		// let state = todo!();

		// unsafe { func.call(state, code, 20, ident) }
	}
}
