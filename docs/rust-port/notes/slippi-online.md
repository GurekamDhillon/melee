# Slippi online: what the Rust engine needs to play real Slippi Dolphin players (2026-11-06, port/slippi-research)

Research from source only. Nothing here contacted a Slippi server or service, and no account file was read.
Source: `project-slippi/dolphin` shallow clone `0653df819` in `_build/dolphin-oracle/src` (the mainline-based
Slippi Dolphin, with the Rust extensions in `Externals/SlippiRustExtensions`). The online Gecko codeset is
`Data/Sys/GameSettings/GALE01r2.ini` there (generated from `slippi-ssbm-asm`; the ASM sources are **not** in the
workspace, so the INI is compiled PPC with one `#source.asm` name per entry). Probe tools: `tools/rust-port/slippi/`.

**Caveat on versions.** Most players in late 2026 may still run the Ishiiruka-based Slippi Dolphin 3.x. This clone
is the mainline client, whose matchmaking is a WebSocket service (`wss://matchmaking.slippi.gg/v1/queue`). The 3.x
client used an ENet connection to the matchmaking server instead. The server pairs clients by `appVersion`, so we
have to match whatever version the server accepts on the day. Item 1 of the build plan checks this.

## Summary: the answer in five lines

1. **Transport** is ENet over UDP. Every message is an `sf::Packet` (big-endian). Players connect peer to peer
   after the matchmaking server swaps their addresses. A direct-code match uses the server too: no client path
   connects to an IP the user types in.
2. **Inputs are not the problem.** The 9-byte pad, acks, timesync and the selections exchange are small and fully
   specified in C++ (sections 1.2 to 1.5).
3. **The problem is game state.** A Slippi client runs Melee **plus about 190 online ASM patches**. It runs them
   through rollback with Dolphin's memory savestates, and it checks a game-computed checksum. Our engine must
   simulate that patched game bit for bit, or every match desyncs.
4. **The checksum and the RNG mixing are now specified** from `slippi-ssbm-asm` `d0ce23a` in
   `slippi-online-patches.md` (2026-11-06) and implemented in Rust behind an off-by-default mode; the checksum matches
   Slippi Dolphin's memory on every frame of a full replay (section 9).
5. **Terms.** Using Slippi's matchmaking and accounts from a client that is not Slippi Dolphin is not covered by
   anything in the source. It needs Slippi's own permission or ToS check before any real-server test (section 3.3).

---

## 1. The netplay protocol (SlippiNetplay.cpp / .h, NetPlayProto.h, EXI_DeviceSlippi.cpp)

### 1.1 Transport
- **ENet** (`enet_host_create(local_addr, 10, 3, 0, 0)`, `SlippiNetplayClient` ctor). It uses 3 channels and up
  to 10 peers. The client binds the port the matchmaker advertised, which keeps hole punching working (see the
  ctor comment). Each side calls `enet_host_connect` to every remote address at once. Duplicate links are pruned
  in `OnData`, and the lower port's player disconnects the extras.
- **Channels** (`Send`): `SLIPPI_PAD`, `SLIPPI_PAD_ACK` and `SLIPPI_PAD_SUGGESTION` go `ENET_PACKET_FLAG_UNSEQUENCED`
  on channel 1. Everything else is reliable on channel 0. The ack sent from inside `OnData` goes unsequenced on
  channel 2.
- `disablePacketThrottleDrops`: `enet_peer_throttle_configure` with deceleration 0. ENet passes this to the peer,
  so our ENet must accept that throttle-configure command.
- On Windows the client sets a QoS flow (`Qos2.h`, `m_qos_handle`). This does not affect the wire.
- **Disconnect reason**: ENet disconnect `data` u32 (`SlippiDisconnectReason`: 0 unspecified, 1 POOR_PERFORMANCE).
- Address choice (`SlippiMatchmaking` match handler): use the peer's external `ipAddress`. When both sides share
  an external IP, use `ipAddressLan` instead.
- **Our transport (`melee_net::transport`/`wire`) is raw UDP with its own 18-byte LE header (magic 0x4E47).** None
  of it carries over. We need an ENet implementation that interoperates with ENet 1.3.x (the protocol, checksum
  off, connect/verify-connect, reliable plus unsequenced, throttle and ping). The candidates are the Rust crate
  `rusty_enet` (a pure-Rust ENet 1.3 port) or binding C ENet (MIT).

### 1.2 Messages (NetPlay::MessageID, all `sf::Packet`: integers big-endian, bool = 1 byte, string = u32 len + bytes)

| id | name | body | sender / use |
|---|---|---|---|
| 0x80 | SLIPPI_PAD | s32 frame (newest), u8 player_idx (port 0-3), s32 checksum_frame, u32 checksum, then N x 9-byte pads newest first (header = 14 bytes) | every frame (`SendSlippiPad`); the client resends all unacked frames, up to 128 |
| 0x81 | SLIPPI_PAD_ACK | s32 frame, u8 player_idx | only when the pad packet added new frames (`inputs_to_copy > 0`) |
| 0x82 | SLIPPI_MATCH_SELECTIONS | u8 char, u8 color, bool char_selected, u8 player_idx, u16 stage, bool stage_selected, u32 rng_offset, u8 team, u8 alt_stage_mode | CSS/SSS (`writeToPacket`) |
| 0x83 | SLIPPI_CONN_SELECTED | (empty) | `SendConnectionSelected` |
| 0x84 | SLIPPI_CHAT_MESSAGE | s32 message_id, u8 player_idx | premade chat (ids checked in `ReadChatMessageFromPacket`) |
| 0x85 | SLIPPI_COMPLETE_STEP | u8 step, u8 char, u8 color, u8 stage0, u8 stage1 | game-prep steps (ranked stage strike / counterpick, `CMD_GP_*`) |
| 0x86 | SLIPPI_SYNCED_STATE | u8 player_idx, string match_id, u32 game_idx, u32 tiebreak_idx, u32 seconds_remaining, 4 x (u8 stocks, u16 health) | ranked desync recovery (`handleReportGame` with end method 7) |
| 0x87 | SLIPPI_PAD_SUGGESTION | u8 player_idx, s32 target_frame, u32 seq, 8-byte pad | early-input "suggestions" from the GC adapter thread (Rust `input-suggestions` crate) |
| 0x88 | SLIPPI_PLAYER_DROP | u8 sender, u8 dropped port, s32 last frame, u8 count, count x pad | 3-4 player games: a dropped player's last inputs (`SLIPPI_DROP_REPORT_INPUT_COUNT` 64) |

`tools/rust-port/slippi/slippi_wire.py --selftest` encodes and decodes these payloads (a codec only, no socket).

### 1.3 Pads, frames, delay and the rollback window
- **Pad formats** (`SlippiPad.h`): `SLIPPI_PAD_INPUT_SIZE` 8 (buttons, sticks, triggers: the game's own pad bytes
  as the ASM gives them); `SLIPPI_PAD_WIRE_SIZE` 9 = the 8 bytes plus **1 random byte**; `SLIPPI_PAD_FULL_SIZE` 12
  (what EXI hands back to the game). `prepareOpponentInputs` ends by appending `Common::Random::GenerateValue<u8>()`:
  *"Random bits for the ASM to send with the next pad, it mixes them into the RNG seed."* **Correction (from the ASM,
  slippi-online-patches.md 1.2-1.3):** the seed is overwritten every frame with `fmix32(rotl(frame,16) + rng_offset)`;
  the pad bytes are XORed into its top byte **only on a frame where a Peach / Game & Watch / Luigi player freshly presses
  B with the stick on the random special** (then the other players' bytes, so nobody can pick a good seed). On every
  other frame the seed depends on the frame and `rng_offset` only. The mixing is in `TriggerSendInput.asm` section 11
  and `InitOnlinePlay.asm` (`FN_SyncRNG`). Our `--live-rng` rule (`seed + ((frame+123) << 16)`) is a different,
  GD-specific scheme. Note the pad's byte 8 is the bits its **owner** drew; Dolphin draws the local byte for the
  *next* pad (it comes back to the game in the RXB).
- **Frames** start at 1 (`handleOnlineInputs`: `frame == 1` resets everything). The game sends over EXI
  (`CMD_ONLINE_INPUTS` 0xB0): s32 frame, s32 finalized_frame, u32 finalized_frame_checksum, u8 delay, 12 pad
  bytes, u8 input_source_port, u8 is_leaving.
- **Delay**: on frame 1, `handleSendInputs` queues `delay` empty pads for frames 1..delay. After that each local pad
  is sent as frame `frame + delay`. The delay comes from `Config::SLIPPI_ONLINE_DELAY` (`prepareDelayResponse`,
  `CMD_GET_DELAY`). Each side picks its own delay, and the two do not have to match.
- **Rollback window**: `ROLLBACK_MAX_FRAMES` 7 (ours defaults to 7, max 12). The stall rule is in
  `shouldSkipOnlineFrame`. A frame halts when `latest_remote - finalized < frame - finalized - 7`. After 7 s of
  stalls (`60*7` frames) the peer is force-dropped.
- **EXI response to the game** (`prepareOpponentInputs`) has this layout:
  1. control byte: 1 run, 2 skip, 3 disconnected, 4 advance;
  2. remote count;
  3. 3 x (checksum_frame, checksum);
  4. 3 x latest frame;
  5. the minimum of those;
  6. 3 x 7 x 12 pad bytes;
  7. 3 x despawn frame;
  8. 3 x 7 x 12 predicted pads;
  9. 1 random byte.

  **Prediction rule**: repeat the latest pad. `slprs_input_suggestions_fill_predictions` can replace it with
  suggested inputs. It is a prediction only, so it cannot desync a match, but it changes how many rollbacks happen.
- Savestates: Dolphin keeps 7 `SlippiSavestate`s (`handleCaptureSavestate`/`handleLoadSavestate`,
  `CMD_CAPTURE_SAVESTATE`/`CMD_LOAD_SAVESTATE`). These are memory copies of `full_backup_regions`
  (`SlippiSavestate.cpp`: 0x80005520.., 0x803B7240-0x804DEC00, the heaps, 0x80BD5C40-0x811AD5A0), minus
  `exclude_sections` (sound, pad, EXI buffers). The **ASM** decides when to save, load and resimulate
  (`Online/Core/LoopEngineForRollback.asm` @801A5014, `StartEngineLoop.asm` @801A4DE4,
  `ForceEngineOnRollback.asm` @801A4DB4, `SkipNewInputFetchOnRollback.asm` @80376A20,
  `Online/Static/SaveState.asm`/`LoadState.asm`). Our snapshot-in-place rollback is equivalent as long as the
  confirmed inputs give the same state. The peer cannot see how we roll back, only our checksums.

### 1.4 Time sync (`OnData` pad path, `CalcTimeOffsetUs`, `shouldSkipOnlineFrame`, `shouldAdvanceOnlineFrame`)
- When a pad arrives the client computes `offset = (now - ping/2) - last_local_send_time + 16683 * (local_frame -
  remote_frame)`. It keeps a ring of 30 samples per peer (`SLIPPI_ONLINE_LOCKSTEP_INTERVAL`). The offset is the
  mean of the middle third of the sorted ring, and the minimum over peers. Ping comes from ack round trips
  (`ack_timers`).
- **Skip** (we are ahead) runs only on frames that are multiples of 30 and at most 120: if the offset is over
  10 ms, skip `(off-10ms)/16683+1` frames, at most 5.
- **Advance** (we are behind) runs on multiples of 30. If the offset is under -(16683+10000) and the frame is over
  120, run up to 3 extra frames, one every 5 frames.
- The client also nudges emulation speed by up to +1% / -0.5% (`MAIN_EMULATION_SPEED`). Our engine's frame pacing
  has to expose the same nudge, or keep the peer from stalling some other way.
- Bot accounts (`is_bot` in the mm result) may run ahead (`opponentRunahead`).
- None of this has to match bit for bit. It only has to keep the peers within the window, and our
  `melee_net::early`/session timing can do that. Getting it badly wrong makes the Slippi peer stall. It cannot
  cause a desync.

### 1.5 Selections, stage and chat (`prepareOnlineMatchState`, `setMatchSelections`)
- Each side sends `MATCH_SELECTIONS`. `rng_offset = generator() % 0xFFFF` per side, and the match uses **the
  decider's (host's)** value (`rng_offset = is_decider ? lps.rng_offset : rps[0].rng_offset`). The stage comes
  from the first selection in port order that has `is_stage_selected`, else Battlefield (0x1F). A random-stage
  pick comes from `getRandomStage()` over the mm `stages` list.
- **Dolphin builds the game info block itself**: the static `online_match_block` with these settings:
  - stock mode 0x32, 4 stocks, 8:00 timer;
  - pause bit set except in direct;
  - items 0xFF (off) unless the mm result sets `items`; item bits 0xF80000000F000000;
  - character, color, team and costume index per port; ports 3-4 off in 1v1;
  - stocks and health from the desync recovery state.

  Our engine has to build the same block, or the start state differs.
- Fixed-rules modes (ranked/unranked) reject characters >= 26 and stages outside the list. Direct and teams allow
  a stage pick, `alt_stage_mode` (frozen PS toggle) and pause.
- Chat: the 16 premade messages come from the user record (`chatMessages`). Only the s32 id is sent.

