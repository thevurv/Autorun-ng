mod raw;
pub use raw::*;

mod result;
pub use result::*;

mod returns;
pub use returns::*;

use crate::{IntoLua, LuaFunction, LuaTable, LuaValue, types::LuaState};
use std::ffi::{CStr, c_int};

#[derive(Debug)]
pub struct LuaApi {
	pub raw: RawLuaApi,
}

#[derive(Debug)]
pub struct DebugInfo {
	pub event: c_int,
	pub name: Option<String>,           // n
	pub namewhat: Option<String>,       // n
	pub what: Option<String>,           // S
	pub source: Option<String>,         // S
	pub currentline: Option<c_int>,     // l
	pub nups: Option<c_int>,            // u
	pub linedefined: Option<c_int>,     // S
	pub lastlinedefined: Option<c_int>, // S
	pub short_src: Option<String>,      // S
	pub func: Option<LuaFunction>,      // f
}

impl LuaApi {
	pub fn new(lib: &libloading::Library) -> Result<Self, libloading::Error> {
		let raw = RawLuaApi::new(lib)?;
		Ok(Self { raw })
	}

	pub fn load(&self, state: *mut LuaState, src: impl AsRef<[u8]>, name: &CStr) -> LuaResult<LuaFunction> {
		let src = src.as_ref();
		self.raw.loadbufferx(state, src, name, c"t")?;
		let func = self.raw.try_to(state, -1)?;
		self.raw.pop(state, -1);

		Ok(func)
	}

	pub fn setfenv(&self, state: *mut LuaState, f: &LuaFunction, env: &LuaTable) -> LuaResult<()> {
		self.raw.push(state, f);
		self.raw.push(state, env);
		self.raw.setfenv(state, -2)?;
		self.raw.pop(state, 1);

		Ok(())
	}

	pub fn getfenv(&self, state: *mut LuaState, f: &LuaFunction) -> LuaResult<Option<LuaTable>> {
		self.raw.push(state, f);
		self.raw.getfenv(state, -1);
		let env: Option<LuaTable> = self.raw.try_to(state, -1)?;
		self.raw.pop(state, 2);

		Ok(env)
	}

	pub fn getregistry(&self, state: *mut LuaState, key: impl IntoLua) -> LuaValue<'_> {
		key.into_lua(&self.raw, state);
		self.raw.rawget(state, REGISTRY_INDEX);
		let value = self.raw.to(state, -1);
		self.raw.pop(state, 1);
		value
	}

	pub fn setregistry(&self, state: *mut LuaState, key: impl IntoLua, value: impl IntoLua) {
		key.into_lua(&self.raw, state);
		value.into_lua(&self.raw, state);
		self.raw.rawset(state, REGISTRY_INDEX);
	}

	pub fn error(&self, state: *mut LuaState, msg: impl IntoLua) -> ! {
		self.raw.push(state, msg);
		self.raw.error(state);
	}

	pub fn equal(&self, state: *mut LuaState, a: impl IntoLua, b: impl IntoLua) -> bool {
		a.into_lua(&self.raw, state);
		b.into_lua(&self.raw, state);
		let result = self.raw.equal(state, -2, -1);
		self.raw.pop(state, 2);
		result
	}

	pub fn rawequal(&self, state: *mut LuaState, a: impl IntoLua, b: impl IntoLua) -> bool {
		a.into_lua(&self.raw, state);
		b.into_lua(&self.raw, state);
		let result = self.raw.rawequal(state, -2, -1);
		self.raw.pop(state, 2);
		result
	}

	pub fn getinfo(&self, state: *mut LuaState, level: c_int, select: &CStr) -> LuaResult<DebugInfo> {
		let info = self.raw.getinfo(state, level, select).ok_or(LuaError::GenericFailure)?;
		let select = select.to_bytes();

		let mut name = None;
		let mut namewhat = None;
		let mut what = None;
		let mut source = None;
		let mut currentline = None;
		let mut nups = None;
		let mut linedefined = None;
		let mut lastlinedefined = None;
		let mut short_src = None;
		let mut func = None;

		if select.contains(&b'f') {
			let f: LuaFunction = self.raw.try_to(state, -1)?;
			self.raw.pop(state, 1);
			func = Some(f);
		}

		if select.contains(&b'n') {
			name = Some(unsafe { CStr::from_ptr(info.name) }.to_string_lossy().to_string());
			namewhat = Some(unsafe { CStr::from_ptr(info.namewhat) }.to_string_lossy().to_string());
		}

		if select.contains(&b'S') {
			what = Some(unsafe { CStr::from_ptr(info.what) }.to_string_lossy().to_string());
			source = Some(unsafe { CStr::from_ptr(info.source) }.to_string_lossy().to_string());
			linedefined = Some(info.linedefined);
			lastlinedefined = Some(info.lastlinedefined);
			short_src = Some(
				unsafe { CStr::from_ptr(info.short_src.as_ptr()) }
					.to_string_lossy()
					.to_string(),
			);
		}

		if select.contains(&b'l') {
			currentline = Some(info.currentline);
		}

		if select.contains(&b'u') {
			nups = Some(info.nups);
		}

		Ok(DebugInfo {
			event: info.event,
			name,
			namewhat,
			what,
			source,
			currentline,
			nups,
			linedefined,
			lastlinedefined,
			short_src,
			func,
		})
	}
}

#[macro_export]
macro_rules! as_lua_function {
	($func:expr) => {{
		extern "C-unwind" fn lua_wrapper(state: *mut $crate::LuaState) -> i32 {
			let lua = autorun_lua::get_api().expect("Failed to get Lua API");
			match $func(lua, state) {
				Ok(ret) => $crate::LuaReturn::into_lua_return(ret, &lua.raw, state),
				Err(e) => {
					lua.error(state, e.to_string());
				}
			}
		}
		lua_wrapper as $crate::LuaCFunction
	}};
}
