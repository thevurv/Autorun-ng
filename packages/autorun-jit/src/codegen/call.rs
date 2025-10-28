use crate::mcode::MCode;

pub enum Arg {
	Imm32(u32),
	Imm64(u64),
}

pub enum Register {
	XCX,
	XDX,
	XDI,
	XSI,
	R8X,
	R9X,
}

impl Register {
	pub fn write_mov_imm64(&self, mcode: &mut MCode, value: u64) {
		match self {
			Register::XCX => mcode.write_mov_rcx_imm64(value),
			Register::XDX => mcode.write_mov_rdx_imm64(value),
			Register::XDI => mcode.write_mov_rdi_imm64(value),
			Register::XSI => mcode.write_mov_rsi_imm64(value),
			Register::R8X => mcode.write_mov_r8_imm64(value),
			Register::R9X => mcode.write_mov_r9_imm64(value),
		}
	}

	pub fn write_mov_imm32(&self, mcode: &mut MCode, value: u32) {
		match self {
			Register::XCX => mcode.write_mov_ecx_imm32(value),
			Register::XDX => mcode.write_mov_edx_imm32(value),
			Register::XDI => mcode.write_mov_edi_imm32(value),
			Register::XSI => mcode.write_mov_esi_imm32(value),
			Register::R8X => mcode.write_mov_r8d_imm32(value),
			Register::R9X => panic!("R9 does not have a 32-bit mov instruction"),
		}
	}

	pub fn write_arg(&self, mcode: &mut MCode, arg: &Arg) {
		match arg {
			Arg::Imm32(val) => self.write_mov_imm32(mcode, *val),
			Arg::Imm64(val) => self.write_mov_imm64(mcode, *val),
		}
	}
}

pub enum CallingConvention {
	Win64,
	SysV64,
}

// Some arguments can be omitted, but they will still "consume" a register, this makes it easier to handle situations where not all arguments are needed
pub type ArgList = Vec<Option<Arg>>;

pub const WIN64_ARG_REGISTERS: [Register; 4] = [Register::XCX, Register::XDX, Register::R8X, Register::R9X];

pub const SYSV64_ARG_REGISTERS: [Register; 6] = [
	Register::XDI,
	Register::XSI,
	Register::XDX,
	Register::XCX,
	Register::R8X,
	Register::R9X,
];

// Of course, you could pass more arguments on the stack, but for now we only support register arguments
impl CallingConvention {
	fn get_max_args(&self) -> usize {
		match self {
			CallingConvention::Win64 => WIN64_ARG_REGISTERS.len(),
			CallingConvention::SysV64 => SYSV64_ARG_REGISTERS.len(),
		}
	}

	fn write_args_with_registers(&self, mcode: &mut MCode, args: &ArgList, registers: &[Register]) {
		if args.len() > self.get_max_args() {
			panic!("Too many arguments for calling convention");
		}

		for (i, arg_opt) in args.iter().enumerate() {
			if let Some(arg) = arg_opt {
				registers[i].write_arg(mcode, arg);
			}
		}
	}

	pub fn setup_arguments(&self, mcode: &mut MCode, args: &ArgList) {
		match self {
			CallingConvention::Win64 => {
				self.write_args_with_registers(mcode, args, &WIN64_ARG_REGISTERS);
			}
			CallingConvention::SysV64 => {
				self.write_args_with_registers(mcode, args, &SYSV64_ARG_REGISTERS);
			}
		}
	}
}
