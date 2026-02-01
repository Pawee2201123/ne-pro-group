// game/player.rs - Player data and behavior
//
// 🎓 Key Concepts:
// - Structs with private fields (encapsulation)
// - Methods vs Associated functions
// - Option<T> for optional data

use crate::types::{PlayerId, Role};

/// Represents a player in the game
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    /// Unique identifier for this player
    id: PlayerId,

    /// Display name chosen by the player
    name: String,

    /// The role assigned to this player (Citizen or Wolf)
    /// None until roles are assigned
    role: Option<Role>,

    /// The theme word assigned to this player
    /// None until themes are assigned
    theme: Option<String>,

    /// Whether this player is still active (not voted out)
    active: bool,
}

impl Player {
    /// 🎓 Constructor: Create a new player
    /// Notice we don't expose the struct fields directly - this is encapsulation!
    pub fn new(id: PlayerId, name: String) -> Self {
        Player {
            id,
            name,
            role: None,      // Not assigned yet
            theme: None,     // Not assigned yet
            active: true,    // Players start active
        }
    }

    // 🎓 Getters: Read-only access to private fields
    // This prevents external code from modifying data in invalid ways

    pub fn id(&self) -> &PlayerId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Option<Role> {
        self.role
    }

    pub fn theme(&self) -> Option<&str> {
        self.theme.as_deref()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_wolf(&self) -> bool {
        self.role == Some(Role::Wolf)
    }

    pub fn is_citizen(&self) -> bool {
        self.role == Some(Role::Citizen)
    }

    // 🎓 Setters: Controlled modification
    // We validate and control how data changes

    /// Assign a role to this player
    pub fn assign_role(&mut self, role: Role) {
        self.role = Some(role);
    }

    /// Assign a theme word to this player
    pub fn assign_theme(&mut self, theme: String) {
        self.theme = Some(theme);
    }

    /// Eliminate this player from the game
    pub fn eliminate(&mut self) {
        self.active = false;
    }

    /// Check if the player has been fully initialized
    pub fn is_ready_to_play(&self) -> bool {
        self.role.is_some() && self.theme.is_some()
    }
}

/// 🎓 Display trait for pretty printing
use std::fmt;

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name,
            if self.active { "active" } else { "eliminated" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new("p1".to_string(), "Alice".to_string());
        assert_eq!(player.id(), "p1");
        assert_eq!(player.name(), "Alice");
        assert!(player.is_active());
        assert!(!player.is_ready_to_play());
    }

    #[test]
    fn test_role_assignment() {
        let mut player = Player::new("p1".to_string(), "Bob".to_string());
        player.assign_role(Role::Wolf);
        assert!(player.is_wolf());
        assert!(!player.is_citizen());
    }

    #[test]
    fn test_theme_assignment() {
        let mut player = Player::new("p1".to_string(), "Charlie".to_string());
        player.assign_theme("りんご".to_string());
        assert_eq!(player.theme(), Some("りんご"));
    }

    #[test]
    fn test_ready_to_play() {
        let mut player = Player::new("p1".to_string(), "David".to_string());
        assert!(!player.is_ready_to_play());

        player.assign_role(Role::Citizen);
        assert!(!player.is_ready_to_play());

        player.assign_theme("みかん".to_string());
        assert!(player.is_ready_to_play());
    }

    #[test]
    fn test_elimination() {
        let mut player = Player::new("p1".to_string(), "Eve".to_string());
        assert!(player.is_active());

        player.eliminate();
        assert!(!player.is_active());
    }
}
