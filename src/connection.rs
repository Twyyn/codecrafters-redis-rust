use bytes::BytesMut;
use std::io;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
        }
    }
}

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
}

impl Server {
    pub async fn new() -> io::Result<Self> {
        const DEFAULT_ADDR: &str = "127.0.0.1:6379";

        let listener = TcpListener::bind(DEFAULT_ADDR).await?;

        Ok(Self {
            listener,
            clients: Vec::new(),
        })
    }
}
