use std::{
	io::{BufReader, BufWriter, Read, Write},
	ops::Deref,
};

use interprocess::{
	local_socket::{prelude::*, GenericNamespaced, Listener, ListenerOptions, Stream, ToNsName},
	TryClone,
};
use nanoserde::{DeBin, SerBin};

const SOCKET_NAME: &str = r"/tmp/autorun_ipc";

#[derive(SerBin, DeBin, Debug, Clone)]
pub enum Message {
	Ping,
	Pong,
	Print(String),
	RunCode(String),
	Shutdown,
}

pub struct Messenger {
	reader: BufReader<Stream>,
	writer: BufWriter<Stream>,
}

impl Messenger {
	pub fn new(stream: Stream) -> anyhow::Result<Self> {
		let reader_stream = stream.try_clone()?;
		let reader = BufReader::new(reader_stream);

		let writer = BufWriter::new(stream);

		Ok(Self { writer, reader })
	}

	pub fn receive(&mut self) -> anyhow::Result<Message> {
		let mut length_buf = [0u8; 4];
		self.reader.read_exact(&mut length_buf)?;
		let length = u32::from_le_bytes(length_buf) as usize;

		let mut msg_buf = vec![0u8; length];
		self.reader.read_exact(&mut msg_buf)?;

		let message = Message::deserialize_bin(&msg_buf)?;

		Ok(message)
	}

	pub fn send(&mut self, message: Message) -> anyhow::Result<()> {
		let msg_buf = message.serialize_bin();
		let length = msg_buf.len() as u32;
		let length_buf = length.to_le_bytes();

		self.writer.write_all(&length_buf)?;
		self.writer.write_all(&msg_buf)?;
		self.writer.flush()?;

		Ok(())
	}
}

pub struct Client {
	messenger: Messenger,
}

impl Deref for Client {
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
