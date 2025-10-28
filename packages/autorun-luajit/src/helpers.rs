use crate::*;

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
	}

	None
}
