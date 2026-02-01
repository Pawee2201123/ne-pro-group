# 📋 Word Wolf - Code Cheat Sheet

---

## 🏗️ Architecture Overview

```
src/
├── types.rs          # Foundation types (shared everywhere)
├── game/             # Pure game logic (no I/O)
│   ├── mod.rs
│   ├── player.rs
│   ├── state.rs
│   ├── theme.rs
│   └── rules.rs
└── rooms/            # Room management (coordinates game + I/O)
    ├── mod.rs
    └── room.rs
```

---

## 📄 File-by-File Breakdown

### **`src/types.rs`** [~100 lines]

**Purpose:** Define foundational types used throughout the app

**Exports:**
| Type | What It Is | Example |
|------|-----------|---------|
| `PlayerId` | String alias for player IDs | `"player123"` |
| `RoomId` | String alias for room IDs | `"room456"` |
| `Role` | Enum: Citizen or Wolf | `Role::Wolf` |
| `ThemeGenre` | Enum: Food/Animal/Place/Object/Custom | `ThemeGenre::Food` |
| `RoomConfig` | Struct with room settings | max_players, wolf_count, etc. |

**Key Methods:**
- `RoomConfig::new()` - Create config
- `RoomConfig::validate()` - Check if config is valid

**Rust Concepts:**
- Type aliases (`type PlayerId = String`)
- Enums with variants
- Struct with validation
- `Result<T, E>` for error handling
- `#[derive(Debug, Clone, ...)]` macros

**Dependencies:** None (foundation layer)

---

### **`src/game/player.rs`** [~130 lines]

**Purpose:** Represent a single player with encapsulation

**Exports:**
| Type/Function | What It Is |
|---------------|-----------|
| `Player` | Struct representing a player |

**Key Methods:**
```rust
Player::new(id, name)           // Create new player
player.assign_role(role)        // Give them Citizen/Wolf role
player.assign_theme(theme)      // Give them a theme word
player.is_wolf()                // Check if they're a wolf
player.is_ready_to_play()       // Has role + theme assigned?
player.eliminate()              // Vote them out
```

**Rust Concepts:**
- **Encapsulation** - Private fields, public methods
- `Option<T>` for optional data (role, theme)
- Getter methods (`&self`) vs setter methods (`&mut self`)
- Custom `Display` trait for pretty printing

**Dependencies:** `types.rs` (uses `PlayerId`, `Role`)

---

### **`src/game/state.rs`** [~200 lines]

**Purpose:** State machine for game phases (THE HEART OF THE GAME)

**Exports:**
| Type/Function | What It Is |
|---------------|-----------|
| `GameState` | Enum with 5 variants (states) |

**State Flow:**
```
Lobby → ThemeSubmission → Discussion → Voting → Finished
```

**Each State Carries Data:**
```rust
enum GameState {
    Lobby { ready_players: HashSet<PlayerId> },
    ThemeSubmission { confirmed_players: HashSet<PlayerId> },
    Discussion { time_remaining: Option<u32> },
    Voting { voted_players: HashSet<PlayerId> },
    Finished { citizens_won: bool, wolves: Vec<PlayerId> },
}
```

**Key Methods:**
```rust
GameState::new()                          // Start in Lobby
state.transition_to_theme_submission()    // Move to next phase
state.mark_player_ready(player_id)        // Track ready players
state.all_players_ready(total)            // Check if ready to start
```

**Rust Concepts:**
- **Enums with associated data** (each variant holds different data)
- **State pattern** implemented via enums
- `matches!()` macro for pattern matching
- State transitions with validation

**Dependencies:** `types.rs` (uses `PlayerId`)

---

### **`src/game/theme.rs`** [~180 lines]

**Purpose:** Theme database and random selection

**Exports:**
| Type/Function | What It Is |
|---------------|-----------|
| `ThemePair` | Struct: citizen_theme + wolf_theme |
| `ThemeDatabase` | Struct: HashMap of themes by genre |

