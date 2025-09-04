use interprocess::local_socket::{traits::Stream, GenericNamespaced, ToNsName};

use crate::{messenger::Messenger, SOCKET_NAME};

pub struct Client {
	messenger: Messenger,
}

impl std::ops::Deref for Client {
	type Target = Messenger;

	fn deref(&self) -> &Self::Target {
		&self.messenger
	}
}

impl std::ops::DerefMut for Client {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.messenger
	}
}

impl Client {
	pub fn connect() -> anyhow::Result<Self> {
		let socket_ns_name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
		let stream = Stream::connect(socket_ns_name)?;

		let messenger = Messenger::new(stream)?;
		Ok(Self { messenger })
	}
}
