// Helper module for managing the frame stack in LuaJIT, which is highly complicated and fragile.
use crate::{BCIns, GCRef, GCfunc, LJ_FR2, LJState, TValue};

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
	payload_copy: TValue,
}

impl Frame {
	pub fn new(tvalue: *mut TValue, size: u32) -> Self {
		Self {
			tvalue,
			size,
			payload_copy: unsafe { *tvalue.offset(-1) },
		}
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

		// traversal is in backwards order
		let start = unsafe { (*state).stack.as_ptr::<TValue>().add(LJ_FR2 as usize) };
		let mut frame = unsafe { (*state).base.offset(-1) };

		while frame > start {
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

	fn overwrite_payload_gcr(&mut self, new_gcr: GCRef) {
		unsafe {
			(*self.tvalue.offset(-1)).gcr = new_gcr;
		}
	}

	fn restore_payload(&mut self) {
		unsafe {
			*self.tvalue.offset(-1) = self.payload_copy;
		}
	}

	pub fn mark_as_dummy_frame(&mut self, state: *mut LJState) {
		// We can overwrite the payload to point to the Lua state,
		// which is a special GCRef that indicates a dummy frame.
		self.overwrite_payload_gcr(GCRef::from_ptr(state));
	}

	pub fn restore_from_dummy_frame(&mut self) {
		self.restore_payload();
	}

	pub fn get_pc(&self) -> *const BCIns {
		unsafe { (*self.tvalue).ftsz as *const BCIns }
	}

	pub fn get_previous_lua_frame(&self) -> Self {
		let bc_ins_ptr = self.get_pc();
		let bc_ins = unsafe { bc_ins_ptr.offset(-1).read_unaligned() };
		let bc_a = bc_ins.a();
		let offset = (1 + LJ_FR2 + (bc_a as u32)) as isize;

		Frame::new(unsafe { self.tvalue.offset(-offset) }, 0) // size is not relevant here
	}

	pub fn get_delta_size(&self) -> u64 {
		unsafe { (*self.tvalue).ftsz & !(FRAME_TYPEP as u64) }
	}

	pub fn get_previous_delta_frame(&self) -> Self {
		let size = self.get_delta_size() as usize;
		Frame::new(unsafe { self.tvalue.byte_sub(size) }, size as u32)
	}
}
