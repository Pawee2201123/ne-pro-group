// network/handlers.rs - HTTP request handlers
//
// 🎓 Key Concepts:
// - Each handler takes HttpRequest and returns HttpResponse
// - Handlers interact with RoomManager
// - This is the "glue" between HTTP and our game logic

use crate::network::http::{HttpRequest, HttpResponse, Method};
use crate::network::sse;
use crate::rooms::RoomManager;
use crate::types::{RoomConfig, ThemeGenre};
use crate::game::Player;
use std::net::TcpStream;

/// Simple URL decoder for handling form-urlencoded data
/// Handles both ASCII (%20 for space) and UTF-8 (%E3%81%82 for Japanese)
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                // Get next two hex digits
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    // Collect bytes for UTF-8 decoding
                    let mut bytes = vec![byte];

                    // Check if this is a multi-byte UTF-8 sequence
                    let extra_bytes = if byte >= 0xF0 {
                        3 // 4-byte UTF-8
                    } else if byte >= 0xE0 {
                        2 // 3-byte UTF-8 (Japanese typically uses this)
                    } else if byte >= 0xC0 {
                        1 // 2-byte UTF-8
                    } else {
                        0 // 1-byte (ASCII)
                    };

                    // Collect additional bytes
                    for _ in 0..extra_bytes {
                        if let Some('%') = chars.next() {
                            let hex: String = chars.by_ref().take(2).collect();
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                bytes.push(byte);
                            }
                        }
                    }

                    // Convert bytes to UTF-8 string
                    if let Ok(s) = String::from_utf8(bytes) {
                        result.push_str(&s);
                    } else {
                        result.push('?'); // Invalid UTF-8
                    }
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            }
            _ => result.push(c),
        }
    }

    result
}

/// Route incoming requests to the appropriate handler
pub fn route_request(
    req: HttpRequest,
    stream: TcpStream,
    room_manager: &RoomManager,
) -> Option<String> {
    // Handle CORS preflight
    if req.method == Method::OPTIONS {
        return Some(HttpResponse::cors_preflight());
    }

    match (req.method, req.path.as_str()) {
        // SSE connection
        (Method::GET, "/events") => {
            handle_sse(req, stream, room_manager);
            None // Connection stays open, no response needed
        }

        // Room operations
        (Method::POST, "/room/create") => Some(handle_create_room(req, room_manager)),
        (Method::POST, "/room/join") => Some(handle_join_room(req, room_manager)),
        (Method::POST, "/room/ready") => Some(handle_mark_ready(req, room_manager)),
        (Method::POST, "/room/start-vote") => Some(handle_start_voting(req, room_manager)),
        (Method::POST, "/room/vote") => Some(handle_vote(req, room_manager)),
        (Method::POST, "/room/theme/confirm") => Some(handle_confirm_theme(req, room_manager)),
        (Method::POST, "/room/chat") => Some(handle_chat_message(req, room_manager)),
        (Method::GET, "/room/list") => Some(handle_list_rooms(room_manager)),
        (Method::GET, "/room/state") => Some(handle_room_state(req, room_manager)),
        (Method::GET, "/room/players") => Some(handle_get_players(req, room_manager)),
        (Method::GET, "/room/timer") => Some(handle_get_timer(req, room_manager)),
        (Method::GET, "/player/theme") => Some(handle_get_player_theme(req, room_manager)),

        // Static file serving (simplified - just return index.html content)
        (Method::GET, "/") => Some(serve_static_file("login.html")),
        (Method::GET, path) if path.ends_with(".html") => {
            Some(serve_static_file(&path[1..])) // Remove leading /
        }

        // 404
        _ => Some(HttpResponse::not_found()),
    }
}

