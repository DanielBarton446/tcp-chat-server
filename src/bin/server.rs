//! To run the server do `cargo run --bin server -- -a <address> -p <port>`
//! Or from the binary `./server -a <address> -p <port>`
use super_mega_chatroom::*;

use clap::Parser;
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

const SERVER_ID: usize = usize::MAX;

#[allow(unused, clippy::empty_loop)]
async fn process(mut stream: TcpStream, id: usize, name: String, tx: Sender) {
    let mut rx = tx.subscribe();
    tx.send(Message {
        name: name.clone(),
        id: SERVER_ID,
        msg: String::from("Connected"),
    });

    loop {
        select!(
            Ok(mut msg) = rx.recv() => {
                let msg_with_name = format!("{}: {}\r\n", msg.name, msg.msg);
                if msg.id != id {
                    write_to_stream(&mut stream, msg_with_name.as_bytes()).await;
                }
            }
            msg = read_from_stream(&mut stream) => {
                match msg {
                    Ok(msg) => {
                        let msg = Message { id, msg, name: name.clone()};
                        if let Err(e) = tx.send(msg.clone()) {
                            eprintln!("Failed to propogate message to other clients: {:?}", msg);
                        }
                    },
                    Err(e) => {
                        tx.send(Message { name: name.clone(), id: SERVER_ID, msg: String::from("Client disconnected") });
                        eprintln!("Client Disconnected: {}", name);
                        break;
                    }
                }
            }
        );
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "localhost")]
    address: String,
    #[arg(short, long, default_value_t = 7878)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listener = match TcpListener::bind((args.address.as_str(), args.port)).await {
        Err(e) => {
            eprintln!(
                "Failed to bind to {}:{}, Error Message: {}",
                args.address, args.port, e
            );
            std::process::exit(1);
        }
        Ok(l) => l,
    };
    eprintln!(
        "Listening for connections on {}:{}",
        args.address, args.port
    );

    let (tx, mut _rx1) = broadcast::channel::<Message>(16);
    let mut id = 0;

    loop {
        let (mut stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to accept client connection: {}", e);
                continue;
            }
        };
        let tx1 = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = write_to_stream(&mut stream, b"What is your name: \n").await {
                eprintln!("Connection to client dropped: {}", e);
                return
            }
            if let Ok(name) = read_from_stream(&mut stream).await {
                eprintln!("Client Connected: {}", name);
                process(stream, id, name, tx1).await;
            } else {
                eprintln!("A client failed to connect");
            }
        });
        id += 1;
    }
}
