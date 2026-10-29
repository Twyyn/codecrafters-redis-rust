#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let listener = TcpListener::bind(LISTEN_ADDR)?;

    // for incoming in listener.incoming() {
    //     let stream = match incoming {
    //         Ok(stream) => stream,
    //         Err(err) => {
    //             eprintln!("{err}");
    //             continue;
    //         }
    //     };

    //     if let Err(err) = handle_connection(stream) {
    //         eprintln!("{err}");
    //     }
    // }

    Ok(())
}

// fn handle_connection(stream: TcpStream) -> io::Result<()> {
//     println!("accepted new connection");

//     let mut connection = Connection::new(stream);

//     if connection.read_chunk()? > 0 {
//         connection.write_all(b"+PONG\r\n")?;
//     }

//     Ok(())
// }