### 1.6 Handshake and version checks
- **There is no version handshake at the netplay layer.** After ENet connects, the peers just exchange
  selections. Compatibility is enforced entirely by the **matchmaking server**: `Join.appVersion` =
  `Common::GetSemVerStr()`, and an `error` reply can carry `latestVersion` (`client.rs handle`). Two clients the
  server pairs are trusted to run the same codeset. The Gecko list is not exchanged; `CMD_GECKO_LIST` only goes
  into replays.
- So a Rust client is accepted by any peer the server pairs it with. Whether it then stays in sync depends only
  on whether our sim matches theirs.

### 1.7 What is synced or verified
- **Inputs only.** The only check is the 32-bit **checksum** for `finalized_frame`, which the game ASM computes and
  passes to EXI in `CMD_ONLINE_INPUTS`. Each pad packet carries it. The game receives the peers' (frame, checksum)
  pairs back, and the **ASM compares them**. Neither C++ nor Rust in Dolphin compares checksums.
- **What the checksum covers** (slippi-online-patches.md 2.2, `StartEngineLoop.asm` `FN_COMPUTE_CHECKSUM`): an XOR of
  each fighter entity's action state, position x/y bits, percent bits and spawn number plus each slot's stocks, folded
  to 16 bits, joined with the low 16 bits of the truncated f32 sum of positions and percents. A **hard** desync is a
  float-sum difference of more than 1 (the game ends); a different high half only shows "Desync Risk". A mismatch shows up
  as a desync. In ranked, `SYNCED_STATE` then restarts the game at merged stocks, percent and timer
  (`GetDesyncRecoveryState`: timer within 1 s, stocks equal, percent within 25, take the lower).
- Our NHV1 word (`docs/rust-port/notes/nhv1.md`) is **our own** curated hash. It is not Slippi's checksum.

## 2. The online game patches (Gecko), and what each one does to gameplay

`python -I tools/rust-port/slippi/gecko_inventory.py <GALE01r2.ini> --md` lists every entry with its opcode and
address. The INI has 7540 code lines. The groups always on online are `Required: General Codes`,
`Required: Slippi Recording`, `Required: Slippi Online`, `Recommended: Normal Lag Reduction`,
`Apply Delay to all In-Game Scenes` and `Lagless FoD`. Six visual codes are optional.

**Codes that change the simulation. Our engine must reproduce each exactly:**

| code (address) | effect | ours |
|---|---|---|
| UCF 0.84: Dashback @800C9A44, Shield Drop @800998A4 / Extended @8009A0B8, Pad Buffer + 1.0 Cardinals @8006B460, SDI @8008E54C, Shield SDI @80093294, Tumble @800908F4, DBOOC SquatRv @800D65EC | controller fixes | `ucf` code (gameplay-codes.md); per-site parity with these exact versions not proven online |
| `External/NeutralSpawn` @8016E510 | spawn positions by port/stage | `neutral-spawn` |
| `External/FreezeGlitchFix` write @801239A8 | Ice Climbers freeze fix | `freeze-fix` |
| `External/PreventWobbling` @800DB880/@800DBBD4/@8008F090 | breaks a grab after repeated pummel-hit cycles | `no-wobble` (may be gated by mode in the ASM; unknown) |
| `Common/NanaDeterminism` @800AC5B8 | makes Nana's AI deterministic. **Essential for any IC match** | not ported |
| `Common/PSCameraIndependentMonitor` @801D24FC | keeps the PS monitor off the camera, so it cannot cause a desync | not ported |
| `Common/FastForward/DynamicsFix` @8009E090 | cloth/hair dynamics during resimulated frames | check |
| `Online/Core/FreezeDeadUpFallPhysics` (3 sites @800D4C1C, @800D4D68, @80080E80) | the star-KO fall | not ported |
| `Online/Core/BrawlOffscreenDamage` @8006A880 | offscreen damage rule | not ported |
| `Online/Core/LGLExceededGameEnd` @802F70C4, `CustomizeMessageLRAS`, `ReportNoContestLRAS`, `Remember Who LRA Started` | game-end rules (ledge-grab limit, LRA+Start) | `lgl` partly |
| `Online/Core/InitOnlinePlay` @8016E748 (186 lines), `InitPause` @8016E904, `PauseCounter`, `PreventPauseStranding`, `Teams/PreventDeadStranding` | match init and pause rules | not ported. **Defines the start state** |
| `Online/Core/Hacks/FD/DesyncProofBGTransformations` @8021AAE4, `Hacks/Stadium/*` (IngameCheckIfFrozen @801D457C, GrPsxIsValid, CustomZeroBuffer, StadiumFileLoad) | stage-hazard and background determinism | frozen-ps partly |
| `Online/Core/WhispyBlowDirFix` @8008653C | Dreamland wind direction | not ported |
| `Online/Core/Sound/*`, `Music/*`, `PreventFileAlarms`, `HandleRumble`, `PreventPadAlarmDuringRollback`, `PreventCharacterCrowdChants` | audio and rumble that rollback must not double-trigger | not gameplay. **Unless** the sound IDs feed the RNG |
| `Online/Core/TriggerSendInput` @80376A28, `StartEngineLoop`, `LoopEngineForRollback`, `ForceEngineOnRollback`, `ForceInputRefetchOnAdvance`, `SkipNewInputFetchOnRollback`, `Static/SaveState`/`LoadState`/`LoadMatchState` | the rollback engine, the **checksum** and the **RNG mixing** | must be read to get the checksum and seed rule |
| `Polling Drift Fix + VB`, `Normal Lag Reduction`, `Apply Delay to all In-Game Scenes` | polling and timing | pad sampling point only |
| `General Codes`: 4 stocks, 8 min, no items, singles stages, skip memcard, unlock all, Salty Runback, CSS tweaks | rules and menus | the rules come in through the match block |
| `Online/Menus/*` (about 90 entries), `Online/Slippi Online Scene/*`, `Online/Static/*UI*` | the online CSS/SSS/results UI | menu-only; our own UI can stand in, **as long as it sends the same EXI-equivalent data** |
| `Slippi Recording/*` | `.slp` writing | we already export `.slp` |
| m-ex compatibility hooks | m-ex support | vanilla: hooks idle |

**The EXI link.** The game talks to the "Slippi" EXI device in slot B (`CEXISlippi::DMAWrite`, command bytes
0xB0-0xE5 in `EXI_DeviceSlippi.h`). For our engine this is just an internal API: our "ASM" is Rust, so we do not
emulate EXI. We only need its **semantics**: what `CMD_ONLINE_INPUTS`, `CMD_GET_MATCH_STATE`, `CMD_GET_NEW_SEED`
(`generator() % 0xFFFFFFFF`) and `CMD_REPORT_GAME` feed in and return.

## 3. Matchmaking and accounts (source only)

### 3.1 Accounts
- The account is `<Dolphin user dir>/Slippi/user.json` (`FileUtil.cpp` `D_SLIPPI_IDX` + `USER_JSON`,
  `EXI_DeviceSlippi.cpp` line 153; `user/src/lib.rs`). Its fields are `uid`, `playKey`, `connectCode`,
  `displayName` and `latestVersion` (plus `chatMessages`). It is written by the website login
  (`https://slippi.gg/online/enable?path=<user.json path>`) and **nothing else**. The client has no password and no
  OAuth.
- **On load**, `attempt_login` calls `overwrite_from_server`: an HTTP GET to
  `https://users-rest-dot-slippi.uc.r.appspot.com/user/<uid>?additionalFields=chatMessages,rank` (plus the rank
  fetcher). **Starting Dolphin with a user.json contacts Slippi's servers before you ever go online.**
- `playKey` is the credential. It goes to matchmaking in `Join.user` and to game reports.

### 3.2 Matchmaking (`matchmaking/src/client.rs`, `protocol.rs`, `SlippiMatchmaking.cpp`)
- WebSocket `wss://matchmaking.slippi.gg/v1/queue`. A build whose version contains "dev" uses
  `matchmaking-dev.slippi.gg`. **`SLIPPI_MM_URL` overrides the URL**, meant "for local testing against a service
  started with authentication disabled" (`URL_ENV_VAR`).
- Flow:
  1. STUN on the netplay socket. The servers (`SlippiMatchmaking.h` `STUN_SERVERS`) are Google, Cloudflare and
     Twilio; two answers classify the NAT.
  2. Client sends `{"type":"join", user{uid,playKey,connectCode,displayName}, search{mode, connectCode: Shift-JIS
     bytes}, appVersion, port, ipAddressLan, candidates[host|srflx|observed], natType}`.
  3. Server replies `joined{ticketId, token}`, then `status` every few seconds. The client can send `resume` and
     `ping`.
  4. Server sends `match{matchId, isHost, players[{uid, displayName, connectCode, port, isLocalPlayer, ipAddress,
     ipAddressLan, isBot, chatMessages, rank}], stages, items}` or `error{error, latestVersion}`.
- Modes: 0 ranked, 1 unranked, **2 direct**, 3 teams, 4 party.
- After the game: `slprs_exi_device_log_game_report` to `https://internal.slippi.gg` (GraphQL, `slippi-gg-api`).
  A report is sent for every online game, direct included (`handleReportGame` has no mode check, only
  `LOCAL_TESTING`). It carries uid, playKey, matchId, per-player stats and the ISO MD5 (`game-reporter/iso_md5_*`).
  Ranked also reports match status ("connecting").
- `LOCAL_TESTING` (a commented `#define` in `EXI_DeviceSlippi.cpp`) replaces the netplay client with a dummy one
  (`SlippiNetplayClient(true)`) and fakes the opponent. It does not exercise the network.

### 3.3 Licence and terms (plain)
- Slippi Dolphin and the Rust extensions are **GPL-2.0-or-later** (`COPYING`, `Externals/SlippiRustExtensions/LICENSE`).
  We may read them and reimplement the protocol. If we copy code (for example the asm or the C++), our engine has
  to be distributable under GPL-compatible terms, and the C++ comments and tables count as code. `slippi-ssbm-asm`
  is not in the workspace, so its licence was not checked.
- **The servers are a different matter.** The GPL covers code, not the right to use `matchmaking.slippi.gg`,
  `users-rest`, or `internal.slippi.gg`. **The source has no terms text at all.** I did not fetch slippi.gg's terms,
  because that would contact a Slippi service. Before any real-server use the user has to read Slippi's current
  terms or ask the Slippi team (Fizzi) directly. The protocol carries the user's real `playKey`. A third-party
  client that desyncs, or that reports games with a non-Slippi ISO MD5, would **affect other players and the
  ranked ladder**. The plan below keeps ranked and unranked off entirely and treats direct-only use as needing
  permission.

## 4. Determinism requirements against Dolphin, mapped to our oracle work

The bar is that, given the same confirmed inputs and the same per-frame seed, our post-frame state equals the
**patched** console's at every frame, and our checksum word equals the ASM's.

| requirement | where we stand (dolphin-oracle.md, slpcheck.md, net.md) |
|---|---|
| vanilla sim bit-exact vs the console | the Dolphin NHV1 oracle (`tools/rust-port/dolphin/oracle.sh`): **74 replays Dolphin-exact**; slpcheck **104/125 frame-exact** on the 3.17 corpus. Slippi's online codes were on in those replays, so this corpus *is* the online sim (UCF, neutral spawn, Nana, wobble rules) for those characters and stages. Every divergence left is a potential online desync |
| online codes | UCF / neutral-spawn / freeze-fix / no-wobble / frozen-PS exist as `--codes`; Nana determinism, PS camera monitor, star-KO fall, offscreen damage, Whispy fix and InitOnlinePlay are not ported |
| start state | build Dolphin's `online_match_block` exactly, then the InitOnlinePlay ASM's effects, then the seed |
| RNG | known (slippi-online-patches.md 1); `melee_core::slippi_online` + the SyncRNG proc, off by default. Unit-tested, not yet against a live peer |
| checksum | known (slippi-online-patches.md 2); Rust equals Slippi Dolphin's memory on all 11293 frames of 01c6d260 (section 9) |
| rollback-safe statics | done for our own netplay (net.md rule on statics, `rollback_pinned`) |
| FMA/float | done (fma.md); a vanilla console has no FMA contraction. The oracle work handled this |
| oracle to verify | 1) the existing `.slp` oracle covers sim parity. 2) for the checksum: run the patched online game in our local Dolphin build with the NHV1 hook and log the ASM checksum for a known input stream (section 6.4) |

## 5. Gap analysis against our engine

| area | have | missing for Slippi |
|---|---|---|
| transport | `melee_net` raw UDP, relay (`melee_relay`), hole punch `GDPH`, STUN, LAN pick | ENet 1.3 interop; sf::Packet BE encoding; the 9 message types; port binding as advertised |
| rollback | `RbSession`, window up to 12, snapshots, SyncTest 0 mismatches, real-UDP proofs with lag/loss | window 7 to match; pad queue/ack semantics (resend unacked, ack only on new); halt rule; delay-by-dummy-pads rule |
| early inputs | `melee_net::early` | maps onto `PAD_SUGGESTION` (optional) |
| timesync | our own pacing | Slippi's skip/advance/speed rules, or close enough that a Slippi peer never stalls |
| match setup | our lobby (`J c0 c1 scene`), lockstep CSS (`--np-css`) | Slippi selections exchange; online_match_block; rng_offset; stage list from mm; chat ids |
| sim | 104/125 slpcheck, 74 Dolphin-exact | the rest of the corpus; online-only codes; IC/Nana; hazards; the **RNG mix and checksum** |
| matchmaking | `melee_net::app::rdv` (our server) | WebSocket mm client (join/resume/ping/match), STUN candidates, user.json loading, game report (or deliberately not reporting) |
| `.slp` | exporter, 3 readers agree; peers' exports byte-identical | Slippi online fields (netplay names/codes, matchId, gecko list, online playedOn). Replays from a mixed match must match Slippi's own `.slp` frame for frame, and the same slpcheck/oracle can verify that |
| Hero's launcher | `--replay-dir` seam | his `mu_net` is a third protocol (net.md); Slippi play is a separate entry point |

