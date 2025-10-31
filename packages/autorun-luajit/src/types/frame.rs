// Helper module for managing the frame stack in LuaJIT, which is highly complicated and fragile.

/* -- Lua stack frame ----------------------------------------------------- */

/* Frame type markers in LSB of PC (4-byte aligned) or delta (8-byte aligned:
**
**    PC  00  Lua frame
** delta 001  C frame
** delta 010  Continuation frame
** delta 011  Lua vararg frame
** delta 101  cpcall() frame
** delta 110  ff pcall() frame
** delta 111  ff pcall() frame with active hook
*/
use crate::{GCfunc, LJState, TValue};

pub const FRAME_TYPE: u8 = 3;
pub const FRAME_P: u8 = 4;
pub const FRAME_TYPEP: u8 = FRAME_TYPE | FRAME_P;
pub enum FrameType {
	Lua = 0,
	C = 1,
	Continuation = 2,
	LuaVararg = 3,
	Cpcall = 5,
	FfPcall = 6,
	FfPcallWithHook = 7,
}

#[derive(Debug)]
pub struct Frame {
	// All frames are TValues.
	pub tvalue: *mut TValue,
	pub size: u32,
}

impl Frame {
	pub fn new(tvalue: *mut TValue, size: u32) -> Self {
		Self { tvalue, size }
	}

	pub fn from_debug_ci(state: *mut LJState, i_ci: i32) -> Self {
		let offset = (i_ci as u32) & 0xffff;
		let size = (i_ci as u32) >> 16;

		/*
				uint32_t offset = (uint32_t)ar->i_ci & 0xffff;
		uint32_t size = (uint32_t)ar->i_ci >> 16;
		lj_assertL(offset != 0, "bad frame offset");
		frame = tvref(L->stack) + offset;
		if (size) nextframe = frame + size;
			 */

		let tvstack = unsafe { (*state).stack.as_ptr::<TValue>() };
		let frametv = unsafe { tvstack.add(offset as usize) };

		Frame::new(frametv, size)
	}

	pub fn get_type(&self) -> FrameType {
		let frame_type = unsafe { (*self.tvalue).ftsz & (FRAME_TYPE as u64) } as u8;
		match frame_type {
			0 => FrameType::Lua,
			1 => FrameType::C,
			2 => FrameType::Continuation,
			3 => FrameType::LuaVararg,
			5 => FrameType::Cpcall,
			6 => FrameType::FfPcall,
			7 => FrameType::FfPcallWithHook,
			_ => panic!("Invalid frame type"),
		}
	}

	pub fn is_lua_frame(&self) -> bool {
		matches!(self.get_type(), FrameType::Lua | FrameType::LuaVararg)
	}

	pub fn is_c_frame(&self) -> bool {
		matches!(self.get_type(), FrameType::C | FrameType::Cpcall)
	}

	pub fn get_gc_func(&self) -> anyhow::Result<&mut GCfunc> {
		// frame_gc equivalent,
		// in FR2 this is: gcval((f)-1)
		unsafe {
			let func_tv = self.tvalue.offset(-1);
			let gcfunc = (*func_tv).as_mut::<GCfunc>()?;
			Ok(gcfunc)
		}
	}

	pub fn get_func_tv(&self) -> *mut TValue {
		unsafe { self.tvalue.offset(-1) }
	}
}
