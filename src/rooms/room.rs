// rooms/room.rs - A single game room
//
// 🎓 Key Concepts:
// - Bringing together all our game components
// - Managing room lifecycle
// - Coordinating state transitions
// - Player management within a room

use crate::game::{GameState, Player, ThemeDatabase};
use crate::types::{PlayerId, RoomConfig, RoomId};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::SystemTime;

/// 🎓 Type alias for SSE senders
/// Each connected client gets an mpsc::Sender to receive updates
pub type Senders = Vec<mpsc::Sender<String>>;

/// A game room containing players and game state
pub struct Room {
    /// Unique identifier for this room
    id: RoomId,

    /// Room configuration (name, player limits, wolf count, etc.)
    config: RoomConfig,

    /// Current game state
    state: GameState,

    /// Players in this room (keyed by player ID)
    players: HashMap<PlayerId, Player>,

    /// SSE connections for broadcasting updates
    /// 🎓 Note: In a real server with Arc<Mutex<_>>, this would be shared
    /// For now, we keep it simple
    senders: Senders,

    /// Votes in the current voting phase
    votes: HashMap<PlayerId, PlayerId>, // voter -> target

    /// When discussion phase started (for timer)
    discussion_started_at: Option<SystemTime>,
}

impl Room {
    /// Create a new room
    pub fn new(id: RoomId, config: RoomConfig) -> Result<Self, String> {
        // Validate config
        config.validate()?;

        Ok(Room {
            id,
            config,
            state: GameState::new(),
            players: HashMap::new(),
            senders: Vec::new(),
            votes: HashMap::new(),
            discussion_started_at: None,
        })
    }

    // 🎓 Getters
    pub fn id(&self) -> &RoomId {
        &self.id
    }

    pub fn config(&self) -> &RoomConfig {
        &self.config
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn players(&self) -> &HashMap<PlayerId, Player> {
        &self.players
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.config.max_players
    }

    // 🎓 Player Management

    /// Add a player to the room
    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.is_full() {
            return Err("Room is full".to_string());
        }

        if !self.state.is_lobby() {
            return Err("Cannot join after game has started".to_string());
        }

        let player_id = player.id().clone();
        self.players.insert(player_id.clone(), player);

        // Broadcast update
        self.broadcast(&format!("Player {} joined", player_id));

        Ok(())
    }

    /// Remove a player from the room
    pub fn remove_player(&mut self, player_id: &PlayerId) -> Result<(), String> {
        if self.players.remove(player_id).is_none() {
            return Err("Player not found".to_string());
        }

        // Broadcast update
        self.broadcast(&format!("Player {} left", player_id));

        Ok(())
    }

    /// Mark a player as ready
    pub fn mark_ready(&mut self, player_id: &PlayerId) -> Result<(), String> {
        if !self.players.contains_key(player_id) {
            return Err("Player not found".to_string());
        }

        self.state.mark_player_ready(player_id.clone())?;

        // Check if all players are ready
        if self.state.all_players_ready(self.players.len()) {
            // Validate we have enough players before starting
            // 🎓 We need more players than wolves to ensure citizens can win
            if self.players.len() <= self.config.wolf_count {
                let needed = self.config.wolf_count + 1;
                self.broadcast(&format!(
                    "あと{}人必要です（現在{}人、ワードウルフ{}人）。部屋ID「{}」を他のプレイヤーに共有してください！",
                    needed - self.players.len(),
                    self.players.len(),
                    self.config.wolf_count,
                    self.id
                ));
                return Ok(());
            }

            self.broadcast("全員準備完了！ゲームを開始します...");
            self.start_game()?;
        }

        Ok(())
    }

    // 🎓 Game Flow

