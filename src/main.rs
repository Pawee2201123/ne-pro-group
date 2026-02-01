// main.rs
mod state;
mod routes;
mod ws;
mod chat;
mod topic;


use axum::{Router};
use tower_http::services::ServeDir;


#[tokio::main]
async fn main() {
    let (tx, _) = tokio::sync::broadcast::channel(16);
    let (chat_tx, _) = tokio::sync::broadcast::channel(64);

    let state = std::sync::Arc::new(state::AppState {
        game: std::sync::Mutex::new(state::GameState {
            phase: state::Phase::Waiting,
            players: Default::default(),
            max_players: 4,
            max_speak: 3,
            genre: "果物".to_string(),
            wolf_id: None,
            remaining_time: 30,
            voting_time: 10,
            is_villager_win: None,
            game_id: 0,
            executed_id: None, 
        }),
        topic_manager: std::sync::Arc::new(std::sync::Mutex::new(topic::TopicManager::new())),
        tx,
        chat_tx,
        
        inner: state::SharedStateInner {
            connections: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_player_id: std::sync::Mutex::new(0),
        },
    });

    let app = Router::new()
        .merge(chat::router())
        .merge(routes::router())
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);
        

    axum::serve(
        tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap(),
        app,
    )
    
    .await
    .unwrap();
}