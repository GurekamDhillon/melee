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
