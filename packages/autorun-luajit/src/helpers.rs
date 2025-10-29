use crate::*;
use anyhow::Context;

pub fn global_state(L: &lua_State) -> *mut global_State {
	L.glref.as_ptr()
}

pub fn registry(L: &lua_State) -> *mut TValue {
	unsafe { &mut (*global_state(L)).registrytv as *mut TValue }
}

pub fn index2adr(L: &lua_State, idx: i32) -> Option<*mut TValue> {
	// for now, support only positive indices
	if idx > 0 {
		let tv = unsafe { L.base.add((idx - 1) as usize) };
		if tv < L.top {
			return Some(tv);
		}
	} else if idx < 0 {
		let abs_idx = (-idx) as usize;
		let stack_size = unsafe { L.top.offset_from(L.base) as usize };
		if abs_idx <= stack_size {
			let tv = unsafe { L.top.offset(-(abs_idx as isize)) };
			return Some(tv);
		}
	}

	None
}

pub fn curr_func(L: &lua_State) -> &GCfunc {
	// #define curr_func(L)		(&gcval(L->base-2)->fn)

	unsafe {
		let func_tv = L.base.offset(-2);
		let gcfunc = (*func_tv).as_ref::<GCfunc>();
		gcfunc
	}
}

pub fn get_gcobj<T: Clone>(L: &lua_State, idx: i32) -> anyhow::Result<T> {
	unsafe {
		let tv = index2adr(L, idx).context("Failed to get TValue for given index.")?;
		let gcobj = (*tv).as_ref::<T>();
		Ok(gcobj.clone()) // Cloning is fine as we don't intend to modify it
	}
}

pub fn get_gcobj_mut<T>(L: &lua_State, idx: i32) -> anyhow::Result<&mut T> {
	unsafe {
		let tv = index2adr(L, idx).context("Failed to get TValue for given index.")?;
		let gcobj = (*tv).as_mut::<T>();
		Ok(gcobj)
	}
}
