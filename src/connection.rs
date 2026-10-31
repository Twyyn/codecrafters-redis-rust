use crate::resp::{Frame, ParseError, Parser, SpannedFrame};

use std::{collections::HashMap, io};

use bytes::BytesMut;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

#[derive(Debug)]
pub struct Listener {
    inner: TcpListener,
}

impl Listener {
    pub async fn try_new() -> Result<Self, ConnectionError> {
        const LISTEN_ADDR: &str = "127.0.0.1:6379";

        let inner = TcpListener::bind(LISTEN_ADDR).await?;

        Ok(Self { inner })
    }

    pub async fn accept(&self) -> Result<Stream, ConnectionError> {
        let (socket, _) = self.inner.accept().await?;

        Ok(Stream::new(socket))
    }
}

#[derive(Debug)]
pub struct Stream {
    socket: TcpStream,
    parser: Parser,
    buf: BytesMut,
}

impl Stream {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            parser: Parser::new(BytesMut::new()),
            buf: BytesMut::new(),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<SpannedFrame>, ConnectionError> {
        loop {
            if let Some(frame) = self.parser.parse()? {
                return Ok(Some(frame));
            }

            let capacity = self.parser.remaining_capacity();

            if capacity == 0 {
                return Err(ParseError::BufferLimitExceeded.into());
            }

            let bytes_read = (&mut self.socket)
                .take(capacity as u64)
                .read_buf(self.parser.buf_mut())
                .await?;

            if bytes_read == 0 {
                if self.parser.is_empty() {
                    return Ok(None);
                }

                return Err(ConnectionError::Disconnect);
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), ConnectionError> {
        self.buf.clear();
        frame.encode_inner(&mut self.buf);
        self.socket.write_all(&self.buf).await?;

        Ok(())
    }

    pub async fn recv(&mut self) -> io::Result<usize> {
        self.socket.read(self.parser.buf_mut()).await
    }
}

#[derive(Debug)]
pub struct Client {
    stream: Stream,
}

impl Client {
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }

    pub fn spawn_writer(
        mut self,
        mut rx: Receiver<Frame>,
    ) -> JoinHandle<Result<(), ConnectionError>> {
        tokio::spawn(async move {
            while let Some(frame) = rx.recv().await {
                self.stream.write_frame(&frame).await?;
            }

            Ok(())
        })
    }

    pub fn spawn_reader(
        mut self,
        tx: Sender<SpannedFrame>,
    ) -> JoinHandle<Result<(), ConnectionError>> {
        tokio::spawn(async move {
            while let Some(frame) = self.stream.read_frame().await? {
                tx.send(frame)
                    .await
                    .map_err(|_| ConnectionError::Disconnect)?;
            }

            Ok(())
        })
    }
}

#[derive(Debug, Default)]
pub struct Server {
    clients: Vec<Sender<u8>>,
    handles: Vec<JoinHandle<Result<(), ConnectionError>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            handles: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("connection closed by peer")]
    Disconnect,
}
