// routes.rs
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    http::StatusCode,
    Json,
};
use crate::state::{
    SharedState,
    Player,
    Phase,
};
use std::collections::HashMap;
use crate::ws::ws_handler;


#[derive(serde::Deserialize)]
struct KeywordPayload {
    keyword: String,
}

#[derive(serde::Deserialize)]
struct PlayerIdQuery {
    player_id: u32,
}


pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/join", post(join))
        .route("/submit_keyword", post(submit_keyword))
        .route("/speak", post(speak))
        .route("/vote/:target", post(vote)) 
        .route("/reset", post(reset))
}
async fn join(
    State(state): State<SharedState>
) -> Result<axum::Json<u32>, StatusCode> {
    let mut game = state.game.lock().unwrap();


    if game.players.len() as u32 >= game.max_players {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = game.players.len() as u32 + 1;
    let max_speak = game.max_speak;

    game.players.insert(id, Player {
        id,
        remaining_speak: max_speak,
        vote: None,
        topic: None,
    });

    drop(game);
    state.broadcast_state(); 

    Ok(axum::Json(id))
}



/*async fn start(State(state): State<SharedState>) -> StatusCode {
    let game_id;
    {
        let mut game = state.game.lock().unwrap();

        
        if game.players.len() < 2 {
            return StatusCode::BAD_REQUEST; // 2人未満なら開始不可
        }

        //お題
        let mut topic_manager = TopicManager::new();
        let msg = topic_manager.start_theme_phase(game.players.len() as usize);
        game.topic = topic_manager.theme.clone();

        // wolf_id ランダム（最小実装）
        let ids: Vec<u32> = game.players.keys().cloned().collect();
        let wolf = ids[rand::random::<usize>() % ids.len()];
        game.wolf_id = Some(wolf);
        game.executed_id = None;

        game.phase = Phase::Playing;
        game.remaining_time = 30;
        game.voting_time = 10;
        game.is_villager_win = None;
        game.game_id = rand::random::<u64>();
        game_id = game.game_id;
        game.executed_id = None;
    }

    state.broadcast_state();

    spawn_playing_timer(state.clone(), game_id);
    spawn_voting_timer(state.clone(), game_id);

    StatusCode::OK
}*/
async fn submit_keyword(
    State(state): State<SharedState>,
    axum::extract::Query(q): axum::extract::Query<PlayerIdQuery>,
    Json(payload): Json<KeywordPayload>,
) -> StatusCode {
    let player_id = q.player_id;

    let mut game = state.game.lock().unwrap();
    if game.phase != Phase::ThemeSelection {
        return StatusCode::BAD_REQUEST;
    }

    let mut topic_mgr = state.topic_manager.lock().unwrap();
    topic_mgr.submit_keyword(player_id, payload.keyword.clone());

    // 全員提出済みなら Playing フェーズ開始
    if topic_mgr.all_submitted() {
        let ids: Vec<u32> = game.players.keys().cloned().collect();
        let wolf = ids[rand::random::<usize>() % ids.len()];
        game.wolf_id = Some(wolf);

        let max_speak = game.max_speak;

        // 各プレイヤーに割り当てワード
        for (id, p) in game.players.iter_mut() {
            if let Some(word) = topic_mgr.get_word_for_player(*id, wolf) {
                p.topic = Some(word);
                p.remaining_speak = max_speak;  // 必要に応じて初期化
            }
        }

        let game_id = game.game_id;

        game.phase = Phase::Playing;
        game.remaining_time = 30;
        game.voting_time = 10;
        drop(topic_mgr);
        drop(game);

        state.broadcast_state();
        spawn_playing_timer(state.clone(), game_id);
    }

    StatusCode::OK
}

async fn speak(
    State(state): State<SharedState>,
    axum::extract::Query(q): axum::extract::Query<PlayerIdQuery>,
) -> StatusCode {
    let player_id = q.player_id;
    let mut game = state.game.lock().unwrap();

    if game.phase != Phase::Playing {
        return StatusCode::BAD_REQUEST;
    }

    let p = match game.players.get_mut(&player_id) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND,
    };

    if p.remaining_speak == 0 {
        return StatusCode::FORBIDDEN;
    }

    p.remaining_speak -= 1;

    drop(game);
    state.broadcast_state(); // 表示変える

    StatusCode::OK
}


async fn vote(
    State(state): State<SharedState>,
    axum::extract::Path(target): axum::extract::Path<u32>,
    axum::extract::Query(q): axum::extract::Query<PlayerIdQuery>,
) -> StatusCode {
    let player_id = q.player_id;

    let mut game = state.game.lock().unwrap();

    if game.phase != Phase::Voting {
        return StatusCode::BAD_REQUEST;
    }

    let p = match game.players.get_mut(&player_id) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND,
    };

    p.vote = Some(target);

    drop(game);
    state.broadcast_state();

    StatusCode::OK
}

async fn reset(
    State(state): State<SharedState>,
) -> StatusCode {
    let mut game = state.game.lock().unwrap();

    game.phase = Phase::Waiting;
    game.players.clear();
    game.wolf_id = None;
    game.is_villager_win = None;
    game.game_id = rand::random();
    game.executed_id = None;

    {
        let mut topic_mgr = state.topic_manager.lock().unwrap();
    *topic_mgr = crate::topic::TopicManager::new();
    }

    let _ = state.chat_tx.send("__CLEAR__".into());

    drop(game);
    state.broadcast_state();

    StatusCode::OK
}



fn spawn_playing_timer(state: SharedState, game_id: u64) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let mut game = state.game.lock().unwrap();

            // 🔴 ゲームが変わったらこのタイマーは終了
            if game.game_id != game_id {
                break;
            }

            // 🔴 フェーズが Playing 以外なら終了
            if game.phase != Phase::Playing {
                break;
            }

            game.remaining_time -= 1;

            if game.remaining_time <= 0 {
                game.phase = Phase::Voting;
                let game_id = game.game_id;
                drop(game);
                state.broadcast_state();
                spawn_voting_timer(state.clone(), game_id);
                break;
            }

            drop(game);
            state.broadcast_state();
        }

        // フェーズ遷移直後にも1回通知
        state.broadcast_state();
    });
}

fn spawn_voting_timer(state: SharedState, game_id: u64) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let mut game = state.game.lock().unwrap();

            // ゲームが変わったら終了
            if game.game_id != game_id {
                break;
            }

            // Voting 以外なら待機
            if game.phase != Phase::Voting {
                continue;
            }

            // 時間を減らす（毎秒更新）
            game.voting_time -= 1;

            // 終了条件： 時間切れ
            if game.voting_time <= 0 {
                drop(game);
                judge_and_executed(state.clone());
                state.broadcast_state();
                break;
            }

            drop(game);
            state.broadcast_state();
        }
    });
}

pub fn judge_and_executed(state: SharedState) {
    let mut game = state.game.lock().unwrap();

    let mut count = HashMap::new();
    for p in game.players.values() {
        if let Some(v) = p.vote {
            *count.entry(v).or_insert(0) += 1;
        }
    }

    let executed = count.into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(id, _)| id);

    game.executed_id = executed;
    game.is_villager_win = match (executed, game.wolf_id) {
        (Some(e), Some(w)) => Some(e == w),
        _ => None,
    };

    game.phase = Phase::Result;
}