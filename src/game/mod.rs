// game/mod.rs - Public API for the game module
//
// 🎓 The mod.rs Pattern
// This file controls what's visible outside the `game` module.
// It's like a gatekeeper - only things marked `pub use` are accessible.

pub mod state;
pub mod player;
pub mod theme;
pub mod rules;

// Re-export commonly used types for convenience
// Now users can write `game::GameState` instead of `game::state::GameState`
pub use state::GameState;
pub use player::Player;
pub use theme::{ThemeDatabase, ThemePair};
pub use rules::{Vote, VoteResult};