/// Handle SSE connection for a room
fn handle_sse(req: HttpRequest, stream: TcpStream, room_manager: &RoomManager) {
    let room_id = match req.query("room_id") {
        Some(id) => id.clone(),
        None => return,
    };

    // Create SSE connection
    let sender = sse::handle_sse_connection(stream);

    // Add sender to the room
    // 🎓 We use with_room because we need to modify the room
    let _ = room_manager.with_room(&room_id, |room| {
        room.add_sender(sender);
        Ok(())
    });
}

/// Create a new room
fn handle_create_room(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse request body (simplified - in real app use JSON)
    // Expected format: "room_id=abc&room_name=Test&max_players=5&wolf_count=1&genre=Food"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            map.insert(key, value);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&"").to_string();
    let room_name = map.get("room_name").unwrap_or(&"Unnamed").to_string();
    let max_players: usize = map
        .get("max_players")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let wolf_count: usize = map
        .get("wolf_count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let discussion_time: u64 = map
        .get("discussion_time")
        .and_then(|s| s.parse().ok())
        .unwrap_or(180); // Default 3 minutes

    let genre = match *map.get("genre").unwrap_or(&"Food") {
        "Food" => ThemeGenre::Food,
        "Animal" => ThemeGenre::Animal,
        "Place" => ThemeGenre::Place,
        "Object" => ThemeGenre::Object,
        _ => ThemeGenre::Food,
    };

    let config = RoomConfig::new(room_name, max_players, wolf_count, genre, discussion_time);

    match room_manager.create_room(room_id.clone(), config) {
        Ok(_) => HttpResponse::ok(&format!("{{\"room_id\":\"{}\"}}", room_id), "application/json"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Join a room
fn handle_join_room(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc&player_id=p1&player_name=Alice"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            // URL decode values (important for Japanese names!)
            let decoded = url_decode(value);
            map.insert(key, decoded);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&String::new()).clone();
    let player_id = map.get("player_id").unwrap_or(&String::new()).clone();
    let player_name = map.get("player_name").unwrap_or(&"Unknown".to_string()).clone();

    let player = Player::new(player_id, player_name);

    match room_manager.with_room(&room_id, |room| room.add_player(player)) {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Mark player as ready
fn handle_mark_ready(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc&player_id=p1"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            map.insert(key, value);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&"").to_string();
    let player_id = map.get("player_id").unwrap_or(&"").to_string();

    match room_manager.with_room(&room_id, |room| room.mark_ready(&player_id)) {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Start the voting phase
fn handle_start_voting(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            map.insert(key, value);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&"").to_string();

    match room_manager.with_room(&room_id, |room| room.start_voting()) {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Submit a vote
fn handle_vote(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc&voter_id=p1&target_id=p2"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            map.insert(key, value);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&"").to_string();
    let voter_id = map.get("voter_id").unwrap_or(&"").to_string();
    let target_id = map.get("target_id").unwrap_or(&"").to_string();

    match room_manager.with_room(&room_id, |room| {
        room.submit_vote(&voter_id, &target_id)
    }) {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// List all rooms
fn handle_list_rooms(room_manager: &RoomManager) -> String {
    let rooms = room_manager.list_rooms();
    let json = format!("{{\"rooms\":{:?}}}", rooms);
    HttpResponse::ok(&json, "application/json")
}

/// Get room state
fn handle_room_state(req: HttpRequest, room_manager: &RoomManager) -> String {
    let room_id = match req.query("room_id") {
        Some(id) => id,
        None => return HttpResponse::bad_request("Missing room_id"),
    };

    match room_manager.get_room_state(room_id) {
        Some(state) => HttpResponse::ok(&state, "application/json"),
        None => HttpResponse::not_found(),
    }
}

/// Confirm player has seen their theme
fn handle_confirm_theme(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc&player_id=p1"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            map.insert(key, value);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&"").to_string();
    let player_id = map.get("player_id").unwrap_or(&"").to_string();

    match room_manager.with_room(&room_id, |room| room.confirm_theme(&player_id)) {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Handle chat message during discussion
fn handle_chat_message(req: HttpRequest, room_manager: &RoomManager) -> String {
    // Parse: "room_id=abc&player_id=p1&player_name=Alice&message=hello"
    let params: Vec<&str> = req.body.split('&').collect();
    let mut map = std::collections::HashMap::new();

    for param in params {
        if let Some((key, value)) = param.split_once('=') {
            // Properly URL decode the value (handles Japanese + special chars)
            let decoded = url_decode(value);
            map.insert(key, decoded);
        }
    }

    let room_id = map.get("room_id").unwrap_or(&String::new()).clone();
    let player_name = map.get("player_name").unwrap_or(&String::new()).clone();
    let message = map.get("message").unwrap_or(&String::new()).clone();

    if message.is_empty() {
        return HttpResponse::bad_request("Empty message");
    }

    // Broadcast the chat message via room's SSE
    let result = room_manager.with_room(&room_id, |room| {
        room.send_chat_message(&player_name, &message);
        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::ok("OK", "text/plain"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Get discussion timer status for a room
fn handle_get_timer(req: HttpRequest, room_manager: &RoomManager) -> String {
    let room_id = match req.query("room_id") {
        Some(id) => id,
        None => return HttpResponse::bad_request("Missing room_id"),
    };

    let result = room_manager.with_room(room_id, |room| {
        match room.get_remaining_time() {
            Some(seconds) => Ok(format!("{{\"remaining\":{}}}", seconds)),
            None => Ok("{\"remaining\":null}".to_string()),
        }
    });

    match result {
        Ok(json) => HttpResponse::ok(&json, "application/json"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Get all players in a room
fn handle_get_players(req: HttpRequest, room_manager: &RoomManager) -> String {
    let room_id = match req.query("room_id") {
        Some(id) => id,
        None => return HttpResponse::bad_request("Missing room_id"),
    };

    // Get player list from room
    let result = room_manager.with_room(room_id, |room| {
        let players = room.players();

        // Build JSON array manually (in production use serde_json)
        let player_list: Vec<String> = players
            .iter()
            .map(|(id, player)| {
                // Only expose non-sensitive info (id, name, alive status)
                // Don't expose role or theme!
                let is_alive = if player.is_active() { "true" } else { "false" };
                format!(
                    "{{\"id\":\"{}\",\"name\":\"{}\",\"alive\":{}}}",
                    id, player.name(), is_alive
                )
            })
            .collect();

        Ok(format!("[{}]", player_list.join(",")))
    });

    match result {
        Ok(json) => HttpResponse::ok(&json, "application/json"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Get a player's assigned theme
fn handle_get_player_theme(req: HttpRequest, room_manager: &RoomManager) -> String {
    let room_id = match req.query("room_id") {
        Some(id) => id,
        None => return HttpResponse::bad_request("Missing room_id"),
    };

    let player_id = match req.query("player_id") {
        Some(id) => id,
        None => return HttpResponse::bad_request("Missing player_id"),
    };

    // Get player info from room
    let result = room_manager.with_room(room_id, |room| {
        // Find the player
        let player = room.players().get(player_id)
            .ok_or("Player not found")?;

        // Get their theme
        let theme = player.theme()
            .ok_or("Theme not assigned yet")?;

        // Get their role
        let role = if player.is_wolf() { "Wolf" } else { "Citizen" };

        // Return as JSON-like string
        Ok(format!("{{\"theme\":\"{}\",\"role\":\"{}\"}}", theme, role))
    });

    match result {
        Ok(json) => HttpResponse::ok(&json, "application/json"),
        Err(e) => HttpResponse::bad_request(&e),
    }
}

/// Serve static HTML files
fn serve_static_file(filename: &str) -> String {
    use std::fs;

    // Try to read the file
    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => return HttpResponse::not_found(),
    };

    HttpResponse::ok(&content, "text/html; charset=utf-8")
}
