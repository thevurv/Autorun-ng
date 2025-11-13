use crate::*;
use anyhow::Context;

pub fn global_state(state: &LJState) -> *mut GlobalState {
	state.glref.as_ptr()
}

pub fn registry(state: &LJState) -> *mut TValue {
	unsafe { &mut (*global_state(state)).registrytv as *mut TValue }
}

pub fn index2adr(state: &LJState, idx: i32) -> Option<*mut TValue> {
	// for now, support only positive indices
	if idx > 0 {
		let tv = unsafe { state.base.add((idx - 1) as usize) };
		if tv < state.top {
			return Some(tv);
		}
	} else if idx < 0 {
		let abs_idx = (-idx) as usize;
		let stack_size = unsafe { state.top.offset_from(state.base) as usize };
		if abs_idx <= stack_size {
			let tv = unsafe { state.top.offset(-(abs_idx as isize)) };
			return Some(tv);
		}
	}

	None
}

pub fn curr_func(state: &LJState) -> anyhow::Result<&GCfunc> {
	// #define curr_func(L)		(&gcval(L->base-2)->fn)

	unsafe {
		let func_tv = state.base.offset(-2);
		let gcfunc = (*func_tv).as_ref::<GCfunc>()?;
		Ok(gcfunc)
	}
}

pub fn get_gcobj<T: Clone + IntoLJType>(state: &LJState, idx: i32) -> anyhow::Result<T> {
	unsafe {
		let tv = index2adr(state, idx).context("Failed to get TValue for given index.")?;
		let gcobj = (*tv).as_ref::<T>()?;
		Ok(gcobj.clone()) // Cloning is fine as we don't intend to modify it
	}
}

pub fn get_gcobj_mut<T: IntoLJType>(state: &mut LJState, idx: i32) -> anyhow::Result<&mut T> {
	unsafe {
		let tv = index2adr(state, idx).context("Failed to get TValue for given index.")?;
		let gcobj = (*tv).as_mut::<T>()?;
		Ok(gcobj)
	}
}

pub fn push_tvalue(state: &mut LJState, tvalue: &TValue) {
	unsafe {
		std::ptr::write(state.top, *tvalue);
		state.top = state.top.add(1);
	}
}

pub fn push_frame_func(state: &mut LJState, frame: &Frame) -> anyhow::Result<()> {
	push_tvalue(state, unsafe { &*frame.get_func_tv() });
	Ok(())
}

// Our number one goal is to avoid having to depend on things like sigscanning to find internal functions.
// Therefore we re-implement the functionality we need here.
/**
/* Allocate new GC object and link it to the root set. */
void * LJ_FASTCALL lj_mem_newgco(lua_State *L, GCSize size)
{
  global_State *g = G(L);
  GCobj *o = (GCobj *)g->allocf(g->allocd, NULL, 0, size);
  if (o == NULL)
	lj_err_mem(L);
  lj_assertG(checkptrGC(o),
		 "allocated memory address %p outside required range", o);
  g->gc.total += size;
  setgcrefr(o->gch.nextgc, g->gc.root);
  setgcref(g->gc.root, o);
  newwhite(g, o);
  return o;
}

#define newwhite(g, x)	(obj2gco(x)->gch.marked = (uint8_t)curwhite(g))
#define curwhite(g)	((g)->gc.currentwhite & LJ_GC_WHITES)
*/

pub fn mem_newgco<T: IntoLJType>(state: &mut LJState, size: GCSize) -> anyhow::Result<*mut T> {
	let global_state = global_state(state);
	let global_state = unsafe { global_state.as_mut().context("Failed to dereference GlobalState")? };
	let allocf = global_state.allocf;
	let allocd = global_state.allocd;
	let obj_ptr = unsafe { allocf(allocd, std::ptr::null_mut(), 0, size as usize) };
	if obj_ptr.is_null() {
		anyhow::bail!("Memory allocation failed in lj_mem_newgco.");
	}

	unsafe {
		global_state.gc.total += size;
		let gc_header_ptr = obj_ptr as *mut GCHeader;
		(*gc_header_ptr).nextgc = global_state.gc.root;
		global_state.gc.root.gcptr64 = obj_ptr as u64;

		// newwhite
		(*gc_header_ptr).marked = (global_state.gc.currentwhite & LJ_GC_WHITES) as u8;
	}

	Ok(obj_ptr as *mut T)
}
