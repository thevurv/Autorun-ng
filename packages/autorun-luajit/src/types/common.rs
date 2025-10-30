// Subset of lj_obj.h

use std::ffi::{c_int, c_void};

// IMPORTANT: GMod's LUA_IDSIZE was randomly changed to 128 instead of 60 like in vanilla LuaJIT
#[cfg(feature = "gmod")]
pub const LUA_IDSIZE: i32 = 128;
#[cfg(not(feature = "gmod"))]
pub const LUA_IDSIZE: i32 = 60;

pub const LJ_TNIL: u32 = !0u32;
pub const LJ_TFALSE: u32 = !1u32;
pub const LJ_TTRUE: u32 = !2u32;
pub const LJ_TLIGHTUD: u32 = !3u32;
pub const LJ_TSTR: u32 = !4u32;
pub const LJ_TUPVAL: u32 = !5u32;
pub const LJ_TTHREAD: u32 = !6u32;
pub const LJ_TPROTO: u32 = !7u32;
pub const LJ_TFUNC: u32 = !8u32;
pub const LJ_TTRACE: u32 = !9u32;
pub const LJ_TCDATA: u32 = !10u32;
pub const LJ_TTAB: u32 = !11u32;
pub const LJ_TUDATA: u32 = !12u32;
pub const LJ_TNUMX: u32 = !13u32;

pub trait IntoLJType {
	const LJ_TYPE: u32;
}

pub type MSize = u64;
pub type GCSize = u64;

#[repr(C)]
pub struct LJDebug {
	pub event: i32,
	pub name: *const u8,
	pub namewhat: *const u8,
	pub what: *const u8,
	pub source: *const u8,
	pub currentline: i32,
	pub nups: i32,
	pub linedefined: i32,
	pub lastlinedefined: i32,
	pub short_src: [u8; LUA_IDSIZE as usize],
	pub i_ci: i32,
}

pub type LuaHookFn = extern "C-unwind" fn(l: *mut LJState, ar: *mut LJDebug);
pub type LuaCFunction = extern "C-unwind" fn(l: *mut LJState) -> i32;
pub type LuaAllocFn = extern "C-unwind" fn(
	ud: *mut core::ffi::c_void,
	ptr: *mut core::ffi::c_void,
	osize: usize,
	nsize: usize,
) -> *mut core::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MRef {
	pub ptr64: u64,
}

impl MRef {
	// equivalent to the mref macro in LuaJIT
	pub fn as_ptr<T>(&self) -> *mut T {
		self.ptr64 as *mut T
	}

