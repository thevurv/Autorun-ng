use anyhow::Context;
use autorun_lua::{DebugInfo, LUA_MULTRET, LuaApi, RawLuaReturn};
use autorun_luajit::{Frame, LJState, push_tvalue};
use autorun_types::LuaState;

pub fn is_function_authorized(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<bool> {
	if !matches!(
		lua.type_id(state, 1),
		autorun_lua::LuaTypeId::Function | autorun_lua::LuaTypeId::Number
	) {
		anyhow::bail!("First argument must be a function or stack level.");
	}

	let frames = Frame::walk_stack(state as *mut LJState);
	dbg!(frames);
	if lua.type_id(state, 1) == autorun_lua::LuaTypeId::Number {
		// attempt to resolve the function at the given stack level
		let mut debug_info = unsafe { std::mem::zeroed::<DebugInfo>() };
		let stack_level = lua.to::<i32>(state, 1);
		lua.pop(state, 1); // remove the stack level argument

		if lua.get_stack(state, stack_level, &raw mut debug_info as _) == 0 {
			anyhow::bail!("Invalid stack level provided.");
		}

		let lj_state = state as *mut LJState;
		let frame_func = Frame::from_debug_ci(lj_state, debug_info.i_ci).get_func_tv();

		// copy the function to the top of the stack
		unsafe {
			let state = lj_state.as_mut().context("Failed to dereference LJState")?;
			push_tvalue(state, &*frame_func);
		}
	}

	Ok(env
		.is_function_authorized(lua, state, None)
		.context("Failed to check function authorization.")?)
}

/// Like pcall, but spoofs the frames so that Autorun is no where to be seen in the call stack.
pub fn safe_call(lua: &LuaApi, state: *mut LuaState, env: crate::EnvHandle) -> anyhow::Result<RawLuaReturn> {
	let nargs = lua.get_top(state);
	if nargs < 1 {
		anyhow::bail!("At least one argument (the function to call) is required.");
	}

	if lua.type_id(state, 1) != autorun_lua::LuaTypeId::Function {
		anyhow::bail!("First argument must be a function to call.");
	}

	let nargs = lua.get_top(state) - 1; // exclude the function itself

	let frames = Frame::walk_stack(state as *mut LJState);
	dbg!(&frames);

	let lj_state = state as *mut LJState;
	let lj_state = unsafe { lj_state.as_mut().context("Failed to dereference LJState")? };

	let mut autorun_frames: Vec<Frame> = frames
		.into_iter()
		.enumerate()
		.filter(|(index, frame)| {
			if *index == 0 {
				return true; // always mark the current frame, cause we dont want it to show up
			}

			// Push frame's function onto the stack
			let tv = frame.get_func_tv();
			unsafe {
				if !(*tv).is_func() {
					false
				} else {
					push_tvalue(lj_state, &*tv);
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

	// push the first frame too, because thats *this* frame and we also dont want it to show up
	dbg!(&autorun_frames);

	// mark each autorun frame as a dummy
	for frame in autorun_frames.iter_mut() {
		dbg!("Marking frame as dummy: {:?}", &frame);
		frame.mark_as_dummy_frame(state as *mut LJState);
	}

	let result = lua.pcall(state, nargs, LUA_MULTRET, 0);

	// restore the frames
	for frame in autorun_frames.iter_mut() {
		dbg!("Restoring frame from dummy: {:?}", &frame);
		frame.restore_from_dummy_frame();
	}

	if let Err(err) = result {
		// Push the error message onto the stack
		lua.push(state, format!("Error during safe_call: {}", err));
		return Ok(RawLuaReturn(1)); // Return 1 result (the error message)
	}

	let nresults = lua.get_top(state); // number of results on the stack
	dbg!("safe_call succeeded");
	dbg!(&nresults);

	Ok(RawLuaReturn(nresults))
}
