use rand::seq::SliceRandom;

/// プレイヤーの提出情報
#[derive(Clone)]
pub struct PlayerKeyword {
    pub player_id: u32,
    pub keyword: String,
}

/// テーマ管理
pub struct TopicManager {
    pub genre: String,                     // 今は theme と呼ばれているものをジャンルとして使う
    submitted_keywords: Vec<PlayerKeyword>,
    expected_players: usize,
    pub selected_words: Option<(String, String)>, // (villager_word, wolf_word)
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            genre: String::new(),
            submitted_keywords: vec![],
            expected_players: 0,
            selected_words: None,
        }
    }

    /// テーマ（ジャンル）決定
    pub fn start_theme_phase(&mut self, players: usize) -> String {
        let genres = vec![
            "フルーツ","スポーツ","都道府県","お菓子",
            "ジュース","麺類","料理","文房具","家具","色","乗り物"
        ];
        self.genre = genres[rand::random::<usize>() % genres.len()].to_string();
        self.submitted_keywords.clear();
        self.expected_players = players;
        self.selected_words = None;
        println!("GENRE = {}", self.genre);
        self.genre.clone()
    }

    /// キーワード提出
    pub fn submit_keyword(&mut self, player_id: u32, keyword: String) {
        // 重複提出は上書き
        if let Some(k) = self.submitted_keywords.iter_mut().find(|k| k.player_id == player_id) {
            k.keyword = keyword;
        } else {
            self.submitted_keywords.push(PlayerKeyword { player_id, keyword });
        }

        // 全員提出済みなら 2つ抽選
        if self.submitted_keywords.len() >= self.expected_players && self.selected_words.is_none() {
            let mut rng = rand::thread_rng();
            let keywords: Vec<_> = self.submitted_keywords.iter().map(|k| k.keyword.clone()).collect();
            let selected = keywords.choose_multiple(&mut rng, 2).cloned().collect::<Vec<_>>();
            self.selected_words = Some((selected[0].clone(), selected[1].clone()));
        }
    }
    /// 全員提出済みか確認
    pub fn all_submitted(&self) -> bool {
        self.selected_words.is_some()
    }

    /// プレイヤーIDごとの割り当てワード
    pub fn get_word_for_player(&self, player_id: u32, wolf_id: u32) -> Option<String> {
        if let Some((ref w1, ref w2)) = self.selected_words {
            if player_id == wolf_id { Some(w2.clone()) } else { Some(w1.clone()) }
        } else {
            None
        }
    }
}