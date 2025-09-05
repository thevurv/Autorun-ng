use std::ffi::c_char;

#[derive(Debug)]
pub struct Tier0Api {
	msg: extern "C" fn(fmt: *const c_char, ...),
}

impl Tier0Api {
	fn new(tier0: &libloading::Library) -> Result<Tier0Api, libloading::Error> {
		let api = Tier0Api {
			msg: *unsafe { tier0.get(b"Msg\0")? },
		};

		Ok(api)
	}

	pub fn msg(&self, str: *const c_char) {
		(self.msg)(c"%s".as_ptr(), str);
	}
}

static TIER0_API: std::sync::OnceLock<Tier0Api> = std::sync::OnceLock::new();

pub fn get_api() -> Result<&'static Tier0Api, libloading::Error> {
	if let Some(api) = TIER0_API.get() {
		return Ok(api);
	}

	let tier0 = unsafe { libloading::Library::new("libtier0_client.so") }?;
	let api = Tier0Api::new(&tier0)?;

	std::mem::forget(tier0);

	TIER0_API.set(api).expect("Should never already be initialized");

	Ok(TIER0_API.get().expect("Should be initialized"))
}
