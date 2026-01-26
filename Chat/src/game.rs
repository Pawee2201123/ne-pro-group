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
}

impl WordWolfGame {
    pub fn new() -> Self {
        WordWolfGame {
            phase: GamePhase::Waiting,
            theme: String::new(),
            submitted_keywords: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    // ゲーム開始：お題を決める
    pub fn start_theme_phase(&mut self) -> String {
        let themes = vec!["フルーツ", "スポーツ", "都道府県", "学校にあるもの", "コンビニのおにぎり"];
        let mut rng = rand::thread_rng();
        self.theme = themes.choose(&mut rng).unwrap_or(&"食べ物").to_string();
        
        self.phase = GamePhase::ThemeSelection;
        self.submitted_keywords.clear();
        self.assignments.clear();

        format!("【システム】ゲーム開始！お題は「{}」です。関連するキーワードを入力してください。", self.theme)
    }

    // キーワードを受け取る
    pub fn submit_keyword(&mut self, player_id: String, keyword: String) -> String {
        if self.phase != GamePhase::ThemeSelection {
            return "【システム】現在はキーワード入力期間ではありません。".to_string();
        }
        
        self.submitted_keywords.insert(player_id, keyword.trim().to_string());
        format!("【システム】キーワード「{}」を受け付けました。（現在 {} 人）", keyword.trim(), self.submitted_keywords.len())
    }

    // 全員の入力が終わったか確認
    pub fn check_ready_to_distribute(&self, player_count: usize) -> bool {
        // 参加者数（接続数）と入力数が一致したら次へ
        // ※検証用に「2人以上入力があればOK」とするなど調整可能
        self.submitted_keywords.len() >= player_count && player_count >= 2
    }

    // 役職とキーワードを配る
    // src/game.rs の distribute_roles 関数全体をこれに置き換えてください

    pub fn distribute_roles(&mut self) {
        let mut rng = rand::thread_rng();
        
        // 入力されたキーワードから2つ選ぶ
        let keywords: Vec<String> = self.submitted_keywords.values().cloned().collect();
        if keywords.len() < 2 { return; }
        
        let chosen_keywords = keywords.choose_multiple(&mut rng, 2).cloned().collect::<Vec<_>>();
        let citizen_word = &chosen_keywords[0];
        let wolf_word = &chosen_keywords[1];

        // 誰をウルフにするか
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
    // プレイヤーごとの秘密のメッセージを取得
    pub fn get_secret_message(&self, player_id: &str) -> Option<String> {
        self.assignments.get(player_id).map(|status| {
            let role = if status.is_wolf { "ウルフ" } else { "市民" };
            format!("【秘密の通知】あなたの役職は「{}」、ワードは「{}」です。", role, status.word)
        })
    }
}