// game/state.rs - Game state machine
//
// 🎓 Key Rust Concept: Enums with Associated Data
// Unlike C/Java enums, Rust enums can hold different data for each variant!
// This is like a tagged union - it's ONE of these states, and carries relevant data.

use crate::types::PlayerId;
use std::collections::HashSet;

/// 🎓 The Game State Machine
///
/// This enum represents all possible states a game can be in.
/// Each variant holds data specific to that state.
///
/// Flow: Lobby → ThemeSubmission → Discussion → Voting → Finished
#[derive(Debug, Clone)]
pub enum GameState {
    /// Waiting for players to join
    Lobby {
        /// Players who have marked themselves as ready
        ready_players: HashSet<PlayerId>,
    },

    /// Players are submitting their themes
    /// (In Word Wolf, players might submit theme suggestions before the game starts,
    /// or this could track who has viewed their assigned theme)
    ThemeSubmission {
        /// Players who have confirmed they've seen their theme
        confirmed_players: HashSet<PlayerId>,
    },

    /// Main game phase - players discuss and try to find the wolf
    Discussion {
        /// Time remaining in seconds (optional timer)
        time_remaining: Option<u32>,
    },

    /// Players vote on who they think is the wolf
    Voting {
        /// Players who have submitted their vote
        voted_players: HashSet<PlayerId>,
    },

    /// Game is over
    Finished {
        /// The winning team: true if citizens won, false if wolves won
        citizens_won: bool,
        /// Players who were wolves
        wolves: Vec<PlayerId>,
    },
}

impl GameState {
    /// 🎓 Constructor: Create initial state
    pub fn new() -> Self {
        GameState::Lobby {
            ready_players: HashSet::new(),
        }
    }

    /// 🎓 State Query: Check current state type
    /// This uses pattern matching to check which variant we're in
    pub fn is_lobby(&self) -> bool {
        matches!(self, GameState::Lobby { .. })
    }

    pub fn is_theme_submission(&self) -> bool {
        matches!(self, GameState::ThemeSubmission { .. })
    }

    pub fn is_discussion(&self) -> bool {
        matches!(self, GameState::Discussion { .. })
    }

    pub fn is_voting(&self) -> bool {
        matches!(self, GameState::Voting { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, GameState::Finished { .. })
    }

    /// 🎓 State Transition: Move to next state
    /// Returns Result because transitions can fail (wrong state, invalid conditions)
    pub fn transition_to_theme_submission(&mut self) -> Result<(), String> {
        match self {
            GameState::Lobby { .. } => {
                *self = GameState::ThemeSubmission {
                    confirmed_players: HashSet::new(),
                };
                Ok(())
            }
            _ => Err("Can only start theme submission from lobby".to_string()),
        }
    }

    pub fn transition_to_discussion(&mut self) -> Result<(), String> {
        match self {
            GameState::ThemeSubmission { .. } => {
                *self = GameState::Discussion {
                    time_remaining: Some(300), // 5 minutes default
                };
                Ok(())
            }
            _ => Err("Can only start discussion from theme submission".to_string()),
        }
    }

    pub fn transition_to_voting(&mut self) -> Result<(), String> {
        match self {
            GameState::Discussion { .. } => {
                *self = GameState::Voting {
                    voted_players: HashSet::new(),
                };
                Ok(())
            }
            _ => Err("Can only start voting from discussion".to_string()),
        }
    }

    pub fn transition_to_finished(
        &mut self,
        citizens_won: bool,
        wolves: Vec<PlayerId>,
    ) -> Result<(), String> {
        match self {
            GameState::Voting { .. } => {
                *self = GameState::Finished {
                    citizens_won,
                    wolves,
                };
                Ok(())
            }
            _ => Err("Can only finish from voting state".to_string()),
        }
    }

    /// 🎓 State-specific actions
    /// These only work in specific states, enforced by pattern matching

    /// Mark a player as ready (only in Lobby)
    pub fn mark_player_ready(&mut self, player_id: PlayerId) -> Result<(), String> {
        match self {
            GameState::Lobby { ready_players } => {
                ready_players.insert(player_id);
                Ok(())
            }
            _ => Err("Can only mark ready in lobby".to_string()),
        }
    }

    /// Check if all players are ready
    pub fn all_players_ready(&self, total_players: usize) -> bool {
        match self {
            GameState::Lobby { ready_players } => ready_players.len() == total_players,
            _ => false,
        }
    }

    /// Mark player as confirmed their theme
    pub fn confirm_theme(&mut self, player_id: PlayerId) -> Result<(), String> {
        match self {
            GameState::ThemeSubmission { confirmed_players } => {
                confirmed_players.insert(player_id);
                Ok(())
            }
            _ => Err("Can only confirm theme during theme submission".to_string()),
        }
    }

    /// Check if all players confirmed
    pub fn all_themes_confirmed(&self, total_players: usize) -> bool {
        match self {
            GameState::ThemeSubmission { confirmed_players } => {
                confirmed_players.len() == total_players
            }
            _ => false,
        }
    }

    /// Record a vote (only in Voting)
    pub fn record_vote(&mut self, player_id: PlayerId) -> Result<(), String> {
        match self {
            GameState::Voting { voted_players } => {
                voted_players.insert(player_id);
                Ok(())
            }
            _ => Err("Can only vote during voting phase".to_string()),
        }
    }

    /// Check if all players voted
    pub fn all_players_voted(&self, total_players: usize) -> bool {
        match self {
            GameState::Voting { voted_players } => voted_players.len() == total_players,
            _ => false,
        }
    }
}

/// 🎓 Default trait: Provide a default value
impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_flow() {
        let mut state = GameState::new();
        assert!(state.is_lobby());

        // Transition to theme submission
        state.transition_to_theme_submission().unwrap();
        assert!(state.is_theme_submission());

        // Transition to discussion
        state.transition_to_discussion().unwrap();
        assert!(state.is_discussion());

        // Transition to voting
        state.transition_to_voting().unwrap();
        assert!(state.is_voting());

        // Transition to finished
        state
            .transition_to_finished(true, vec!["player1".to_string()])
            .unwrap();
        assert!(state.is_finished());
    }

    #[test]
    fn test_invalid_transitions() {
        let mut state = GameState::new();

        // Can't go directly to voting from lobby
        let result = state.transition_to_voting();
        assert!(result.is_err());
    }

    #[test]
    fn test_ready_tracking() {
        let mut state = GameState::new();

        state.mark_player_ready("player1".to_string()).unwrap();
        assert!(!state.all_players_ready(2));

        state.mark_player_ready("player2".to_string()).unwrap();
        assert!(state.all_players_ready(2));
    }
}
