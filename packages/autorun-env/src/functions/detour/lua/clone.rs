//! Provides advanced cloning functionality for Lua functions.
//! This is an ultimate deep clone that basically duplicates everything about a Lua function,
//! including its upvalues, bytecode, and other internal structures.

use anyhow::Context;
use autorun_luajit::{
	BCIns, GCHeader, GCProto, GCRef, GCSize, GCUpval, GCfunc, GCfuncL, LJState, TValue, mem_newgco, push_tvalue,
};
use std::mem::offset_of;

/// Clones the given Lua function deeply, duplicating its internal structures.
/// Pushes the cloned function onto the Lua stack.
pub fn clone(lj_state: &mut LJState, target_func: *mut GCfuncL) -> anyhow::Result<()> {
	dbg!(&target_func);
	// Proto must be cloned first.
	let proto = unsafe { (*target_func).get_proto()? };
	let proto_size = unsafe { (*proto).sizept } as GCSize;
	let proto_uv_size = unsafe { (*proto).sizeuv } as GCSize;

	dbg!(unsafe { &(*proto) });
	dbg!(&proto_size);
	dbg!(&proto_uv_size);
	// What we want to do is allocate a new proto and copy over everything, but
	// keep the GCHeader intact or else the GC system will get super confused.
	let new_proto_ptr = unsafe { mem_newgco::<GCProto>(lj_state, proto_size)? };
	unsafe {
		// Treat every pointer as raw bytes, since sizept is specified as bytes.
		std::ptr::copy_nonoverlapping(
			(proto as *const u8).byte_add(size_of::<GCHeader>()),
			(new_proto_ptr as *mut u8).byte_add(size_of::<GCHeader>()),
			proto_size as usize - size_of::<GCHeader>(),
		);
	};

	dbg!("Fixing up proto offsets...");

	// Fix up internal offsets within the proto
	fixup_proto_offsets(proto, new_proto_ptr)?;

	// Now create the new function, we'll keep everything about it intact, of course except for the GCHeader like
	// the proto.

	// Lua functions store their upvalue array in a contiguous block after the main struct
	let func_size =
		size_of::<GCfuncL>() as GCSize - size_of::<GCRef>() as GCSize + size_of::<GCRef>() as GCSize * proto_uv_size;
	let new_func_ptr = unsafe { mem_newgco::<GCfunc>(lj_state, func_size)? };

	unsafe {
		let target_func = target_func as *const GCfuncL as *const u8;

		std::ptr::copy_nonoverlapping(
			target_func.byte_add(size_of::<GCHeader>()),
			(new_func_ptr as *mut u8).byte_add(size_of::<GCHeader>()),
			func_size as usize - size_of::<GCHeader>(),
		);
	};

	// Fix pc pointer to point to the new proto's bytecode
	unsafe {
		// Bytecode is located immediately after the GCProto struct
		let bc_ptr = new_proto_ptr.byte_add(size_of::<GCProto>()) as *mut BCIns;
		(*new_func_ptr).header_mut().pc.set_ptr(bc_ptr);
	}

	clone_upvalue_list(lj_state, new_func_ptr)?;

	dbg!(unsafe { &(*new_func_ptr).l });
	// Create a TValue for the new function and push it onto the stack
	let func_tvalue = TValue::from_ptr(new_func_ptr);
	push_tvalue(lj_state, &func_tvalue);

	Ok(())
}

pub fn fixup_proto_offsets(original_proto: *mut GCProto, new_proto: *mut GCProto) -> anyhow::Result<()> {
	// Basically, the proto contains several offsets that point to various internal structures within its own allocation.
	// Technically speaking, we can hardcode these, but it would be better to read them from the original proto and adjust them accordingly.

	let original_base = original_proto as usize;
	let new_base = new_proto as usize;

	let k_offset = unsafe { (*original_proto).k.ptr64 as usize - original_base };
	let uv_offset = unsafe { (*original_proto).uv.ptr64 as usize - original_base };
	let lineinfo_offset = unsafe { (*original_proto).lineinfo.ptr64 as usize - original_base };
	let uvinfo_offset = unsafe { (*original_proto).uvinfo.ptr64 as usize - original_base };
	let varinfo_offset = unsafe { (*original_proto).varinfo.ptr64 as usize - original_base };

	// apply offsets to new proto
	unsafe {
		(*new_proto).k.ptr64 = (new_base + k_offset) as u64;
		(*new_proto).uv.ptr64 = (new_base + uv_offset) as u64;
		(*new_proto).lineinfo.ptr64 = (new_base + lineinfo_offset) as u64;
		(*new_proto).uvinfo.ptr64 = (new_base + uvinfo_offset) as u64;
		(*new_proto).varinfo.ptr64 = (new_base + varinfo_offset) as u64;
	}

	Ok(())
}

