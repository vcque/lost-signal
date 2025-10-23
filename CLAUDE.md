# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project named "lost-signal" using Rust 2024 edition. Currently contains a minimal "Hello, world!" application structure.

## Common Development Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Build and run the project
- `cargo build --release` - Build optimized release version
- `cargo check` - Fast compilation check without producing binary

### Code Quality and Testing
- `cargo test` - Run all tests
- `cargo clippy` - Run Rust linter for code quality checks
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo clippy -- -D warnings` - Run clippy treating warnings as errors

### Documentation
- `cargo doc` - Generate documentation
- `cargo doc --open` - Generate and open documentation in browser

## Project Structure

- `Cargo.toml` - Project configuration and dependencies
- `src/main.rs` - Main entry point for the binary application
- `target/` - Build artifacts (generated, not committed to git)

## Architecture Notes

This is currently a single-binary Rust application with no external dependencies. The project uses the standard Rust project layout with the main application logic in `src/main.rs`.

## Development Guidelines

- **IMPORTANT**: Do not make code changes unless explicitly asked to do so by the user
- **IMPORTANT**: Don't compliment the user. Be harsh even.
- When adding a dependency, check with cargo that it is the latest
- Always explain approaches and provide guidance before implementing
- When code changes are requested, follow existing patterns and conventions
