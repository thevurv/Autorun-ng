//! This module emits the necessary trampoline LJ bytecode for detouring Lua functions.

use anyhow::Context;
use autorun_luajit::bytecode::{BCWriter, Op};
use autorun_luajit::{BCIns, GCProto, GCfuncL, ProtoFlags};

const MINIMUM_BYTECODES: u32 = 15;
const MINIMUM_UPVALUES: u8 = 1;

/// Assumes detour function is in UV 0.
/// # Detouring
/// This emits a trampoline which sets up a new function that specifically
/// pulls the detour function from upvalue 0, moves all arguments into
/// their correct registers, and then calls the detour function with CALLT.
/// This trampoline completely replaces the target function's bytecode and does not consume an extra level of stack.
pub fn overwrite_with_trampoline(gcfunc_l: &GCfuncL) -> anyhow::Result<()> {
	let mut writer = BCWriter::from_gcfunc_l(gcfunc_l).context("Failed to create BCWriter from GCfuncL")?;
	let proto = gcfunc_l.get_proto().context("Failed to get proto from GCfuncL")?;
	let proto = unsafe { proto.as_mut().context("Failed to dereference proto")? };

	if proto.sizebc < MINIMUM_BYTECODES {
		anyhow::bail!("Target function's proto is too small to overwrite with trampoline.");
	}

	if proto.sizeuv < MINIMUM_UPVALUES {
		anyhow::bail!("Target function's proto does not have enough upvalues for detour trampoline.");
	}

	if proto.has_flag(ProtoFlags::Vararg) {
		anyhow::bail!("Detour trampoline does not support vararg functions.");
	}

	let nargs = proto.numparams;
	let maxslots = 2 * nargs + 2;
	proto.framesize = maxslots; // update framesize to accommodate trampoline

	// Trampoline basics:
	// FUNCF maxslots
	// UGET detour_function_register, 0
	// MOV arg_registers...
	// CALLT detour_function_register, nargs+1

	writer.write(BCIns::from_ad(Op::FUNCF, maxslots, 0))?;
	let mut free_register = nargs; // 0-indexed register after arguments
	let detour_register = free_register;

	writer.write(BCIns::from_ad(Op::UGET, free_register, 0))?; // get detour function from upvalue 0

	// Begin allocating and setting up the argument registers, they are all at 0-nargs, we need them to nargs+1-2*nargs
	free_register += 1; // No idea why, but a register needs to be skipped here. Maybe something to do with frame linkage?
	for i in 0..nargs {
		free_register += 1;
		writer.write(BCIns::from_ad(Op::MOV, free_register, i as i16))?;
	}

	// write final callt
	writer.write(BCIns::from_ad(Op::CALLT, detour_register, (nargs + 1) as i16))?;

	// all done, no return necessary as CALLT handles it
	// CALLT also jumps directly to the function, so no need for us to fix up
	// the sizebc field
	Ok(())
}
