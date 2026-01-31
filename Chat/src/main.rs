mod game;

use std::net::{TcpListener, TcpStream};
use std::env;
#[macro_use]
extern crate log;
use std::{thread, sync::{Arc, Mutex, mpsc}};
use core::str;
use std::io::{Read, Write};
use std::fs::File;
use std::collections::HashMap;

type Senders = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>;
type GameManager = Arc<Mutex<game::WordWolfGame>>;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    let senders: Senders = Arc::new(Mutex::new(HashMap::new()));
    let game_manager: GameManager = Arc::new(Mutex::new(game::WordWolfGame::new()));

    let listener = TcpListener::bind(&addr).expect("Failed to bind address");
    info!("Server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let senders_clone = Arc::clone(&senders);
                let game_clone = Arc::clone(&game_manager);
                thread::spawn(move || {
                    if let Err(e) = handler(stream, senders_clone, game_clone) {
                        error!("Error handling client: {:?}", e);
                    }
                });
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
}

fn handler(mut stream: TcpStream, senders: Senders, game_manager: GameManager) -> Result<(), failure::Error> {
    let mut buffer = [0u8; 1024];
    let nbytes = stream.read(&mut buffer)?;
    if nbytes == 0 { return Ok(()); }
    
    let request_str = str::from_utf8(&buffer[..nbytes])?;
    
    // リクエストライン（1行目）を取得
    let request_line = request_str.lines().next().unwrap_or("");
    
    if request_line.starts_with("GET /events") {
        // --- SSE 接続 ---
        // URLからIDを抽出
        let client_id = if let Some(idx) = request_line.find("?id=") {
            let part = &request_line[idx+4..];
            // 空白（HTTPバージョンとの区切り）までを取得
            part.split_whitespace().next().unwrap_or("unknown").to_string()
        } else {
            // IDがない場合はIPで代用
            stream.peer_addr()?.to_string()
        };

        let (tx, rx) = mpsc::channel();
        {
            let mut s = senders.lock().unwrap();
            s.insert(client_id.clone(), tx);
            info!("Client connected with ID: {}", client_id);
        }

        let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
        stream.write_all(header.as_bytes())?;

        //接続直後にウェルカムメッセージを送信する
        let welcome_msg = "【システム】ようこそ！\n「start 参加人数」を入力してゲームを開始してください。（例：start 3）";
        let mut formatted_welcome = String::new();
        for line in welcome_msg.lines() {
            formatted_welcome.push_str(&format!("data: {}\n", line));
        }
        formatted_welcome.push_str("\n");
        stream.write_all(formatted_welcome.as_bytes())?;
        stream.flush()?;
        //受信ループ
        while let Ok(msg) = rx.recv() {
            let mut formatted_msg = String::new();
            for line in msg.lines() {
                formatted_msg.push_str(&format!("data: {}\n", line));
            }
            formatted_msg.push_str("\n"); 

            if stream.write_all(formatted_msg.as_bytes()).is_err() {
                break;
            }
            stream.flush()?;
        }
        
        {
            let mut s = senders.lock().unwrap();
            s.remove(&client_id);
        }
        debug!("Client disconnected: {}", client_id);

    } else if request_line.starts_with("POST /send") {
        // --- メッセージ送信 ---
        // ボディを取得
        let body_full = request_str.split("\r\n\r\n").last().unwrap_or("").trim();
        
        if !body_full.is_empty() {
            // 1行目がID、2行目以降が本文というルールにする
            let mut lines = body_full.lines();
            let client_id = lines.next().unwrap_or("unknown").trim().to_string();
            let message_body = lines.collect::<Vec<&str>>().join("\n"); // 残りを結合

            if !client_id.is_empty() && !message_body.is_empty() {
                let mut game = game_manager.lock().unwrap();
                let mut s = senders.lock().unwrap();

                if message_body.starts_with("start") {
                    let parts: Vec<&str> = message_body.split_whitespace().collect();
                    let num = if parts.len() > 1 {
                        parts[1].parse::<usize>().unwrap_or(2)
                    } else {
                        2
                    };
                    let msg = game.start_theme_phase(num);
                    broadcast(&mut s, &msg);

                } else if game.phase == game::GamePhase::ThemeSelection {
                    // キーワード入力
                    let old_count = game.submitted_keywords.len();
                    // IDを使ってキーワード登録
                    let reply_msg = game.submit_keyword(client_id.clone(), message_body.clone());
                    let new_count = game.submitted_keywords.len();

                    if new_count > old_count {
                        broadcast(&mut s, &format!("誰かがキーワードを入力しました（現在 {}/{} 人）", new_count, game.expected_players));

                        if game.check_ready_to_distribute() {
                            game.distribute_roles();
                            for (pid, sender) in s.iter() {
                                if let Some(secret_msg) = game.get_secret_message(pid) {
                                    let _ = sender.send(secret_msg);
                                }
                            }
                        }
                    } else {
                        // エラー時は本人だけに通知
                        if let Some(tx) = s.get(&client_id) {
                            let _ = tx.send(reply_msg);
                        }
                    }

                } else {
                    // 通常チャット
                    let chat_msg = format!("User: {}", message_body);
                    broadcast(&mut s, &chat_msg);
                }
            }
        }
        send_response(&mut stream, "OK", "text/plain")?;

    } else {
        let path = format!("{}/webroot/index.html", env::current_dir()?.display());
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            send_response(&mut stream, &contents, "text/html")?;
        } else {
            send_response(&mut stream, "404 Not Found", "text/plain")?;
        }
    }
    Ok(())
}

fn broadcast(senders: &mut HashMap<String, mpsc::Sender<String>>, msg: &str) {
    for sender in senders.values() {
        let _ = sender.send(msg.to_string());
    }
}

fn send_response(stream: &mut TcpStream, content: &str, content_type: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {};charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        content_type, content.len(), content
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}