## 6. Single-machine test plan with two accounts

### 6.1 What a direct-code match between two local Dolphins touches (from the source)
1. **At start-up**, per instance with a user.json: HTTP GET to `users-rest-dot-slippi.uc.r.appspot.com/user/<uid>`
   (user info and rank). The rank fetcher may also call `internal.slippi.gg`. Update checks may run too.
2. **On "Direct"** with the other account's code: three STUN queries go to Google, Cloudflare and Twilio, then a
   WebSocket to `matchmaking.slippi.gg` with uid, **playKey**, codes, appVersion, the LAN IP and the public IP.
   **Yes: direct connect goes through Slippi's matchmaking server.** Neither the client nor the mm result has a
   type-in-an-IP path.
3. **Match**: the server returns both players. Both share an external IP, so each uses the other's
   `ipAddressLan` (`SlippiMatchmaking.cpp` second loop). The ENet game traffic then stays **on the LAN interface,
   inside this machine**. It does not go through loopback, because the LAN IP is the adapter's. No relay is
   involved.
4. **After each game**: a game report to `internal.slippi.gg` (uid, playKey, matchId, stats, ISO MD5) per instance.

So the traffic leaving the machine is the user API, STUN, the matchmaking WebSocket and the game reports. The
gameplay itself stays local.

### 6.2 Pointing each Dolphin at a different profile
Dolphin's `-u <user dir>` (or `portable.txt` next to the exe) selects the user directory. The account is
`<user dir>/Slippi/user.json`. Two instances means two user dirs, each with its own `Slippi/user.json` (the user
copies their own profile files there; no agent touches them). Each instance also needs its own `Dolphin.ini`
netplay port, `SLIPPI_ONLINE_DELAY`, a window title, and audio at 3%. Our oracle runner already uses `-u`
(`run_dolphin.ps1`).

### 6.3 Terms implications
Two real accounts playing each other on one machine is ordinary use of Slippi, which players do anyway when
testing. It sends real reports, so use direct mode only (no rating). Putting **our** engine on one side changes
that: a non-Slippi client then talks to Slippi's servers with a real playKey, and its game reports carry a wrong
ISO MD5 or a desynced result. That needs the user's decision and, I'd say, Slippi's OK (section 3.3).

### 6.4 Server-free tests that are allowed now (no Slippi contact)
1. **Mock matchmaking**: a local WebSocket server (`tools/rust-port/slippi/`, to be written) that answers `join`
   with `joined` and pairs two direct searches into `match` JSON pointing both at 127.0.0.1. Point Dolphin at it
   with `SLIPPI_MM_URL=ws://127.0.0.1:PORT/v1/queue`. Blockers:
   - The client needs a user.json to show the online menu, and loading one calls users-rest. Use a **dummy**
     user.json (fake uid, no real key) in a throwaway user dir, plus a local patch in our oracle Dolphin build
     (`slippi-dolphin-nhv1.patch`, local only) that skips `overwrite_from_server`, the rank fetch and the game
     reporter.
   - Windows Firewall outbound block rules for the two exe paths as a belt and braces.
   - STUN: with no answers, the client "will use the address it observes", which is fine against the mock.
   - Both mock players get `ipAddress` 127.0.0.1 with different ports, so the ENet traffic is pure loopback and
     can be captured (`pktmon` or Wireshark on the loopback adapter).
2. With (1): **Dolphin vs Dolphin** on loopback with scripted pads (the oracle's comm-file style input or Dolphin's
   pipe input, never OS input). Capture the pad packets, decode them with `slippi_wire.py`, and log the checksum
   the ASM sends per frame. This gives the **checksum and RNG oracle** without the ASM source.
3. Then **Rust vs Dolphin** on the same mock: phases 2-4 below.

## 7. Phased build plan

| phase | work | estimate | needs approval |
|---|---|---|---|
| 0 | get `slippi-ssbm-asm` source (GitHub clone, read only) and pin the asm commit that matches the INI; read `TriggerSendInput`, `StartEngineLoop`, `LoopEngineForRollback`, `InitOnlinePlay`, `SaveState`; write the checksum and seed rule down | 2-3 days | **yes**: network fetch of the repo |
| 1 | mock mm server + Dolphin patch (skip user API and reports) + dummy profiles; Dolphin vs Dolphin on loopback; packet capture; per-frame checksum/seed log from the NHV1 hook | 3-4 days | yes: firewall rules, the Dolphin rebuild (about 6 GB already there) |
| 2 | `melee_net::slippi`: ENet interop (`rusty_enet` or C ENet), sf::Packet codec, the 9 messages, pad queue and ack, time sync; unit tests against captures from phase 1 | 1-2 weeks | adding a dependency |
| 3 | online game state: `online_match_block`, the InitOnlinePlay effects, the RNG mix, the checksum; port the online-only gameplay codes (Nana, star-KO fall, offscreen damage, Whispy, PS camera, stage hacks) as an "online" code preset | 2-4 weeks | none |
| 4 | Rust vs Dolphin via the mock: 1v1 Fox/Falcon FD, then the slpcheck-exact characters and stages; zero desyncs over the corpus inputs; `.slp` from both sides compared | 1-2 weeks | none |
| 5 | close the sim gaps (the 21 non-exact slpcheck files, IC, items off) that online play hits | open-ended | none |
| 6 | real matchmaking client (WebSocket join, STUN candidates, user.json read by the engine at run time only), **direct mode only**, no reporting or honest reporting | 3-5 days | **yes**: ToS / Slippi permission; first real-server contact; credential handling |
| 7 | launcher entry for Hero (melee-unlocked) | 2 days | yes: anything sent to Hero |

## 8. Hardest blockers
1. ~~The checksum and the RNG mixing~~: specified and implemented (section 9). What is left is checking the seed
   against a live Slippi peer, which needs the loopback Dolphin test of section 9 to run.
2. **Sim parity at the patched-console level** for every character and stage played online: 21 of 125 files still
   diverge, and the online-only codes are unported.
3. **Server terms and credentials.** Direct play still needs Slippi's matchmaking and a real playKey, and game
   reports go out automatically. That is a permission question, not an engineering one.
4. **Version drift.** The server gates on `appVersion` and the codeset changes between Slippi releases. Our client
   has to follow each release's asm, or the server stops pairing it (or pairs it with peers running different
   code).
5. ENet interop, an unfamiliar but bounded dependency.

## 9. Phase 0/1 results (2026-11-06, port/slippi-p1)

**Source.** `project-slippi/slippi-ssbm-asm` `d0ce23a` cloned read-only to `_build/slippi-src/ssbm-asm` (not in our
repo). The Dolphin clone's online INI carries the compiled new seed scheme (murmur constants), so it is `e03af7f` or later.
Spec: `slippi-online-patches.md` (checksum, seed, every gameplay-affecting online patch with addresses).

**Rust.** `melee_core::slippi_online` (arithmetic + state, `enable(rng_offset)` turns it on, default off),
`melee_fighter::slippi_online::checksum()`, the InitOnlinePlay seed and SyncRNG proc in `fn_8016E730` (only when on),
`melee slpcheck --slippi-checksum-out`. Default gameplay is untouched: slpcheck 104/125 frame-exact before and after.

**Checksum vs Dolphin.** The Dolphin oracle's per-frame memory records (`run_dolphin.ps1`) of `20476_01c6d260`
through `tools/rust-port/slippi/checksum_nhraw.py`, against `slpcheck --slippi-checksum-out` on the same replay:
11293 of 11293 frames equal (high and low halves). The NHV1 differences of that replay (the -0.0 knockback velocity
from frame 1388) are not in the checksum's fields.

**Loopback Dolphin vs Dolphin: built, not run.** Prepared:
- `_build/dolphin-netplay/`: a second link of the oracle build (the oracle exe was in use by another run) with the Slippi
  `Data/Sys` (online codeset), and a local patch (`SLIPPI_LOCAL_TEST=1`): the Rust web client refuses every request
  before DNS (user API, rank, game reports), the matchmaking socket may only reach loopback, no STUN servers (no DNS,
  no packets), LAN address `127.0.0.1`, both ENet hosts bound to `127.0.0.1`, no login/update browser pages,
  `SLIPPI_PIPE_PREFIX` for per-instance pad pipes, `SLIPPI_CHECKSUM_LOG` (every checksum sent and received),
  `MELEE_DOLPHIN_STATELOG` (scene/memory per frame for closed-loop menu driving). Spectator server off in the config.
- user dirs with generated test profiles (`user.json` with fake uid/playKey; no real account read or used).
- `tools/rust-port/slippi/mock_mm.py`: WebSocket mock (join -> joined/status, pairs two joins into `match`) and a
  loopback UDP relay that records every ENet datagram (the probe); `run_np_dolphin.ps1` starts one instance.
- Blocker: the netplay instance exits with an unhandled C++ exception (0xE06D7363) a frame after boot, before Dolphin's
  log file opens. Reading its console log to find the throw was refused by the permission system, so the run stopped
  there. Nothing reached the network (every socket path is loopback-only or refused).

**Protocol claims of sections 1-3 not confirmed on the wire yet** (no capture). Corrected from the ASM: the RNG rule
(1.3), what the checksum covers and how it is compared (1.7).

**Phase 2 blockers (ENet + our engine as a peer).**
1. The loopback capture above (the exception, then menu driving over the pipes: online menu, direct, a code, CSS lock,
   stage), to get real pad/ack/checksum packets and confirm the frame numbering (pad frame F vs `gm_80479D58.unk_8`,
   checksum "frame F = state entering F").
2. ENet 1.3 interop (rusty_enet or C ENet) and the sf::Packet codec; the 9th pad byte drawn per sent frame.
3. The start state: Dolphin's `online_match_block` + InitOnlinePlay (match block copy, pad clear at 0x804C20BC, seed),
   and the online-only gameplay codes still missing (Nana determinism, PS monitor, star-KO fall, offscreen damage,
   Whispy, FD background seed save, init stage/player data).
4. Wiring `slippi_online` into our rollback session: the seed-bit history and B mask are game state (snapshot them),
   and section 1.3 must run on resimulated frames with the confirmed pads.
5. Sim parity: 21 of 125 slpcheck files still diverge.

## 10. Phase 2: the Slippi transport in Rust (2026-11-07, port/slippi2)

**Code.** `melee_net::enet`: our own ENet 1.3 host (connect / verify-connect with session ids, simultaneous connects
both ways, reliable in-order channels with acks and retransmission, unsequenced packets with the 1024-group window,
ping, throttle-configure / bandwidth-limit accepted, disconnect with a reason word; no compression, checksum or
fragments, none of which Slippi uses). `melee_net::slippi`: the `sf::Packet` codec, MATCH_SELECTIONS / PAD / PAD_ACK /
CHAT, and `Client` (Slippi's 1v1 netplay client: pads unsequenced on channel 1, acks on channel 2 and only for pads
that added frames, the local queue resent until acked and capped at 128 frames, ping from acks, `CalcTimeOffsetUs`,
duplicate-link pruning by the lower port). `melee slippi-netplay --host|--join --port P --peer IP:PORT`: one peer on
the native engine with Slippi online mode on (`slippi_online::enable(decider's rng_offset)`), our rollback session
(window 7), Slippi frame numbering (online frame 1 = session `FIRST`), the checksum of the state entering each frame
sent with every pad for the newest finalized frame and compared with the peer's (`slippi_online::compare`), Slippi's
skip rule on multiples of 30. `--padgen SEED` uses the session's pad generator instead of the 1800-frame script.
`tools/rust-port/slippi/np_pair.sh BASEPORT OUTDIR [args]` runs two peers on loopback and compares their CSVs.
Direct and local only: no matchmaking, STUN or Slippi service.

**Engine vs engine (loopback, inbound lag/loss on both peers, Falcon vs Fox FD).** Every run: 0 soft, 0 hard
desyncs; both peers' finalized checksum CSVs equal on every common frame.

| pads | frames | delay | lag ms | loss % | compared | rollbacks (host) |
|---|---|---|---|---|---|---|
| script | 2489 | 2 | 40 | 5 | 2207 | 79 |
| script | 10889 | 2 | 60 | 10 | 8770 | 79 |
| script | 7288 | 0 | 25 | 20 | 4771 | 79 |
| script | 7288 | 4 | 100 | 3 | 5540 | 79 |
| padgen 777 | 3688 | 2 | 40 | 10 | 2964 | 3274 |
| padgen 4242 | 28888 (8 min) | 2 | 60 | 10 | 23022 | 25163 (max depth 7, 845 stall ticks) |

The script's pads stop at 1800 frames (the state is constant from online frame 1832), so the padgen runs are the
real long-match evidence (25075 distinct checksums in the 8-minute run). `cargo test -p melee_net --test slippi`: the ENet client pair
over real loopback UDP, clean and with 10% loss + 30 ms.

