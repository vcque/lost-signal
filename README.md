![CI](https://github.com/vcque/lost-signal/workflows/CI/badge.svg)

Lost-Signal is a little would-be multiplayer (trad?) roguelike about perception and time.

# Launching it

server:
```sh
cargo server
```

client:
```sh
cargo run --bin losig-term $player_id
```

web-client:
```sh
cd crates/client-wasm
trunk serve
```

# Design goals

The main goal is to make a working multiplayer traditional roguelike.

Also: 
- The server should support any third-party client
- Mostly cooperative
- Perception/infowar based. Resource is used for gathering information.