pub fn clone_upvalue_list(lj_state: &mut LJState, func: *mut GCfunc) -> anyhow::Result<()> {
	// The function stores a contiguous array of upvalue **references** after the main struct.
	// We need to go through each upvalue reference and clone the actual upvalue objects they point to.

	let gcfunc = unsafe { func.as_mut().context("Failed to dereference GCfunc.")? };
	let gcfunc_l = gcfunc.as_l_mut().context("Function is not a Lua function.")?;
	let nupvalues = gcfunc_l.header.nupvalues;
	autorun_log::debug!("Cloning {} upvalues...", nupvalues);
	unsafe {
		autorun_log::debug!("uvptr: {:p}", (gcfunc_l as *mut GCfuncL).byte_add(offset_of!(GCfuncL, uvptr)));
		autorun_log::debug!("uvptr offset: {}", offset_of!(GCfuncL, uvptr));
	}

	for i in 0..nupvalues {
		autorun_log::debug!("Cloning upvalue {}...", i);
		let upvalue_gcr = unsafe { gcfunc_l.uvptr.as_mut_ptr().add(i as usize) };
		let upvalue_gcr = unsafe { upvalue_gcr.as_mut().context("Failed to deref upvalue GCRef.")? };

		let upvalue = unsafe { upvalue_gcr.as_direct_ptr::<GCUpval>() };
		if upvalue.is_null() {
			anyhow::bail!("Upvalue pointer is null.");
		}

		let new_upvalue_ptr = unsafe { mem_newgco::<GCUpval>(lj_state, size_of::<GCUpval>() as GCSize)? };

		let expected_address = (gcfunc_l as *const GCfuncL as usize) + 0x28usize + ((i as usize) * 8);

		autorun_log::debug!(
			"Upvalue {}: reading from {:p}, expected 0x{:x}",
			i,
			upvalue_gcr as *mut GCRef,
			expected_address,
		);

		autorun_log::debug!("Original GCR pointer: {:p}", upvalue_gcr.gcptr64 as *const ());
		autorun_log::debug!("Original upvalue pointer: {:p}", upvalue);
		autorun_log::debug!("New upvalue pointer: {:p}", new_upvalue_ptr);

		dbg!(&unsafe { &*upvalue });
		// copy excluding GCHeader
		unsafe {
			std::ptr::copy_nonoverlapping(
				(upvalue as *const u8).byte_add(size_of::<GCHeader>()),
				(new_upvalue_ptr as *mut u8).byte_add(size_of::<GCHeader>()),
				size_of::<GCUpval>() - size_of::<GCHeader>(),
			);
		}

		// Close it out
		close_upvalue(new_upvalue_ptr)?;

		// update the reference to point to the new upvalue
		upvalue_gcr.set_ptr(new_upvalue_ptr);
		autorun_log::debug!("Upvalue {} cloned.", i);
	}

	Ok(())
}

fn close_upvalue(new_upvalue_ptr: *mut GCUpval) -> anyhow::Result<()> {
	unsafe {
		let new_upvalue = new_upvalue_ptr.as_mut().context("Failed to deref new upvalue.")?;
		dbg!(&new_upvalue);
		new_upvalue.closed = 1;

		// copy TV from wherever it was pointing to
		autorun_log::debug!("Reading original TV pointer: {:p}", new_upvalue.v.as_ptr::<TValue>());
		new_upvalue.uv.tv = new_upvalue.v.as_ptr::<TValue>().read();
		// point v to the new TV
		new_upvalue
			.v
			.set_ptr(new_upvalue_ptr.byte_add(offset_of!(GCUpval, uv)) as *mut TValue);
	};

	Ok(())
}
