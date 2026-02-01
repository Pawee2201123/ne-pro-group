// types.rs - Shared types used throughout the application
//
// 🎓 Learning Points:
// - Type aliases make code more readable and easier to change
// - Strong typing prevents mixing up different kinds of IDs
// - Derive macros automatically implement common traits

use std::fmt;

/// 🎓 Type Alias: A new name for an existing type
/// This is just a String, but the name makes intent clear
/// We could change this to a struct later for more type safety
pub type PlayerId = String;
pub type RoomId = String;

/// 🎓 Enum: Represents one of several possible values
/// This is safer than using strings like "citizen" or "wolf"
/// The compiler ensures you handle all cases!
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Citizen,  // Regular player
    Wolf,     // The player with the different word
}

/// 🎓 Implementing Display trait for pretty printing
/// This lets us use {} in format! and println!
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Citizen => write!(f, "市民"),
            Role::Wolf => write!(f, "ワードウルフ"),
        }
    }
}

/// Theme genre selection
/// 🎓 Hash trait is needed to use this as a HashMap key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeGenre {
    Food,
    Animal,
    Place,
    Object,
    Custom(String),  // For user-defined themes
}

/// 🎓 Config struct: Immutable settings for a game room
/// Using a struct groups related data together
#[derive(Debug, Clone)]
pub struct RoomConfig {
    pub room_name: String,
    pub max_players: usize,
    pub wolf_count: usize,
    pub theme_genre: ThemeGenre,
    /// Discussion time in seconds (e.g., 180 = 3 minutes)
    pub discussion_time: u64,
}

impl RoomConfig {
    /// 🎓 Constructor pattern: new() is idiomatic in Rust
    /// This is an "associated function" (like a static method)
    pub fn new(
        room_name: String,
        max_players: usize,
        wolf_count: usize,
        theme_genre: ThemeGenre,
        discussion_time: u64,
    ) -> Self {
        RoomConfig {
            room_name,
            max_players,
            wolf_count,
            theme_genre,
            discussion_time,
        }
    }

    /// 🎓 Validation method: Returns Result for error handling
    /// This prevents invalid configs from being created
    pub fn validate(&self) -> Result<(), String> {
        if self.max_players < 3 {
            return Err("最低3人必要です".to_string());
        }

        if self.wolf_count == 0 {
            return Err("最低1人のワードウルフが必要です".to_string());
        }

        // 🎓 NEW: Ensure wolves are always in the minority
        // This prevents unbalanced games like 3 wolves vs 1 citizen
        let max_allowed_wolves = (self.max_players - 1) / 2;
        if self.wolf_count > max_allowed_wolves {
            return Err(format!(
                "{}人部屋では最大{}人のワードウルフまでです（少数派を保つため）",
                self.max_players, max_allowed_wolves
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config = RoomConfig::new(
            "テストルーム".to_string(),
            5,
            1,
            ThemeGenre::Food,
            180, // 3 minutes
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_too_few_players() {
        let config = RoomConfig::new(
            "テストルーム".to_string(),
            2,
            1,
            ThemeGenre::Food,
            180,
        );
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_config_too_many_wolves() {
        let config = RoomConfig::new(
            "テストルーム".to_string(),
            5,
            5,
            ThemeGenre::Food,
            180,
        );
        assert!(config.validate().is_err());
    }
}