**Key Methods:**
```rust
ThemeDatabase::new()                // Create with pre-loaded themes
db.get_random_theme(&genre)         // Get random pair for genre
db.add_custom_theme(genre, pair)    // Add custom theme
```

**Pre-loaded Themes:**
- Food: りんご/みかん, カレー/シチュー, etc.
- Animal: 犬/猫, ライオン/トラ, etc.
- Place: 海/山, 図書館/書店, etc.
- Object: 鉛筆/ペン, 椅子/ソファ, etc.

**Rust Concepts:**
- `HashMap<K, V>` for key-value storage
- Pure functions (no side effects)
- `Option<T>` return type (theme might not exist)
- `Default` trait implementation

**Dependencies:** `types.rs` (uses `ThemeGenre`)

---

### **`src/game/rules.rs`** [~230 lines]

**Purpose:** Pure game logic functions (voting, victory conditions)

**Exports:**
| Type/Function | What It Is |
|---------------|-----------|
| `Vote` | Struct: voter → target |
| `VoteResult` | Struct: eliminated player + vote count |

**Key Functions (all pure!):**
```rust
tally_votes(&votes)                  // Count votes, find who's eliminated
check_citizen_victory(&players)      // All wolves eliminated?
check_wolf_victory(&players)         // Wolves >= citizens?
is_game_over(&players)              // Return Some(bool) if game over
assign_roles(&mut players, count)    // Randomly assign wolf roles
```

**Rust Concepts:**
- **Pure functions** - No I/O, just calculations
- **Iterator methods** - `.filter()`, `.max_by_key()`, `.collect()`
- Borrowing (`&[Vote]`) vs ownership
- Functional programming style

**Dependencies:** `types.rs`, `game/player.rs`

---

### **`src/game/mod.rs`** [~15 lines]

**Purpose:** Game module's public API (facade pattern)

**What it does:**
```rust
pub mod state;
pub mod player;
pub mod theme;
pub mod rules;

// Re-export for convenience
pub use state::GameState;
pub use player::Player;
pub use theme::{ThemeDatabase, ThemePair};
pub use rules::{Vote, VoteResult};
```

**Rust Concepts:**
- Module system (`mod`, `pub mod`)
- Re-exports (`pub use`)
- Facade pattern (clean public API)

---

### **`src/rooms/room.rs`** [~300 lines]

**Purpose:** A single game room - brings EVERYTHING together

**Exports:**
| Type/Function | What It Is |
|---------------|-----------|
| `Room` | Struct managing one game room |
| `Senders` | Type alias for SSE connections |

**What Room Contains:**
```rust
struct Room {
    id: RoomId,                          // Unique room ID
    config: RoomConfig,                  // Settings
    state: GameState,                    // Current phase
    players: HashMap<PlayerId, Player>,  // All players
    senders: Vec<mpsc::Sender<String>>,  // SSE connections
    votes: HashMap<PlayerId, PlayerId>,  // Current votes
}
```

**Key Methods:**
```rust
Room::new(id, config)              // Create room
room.add_player(player)            // Player joins
room.mark_ready(player_id)         // Player ready
room.confirm_theme(player_id)      // Saw their theme
room.submit_vote(voter, target)    // Vote for elimination
room.broadcast(message)            // Send to all clients via SSE
```

**Game Flow in Room:**
1. Players join → 2. Mark ready → 3. Auto-start game → 4. Assign roles/themes →
5. Confirm themes → 6. Discussion → 7. Voting → 8. Check victory → 9. Repeat or finish

**Rust Concepts:**
- **Composition** - Room contains Player, GameState, etc.
- **Orchestration** - Coordinates all game components
- `mpsc::Sender` for message passing
- State management
- Error propagation (`?` operator)

