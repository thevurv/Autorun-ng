use interprocess::local_socket::{prelude::*, GenericNamespaced, Listener, ListenerOptions, ToNsName};

use crate::{messenger::Messenger, SOCKET_NAME};

pub struct Server {
	listener: Listener,
}

impl Server {
	pub fn start() -> anyhow::Result<Self> {
		let socket_ns_name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
		let opts = ListenerOptions::new().name(socket_ns_name);
		let listener = opts.create_sync()?;

		Ok(Self { listener })
	}

	pub fn accept(&self) -> anyhow::Result<Messenger> {
		let stream = self.listener.accept()?;
		Messenger::new(stream)
	}
}
