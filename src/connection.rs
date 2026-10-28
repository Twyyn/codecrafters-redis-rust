use bytes::BytesMut;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

const READ_CHUNK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::default(),
        }
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write_all(bytes)
    }

    pub fn read_chunk(&mut self) -> io::Result<usize> {
        let buffer = &mut self.buffer;
        let original_length = buffer.len();

        buffer.resize(original_length + READ_CHUNK_SIZE, 0);

        loop {
            match self.stream.read(&mut buffer[original_length..]) {
                Ok(bytes_read) => {
                    buffer.truncate(original_length + bytes_read);
                    return Ok(bytes_read);
                }
                Err(err) if matches!(err.kind(), io::ErrorKind::Interrupted) => {}
                Err(err) => {
                    buffer.truncate(original_length);
                    return Err(err);
                }
            }
        }
    }
}
