// game/theme.rs - Theme selection and assignment logic
//
// 🎓 Key Concepts:
// - Pure functions (no side effects, easy to test)
// - HashMap for key-value storage
// - Random number generation
// - Borrowing and ownership

use crate::types::ThemeGenre;
use std::collections::HashMap;

/// A pair of related themes (citizen theme and wolf theme)
#[derive(Debug, Clone)]
pub struct ThemePair {
    pub citizen_theme: String,
    pub wolf_theme: String,
}

impl ThemePair {
    pub fn new(citizen_theme: String, wolf_theme: String) -> Self {
        ThemePair {
            citizen_theme,
            wolf_theme,
        }
    }
}

/// 🎓 Theme database
/// In a real app, this might come from a database or config file
/// For now, we hardcode some examples
pub struct ThemeDatabase {
    themes: HashMap<ThemeGenre, Vec<ThemePair>>,
}

impl ThemeDatabase {
    /// Create a new theme database with predefined themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Food themes
        themes.insert(
            ThemeGenre::Food,
            vec![
                ThemePair::new("りんご".to_string(), "みかん".to_string()),
                ThemePair::new("カレー".to_string(), "シチュー".to_string()),
                ThemePair::new("ラーメン".to_string(), "うどん".to_string()),
                ThemePair::new("寿司".to_string(), "刺身".to_string()),
            ],
        );

        // Animal themes
        themes.insert(
            ThemeGenre::Animal,
            vec![
                ThemePair::new("犬".to_string(), "猫".to_string()),
                ThemePair::new("ライオン".to_string(), "トラ".to_string()),
                ThemePair::new("イルカ".to_string(), "クジラ".to_string()),
            ],
        );

        // Place themes
        themes.insert(
            ThemeGenre::Place,
            vec![
                ThemePair::new("海".to_string(), "山".to_string()),
                ThemePair::new("図書館".to_string(), "書店".to_string()),
                ThemePair::new("公園".to_string(), "遊園地".to_string()),
            ],
        );

        // Object themes
        themes.insert(
            ThemeGenre::Object,
            vec![
                ThemePair::new("鉛筆".to_string(), "ペン".to_string()),
                ThemePair::new("椅子".to_string(), "ソファ".to_string()),
                ThemePair::new("時計".to_string(), "タイマー".to_string()),
            ],
        );

        ThemeDatabase { themes }
    }

    /// 🎓 Pure function: Get a random theme pair for a genre
    /// Takes a reference (&self) so it doesn't consume self
    /// Returns Option because the genre might not exist
    pub fn get_random_theme(&self, genre: &ThemeGenre) -> Option<ThemePair> {
        // Handle Custom genre separately
        match genre {
            ThemeGenre::Custom(_) => {
                // For custom themes, we'd need a different approach
                // For now, just return a default pair
                Some(ThemePair::new("テーマA".to_string(), "テーマB".to_string()))
            }
            _ => {
                // Get the theme list for this genre
                let theme_list = self.themes.get(genre)?;

                // Pick a random index
                // 🎓 Note: We'll use a simple approach here
                // In production, you'd use rand crate
                let index = self.simple_random(theme_list.len());

                // Return a clone of the selected theme
                Some(theme_list[index].clone())
            }
        }
    }

    /// 🎓 Simple pseudo-random number generator
    /// In production, use the `rand` crate instead!
    /// This uses the current time as a seed (not cryptographically secure)
    fn simple_random(&self, max: usize) -> usize {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        (nanos as usize) % max
    }

    /// Add a custom theme pair
    pub fn add_custom_theme(&mut self, genre: ThemeGenre, pair: ThemePair) {
        self.themes.entry(genre).or_insert_with(Vec::new).push(pair);
    }
}

/// 🎓 Default trait implementation
impl Default for ThemeDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_pair_creation() {
        let pair = ThemePair::new("犬".to_string(), "猫".to_string());
        assert_eq!(pair.citizen_theme, "犬");
        assert_eq!(pair.wolf_theme, "猫");
    }

    #[test]
    fn test_database_creation() {
        let db = ThemeDatabase::new();
        assert!(db.themes.contains_key(&ThemeGenre::Food));
        assert!(db.themes.contains_key(&ThemeGenre::Animal));
        assert!(db.themes.contains_key(&ThemeGenre::Place));
        assert!(db.themes.contains_key(&ThemeGenre::Object));
    }

    #[test]
    fn test_get_random_theme() {
        let db = ThemeDatabase::new();
        let theme = db.get_random_theme(&ThemeGenre::Food);
        assert!(theme.is_some());

        let pair = theme.unwrap();
        assert!(!pair.citizen_theme.is_empty());
        assert!(!pair.wolf_theme.is_empty());
    }

    #[test]
    fn test_custom_genre() {
        let db = ThemeDatabase::new();
        let theme = db.get_random_theme(&ThemeGenre::Custom("test".to_string()));
        assert!(theme.is_some());
    }

    #[test]
    fn test_add_custom_theme() {
        let mut db = ThemeDatabase::new();
        let custom_genre = ThemeGenre::Custom("test".to_string());
        let custom_pair = ThemePair::new("A".to_string(), "B".to_string());

        db.add_custom_theme(custom_genre.clone(), custom_pair);
        let theme = db.get_random_theme(&custom_genre);
        assert!(theme.is_some());
    }
}
