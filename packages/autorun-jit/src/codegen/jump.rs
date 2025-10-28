pub enum Jump {
	// We currently have no interest in relative jumps
	Absolute(u64),
}

impl Jump {
	pub fn write_to_mcode(&self, mcode: &mut crate::mcode::MCode) {
		match self {
			Jump::Absolute(target) => {
				// mov rax, target
				// jmp rax
				mcode.write_mov_rax_imm64(*target);
				mcode.write_jmp_rax();
			}
		}
	}
}
