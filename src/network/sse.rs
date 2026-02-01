// network/sse.rs - Server-Sent Events (SSE) handling
//
// 🎓 Key Concepts:
// - SSE is a one-way communication channel (server → client)
// - The connection stays open, server pushes updates
// - Format: "data: message\n\n"

use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;

/// Handle an SSE connection
///
/// 🎓 This function:
/// 1. Sends the SSE header to establish the connection
/// 2. Returns an mpsc::Sender that can be used to send messages
/// 3. Spawns a thread that listens for messages and writes to the stream
///
/// The pattern: "Give me a sender, I'll handle the connection"
pub fn handle_sse_connection(mut stream: TcpStream) -> mpsc::Sender<String> {
    // Create a channel for sending messages to this client
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a thread to handle this SSE connection
    std::thread::spawn(move || {
        // Send SSE header
        let header = crate::network::http::HttpResponse::sse_header();
        if stream.write_all(header.as_bytes()).is_err() {
            return;
        }
        if stream.flush().is_err() {
            return;
        }

        // Keep receiving messages and sending them to the client
        while let Ok(message) = rx.recv() {
            // SSE format: "data: message\n\n"
            let sse_message = format!("data: {}\n\n", message);

            // Write to stream
            if stream.write_all(sse_message.as_bytes()).is_err() {
                break; // Client disconnected
            }

            if stream.flush().is_err() {
                break;
            }
        }

        // Connection closed
    });

    // Return the sender so the caller can send messages to this client
    tx
}

/// Format a message as JSON-like string
/// (In a real app, use serde_json)
pub fn format_event(event_type: &str, data: &str) -> String {
    format!("{{\"type\":\"{}\",\"data\":\"{}\"}}", event_type, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_event() {
        let event = format_event("player_joined", "Alice");
        assert!(event.contains("player_joined"));
        assert!(event.contains("Alice"));
    }
}
