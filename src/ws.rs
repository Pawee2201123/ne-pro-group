use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    extract::State,
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::StreamExt;
use crate::state::{SharedState, Player, PublicState};
use crate::state::Phase;

#[derive(serde::Deserialize)]
#[serde(tag="type")]
enum ClientMsg {
    #[serde(rename="join")]
    Join,
    #[serde(rename="start")]
    Start,
    #[serde(rename="speak")]
    Speak,
    #[serde(rename="vote")]
    Vote { target: u32 },
    #[serde(rename="reset")]
    Reset,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
)-> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}


async fn handle_socket(
    mut socket: WebSocket,
    state: SharedState,
){
    // player_id (接続id)割当
    
    let conn_id = {
        let mut id = state.inner.next_player_id.lock().unwrap();
        *id += 1;
        *id
    };
    println!("Assigned conn_id: {}", conn_id);


    state.inner.connections.lock().unwrap().insert(conn_id as usize, conn_id);

    let welcome = serde_json::json!({
        "type": "welcome",
        "conn_id": conn_id
    });

    let _ = socket.send(Message::Text(welcome.to_string())).await;


    state.broadcast_state();

    // broadcast
    let mut rx = state.tx.subscribe();
    

    
loop {
        tokio::select! {
            
            // サーバ→クライアント（state配信）
            Ok(msg) = rx.recv() => {
                // state をパース
                if let Ok(public) = serde_json::from_str::<PublicState>(&msg) {

                    // ★ Playing に入った瞬間だけ
                    if public.phase == Phase::Playing {
                        let my_topic_opt = {
                            let game = state.game.lock().unwrap();
                            let topic_mgr = state.topic_manager.lock().unwrap();

                            match (game.wolf_id, topic_mgr.selected_words.clone()) {
                                (Some(wolf_id), Some((villager, wolf))) => {
                                    if conn_id == wolf_id {
                                        Some(wolf)
                                    } else {
                                        Some(villager)
                                    }
                                }
                                _ => None,
                            }
                            // ← ここで game / topic_mgr は drop される
                        };
                        if let Some(my_topic) = my_topic_opt {
                            send_private_topic(&mut socket, &my_topic).await;
                        }
                    }
                }

                // ★ 通常の state 送信
                let _ = socket.send(Message::Text(msg)).await;
            }

            // クライアント→サーバ（操作）
            maybe_msg = socket.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<ClientMsg>(&text) {
                            handle_command(&state, conn_id, cmd);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

}
fn handle_command(state: &SharedState, conn_id: u32, cmd: ClientMsg) -> Option<String> {
    match cmd {
        ClientMsg::Join => {
            let mut game = state.game.lock().unwrap();

            if game.players.contains_key(&conn_id) {
                // すでに参加済み：joinedだけ返してもOK
                return Some(serde_json::json!({"type":"joined","player_id":conn_id}).to_string());
            }

            if game.players.len() as u32 >= game.max_players {
                return Some(serde_json::json!({"type":"error","message":"満員です"}).to_string());
            }

            let max_speak = game.max_speak;
            game.players.insert(conn_id, Player {
                id: conn_id,
                remaining_speak: max_speak,
                vote: None,
                topic: None,
            });

            drop(game);
            state.broadcast_state();

            // ★本人に joined を返す（UIが確定できる）
            Some(serde_json::json!({"type":"joined","player_id":conn_id}).to_string())
        }

        ClientMsg::Speak => {
            let mut game = state.game.lock().unwrap();

            // ★未参加ならエラー
            if !game.players.contains_key(&conn_id) {
                return Some(serde_json::json!({"type":"error","message":"参加してから発言してください"}).to_string());
            }

            if game.phase != Phase::Playing { return None; }

            if let Some(p) = game.players.get_mut(&conn_id) {
                if p.remaining_speak > 0 {
                    p.remaining_speak -= 1;
                }
            }

            drop(game);
            state.broadcast_state();
            None
        }

        ClientMsg::Vote { target } => {
            let mut game = state.game.lock().unwrap();

            // ★未参加ならエラー
            if !game.players.contains_key(&conn_id) {
                return Some(serde_json::json!({"type":"error","message":"参加してから投票してください"}).to_string());
            }

            if game.phase != Phase::Voting { return None; }

            if let Some(p) = game.players.get_mut(&conn_id) {
                p.vote = Some(target);
            }

            drop(game);
            state.broadcast_state();
            None
        }

        ClientMsg::Start => {
            let mut game = state.game.lock().unwrap();
            if game.players.len() < 2 { return None; }

            // テーマを設定
            let mut topic_mgr = state.topic_manager.lock().unwrap();
            topic_mgr.start_theme_phase(game.players.len());
            game.genre = topic_mgr.genre.clone();

            // フェーズ更新
            game.phase = Phase::ThemeSelection;
            game.game_id = rand::random();

            drop(topic_mgr);
            drop(game);

            // ThemeSelection を即送信
            state.broadcast_state();

            None
        }

        ClientMsg::Reset => {
            let mut game = state.game.lock().unwrap();
            game.phase = Phase::Waiting;
            game.players.clear();
            game.wolf_id = None;
            game.executed_id = None;
            game.is_villager_win = None;
            game.game_id = rand::random();
            drop(game);
            let _ = state.chat_tx.send("__CLEAR__".into());
            state.broadcast_state();

            // ★全員未参加に戻ったので、本人にも通知してUIを戻せるように
            Some(serde_json::json!({"type":"reset_done"}).to_string())
        }
    }
}

async fn send_private_topic(
    socket: &mut WebSocket,
    topic: &str,
) {
    let msg = serde_json::json!({
        "type": "your_topic",
        "topic": topic
    });
    let _ = socket.send(Message::Text(msg.to_string())).await;
}