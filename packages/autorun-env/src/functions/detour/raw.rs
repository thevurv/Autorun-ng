use autorun_jit::Function;
use autorun_jit::{Arg, CallingConvention, Jump};
use autorun_lua::{LuaApi, LuaFunction, LuaState};
use std::ffi::c_int;

const CALLBACK_REF_BITS: u32 = 24;
const RESERVED_BITS: u32 = 8;

#[cfg(target_os = "windows")]
const CALLING_CONVENTION: CallingConvention = CallingConvention::Win64;

#[cfg(target_os = "linux")]
const CALLING_CONVENTION: CallingConvention = CallingConvention::SysV64;

#[derive(Debug, Clone, Copy)]
pub struct DetourMetadata(i32);

// Register-sized metadata passed to the detour handler, encodes a callback reference and reserved bits
// In the future, we can use these for flags or other data, but for now they are unused.
impl DetourMetadata {
	pub fn new(callback_ref: i32, reserved: i32) -> Self {
		let packed =
			((callback_ref & ((1 << CALLBACK_REF_BITS) - 1)) << RESERVED_BITS) | (reserved & ((1 << RESERVED_BITS) - 1));
		Self(packed)
	}

	pub fn from_packed(packed: i32) -> Self {
		Self(packed)
	}

	pub fn callback_ref(&self) -> i32 {
		(self.0 >> RESERVED_BITS) & ((1 << CALLBACK_REF_BITS) - 1)
	}

	pub fn reserved(&self) -> i32 {
		self.0 & ((1 << RESERVED_BITS) - 1)
	}
}

type HandlerType = extern "C-unwind" fn(
	state: *mut LuaState,
	metadata: i32,
	lua: *const LuaApi,
	orignal_function: *const LuaFunction,
) -> c_int;
type RetourHandlerType = extern "C-unwind" fn(state: *mut LuaState, detour: *const retour::GenericDetour<LuaFunction>) -> c_int;

const TRAMPOLINE_SIZE: usize = 64;

pub fn make_detour_trampoline(
	lua: &LuaApi,
	callback_ref: i32,
	original_function_ptr: *const usize,
	handler: HandlerType,
) -> anyhow::Result<Function> {
	let mut trampoline = Function::allocate(TRAMPOLINE_SIZE);
	let metadata = DetourMetadata::new(callback_ref, 0).0;
	let lua_ptr = lua as *const LuaApi as usize;

	CALLING_CONVENTION.emit_call(
		&mut trampoline.mcode,
		&vec![
			None,                                           // don't overwrite the lua_State pointer
			Some(Arg::Imm32(metadata as u32)),              // metadata
			Some(Arg::Imm64(lua_ptr as u64)),               // lua api pointer
			Some(Arg::Imm64(original_function_ptr as u64)), // original function pointer
		],
	);

	let jmp = Jump::Absolute(handler as u64);
	jmp.write_to_mcode(&mut trampoline.mcode);

	trampoline
		.make_executable()
		.map_err(|_| anyhow::anyhow!("Failed to make detour trampoline executable"))?;
	Ok(trampoline)
}

pub fn make_retour_lua_trampoline(
	detour_ptr: *const retour::GenericDetour<LuaFunction>,
	handler: RetourHandlerType,
) -> anyhow::Result<Function> {
	let mut trampoline = Function::allocate(TRAMPOLINE_SIZE);
	let detour_ptr_usize = detour_ptr as usize;

	CALLING_CONVENTION.emit_call(
		&mut trampoline.mcode,
		&vec![
			None,                                      // don't overwrite the lua_State pointer
			Some(Arg::Imm64(detour_ptr_usize as u64)), // detour pointer
		],
	);

	let jmp = Jump::Absolute(handler as u64);
	jmp.write_to_mcode(&mut trampoline.mcode);

	trampoline
		.make_executable()
		.map_err(|_| anyhow::anyhow!("Failed to make retour trampoline executable"))?;
	Ok(trampoline)
}
