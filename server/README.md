# Bevy Game Server

## Requirements
- [Rust stable 1.64+](https://www.rustlang.org)
- [Cargo watch](https://lib.rs/crates/cargo-watch)

## Development
1. Run the dev watcher with `cargo watch -x run`

## Documentation
1. Build the internal documentation with `cargo doc --no-deps --document-private-items`
2. Open `target/doc/bevy_game_server/index.html` in a web browser

## Building
1. Build the application with `cargo build --release`
2. Deploy the `bevy-game-server` binary in `target/release` along with any adjacent dll/lib files
