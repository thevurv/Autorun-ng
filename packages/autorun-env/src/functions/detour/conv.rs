use crate::functions::detour::mcode::MCode;

pub enum CallingConvention {
    Win64,
    SysV64,
}

pub enum ArgumentValue {
    Imm32(u32),
    Imm64(u64),
}

// Implementations for writing 1-offset arguments according to different calling conventions
impl CallingConvention {
    // TODO: Support argument values for each argument (not needed for now as only the first argument can be u32/u64)
    fn write_win64_args(&self, mcode: &mut MCode, arg1: ArgumentValue, arg2: Option<u64>, arg3: Option<u64>) {
        match arg1 {
            ArgumentValue::Imm32(val) => mcode.write_mov_edx_imm32(val),
            ArgumentValue::Imm64(val) => mcode.write_mov_rdx_imm64(val),
        }

        if let Some(arg2) = arg2 {
            mcode.write_mov_r8_imm64(arg2);
        }

        if let Some(arg3) = arg3 {
            mcode.write_mov_r9_imm64(arg3);
        }
    }

    fn write_sysv64_args(&self, mcode: &mut MCode, arg1: ArgumentValue, arg2: Option<u64>, arg3: Option<u64>) {
        match arg1 {
            ArgumentValue::Imm32(val) => mcode.write_mov_esi_imm32(val),
            ArgumentValue::Imm64(val) => mcode.write_mov_rsi_imm64(val),
        }

        if let Some(arg2) = arg2 {
            mcode.write_mov_rdx_imm64(arg2);
        }

        if let Some(arg3) = arg3 {
            mcode.write_mov_rcx_imm64(arg3);
        }
    }

    pub fn write_args(&self, mcode: &mut MCode, arg1: ArgumentValue, arg2: Option<u64>, arg3: Option<u64>) {
        match self {
            CallingConvention::Win64 => self.write_win64_args(mcode, arg1, arg2, arg3),
            CallingConvention::SysV64 => self.write_sysv64_args(mcode, arg1, arg2, arg3),
        }
    }
}
