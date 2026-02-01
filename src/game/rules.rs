// game/rules.rs - Game rules and winner calculation
//
// 🎓 Key Concepts:
// - HashMap for counting votes
// - Iterator methods (filter, max_by_key, etc.)
// - Pure functions for testability

use crate::game::Player;
use crate::types::{PlayerId, Role};
use std::collections::HashMap;

/// Represents a single vote
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: PlayerId,
    pub target: PlayerId,
}

/// Result of vote tallying
#[derive(Debug, Clone)]
pub struct VoteResult {
    /// Player who received the most votes
    pub eliminated_player: PlayerId,
    /// Number of votes they received
    pub vote_count: usize,
    /// Full vote breakdown
    pub vote_breakdown: HashMap<PlayerId, usize>,
}

/// 🎓 Pure function: Tally votes and find who should be eliminated
/// This function has NO side effects - it just calculates!
/// Easy to test because it's predictable
pub fn tally_votes(votes: &[Vote]) -> Option<VoteResult> {
    if votes.is_empty() {
        return None;
    }

    // 🎓 HashMap to count votes per player
    let mut vote_counts: HashMap<PlayerId, usize> = HashMap::new();

    // Count votes
    for vote in votes {
        *vote_counts.entry(vote.target.clone()).or_insert(0) += 1;
    }

    // 🎓 Iterator method: Find the player with most votes
    // max_by_key takes a closure that extracts the comparison key
    let (eliminated_player, vote_count) = vote_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(player, count)| (player.clone(), *count))?;

    Some(VoteResult {
        eliminated_player,
        vote_count,
        vote_breakdown: vote_counts,
    })
}

/// 🎓 Pure function: Determine if citizens won
/// Citizens win if all wolves are eliminated
pub fn check_citizen_victory(players: &[Player]) -> bool {
    // 🎓 Iterator method: filter and all
    // "Do all wolves have active=false?"
    players
        .iter()
        .filter(|p| p.is_wolf())
        .all(|p| !p.is_active())
}

/// 🎓 Pure function: Determine if wolves won
/// Wolves win if they equal or outnumber active citizens
pub fn check_wolf_victory(players: &[Player]) -> bool {
    let active_citizens = players
        .iter()
        .filter(|p| p.is_citizen() && p.is_active())
        .count();

    let active_wolves = players
        .iter()
        .filter(|p| p.is_wolf() && p.is_active())
        .count();

    // Wolves win if they equal or outnumber citizens
    active_wolves > 0 && active_wolves >= active_citizens
}

/// 🎓 Pure function: Check if game is over
pub fn is_game_over(players: &[Player]) -> Option<bool> {
    if check_citizen_victory(players) {
        Some(true) // Citizens won
    } else if check_wolf_victory(players) {
        Some(false) // Wolves won
    } else {
        None // Game continues
    }
}

/// 🎓 Pure function: Assign roles to players
/// Takes a mutable slice and assigns roles randomly
/// Returns the indices of wolf players
pub fn assign_roles(players: &mut [Player], wolf_count: usize) -> Vec<PlayerId> {
    use std::collections::HashSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    if wolf_count >= players.len() {
        panic!("Wolf count must be less than player count");
    }

    let mut wolf_indices = HashSet::new();
    let player_count = players.len();

    // 🎓 Simple random selection
    // In production, use the rand crate!
    while wolf_indices.len() < wolf_count {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let index = (nanos as usize) % player_count;
        wolf_indices.insert(index);
    }

    // Assign roles
    let mut wolf_ids = Vec::new();
    for (i, player) in players.iter_mut().enumerate() {
        if wolf_indices.contains(&i) {
            player.assign_role(Role::Wolf);
            wolf_ids.push(player.id().clone());
        } else {
            player.assign_role(Role::Citizen);
        }
    }

    wolf_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tally_votes() {
        let votes = vec![
            Vote {
                voter: "p1".to_string(),
                target: "p3".to_string(),
            },
            Vote {
                voter: "p2".to_string(),
                target: "p3".to_string(),
            },
            Vote {
                voter: "p3".to_string(),
                target: "p1".to_string(),
            },
        ];

        let result = tally_votes(&votes).unwrap();
        assert_eq!(result.eliminated_player, "p3");
        assert_eq!(result.vote_count, 2);
    }

    #[test]
    fn test_citizen_victory() {
        let mut players = vec![
            Player::new("p1".to_string(), "Alice".to_string()),
            Player::new("p2".to_string(), "Bob".to_string()),
            Player::new("p3".to_string(), "Charlie".to_string()),
        ];

        players[0].assign_role(Role::Citizen);
        players[1].assign_role(Role::Citizen);
        players[2].assign_role(Role::Wolf);

        // Wolf is still active
        assert!(!check_citizen_victory(&players));

        // Eliminate the wolf
        players[2].eliminate();
        assert!(check_citizen_victory(&players));
    }

    #[test]
    fn test_wolf_victory() {
        let mut players = vec![
            Player::new("p1".to_string(), "Alice".to_string()),
            Player::new("p2".to_string(), "Bob".to_string()),
            Player::new("p3".to_string(), "Charlie".to_string()),
        ];

        players[0].assign_role(Role::Citizen);
        players[1].assign_role(Role::Citizen);
        players[2].assign_role(Role::Wolf);

        // Initially, citizens outnumber wolves
        assert!(!check_wolf_victory(&players));

        // Eliminate one citizen
        players[0].eliminate();
        // Now it's 1 citizen vs 1 wolf - wolves win!
        assert!(check_wolf_victory(&players));
    }

    #[test]
    fn test_assign_roles() {
        let mut players = vec![
            Player::new("p1".to_string(), "Alice".to_string()),
            Player::new("p2".to_string(), "Bob".to_string()),
            Player::new("p3".to_string(), "Charlie".to_string()),
            Player::new("p4".to_string(), "David".to_string()),
        ];

        let wolf_ids = assign_roles(&mut players, 1);

        // Exactly 1 wolf
        assert_eq!(wolf_ids.len(), 1);

        // Count roles
        let wolf_count = players.iter().filter(|p| p.is_wolf()).count();
        let citizen_count = players.iter().filter(|p| p.is_citizen()).count();

        assert_eq!(wolf_count, 1);
        assert_eq!(citizen_count, 3);
    }
}
