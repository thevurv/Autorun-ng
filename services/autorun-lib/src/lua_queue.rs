///! The purpose of the Lua Queue is to allow other threads to schedule code to run on the main lua thread.
///! Otherwise, if you try to run lua outside the main thread, you'll most definitely crash the game.
use std::sync::{Arc, LazyLock, Mutex};

use autorun_lua::LuaApi;

type LuaCallback = Box<dyn FnOnce(&LuaApi) -> anyhow::Result<()> + Send + 'static>;

static LUA_QUEUE: LazyLock<Arc<Mutex<Vec<LuaCallback>>>> = LazyLock::new(|| Arc::new(Mutex::new(vec![])));

pub fn push(f: impl FnOnce(&LuaApi) -> anyhow::Result<()> + Send + 'static) {
	let mut queue = LUA_QUEUE.lock().unwrap();
	queue.push(Box::new(f));
}

pub fn pop() -> Option<LuaCallback> {
	let mut queue = LUA_QUEUE.lock().unwrap();
	if queue.is_empty() { None } else { Some(queue.remove(0)) }
}
