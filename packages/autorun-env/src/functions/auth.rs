pub mod hooks;

use anyhow::Context;
use autorun_lua::{DebugInfo, LUA_MULTRET, LuaApi, LuaTypeId, RawLuaReturn};
use autorun_luajit::{Frame, FrameType, GCfunc, LJState, get_gcobj, push_frame_func, push_tvalue};
use autorun_types::LuaState;

pub const ERROR_FFI_ID: u8 = 19;

pub fn is_function_authorized(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	if !matches!(lua.type_id(state, 1), LuaTypeId::Function | LuaTypeId::Number) {
		anyhow::bail!("First argument must be a function or stack level.");
	}

	if lua.type_id(state, 1) == LuaTypeId::Number {
		// attempt to resolve the function at the given stack level
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };
		let stack_level = lua.to::<i32>(state, 1);
		lua.pop(state, 1); // remove the stack level argument

		if lua.get_stack(state, stack_level, &raw mut debug_info as _) == 0 {
			anyhow::bail!("Invalid stack level provided.");
		}

		let lj_state = state as *mut LJState;
		let lj_state = unsafe { lj_state.as_mut() }.context("Failed to dereference LJState")?;
		let frame_func = Frame::from_debug_ci(lj_state, debug_info.i_ci).get_func_tv();

		// copy the function to the top of the stack
		unsafe {
			push_tvalue(lj_state, &*frame_func);
		}
	}

	env.is_function_authorized(lua, state, None)
		.context("Failed to check function authorization.")
}

/// Like pcall, but spoofs the frames so that Autorun is no where to be seen in the call stack.
pub fn safe_call(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	let nargs = lua.get_top(state);
	if nargs < 1 {
		anyhow::bail!("At least one argument (the function to call) is required.");
	}

	if lua.type_id(state, 1) != LuaTypeId::Function {
		anyhow::bail!("First argument must be a function to call.");
	}

	let is_error_fn = unsafe {
		let lj_state = state as *mut LJState;
		let lj_state = lj_state.as_ref().context("Failed to dereference LJState")?;
		let gcfunc = get_gcobj::<GCfunc>(lj_state, 1)?;

		gcfunc.is_fast_function() && gcfunc.header().ffid == ERROR_FFI_ID
	};

	let nargs = lua.get_top(state) - 1; // exclude the function itself

	let frames = Frame::walk_stack(state as *mut LJState);
	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState")? };

	let mut autorun_frames: Vec<Frame> = frames
		.into_iter()
		.enumerate()
		.filter(|(index, frame)| {
			if *index == 0 && frame.is_lua_frame() {
				// Typically our closure wrapper frame, although it is not necessarily marked as a C frame.
				// Its some tail call magic, but we always want to remove it. It does leak in debug.traceback,
				// for example.

				return true;
			}

			if *index == 0 && frame.is_c_frame() {
				let gc_func = match frame.get_gc_func() {
					Ok(func) => func,
					Err(_) => return false,
				};

				if gc_func.is_c() {
					let cfunc = gc_func.as_c().unwrap();
					let func_ptr = cfunc.c as usize;

					// check if it's safe_call, although we need to push safe call since its not directly accessible here
					env.push_autorun_table(lua, state);
					lua.get_field(state, -1, c"safeCall".as_ptr());
					let safe_call_ptr = lua.to_function(state, -1).unwrap() as usize;
					lua.pop(state, 2); // pop both the function and the env table

					return func_ptr == safe_call_ptr;
				}
			}

			// Push frame's function onto the stack
			let tv = frame.get_func_tv();

			unsafe {
				if !(*tv).is_func() {
					false
				} else {
					push_frame_func(lj_state, frame).expect("Failed to push frame function onto stack");
					// ask env if this function is authorized
					let authorized = env.is_function_authorized(lua, state, None).unwrap_or(false);
					// pop the function off the stack
					lua.pop(state, 1);

					authorized
				}
			}
		})
		.map(|(_index, frame)| frame)
		.collect();

	// mark each autorun frame as a dummy
	for frame in autorun_frames.iter_mut() {
		frame.mark_as_dummy_frame(state as *mut LJState);
	}

	let potential_level = if is_error_fn && lua.type_id(state, 3) == LuaTypeId::Number {
		// get the level from the stack
		let mut level = lua.to::<i32>(state, 3); // first arg is func, second is message, third is level
		level -= 1; // adjust for closure wrapper

		// replace it on the stack with 1, since we've removed our frames
		lua.push(state, level);
		lua.replace(state, 3);
		Some(level)
	} else {
		None
	};

	let result = lua.pcall_forward(state, nargs, LUA_MULTRET, 0);
	if result.is_err() {
		// before we forward this error, check if it's from an error ff, and if so,
		// pass the level as well.
		return lua.error(state, potential_level, false);
	}

	// restore the frames
	for frame in autorun_frames.iter_mut() {
		frame.restore_from_dummy_frame();
	}

	let nresults = lua.get_top(state); // number of results on the stack
	Ok(RawLuaReturn(nresults))
}

pub fn is_proto_authorized(_lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	// Protos dont play nice with the usual public API types, so we just have to do it manually
	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_ref().context("Failed to dereference LJState")? };
	let proto_tv = index2adr(lj_state, 1).context("Failed to get TValue for given index.")?;
	let proto_tv = unsafe { &*proto_tv };

	// TODO: When the stack spoof PR is merged, replace this with the new type check helper
	if proto_tv.itype() != LJ_TPROTO {
		anyhow::bail!("First argument must be a proto.");
	}

	let proto_gc = get_gcobj::<GCProto>(lj_state, 1).context("Failed to get GCProto for given index.")?;
	let proto_chunk_name = proto_gc.chunk_name_str().context("Failed to get chunk name from proto.")?;
	let proto_chunk_name_cstr =
		std::ffi::CString::new(proto_chunk_name.clone()).context("Failed to convert chunk name to CString.")?;

	Ok(env.is_chunk_name_authorized(&proto_chunk_name_cstr))
}