**Dependencies:** Everything! (types, game/*, HashMap, mpsc)

---

### **`src/rooms/mod.rs`** [~5 lines]

**Purpose:** Rooms module's public API

**What it does:**
```rust
pub mod room;
pub use room::Room;
```

---

## 🎯 How Everything Connects

```
User Action             →  Room Method           →  Uses
────────────────────────────────────────────────────────────
"Create room"          →  Room::new()            →  RoomConfig::validate()
"Join room"            →  room.add_player()      →  Player::new()
"Ready up"             →  room.mark_ready()      →  GameState::mark_player_ready()
[All ready]            →  room.start_game()      →  rules::assign_roles()
                                                  →  ThemeDatabase::get_random_theme()
                                                  →  Player::assign_role/theme()
"Confirm theme"        →  room.confirm_theme()   →  GameState::confirm_theme()
[Discussion happens]
"Start vote"           →  room.start_voting()    →  GameState::transition_to_voting()
"Submit vote"          →  room.submit_vote()     →  GameState::record_vote()
[All voted]            →  room.tally_votes()     →  rules::tally_votes()
                                                  →  rules::is_game_over()
                                                  →  Player::eliminate()
```

---

## 🧪 Testing Status

**Total:** 23 tests, all passing ✅

| Module | Tests |
|--------|-------|
| types.rs | 3 tests (validation) |
| game/player.rs | 5 tests (creation, roles, themes) |
| game/state.rs | 3 tests (transitions, flow) |
| game/theme.rs | 5 tests (database, selection) |
| game/rules.rs | 4 tests (voting, victory) |
| rooms/room.rs | 3 tests (creation, players) |

---

## 🔑 Key Rust Concepts Map

| Concept | Where to See It | Line(s) |
|---------|----------------|---------|
| **Enums with data** | `game/state.rs` | Lines 17-51 |
| **Type aliases** | `types.rs` | Lines 13-14 |
| **Encapsulation** | `game/player.rs` | Lines 14-28 (private fields) |
| **Option<T>** | `game/player.rs` | Lines 21-26 |
| **Result<T, E>** | `types.rs` | Lines 66-76 |
| **Pure functions** | `game/rules.rs` | Lines 31-57 |
| **Iterator methods** | `game/rules.rs` | Lines 48-51 |
| **Pattern matching** | `game/state.rs` | Lines 85-94 |
| **HashMap** | `rooms/room.rs` | Line 28 |
| **mpsc channels** | `rooms/room.rs` | Lines 32, 269-272 |
| **Borrowing (&)** | Everywhere | All getters |
| **Mutable refs (&mut)** | Everywhere | All setters |

---

## 📦 What's Left To Build

1. **`rooms/manager.rs`** - Manage multiple rooms with Arc<Mutex<HashMap>>
2. **`network/` module** - SSE + HTTP handling (based on Chat/main.rs)
3. **`main.rs`** - TCP listener, wire everything together

---

## 🚀 Quick Reference Commands

```bash
# Build project
nix develop --command cargo build

# Run tests
nix develop --command cargo test

# Run server (when complete)
nix develop --command cargo run

# Check without building
nix develop --command cargo check
```

---

## 🎨 Design Patterns Used

### **Primary: Layered Architecture (3-Tier)**
- Network Layer (HTTP/SSE) - not built yet
- Service Layer (rooms/)
- Domain Layer (game/, types.rs)

### **Specific Patterns:**
1. **State Pattern** - `game/state.rs` (GameState enum)
2. **Domain-Driven Design** - Organized by business domains
3. **Functional Core, Imperative Shell** - Pure logic in game/, I/O in rooms/
4. **Type-Driven Design** - Use types to prevent invalid states
5. **Repository Pattern** - `ThemeDatabase`
6. **Observer Pattern** - SSE broadcasting via channels
7. **Facade Pattern** - `game/mod.rs`, `rooms/mod.rs`

---

**End of Cheat Sheet** 📋
