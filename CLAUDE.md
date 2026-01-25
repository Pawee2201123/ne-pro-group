# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a "Word Wolf" (ワードウルフ) web game project using Rust and HTML. The game is a social deduction game where players receive related themes and must identify who has the different word.

The project is in early development with:
- A Rust backend (currently minimal, just prints "Hello, world!")
- HTML frontend pages for the game flow
- Nix flake for development environment management

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

### Frontend Structure

The game follows a sequential flow through HTML pages:

1. **login.html** - Player name entry
2. **home.html** - Main menu (create/join room)
3. **room_create.html** - Room creation with settings (room name, player count, wolf count, theme genre)
4. **room_join.html** - Join room by ID
5. **stay.html** - Waiting room for other players
6. **theme.html** - Theme submission phase
7. **game.html** - Main game/deduction phase

All HTML files are currently standalone without CSS styling linked (class references exist but no stylesheet).

### Backend Structure

- **src/main.rs** - Entry point (currently placeholder)
- **Cargo.toml** - Uses Rust edition 2024, no dependencies yet

### Missing Components

The following need to be implemented:
- Backend game logic (room management, player state, theme assignment, wolf role assignment)
- WebSocket or polling for real-time multiplayer
- Frontend JavaScript for navigation and backend communication
- CSS styling (referenced but not present)
- Database or state management for rooms and players

## Project Notes

- The project uses Japanese for UI text
- There's a comment in home.html requesting a game name suggestion
- The Chat/ directory contains only a placeholder temp file
- No test files exist yet
- No CI/CD configuration present
