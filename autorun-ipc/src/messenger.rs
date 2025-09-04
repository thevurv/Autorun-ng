//! This is a generic interface between the Server and Client so they can both read and write.
use std::io::{BufReader, BufWriter, Read, Write};

use interprocess::{local_socket::Stream, TryClone};
use nanoserde::{DeBin, SerBin};

use crate::message::Message;

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
