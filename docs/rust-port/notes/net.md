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
  `gen_css_scripts.py`: Mario v Fox on Battlefield, two matches, the second a rematch from the results).
