use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    constants::{MAGIC_NUMBER, PROTOCOL_VERSION},
    message::Message,
};

/// Message frame is composed as below
/// [1 byte - version][1 byte - magic number][4 bytes - length][... payload]
pub struct MessageFrame {
    version: u8,
    pub payload: Message,
}

impl MessageFrame {
    /// Attempt to parse a full MessageFrame from bytes
    /// If incomplete, return None
    /// If successfully parsed, return MessageFrame and remaining bytes
    pub fn parse(buffer: &BytesMut) -> Option<(Self, usize)> {
        // Need to at least read the frame header first
        if buffer.len() < 6 {
            return None;
        }

        let version = buffer[0];
        let magic = buffer[1];
        if magic != MAGIC_NUMBER {
            eprintln!("Invalid magic number: {}", magic);
            return None;
        }

        // Get length of payload
        let len = u32::from_be_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]) as usize;
        if buffer.len() < 6 + len {
            return None; // incomplete payload
        }

        // Grab the full payload & deserialize
        let payload_bytes = &buffer[6..6 + len];
        let payload: Message = serde_json::from_slice(&payload_bytes).ok()?;

        Some((
            MessageFrame { version, payload },
            6 + len, // bytes consumed
        ))
    }

    /// Convert a Message into a MessageFrame
    pub fn from_message(message: Message) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            payload: message,
        }
    }

    /// Encode a message frame into bytes to be sent on the wire
    pub fn encode(self) -> Bytes {
        let payload_bytes = serde_json::to_vec(&self.payload).expect("Failed to serialize message");
        let mut buf = BytesMut::with_capacity(2 + 4 + payload_bytes.len());
        buf.put_u8(self.version);
        buf.put_u8(MAGIC_NUMBER);
        buf.put_u32(payload_bytes.len() as u32);
        buf.extend_from_slice(&payload_bytes);
        buf.freeze()
    }
}
