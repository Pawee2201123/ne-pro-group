//chat.rs

use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
    response::sse::{Sse, Event},
};
use futures_util::stream::Stream;
use tokio::sync::broadcast::error::RecvError;
use crate::state::SharedState;

#[derive(serde::Deserialize)]
pub struct ChatMessage {
    pub msg: String,
}

/// ルーター生成
pub fn router() -> Router<SharedState> {
    Router::new()
    
        .route("/chat/events", get(chat_sse))
        .route("/chat/send", post(chat_send))
}

/// SSEでクライアントにチャットを配信
pub async fn chat_sse(
    State(state): State<SharedState>
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = state.chat_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(msg) => yield Ok(Event::default().data(msg)),
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(_)) => continue,
            }
        }
    };

    Sse::new(stream)
}

/// POSTでメッセージを送信
async fn chat_send(
    State(state): State<SharedState>,
    Json(payload): Json<ChatMessage>,
) -> &'static str {
    let _ = state.chat_tx.send(payload.msg);
    "OK"
}
