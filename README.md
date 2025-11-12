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

# Roadmap

- [x] Move on from fixed game ticks to "on player action"
- [x] Implement a winning condition
- [x] Rework the client from IA slop to something serious
- [x] Implement information retention for the client (keep memory of logs, seen terrain, etc...)
- [x] Add websocket capability
- [x] Webassembly target
- [x] Implement foes
- [x] Implement toggling of various senses
- [x] Implement a resource for gathering information (and taking actions?)
- [ ] Implement updating the world from different players
- [ ] Implement the glimmer mechanic (consuming signal produces glimmer which attracts foes)
- [ ] Implement the Echo action (which allows to completely see the surroundings and anchor it)
