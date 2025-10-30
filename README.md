Lost-Signal is a little would-be multiplayer (trad?) roguelike.

# Launching it

server:
```sh
cargo run --bin server
```

client:
```sh
cargo run --bin client $player_id
```

# Design goals

The main goal is to make traditional roguelike (turn-based) and multiplayer (async) work. 

To do this, the "reality" (or game state) would be flexible by:
- allowing updates in the past (from late players) to happen, potentially invalidating actions from other players. 
- allowing players to pin reality to their current state, forcing other players and foes to conform to this pinned reality.

Another goal is to have the real game only be the server. The client should not have access to any information on the game that its player do not have.

# "Roadmap"

- [x] Move on from fixed game ticks to "on player action"
- [x] Implement a winning condition
- [x] Rework the client from IA slop to something serious
- [x] Implement information retention for the client (keep memory of logs, seen terrain, etc...)
- [x] Add websocket capability
- [x] Webassembly target
- [ ] Implement foes
- [ ] Implement toggling of various senses
- [ ] Implement a resource for gathering information (and taking actions?)
- [ ] Implement updating the world from different players
- [ ] Implement the Echo action
