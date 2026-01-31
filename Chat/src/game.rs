use rand::seq::{SliceRandom, IteratorRandom};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    Waiting,
    ThemeSelection,
    GameStart,
}

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub is_wolf: bool,
    pub word: String,
}

pub struct WordWolfGame {
    pub phase: GamePhase,
    pub theme: String,
    pub submitted_keywords: HashMap<String, String>,
    pub assignments: HashMap<String, PlayerStatus>,
    pub expected_players: usize,
}

impl WordWolfGame {
    pub fn new() -> Self {
        WordWolfGame {
            phase: GamePhase::Waiting,
            theme: String::new(),
            submitted_keywords: HashMap::new(),
            assignments: HashMap::new(),
            expected_players: 0,
        }
    }

    // ゲーム開始
    pub fn start_theme_phase(&mut self, players: usize) -> String {
        let themes = vec!["フルーツ","スポーツ","都道府県","お菓子","ジュース","麺類","料理","文房具","家具","色","麺類","乗り物"];
        let mut rng = rand::thread_rng();
        self.theme = themes.choose(&mut rng).unwrap_or(&"食べ物").to_string();
        
        self.phase = GamePhase::ThemeSelection;
        self.submitted_keywords.clear();
        self.assignments.clear();
        self.expected_players = players;

        format!("【システム】ゲーム開始！(参加予定: {}人) お題は「{}」です。関連ワードを入力してください。", players, self.theme)
    }

    // キーワードを受け取る (重複チェック追加)
    pub fn submit_keyword(&mut self, player_id: String, keyword: String) -> String {
        if self.phase != GamePhase::ThemeSelection {
            return "【システム】現在はキーワード入力期間ではありません。".to_string();
        }

        // 既に入力済みなら更新せずエラーを返す
        if self.submitted_keywords.contains_key(&player_id) {
            return "【エラー】キーワードは既に入力済みです。変更できません。".to_string();
        }
        
        self.submitted_keywords.insert(player_id, keyword.trim().to_string());
        format!("【システム】キーワードを受け付けました。（現在 {}/{} 人）", self.submitted_keywords.len(), self.expected_players)
    }

    // 全員の入力が終わったか確認
    pub fn check_ready_to_distribute(&self) -> bool {
        self.submitted_keywords.len() >= self.expected_players && self.expected_players >= 2
    }

    // 役職配布
    pub fn distribute_roles(&mut self) {
        let mut rng = rand::thread_rng();
        let keywords: Vec<String> = self.submitted_keywords.values().cloned().collect();
        if keywords.len() < 2 { return; }
        
        let chosen_keywords = keywords.choose_multiple(&mut rng, 2).cloned().collect::<Vec<_>>();
        let citizen_word = &chosen_keywords[0];
        let wolf_word = &chosen_keywords[1];

        let player_ids: Vec<String> = self.submitted_keywords.keys().cloned().collect();
        let wolf_id = player_ids.choose(&mut rng).unwrap().clone();

        for pid in player_ids {
            let (is_wolf, word) = if pid == wolf_id {
                (true, wolf_word.clone())
            } else {
                (false, citizen_word.clone())
            };
            self.assignments.insert(pid, PlayerStatus { is_wolf, word });
        }
        self.phase = GamePhase::GameStart;
    }

    pub fn get_secret_message(&self, player_id: &str) -> Option<String> {
        self.assignments.get(player_id).map(|status| {
            let role = if status.is_wolf { "ウルフ" } else { "市民" };
            format!("【秘密の通知】あなたの役職は「{}」、ワードは「{}」です。", role, status.word)
        })
    }
}