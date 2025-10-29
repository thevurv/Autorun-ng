// Subset of lj_obj.h

// IMPORTANT: GMod's LUA_IDSIZE was randomly changed to 128 instead of 60 like in vanilla LuaJIT
pub const LUA_IDSIZE: i32 = 128;
pub type MSize = u64;
pub type GCSize = u64;

#[repr(C)]
pub struct lua_Debug {
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

pub type lua_Hook = extern "C-unwind" fn(L: *mut lua_State, ar: *mut lua_Debug);
pub type lua_CFunction = extern "C-unwind" fn(L: *mut lua_State) -> i32;
pub type lua_Alloc = extern "C-unwind" fn(
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

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GCHeader {
	pub nextgc: GCRef,
	pub marked: u8,
	pub gct: u8,
}

// #define LJ_GCVMASK		(((uint64_t)1 << 47) - 1)
pub const LJ_GCVMASK: u64 = ((1u64 << 47) - 1);

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub union TValue {
	pub u64: u64,
	pub f64: f64,
	pub gcr: GCRef,
	pub it64: i64,
}

impl TValue {
	pub fn as_ptr<T>(&self) -> *mut T {
		dbg!("TValue::as_ptr called");
		unsafe {
			dbg!(self.u64);
			dbg!(self.gcr.gcptr64);
			dbg!(self.itype());
		}

		unsafe { (self.gcr.gcptr64 & LJ_GCVMASK) as *mut T }
	}

	pub fn as_ref<T>(&self) -> &T {
		unsafe { &*self.as_ptr::<T>() }
	}

	pub fn as_mut<T>(&mut self) -> &mut T {
		unsafe { &mut *self.as_ptr::<T>() }
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
	pub c: lua_CFunction,
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

pub const FF_LUA: u8 = 0;
pub const FF_C: u8 = 1;

impl GCfunc {
	pub fn as_l(&self) -> &GCfuncL {
		unsafe { &self.l }
	}

	pub fn as_c(&self) -> &GCfuncC {
		unsafe { &self.c }
	}

	pub fn as_l_mut(&mut self) -> &mut GCfuncL {
		unsafe { &mut self.l }
	}

	pub fn as_c_mut(&mut self) -> &mut GCfuncC {
		unsafe { &mut self.c }
	}

	pub fn is_lua(&self) -> bool {
		let ffid = self.as_c().header.ffid;
		ffid == FF_LUA
	}

	pub fn is_c(&self) -> bool {
		let ffid = self.as_c().header.ffid;
		ffid == FF_C
	}

	pub fn is_fast_function(&self) -> bool {
		let ffid = self.as_c().header.ffid;
		ffid > FF_C
	}
}

/*
/* Per-thread state object. */
struct lua_State {
  GCHeader;
  uint8_t dummy_ffid;	/* Fake FF_C for curr_funcisL() on dummy frames. */
  uint8_t status;	/* Thread status. */
  MRef glref;		/* Link to global state. */
  GCRef gclist;		/* GC chain. */
  TValue *base;		/* Base of currently executing function. */
  TValue *top;		/* First free slot in the stack. */
  MRef maxstack;	/* Last free slot in the stack. */
  MRef stack;		/* Stack base. */
  GCRef openupval;	/* List of open upvalues in the stack. */
  GCRef env;		/* Thread environment (table of globals). */
  void *cframe;		/* End of C stack frame chain. */
  MSize stacksize;	/* True stack size (incl. LJ_STACK_EXTRA). */
};
 */

#[repr(C)]
pub struct lua_State {
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
	pub cframe: *mut core::ffi::c_void,
	pub stacksize: MSize,
}

#[repr(C)]
pub struct Sbuf {
	pub p: MRef,
	pub e: MRef,
	pub b: MRef,
	pub L: MRef,
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

#[repr(C)]
pub struct Node {
	pub val: TValue,
	pub key: TValue,
	pub next: MRef,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GCupval_uv_link {
	pub next: GCRef,
	pub prev: GCRef,
}

#[repr(C)]
pub union GCupval_uv {
	pub tv: TValue,
	pub link: GCupval_uv_link,
}

#[repr(C)]
pub struct GCupval {
	pub header: GCHeader,
	pub closed: u8,
	pub immutable: u8,
	pub uv: GCupval_uv,
	pub v: MRef,
	pub dhash: u32,
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

/*
/* Global state, shared by all threads of a Lua universe. */
typedef struct global_State {
  GCRef *strhash;	/* String hash table (hash chain anchors). */
  MSize strmask;	/* String hash mask (size of hash table - 1). */
  MSize strnum;		/* Number of strings in hash table. */
  lua_Alloc allocf;	/* Memory allocator. */
  void *allocd;		/* Memory allocator data. */
  GCState gc;		/* Garbage collector. */
  volatile int32_t vmstate;  /* VM state or current JIT code trace number. */
  SBuf tmpbuf;		/* Temporary string buffer. */
  GCstr strempty;	/* Empty string. */
  uint8_t stremptyz;	/* Zero terminator of empty string. */
  uint8_t hookmask;	/* Hook mask. */
  uint8_t dispatchmode;	/* Dispatch mode. */
  uint8_t vmevmask;	/* VM event mask. */
  GCRef mainthref;	/* Link to main thread. */
  TValue registrytv;	/* Anchor for registry. */
  TValue tmptv, tmptv2;	/* Temporary TValues. */
  Node nilnode;		/* Fallback 1-element hash part (nil key and value). */
  GCupval uvhead;	/* Head of double-linked list of all open upvalues. */
  int32_t hookcount;	/* Instruction hook countdown. */
  int32_t hookcstart;	/* Start count for instruction hook counter. */
  lua_Hook hookf;	/* Hook function. */
  lua_CFunction wrapf;	/* Wrapper for C function calls. */
  lua_CFunction panic;	/* Called as a last resort for errors. */
  BCIns bc_cfunc_int;	/* Bytecode for internal C function calls. */
  BCIns bc_cfunc_ext;	/* Bytecode for external C function calls. */
  GCRef cur_L;		/* Currently executing lua_State. */
  MRef jit_base;	/* Current JIT code L->base or NULL. */
  MRef ctype_state;	/* Pointer to C type state. */
  GCRef gcroot[GCROOT_MAX];  /* GC roots. */
} global_State;
 */

#[repr(C)]
pub struct global_State {
	pub strhash: *mut GCRef,
	pub strmask: MSize,
	pub strnum: MSize,
	pub allocf: lua_Alloc,
	pub allocd: *mut core::ffi::c_void,
	pub gc: GCState,
	pub vmstate: core::ffi::c_int,
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
	pub uvhead: GCupval,
	pub hookcount: core::ffi::c_int,
	pub hookcstart: core::ffi::c_int,
	pub hookf: lua_Hook,
	pub wrapf: lua_CFunction,
	pub panic: lua_CFunction,
	pub bc_cfunc_int: BCIns,
	pub bc_cfunc_ext: BCIns,
	pub cur_L: GCRef,
	pub jit_base: MRef,
	pub ctype_state: MRef,
	pub gcroot: [GCRef; GCROOT_MAX],
}
