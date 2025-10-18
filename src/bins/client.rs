use anyhow::Error;
use bytes::{Buf, BytesMut};
use entangled_lib::constants::DEFAULT_PORT;
use entangled_lib::message::Message;
use entangled_lib::protocol::MessageFrame;
use rand::random_range;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedSender};

async fn connect(host: &str, port: u16) -> Result<(), Error> {
    eprintln!("Connecting to {}:{}", host, port);
    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)).await {
        eprintln!("Connected to {}:{}", host, port);

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        tokio::spawn(async move {
            handle_collectors(tx).await;
        });

        let mut buffer = BytesMut::with_capacity(4096);

        loop {
            tokio::select! {
                // Send
                Some(message) = rx.recv() => {
                    let encoded = MessageFrame::from_message(message).encode();
                    if let Err(e) = stream.write_all(&encoded).await {
                        eprintln!("Error sending message to server: {}", e);
                    }
                }
                // Read
                result = stream.read_buf(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            eprintln!("Server closed connection");
                            break;
                        }
                        Ok(_) => {
                            while let Some((frame, consumed)) = MessageFrame::parse(&buffer) {
                                eprintln!("Received frame: {:?}", frame.payload);
                                buffer.advance(consumed);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error occured whilst reading bytes from server: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Disconnected"))
    } else {
        Err(anyhow::anyhow!("Failed to connect to server"))
    }
}

/// Handle the collectors
async fn handle_collectors(tx: UnboundedSender<Message>) {
    let _ = tx.send(Message::Ping);
}

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let host: String = args
        .get(1)
        .and_then(|host| host.parse().ok())
        .unwrap_or("0.0.0.0".parse().unwrap());

    let port: u16 = args
        .get(2)
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // Client behaviour is to always try connect to the server.
    let mut delay: u64 = 0;
    loop {
        // Jitter to prevent thundering herd
        if delay != 0 {
            eprintln!("Waiting {} before connecting again", delay);
            thread::sleep(Duration::from_secs(delay));
        }
        if let Err(e) = connect(&host, port).await {
            eprintln!("Error: {}", e);
            delay = random_range(0..=60);
            continue;
        }
    }
}
