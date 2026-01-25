use std::net::{TcpListener, TcpStream};
use std::env;
#[macro_use]
extern crate log;
use std::{thread, sync::{Arc, Mutex, mpsc}}; 
use core::str;
use std::io::{Read, Write};
use std::fs::File;

// クライアントにメッセージを送るための送信元のリスト
type Senders = Arc<Mutex<Vec<mpsc::Sender<String>>>>;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("Please enter [addr:port]");
        std::process::exit(1);
    }

    // SSE接続中のクライアントリスト
    let senders: Senders = Arc::new(Mutex::new(Vec::new()));

    let _address: &str = &args[1];
    let listener = TcpListener::bind(_address).unwrap();
    info!("Server listening on {}", _address);

    loop {
        let (stream, _) = listener.accept().unwrap();
        let senders_clone = Arc::clone(&senders);

        thread::spawn(move || {
            handler(stream, senders_clone).unwrap_or_else(|error| error!("{:?}", error));
        });
    }
}

fn handler(mut stream: TcpStream, senders: Senders) -> Result<(), failure::Error> {
    let mut buffer = [0u8; 1024];
    let nbytes = stream.read(&mut buffer)?;
    if nbytes == 0 { return Ok(()); }
    let request = str::from_utf8(&buffer[..nbytes])?;

    if request.contains("GET /events") {
        // --- SSE 接続の開始 ---
        let (tx, rx) = mpsc::channel();
        {
            senders.lock().unwrap().push(tx);
        }

        let header = "HTTP/1.1 200 OK\r\n\
                      Content-Type: text/event-stream\r\n\
                      Cache-Control: no-cache\r\n\
                      Connection: keep-alive\r\n\
                      Access-Control-Allow-Origin: *\r\n\r\n";
        stream.write_all(header.as_bytes())?;

        // チャンネルからメッセージが来るのを待機し、ストリームに流し続ける
        while let Ok(msg) = rx.recv() {
            // SSEのフォーマットは "data: メッセージ\n\n"
            let fmt_msg = format!("data: {}\n\n", msg);
            if let Err(_) = stream.write_all(fmt_msg.as_bytes()) {
                break; // クライアントが切断したらループを抜ける
            }
            stream.flush()?;
        }
        debug!("SSE Connection closed.");

    } else if request.contains("POST /send") {
        // --- メッセージの送信 (ブロードキャスト) ---
        let body = request.split("\r\n\r\n").last().unwrap_or("");
        if !body.is_empty() {
            let mut s = senders.lock().unwrap();
            // 生きている全クライアントへ送信（切断済みは削除）
            s.retain(|tx| tx.send(body.to_string()).is_ok());
        }
        send_response(&mut stream, "OK", "text/plain")?;

    } else {
        // index.html の提供
        let path = format!("{}/webroot/index.html", env::current_dir()?.display());
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        send_response(&mut stream, &contents, "text/html")?;
    }
    Ok(())
}

fn send_response(stream: &mut TcpStream, content: &str, content_type: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {};charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        content_type, content.len(), content
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}