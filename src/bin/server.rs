//! To run the server do `cargo run --bin server`
use super_mega_chatroom::*;

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


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("localhost:7878").await.unwrap();
    let (tx, mut _rx1) = broadcast::channel::<Message>(16);
    let mut id = 0;

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let tx1 = tx.clone();

        tokio::spawn(async move {
            write_to_stream(&mut stream, b"What is your name: \n").await;
            let name = read_from_stream(&mut stream).await;
            dbg!(&name);
            process(stream, id, name, tx1).await;
        });
        id += 1;
    }
}
