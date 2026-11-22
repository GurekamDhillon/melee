# GD's Melee: native Rust engine

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="pc/docs/readme/brand/logo_horizontal_dark.png">
    <source media="(prefers-color-scheme: light)" srcset="pc/docs/readme/brand/logo_horizontal_light.png">
    <img alt="GD's Melee" src="pc/docs/readme/brand/logo_horizontal_light.png" width="600">
  </picture>
</p>

<p align="center">
  <img alt="Platform: Windows x64 and Linux x86_64" src="pc/docs/readme/brand/windows.svg" height="28">
  <img alt="Netplay: rollback" src="pc/docs/readme/brand/rollback.svg" height="28">
  <img alt="Replays: Slippi" src="pc/docs/readme/brand/slippi.svg" height="28">
  <a href="https://discord.gg/FU4KTGQS5m"><img alt="Join GD's Workshop on Discord" src="pc/docs/readme/brand/discord.svg" height="28"></a>
  <a href="rust/LICENSE"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/engine-GPL--3.0-3a3f4b"></a>
  <a href="docs/specs/LICENSE-MIT"><img alt="Specs: MIT" src="https://img.shields.io/badge/specs-MIT-3a3f4b"></a>
</p>

<p align="center">
  <a href="https://0rwx.com"><img alt="0rwx: AI-X, made with AI" src="pc/docs/readme/brand/0rwx-AI-X.png" width="120"></a><br>
  <b>AI level: <a href="https://0rwx.com">X (execute)</a>.</b> AI agents write most of this code, and no one reviews it line by line.
  It is checked by machine instead: against Slippi replays frame by frame, and against the console's own memory through a Dolphin oracle.
</p>

A native engine for *Super Smash Bros. Melee* (NTSC 1.02, `GALE01`), written in Rust from the
[doldecomp/melee](https://github.com/doldecomp/melee) decompilation. No emulator and no recompiled
binary: the game logic is ordinary Rust, and it reads its assets from **your own disc**.

## Status

| | |
|---|---|
| Replay accuracy | **12,012 of 12,265** Slippi replays (97.9%) match byte for byte, across 5 replay sets ([results](docs/rust-port/notes/replay-results.md)) |
| Rollback netplay | **0 desyncs** in the test matches |
| Slippi online | Unranked and Direct menus, following Slippi's own client. **In test**, not released |
| Smoothness | Gate: no frame over 16 ms caused by the game, in any tested scene |
| Speed | Faster on average than the other native engines we measured, on one machine. **Provisional**; 1% lows still trail ([bench-live](docs/rust-port/notes/bench-live.md)) |

This is a work in progress. Expect rough edges and report bugs on [Discord](https://discord.gg/FU4KTGQS5m) or in Issues.

## Features

- **Mods** as sandboxed WebAssembly: no file or network access, deterministic, safe for netplay.
  The mod API and specs are MIT, so anyone can build on them ([docs/specs](docs/specs/)).
- **Stage switches**: toggle stage hazards and variants.
- **Quick-swap**: change character in place with one button ([notes](docs/rust-port/notes/quick-swap.md)).
- **Envoy**: a single-player and party mode built on the engine.
- **Slippi replays**: plays `.slp` files and checks itself against them.

## Build and run

You need a retail NTSC 1.02 Melee disc image that you made from your own disc. This repo contains no game data.

**Windows** (Rust from [rustup](https://rustup.rs)):

```sh
git clone https://github.com/GurekamDhillon/melee
cd melee/rust
cargo build --release -p melee
```

**Linux**: install the system packages first (`bash tools/rust-port/linux/deps.sh`, or `NOSUDO=1` without sudo;
see [linux.md](docs/rust-port/notes/linux.md)), then run the same `cargo build`.

Point the engine at your disc with the `GW_ISO_VANILLA` environment variable and start
`build/cargo/release/melee` (`melee.exe` on Windows).

## Docs

| Start here | |
|---|---|
| [STATUS.md](docs/rust-port/STATUS.md) | current state, crate map, commands |
| [notes/](docs/rust-port/notes/) | one file per topic: [netplay](docs/rust-port/notes/net.md), [Slippi online](docs/rust-port/notes/slippi-online.md), [mod host](docs/rust-port/notes/modhost.md), [zero hitch](docs/rust-port/notes/zero-hitch.md), [performance](docs/rust-port/notes/perf.md), [Dolphin oracle](docs/rust-port/notes/dolphin-oracle.md), [external replays](docs/rust-port/notes/replays-ext.md) |
| [docs/specs](docs/specs/) | mod ABI ([mod-c-v1](docs/specs/mod-c-v1.md)), hooks, NHV1 state hash, Geno |
| [CONTRIBUTING](.github/CONTRIBUTING.md) | coding style (inherited from the decomp; a Rust section is planned) |

## Licences

- Engine and tools: [GPL-3.0](rust/LICENSE).
- Specs, headers and examples in `docs/specs/`: [MIT](docs/specs/LICENSE-MIT).
- Third-party code keeps its own licence (see each folder and [CREDITS.md](CREDITS.md)).

## Credits

- [doldecomp/melee](https://github.com/doldecomp/melee) and its contributors: the decompilation this engine is built from.
- Hero and Melee Unlocked: we follow his design for Slippi online, with his permission.
- [Project Slippi](https://github.com/project-slippi): replays and the online protocol we aim to match.
- See [CREDITS.md](CREDITS.md) for the rest.

## Legal

This project is not affiliated with or endorsed by Nintendo, HAL Laboratory or Project Slippi. It contains no
Nintendo code, assets or disc data. You must supply a disc image you own. *Super Smash Bros. Melee* is a trademark
of Nintendo.
