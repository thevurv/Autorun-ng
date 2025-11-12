// Subset of lj_obj.h

use crate::bytecode::Op;
use anyhow::Context;
use std::ffi::{c_int, c_void};
use std::fmt::Debug;

// IMPORTANT: GMod's LUA_IDSIZE was randomly changed to 128 instead of 60 like in vanilla LuaJIT
#[cfg(feature = "gmod")]
pub const LUA_IDSIZE: i32 = 128;
#[cfg(not(feature = "gmod"))]
pub const LUA_IDSIZE: i32 = 60;

pub const LJ_FR2: u32 = 1;

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

pub type MSize = u32;
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

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MRef {
	pub ptr64: u64,
}

impl MRef {
	// equivalent to the mref macro in LuaJIT
	pub fn as_ptr<T>(&self) -> *mut T {
		(self.ptr64 & LJ_GCVMASK) as *mut T
	}

	pub fn tvref(&self) -> *mut TValue {
		self.as_ptr::<TValue>()
	}
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GCRef {
	pub gcptr64: u64,
}

impl GCRef {
	pub fn from_ptr<T>(ptr: *mut T) -> Self {
		Self {
			gcptr64: (ptr as u64) & LJ_GCVMASK,
		}
	}

	// equivalent to the gcref macro in LuaJIT
	pub fn as_ptr<T>(&self) -> *mut T {
		self.gcptr64 as *mut T
	}
}

impl PartialEq<Self> for GCRef {
	fn eq(&self, other: &Self) -> bool {
		(self.gcptr64 & LJ_GCVMASK) == (other.gcptr64 & LJ_GCVMASK)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GCHeader {
	pub nextgc: GCRef,
	pub marked: u8,
	pub gct: u8,
}

impl GCHeader {
	pub fn check_type(&self, lj_type: u32) -> bool {
		// GCT is stored as the bitwise NOT of the type
		self.gct as u32 == (!lj_type & 0xFF)
	}
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
	pub ftsz: u64,
}

impl Debug for TValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "TValue {{ itype: {}, gcr: {:x} }}", self.itype(), unsafe {
			self.gcr.gcptr64
		})
	}
}

