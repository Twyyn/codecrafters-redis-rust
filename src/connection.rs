use bytes::BytesMut;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    buf: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: BytesMut::default(),
        }
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write_all(bytes)
    }

    pub fn fill_buf(&mut self) -> io::Result<usize> {
        let buf = &mut self.buf;
        let original_len = buf.len();

        buf.resize(original_len + 4096, 0);

        let result = self.stream.read(&mut buf[original_len..]);
        buf.truncate(original_len + result.as_ref().map_or(0, |&n| n));

        result
    }
}
