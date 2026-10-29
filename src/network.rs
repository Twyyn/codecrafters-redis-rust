use std::{collections::HashMap, io::Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

#[derive(Debug, Default)]
pub struct Server {
    clients: HashMap<usize, Sender<u8>>,
    threads: Vec<JoinHandle<Result<()>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            threads: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Client {
    stream: Stream,
}

impl Client {
    pub fn spawn_writer(mut self, mut rx: Receiver<u8>) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            while let Some(byte) = rx.recv().await {
                self.stream.write(&[byte]).await?;
            }

            Ok(())
        })
    }
}

#[derive(Debug)]
pub struct Listener {
    inner: TcpListener,
}

impl Listener {
    pub async fn try_new() -> Result<Self> {
        const LISTEN_ADDR: &str = "127.0.0.1:6379";

        let inner = TcpListener::bind(LISTEN_ADDR).await?;

        Ok(Self { inner })
    }
}

#[derive(Debug)]
pub struct Stream {
    socket: TcpStream,
}

impl Stream {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.socket.read(buf).await
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.socket.write_all(buf).await
    }
}
