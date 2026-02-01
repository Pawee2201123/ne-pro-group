use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use serde_json;
use crate::topic::TopicManager;

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub remaining_speak: u32,
    pub vote: Option<u32>,
    pub topic: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Phase {
    Waiting,
    ThemeSelection,
    Playing,
    Voting,
    Result,
}

//サーバ内部の真の状態：wolf_id 等も含む
#[derive(Clone, Serialize)]
pub struct GameState {
    pub phase: Phase,
    pub players: HashMap<u32, Player>,
    pub max_players: u32,
    pub max_speak: u32,
    pub genre: String,           // 全員に見せるジャンル
    pub wolf_id: Option<u32>,
    pub executed_id: Option<u32>,
    pub remaining_time: i32,
    pub voting_time: i32,
    pub is_villager_win: Option<bool>,
    pub game_id: u64,
}

//クライアントへ送る状態：wolf_id は含めない
#[derive(Clone, Serialize, Deserialize)]
pub struct PublicState {
    pub phase: Phase,
    pub players: HashMap<u32, Player>,
    pub max_players: u32,
    pub max_speak: u32,
    pub wolf_id: Option<u32>,      // Result の時だけ入れる
    pub genre: String,          // 全員に見せるジャンル
    pub executed_id: Option<u32>,
    pub remaining_time: i32,
    pub voting_time: i32,
    pub is_villager_win: Option<bool>,
    pub game_id: u64,
}


impl From<&GameState> for PublicState {
    fn from(g: &GameState) -> Self {
        let reveal_wolf = matches!(g.phase, Phase::Result);

        Self {
            phase: g.phase.clone(),
            players: g.players.clone(),
            max_players: g.max_players,
            max_speak: g.max_speak,
             genre: g.genre.clone(),
            remaining_time: g.remaining_time,
            voting_time: g.voting_time,
            is_villager_win: g.is_villager_win,
            game_id: g.game_id,

            // Result の時だけ公開
            wolf_id: if reveal_wolf { g.wolf_id } else { None },
            executed_id: if reveal_wolf { g.executed_id } else { None },

        }
    }
}


//WS接続やplayer_id採番などの内部管理
pub struct SharedStateInner {
    pub connections: Mutex<HashMap<usize, u32>>, // ws_id -> player_id（今は未使用でもOK）
    pub next_player_id: Mutex<u32>,
}

// 🔵 WebSocket 用の共有 State


pub struct AppState {
    pub game: Mutex<GameState>,
    pub topic_manager: Arc<Mutex<TopicManager>>,
    pub tx: broadcast::Sender<String>,
    pub chat_tx: broadcast::Sender<String>,
    pub inner: SharedStateInner,
}



impl AppState {
    pub fn broadcast_state(&self) {
        let game = self.game.lock().unwrap();
        let public = PublicState::from(&*game);
        let json = serde_json::to_string(&public).unwrap();
        let _ = self.tx.send(json);
    }
}


pub type SharedState = std::sync::Arc<AppState>;