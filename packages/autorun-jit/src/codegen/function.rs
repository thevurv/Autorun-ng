use crate::MCode;
use region::Allocation;

pub struct Function {
	allocation: Allocation,
	pub mcode: MCode,
	pub size: usize,
}

impl Function {
	pub fn allocate(size: usize) -> Self {
		let mut allocation =
			region::alloc(size, region::Protection::READ_WRITE).expect("Failed to allocate memory for Function");

		let mcode = MCode::new(allocation.as_mut_ptr(), size);

		Self { allocation, mcode, size }
	}

	pub fn make_executable(&mut self) -> anyhow::Result<()> {
		unsafe {
			region::protect(self.allocation.as_ptr::<u8>(), self.size, region::Protection::READ_EXECUTE)
				.map_err(|_| anyhow::anyhow!("Failed to set function memory as executable"))
		}
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.allocation.as_ptr::<u8>()
	}
}
