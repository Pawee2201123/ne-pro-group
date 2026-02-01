# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a fully functional "Word Wolf" (ワードウルフ) multiplayer web game using a custom Rust backend and vanilla JavaScript frontend. The game is a social deduction game where players receive related themes and must identify who has the different word (the "wolf").

**Project Status:** ✅ Complete and functional

The project features:
- A custom Rust HTTP/SSE server (no external web frameworks - pure stdlib)
- Real-time multiplayer using Server-Sent Events (SSE)
- 7 fully integrated HTML pages with JavaScript
- Complete game logic with voting, role assignment, and winner determination
- ~2,600 lines of Rust code with 35 passing tests
- Nix flake for reproducible development environment

## Development Environment

This project uses Nix flakes for reproducible development environments:

```bash
# Enter development environment
nix develop

# The flake provides:
# - Rust stable toolchain with rust-src and rust-analyzer
# - pkg-config and openssl for common Rust dependencies
```

## Build and Run Commands

```bash
# Build the project
cargo build

# Run in development
cargo run

# Build for release
cargo build --release

# Check code without building
cargo check

# Run tests
cargo test

# Run a single test
cargo test test_name
```

## Architecture

### Backend Structure (Layered Architecture)

```
src/
├── main.rs              # TCP server entry point
├── types.rs             # Shared types (Role, ThemeGenre, RoomConfig)
├── game/                # Domain layer - Pure game logic
│   ├── mod.rs
│   ├── player.rs        # Player state management
│   ├── state.rs         # Game state machine (FSM)
│   ├── theme.rs         # Theme database
│   └── rules.rs         # Game rules (voting, winner determination)
├── rooms/               # Service layer - Room management
│   ├── mod.rs
│   ├── room.rs          # Single room lifecycle
│   └── manager.rs       # Multi-room coordination (Arc<Mutex<>>)
└── network/             # Network layer - HTTP/SSE
    ├── mod.rs
    ├── http.rs          # HTTP request/response parsing
    ├── sse.rs           # Server-Sent Events implementation
    └── handlers.rs      # 10+ HTTP endpoints
```

**Key Technologies:**
- Custom HTTP server using `std::net::TcpListener`
- Server-Sent Events (SSE) for real-time updates
- Thread-safe state management with `Arc<Mutex<T>>`
- Zero external dependencies (pure standard library)

### Frontend Structure

The game follows a sequential flow through HTML pages:

1. **login.html** - Player name entry → localStorage
2. **home.html** - Main menu (create/join room)
3. **room_create.html** - Room creation with settings → API calls
4. **room_join.html** - Join room by ID → API
5. **stay.html** - Waiting room with SSE real-time updates
6. **theme.html** - Theme/role display phase
7. **game.html** - Discussion and voting phase

All pages include vanilla JavaScript for API communication and SSE handling.

## API Endpoints

The backend provides the following HTTP endpoints:

- `GET /` - Serve login.html
- `GET /*.html` - Serve static HTML pages
- `POST /room/create` - Create a new room
- `POST /room/join` - Join an existing room
- `POST /room/ready` - Mark player as ready
- `POST /room/theme/confirm` - Confirm theme viewing
- `POST /room/vote` - Submit a vote
- `POST /room/chat` - Send chat message (if implemented)
- `GET /room/list` - List all rooms
- `GET /room/state?room_id=X` - Get room state
- `GET /room/players?room_id=X` - Get player list
- `GET /player/theme?room_id=X&player_id=Y` - Get player's theme
- `GET /events?room_id=X` - SSE connection for real-time updates

## Testing

The project includes 35 unit tests covering:
- Game rules (voting, winner determination, role assignment)
- Player state management
- Room management
- State machine transitions
- HTTP parsing
- URL decoding (UTF-8 support for Japanese)

Run tests with:
```bash
cargo test
```

## Project Notes

- The project uses Japanese for UI text
- All code comments and documentation are in Japanese and English
- No external dependencies - pure Rust standard library
- Educational/learning project demonstrating:
  - Custom HTTP server implementation
  - Concurrency with Arc<Mutex<T>>
  - State machine pattern
  - Domain-driven design
  - SSE for real-time communication
