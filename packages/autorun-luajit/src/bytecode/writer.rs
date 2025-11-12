use crate::{BCIns, GCfuncL};
use anyhow::Context;

pub struct BCWriter {
	ptr: *mut BCIns,
	size: usize,
	offset: usize,
}

impl BCWriter {
	pub fn from_bc_ptr(ptr: *mut BCIns, size: usize) -> Self {
		Self { ptr, size, offset: 0 }
	}

	pub fn from_gcfunc_l(gcfunc: &GCfuncL) -> anyhow::Result<Self> {
		let proto = gcfunc.get_proto().context("Failed to get proto from GCfuncL")?;
		let proto = unsafe { proto.as_mut().context("Failed to dereference proto")? };
		let bc_ptr = gcfunc.get_bc_ins()?;

		Ok(Self::from_bc_ptr(bc_ptr, proto.sizebc as usize))
	}

	pub fn reset(&mut self) {
		self.offset = 0;
	}

	pub fn set_offset(&mut self, offset: usize) -> anyhow::Result<()> {
		if offset >= self.size {
			anyhow::bail!(
				"Bytecode writer set_offset out of bounds: offset {} exceeds size {}.",
				offset,
				self.size
			);
		}

		self.offset = offset;
		Ok(())
	}

	pub fn write(&mut self, instruction: BCIns) -> anyhow::Result<()> {
		if self.offset >= self.size {
			anyhow::bail!("Bytecode writer overflow: attempted to write beyond allocated size.");
		}

		unsafe {
			std::ptr::write(self.ptr.add(self.offset), instruction);
		}

		self.offset += 1;
		Ok(())
	}

	pub fn replace(&mut self, instruction: BCIns) -> anyhow::Result<BCIns> {
		if self.offset >= self.size {
			anyhow::bail!("Bytecode writer replace out of bounds: no instruction to replace at current offset.");
		}

		let target_ptr = unsafe { self.ptr.add(self.offset) };
		let old_instruction = unsafe { std::ptr::read(target_ptr) };

		unsafe {
			std::ptr::write(target_ptr, instruction);
		}

		self.offset += 1;
		Ok(old_instruction)
	}

	pub fn get_ptr(&self) -> *mut BCIns {
		self.ptr
	}
}
