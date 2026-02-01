// rooms/manager.rs - Manage multiple game rooms concurrently
//
// 🎓 Key Concepts:
// - Arc<Mutex<T>> for thread-safe shared state
// - Concurrent access from multiple threads
// - Interior mutability pattern

use crate::rooms::Room;
use crate::types::{RoomId, RoomConfig};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 🎓 Type alias for our shared room storage
///
/// Breaking it down:
/// - HashMap<RoomId, Room>  = The actual data (rooms by ID)
/// - Mutex<...>             = Only one thread can access at a time
/// - Arc<...>               = Multiple threads can own references to it
///
/// This is called the "Interior Mutability" pattern in Rust
pub type SharedRooms = Arc<Mutex<HashMap<RoomId, Room>>>;

/// Manager for all game rooms
///
/// 🎓 Note: This struct is just a wrapper around SharedRooms
/// The real magic is in the Arc<Mutex<>> type!
#[derive(Clone)]
pub struct RoomManager {
    rooms: SharedRooms,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        RoomManager {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new room
    ///
    /// 🎓 Watch how we use the Mutex:
    /// 1. Lock the mutex (blocks if another thread has it)
    /// 2. Get mutable access to the HashMap
    /// 3. Modify it
    /// 4. Lock is automatically released when we return
    pub fn create_room(&self, room_id: RoomId, config: RoomConfig) -> Result<(), String> {
        // 🎓 Lock the mutex - this gives us exclusive access
        // The lock is automatically released when `rooms` goes out of scope
        let mut rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in create_room, recovering...");
            poisoned.into_inner()
        });

        // Check if room already exists
        if rooms.contains_key(&room_id) {
            return Err(format!("Room {} already exists", room_id));
        }

        // Create the room
        let room = Room::new(room_id.clone(), config)?;

        // Insert into HashMap
        rooms.insert(room_id, room);

        Ok(())
    }

    /// Get a room by ID (for read-only operations)
    ///
    /// 🎓 Problem: We can't return a reference to the Room because
    /// the Mutex lock would be released when this function returns!
    ///
    /// Solution: Return a clone of the room ID list, or perform
    /// the operation inside this function
    pub fn room_exists(&self, room_id: &RoomId) -> bool {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in room_exists, recovering...");
            poisoned.into_inner()
        });
        rooms.contains_key(room_id)
    }

    /// Get count of players in a room
    pub fn get_player_count(&self, room_id: &RoomId) -> Option<usize> {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in get_player_count, recovering...");
            poisoned.into_inner()
        });
        rooms.get(room_id).map(|room| room.player_count())
    }

    /// Check if a room is full
    pub fn is_room_full(&self, room_id: &RoomId) -> Option<bool> {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in is_room_full, recovering...");
            poisoned.into_inner()
        });
        rooms.get(room_id).map(|room| room.is_full())
    }

    /// Get a snapshot of room state (as JSON-like string)
    pub fn get_room_state(&self, room_id: &RoomId) -> Option<String> {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in get_room_state, recovering...");
            poisoned.into_inner()
        });
        rooms.get(room_id).map(|room| room.get_state_snapshot())
    }

    /// List all room IDs
    pub fn list_rooms(&self) -> Vec<RoomId> {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in list_rooms, recovering...");
            poisoned.into_inner()
        });
        rooms.keys().cloned().collect()
    }

    /// Delete a room
    pub fn delete_room(&self, room_id: &RoomId) -> Result<(), String> {
        let mut rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in delete_room, recovering...");
            poisoned.into_inner()
        });

        if rooms.remove(room_id).is_none() {
            return Err(format!("Room {} not found", room_id));
        }

        Ok(())
    }

    /// 🎓 Advanced: Execute an operation on a room
    ///
    /// This uses a closure (function as parameter) to perform
    /// any operation on a room while holding the lock
    ///
    /// Why? Because we can't return a mutable reference to a Room
    /// (the lock would be released), so we pass in the operation instead!
    pub fn with_room<F, R>(&self, room_id: &RoomId, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut Room) -> Result<R, String>,
    {
        // Handle poison error gracefully
        let mut rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            // If the mutex is poisoned, we can still access the data
            // but we should log this situation
            eprintln!("Warning: Mutex was poisoned, recovering...");
            poisoned.into_inner()
        });

        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| format!("Room {} not found", room_id))?;

        f(room)
    }

    /// Get the total number of rooms
    pub fn room_count(&self) -> usize {
        let rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in room_count, recovering...");
            poisoned.into_inner()
        });
        rooms.len()
    }

    /// Check all rooms for expired discussion timers and auto-start voting
    /// Called periodically by background timer thread
    pub fn check_all_timers(&self) {
        let mut rooms = self.rooms.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: Mutex was poisoned in check_all_timers, recovering...");
            poisoned.into_inner()
        });

        for room in rooms.values_mut() {
            room.check_and_auto_vote();
        }
    }
}

