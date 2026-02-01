// rooms/mod.rs - Public API for the rooms module

pub mod room;
pub mod manager;

pub use room::Room;
pub use manager::{RoomManager, SharedRooms};
