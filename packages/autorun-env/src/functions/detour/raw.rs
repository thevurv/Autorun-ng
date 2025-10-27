use std::ffi::{c_void, c_int};
use region::Allocation;
use autorun_lua::{LuaState, LuaFunction, LuaApi};
use crate::functions::detour::conv::{ArgumentValue, CallingConvention};
use crate::functions::detour::mcode::MCode;

// In practice, there should never be more than 16 million refs in a single Lua state
// and I think LuaJIT limits the number of arguments to 255
//
// There can also be up to 64 bits used, but we dont need that much as of now
const CALLBACK_REF_BITS: u32 = 24;
const ARGUMENTS_BITS: u32 = 8;

#[cfg(target_os = "windows")]
const CALLING_CONVENTION: CallingConvention = CallingConvention::Win64;

#[cfg(target_os = "linux")]
const CALLING_CONVENTION: CallingConvention = CallingConvention::SysV64;

#[derive(Debug, Clone, Copy)]
pub struct DetourMetadata(i32);

impl DetourMetadata {
    pub fn new(callback_ref: i32, num_arguments: i32) -> Self {
        let packed = ((callback_ref & ((1 << CALLBACK_REF_BITS) - 1)) << ARGUMENTS_BITS)
            | (num_arguments & ((1 << ARGUMENTS_BITS) - 1));
        Self(packed)
    }

    pub fn from_packed(packed: i32) -> Self {
        Self(packed)
    }

    pub fn callback_ref(&self) -> i32 {
        (self.0 >> ARGUMENTS_BITS) & ((1 << CALLBACK_REF_BITS) - 1)
    }

    pub fn num_arguments(&self) -> i32 {
        self.0 & ((1 << ARGUMENTS_BITS) - 1)
    }
}

type HandlerType = extern "C-unwind" fn(state: *mut LuaState, metadata: i32, lua: *const LuaApi, orignal_function: *const LuaFunction) -> c_int;
type RetourHandlerType = extern "C-unwind" fn(state: *mut LuaState, detour: *const retour::GenericDetour<LuaFunction>) -> c_int;

const TRAMPOLINE_SIZE: usize = 64;

pub struct CallbackTrampoline {
    allocation: Allocation,
    // Used to send a function pointer to a C function that will call the original function via indirection
    // to break a circular dependency
    original_function_indirection: Allocation
}

impl CallbackTrampoline {
    pub fn allocate() -> anyhow::Result<Self> {
        let trampoline = region::alloc(TRAMPOLINE_SIZE, region::Protection::READ_WRITE);
        let original_function_indirection = region::alloc(std::mem::size_of::<usize>(), region::Protection::READ_WRITE);

        if let Ok(allocation) = trampoline && let Ok(original_function_indirection) = original_function_indirection {
            Ok(Self { allocation, original_function_indirection } )
        } else {
            Err(anyhow::anyhow!("Failed to allocate trampoline"))
        }
    }

    pub unsafe fn make_executable(&mut self) -> anyhow::Result<()> {
        unsafe {
            region::protect(self.allocation.as_ptr::<c_void>(), TRAMPOLINE_SIZE, region::Protection::READ_EXECUTE)
                .map_err(|_| anyhow::anyhow!("Failed to set trampoline as executable"))
        }
    }

    pub fn generate_code(&mut self, callback_ref: i32, lua: &LuaApi, num_arguments: i32, handler: HandlerType) {
        let lua_ptr = lua as *const LuaApi as usize;
        let trampoline_ptr = self.allocation.as_mut_ptr::<u8>();
        let metadata = DetourMetadata::new(callback_ref, num_arguments).0;
        let mut mcode = MCode::new(trampoline_ptr, TRAMPOLINE_SIZE);

        CALLING_CONVENTION.write_args(&mut mcode, ArgumentValue::Imm32(metadata as u32), Some(lua_ptr as u64), Some(self.original_function_indirection.as_ptr::<u8>() as u64));

        mcode.write_mov_rax_imm64(handler as u64);
        mcode.write_jmp_rax();
    }

    pub fn write_original_function_pointer(&mut self, func: LuaFunction) {
        unsafe {
            let func_ptr = func as usize;
            std::ptr::copy_nonoverlapping(&func_ptr as *const usize as *const u8, self.original_function_indirection.as_mut_ptr::<u8>(), std::mem::size_of::<usize>());
        }
    }
}

impl Into<LuaFunction> for &CallbackTrampoline {
    fn into(self) -> LuaFunction {
        unsafe { std::mem::transmute(self.allocation.as_ptr::<c_void>()) }
    }
}

// A JIT-ed trampoline to call an original C function as a Lua C function
// Requires a retour detour at a specific location to jump to this trampoline
pub struct RetourLuaTrampoline {
    allocation: Allocation,
}

impl RetourLuaTrampoline {
    pub fn allocate() -> anyhow::Result<Self> {
        let trampoline = region::alloc(TRAMPOLINE_SIZE, region::Protection::READ_WRITE);

        if let Ok(allocation) = trampoline {
            Ok(Self { allocation } )
        } else {
            Err(anyhow::anyhow!("Failed to allocate retour trampoline"))
        }
    }

    pub unsafe fn make_executable(&mut self) -> anyhow::Result<()> {
        unsafe {
            region::protect(self.allocation.as_ptr::<c_void>(), TRAMPOLINE_SIZE, region::Protection::READ_EXECUTE)
                .map_err(|_| anyhow::anyhow!("Failed to set retour trampoline as executable"))
        }
    }

    pub unsafe fn generate_code(&mut self, detour_ptr: *const retour::GenericDetour<LuaFunction>, handler: RetourHandlerType) {
        let trampoline_ptr = self.allocation.as_mut_ptr::<u8>();
        let detour_ptr_usize = detour_ptr as usize;
        let mut mcode = MCode::new(trampoline_ptr, TRAMPOLINE_SIZE);

        CALLING_CONVENTION.write_args(&mut mcode, ArgumentValue::Imm64(detour_ptr_usize as u64), None, None);

        mcode.write_mov_rax_imm64(handler as u64);
        mcode.write_jmp_rax();
    }

    pub fn as_function(&self) -> LuaFunction {
        unsafe { std::mem::transmute(self.allocation.as_ptr::<c_void>()) }
    }
}