use tokio::net::*;
use tokio::io::*;

pub async fn read_from_stream(stream: &mut TcpStream) -> String {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let b = stream.read_u8().await.unwrap();
        if b == b'\n' {
            break;
        }
        buf.push(b);
    }
    let s: String = buf.into_iter().map(|b| b as char).collect();
    s.trim().to_string()
}

pub async fn write_to_stream<B: AsRef<[u8]>>(stream: &mut TcpStream, msg: B) {
    stream
        .write_all(msg.as_ref())
        .await
        .expect("failed to write git gud");
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: usize,
    pub name: String,
    pub msg: String,
}
