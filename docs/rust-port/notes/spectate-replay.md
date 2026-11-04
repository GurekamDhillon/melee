# Spectating and the replay browser (port/spectate, 2026-11-04)

Written from scratch on top of `play_net.rs` (the `.mrpl` replay, `NetDriver`) and `slp_export.rs` / `melee_data::slp`.
Hero's melee-unlocked was read for reference only (unlocked-survey.md section 5); nothing from it is used.

## Spectating (`rust/melee/src/spectate.rs`)
- Host: `melee play --netplay host ... --np-spectate PORT [--np-spectate-bind ADDR]` (default bind 127.0.0.1) and/or
  `--np-spectate-relay HOST:PORT [--np-spectate-code CODE]` (through `melee_relay`; the host logs `SPECTATE CODE X`).
- Spectator: `melee spectate HOST:PORT` or `melee spectate relay:HOST:PORT/CODE` (= `melee play --spectate ...`), plus
  `--spec-delay N` (ticks held back, default 6), `--spec-catch-up N` (ticks per render tick while behind, default 16),
  `--spec-timeout S`, any window options, and the replay controls below.
- What is streamed: the host's replay as it is confirmed (the `.mrpl` header, then one 68-byte record per engine tick:
  four pads and the neutral hash). `NetDriver::tick_match` records a match frame only once its inputs are final, so a
  spectator never sees a prediction and never rolls back. With no `--record` the host keeps the replay in memory only.
- Read-only: the spectate socket is separate from the players' room; the host only reads hello / ack / bye from it and
  nothing a spectator does reaches the session. A slow or lost spectator never holds the match.
- Handshake: `H` carries the protocol version, `build_hash()` (git commit) and the mods digest (`mods_identity`, the
  same digest the player handshake uses: content mods, wasm code mods, geno). Any difference: `R reason`, the spectator
  exits 2 with `the host refused: ...`. The spectator's own `--mods` is used (the host's path from the header is dropped).
- Transport: UDP, go-back-N over the byte stream (48 chunks of 1100 bytes in flight, back to the ack after 250 ms
  without progress), end marker `E total`. After the match the host serves up to 8 s more until every spectator has
  the whole stream. Through the relay the same datagrams travel in `GDMS` (melee_relay `SPECHOST` / `SPEC`).
- Mid-match join: the stream always starts at the boot and the spectator replays the history unpaced, then plays
  `delay` ticks behind in real time. **Deviation from the brief ("snapshot plus inputs")**: a native snapshot holds
  this process's addresses (arena, statics, heap pointers) and cannot cross processes, so the deterministic boot is the
  snapshot; catch-up to ~1500 ticks took a few seconds headless.

## Replay browser and playback
- `melee replays [DIR] [--json]`: `.slp` and `.mrpl` files, newest first: characters, stage, date (Slippi `startAt`, the
  `melee-<unix>` name, else mtime, UTC), duration (match frames / 60), version / ticks / netplay role / seekable.
  `melee replays DIR --play N [options]`: `.mrpl` -> `melee play --replay`; `.slp` -> `melee golden --slp` (the sim's
  Slippi player: headless, a hash per frame, only what the sim covers, no controls).
- Controls (`rust/melee/src/replay_ctl.rs`), for `--replay` and `--spectate`: `--replay-paused`, `--replay-speed X`
  (0.25..8), `--replay-seek T|mN|+N|-N|first`, `--replay-camera game|fixed|standard`, `--replay-snap-every N` (300),
  `--replay-seekable`; a test script `--replay-ctl "TICK:CMD,..."` (or `MELEE_REPLAY_CTL`; commands pause, play, toggle,
  step[=n], back[=n], speed=X, faster, slower, seek=..., camera=...), keyed by render tick; in a window (no UI mod) the
  keys space / `.` / `,` / `+` / `-` / left / right (10 s) / `c`.
- Seek: forward runs unpaced. Back loads the nearest snapshot at or before the target (taken in the VS match every N
  ticks, `melee_engine::snapshot::Base`, one base per match) and runs forward. Snapshots need the rollback allocator
  modes: netplay replays have them (flag bit 0); for others `--replay-seekable` or a seek control turns them on.
  Re-played ticks are hash-checked again (`after seeks N frames re-played, M re-play mismatches`). Seeking back across
  a scene boundary is not possible (the base lives for one match).
- Camera: `fixed` / `standard` set the game camera's mode before each match tick. It is game state but not in the
  neutral hash; the playback's hash check showed no gameplay change with `fixed`.
- No gd-ui-v1 screen was made (optional in the brief); the HUD and Hero's launcher are untouched; nothing in melee_gx.

## Proofs (this machine, headless, loopback, MELEE_VOLUME=3, no OS input)
`bash tools/rust-port/netplay/spectate_pair.sh NAME direct|relay JOIN_AFTER_S [LAG LOSS]` (host, guest, spectator):
- `d0 direct 0`: 3288 frames, host = guest; spectator 3289 ticks, 3289 match frames hashed, 0 mismatches, final
  7824E9DB = host final.
- `r25 relay 25 50 5` (relay, joined at tick 1461 mid-match, 50 ms inbound lag + 5 % loss on the players): 0 player
  differences; spectator 3498 hashed, 0 mismatches, final EBFF5B29 = host.
- `SPEC_REFUSE=1 ... bad direct 3`: refused (`different build`), players unaffected.
- Playback: `--replay r25-host.mrpl` 0 mismatches, final = recording; with pause / 3 steps / speed 0.5 / 4 / seek back
  to 300 (snapshot) / seek m2500 / back 10 / camera fixed: 0 mismatches, 1379 re-played frames, 0 re-play mismatches.
  An offline `--record` (not seekable by flag) with seek controls: 0 / 0, 798 re-played.
- Three game processes run at once in the spectator test (the brief's pair plus the spectator), all headless.
