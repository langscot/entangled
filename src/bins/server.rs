use std::collections::HashMap;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use entangled_lib::{constants::DEFAULT_PORT, message::Message, protocol::MessageFrame};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, mpsc},
};
use uuid::Uuid;

struct ClientSession {
    device_id: Option<String>,
    tx: mpsc::UnboundedSender<Message>,
}

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let port: u16 = args
        .get(1)
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    eprintln!("Server started on port {}", port);

    // Track clients
    let clients: Arc<Mutex<HashMap<Uuid, ClientSession>>> = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((socket, _)) = listener.accept().await {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();

        let client_session = ClientSession {
            device_id: None, // none until authenticate
            tx: tx,
        };

        let client_id = Uuid::new_v4();

        {
            let mut lock = clients.lock().await;
            lock.insert(client_id, client_session);
        }

        let client_arc = clients.clone();

        tokio::spawn(async move {
            handle_client(client_id, socket, rx, client_arc).await;
        });
    }
}

/// Handles client connections
async fn handle_client(
    client_id: Uuid,
    mut socket: TcpStream,
    mut rx: mpsc::UnboundedReceiver<Message>,
    clients: Arc<Mutex<HashMap<Uuid, ClientSession>>>,
) {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        tokio::select! {
            // Forward messages onto client
            Some(message) = rx.recv() => {
                eprintln!("Message received");
                let encoded = MessageFrame::from_message(message).encode();
                if let Err(e) = socket.write_all(&encoded).await {
                    eprintln!("Error sending message to client: {}", e);
                }
            }
            result = socket.read_buf(&mut buffer) => {
                match result {
                    // Client stopped sending, connection closed
                    Ok(0) => {
                        eprintln!("Client closed connection");
                        break;
                    },
                    // Client sent n bytes
                    Ok(_) => {
                        while let Some((frame, consumed)) = MessageFrame::parse(&buffer) {
                            eprintln!("Recieved frame: {:?}", frame.payload);
                            let clients_arc = clients.clone();
                            tokio::spawn(async move {
                                handle_frame(client_id, frame, clients_arc).await;
                            });
                            buffer.advance(consumed);
                        }
                    },
                    // Error occured
                    Err(e) => {
                        eprintln!("Error occured whilst reading bytes from client: {}", e);
                        break;
                    }
                }
            }
        }
    }

    clients.lock().await.remove(&client_id);
}

async fn handle_frame(
    client_id: Uuid,
    frame: MessageFrame,
    clients: Arc<Mutex<HashMap<Uuid, ClientSession>>>,
) {
    match frame.payload {
        Message::Ping => {
            let _ = clients
                .lock()
                .await
                .get(&client_id)
                .unwrap()
                .tx
                .send(Message::Pong);
        }
        _ => {}
    }
}
