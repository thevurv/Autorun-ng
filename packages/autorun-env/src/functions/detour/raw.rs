use std::ffi::{c_void, c_int};
use region::Allocation;
use autorun_lua::{LuaState, LuaFunction, LuaApi};

// In practice, there should never be more than 16 million refs in a single Lua state
// and I think LuaJIT limits the number of arguments to 255
//
// There can also be up to 64 bits used, but we dont need that much as of now
const CALLBACK_REF_BITS: u32 = 24;
const ARGUMENTS_BITS: u32 = 8;

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

    // Per-OS as calling conventions differ
    #[cfg(target_os = "windows")]
    pub unsafe fn generate_code(&mut self, callback_ref: i32, lua: &LuaApi, num_arguments: i32, handler: HandlerType) {
        unsafe {
            let lua_ptr = lua as *const LuaApi as usize;
            let trampoline_ptr = self.allocation.as_mut_ptr::<u8>();
            let metadata = DetourMetadata::new(callback_ref, num_arguments).0;
            let mut offset = 0;


            // mov edx, metadata (only lower 32-bits needed)
            std::ptr::write(trampoline_ptr.add(offset), 0xBA);
            offset += 1; // opcode for mov edx, imm32
            std::ptr::copy_nonoverlapping(&metadata as *const i32 as *const u8, trampoline_ptr.add(offset), 4);
            offset += 4;

            // mov r8, lua pointer (need to extend to 64-bit with REX prefixa)
            std::ptr::write(trampoline_ptr.add(offset), 0x49);
            offset += 1; // REX.W prefix
            std::ptr::write(trampoline_ptr.add(offset), 0xB8);
            offset += 1; // opcode for mov r8, imm64
            std::ptr::copy_nonoverlapping(&lua_ptr as *const usize as *const u8, trampoline_ptr.add(offset), 8);
            offset += 8;


            // mov r9, original function pointer (need to extend to 64-bit with REX prefix)
            std::ptr::write(trampoline_ptr.add(offset), 0x49);
            offset += 1; // REX.W prefix
            std::ptr::write(trampoline_ptr.add(offset), 0xB9);
            offset += 1; // opcode for mov r9, imm64
            let original_func_ptr = self.original_function_indirection.as_ptr::<u8>() as usize;
            std::ptr::copy_nonoverlapping(&original_func_ptr as *const usize as *const u8, trampoline_ptr.add(offset), 8);
            offset += 8;

            // mov rax, handler address (need to extend to 64-bit with REX prefix)
            std::ptr::write(trampoline_ptr.add(offset), 0x48);
            offset += 1; // REX.W prefix
            std::ptr::write(trampoline_ptr.add(offset), 0xB8);
            offset += 1; // opcode for mov rax, imm
            let handler_addr = handler as usize;
            std::ptr::copy_nonoverlapping(&handler_addr as *const usize as *const u8, trampoline_ptr.add(offset), 8);
            offset += 8;

            // jmp rax
            std::ptr::write(trampoline_ptr.add(offset), 0xFF);
            offset += 1; // opcode for jmp r/m64
            std::ptr::write(trampoline_ptr.add(offset), 0xE0);
            offset += 1; // modrm byte for jmp rax

            debug_assert!(offset <= TRAMPOLINE_SIZE, "Trampoline code exceeds allocated size");
        }
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

    // Per-OS as calling conventions differ
    #[cfg(target_os = "windows")]
    pub unsafe fn generate_code(&mut self, detour_ptr: *const retour::GenericDetour<LuaFunction>, handler: RetourHandlerType) {
        unsafe {
            let trampoline_ptr = self.allocation.as_mut_ptr::<u8>();
            let detour_ptr_usize = detour_ptr as usize;
            let mut offset = 0;

            // mov rdx, detour_ptr (need to extend to 64-bit with REX prefix)
            std::ptr::write(trampoline_ptr.add(offset), 0x48);
            offset += 1; // REX.W prefix
            std::ptr::write(trampoline_ptr.add(offset), 0xBA);
            offset += 1; // opcode for mov rdx, imm64
            std::ptr::copy_nonoverlapping(&detour_ptr_usize as *const usize as *const u8, trampoline_ptr.add(offset), 8);
            offset += 8;

            // mov rax, handler address (need to extend to 64-bit with REX prefix)
            std::ptr::write(trampoline_ptr.add(offset), 0x48);
            offset += 1; // REX.W prefix
            std::ptr::write(trampoline_ptr.add(offset), 0xB8);
            offset += 1; // opcode for mov rax, imm
            let handler_addr = handler as usize;
            std::ptr::copy_nonoverlapping(&handler_addr as *const usize as *const u8, trampoline_ptr.add(offset), 8);
            offset += 8;

            // jmp rax
            std::ptr::write(trampoline_ptr.add(offset), 0xFF);
            offset += 1; // opcode for jmp r/m64
            std::ptr::write(trampoline_ptr.add(offset), 0xE0);
            offset += 1; // modrm byte for jmp rax

            debug_assert!(offset <= TRAMPOLINE_SIZE, "Retour trampoline code exceeds allocated size");
        }

    }

    pub fn as_function(&self) -> LuaFunction {
        unsafe { std::mem::transmute(self.allocation.as_ptr::<c_void>()) }
    }
}