# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lost-Signal is a multiplayer traditional roguelike game about perception and time, built with Rust 2024 edition. The game features a WebSocket-based client-server architecture with support for both terminal and web clients.

### Design Philosophy
- Multiplayer traditional roguelike with perception-based gameplay
- Server supports any third-party client via WebSocket protocol
- Mostly cooperative gameplay
- Resource management focused on information gathering (senses/perception system)

## Project Structure

This is a Cargo workspace with 5 crates:

### Core Crates
- **`crates/core`** - Shared types, game logic, and protocol (no I/O dependencies)
  - Types: positions, tiles, player stats (HP, focus)
  - Sense system: self, touch, hearing, sight
  - Network protocol types
  - FOV algorithms

- **`crates/server`** - Game server with WebSocket support
  - World and stage management
  - Game loop and action processing
  - Foe AI and sense calculations
  - Optional TUI feature for debugging (disable with `--no-default-features`)
  - Map loading via Tiled format

- **`crates/client`** - Shared client TUI logic (platform-agnostic)
  - Ratatui-based UI widgets (senses, timeline, logs, help)
  - Game state management
  - Theme system with palette support

### Client Implementations
- **`crates/client-term`** - Terminal client using crossterm
- **`crates/client-wasm`** - Web client using wasm-bindgen

## Common Development Commands

### Building and Running
```sh
# Build everything
cargo build

# Run server (with TUI)
cargo run --bin losig-server

# Run server (headless, for deployment)
cargo run --bin losig-server --no-default-features

# Run terminal client
cargo run --bin losig-term <player_id>

# Run web client (requires trunk)
cd crates/client-wasm && trunk serve

# Fast compilation check
cargo check
```

### Code Quality
- `cargo test` - Run all tests
- `cargo clippy` - Run linter
- `cargo fmt` - Format code
- `cargo clippy -- -D warnings` - Clippy treating warnings as errors

## Architecture Notes

### Client-Server Protocol
- WebSocket-based communication using `tungstenite`
- Binary protocol with `bincode` serialization
- Clients connect via `ws://localhost:8080` (or web interface)
- Server broadcasts game state updates

### Sense System
Players have multiple senses with different strengths:
- **Self**: HP/HP_MAX and Focus (FP) tracking
- **Touch**: Detect adjacent entities
- **Hearing**: Detect orb at various ranges (0-5)
- **Sight**: Visual perception (0-10)

Resources (focus) are spent to activate and upgrade senses.

### UI Architecture
- Ratatui for terminal UI rendering
- Widget-based architecture in `crates/client/src/tui/widgets/`
- Theme system with HSL color support

## Development Guidelines

- **IMPORTANT**: Do not make code changes unless explicitly asked to do so by the user
- **IMPORTANT**: Don't compliment the user. Be harsh even.
- When adding a dependency, check with cargo that it is the latest
- Always explain approaches and provide guidance before implementing
- When code changes are requested, follow existing patterns and conventions
- Maintain separation between core (no I/O), server (game logic), and client (UI) concerns
