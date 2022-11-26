use tokio::io::*;
use tokio::net::*;

pub async fn read_from_stream(stream: &mut TcpStream) -> Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let b = stream.read_u8().await?;
        if b == b'\n' {
            break;
        }
        buf.push(b);
    }
    let s: String = buf.into_iter().map(|b| b as char).collect();
    Ok(s.trim().to_string())
}

pub async fn write_to_stream<B: AsRef<[u8]>>(stream: &mut TcpStream, msg: B) -> Result<()> {
    stream
        .write_all(msg.as_ref())
        .await
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: usize,
    pub name: String,
    pub msg: String,
}
