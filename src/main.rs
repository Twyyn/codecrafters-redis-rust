use codecrafters_redis::Connection;
use std::io;
use std::net::{TcpListener, TcpStream};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        if let Err(err) = stream.and_then(handle_connection) {
            eprintln!("{err}");
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    println!("accepted new connection");

    let mut connection = Connection::new(stream);

    if connection.fill_buf()? > 0 {
        connection.send(b"+PONG\r\n")?;
    }

    Ok(())
}
