# Netplay and rollback

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 381-399 -->
## Platform track, night 3 step 2: gw_rollback.c session + SyncTest (2026-10-21)
- `melee_net::session` is gw_rollback.c: `RbSession<G: RbGame>` holds the C's file-static `rb`
  (used/truth rings of 64, conf_slot, first_wrong, plan, hring, early buffers, fake network,
  latches). The C's externs are `RbGame`: `frame` (gw_Replay_Frame), `peek_input`
  (gw_Replay_PeekInput), `save`/`load` (gw_snap_save/load: the native game state; the session keeps
  a ring of maxb+3), `state_hash` (gw_Snap_Checksum), `game_hash` (gw_RB_GameHashForFrame, the
  neutral hash), `run_frame(inputs, resim)`. `run_tick` is the scene loop: `iterations` ->
  per iteration `iter_start` / `run_frame` / `iter_end`. Netplay: `RbConfig { net: true }` and
  `NetAdapter::tick(&mut net, &session)` before `run_tick` (the C calls gw_Netplay_Tick inside
  gw_RB_Iterations; the transport's callbacks borrow the session, so the poll is hoisted).
- `SyncTest<G>`: gw_snap.c's SyncTest on native snapshots (whole-state and curated hash compared at
  the start of each resimulated frame and at the end of the window).
- Verified: `cargo test -p melee_net --test session_oracle` (rb_hash/rb_gen/rb_arrival vs the C cut
  from gw_rollback.c, `oracle/rb_oracle.sh`), `--test session` (see the row). Snapshot lesson: the
  state is restored in place as bytes, so pointers into the fighters (e.g. CpuFighter.csP) stay valid;
  a snapshot that moved the Fighter would break them.
- Not done: wiring into a game loop (needs a Sim that reads pads), the Slippi mode, and comparing
  the stateful session against the C session itself (only its pure functions are oracle-checked).


<!-- RUST_PORT.md lines 1414-1469 -->
## Platform track: netplay on the real game loop (port/plat-net, 2026-10-24)
- `melee netplay` (`rust/melee/src/netplay.rs`, `netmatch.rs`, `np_env.rs`) drives the native VS match
  (`VsMatch`) through `melee_net::session::RbSession` over `melee_net::Net` on real UDP. `NetMatch` is the
  `RbGame`: `save`/`load` = `VsMatch::rb_save/rb_load` (the `state_dump` component set, fighters restored in
  place), the desync checksum = the neutral hash (`game_hash`), pads from the session replace the script
  (`pad_feed`). `RbGame::halted` (new): a frame that died half-stepped in code that is not ported yet
  (typically a mispredicted input) is neither snapshotted nor hashed; the next tick reloads the clean
  snapshot of that frame (`iter_start`), so the confirmed inputs roll it back like any misprediction.
- App layer, `melee_net::app` (gw_netplay.c, direct and server paths): `scene` (np_build_scene and its
  parse), `lobby` (lb_apply_, S/L/A/J/N/C/G messages, strike 1-2-2, ban/pick, counterpick characters,
  3 s countdown; 5 unit tests), `rdv` (the matchmaking client: REG/JOIN/PEER/CODE, hole punch `GDPH`,
  relay `GDMD`, RAND/MATCH/QUEUED/TIMEOUT, STUN, LAN pick; unit test), `room` (np_start_session, np_poll,
  guest hello, ACCEPT "J c0 c1 scene" blob, lobby enter/tick, lb_go, deferred session start for room codes
  and random matches).
- Proofs (all on this machine, UDP loopback, `GW_ISO_VANILLA`, pad_falcon_fox_fd, 1801 golden frames):
  - `melee synctest --depth 12 --golden docs/rust-port/golden/pad_falcon_fox_fd.csv` (12 = the max rollback
    depth, `maxb` clamp): 1802 frames, 21504 hash checks, 0 mismatches, 1801/1801 frames golden-equal
    (the run stops at frame 1802: `ftCo_RunBrake_Enter` is not ported).
  - two Rust instances, `melee netplay --host` / `--join`, 60 fps pacing: 0 rollbacks, 1801/1801 equal to the
    golden on both, identical CSVs. With `--lag-ms 70 --loss-pct 5` (inbound, both sides): 78 rollbacks,
    depth 6-7, 0 session or transport desyncs, still 1801/1801 golden-equal and identical CSVs
    (`--lag-ms 90 --loss-pct 10` too).
  - the same through the lobby (`--lobby`: blind character pick, strike, ready, countdown, `G`), through
    `tools/netplay/server/gdmelee_server.py` room codes (`--host --server S` then `--room CODE --server S`) and
    through random matchmaking (`--random --server S` on both): all golden-equal, identical CSVs, rollbacks
    included when lag/loss are on.
- Golden comparison needs `--seed 12345` on the host (the match seed is random otherwise, so the hashes differ from the golden CSV from frame 0 while both peers still agree). Re-verified 2026-10-24 on the merged tree (no window, no OS input): synctest depth 12 0 mismatches / 1801 golden-equal; host+guest `--seed 12345 --lag-ms 70 --loss-pct 5`: 78 rollbacks, 0 desyncs, 1801/1801 golden-equal, identical CSVs.
- `--live-rng`: the online per-frame RNG rule (`gw_Replay_ChecksumSeed`: a frame starts from
  `seed + ((frame + 123) << 16)`), needed to match a C netplay match; the checksum uses it too. Off by default
  (the golden CSVs are offline).
