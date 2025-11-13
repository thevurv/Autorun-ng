//! Provides advanced cloning functionality for Lua functions.
//! This is an ultimate deep clone that basically duplicates everything about a Lua function,
//! including its upvalues, bytecode, and other internal structures.

use anyhow::Context;
use autorun_luajit::{BCIns, GCHeader, GCProto, GCRef, GCSize, GCfunc, GCfuncL, LJState, TValue, mem_newgco, push_tvalue};

/// Clones the given Lua function deeply, duplicating its internal structures.
/// Pushes the cloned function onto the Lua stack.
pub fn clone(lj_state: &mut LJState, target_func: &GCfuncL) -> anyhow::Result<()> {
	dbg!(&target_func);
	// Proto must be cloned first.
	let proto = target_func.get_proto()?;
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
		dbg!(&target_func);

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
