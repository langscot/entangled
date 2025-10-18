use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AuthRequest {
    device_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    AuthRequest,
    Ping,
    Pong,
}