**Not done.** Rust vs Slippi Dolphin. The patched netplay Dolphin (`_build/dolphin-netplay`) did start and stay up this
time (no 0xE06D7363 within 20 s, empty stdout/stderr), but stopping that process was refused by the permission
system, so no further Dolphin run was made (it needs the mock matchmaking plus menu driving over the pad pipes:
online menu, direct, a code, CSS, stage). So: no wire capture against real ENet, and the frame-numbering and
"checksum of the state entering F" assumptions are still unconfirmed against Dolphin. Also open: the seed-bit mixing
(TriggerSendInput section 11) is not wired (seed bits stay 0, right except on a fresh Peach/G&W/Luigi random special),
the online-only gameplay codes and `online_match_block` (section 9, blockers 3 and 5), time sync beyond the skip rule.

## 11. Phase 3: engine vs Slippi Dolphin, mock matchmaking (2026-11-07, port/slippi3)

**Setup (loopback only, no Slippi service).** `_build/dolphin-netplay` (the oracle's local-test link) with a throwaway
test profile `userP3A` made for this test (fake uid, no key; code `TSTA#001`, `Slippi/direct-codes.json` seeded with
`TSTB#001`). `tools/rust-port/slippi/`:
- `mock_mm.py`: the mock matchmaking server. Direct mode pairs a join with the queued join whose own code it searches
  for; `--stages` sets the match's stage list; the relay that both ENet peers talk through takes `--lag-ms`/`--loss-pct`.
- `dolphin_drive.py`: drives Dolphin's menus through its pad pipe, closed loop on the state log (main menu -> Online ->
  Direct -> CSS hand to X,Y + A -> START -> Z (take the autocomplete) -> START/A -> search), then plays scripted random
  pads in game. No OS input.
- `np_dolphin_pair.sh OUTDIR [--lag-ms --loss-pct --frames --pads --css X,Y] [-- engine args]`: mock + Dolphin + driver
  + `melee slippi-netplay --mm ...`; Dolphin joins first (player 1, decider), the engine is player 2.
- `run_np_dolphin.ps1` fixes: the arguments go as one quoted string (an array made `-u` split at the space, Dolphin then
  used `E:\Projects\Melee` as its user dir: no profile, no config, which is why the old runs "stayed up" but took no
  pad input); `[Input] BackgroundInput = True` in the profile (no window focus headless); pipe commands must be LF only.

**Engine.** `melee_net::slippi_mm` (WebSocket client, join/joined/status/match, refuses any non-loopback address).
`melee slippi-netplay --mm ws://127.0.0.1:P/v1/queue --port P --code C --search C [--char EXT --color N --stage EXT]`.
The match is built from both MATCH_SELECTIONS the way `prepareOnlineMatchState` does (`slippi_np::online_scene`:
characters/colors by port, the first selected stage in port order else Battlefield, the decider's rng_offset, stock
4, 8:00, items off), plus the block's remaining fields in `fn_8016E730` (friendly fire, rules +0x9, the item mask, slot
0 / spawn point = index / CPU level 1). Seed bits: TriggerSendInput section 11 now runs inside SyncRNG at its own frame
(`State::sync`) on the pads the driver hands the game, with byte 8 of each port's pad (ours and the peer's) per frame.

**The frame alignment (measured).** Slippi Dolphin's first in-game tick takes no pad: online frame F's pads run in tick
F + 1, and the checksum it sends for frame F is the state after F ticks. Found by logging both fighters' action state
and position per tick from Dolphin (state log at the heap addresses, deterministic per boot) against ours: equal per tick,
while our pads acted one tick earlier. `EngineGame::slippi_pre_tick` runs that first pad-less tick before the session.
Before it, frames ~6-89 (entry) mismatched (hard by the ASM rule) and pads were applied a tick early.