macro_rules! impl_tvalue_type_check {
	($function_name:ident, $lj_type_const:expr) => {
		pub fn $function_name(&self) -> bool {
			self.itype() == $lj_type_const
		}
	};
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

	impl_tvalue_type_check!(is_nil, LJ_TNIL);
	impl_tvalue_type_check!(is_false, LJ_TFALSE);
	impl_tvalue_type_check!(is_true, LJ_TTRUE);
	impl_tvalue_type_check!(is_lightud, LJ_TLIGHTUD);
	impl_tvalue_type_check!(is_str, LJ_TSTR);
	impl_tvalue_type_check!(is_upval, LJ_TUPVAL);
	impl_tvalue_type_check!(is_thread, LJ_TTHREAD);
	impl_tvalue_type_check!(is_proto, LJ_TPROTO);
	impl_tvalue_type_check!(is_func, LJ_TFUNC);
	impl_tvalue_type_check!(is_trace, LJ_TTRACE);
	impl_tvalue_type_check!(is_cdata, LJ_TCDATA);
	impl_tvalue_type_check!(is_tab, LJ_TTAB);
	impl_tvalue_type_check!(is_udata, LJ_TUDATA);
	impl_tvalue_type_check!(is_numx, LJ_TNUMX);
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GCFuncHeader {
	pub header: GCHeader,
	pub ffid: u8,
	pub nupvalues: u8,
	pub env: GCRef,
	pub gclist: GCRef,
	// Compiler randomly adds 4 bytes of padding here for alignment, not too sure why since it is packed
	pub _pad: [u8; 4],
	pub pc: MRef,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GCfuncC {
	pub header: GCFuncHeader,
	pub c: LuaCFunction,
	pub upvalue: [TValue; 1],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GCfuncL {
	pub header: GCFuncHeader,
	pub uvptr: [GCRef; 1],
}

impl GCfuncL {
	pub fn get_proto(&self) -> anyhow::Result<*mut GCProto> {
		let pc_ref = self.header.pc;
		// proto starts immediately before the pc pointer
		let proto = unsafe { pc_ref.as_ptr::<GCProto>().offset(-1) };
		Ok(proto)
	}

	pub fn get_bc_ins(&self) -> anyhow::Result<*mut BCIns> {
		let pc_ref = self.header.pc;
		Ok(pc_ref.as_ptr::<BCIns>())
	}
}

#[repr(C, packed)]
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

/// NOTE: Incompatibility with LuaJIT 2.1 here.
/// 2.1 has the 'sid' field, while GMod's 2.1.0-beta3 does not.
/// 2.1's hash field is a uint32_t, while GMod's is a MSize.
#[repr(C, packed)]
pub struct GCstr {
	pub header: GCHeader,
	pub reserved: u8,
	pub unused: u8,
	pub hash: MSize,
	pub len: MSize,
	pub _padding: u32, // The two bytes (reserved + unused) causes major misalignment, so we need padding here
}

impl GCstr {
	fn data(&self) -> *const u8 {
		// payload is stored immediately after the GCstr struct
		unsafe { (self as *const GCstr).add(1) as *const u8 }
	}

	fn as_bytes(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.data(), self.len as usize) }
	}

	pub fn as_str(&self) -> anyhow::Result<&str> {
		if self.len == 0 || self.data().is_null() {
			return Ok("");
		}

		let bytes = self.as_bytes();
		let s = std::str::from_utf8(bytes).context("GCstr contains invalid UTF-8 data")?;
		Ok(s)
	}
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

#[derive(Clone, Copy)]
pub struct BCIns(u32);

impl BCIns {
	// instruction consisting of an opcode, 8-bit A and 16-bit D fields
	pub fn from_ad(opcode: Op, a: u8, d: i16) -> Self {
		Self((opcode as u32) | ((a as u32) << 8) | (((d as i32 as u32) & 0xFFFF) << 16))
	}

	pub fn from_abc(opcode: Op, a: u8, b: u8, c: u8) -> Self {
		Self((opcode as u32) | ((a as u32) << 8) | ((c as u32) << 16) | ((b as u32) << 24))
	}

	pub fn opcode(&self) -> Op {
		//#define bc_op(i)		((BCOp)((i)&0xff))
		Op::try_from((self.0 & 0xff) as u8).unwrap()
	}

	/*
		#define bc_a(i)		((BCReg)(((i)>>8)&0xff))
	#define bc_b(i)		((BCReg)((i)>>24))
	#define bc_c(i)		((BCReg)(((i)>>16)&0xff))
	#define bc_d(i)		((BCReg)((i)>>16))
		 */
	pub fn a(&self) -> u8 {
		//#define bc_a(i)		((BCReg)(((i)>>8)&0xff))
		((self.0 >> 8) & 0xff) as u8
	}

	pub fn b(&self) -> u8 {
		//#define bc_b(i)		((BCReg)((i)>>24))
		((self.0 >> 24) & 0xff) as u8
	}

	pub fn c(&self) -> u8 {
		//#define bc_c(i)		((BCReg)(((i)>>16)&0xff))
		((self.0 >> 16) & 0xff) as u8
	}

	pub fn d(&self) -> i16 {
		//#define bc_d(i)		((BCReg)((i)>>16))
		((self.0 >> 16) & 0xffff) as i16
	}
}

impl Debug for BCIns {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"BCIns {{ opcode: {:?}, a: {}, b: {}, c: {}, d: {}, raw: 0x{:08x} }}",
			self.opcode(),
			self.a(),
			self.b(),
			self.c(),
			self.d(),
			self.0
		)
	}
}

pub type BCLine = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GCProto {
	header: GCHeader,
	pub numparams: u8,
	pub framesize: u8,
	pub sizebc: MSize,
	pub unused: u32,
	pub gclist: GCRef,
	pub k: MRef,
	pub uv: MRef,
	pub sizekgc: MSize,
	pub sizekn: MSize,
	pub sizept: MSize,
	pub sizeuv: u8,
	pub flags: u8,
	pub trace: u16,
	pub chunkname: GCRef,
	pub firstline: BCLine,
	pub numline: BCLine,
	pub lineinfo: MRef,
	pub uvinfo: MRef,
	pub varinfo: MRef,
	// padding for alignment
	pub _pad: [u8; 4],
}

impl GCProto {
	pub fn chunk_name_str(&self) -> anyhow::Result<&str> {
		let chunk_name = unsafe {
			self.chunkname
				.as_ptr::<GCstr>()
				.as_ref()
				.context("Failed to dereference chunk name GCstr")?
		};

		chunk_name.as_str()
	}
}

impl IntoLJType for GCProto {
	const LJ_TYPE: u32 = LJ_TPROTO;
}

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
