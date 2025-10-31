// Helper module for managing the frame stack in LuaJIT, which is highly complicated and fragile.
use crate::{BCIns, GCfunc, LJ_FR2, LJState, TValue};

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

#[derive(Debug, Copy, Clone)]
/// Frames are stored in the LJ state as TValues with their payloads below them in the stack.
/// This struct helps manage and interpret these frames.
pub struct Frame {
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
		let tvstack = unsafe { (*state).stack.as_ptr::<TValue>() };
		let frametv = unsafe { tvstack.add(offset as usize) };

		Frame::new(frametv, size)
	}

	pub fn walk_stack(state: *mut LJState) -> Vec<Frame> {
		let mut frames = Vec::new();
		// We are going to copy this function, just without the levels so we walk every frame
		/*
				cTValue *lj_debug_frame(lua_State *L, int level, int *size)
		{
		  cTValue *frame, *nextframe, *bot = tvref(L->stack)+LJ_FR2;
		  /* Traverse frames backwards. */
		  for (nextframe = frame = L->base-1; frame > bot; ) {
			if (frame_gc(frame) == obj2gco(L))
			  level++;  /* Skip dummy frames. See lj_err_optype_call(). */
			if (level-- == 0) {
			  *size = (int)(nextframe - frame);
			  return frame;  /* Level found. */
			}
			nextframe = frame;
			if (frame_islua(frame)) {
			  frame = frame_prevl(frame);
			} else {
			  if (frame_isvarg(frame))
			level++;  /* Skip vararg pseudo-frame. */
			  frame = frame_prevd(frame);
			}
		  }
		  *size = level;
		  return NULL;  /* Level not found. */
		}

				 */

		let bot = unsafe { (*state).stack.as_ptr::<TValue>().add(LJ_FR2 as usize) };
		let mut frame = unsafe { (*state).base.offset(-1) };

		while frame > bot {
			let current_frame = Frame::new(frame, 0); // size is not relevant here
			frames.push(current_frame);

			if current_frame.is_lua_frame() {
				frame = current_frame.get_previous_lua_frame().tvalue;
			} else {
				frame = current_frame.get_previous_delta_frame().tvalue;
			}
		}

		frames
	}

	pub fn get_type(&self) -> FrameType {
		let frame_type = unsafe { (*self.tvalue).ftsz & (FRAME_TYPE as u64) } as u8;
		dbg!(&frame_type);
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
		matches!(self.get_type(), FrameType::Lua)
	}

	pub fn is_c_frame(&self) -> bool {
		matches!(self.get_type(), FrameType::C)
	}

	pub fn get_gc_func(&self) -> anyhow::Result<&mut GCfunc> {
		unsafe {
			let func_tv = self.tvalue.offset(-1);
			let gcfunc = (*func_tv).as_mut::<GCfunc>()?;
			Ok(gcfunc)
		}
	}

	pub fn get_func_tv(&self) -> *mut TValue {
		unsafe { self.tvalue.offset(-1) }
	}

	pub fn replace_func_tv(&mut self, new_func_tv: *mut TValue) {
		unsafe {
			std::ptr::copy_nonoverlapping(new_func_tv, self.tvalue.offset(-1), 1);
		}
	}

	pub fn get_pc(&self) -> *const BCIns {
		// #define frame_pc(f)		((const BCIns *)frame_ftsz(f))

		unsafe { (*self.tvalue).ftsz as *const BCIns }
	}

	// #define frame_prevl(f)		((f) - (1+LJ_FR2+bc_a(frame_pc(f)[-1])))
	pub fn get_previous_lua_frame(&self) -> Self {
		dbg!(&self);
		let bc_ins_ptr = self.get_pc();
		dbg!(&bc_ins_ptr);
		let bc_ins = unsafe { bc_ins_ptr.offset(-1).read_unaligned() };
		dbg!(&bc_ins);
		let bc_a = bc_ins.a();
		dbg!(&bc_a);
		let offset = (1 + LJ_FR2 + (bc_a as u32)) as isize;
		dbg!(&offset);

		Frame::new(unsafe { self.tvalue.offset(-offset) }, 0) // size is not relevant here
	}

	pub fn get_sized(&self) -> u64 {
		// #define frame_sized(f)		(frame_ftsz(f) & ~FRAME_TYPEP)
		unsafe { (*self.tvalue).ftsz & !(FRAME_TYPEP as u64) }
	}

	pub fn get_previous_delta_frame(&self) -> Self {
		// #define frame_prevd(f)		((TValue *)((char *)(f) - frame_sized(f)))
		let size = self.get_sized() as usize;
		dbg!(&size);
		Frame::new(unsafe { self.tvalue.byte_sub(size) }, size as u32)
	}
}
