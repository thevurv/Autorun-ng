use std::sync::{Arc, LazyLock, Mutex};

use autorun_types::Realm;

type LuaExecutionBundle = (Realm, String);

pub static LUA_QUEUE: LazyLock<Arc<Mutex<Vec<LuaExecutionBundle>>>> = LazyLock::new(|| Arc::new(Mutex::new(vec![])));
