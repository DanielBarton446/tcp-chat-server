use tokio::io::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast;

/**
*
* Next Steps:
* [x] read from streams and echo back
* [] Try to add multithreading with spawn -- this is going to fail at first
* [] fix the multithreading issue with arc<mutex<something>>
*
*/

#[derive(Clone, Debug)]
struct Message {
    id: usize,
    name: String,
    msg: String,
}

type Sender = broadcast::Sender<Message>;

#[allow(unused, clippy::empty_loop)]
async fn process(mut stream: TcpStream, id: usize, name: String, tx: Sender) {
    let mut rx = tx.subscribe();

    loop {
        select!(
            Ok(mut msg) = rx.recv() => {
                let msg_with_name = format!("{}: {}\r\n", msg.name, msg.msg);
                if msg.id != id {
                    write_to_stream(&mut stream, msg_with_name.as_bytes()).await;
                }
            }
            msg = read_from_stream(&mut stream) => {
                tx.send(Message { id, msg, name: name.clone()}).unwrap();
            }
        );
    }
}

async fn read_from_stream(stream: &mut TcpStream) -> String {
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

async fn write_to_stream(stream: &mut TcpStream, msg: &[u8]) {
    stream
        .write_all(msg)
        .await
        .expect("failed to write git gud");
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    let (tx, mut _rx1) = broadcast::channel::<Message>(16);
    let mut id = 0;

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let tx1 = tx.clone();

        tokio::spawn(async move {
            write_to_stream(&mut stream, b"What is your name: ").await;
            let name = read_from_stream(&mut stream).await;
            process(stream, id, name, tx1).await;
        });
        id += 1;
    }
}
