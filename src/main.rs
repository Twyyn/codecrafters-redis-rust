use bytes::Bytes;
use codecrafters_redis::{Client, Frame, Listener, Server, Stream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = Listener::try_new().await?;

    let mut stream = listener.accept().await?;

    let simple_frame = Frame::Simple(Bytes::from_static(
        b"*2\r\n$4\r\nLLEN\r\n$6\r\nmylist\r\n
",
    ));
    stream.write_frame(&simple_frame).await?;
    Ok(())
}
