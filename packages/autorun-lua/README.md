# autorun-lua

This implements Autorun's lua abstractions and being able to access the lua api from `lua_shared` provided by the game.

## Example

```rust
/// A basic example of creating a binary module using `autorun-lua`
/// Dependencies:
/// libloading
/// anyhow
/// { git = "https://github.com/thevurv/Autorun-ng", package = "autorun-lua" }
use autorun_lua::*;
use std::ffi::c_int;

fn lua_adder(lua: &LuaApi, state: *mut LuaState) -> anyhow::Result<f64> {
	let x = lua.check_number(state, 1);
	let y = lua.check_number(state, 2);

	// This pushes it onto lua's stack for you.
	// You can return multiple values via a tuple of values
	// Additionally, Option<T> values work too, where None pushes nil.
	Ok(x + y)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn gmod13_open(state: *mut LuaState) -> c_int {
	let lua = autorun_lua::get_api().expect("Failed to get lua api");

	lua.push_globals(state); // Push _G

	lua.push(state, "adder");
	lua.push(state, as_lua_function!(lua_adder));
	lua.set_table(state, -3); // _G["adder"] = lua_adder

	0
}
```
