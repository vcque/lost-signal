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

# Road to demo

- [ ] Dev mode: have a watcher restart either the server or client on code change
- [ ] Maps: have at least 10 maps, rotate between them
- [ ] Big win: to win, you must complete the 10 maps. Respawning take you back one map
- [ ] Leaderboard: show (and persist ?) users who have won the game
- [ ] Help: have a help section to explain the goal and controls 
- [ ] Rice: make it somewhat less ugly
- [ ] Desyncs: handle discrepancies when data returned from server does not match the cache (mostly terrain data)