/// 🎓 Default trait implementation
impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ThemeGenre;
    use crate::game::Player;

    fn create_test_config() -> RoomConfig {
        RoomConfig::new(
            "Test Room".to_string(),
            4,
            1,
            ThemeGenre::Food,
            180,
        )
    }

    #[test]
    fn test_create_room() {
        let manager = RoomManager::new();
        let config = create_test_config();

        assert!(manager.create_room("room1".to_string(), config).is_ok());
        assert_eq!(manager.room_count(), 1);
    }

    #[test]
    fn test_duplicate_room() {
        let manager = RoomManager::new();
        let config = create_test_config();

        manager.create_room("room1".to_string(), config.clone()).unwrap();

        // Try to create again
        let result = manager.create_room("room1".to_string(), config);
        assert!(result.is_err());
    }

    #[test]
    fn test_room_exists() {
        let manager = RoomManager::new();
        let config = create_test_config();

        assert!(!manager.room_exists(&"room1".to_string()));

        manager.create_room("room1".to_string(), config).unwrap();

        assert!(manager.room_exists(&"room1".to_string()));
    }

    #[test]
    fn test_list_rooms() {
        let manager = RoomManager::new();
        let config = create_test_config();

        manager.create_room("room1".to_string(), config.clone()).unwrap();
        manager.create_room("room2".to_string(), config.clone()).unwrap();

        let rooms = manager.list_rooms();
        assert_eq!(rooms.len(), 2);
        assert!(rooms.contains(&"room1".to_string()));
        assert!(rooms.contains(&"room2".to_string()));
    }

    #[test]
    fn test_delete_room() {
        let manager = RoomManager::new();
        let config = create_test_config();

        manager.create_room("room1".to_string(), config).unwrap();
        assert_eq!(manager.room_count(), 1);

        manager.delete_room(&"room1".to_string()).unwrap();
        assert_eq!(manager.room_count(), 0);
    }

    #[test]
    fn test_with_room() {
        let manager = RoomManager::new();
        let config = create_test_config();

        manager.create_room("room1".to_string(), config).unwrap();

        // Use with_room to add a player
        let result = manager.with_room(&"room1".to_string(), |room| {
            let player = Player::new("p1".to_string(), "Alice".to_string());
            room.add_player(player)
        });

        assert!(result.is_ok());

        // Check player count
        assert_eq!(manager.get_player_count(&"room1".to_string()), Some(1));
    }

    #[test]
    fn test_clone_manager() {
        // 🎓 This tests that Arc works - we can clone the manager
        // and both clones point to the SAME underlying data
        let manager1 = RoomManager::new();
        let manager2 = manager1.clone();  // Clone the Arc, not the data!

        let config = create_test_config();
        manager1.create_room("room1".to_string(), config).unwrap();

        // Both managers see the same room!
        assert_eq!(manager1.room_count(), 1);
        assert_eq!(manager2.room_count(), 1);
    }
}
