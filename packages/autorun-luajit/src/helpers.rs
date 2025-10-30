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
