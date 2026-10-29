use bytes::{Buf, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Argument {
    pub position: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct ParsedFrame {
    source: Bytes,
    argv: Vec<Argument>,
}

#[derive(Debug)]
pub struct Parser {
    source: BytesMut,
}

impl Parser {
    pub fn new(source: BytesMut) -> Self {
        Self { source }
    }
}