	pub fn tvref(&self) -> *mut TValue {
		self.as_ptr::<TValue>()
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GCRef {
	pub gcptr64: u64,
}

impl GCRef {
	// equivalent to the gcref macro in LuaJIT
	pub fn as_ptr<T>(&self) -> *mut T {
		self.gcptr64 as *mut T
	}
}

impl PartialEq<Self> for GCRef {
	fn eq(&self, other: &Self) -> bool {
		// some GCRefs don't have encoded TV information, so strip them
		let masked_self = self.gcptr64 & LJ_GCVMASK;
		let masked_other = other.gcptr64 & LJ_GCVMASK;
		dbg!(masked_self);
		dbg!(masked_other);

		masked_self == masked_other
	}
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GCHeader {
	pub nextgc: GCRef,
	pub marked: u8,
	pub gct: u8,
}

// #define LJ_GCVMASK		(((uint64_t)1 << 47) - 1)
pub const LJ_GCVMASK: u64 = (1u64 << 47) - 1;

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub union TValue {
	pub u64: u64,
	pub f64: f64,
	pub gcr: GCRef,
	pub it64: i64,
}

impl TValue {
	pub fn as_ptr<T: IntoLJType>(&self) -> anyhow::Result<*mut T> {
		if self.itype() != T::LJ_TYPE {
			anyhow::bail!("TValue type mismatch: expected {}, got {}", T::LJ_TYPE, self.itype());
		}

		Ok(unsafe { (self.gcr.gcptr64 & LJ_GCVMASK) as *mut T })
	}

	pub fn as_ref<T: IntoLJType>(&self) -> anyhow::Result<&T> {
		Ok(unsafe { &*self.as_ptr::<T>()? })
	}

	pub fn as_mut<T: IntoLJType>(&mut self) -> anyhow::Result<&mut T> {
		Ok(unsafe { &mut *self.as_ptr::<T>()? })
	}

	pub fn itype(&self) -> u32 {
		unsafe { ((self.it64 >> 47) & 0xFFFFFFFF) as u32 }
	}
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GCFuncHeader {
	pub header: GCHeader,
	pub ffid: u8,
	pub nupvalues: u8,
	pub env: GCRef,
	pub gclist: GCRef,
	pub pc: MRef,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GCfuncC {
	pub header: GCFuncHeader,
	pub c: LuaCFunction,
	pub upvalue: [TValue; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GCfuncL {
	pub header: GCFuncHeader,
	pub uvptr: [GCRef; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GCfunc {
	pub c: GCfuncC,
	pub l: GCfuncL,
}

impl IntoLJType for GCfunc {
	const LJ_TYPE: u32 = LJ_TFUNC;
}

pub const FF_LUA: u8 = 0;
pub const FF_C: u8 = 1;

impl GCfunc {
	pub fn header(&self) -> &GCFuncHeader {
		unsafe { &self.c.header }
	}

	pub fn header_mut(&mut self) -> &mut GCFuncHeader {
		unsafe { &mut self.c.header }
	}

	pub fn as_l(&self) -> Option<&GCfuncL> {
		if !self.is_lua() {
			return None;
		}

		Some(unsafe { &self.l })
	}

	pub fn as_c(&self) -> Option<&GCfuncC> {
		if self.is_lua() {
			return None;
		}

		Some(unsafe { &self.c })
	}

	pub fn as_l_mut(&mut self) -> Option<&mut GCfuncL> {
		if !self.is_lua() {
			return None;
		}

		Some(unsafe { &mut self.l })
	}

	pub fn as_c_mut(&mut self) -> Option<&mut GCfuncC> {
		if self.is_lua() {
			return None;
		}

		Some(unsafe { &mut self.c })
	}

	pub fn is_lua(&self) -> bool {
		let ffid = self.header().ffid;
		ffid == FF_LUA
	}

	pub fn is_c(&self) -> bool {
		let ffid = self.header().ffid;
		ffid == FF_C
	}

	pub fn is_fast_function(&self) -> bool {
		let ffid = self.header().ffid;
		ffid > FF_C
	}
}

#[repr(C)]
pub struct LJState {
	pub header: GCHeader,
	pub dummy_ffid: u8,
	pub status: u8,
	pub glref: MRef,
	pub gclist: GCRef,
	pub base: *mut TValue,
	pub top: *mut TValue,
	pub maxstack: MRef,
	pub stack: MRef,
	pub openupval: GCRef,
	pub env: GCRef,
	pub cframe: *mut c_void,
	pub stacksize: MSize,
}

#[repr(C)]
pub struct Sbuf {
	pub p: MRef,
	pub e: MRef,
	pub b: MRef,
	pub l: MRef,
}

#[repr(C)]
pub struct GCState {
	pub total: GCSize,
	pub threshold: GCSize,
	pub currentwhite: u8,
	pub state: u8,
	pub nocdatafin: u8,
	pub unused2: u8,
	pub sweepstr: MSize,
	pub root: GCRef,
	pub sweep: MRef,
	pub gray: GCRef,
	pub grayagain: GCRef,
	pub weak: GCRef,
	pub mmudata: GCRef,
	pub debt: GCSize,
	pub estimate: GCSize,
	pub stepmul: MSize,
	pub pause: MSize,
}

#[repr(C)]
pub struct GCstr {
	pub header: GCHeader,
	pub udtype: u8,
	pub unused: u8,
	pub env: GCRef,
	pub len: MSize,
	pub metatable: GCRef,
	pub align1: u32,
}

impl IntoLJType for GCstr {
	const LJ_TYPE: u32 = LJ_TSTR;
}

#[repr(C)]
pub struct Node {
	pub val: TValue,
	pub key: TValue,
	pub next: MRef,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GCUpvalUVLink {
	pub next: GCRef,
	pub prev: GCRef,
}

#[repr(C)]
pub union GCUpvalUV {
	pub tv: TValue,
	pub link: GCUpvalUVLink,
}

#[repr(C)]
pub struct GCUpval {
	pub header: GCHeader,
	pub closed: u8,
	pub immutable: u8,
	pub uv: GCUpvalUV,
	pub v: MRef,
	pub dhash: u32,
}

impl IntoLJType for GCUpval {
	const LJ_TYPE: u32 = LJ_TUPVAL;
}

pub type BCIns = u32;

// Metamethod enum
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaMethod {
	Index = 0,
	NewIndex,
	Gc,
	Mode,
	Eq,
	Len,
	// Fast metamethods end here (max 8)
	Lt,
	Le,
	Concat,
	Call,
	// ORDER ARITH
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Pow,
	Unm,
	// Standard library
	Metatable,
	ToString,
}

impl MetaMethod {
	pub const MAX: usize = 19; // Metamethods in GMod, which has neither 5.2 or FFI

	pub const FAST: Self = Self::Len;
}

// GC Root IDs
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GCRootID {
	MMName = 0,
	MMNameLast = MetaMethod::MAX - 1,
	BaseMT = MetaMethod::MAX,
	BaseMTNum = MetaMethod::MAX + 13, // ~LJ_TNUMX
	IOInput = MetaMethod::MAX + 14,
	IOOutput = MetaMethod::MAX + 15,
}

pub const GCROOT_MAX: usize = MetaMethod::MAX + 16;

#[repr(C)]
pub struct GlobalState {
	pub strhash: *mut GCRef,
	pub strmask: MSize,
	pub strnum: MSize,
	pub allocf: LuaAllocFn,
	pub allocd: *mut c_void,
	pub gc: GCState,
	pub vmstate: c_int,
	pub tmpbuf: Sbuf,
	pub strempty: GCstr,
	pub stremptyz: u8,
	pub hookmask: u8,
	pub dispatchmode: u8,
	pub vmevmask: u8,
	pub mainthref: GCRef,
	pub registrytv: TValue,
	pub tmptv: TValue,
	pub tmptv2: TValue,
	pub nilnode: Node,
	pub uvhead: GCUpval,
	pub hookcount: c_int,
	pub hookcstart: c_int,
	pub hookf: LuaHookFn,
	pub wrapf: LuaCFunction,
	pub panic: LuaCFunction,
	pub bc_cfunc_int: BCIns,
	pub bc_cfunc_ext: BCIns,
	pub cur_l: GCRef,
	pub jit_base: MRef,
	pub ctype_state: MRef,
	pub gcroot: [GCRef; GCROOT_MAX],
}
