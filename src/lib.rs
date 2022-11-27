use tokio::io::*;
use tokio::net::*;

struct Header {
    msg_length: u64,
}

impl Header {
    fn new(msg: &[u8]) -> Self {
        Self {
            msg_length: msg.len() as u64,
        }
    }
    
    async fn read_header(stream: &mut TcpStream) -> Result<Self> {
        Ok(Self {
            msg_length: stream.read_u64().await?,
        })
    }

    async fn write_header(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write_u64(self.msg_length).await
    }

}


pub async fn read_from_stream(stream: &mut TcpStream) -> Result<String> {
    let header = Header::read_header(stream).await?;
    let mut msg_buf = Vec::with_capacity(header.msg_length as usize);

    // SAFETY: Elements must be initializsed by read_exact
    unsafe {
        msg_buf.set_len(header.msg_length as usize);
    }

    stream.read_exact(msg_buf.as_mut_slice()).await?;

    Ok(String::from_utf8_lossy(&msg_buf).to_string())


}

pub async fn write_to_stream<B: AsRef<[u8]>>(stream: &mut TcpStream, msg: B) -> Result<()> {
    let msg = msg.as_ref();
    Header::new(msg).write_header(stream).await?;
    stream.write_all(msg.as_ref()).await
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: usize,
    pub name: String,
    pub msg: String,
}
