/// A basic example of creating a binary module using `autorun-lua`
/// Add this to your deps
/// { git = "https://github.com/thevurv/Autorun-ng", package = "autorun-lua" }
use autorun_lua::*;

fn lua_adder(lua: &LuaApi, state: *mut LuaState) -> Result<f64, Box<dyn std::error::Error>> {
	let x = lua.check_number(state, 1);
	let y = lua.check_number(state, 2);

	// This pushes it onto lua's stack for you.
	// You can return multiple values via a tuple of values
	// Additionally, Option<T> values work too, where None pushes nil.
	Ok(x + y)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn gmod13_open(state: *mut LuaState) -> std::ffi::c_int {
	let lua = autorun_lua::get_api().expect("Failed to get lua api");

	// _G.adder = lua_adder
	let globals = lua.globals(state);
	lua.set(state, &globals, "adder", as_lua_function!(lua_adder));

	0
}