    /// Start the game (assign roles and themes)
    fn start_game(&mut self) -> Result<(), String> {
        // Transition to theme submission
        self.state.transition_to_theme_submission()?;

        // Assign roles
        // 🎓 Convert HashMap values to a Vec so we can pass a mutable slice
        let mut players_vec: Vec<Player> = self.players.values().cloned().collect();
        let wolf_ids = crate::game::rules::assign_roles(
            &mut players_vec,
            self.config.wolf_count,
        );

        // 🎓 Update the players in the HashMap with their assigned roles
        for player in players_vec {
            self.players.insert(player.id().clone(), player);
        }

        // Assign themes
        let theme_db = ThemeDatabase::new();
        let theme_pair = theme_db
            .get_random_theme(&self.config.theme_genre)
            .ok_or("Failed to get theme")?;

        for player in self.players.values_mut() {
            let theme = if wolf_ids.contains(player.id()) {
                theme_pair.wolf_theme.clone()
            } else {
                theme_pair.citizen_theme.clone()
            };
            player.assign_theme(theme);
        }

        self.broadcast("Game started! Check your roles and themes.");

        Ok(())
    }

    /// Confirm a player has seen their theme
    pub fn confirm_theme(&mut self, player_id: &PlayerId) -> Result<(), String> {
        if !self.players.contains_key(player_id) {
            return Err("Player not found".to_string());
        }

        self.state.confirm_theme(player_id.clone())?;

        // Check if all confirmed
        if self.state.all_themes_confirmed(self.players.len()) {
            self.state.transition_to_discussion()?;

            // 🎓 Start the discussion timer
            self.discussion_started_at = Some(SystemTime::now());

            let minutes = self.config.discussion_time / 60;
            let seconds = self.config.discussion_time % 60;
            self.broadcast(&format!(
                "全員確認完了！ディスカッションを開始します。制限時間: {}分{}秒",
                minutes, seconds
            ));
        }

        Ok(())
    }

    /// Start voting phase
    pub fn start_voting(&mut self) -> Result<(), String> {
        self.state.transition_to_voting()?;
        self.votes.clear();
        self.broadcast("投票フェーズが始まりました！ワードウルフだと思う人に投票してください。");
        Ok(())
    }

    /// Submit a vote
    pub fn submit_vote(&mut self, voter_id: &PlayerId, target_id: &PlayerId) -> Result<(), String> {
        if !self.players.contains_key(voter_id) {
            return Err("Voter not found".to_string());
        }

        if !self.players.contains_key(target_id) {
            return Err("Target not found".to_string());
        }

        self.votes.insert(voter_id.clone(), target_id.clone());
        self.state.record_vote(voter_id.clone())?;

        // Check if all voted
        if self.state.all_players_voted(self.players.len()) {
            self.tally_votes()?;
        }

        Ok(())
    }

    /// Tally votes and eliminate player
    /// 🎓 In Word Wolf, game ALWAYS ends after one vote!
    fn tally_votes(&mut self) -> Result<(), String> {
        let votes: Vec<crate::game::Vote> = self
            .votes
            .iter()
            .map(|(voter, target)| crate::game::Vote {
                voter: voter.clone(),
                target: target.clone(),
            })
            .collect();

        let result = crate::game::rules::tally_votes(&votes)
            .ok_or("Failed to tally votes")?;

        // Check if eliminated player was a wolf BEFORE eliminating
        let eliminated_was_wolf = self
            .players
            .get(&result.eliminated_player)
            .map(|p| p.is_wolf())
            .unwrap_or(false);

        // Eliminate the player
        if let Some(player) = self.players.get_mut(&result.eliminated_player) {
            player.eliminate();
            self.broadcast(&format!(
                "{}さんが{}票で脱落しました",
                result.eliminated_player, result.vote_count
            ));
        }

        // 🎓 WORD WOLF RULE: Game ALWAYS ends after one vote
        // Citizens win if they eliminated a wolf, wolves win otherwise
        let citizens_won = eliminated_was_wolf;

        let players_vec: Vec<Player> = self.players.values().cloned().collect();
        let wolf_ids: Vec<PlayerId> = players_vec
            .iter()
            .filter(|p| p.is_wolf())
            .map(|p| p.id().clone())
            .collect();

        self.state.transition_to_finished(citizens_won, wolf_ids)?;

        let winner_msg = if citizens_won {
            "ゲーム終了！市民の勝利です！ワードウルフを見つけました！"
        } else {
            "ゲーム終了！ワードウルフの勝利です！市民を騙すことに成功しました！"
        };
        self.broadcast(winner_msg);

        Ok(())
    }

