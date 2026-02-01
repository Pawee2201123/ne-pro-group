// 🎓 Module declarations: Tell Rust about our code files
mod types;
mod game;    // Game logic module
mod rooms;   // Rooms module
mod network; // Network layer (HTTP + SSE)

use rooms::RoomManager;
use network::{HttpRequest, route_request};
use std::io::Read;
use std::net::TcpListener;
use std::env;
use std::thread;

fn main() {
    println!("🐺 Word Wolf Server Starting...\n");

    // Get address from command line or use default
    let args: Vec<String> = env::args().collect();
    let address = if args.len() >= 2 {
        &args[1]
    } else {
        "127.0.0.1:8080"
    };

    // Create the room manager (shared across all threads)
    let room_manager = RoomManager::new();

    // Bind TCP listener
    let listener = match TcpListener::bind(address) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", address, e);
            std::process::exit(1);
        }
    };

    println!("✓ Server listening on {}", address);
    println!("✓ Room manager initialized");

    // 🎓 Spawn background timer thread
    // This thread checks all rooms every second for expired discussion timers
    {
        let timer_manager = room_manager.clone();
        thread::spawn(move || {
            use std::time::Duration;
            loop {
                thread::sleep(Duration::from_secs(1));
                timer_manager.check_all_timers();
            }
        });
    }
    println!("✓ Background timer thread started");

    println!("\n📋 Available endpoints:");
    println!("  GET  /                    - Serve login.html");
    println!("  GET  /events?room_id=X    - SSE connection for room X");
    println!("  POST /room/create         - Create a new room");
    println!("  POST /room/join           - Join a room");
    println!("  POST /room/ready          - Mark player as ready");
    println!("  POST /room/vote           - Submit a vote");
    println!("  GET  /room/list           - List all rooms");
    println!("  GET  /room/state?room_id=X - Get room state");
    println!("\n🎮 Server ready for connections!\n");

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Clone RoomManager for this thread (cheap! just Arc clone)
                let manager = room_manager.clone();

                // Spawn a thread to handle this connection
                thread::spawn(move || {
                    // Read the HTTP request
                    let mut buffer = [0u8; 4096];
                    let nbytes = match stream.read(&mut buffer) {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };

                    // Parse HTTP request
                    let request_str = match std::str::from_utf8(&buffer[..nbytes]) {
                        Ok(s) => s,
                        Err(_) => return,
                    };

                    let request = match HttpRequest::parse(request_str) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse request: {}", e);
                            return;
                        }
                    };

                    // Log the request
                    println!("{:?} {} {}",
                             request.method,
                             request.path,
                             request.query_params.iter()
                                 .map(|(k, v)| format!("{}={}", k, v))
                                 .collect::<Vec<_>>()
                                 .join("&"));

                    // Route the request
                    if let Some(response) = route_request(request, stream.try_clone().unwrap(), &manager) {
                        use std::io::Write;
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    // If None, it's an SSE connection that stays open
                });
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}
