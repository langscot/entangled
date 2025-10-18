use anyhow::Error;
use entangled_lib::constants::DEFAULT_PORT;
use entangled_lib::message::Message;
use entangled_lib::protocol::MessageFrame;
use rand::random_range;
use std::thread;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

async fn connect(host: &str, port: u16) -> Result<(), Error> {
    eprintln!("Connecting to {}:{}", host, port);
    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)).await {
        eprintln!("Connected to {}:{}", host, port);

        // Send a simple ping to test
        let encoded = MessageFrame::from_message(Message::Ping).encode();
        stream.write_all(&encoded).await.unwrap();

        loop {}
    } else {
        Err(anyhow::anyhow!("Failed to connect to server"))
    }
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
        if (delay != 0) {
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