    // 🎓 SSE Broadcasting

    /// Add an SSE connection
    pub fn add_sender(&mut self, sender: mpsc::Sender<String>) {
        self.senders.push(sender);
    }

    /// Broadcast a message to all connected clients
    fn broadcast(&mut self, message: &str) {
        // 🎓 Retain only senders that successfully receive
        // This automatically removes disconnected clients
        self.senders.retain(|sender| sender.send(message.to_string()).is_ok());
    }

    /// Public method to broadcast chat messages
    pub fn send_chat_message(&mut self, player_name: &str, message: &str) {
        let formatted = format!("CHAT|{}|{}", player_name, message);
        self.broadcast(&formatted);
    }

    /// Get remaining discussion time in seconds (returns None if not in discussion)
    pub fn get_remaining_time(&self) -> Option<u64> {
        if !self.state.is_discussion() {
            return None;
        }

        let started_at = self.discussion_started_at?;
        let elapsed = SystemTime::now()
            .duration_since(started_at)
            .ok()?;

        let elapsed_secs = elapsed.as_secs();
        let total_time = self.config.discussion_time;

        if elapsed_secs >= total_time {
            Some(0) // Time's up
        } else {
            Some(total_time - elapsed_secs)
        }
    }

    /// Check if discussion timer has expired and auto-start voting if so
    /// Returns true if voting was auto-started
    pub fn check_and_auto_vote(&mut self) -> bool {
        if let Some(remaining) = self.get_remaining_time() {
            if remaining == 0 {
                // Timer expired! Auto-start voting
                if let Ok(_) = self.start_voting() {
                    return true;
                }
            }
        }
        false
    }

    /// Get the current game state as JSON-like string
    /// (In real app, use serde_json)
    pub fn get_state_snapshot(&self) -> String {
        format!(
            "{{\"room_id\":\"{}\",\"player_count\":{},\"max_players\":{},\"state\":\"{}\"}}",
            self.id,
            self.players.len(),
            self.config.max_players,
            if self.state.is_lobby() {
                "lobby"
            } else if self.state.is_discussion() {
                "discussion"
            } else if self.state.is_voting() {
                "voting"
            } else if self.state.is_finished() {
                "finished"
            } else {
                "unknown"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ThemeGenre;

    fn create_test_room() -> Room {
        let config = RoomConfig::new(
            "Test Room".to_string(),
            4,
            1,
            ThemeGenre::Food,
            180,
        );
        Room::new("room1".to_string(), config).unwrap()
    }

    #[test]
    fn test_room_creation() {
        let room = create_test_room();
        assert_eq!(room.id(), "room1");
        assert_eq!(room.player_count(), 0);
        assert!(!room.is_full());
    }

    #[test]
    fn test_add_player() {
        let mut room = create_test_room();
        let player = Player::new("p1".to_string(), "Alice".to_string());

        room.add_player(player).unwrap();
        assert_eq!(room.player_count(), 1);
    }

    #[test]
    fn test_room_full() {
        let mut room = create_test_room();

        for i in 0..4 {
            let player = Player::new(format!("p{}", i), format!("Player{}", i));
            room.add_player(player).unwrap();
        }

        assert!(room.is_full());

        // Try to add one more
        let extra = Player::new("p5".to_string(), "Extra".to_string());
        assert!(room.add_player(extra).is_err());
    }
}
