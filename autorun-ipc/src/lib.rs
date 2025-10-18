mod client;
mod message;
mod messenger;
mod server;

pub use client::Client;
pub use message::Message;
pub use messenger::Messenger;
pub use server::Server;

#[cfg(target_os = "linux")]
pub const SOCKET_NAME: &str = r"/tmp/autorun_ipc";

#[cfg(target_os = "windows")]
pub const SOCKET_NAME: &str = r"\\.\pipe\autorun_ipc";