- Not ported: UPnP, Envoy/stage-run/saved-run lobby messages, mod sync and the identity exchange (tokens are
  `ck:N` / `ext:N`; the Rust sim plays Falcon, Fox on Final Destination, Battlefield is only a lobby stage),
  controls words other than 0, Turbo rules, rematch/rejoin after a match, CPUs/items in the scene.
- C interop: see the section below for what was measured.

### Rust vs C build over the wire (2026-10-24)
- The wire protocol allows it. The C build (this tree, `GW_BUILD_ROOT=_build/agents/plat-net`) plays scripted
  netplay (`MELEE_NETPLAY=host|join:`, `tools/rust-port/golden/pad_np.lua`); the Rust peer takes the C's
  handshake words (`--exe-hash` = the C `build id`, `MELEE_NP_GLOBAL_HASH` = `mods=` of the new "handshake
  words" log line) and the C's content identities (`MELEE_NP_IDS`, run time only). HELLO/ACCEPT, the
  `J c0 c1 scene` blob (tokens `ck:N`/`ext:N` are accepted by the C scene parser), hold/START, inputs and
  checksums all interoperate in both directions.
- C change needed: the C's netplay checksum is RB_GameHash (a wider word set), not the neutral hash. With
  `MELEE_RB_NEUTRAL_HASH=1` (gw_rollback.c + `RB_NeutralHashForFrame` in fighter.c, same online seed rule) both
  sides compute NHV1. The Rust peer also needs `--live-rng` (the C live-mode per-frame seed rule).
- C vs C (neutral checksum, scripted pads): 3224 frames, both neutral CSVs identical, no desync.
- Rust guest vs C host, full match (1893 frames, 2 rollbacks C side / 1 Rust side): 0 desyncs reported by either
  side; the Rust per-frame words (fighters, items) equal the C's MELEE_NEUTRAL_HASHLOG on all 1893 frames.
  Only the post-frame seed column differs on 548 frames (more or fewer logic RNG draws per frame in effect /
  particle code; the seed is re-forced every frame online, so it cannot reach the next frame's state).
- Rust host vs C guest: identical for 205 frames, then a checksum DESYNC reported at session frame 89 (golden
  index ~212): at index 205 the C draws 38 logic RNG values and the Rust 7 (an effect / particle spawn the
  Rust effect code does not make on that path), and at index 211 Falcon's landing word differs (5715B485 vs
  5615B536). This is a sim gap found by netplay (different input timing from the offline golden), not a
  transport problem; fixing it belongs to the fighter / effect ports.

## Netplay and replays in `melee play` (port/netplay, 2026-11-03)
- `melee play --netplay host [--np-port 51500]` / `--netplay join 127.0.0.1[:PORT]`: a 2-player rollback match on the
  retail flow (`rust/melee/src/play_net.rs`, `retail_flow.rs` `step`). The connection is the direct room of
  `melee_net::app::room` (HELLO / ACCEPT / the `J c0 c1 scene` blob, the same wire as `melee netplay` and the C
  build's gw_netplay.c), made before the window opens. The host's `--match a,STAGE` gives its fighter and the stage,
  the guest's `--match b` its fighter; `--stocks N`, `--time-secs S` (minutes, rounded up) the rules. Port 1 is the
  host, port 2 the guest; each plays with its own port-1 controller. Options: `--np-delay D` (input delay, default 2,
  the guest adopts the host's), `--np-rollback N` (window, default 7, max 12), `--np-seed`, `--np-lag-ms` /
  `--np-loss-pct` (inbound degradation, the loopback test), `--np-hash-out F.csv` (final hash per frame),
  `--np-timeout S`. The socket binds 127.0.0.1 (loopback only in this build).
- The session (`RbSession` over `ModesGame`, the mode loop's engine) takes over one frame into the VS scene; before that
  both peers run the boot with neutral pads, so they enter it in the same state. Desync check: the transport's
  checksum exchange (neutral hash per final frame); a mismatch ends both peers with `netplay host|guest: ERROR: DESYNC
  at frame F (game frame G): local hash X, peer hash Y` and a non-zero exit. One match per connection: after the
  match's last confirmed frame both peers linger 1.5 s (the peer's last checksums) and quit. The CSS/SSS flow over the
  wire is not done (follow-up: a session governing the CSS scene too).
- `MELEE_SHOW_FPS=1`: the title line gets `netplay host | ping N ms | delay D | rollbacks R/s (total T, max K frames)
  | frame F` (`Game::status_line`, melee_platform), also logged as `gw: status:`.
- `MELEE_PLAY_SYNCTEST=K`: GGPO SyncTest on every match frame of `melee play` (windowed or headless).
- Replays: `melee play ... --record FILE|auto` (auto = `Replays/melee-<unix time>.mrpl`), `melee play --replay FILE`
  [`--headless`]. Native format `.mrpl` (play_net.rs `Replay`): `MRPL`, u16 version 1, u16 flags (bit 0: recorded with
  the rollback allocator modes), u32 + UTF-8 header (`arg=` the play options that set the boot up, `seed=`, `build=`,
  `np_scene=` / `np_role=` for a netplay match), u32 count, then per engine tick 4 x 16-byte pads (button u16, stick,
  c-stick, L, R, analog A/B, err, extButton u32) and the u32 neutral hash (|1) after a match frame (0 elsewhere). The
  playback feeds the same pads to the same boot and checks every hash; `replay: N ticks played, M match frames
  hashed, K mismatches, final hash H (file H)`. A netplay peer records the confirmed inputs (both ports), so either
  peer's file replays the whole match offline. `.slp`: not written (follow-up: a Slippi exporter needs the pre/post
  frame records per fighter, which the engine can produce from the same playback; the reader is melee_data::slp).
- Memory rules a rollback in the windowed process needs (all in this branch): the arena is the game thread's only
  (`arena::GAME_THREAD`); a system-heap block the game frees or grows while a snapshot base exists is kept
  (`main.rs rollback_pinned`); the drawn frame is detached from the arena with fresh Arcs (`play_net::detach_frame`);
  the GX shim is created inside the game scope; the sound mixes on the game thread (`MELEE_AUDIO_GAME_THREAD`: the
  mixer thread held the sound's lock while a snapshot copied it, and a load restored it locked = a hang); a
  resimulated frame runs its render pass with display lists suppressed; disc reads pre-touch their buffer while the
  arena is copy-on-write (`ReadFile` fails with ERROR_NOACCESS on a protected page). `MELEE_HANG_SECS=N` with
  `MELEE_SAMPLE` prints the main thread's stack of a tick that has not returned for N s.
- Rule: no static may hold system-heap or GPU state that changes during a rollback match. A load rewrites every
  static (the executable's writable sections) and the arena; only system blocks freed or grown INSIDE the game scope
  are pinned (`rollback_pinned`), so a static touched outside it (presenter, render thread, network) would be put
  back pointing at freed or reused memory. Keep such state in values the harness owns (`play_net::Detacher` is a
  field of `ModesGame` / the retail flow; the renderer's caches and `GpuHealth` are in `Renderer`). Statics the
  game itself fills inside the game scope live in the arena and roll back consistently. Audit (#42): the statics of
  melee / melee_platform / melee_gx are OnceLocks set before the match, the test-only log capture, and
  `melee_gx`'s thread-local error guard (empty between calls); no wgpu handle is stored in game memory.
  Geno define audio (port/genorb): the clip cache / PCM / start ledger are in `geno_host::GnHost` (system heap, set-once
  pointer `GN_HOST`, every change under `host_heap`), the mixer's native clip slots in `Ax`'s boxed `ClipBank`; resim and
  confirmed frames reach it through `geno_host::net_frame` / `net_confirmed` called by the harness (no rewritten static).
- Headless 50 ms crash (`access violation (execute at 0x0)` in ~1 of 3 `play_pair.sh NAME 50 5 --headless` runs, often
  both peers at the same tick): `Audio::capture` (the headless mix kept for `--audio-wav`, a harness value) grew inside
  the game scope (`retail_flow` ticks the audio under `GameScope` in a rollback run), so its buffer moved into the arena.
  A load put the free lists back to a state where that block was free; the game handed it out again (decoded archive
  objects) and the capture's next growth freed it under them; `reclaim_overlapping` later dropped a `GcArena` whose
  `Owned` entries had been overwritten and called a null `free`. Fix: the capture grows with the arena off. Guard:
  arena free blocks carry a mark (`arena::FREE_MARK`); a double free or a write after free prints
  `melee: ARENA MISUSE: ...` with raw return addresses (symbolise with llvm-symbolizer against the exe). The fault
  report now lists the code addresses on top of the faulting stack, which is what a call through null leaves.
- #42 (windowed peers losing the GPU device, "Texture with 'GX texture' label is invalid"): `detach_frame` made
  fresh `Arc`s every frame, and the renderer caches GPU textures by the data `Arc`'s address, so every texture
  missed every frame: ~63k live textures, ~2.7 GB of video memory per peer by the 600-frame eviction (offline:
  ~200). With other games open the driver reset the device. `Detacher` now keeps the copies by a 128-bit content
  hash (~380 textures, ~100 MB per peer). The panic itself: wgpu error scopes are per device, not per thread, so
  the decode thread's pipeline scope swallowed the main thread's `create_texture` error and the next use of the
  invalid texture hit wgpu's panicking default handler. `melee_gx` now installs an uncaptured-error handler and a
  device-lost callback (`Renderer::gpu_fault`): device lost / unexpected errors stop the game cleanly with
  `gx: GPU error: ...`; out of video memory on a texture evicts the texture cache and retries once.
- Proofs (`bash tools/rust-port/netplay/play_pair.sh NAME LAG LOSS [args]`: two windowed peers on 127.0.0.1, scripted pads
  np_fight.txt / np_fight2.txt, Falcon v Fox on FD, 4 stocks, 1 minute; per-frame hash CSVs compared; the host's
  replay played back headless):
  - 0 ms: 2936 frames, all equal; rollbacks 34 / 25 (max depth 2 / 5); replay final 1A9DC5B9 = live.
  - 50 ms inbound each side + 5 % loss (ping ~160 ms): 2121 frames, all equal; rollbacks 263 / 256 (max 7); replay equal.
  - 150 ms + 5 % loss, `--np-delay 4 --np-rollback 12` (ping ~335 ms): 2050 frames, all equal; rollbacks 247 / 241
    (max 10 / 12); replay equal.
  - `MELEE_NP_TEST_DESYNC=600` on the guest: both stop at game frame 601 with the DESYNC error.
  - windowed `MELEE_PLAY_SYNCTEST=4`, whole match: 0 neutral mismatches (the whole-state hash differs on resimulated
    frames because their display lists are not decoded).
- Melee Unlocked (reference only, nothing taken): its players use `mu_net` v1 (docs/mu-net-protocol.md there: ENet
  framing, Ed25519 identities, X25519 + XChaCha20-Poly1305, its own message ids), not this wire, so the two cannot
  meet; joining his launcher as a third engine would need a `mu_net` implementation. His users expect `.slp` replays
  in `Replays\` and `--replay <file.slp>`, so the `.slp` exporter is the replay follow-up that matters for them.

## Slippi export, CSS netplay with rematches (port/net2, 2026-11-03)
### `.slp` export (`--slp FILE|auto`, `rust/melee/src/slp_export.rs`)
- Written from the public spec (slippi-wiki SPEC.md), declared Slippi 3.18.0: Event Payloads; Game Start (0x36); per
  frame Frame Start (0x3A), one Pre-Frame (0x37) and one Post-Frame (0x38) per fighter (port order, leader before
  follower), Frame Bookend (0x3C, latest finalized = the frame); Game End (0x39); UBJSON `metadata` (`startAt`,
  `lastFrame`, `players.N.characters` / `names.netplay`, `playedOn`). No Item Update / Message Splitter / Gecko list.
- Sources: VS `Replay_RecordMatch` (gmvs fn_8016E730: the StartMeleeData and the seed), the fighter code's
  `Replay_RecordInput` (the pre-frame point) and `Replay_TraceFighter` at the 3.x post-frame point
  (`Replay_TraceAtProcMap` = 0). `begin_tick` / `end_tick` around each engine tick of `retail_flow::step` group them; the
  first tick with fighter input is frame -123. `auto` = `MELEE_SLP_DIR` (default `Replays`) `/Game_<UTC>.slp`; later
  matches of a run with an explicit name get `-2`, `-3`.
- Rollback runs: the hooks also fire on resimulated frames, so a netplay peer with `--slp` needs `--record` and exports
  after the match from a headless child `melee play --replay <mrpl> --slp <spec>` (a linear playback). `--slp` with a
  synctest run is refused.
- Faithful (engine values at the spec's points): Game Info Block (encoded GC-style field by field from the native
  StartMeleeData: bitfields, stage, timer, item bits, damage ratio, the six player records), Random Seed, frame
  numbers, pre-frame seed / action / position / facing / processed sticks, trigger and buttons / percent, post-frame
  internal character, action, position, facing, percent, shield, last hitting attack (`x208C`), combo count (`x2090`),
  stocks, action state frame, ground/air, the five speeds, hitlag. Game End method: GAME! (one or no fighter with
  stocks), TIME! (timed match past its clock), else No Contest; placements by stocks then percent.
- Derived / approximate: physical buttons (the PADStatus bits, which are the spec's), physical L/R (`analog / 140`),
  raw UCF stick bytes (the pad's sticks), last hit by (`dmg.x18c4_source_ply`, 6 = none), misc AS (first word of the
  motion vars), last ground id (`coll_data.floor.index`), jumps remaining (`max_jumps - jumpsUsed`), hurtbox state
  (`x198C`), animation index (`anim_id`). Zero: state bit flags 1-5, L-cancel status, instance ids, nametags, display
  names / codes, dashback / shield-drop fixes (vanilla: off, which is the truth), scene frame counter is the frame index.
  `playedOn` is `network` for a netplay match and `console` otherwise (py-slippi accepts only Slippi's values).
- Validation, three independent readers: peppi-py 0.8 and py-slippi 1.6 (pip, outside the repo) parse every file
  (version, stage, players, frame count, end, metadata); `melee slpcheck` (melee_data::slp + the engine) re-simulates
  each export from its Game Start and pre-frame inputs and compares every post-frame row: netplay 0 ms (from the
  .mrpl) 6722 fighter-frames equal, offline vs a level-9 CPU 7448 equal, netplay 50 ms + 5 % loss 3240 equal; the two
  peers' exports of one match are byte-identical.
- Launcher seam: `--replay-dir D` -> `MELEE_SLP_DIR=D --slp auto` (his users' Replays folder).

### Netplay from the retail CSS (`--np-css`) and rematches
- `melee play --netplay host|join ... --np-css`: after the room handshake both peers boot straight into VS mode
  (`MELEE_RETAIL_FIRST=2`, no memory card: a card's unlocks would differ per machine) and play the character select,
  the stage select and the results in **lockstep** (`np_lockstep.rs`, its own socket at room port + 1: every packet
  repeats the unacknowledged pads, a frame runs when both ports' pads are in; port 1 = host, port 2 = guest). Each
  render tick schedules one local pad, so a pad script's frame k is menu frame k + delay whatever the stalls. The match
  scene is handed to the rollback session as before; the hand-over now counts engine frames (`e.frames`), not render
  ticks, so a stall cannot move it.
- After a match both peers go back to the menus (results -> CSS -> START -> SSS) and the next match opens a new direct
  room on port base + 1 + k (a `Net` is one session from frame -123; reusing it would need a frame base like mu_net's).
  `--quit-after-matches N` ends the run after N results. Both peers must pass the same rules options (`--time-secs`,
  `--stocks`, `--items`): the menus' rules come from each side's presets.
- The `.mrpl` of such a run (meta `np_css=1`) holds the menu ticks and every match; the end frame of each match is
  recorded at the hand-back (`flush_rec`: it has no start-of-next-frame checksum).
- Test: `bash tools/rust-port/netplay/css_pair.sh NAME LAG LOSS [args]` (pad scripts generated by
  `gen_css_scripts.py`: Mario v Fox, two matches, the second a rematch from the results; match 1 on Battlefield,
  match 2 wherever the SSS cursor then is, ext 9 in the runs).
- Verified (windowed peers, loopback, this machine): 0 ms: 2 matches, 12681 frames, 0 hash differences; 50 ms + 5 % loss
  with `--slp auto`: 8051 frames (3973 in match 2), 0 differences, host replay 9792 ticks / 8053 match frames, 0
  mismatches; four .slp files (two per peer, one per match) parse in peppi and py-slippi. `play_pair.sh` after the
  change: 0 ms 2154 frames, 50 ms + 5 % 1727, 150 ms + 5 % (`--np-delay 4 --np-rollback 12`) 3080, all equal on both
  peers and in the replay.
- Open: `melee slpcheck` (SLPCHECK_VANILLA_SPAWN=1) on the exports of the CSS-started matches diverges at frame ~870-930
  (a hitlag / x difference), while the same characters and stage started from `--match` (Mario v Fox, Battlefield)
  match over all 7448 fighter-frames. The file's values are the live engine's; the gap is state that a CSS-started
  match carries and the Game Info Block (slpcheck's scenario boot) does not. Not isolated.
- Slippi playback caveat: Slippi players apply Neutral Spawn (and UCF) to every replay; these exports were played
  without either, so a Slippi Dolphin viewer can drift from frame -123 on stages whose vanilla spawns differ (FD
  matched in slpcheck with Neutral Spawn on, Battlefield did not).

## Internet play (port/net3, 2026-11-03)
### How two people play
Both players need the same `melee.exe` build (the same git commit; the handshake refuses another build with
`different game build (host X, you Y)`), the same disc and the same mods (`different global game data; host has: mods
...`). The host's rules are sent to the guest: the guest's own `--stocks` / `--time-secs` / `--items` are replaced (the
log says `the host's rules win: ...`), so only the host sets them.

**Direct IP** (one player can receive UDP: a port forward, the same LAN, or IPv6 without a blocking firewall):
- host: `melee play --iso DISC --netplay host [--np-port 51500] [--np-css] [--stocks 4 --time-secs 480]`
- guest: `melee play --iso DISC --netplay join HOST[:PORT] [--np-css]`, where HOST is an IPv4 address (`203.0.113.7`),
  an IPv6 address (`[2001:db8::7]:51500`, or bare without a port) or a host name. Port default 51500.
- The host binds UDP `--np-port` on all interfaces, IPv4 and IPv6 (`--np-bind IP` binds one address). Across the
  internet over IPv4 the host's router must forward UDP 51500 to the host PC; on a LAN use the host's LAN address.

**Match code** (neither can receive: hole punching through a rendezvous server, relay as the fallback):
- someone runs the server, `melee_relay [--port 51600] [--bind 0.0.0.0] [--bind6 ::|off]` (rust/melee_relay, std only,
  no state on disk) on a machine both can reach (a VPS, or a PC with UDP 51600 forwarded). Nothing is baked in: the
  address is `--np-server HOST[:PORT]` or `MELEE_NP_SERVER`.
- host: `melee play ... --netplay host --np-server SERVER` prints `netplay host: MATCH CODE K7QX`;
- guest: `melee play ... --netplay join K7QX --np-server SERVER`.
- The server tells each side the other's public and LAN address (`PEER`); both send punch packets and the room's
  HELLO to it (same public IP = dial the LAN address). With no direct packet after 3 s both switch to the server's relay
  (`no direct path after 3000 ms - relaying through the server`); `--np-relay` forces it. The log says which:
  `path direct (hole punched)` / `path RELAYED through the server`. The relay costs the server about 60 small packets
  a second each way.

**Windows firewall**: the first time `melee.exe` listens on a non-loopback address Windows asks whether to allow it;
allow it on Private networks (Public if the internet link is classed public). Without that, inbound UDP is blocked and
only the outbound half of hole punching works (direct IP as the host fails; match codes usually still work, or fall
back to the relay). An explicit rule, for whoever runs it (not done by any agent): `netsh advfirewall firewall add rule
name="melee netplay" dir=in action=allow protocol=UDP localport=51500` (51600 for `melee_relay`). The guest needs no
rule. Routers: forward UDP 51500 for direct IPv4 hosting; nothing for match codes.

**Errors** end the run with a reason: no answer in `--np-timeout` s (default 60, with what to check); refused for
build / mods / protocol; `connection lost: peer timed out (the other player quit, or the network dropped)` after 5 s
of silence in a match; `no packets from the other player for 20 s in the menus` in the lockstep menus; DESYNC as
before. No reconnect.

### What changed
- `melee_net::transport`: `Addr` gains `ip6` (IPv6; `ip` is then a fold used only as a hash input), `Display`
  (`1.2.3.4:5` / `[v6]:5`), `addr_resolve` (literals of both families and host names). `UdpTransport::open(0, port)`
  binds IPv4 and IPv6 on the same port (two sockets: Windows' IPv6 sockets are V6ONLY); `open_at(ip)` one address.
  The netcode compares whole addresses. `rdv` (the server client) reads `[v6]:port` addresses and resolves the server
  by name; `stun` is not used by `melee play` (it would contact public STUN servers).
- `rust/melee/src/np_mux.rs`: one socket per run. The first room is unprefixed (gw_net `GN` packets, C-compatible), the
  lockstep menus `MLS1`, rematch room k `MRM`+k. So a punched NAT mapping / relay serves every match of `--np-css`
  (before, the lockstep was room port + 1 and rematch k room port + 1 + k: three ports through a NAT).
- Handshake words: exe = `build_hash()` (git commit of the build; `MELEE_NP_BUILD_HASH=hex` fakes another for the
  test), global data = `mods_identity()` (FNV of the mounted mod ids + the mod set fingerprint, 0 = vanilla, described
  as `mods a, b (fingerprint)`). Before, both words were the git hash and the mods were not compared. A dirty tree on
  the same commit has the same hash.
- Host rules: the agreed scene carries `;np_rules=s<stocks>,t<secs>,i<items>` (`-` = default). The guest adopts it
  for the presets (CSS) and for its replay's boot options; the `--match` path takes `t` as the time limit.
- Network simulation (`Degraded`, inbound): `--np-lag-ms`, `--np-jitter-ms` (uniform extra 0..J), `--np-loss-pct`,
  `--np-dup-pct`, `--np-reorder-pct` (held lag/2 + 10 ms behind its successors).
- MTU: the largest datagram is a lockstep packet, now at most 90 pads (1093 bytes; + 4 relay + 48 IPv6/UDP < 1280).
- Fixed: the menus' 20 s silence check counted the match's duration (the lockstep is idle during a match), so a
  peer that reached the results a moment later ended the run; `Lockstep::resume` at the hand-back.
- Test pads: `MELEE_NP_SCRIPT_NO_START=1` (set by css_pair.sh) drops START inside a match: the scripts are timed in
  render ticks, and with the match-code flow (both peers start together) a results START landed in match 2 and paused it.

### Verified (one machine; non-loopback = the Hyper-V adapter 172.21.48.1, IPv6 = ::1; local melee_relay)
- `play_pair.sh` (scripted pads, per-frame hashes of both peers + the host's replay): loopback 0 ms 2011 frames equal;
  `NP_BIND=172.21.48.1` 50 ms + 5 % loss + 30 ms jitter + 5 % dup + 10 % reorder 2230 equal (281 rollbacks, max 7);
  `NP_BIND=::1` 150 ms + 5 % loss + 20 ms jitter + 3 % dup + 5 % reorder, `--np-delay 4 --np-rollback 12`: 2687
  equal (max depth 12); match code over IPv6 (`NP_SERVER=[::1]:51990`): 2893 equal, punched; match code with the LAN
  address made unreachable (`--np-lan 127.0.0.2`): relay fallback after 3 s, 50 ms + 5 %, 2152 equal. Replays equal.
- `css_pair.sh` (CSS, two matches, rematch on the same socket): loopback 0 ms 7911 frames equal; match code on
  172.21.48.1 0 ms 7674 equal (punched); forced relay 50 ms + 5 % loss + jitter / dup / reorder 7674 equal.
- Refusals: guest with another build hash, host with a mod the guest lacks; timeouts on both sides; guest killed
  mid-match -> host `connection lost: peer timed out ...` 5 s later.
- Not verified: two real machines, a real NAT (hole punching was only exercised between two local sockets), the
  firewall prompt path, IPv6 over a real network, host names.

## Early inputs (port/earlyin, 2026-11-03)
Credit: our early inputs are modelled on Hero's early-input work in **melee-unlocked**. His work in turn credits
**Arte** (JulienBernard3383279) for subframe rollback, from the Kristal build and the "Subframe rollback" draft
for Slippi's Dolphin. Thanks to both. The first version of `melee_net/src/early.rs` (port/earlyin) followed Hero's early-input code too
closely, including some of his comment wording, while claiming to share nothing with it; Hero pointed this out.
On 2026-11-05 (port/hero-fix) the file was rewritten clean-room from our own tests and this note only, without
opening his source; behaviour is unchanged (the existing tests plus a 2.5M-case randomized old-vs-new comparison of
every function, run once and not kept).

### What it does
Prediction used to be repeat-last only. Now a sampler thread reads the local controller about every millisecond
between the game's polls, and when the pad changes in a way the game acts on, it sends the new pad to the peer at once,
stamped with the session frame it will first appear in. The receiver predicts an unconfirmed frame from the newest
early pad for a frame in (last confirmed, frame]; with none it repeats the last confirmed input as before. **A
confirmed input always wins** (a misprediction rolls back exactly as before), so the simulation, the replays, the
`.slp` exports and the checksums are unchanged; only how often a prediction is right changes.

- Rules (`melee_net::early::matters`, Melee Unlocked's thresholds): A/B/Start pressed or released; X/Y/Z/L/R either
  way; no D-pad; main stick X 23 / 64, Y 23 / 53 / -23 / -56, -53 while shielding (shield drop), C-stick X 23 / 64,
  Y 23 / 53, both signs; analog trigger 43. Sticks within 2 of neutral count as zero for the judgement only (the bytes
  sent are the pad as read, which is what the confirmed input will carry). `MELEE_EARLY_RULES=presses` = outward
  crossings and presses only. Follow-ups: after a crossing for frame F, any further byte change before F's poll is
  sent too, at most one per 2 ms. After a tick that scheduled no pad (stall / time-sync wait) the next read is sent if
  it differs at all.
- Target frame: the game thread tells the sampler each local frame it schedules (`after_tick`, from the session's
  local truth) and when it read the controller (`note_poll`, right after `PadSystem::read`). A change is for
  last scheduled + 1, or + 2 between the read and the scheduling.
- Packet: `MEI1` | room number u8 (rematch rooms of `--np-css` ignore each other's) | target i32 BE | in-game port u8 |
  the 11 `np_encode` bytes; 21 bytes, unreliable, never retransmitted. Mux channel `EARLY_CH` (np_mux.rs).
- Sending off the game thread: `Transport::thread_sender` gives cloned sockets (`ThreadSender`, with the rendezvous
  relay's `GDMD` wrapping when relayed), so a send never waits for a tick. Reading off the game thread:
  `PadSystem::sampler()` = `PadSampler` (the same keyboard / gamepad maps / adapter pipeline, its own gilrs instance,
  `AdapterPeek` over the adapter's newest report; no script, no reset combo, no F9).
- Receiver (`RbSession::submit_hint` / `prepare`): at most 8 entries per remote slot, a second pad for one target
  replaces the first, entries at or before the confirmed frame are dropped as inputs arrive.
- Trust: a peer whose early guesses are wrong more than 60 % of the time after 64 judged guesses has its early pads
  ignored for the rest of the match (the summary says `IGNORED (wrong W of G)`). `MELEE_TEST_EARLY_LIE=1` sends
  made-up early pads (first button byte inverted).
- Compatibility: a trailing capability byte on HELLO (guest) and ACCEPT (host), bit 0 = early inputs. It is only
  written when non-zero, so with the feature off the wire is byte-identical to the C build's (net_oracle); the C
  build and older Rust builds ignore trailing bytes and never announce the bit, and nothing is sent to (or used
  from) a peer that did not announce it (`early inputs off for this match: the peer does not announce them`).
- Settings: on by default. Off: `--np-no-early` (or `--np-early 0`), `MELEE_EARLY_INPUTS=0`, or `early_inputs=0` in
  settings.cfg (cfg.rs, env wins). The match summary line ends with `early inputs: sent N changes (...), received R,
  predicted P frames from them: right / wrong (helped H = right where repeat-last was wrong, hurt = wrong where
  repeat-last was right); repeat-last right / wrong`.
- Scripted tests: a pad script changes exactly at a poll, so nothing ever changes between polls.
  `MELEE_TEST_EARLY_LEAD_MS=N` makes the sampler send the script's next pad (`PadScript::peek_next`) N ms before the
  poll that takes it: the same inputs reach the game either way, only when the peer learns them differs.

### Measured (play_pair.sh, windowed peers on loopback, Falcon v Fox FD, `MELEE_TEST_EARLY_LEAD_MS=8`, jitter 10 ms)
Rollbacks per 1000 match frames, host / guest (match lengths differ run to run, so per 1000 frames):

| run | early on | early off (`--np-no-early`) |
|---|---|---|
| 50 ms each way + 10 ms jitter, run 1 | 74.9 / 66.8 (resim 246 / 224) | 105.2 / 100.2 (resim 371 / 340) |
| 50 ms, run 2 | 72.1 / 71.2 (resim 240 / 271) | 106.7 / 95.8 (resim 366 / 351) |
| 150 ms + 10 ms jitter, delay 4, window 12, run 1 | 74.5 / 80.9 (resim 342 / 394) | 94.8 / 98.6 (resim 365 / 427) |
| 150 ms, run 2 | 63.7 / 79.9 (resim 322 / 518) | 90.8 / 96.5 (resim 496 / 572) |

About 30 % fewer rollbacks at 50 ms and about 20 % fewer at 150 ms. Rollback depth is set by the lag (max 7 at
50 ms, 12 at 150 ms) in both modes; total resimulated frames drop at 50 ms and drop less at 150 ms. Early-pad
predictions: right 65-87 % (most of the misses are the frame right after a script segment change), helped 250-390
frames per match, hurt at most 7. The second 150 ms "off" run is m150off3 (m150off2 crashed on both peers at once with
a wgpu `GX texture is invalid` panic; early inputs were off and no sampler was running; its replay matched; rerun).

### Verified
- Desync tests with early inputs on (`MELEE_TEST_EARLY_LEAD_MS=8`): 0 ms, 50 ms + 5 % loss, 150 ms + 5 % loss
  (`--np-delay 4 --np-rollback 12`): every frame's hash equal on both peers (0 differences), the host's replay played
  back with 0 mismatches. Early pads were used in all three (predicted 69..1039 frames per peer).
- The live sampler with no lead (scripted, so it never sees the script's pads: 0 sent) at 50 ms + 5 %: equal, replay
  equal.
- Lying peer (`MELEE_TEST_EARLY_LIE=1`): both sides IGNORED the other's early pads after 64-65 guesses (all wrong);
  hashes equal.
- Mixed peers (host on, guest `MELEE_EARLY_INPUTS=0`): no caps bit from the guest, the host logs `early inputs off for
  this match: the peer does not announce them`, nothing is sent; equal.
- `melee_net` unit tests: thresholds (every line both ways, both rule sets, shield drop, rest clamp), wire,
  buffer order / replace / cap / best, trust, sender targets / follow-ups / skips.

### Open
- Not tested with a real controller (agents send no OS input): the live sampler's device path (gilrs second
  instance, `AdapterPeek`, keyboard via the shared key set) only ran with no devices. A second gilrs instance
  should see the same pads in the same order; unverified.
- Two runs out of about 16 ended in a wgpu panic in `Presenter::present_frames` (`Texture with 'GX texture' label is
  invalid`, crashlogs/crash-20261103-1638*, -1648*); one of them had early inputs off and no sampler running, so it is not
  this change; not investigated.
- (Fixed by port/npsend, below.) The local input used to be sent at the NEXT tick's `Net::poll`, ~16 ms late.
- `MELEE_EARLY_ROLLBACK` (roll back as soon as an early pad contradicts a frame already run) is not implemented.

## Local input sent right after run_tick (port/npsend, 2026-11-04)
- `tick_match` calls `Net::send_local_now(current_frame)` right after `run_tick` (the session borrow is dropped first,
  because the callbacks borrow it): it pulls the inputs `run_tick` just scheduled and sends an INPUT packet at once if
  there were any. Same packet, so no protocol change and no handshake bump. The next tick's `poll` still sends as before
  (acks, checksums, resends). Early inputs untouched. No new static: the A/B toggle `MELEE_NP_LATE_SEND=1` (old timing,
  for measurements) is a `NetDriver` field read when the driver is made.
- New session stat `arrive_n` / `arrive_sum`: for each remote frame confirmed in order, this side's current frame minus
  that frame. The match summary prints the mean as `input arrived X frames after due` (the effective input lag, in frames).
- Measured: play_pair.sh `--headless`, loopback, `MELEE_TEST_EARLY_LEAD_MS=8`, jitter 10 ms, no loss; 150 ms with
  `--np-delay 4 --np-rollback 12`; two runs each. host / guest:

| run | before (`MELEE_NP_LATE_SEND=1`) | after |
|---|---|---|
| 50 ms, input lag (frames) | 2.15 / 2.37, 2.28 / 2.62 | 2.00 / 1.10, 1.99 / 1.14 |
| 50 ms, rollbacks per 1000 frames (resim) | 114 / 113 (934 / 984), 109 / 117 (1067 / 1325) | 114 / 157 (1027 / 1012), 113 / 155 (1024 / 1022) |
| 50 ms, max depth | 7 / 7 | 4-5 / 3-7 |
| 150 ms, input lag | 6.88 / 5.99, 6.89 / 5.99 | 3.67 / 3.65, 5.12 / 6.12 |
| 150 ms, rollbacks per 1000 frames (resim) | 115 / 117 (2825 / 2558), 113 / 118 (2785 / 2565) | 102 / 126 (1733 / 2097), 116 / 158 (2210 / 3519) |

  All 8 matches had every frame's hash equal on both peers, 0 session/checksum desyncs, and the host's replay played back
  with 0 mismatches. Input lag goes down in every run (about 0.2-1.3 frames at 50 ms, up to 3 frames at 150 ms). Rollbacks:
  shallower at 50 ms, but the guest rolls back more often (more of its rollbacks are 1 frame, total resim about the same).
  At 150 ms run 2 the time sync settled differently (h150after4: guest lag 6.1, 158 rollbacks per 1000 frames), so the
  150 ms rollback count is noisy and two runs do not settle it. Windowed runs of the same set lost the GPU device on both
  peers (driver reset, the environment issue described under #42); those runs were dropped.
- Verified: GATE OK, `rematch.sh css` OK, `cargo test -p melee_net` OK.
