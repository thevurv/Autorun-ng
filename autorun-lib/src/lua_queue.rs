use std::sync::{Arc, LazyLock, Mutex};

pub static LUA_QUEUE: LazyLock<Arc<Mutex<Vec<(autorun_types::Realm, std::ffi::CString)>>>> =
	LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));