**Results** (Kirby (Dolphin, picked on the CSS) vs Fox (engine), FD; Dolphin's checksums from `SLIPPI_CHECKSUM_LOG`, ours
from `--out`; "compared" is the engine's live compare, "offline" every frame both logged):

| relay lag/loss | engine-side lag/loss | frames | compared live | offline equal | desyncs | rollbacks |
|---|---|---|---|---|---|---|
| 0 / 0 | - | 1290 | 1287 | - | 0 | 0 |
| 30 ms / 5 % | - | 2594 (game ended) | 2262 | 2375 / 2375 | 0 | 220 (depth 7) |
| 20 ms / 3 % | 40 ms / 10 % | 1903 (game ended) | 1406 | 1563 / 1563 | 0 | 191 (depth 7) |
| 60 ms / 10 % | - | 2636 | 1819 | 1993 / 1997 | 4 soft, 0 hard (live) | 334 |

The 60 ms / 10 % run diverged at frame 2544 (float sum off by 1, action-state half different on 4 frames) and Dolphin
then left the game: an open sim or rollback question (not reproduced at lower loss; engine vs engine never showed it).
Engine vs engine through the mock: 0 desyncs (1285 compared).

**Not done / gaps.** The seed-bit mix is wired but not exercised (no Peach/G&W/Luigi fresh roll in these runs). The
run-B divergence. Only FD and Battlefield boot on the engine (`online_scene` refuses others; the mock is run with
`--stages 0x20`). The CSS hand position of each character is not mapped (`--css 0,15` is Kirby). The color "shade" for
mirror matches is not applied. Real-network tests: none (the engine's client only connects to loopback).

## 12. Phase 4: the 60/10 divergence, seed bits, stages, the real server (2026-11-07, port/slippi4)

**The 60 ms / 10 % divergence was a sim gap, not rollback.** Reproduced (run b60-4: 4 soft frames, the same
signature: float sum off by 1, high half different) and then replayed offline from the engine's own trace with
`melee slippi-replay` (no network, no rollback): the same 4 frames differed, while `slpcheck` on Dolphin's replay of
the game matched. The differing field was an offscreen fighter's percent: the 1% offscreen tick
(`dmg.x1910` vs `x7AC`, `Fighter_8006A360`) came 4 frames late. `ftSlippi_IsOffscreen` took the HUD-magnifier path
(`ifMagnify_802FC998`, follows the live camera) because `slippi-netplay` never turned the online codeset on;
Slippi's `BrawlOffscreenDamage` uses the stage camera bounds. Fix: `slippi_np::online_codes` sets `Slippi_Codes() = 2`
(platform hook, `melee_fighter::hooks`, grlast) and the online gameplay bits (UCF, NeutralSpawn, FreezeGlitchFix,
PreventWobbling, LGL 45, frozen PS unless `alt_stage_mode`). UCF must be in the word: setting the codes otherwise
turns the default UCF off (57-736 soft frames in replays). Neutral spawn is what Battlefield needed (spawns were
(0,8)/(0,62.4) instead of (+-38.8, 35.2): hard desync from frame 1 in phase 3's terms).

**Deterministic repro.** `slippi-netplay --trace F`: per finalized frame both pads + the seed byte, the checksum and
its inputs. `melee slippi-replay --trace F --log engine.log [--lag N --drop-pct P --drop-seed S] [--out CSV]
[--trace-out T]`: the run offline, straight or through the rollback session with deterministic late/dropped remote
pads (lag 4-6, 20-40 % "drops": 760-910 rollbacks, depth 7). `np_compare.py OUT NAME` compares every frame with
Dolphin's `SLIPPI_CHECKSUM_LOG`.

**Seed bits under rollback.** A frame whose mix read a pad byte (a port rolled) records the bytes it used; a confirmed
remote pad with a different byte forces a resim (`RbSession::force_resim_from`). Slippi Dolphin sends 0 in byte 8 on
every frame we logged, so against Dolphin it never fired (forced resims 0).

**Live results vs Slippi Dolphin (mock, loopback; after the fix).** All: 0 soft, 0 hard live and offline.

| run | relay lag/loss | Dolphin vs engine | stage | compared live / offline | rollbacks | rolled frames |
|---|---|---|---|---|---|---|
| fx-1..3 | 60 / 10 | Kirby vs Fox | FD | 2273-2409 / 2488-2666 | 372-409 | - |
| fx-4 | 80 / 15 | Peach vs Fox | FD | 2166 / 2473 | 393 | - |
| fx-5 | 100 / 20 | Kirby vs Fox | FD | 1786 / 2092 | 386 | - |
| fx-6 | 30 / 5 | Peach vs Luigi | FD | 2717 / 2844 | 394 | 11 |
| fx-7 | 60 / 10 | Marth vs Peach | FD | 2382 / 2643 | 380 | 0 |
| fx-8 | 60 / 10 | Luigi vs G&W | FD | 2448 / 2695 | 361 | 23 |
| st-7 | 60 / 10 | G&W vs Luigi | FD | 2266 / 2494 | 401 | 30 |
| st-1 | 30 / 5 | Kirby vs Fox | Battlefield | 2593 / 2730 | 141 | - |
| st-3 | 30 / 5 | Ness vs Pichu | Fountain of Dreams | 2525 / 2635 | 371 | - |
| st-4 | 30 / 5 | Falcon vs Marth | Pokemon Stadium | 2436 / 2564 | 153 | - |
| st-5 | 30 / 5 | Marth vs Falcon | Yoshi's Story | 2692 / 2838 | 397 | - |
| st-6 | 30 / 5 | Jigglypuff vs Jigglypuff (shade 1) | Dream Land | 2732 / 2873 | 398 | - |

Pre-fix b60-1..7 (60/10, 80/15, 100/20): six clean, b60-4 the divergence; all eight pre-fix traces replay clean
with the fix. Mirror shade: Dolphin's replay of a Peach mirror has port 2 `sub_color` 1; `online_scene` adds `shade1`.

**Gaps.** `slpcheck` of several online replays reports `state_frame` / `last_attack` differences late in the game
(fields outside the online checksum; seeds are forced there; not investigated). The CSS table
(`dolphin_drive.py --play NAME`, icon centres from mncharsel.c) misses now and then (Fox -> Ness, Kirby -> Peach,
Peach -> no character once): the hand is not settled when A is pressed. Battlefield's Peach mirror has no clean live
run (the one attempt failed at the CSS).

**The real network.** `--mm-real` alone allows a non-loopback server: `wss://` (rustls behind `melee_net`'s `real-net`
feature, `--features real-net`), the account from `--user-json` (a copy; no field is printed; real-server messages are
logged by type only), the netplay socket on 0.0.0.0, `--mm-mode unranked` only with `--mm-leave-after-joined`, never
ranked. One real test was run: an unranked join to `wss://matchmaking.slippi.gg/v1/queue` from a copied profile, which
the server refused ("limited to dev accounts"). The installed production Slippi Dolphin talks to `mm.slippi.gg` /
`mm2.slippi.gg` instead (the older ENet matchmaking); that client is not written. Only one Slippi profile exists in
AppData (`Slippi Launcher/netplay/User/Slippi/user.json`), so a direct engine-vs-Dolphin or engine-vs-engine game
through the real servers (two accounts) was not possible.

## 13. Phase 5: production matchmaking, real-network engine vs engine (2026-11-07, port/slippi5)

**Correction to section 12.** `mm2.slippi.gg` is the *dev* host; production Slippi Dolphin 3.x (the installed 3.6.4)
uses `mm.slippi.gg:43113` (`SlippiMatchmaking.h` `MM_HOST_PROD`, `MM_PORT`, Ishiiruka tag v3.6.4, read as a protocol
reference only). It is ENet, not WebSocket: the client binds its netplay port, ENet-connects (3 channels, data 0) and
sends JSON text as reliable packets on channel 0: `create-ticket{user{uid,playKey,connectCode,displayName},
search{mode, connectCode: bytes}, appVersion, ipAddressLan}` -> `create-ticket-resp{error?}`, then
`get-ticket-resp{error?, latestVersion?, matchId, isHost, players[...], stages, items}` (the same player fields as the
WebSocket `match`). No STUN: the server's view of the source port is the hole punch, so the netplay socket must be the
same port once the mm connection closes. The mainline client (`_build/dolphin-oracle`) moved this to the WebSocket
service, which refuses non-dev accounts.

**Code.** `melee_net::slippi_mm::join_enet` (+ `create_ticket_message`, `local_ip_towards`), over our own
`melee_net::enet::Host`; loopback only unless `Target::Real`. `melee slippi-netplay --mm enet://mm.slippi.gg:43113
--mm-real --user-json COPY --search CODE --port P`. The default stays loopback. Test `tests/slippi_mm_enet.rs` (an
in-test ENet server). `tools/rust-port/slippi/np_real_pair.sh OUT U1 U2 [-- args]` (account copies; codes passed
on the command line, never printed).

**Engine vs engine through the real server (direct mode, two real accounts).** Both peers on this machine: the server
gave both the same external IP, so (as Slippi does) the LAN addresses were used and the game traffic itself stayed on
the machine; matchmaking was real. The lag/loss columns are the engine's own inbound degradation on both peers.
CSVs of both peers equal on every common frame in every run.

| run | pads | stage | lag ms / loss % | frames | compared (dec / guest) | desyncs | rollbacks (dec / guest) | max depth |
|---|---|---|---|---|---|---|---|---|
| r1 | script | FD | 0 / 0 | - (matched, connected; no ISO env in the script) | - | - | - | - |
| r2 | script | FD | 0 / 0 | 2489 | 2481 / 2485 | 0 | 0 / 1 | 6 |
| r3 | padgen 777 | FD | 40 / 5 | 1785 (game ended) | 1583 / 1579 | 0 | 1646 / 1647 | 5 |
| r4 | padgen 4242 | Battlefield | 60 / 10 | 1512 (game ended) | 1219 / 1223 | 0 | 1324 / 1326 | 7 |

**Not done: Slippi Dolphin on the real network (Dolphin vs Dolphin, engine vs Dolphin).** The only build that speaks
the production service is the installed Slippi Dolphin 3.6.4. It has no nogui build and no state log for the closed
loop menu driver, and on Windows its pad pipes are fixed global names `\.\pipe\slippibot1..4` (one instance each),
which a second copy, or the user's own running Slippi Dolphin, would collide with. Driving it would need OS input
(not allowed). The oracle build only speaks the WebSocket service (refused). Options: patch the oracle's Rust
matchmaking to the ENet protocol for a local test build, or a 3.x build with per-instance pipe names.
No stranger (unranked) check was run.

## 14. Phase 6: Slippi Dolphin on the real network (2026-11-07, port/slippi6)

**The build.** `tools/rust-port/slippi/dolphin_enet_mm.patch` (on top of `_build/dolphin-netplay/localtest-cpp.patch`,
mainline `0653df8`): `SLIPPI_MM_ENET=HOST:PORT` makes `SlippiMatchmaking` speak the 3.x ENet protocol itself (the
netplay-port ENet host connects, `create-ticket` as reliable JSON on channel 0, waits for `create-ticket-resp` /
`get-ticket-resp`, then the existing result parsing), no STUN, LAN address from `getLocalAddress` towards the server.
`SLIPPI_REAL_NET=1` lifts the `SLIPPI_LOCAL_TEST` loopback binding (matchmaking and netplay hosts); the web API, game
reports, STUN and browser pages stay off. `SLIPPI_MM_APP_VERSION` sets `appVersion` (the scripts send the engine's
default). Ranked is refused; only message types are logged. Pipe names were already per instance
(`SLIPPI_PIPE_PREFIX`). Built in `_build/dolphin-oracle/build`, the exe/dll copied to `_build/dolphin-slippi6` (not
committed), the oracle's sources and binaries restored afterwards.

**Scripts.** `run_np_dolphin.ps1 -Dir -MmEnet -RealNet -AppVersion`; `np_real_dolphin_pair.sh OUT dd|de|ed|eh [-- engine
args]` (throwaway user dirs `userR1/R2` from copies of the two accounts, `direct-codes.json` holding the other code);
`dd_compare.py OUT` (Dolphin vs Dolphin: each side's TX checksums, and RX against the peer's TX).

**Results (all on the real network: mm.slippi.gg:43113, direct mode, our two accounts; same external IP, so the game
traffic used the LAN addresses).** Dolphin rollbacks are not logged by this build.

| run | pairing (decider first) | pads | engine lag / loss | frames | compared | desyncs | rollbacks (engine) |
|---|---|---|---|---|---|---|---|
| dd1 | Dolphin acct1 (Kirby) vs Dolphin acct2 (Fox) | drive 11 / 22 | - | 2717 / 2720 | 2431 TX common, RX 2561 + 2548 | 0 | - |
| dd2 | Dolphin acct1 vs Dolphin acct2 | drive 11 / 22 | - | 2709 / 2712 | 2696 TX common, RX 2706 + 2700 | 0 | - |
| de1 | Dolphin acct1 (Kirby) vs engine acct2 (Fox) | drive 11 / script | 0 / 0 | 2489 | 2486 live, 2486 offline | 0 | 2 (depth 1) |
| ed1 | Dolphin acct2 vs engine acct1 | drive 22 / script | 0 / 0 | 2489 | 2486 live, 2486 offline | 0 | 0 |
| eh1 | Dolphin acct2 vs engine acct1 (engine queued first) | drive 22 / script | 40 / 5 | 2490 | 2264 live, 2389 offline | 0 | 321 (depth 2) |

**Gaps.** The engine was the guest in every engine-vs-Dolphin run, queued first or not, either account: the server
chose Dolphin as decider each time (an engine decider vs Dolphin is untested here). Dolphin-side rollback counts are
not recorded. No unranked/stranger check was run.

## 15. Phase 7: the engine as decider vs Slippi Dolphin, replays, clean-ups (2026-11-07, port/slippi7)

**How the server picks the host.** The decider is the server's `isHost` (`SlippiMatchmaking.cpp`:
`m_is_host = get_resp.value("isHost", false)`, passed to `SlippiNetplayClient` as `is_decider`). Both clients send
the same `create-ticket` (same fields; nlohmann only reorders the keys), so the server cannot be choosing by client.
In phase 6 the engine's ticket always arrived first: the engine joins at once, and Dolphin needs 20+ s of boot and
menus, so even "Dolphin first" runs had the engine queued first. Measured: **the player whose ticket completes the
pair (the second ticket) is the host.** `np_real_dolphin_pair.sh hd|hd2` waits for Dolphin's driver to start the
search (`drive: searching`, +5 s), then starts the engine. The engine was the decider in every such run. This build
writes no Dolphin log file, so the create-ticket-resp itself is not visible.

**Dolphin-side rollbacks without rebuilding Dolphin.** Slippi Dolphin writes a frame's events again each time it
resimulates it. `slp_rollbacks.py` counts the repeats in its `.slp` (`SaveReplays` on, `ReplayDir` per run in the
throwaway user dir). The patched build was not changed or rebuilt, and the oracle was not touched.

**Replays.** `melee slippi-replay --trace T --log L --slp FILE` records the straight (confirmed-input) run through
`slp_export`, hooked around `EngineGame::run_frame` and the pad-less pre-tick. `slp_compare.py A B` compares the
post-frame state per frame and port (action, x, y, facing, percent, shield, stocks), taking the last copy of a
rolled-back frame.

**Results (real network: mm.slippi.gg:43113, direct, our two accounts; game traffic on the LAN addresses).**

| run | decider (engine) vs Dolphin | engine lag / loss | frames | compared live / offline | desyncs | rollbacks engine | Dolphin .slp rollbacks (rolled frames, depth) | slpcheck Dolphin .slp | slpcheck engine .slp | .slp vs .slp |
|---|---|---|---|---|---|---|---|---|---|---|
| hd1 | engine acct1 vs Dolphin acct2 (Kirby vs Fox) | 0 / 0 | 2489 | 2353 / 2353 | 0 | 19 (depth 7) | - (replay not written: run stopped early) | - | match, 4982 fighter-frames | - |
| hd2 | engine acct2 vs Dolphin acct1 | 0 / 0 | 2417 | 2197 / 2197 | 0 | 1 | 7 (13, 4) | match, 4832 | match, 4838 | 4832 equal |
| hdL | engine acct1 vs Dolphin acct2 | 40 ms / 5 % | 2490 | 2178 / 2272 | 0 | 191 (depth 7) | 45 (91, 4) | match, 4990 | match, 4984 | 4984 equal |

**Clean-ups.**
- `dolphin_drive.py css_goto`: after the stick is centred, the hand has to settle (inside 0.6 for 8 still frames)
  before A is pressed, otherwise it is steered again. Near the target the stick is held just past the dead zone.
  Every phase 7 pick landed (Kirby x3, Peach).
- **Peach mirror on Battlefield** (mock, loopback, 30 ms / 5 %; Dolphin Peach vs engine Peach shade 1, stage 0x1F):
  2689 frames, 2140 compared live, 2245/2245 offline, 0 desyncs, 224 rollbacks.

**Real NAT test.** The test is written but was not run. `tools/rust-port/slippi/REAL_NAT_TEST.md` covers machine A
(Windows) and machine B (a Linux laptop on a phone hotspot). `real_nat_side.sh` runs one side and, with `--compare`,
checks the two CSVs against each other. The doc lists the likely Linux build trouble spots. A Windows-side
`cargo check --target x86_64-unknown-linux-gnu` stopped at the C build scripts. One real gap was fixed:
`melee_engine::cow` had no non-Windows build.

**Not done.** No stranger (unranked) check.

## 16. Linux: the local-test Slippi Dolphin, the engine as decider, time sync A/B (2026-11-22, port/deciderharness)

**The build.** `tools/rust-port/slippi/build_dolphin_linux.sh DIR` builds dolphin-emu-nogui of project-slippi/dolphin
mainline `0653df8` on Linux (Arch, GCC 16, CMake 4, rustc 1.99.1 for the pinned 1.88.0): the same patches as the
Windows netplay link (`slippi-dolphin-nhv1.patch`, `dolphin_enet_mm.patch`), plus `dolphin_localtest_rust.patch`
(the Rust half of `SLIPPI_LOCAL_TEST`, which the Windows-only `localtest-cpp.patch` held: every web request refused
in the resolver before DNS, no browser pages, matchmaking only to `ws://127.0.0.1:`) and `dolphin_linux_fixups.patch`
(a missing include MSVC's precompiled header hid). Needed: `USE_SYSTEM_LIBS=OFF` (Arch's fmt 12 breaks 0653df8),
`CMAKE_C_FLAGS=-std=gnu17` (GCC 16's C23 default breaks the bundled libiconv 1.14), `ENABLE_SDL=OFF`,
`CMAKE_POLICY_VERSION_MINIMUM=3.5`. No root needed. About an hour on 8 jobs.

**The harness.** `run_np_dolphin.sh` starts one Dolphin with a fresh throwaway user dir per run (test profile
`TSTA#001`, fake uid, no play key; `latestVersion` must parse as semver, an empty one throws in
`prepareOnlineStatus`), Null video, no audio, emulation speed 1.0, spectator/jukebox/replays off, pads from the FIFO
`<userdir>/Pipes/slippibot1` (`dolphin_drive.py --pipe`). `np_dolphin_pair_linux.sh OUT --decider engine|dolphin
--stage S --lag-ms N` runs mock + Dolphin + driver + `melee play --slippi-mm` inside `unshare -rn` (a private network
namespace with only loopback: nothing can reach a Slippi service whatever the config). The mock makes the first
queued ticket the host, so `--decider engine` queues the engine first and Dolphin joins second as the guest (port 2).
`--engine-out-ms` / `--dolphin-out-ms` add one-way delay to one direction only (`mock_mm.py --extra-ab-ms/--extra-ba-ms`);
`--engine-hitch-ms` SIGSTOPs the engine for a while. `np_dolphin_matrix_linux.sh OUT N FRAMES` runs the matrix.

**Guest pairing.** Dolphin as the guest paired, connected and played every time here (all engine-decider runs below,
plus one with the engine left 45 s alone in the queue). The mock's answer to the second joiner (`isHost:false`, its own
entry `port 2`, the decider at the relay) is what Slippi's `handleMatchmaking` expects; the Dolphin side keeps one link
(the engine's CONNECT reached it after it had bound its netplay port). The desktop's disconnect was not reproduced.

**Found on the way.** `UdpTransport::stamp_arrivals`' reader thread ended on EINTR: on Linux a socket read with
SO_RCVTIMEO fails with EINTR after a stop signal and SIGCONT, and the link then went deaf (still sending, never
receiving, until the peer timed it out). Interrupted is now a timeout like WouldBlock.

**Results** (engine vs Slippi Dolphin, loopback, mock relay lag one way in both directions, 3600-frame target; a
match can end early on stocks; compared = Dolphin's TX checksums vs the engine's finalized ones, every common frame):

| decider | stage | lag ms | extra | time sync | matches | frames compared | desyncs (soft/hard) | engine holds per match | engine catch-up frames | engine rollbacks per match |
|---|---|---|---|---|---|---|---|---|---|---|
| engine | Dream Land (0x1C) | 30 | - | old (default) | 3 | 9461 | 0/0 | 8, 10, 10 | 0 | 10, 19, 17 |
| engine | Dream Land | 30 | - | new | 3 | 9648 | 0/0 | 0, 0, 0 | 0 | 151, 191, 201 |
| engine | Dream Land | 60 | - | old | 3 | 9939 | 0/0 | 4, 4, 5 | 0 | 308, 453, 457 |
| engine | Dream Land | 60 | - | new | 3 | 8229 | 0/0 | 3, 1, 2 | 4 (one match) | 251, 335, 455 |
| engine | Final Destination (0x20) | 30 | - | old | 3 | 10340 | 0/0 | 8, 10, 9 | 0 | 11, 12, 12 |
| engine | Final Destination | 30 | - | new | 3 | 10781 | 0/0 | 0, 0, 0 | 0 | 238, 198, 175 |
| engine | Final Destination | 60 | - | old | 3 | 8625 | 0/0 | 5, 6, 5 | 0 | 232, 363, 457 |
| engine | Final Destination | 60 | - | new | 3 | 10734 | 0/0 | 1, 0, 0 | 0 | 413, 456, 460 |
| Dolphin (control) | Dream Land | 30 | - | old | 1 | 3566 | 0/0 | 9 | 0 | 18 |
| Dolphin (control) | Dream Land | 30 | - | new | 1 | 3681 | 0/0 | 0 | 0 | 202 |
| engine | Dream Land | 30 | engine->Dolphin +80 | old / new | 3 / 3 | 9940 / 9979 | 0/0 | 4 / 0 | 0 | ~300-460 |
| engine | Dream Land | 30 | Dolphin->engine +80 | old / new | 3 / 3 | 9779 / 9443 | 0/0 | 3-4 / 0 | 0 | ~300-460 |
| engine | Dream Land | 30 | engine stopped 200 ms at ~1 s | old / new | 3 / 3 | 9137 / 9638 | 0/0 | 8-10 / 0 | 0 | 8-18 / 121-197 |
| engine | Dream Land | 30 | engine stopped 300 ms before its start | old / new | 3 / 3 | 8938 / 9059 | 0/0 | 3 / 0-1 | 0 | 10-13 / 79-167 |

No desync in 50 matches with either rule. The new rule's difference here is that it does not hold after the opening
(frame 120), so the engine stays a little ahead and rolls back 10-20x more at 30 ms; its catch-up path (run extra
frames when behind, first possible on online frame 125) ran in one match only (4 frames, 60 ms, no desync): the engine
measured itself ahead of Dolphin in every other match, whichever direction carried the extra delay and with the engine
frozen. So these loopback runs neither confirm nor rule out time sync as the cause of the live desync (Dream Land,
online frame 125, the remote checksum constant): frame 125 is exactly the first frame the new rule can run an extra
frame on, which a loopback peer on the same clock rarely makes it do. A test that needs the engine to start behind a
faster-loading Dolphin, or a forced catch-up, is what would decide it.

## 17. The forced catch-up vs Slippi Dolphin (2026-11-23, port/catchupfix)

**Correction to section 16.** The new rule's first possible extra frame is online frame 150, not 125:
`advance_frames` returns 0 on a check frame up to the opening (120), so the first catch-up starts at the check on 150
(150, 155, 160). Frame 125 is where a hold from the last opening check (120, up to 5 frames under either rule) ends.

**The hook.** `MELEE_TIMESYNC_FORCE_CATCHUP=N[:K]` (new rule only, read once per match in `TimeSync::new`, a field of
the match's `TimeSync`, no static): on every Nth offset check after the opening a catch-up of K frames (default 3,
`MAX_ADVANCE`) starts whatever the offset. `np_dolphin_matrix_linux.sh` `RULES=force` (`FORCE=N`).

**Results.** Engine vs Slippi Dolphin (loopback mock, `MELEE_TIMESYNC_NEW=1`, 2400-frame target, 3 matches per row;
"inputs" = `slp_inputs_diff.py` on Dolphin's .slp vs the engine's `play --slp`: per frame and port the buttons, sticks,
triggers and pre-frame seed from frame -39, plus the post-frame state):

| decider | stage | lag ms | catch-up | frames compared | desyncs | extra frames run (per match) | rollbacks | inputs / state |
|---|---|---|---|---|---|---|---|---|
| engine | Dream Land | 30 | forced (N=1) | 6716 | 0 | 217, 239, 239 | ~290 | equal |
| engine | Dream Land | 30 | natural | 7460 | 0 | 0, 0, 0 | ~108 | equal (one seed on the file's last frame) |
| engine | Dream Land | 60 | forced | 6882 | 0 | 239 x3 | ~325 | equal |
| engine | Dream Land | 60 | natural | 7409 | 0 | 0 | ~335 | equal |
| engine | Pokemon Stadium (3) | 30 | forced | 6911 | 0 | 239 x3 | ~308 | equal |
| engine | Pokemon Stadium | 30 | natural | 6640 | 0 | 0, 2, 2 | ~95 | equal |
| engine | Pokemon Stadium | 60 | forced | 5900 | 0 | 226, 232, 231 | ~311 | equal |
| engine | Pokemon Stadium | 60 | natural | 7208 | 0 | 0, 2, 7 | ~343 | equal |
| Dolphin (engine guest) | Dream Land | 30 | forced | 6190 | 0 | 200, 221, 168 | ~259 | equal |
| Dolphin | Dream Land | 30 | natural | 6102 | 0 | 3, 0, 3 | ~147 | equal |
| Dolphin | Dream Land | 60 | forced | 6061 | 0 | 219, 236, 204 | ~305 | equal |
| Dolphin | Dream Land | 60 | natural | 6199 | 0 | 8, 8, 9 | ~322 | equal |
| Dolphin | Pokemon Stadium | 30 | forced | 6918 | 0 | 229, 227, 239 | ~303 | equal |
| Dolphin | Pokemon Stadium | 30 | natural | 6641 | 0 | 3, 0, 0 | ~95 | equal |
| Dolphin | Pokemon Stadium | 60 | forced | 6162 | 0 | 232, 227, 222 | ~317 | equal |
| Dolphin | Pokemon Stadium | 60 | natural | 6441 | 0 | 9, 8, 0 | ~330 | equal |

The catch-up path ran about 5,300 extra frames in 24 forced matches without a desync, decider or guest, on both stages:
it is not what desynced live, and needs no fix. Slippi Dolphin mainline catches up itself ("Advancing on frame 30
due to time sync" in its log), so both sides running extra frames is normal play. The engine's .slp numbers frames
one lower than Dolphin's and records its own stick a frame or two early before frame -39 (the game state is unaffected);
`slp_inputs_diff.py` finds the shift itself and counts those apart.

## 18. Desync capture (2026-11-27, port/desynccap)

Every Soft or Hard checksum mismatch seen in an online match is written to disk so it can be replayed offline. Code:
`rust/melee/src/desynccap.rs` (capture), `desync_replay.rs` (`melee desync-replay`), hooks in `slippi_np.rs`
(`slippi-netplay`), `play_net.rs` (the rendered `play --slippi-mm` / online select loop), `netmatch.rs`
(`EngineGame`), `slp_export.rs` (`snapshot_bytes`, `arm_for_desync`).

**What is written.** `<root>/<timestamp>/` with `report.json`, `desync.slp` and `bits.txt`. Root: `MELEE_DESYNC_DIR`, else
`$YOUGAME_DATA/desyncs`, else `./desyncs`. The first Soft or Hard of a game gets a directory, and the first Hard gets
another one when the first was a Soft (so at most two per game and side). No player names, connect codes or account
data: only character ids, colours, team, stage, rng_offset and the like.

- `report.json`: `kind` (soft / hard), `tag` (decider / guest), `online_frame` (the compare's frame number),
  `slippi_frame` (= online frame - 125, the .slp's frame number of the same state), `session_online_frame_at_detection`,
  `local_checksum`, `remote_checksum`, `build_id`, `slippi_app_version` (what the matchmaking join sent; null on a direct
  link), `match` (stage, players with character / colour / team, rng_offset, alt_stage_mode, decider, local port, rules,
  the scene string), `codes` (the gameplay codes word, LGL limit, codeset), `rollback` (session stats at detection),
  `state_dump` (`melee_fighter::slippi_online::state_dump()` at detection; the session is a few frames ahead of the
  compared frame), `slp`, `slp_last_frame_at_detection`, `slp_through`.
- `desync.slp`: the engine's own recording of the match (the live exporter's confirmed frames, -123 on). A Hard's file runs
  through the last confirmed frame at detection. A Soft's is written at detection as well and rewritten at the game's end
  with every confirmed frame (`slp_through: game_end`; a peer that left early leaves the tail unconfirmed, so the file ends
  there). It plays with `melee slpcheck --slp` like any engine .slp.
- `bits.txt`: the seed-bit bytes (`<tick> <b0> <b1> <b2> <b3>` hex, tick 0 = Slippi frame -123) of the frames where a port
  rolled a random special (Peach, Luigi, G&W): the .slp has no such byte and the replay needs it to draw the same seeds.
  Empty when nobody rolled.

**Cost.** The match is recorded in memory by the live exporter (`slp_export::arm_for_desync`, on for every online match; no
.slp unless a desync happens, `MELEE_DESYNC_CAPTURE=0` turns it off). At detection (once or twice per game) the .slp is
encoded in memory and a worker thread writes the files; the game thread does no file I/O. The state lives on the system
heap behind a pointer made once, because a rollback load rewinds statics and the arena. Loopback, 1700-frame matches, 30 ms
lag each way, two peers: tick work (poll + sim + send) mean 1.87 / 1.98 ms with capture, 2.27 / 2.14 ms with it off (machine
noise is larger than the difference); max tick after the first 200 ticks 12.3 / 15.9 ms with capture, 0 ticks over 16 ms.
The capture-off run had 7 ticks over 16 ms (up to 104 ms) on the same busy machine, so the loopback numbers say "no
measurable cost", not "faster". The detection ticks themselves (online frames 416 and 419) were not slow.

**A Hard now ends the game, as Slippi's does** (section 18.1). Capture is unchanged: the report is written at detection,
before the game ends.

**`melee desync-replay DIR [--iso DISC]`.** Boots the match as the online match it was (online mode on with the report's
rng_offset, the online codeset, the scene string, `bits.txt`), feeds it the Pre-Frame pads of `desync.slp` (the pads the
simulation saw, raw sticks and buttons) straight with no rollback, and prints the engine's checksum at the detection frame
beside the two recorded ones. Reproducing the local one means this side's state follows from the recorded inputs; reproducing
the remote one means this side differs from a clean re-run. Neither means the .slp inputs do not account for the state.
Run it without `MELEE_NP_TEST_DESYNC` set. A field desync that is a simulation difference shows as one side matching and
the other not; a case where the two sides saw different inputs shows as each side matching itself.

**Test hook.** `MELEE_NP_TEST_DESYNC=<game frame>[:<count>]` (headless `slippi-netplay`, `EngineGame`): from that tick, for
`count` ticks (30), this side's simulation sees both ports' sticks reversed and Y pressed while the pads it sent are
unchanged. The recorded .slp holds the pads the sim saw, so a replay reproduces this side's own checksums. (The
`play_net` hook of the same name perturbs the logic RNG and has no effect under Slippi's SyncRNG.)

**Results** (loopback, host with `MELEE_NP_TEST_DESYNC=400`, 900 frames, `--lag-ms 30`, Falcon vs Fox on Final Destination):
both sides logged a Soft at online frame 416 and a Hard at 419; four directories, `desync-replay` on each:

| side | kind | online frame | local | remote | replay |
|---|---|---|---|---|---|
| decider (perturbed) | soft | 416 | 917d0010 | ffcd0010 | 917d0010 |
| decider | hard | 419 | 9183000a | ab690008 | 9183000a |
| guest | soft | 416 | ffcd0010 | 917d0010 | ffcd0010 |
| guest | hard | 419 | ab690008 | 9183000a | ab690008 |

Each side's replay reproduces its own checksum. A clean 900-frame and a 1700-frame match (both lag 30 ms, with rollbacks)
wrote no directory. Not covered: the `play` loops were only compiled (the capture calls are the same), a Slippi Dolphin peer
(its checksum is recorded as received; the replay then says whether our side or its side differs), and the Direct set's
second game (the state resets per game in `set_match`).

### 18.1 A Hard desync ends the game (2026-11-27, owner: "It should follow slippi")

**Slippi's behaviour, as read.** Sources: `slippi-ssbm-asm` d0ce23a (`_build/slippi-src/ssbm-asm`: `Online/Core/StartEngineLoop.asm`,
`InitOnlinePlay.asm`, `Online.s`) and Slippi Dolphin `project-slippi/dolphin` branch `slippi` (`EXI_DeviceSlippi.cpp`,
`SlippiNetplay.cpp`). The workspace has only the ASM tree, so the two Dolphin files were read from GitHub's raw files.

- *What is compared.* Every engine frame the game writes its own checksum for the frame and a recovery record (timer
  `0x8046B6C8`, stocks and percent of 4 slots; StartEngineLoop.asm 446-495). Then, for each remote entry: skipped while the entry's
  frame is greater than `ODB_STABLE_FINALIZED_FRAME` (line 529) or `<= UNFREEZE_INPUTS_FRAME` = 84 (line 530). The peer reports one
  (frame, checksum) per pad packet, so the compared frame is the newest one the peer sent that is finalized here. Once a Hard is
  displayed (`ODB_IS_DESYNC_STATE_DISPLAYED`, checked at 516, set at 583) nothing is compared again.
- *Hard.* The low 16 bits (the truncated float sum of position x/y and percent) of the two checksums, as s16, differ by more than 1
  (lines 559-568). Soft: only the high 16 bits (XOR of action state, position and percent bits, spawn count, stocks) differ: the
  "Desync Risk" text once (after 592), the game goes on.
- *What a Hard does* (lines 569-587): error sound (`SFX_Menu_CommonSound` 3), HUD text "DESYNC DETECTED", then `FN_END_GAME`
  (lines 224-246): `match[+1]` = the local player's index ("pauser"), `match[+8]` = 7 (exit type LRAS: the game's own end-of-match
  check takes over, `OUTCOME_NO_CONTEST`), `match[+0x24D5]` = 90 (the GAME! screen's maximum length; 110 in vanilla, 55 in 9817968,
  90 now). The game then runs its normal LRAS ending: GAME!, then the scene exits to the online menu flow; no result screen of its
  own. Each side detects on its own: both see the same two checksums, so both end (a few frames apart), each with itself as
  the quitter.
- *All modes.* ssbm-asm 9817968 ("always end game on disconnect or desync, previously would only happen in ranked") removed the
  ranked-only guard: Unranked, Ranked, Direct and Teams all end the game. The disconnect path (line 325) does the same.
- *Reporting.* `InitOnlinePlay.asm` 397-455 hands Dolphin the report: winner index -2 (desync; -3 disconnect), game end method
  (`match[+8]`, 7), the LRAS initiator, the synced timer, and per slot the synced stocks and damage.
- *Recovery state kept.* Every compare pass that is not a Hard copies the compared frame's timer, stocks and percent into
  `ODB_DESYNC_RECOVERY_*` (lines 626-640); the Hard pass jumps over the copy (line 587 to the exit); frame 0's own state seeds
  it (line 505ff.). So the kept state is that of the last frame compared before the Hard.
- *How it is applied: ranked only.* `CEXISlippi::handleReportGame` (EXI_DeviceSlippi.cpp:3149): only when `online_mode == Ranked
  && game_end_method == 7` does Dolphin send `SLIPPI_SYNCED_STATE` (0x86) to the peer and set `is_desync_recovery`. For the next
  game `GetDesyncRecoveryState` (SlippiNetplay.cpp:2127-2203) waits until the peer's state for the same game and tiebreak index
  arrived, then reconciles: timers within 1 s (the larger is used), stocks of every slot equal, percent within 25 (the lower);
  any failure drops the connection. `prepareOnlineMatchState` (EXI_DeviceSlippi.cpp:2560-2590) writes the merged stocks and
  percent into the match block and the merged seconds into its timer word; without a recovery it writes the defaults (4 stocks,
  0 %, 8:00). **Unranked, Direct and Teams send nothing: the next game of a Direct set starts at 4 stocks, 0 %, 8:00 as always.**
  (`is_desync_recovery` is cleared when the next selections go out, SlippiNetplay.cpp:1433.)

**Implemented** (`melee_core::slippi_online`: `DesyncWatch`, `merge_recovery`, `SyncedState`; `melee_fighter::slippi_online`:
`synced_state`, `end_game_now`, `set_start_percent`, `set_start_seconds`; `melee_net::slippi`: `SyncedMsg`,
`Client::send_synced_state`, `recovery_state`; both loops):

- Frames <= 84 are not compared. A Hard sets `hard_end` on the game (`EngineGame` for `slippi-netplay`, `SlippiSim` for the
  rendered `play` loop) and stops the compare. The game then writes `FN_END_GAME`'s three fields (`state.pauser` = local port,
  `state.match_result` = 7, `start.xD` = 90) before every frame it runs, resimulated frames included (a rollback load rewinds those
  statics), and the game's own end-of-match code does the rest. The rendered loop shows the existing "DESYNC DETECTED" notice
  (retail_flow, from `slippi_desyncs`); the capture report is written first; the link closes, or goes on to the next game of a
  set, as for any ended game. Soft is a warning only, as before. The winner reported to the select is -1 for a Hard (already so).
- Not done: the error sound (3).
- Recovery (ranked only; never reachable in our client, `--mm-mode ranked` is refused): `DesyncWatch` keeps the last non-Hard
  compared frame's record; a game that ended on a Hard in ranked sends `SYNCED_STATE`, waits for the peer's, merges as Slippi does
  and writes the result (`--recovery-out FILE`); `--recovery-in "secs;stocks,pct;x4"` starts a game from it (scene tokens
  `/stocksN` and `seconds=`; the percent and the exact timer are written before frame 1). Only `slippi-netplay` has this. The
  rendered loop has no ranked mode, so a Direct set's next game there starts from the defaults, which is Slippi's behaviour.
  Test hook for the exchange: `MELEE_NP_TEST_RANKED_RECOVERY=1`.
- New test hooks: `MELEE_NP_TEST_SOFT=<online frame>[:count]` (flips the high half of this side's checksums: a pure Soft);
  `MELEE_NP_TEST_DESYNC` now also perturbs the rendered loop (the same stick reversal as in `slippi-netplay`).

**Tests run** (loopback, hidden windows, `MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3`, fast profile; Falcon vs Fox on
Final Destination, `--frames 900 --lag-ms 30`):

| run | result |
|---|---|
| `slippi-netplay`, host `MELEE_NP_TEST_DESYNC=400` | Soft at online frame 416, Hard at 419 on both sides; each side ends the game (detected at session frame 423 / 424, game over at 517 / 518: 94 frames later); the reports were written first |
| `slippi-netplay`, host `MELEE_NP_TEST_SOFT=300:3` | 3 Softs on both sides, no Hard; the game ran on to frame 990 |
| `slippi-netplay`, clean | 0 soft, 0 hard, 990 frames, rc 0, no capture directory |
| ranked exchange (`MELEE_NP_TEST_RANKED_RECOVERY=1`, `--recovery-out`) | both sides sent `475;3,0;4,0;4,0;4,0` (the state of the last frame before the Hard) and merged the same |
| recovery game (`--recovery-in` that state, 900 frames) | the game read back the same state at frame 0 and 474 s at frame 150; 0 soft, 0 hard, both rc 0. With `475;3,45;4,20;...` the checksum trace showed 45 and 20 percent from frame 1 |
| rendered `play --slippi-mm` (loopback mock, Direct, hidden windows), host `MELEE_NP_TEST_DESYNC=300`, 1200 frames | Soft 324, Hard 325 on both; both log DESYNC DETECTED and end after about 95 frames (420 / 421 final checksums), exit 0. A clean 600-frame run: 0 soft, 0 hard |
| 16 ms rule | `slippi-netplay` after the first 200 ticks: max 2.3 - 4.3 ms, 0 ticks over 16 ms (hard, recovery and clean runs); rendered: the only tick over 16 ms is the intro's tick 1 (clean runs have it too) |

Unit tests: `DesyncWatch` (the 84 skip, Soft goes on, Hard stops and keeps the previous state), `merge_recovery` (the 1 s / 25 % /
equal-stocks rules), the 0x86 codec, the recovery argument codec. Not covered: a Slippi Dolphin peer; a two-game Direct set through
the rendered select (the next game needs nothing: it starts from the defaults); the error sound.

## Peer build message (Hero's 0xE0), receive only

Melee Unlocked clients send `NP_MSG_MU_BUILD` (0xE0) to their peer (his `slippi_net.cpp` `SendBuild`): `'M' 'U' 'B'`, version 1, player
port, mod_view, fingerprint len + bytes, name len + bytes, then optional capabilities (bit 0 early inputs, 1 answers pre-match pings,
2 Static engine, 3 ranked rules digest follows), optional shield colour (1 + RGB, or 0), optional 64-byte rules digest.
`melee_net::slippi::PeerBuild::parse` reads it tolerantly (any field after the tag may be missing; a bad tag, an unknown version or a
truncated message never fails anything; control bytes in strings become `?`). We send nothing and answer nothing (no reply to his
pre-match pings), and no game behaviour changes. Stock Slippi Dolphin and other clients send no 0xE0.

- Log line when it arrives: `slippi: opponent (port N): Melee Unlocked build: Source Port | Static Recomp; retail | mod <name> <fingerprint 16>; capabilities ...`.
- `slippi-netplay` summary: `melee: <tag>: peer: <same text>`, or `no Melee Unlocked build message (stock Slippi Dolphin or another client)`.
- `report.json` (desynccap): `peer_build`, null when no 0xE0 came, else `{engine, message_version, mod_view, mod, fingerprint_prefix, capabilities[], truncated, text}`.
- A link handed over from the select to the match process carries the message with it (`Client::export` trailer).
- Test hook, off by default: `MELEE_NP_TEST_SEND_BUILD=1|mod|static|<hex of a whole packet>` makes that side send one synthetic 0xE0 when
  it sends its selections.

Tests: unit test `slippi::tests::peer_build_parse` (full, older build, truncated, unknown version, garbage). Loopback (`np_pair`-style, 900
frames, `MELEE_NP_TEST_DESYNC=400` on the host): a normal pair logs "no Melee Unlocked build message" and every report has
`"peer_build": null`; with `MELEE_NP_TEST_SEND_BUILD=static` on the host the guest logs the build line and its reports carry the
object, the host's stay null.

### 18.2 A stale remote checksum (live, 2026-11-28, port/desync-fd)

A real Slippi Dolphin peer (unranked, FoD, Falco vs Marth, build 8e974c2fd) ended our game with a Hard at online frame 5782:
local `d11a004a`, remote `4079004e`. `melee desync-replay DIR --ck-out ck.txt` reproduces the local value (the capture is
self-consistent), and the remote value is, all 32 bits, our own checksum of online frame **5781**. So both halves of the peer's report
match our previous frame: Falco (DeadDown, last stock) and Marth were the same on both sides. The low halves differ by 4 only because
Marth moved (-1.08, -2.5) during 5782. The 5643 compares before it had 0 Softs, so the two simulations agreed throughout. The peer
put an older frame's value under a newer frame. A likely path, read from the ASM and not proven:
`TriggerSendInput.asm` FIND_CHECKSUM_LOOP writes `TXB_FINALIZED_FRAME_CHECKSUM` only when the desync ring has an entry for
`ODB_STABLE_FINALIZED_FRAME`. Otherwise it leaves the previous packet's value in the buffer, while `TXB_FINALIZED_FRAME` always
carries the new frame.

Handling: `DesyncWatch::check_with_earlier` passes over a mismatching remote value that equals one of our own checksums for the 3
frames before. It logs "stale remote checksum", counts it in `stale`, and the frame is not compared. A real one-frame lag on the peer
still ends the game on the next compare, because every later frame differs too. This is a deviation from Slippi, whose compare has no
such check. At a Hard, the log now also lists the peer's last 6 reports next to our values.

## Sets against Slippi Dolphin: the next lock-in before our match is over (Nov 27)

Owner bug: three Unranked games against real Slippi Dolphin peers, and each set ended after game 1 ("the set's link taken
over (Disconnected)" or "the opponent's link is down" on the rematch select). The cause was our client, not Dolphin:

- Slippi Dolphin keeps the link through an Unranked or Direct set. ssbm-asm `Online/Slippi Online Scene/main.asm`
  `VSSceneDecide` sends `CleanupConnections` only for Ranked set ends, Party, and disconnects. Unranked and Direct go
  straight to `VSSceneDecide_GoToNextScene` (the CSS) the frame the VS scene ends.
- On that CSS, Start locks in at once (`Online/Menus/CSS/HandleInputsOnCSS.asm` `HANDLE_CONNECTED_UNRANKED` ->
  `FN_TX_LOCK_IN` -> EXI `setMatchSelections` -> `SlippiNetplayClient::SetMatchSelections`). Only the CSS's first frame
  is skipped, so a player who mashes Start through GAME! sends `SLIPPI_MATCH_SELECTIONS` almost as soon as Dolphin's VS
  scene ends. Dolphin merges selections whenever they come (`SlippiNetplay.cpp` `OnData`,
  `case SLIPPI_MATCH_SELECTIONS`) and never drops a link over them.
- Our match lingers after its end for the last confirmed frames (`play_net.rs`, 1.5 s after `done_at`), and only then
  calls `Client::end_game`. Meanwhile `melee_net::slippi::Client::on_data` took any `MSG_MATCH_SELECTIONS` that came
  while `in_game` for a restarted peer process. It sent DISCONNECT on every link and set `Disconnected` without a log
  line, so the next select took a dead link and the Dolphin side saw its opponent leave.

The fix: selections on the match's own (active) link while `in_game` are the opponent's next lock-in. They are held
(`next_selections`, logged once: "the opponent locked in for the next game before this match was over") and delivered by
`end_game` like any lock-in on the select. Selections on another link in a match (a new connection from a restarted
process) still end the match as before.

Tests: `melee_net/tests/slippi.rs` `three_games_when_the_dolphin_peer_locks_in_before_our_match_is_over` (the peer
locks in while ours lingers 1.5 s, three games on one link; it fails without the fix, at the first early lock-in).
`tools/rust-port/slippi/menu_rematch_three.sh OUT` runs two of our builds through mock_mm, one window each. A lingers
5 s (`MELEE_SLIPPI_TEST_LINGER_MS`), so B is back on its select and locks in first, as Dolphin does. It plays three
games, through GAME! and the select each time, on one link, and is back on the select for game 4 still linked. Nov 27:
PASS with the fix (one mock join per side, the early lock-in held twice, 0 desyncs). The exe without the fix failed it
with the owner's log lines exactly: A "taken over (Disconnected)", B "taken over (Connected)", then "link is down
(reason 0)", and both on the idle select after game 1.

Not proven against real Slippi Dolphin: no Dolphin peer was run (only the Rust pair and a Dolphin-like protocol test).
This covers the silent `(Disconnected)` case. A peer DISCONNECT while our match still lingers ("link is down (reason 0,
in a match)") can also be a Dolphin player backing out. That sends `CleanupConnections` too, and nothing in our log
tells the two apart.

## 19. Real Slippi Dolphin peer on Windows: sets through the CSS (2026-11-27, port/dolphinpeer)

Our build (`play --slippi-menus on`, the owner's path: online select, in-process match, the set's link kept between
games) against a local Slippi Dolphin on the desktop, Unranked, several games per set, paired by `mock_mm.py`, with lag
and jitter from the mock's relay. Nothing reached a Slippi service or any address off the machine (below).

**The Dolphin.** The mainline-based Slippi Dolphin of sections 9-16 (`_build/dolphin-oracle/src`, project-slippi/dolphin
`0653df8`), built a second time in its own tree with `-DGD_DOLPHINPEER=ON` (`_build/dolphin-oracle/build-peer.bat`,
under the build lock, 8 jobs). The patch is local only (Dolphin is GPL): branch `gd/dolphinpeer` in that clone (on top of
`gd/oracle-localtest-base`, the oracle + `SLIPPI_LOCAL_TEST` working tree as it was found, recorded without touching the
working tree), and `_build/dolphin-oracle/dolphinpeer.patch` (this build's change only) and `dolphinpeer-full.patch`
(against upstream, Rust extensions included). With the define on and the oracle build unchanged (the define is off there):
- `MainNoGUI` forces `SLIPPI_LOCAL_TEST=1` whatever the caller's environment (the Rust web API, rank fetch and game
  reports are refused before DNS, no STUN, no browser pages, every netplay socket bound to 127.0.0.1), and any
  `SLIPPI_MM_URL` that is not `ws://127.0.0.1:` becomes `ws://127.0.0.1:9/v1/queue`; `SLIPPI_MM_ENET`/`SLIPPI_REAL_NET` are
  cleared.
- `Common::HttpRequest` refuses every request (analytics, updates, achievements, code downloads).
- `SLIPPI_PEER_LOG=<file>`: `GAME_START`, one `LOAD frame F` per savestate load (a rollback), and the game report
  (`REPORT ... end_method E winner W`; Slippi's Hard-desync end is end method 7 with winner -2).
- the binaries go to `build-peer/Binaries`, so the oracle's `src/Binary/x64` is untouched.
The run copy is `_build/dolphin-peer/` (exe, Rust DLL, the online `Sys`; 27 MB): the only Dolphin path the harness runs.
Input is the existing per-instance named pipe (`SLIPPI_PIPE_PREFIX`, `\.\pipe\<tag>slippibot1`: the Windows pipe
backend already existed in this Dolphin, so no new controller backend was needed), driven closed-loop on the state log
by `dolphin_drive.py`. Each run makes a fresh user dir: only the pipe as a device (`GCPad1`, pads 2-4 empty), SI device
6 (standard controller, not the adapter), no audio, Null video, no window, a fake `user.json` (`DPEER#001`, fake uid,
no play key). No real user.json or Slippi Launcher file was read.

**The harness** (`tools/rust-port/slippi/`):
- `dolphin_peer.sh OUT --ours C --theirs C --stage S --rtt-ms N --jitter-ms N --games N --game-frames N [--end sd|lras]
  [--decider engine|dolphin]`: one set. Our side: `MELEE_PAD_SCRIPT` to the Unranked select and the pick
  (`dolphin_peer_pads.py`: the hand starts at (-31, -21.5) and moves 1.24 a frame at full deflection, measured with
  `MELEE_SLIPPI_CSS_TRACE`), Start only on the set's later selects (the fighter is kept), generated in-game pads
  (`MELEE_SLIPPI_NP_PADGEN`), `--slp`, `MELEE_SLIPPI_NP_OUT`, `MELEE_DESYNC_DIR`, and the agent env (`MELEE_INPUT=n`,
  `MELEE_PAD_IGNORE_ADAPTER=1`, hidden, unfocused, volume 3, a titled window). Dolphin's player plays random pads for
  `--game-frames`, then runs off the stage until it is out of stocks (`--sd`: a normal stock end in seconds), or quits
  with L+R+A+Start (`--lras`). The relay adds half the round trip each way plus 0..jitter ms per datagram (reordering).
- `dolphin_peer_summary.py OUT`: per game our live verdict (soft/hard/stale), Dolphin's verdict recomputed from its
  checksum log by the ASM rule (frames 85.. up to its game's end), the offline compare of every common finalized frame,
  `.slp` against `.slp` (`slp_inputs_diff.py`: pre-frame inputs and post-frame state), rollbacks (ours from the session
  line; Dolphin's from `LOAD` lines and its `.slp`), disconnect lines, and the socket check. One `ROW` per set.
- `dolphin_peer_batch.sh ROOT`: 6 legal stages x RTT 0/60/120 ms (jitter 4/10/20), character pairs from Young Link,
  Falco, Ice Climbers and Fox, deciders alternating, sets of 3, 2 pairs at most, waiting while any
  `melee-slippi*.exe` / `Slippi Dolphin.exe` runs or an owner-play lock is held, stopping under 20 GB free.
- `dolphin_peer_stop.sh DIR` stops one run's processes only.

**Network proof.** `netwatch.txt` (every 2 s, every TCP connection and UDP endpoint owned by the run's Dolphin and game
processes, `Get-NetTCPConnection` / `Get-NetUDPEndpoint`): Dolphin's sockets were all bound to 127.0.0.1 in every run
(its only TCP peer the mock at 127.0.0.1), ours had TCP only to 127.0.0.1, and our log says `STUN not asked`. The batch
also lists the DNS client cache's Slippi / STUN / Google names before and after (`dns_before.txt`, `dns_after.txt`).
Packet capture (pktmon) needs admin, which this session does not have. One gap found: our UDP netplay socket was bound
to 0.0.0.0 / [::] a few seconds after pairing on 127.0.0.1: the in-process match's link takeover (`slippi_link::take`
from `slippi_np` `--link` and from the select's rematch) and the set's hand-back (`slippi_link::hand_back`) re-opened the
socket with bind IP 0. All three now bind at the address the pairing chose (`slippi_np::pair` records it with
`slippi_link::set_bind_ip`: 127.0.0.1 unless `--mm-real` / `--mm-lan`; 0, the old behaviour, when no pairing ran in the
process). It only ever sent to the relay at 127.0.0.1, but a wildcard bind raises the Windows Firewall prompt (for
`_build/dolphinpeer/melee.exe`, the one fixed exe path these runs used) and listens on the LAN. Checked after the fix: a
2-game set had every socket of both processes on 127.0.0.1.

**Batch 1** (`dolphin_peer_batch.sh`, build 976a7b995 + this branch's harness; 3-game Unranked sets; Dolphin's player
plays random pads for 1500 frames, then runs off the stage; ours plays generated pads throughout; "frames equal" = our
finalized checksums equal to Dolphin's own on every common frame up to its game's end; "Dolphin" verdict = our reported
checksums against its own by the ASM rule; `.slp` = `slp_inputs_diff.py` pre-frame inputs and post-frame state, all games;
rollbacks = ours (session) / Dolphin (savestate loads); every row had 0 non-loopback sockets):

| set | ours vs Dolphin | stage | RTT / jitter ms | decider | games | frames equal | ours soft/hard/stale | Dolphin soft/hard | .slp | rollbacks ours / Dolphin | disconnects |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 01 | Young Link vs Falco | FD | 0 / 4 | Dolphin | 3/3 | 21204/21204 | 0/0/0 | 0/0 | equal | 2 / 1463 | 0 |
| 02 | Falco vs Young Link | Battlefield | 0 / 4 | engine | 3/3 (rerun) | 10355/10355 | 0/0/0 | 0/0 | equal | 0 / 0 | 0 |
| 03 | Ice Climbers vs Fox | Yoshi's Story | 0 / 4 | Dolphin | 3/3 | 9909/9909 | 0/0/0 | 0/0 | equal | 6 / 1797 | 0 |
| 04 | Fox vs Ice Climbers | Dream Land | 0 / 4 | engine | 3/3 | 10049/10049 | 0/0/0 | 0/0 | equal | 0 / 1 | 0 |
| 05 | Falco vs Ice Climbers | Fountain | 0 / 4 | Dolphin | 3/3 | 4653/4653 | 0/0/0 | 0/0 | equal | 0 / 0 | 0 |
| 06 | Ice Climbers vs Young Link | Stadium | 0 / 4 | engine | 3/3 | 24908/24908 | 0/0/0 | 0/0 | equal | 0 / 628 | 0 |
| 07 | Falco vs Young Link | FD | 60 / 10 | Dolphin | 3/3 | 10869/10869 | 0/0/0 | 0/0 | equal | 551 / 9512 | 0 |
| 08 | Ice Climbers vs Fox | Battlefield | 60 / 10 | engine | 3/3 | 11062/11062 | 0/0/0 | 0/0 | equal | 572 / 9590 | 0 |
| 09 | Fox vs Ice Climbers | Yoshi's Story | 60 / 10 | Dolphin | 3/3 | 5246/5246 | 0/0/0 | 0/0 | equal | 486 / 4508 | 0 |
| 10 | Falco vs Ice Climbers | Dream Land | 60 / 10 | engine | 3/3, game 3 ended "no pads from the peer" | 2939/2939 | 0/0/0 | 0/0 | equal | 281 / 2524 | 1 (finding 2) |
| 11 | Ice Climbers vs Young Link | Fountain | 60 / 10 | Dolphin | 1/3; rerun 3/3 | 2512/2512; rerun 8980/8980 | 0/0/0 | 0/0 | equal | 133 / 2169; rerun 523 / 7637 | 1 (finding 2); rerun 0 |
| 12 | Young Link vs Falco | Stadium | 60 / 10 | engine | 3/3 | 16472/16472 | 0/0/0 | 0/0 | equal | 526 / 14365 | 0 |
| 13 | Ice Climbers vs Fox | FD | 120 / 20 | Dolphin | 3/3 | 10988/10988 | 0/0/0 | 0/0 | equal | 694 / 10689 | 0 |
| 14 | Fox vs Ice Climbers | Battlefield | 120 / 20 | engine | 3/3 | 4113/4113 | 0/0/0 | 0/0 | equal | 642 / 3860 | 0 |
| 15 | Falco vs Ice Climbers | Yoshi's Story | 120 / 20 | Dolphin | 3/3 | 3409/3409 | 0/0/0 | 0/0 | equal | 636 / 3163 | 0 |
| 16 | Ice Climbers vs Young Link | Dream Land | 120 / 20 | engine | 3/3 | 7762/7762 | 0/0/0 | 0/0 | equal | 673 / 7492 | 0 |
| 17 | Young Link vs Falco | Fountain | 120 / 20 | Dolphin | 3/3 | 16508/16508 | 0/0/0 | 0/0 | equal | 689 / 16163 | 0 |
| 18 | Falco vs Young Link | Stadium | 120 / 20 | engine | 3/3 | 8610/8610 | 0/0/0 | 0/0 | equal | 663 / 8323 | 0 |

55 games, 190,548 frames compared: no desync on either side, and no differing frame offline or in the `.slp` files. Set
02's first attempt never started: the Dolphin driver could not put its hand on Young Link (centre x 27.3; the hand stops at
26). That is fixed in `dolphin_drive.py`. At 60 and 120 ms Dolphin rolls back on most frames and we rarely do. Our
generated pads change every frame (`rb_gen`'s per-frame hash), so its repeat-the-last-pad prediction misses, while its
driver holds a pad for 3-12 frames. Also run: a 2-game set (Young Link vs Falco, Battlefield, 60/10: 8519 frames equal) and
the bind check (Marth vs Fox, Dream Land, 30/6, 2 games: 16526 equal).

**Findings for triage (not fixed here):**
1. *A Slippi Dolphin player's quit-out (client pause, then L+R+A+Start) desyncs us.* Slippi's Unranked enables a client
   pause (`InitPause.asm`; not in ranked), and LRAS from it ends the game on both sides. Dolphin vs Dolphin control
   (scratch `dvd.sh`, the local-only build, FD, Falco vs Fox): both froze at online frame 1508 with the same checksum
   (5462004B), both reported end method 7 with LRAS initiator 0 at 1517 frames, and there was no desync. Ours vs Dolphin
   (smoke run, Fox vs Falco, FD, 0 ms): both sims froze identically through 1815 (15CFFFD9) and Dolphin stayed frozen, reporting
   end method 7 at 1814. Ours started moving again at 1816 (B383FFD9 ...). Then came 3 "stale remote checksum" skips, a Soft
   at 1819, a Hard at 1820, "DESYNC DETECTED", "no pads from the peer" and DISCONNECTED. Offline, 1813/1813 frames were equal
   before the quit. Our match scene runs with `pause=0`, and the client pause / LRAS rule of `InitPause.asm` (`ClientPause`,
   `ODB_IS_LEAVING`) is not in our engine. Capture: `_build/dolphinpeer/runs/smoke1/desyncs/` (two report dirs).
2. *Dolphin's GAME! ended early and our set died.* In 2 of 57 games (set 10 game 3 and set 11 game 1, both at 60 ms), Dolphin
   left the game scene 76 / 28 frames after the last KO. In every other game (55) it sent pads for 105-106 frames after the
   KO, and our match ends on that same frame. In the 2 games, our session waited for pads to finish its own GAME! tail and
   stalled ("no peer input within the rollback window"). After `SILENCE_S` (7 s) it ended the match with "ERROR: no pads from
   the peer", showed DISCONNECTED and went back to an idle select, so the set was over. In set 11 the opponent's next lock-in
   had already arrived and was being held ("the opponent locked in for the next game before this match was over"). The
   checksums agreed to the end (set 11: 2515 compared, 0 differing), and the rerun of set 11 did not repeat it. What shortens
   Dolphin's GAME! is not known (its player was holding the stick to one side, no buttons). Two things to look at: the GAME!
   tail's exit condition in our engine against the console's, and treating a held next lock-in (or the peer's pads ending once
   our confirmed state is past the last KO) as the end of the match instead of an error. Evidence: `batch1/10-*/` and
   `batch1/11-*/` (`ours.log`; `d_checksums.txt`, last TX frame, against the `REPORT` frames in `d_peer.txt`).
3. *Fixed here:* the in-process link takeover and the set's hand-back re-bound on 0.0.0.0 / [::] (see the network proof).

**Not done.** The Ishiiruka-based 3.x Dolphin (what most players run): there is no clone of it here with a nogui or
state-log build, only the installed 3.6.4, which this harness does not drive (no OS input; its pipes have fixed global names,
section 13). This batch did not cover Direct mode, teams, Peach/Luigi/G&W seed bits or items. Packet capture was not
possible (no admin), so the socket watch and the loopback-only binds are the evidence.
