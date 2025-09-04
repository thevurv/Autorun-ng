use nanoserde::{DeBin, SerBin};

#[derive(SerBin, DeBin, Debug, Clone)]
pub enum Message {
	Ping,
	Pong,
	Print(String),
	RunCode(autorun_types::Realm, String),
	Shutdown,
}
