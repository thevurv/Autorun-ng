pub struct MCode {
	ptr: *mut u8,
	offset: usize,
	size: usize,
}

macro_rules! write_byte {
	($mcode:expr, $byte:expr) => {
		assert!($mcode.offset < $mcode.size, "Not enough space to write byte");
		unsafe {
			std::ptr::write($mcode.ptr.add($mcode.offset), $byte);
		}
		$mcode.offset += 1;
	};
}

macro_rules! write_bytes {
    ($mcode:expr, [$($byte:expr),*]) => {
        $(
            write_byte!($mcode, $byte);
        )*
    };
}

impl MCode {
	pub fn new(ptr: *mut u8, size: usize) -> Self {
		Self { ptr, offset: 0, size }
	}

	pub fn write_imm64(&mut self, value: u64) {
		assert!(self.offset + 8 <= self.size, "Not enough space to write imm64");
		unsafe {
			std::ptr::copy_nonoverlapping(&value as *const u64 as *const u8, self.ptr.add(self.offset), 8);
		}
		self.offset += 8;
	}

	pub fn write_imm32(&mut self, value: u32) {
		assert!(self.offset + 4 <= self.size, "Not enough space to write imm32");
		unsafe {
			std::ptr::copy_nonoverlapping(&value as *const u32 as *const u8, self.ptr.add(self.offset), 4);
		}
		self.offset += 4;
	}

	pub fn write_mov_edx_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov edx, imm32");
		write_byte!(self, 0xBA); // opcode for mov edx, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_esi_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov esi, imm32");
		write_byte!(self, 0xBE); // opcode for mov esi, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_rsi_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov rsi, imm64");
		write_bytes!(self, [0x48, 0xBE]); // REX.W prefix and opcode for mov rsi, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_rdx_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov rdx, imm64");
		write_bytes!(self, [0x48, 0xBA]); // REX.W prefix and opcode for mov rdx, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_rdi_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov rdi, imm64");
		write_bytes!(self, [0x48, 0xBF]); // REX.W prefix and opcode for mov rdi, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_edi_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov edi, imm32");
		write_byte!(self, 0xBF); // opcode for mov edi, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_rcx_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov rcx, imm64");
		write_bytes!(self, [0x48, 0xB9]); // REX.W prefix and opcode for mov rcx, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_ecx_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov ecx, imm32");
		write_byte!(self, 0xB9); // opcode for mov ecx, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_r8_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov r8, imm64");
		write_bytes!(self, [0x49, 0xB8]); // REX.W prefix and opcode for mov r8, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_r8d_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov r8d, imm32");
		write_bytes!(self, [0x41, 0xB8]); // REX prefix and opcode for mov r8d, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_r9_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov r9, imm64");
		write_bytes!(self, [0x49, 0xB9]); // REX.W prefix and opcode for mov r9, imm64
		self.write_imm64(value);
	}

	pub fn write_mov_r9d_imm32(&mut self, value: u32) {
		assert!(self.offset + 5 <= self.size, "Not enough space to write mov r9d, imm32");
		write_bytes!(self, [0x41, 0xB9]); // REX prefix and opcode for mov r9d, imm32
		self.write_imm32(value);
	}

	pub fn write_mov_rax_imm64(&mut self, value: u64) {
		assert!(self.offset + 10 <= self.size, "Not enough space to write mov rax, imm64");
		write_bytes!(self, [0x48, 0xB8]); // REX.W prefix and opcode for mov rax, imm64
		self.write_imm64(value);
	}

	pub fn write_jmp_rax(&mut self) {
		assert!(self.offset + 2 <= self.size, "Not enough space to write jmp rax");
		write_bytes!(self, [0xFF, 0xE0]); // opcode for jmp r/m64 (rax)
	}
}
