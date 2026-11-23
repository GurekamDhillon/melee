# P2P Direct on the Rust engine: protocol spec and plan

2026-11-22, lane `port/p2pdirect`. Source read: Melee Unlocked `d9448c66` (his main of 2026-11-22), through a
read-only scratch clone. `H:` = a path in his repo. His code is GPL: it was read to learn the wire format and the
behaviour; nothing is copied. The spec below is written in our words from `H:docs/mu-net-protocol.md` (mu_net
version 1, revision 2) and checked against the reference behaviour where the document is silent.

Goal of the lane: his launcher's lobby (P2P Direct) can start **our** engine on both PCs and the two play each other
(Rust vs Rust). A Rust peer against his Source Port is out of scope: the descriptor carries the build id, so the
handshake refuses that pair cleanly (BYE reason 1) instead of desyncing (see "Risks").

## 1. What the launcher passes and expects back

His lobby agrees the whole match over its own encrypted lobby channel; the engine gets it on the command line
(`H:port/app/launcher_lobby_p2p.cpp:p2p_arguments`, `H:port/app/launcher.cpp:2841-2891`):

```
--p2p-port P                    local UDP port (the lobby's own; the lobby closes its socket just before)
--p2p-peer A.B.C.D:PORT         1 to 6 times: candidate endpoints of the other player, in dial order
--p2p-slot 0|1                  our slot
--p2p-chars C/c:C/c             external character id / colour, slot 0 then slot 1
--p2p-stage S                   external stage id (one of the legal six)
--p2p-seed XXXXXXXX             8 hex digits
--p2p-delay 1..15               input delay
--p2p-identity "<file>"         <launcher dir>\lobby-peer-identity.json (JSON, "ed_seed": 64 hex)
--p2p-expect <64 hex>           the other player's Ed25519 public key
--p2p-result "<file>"           <launcher dir>\p2p-results\<id>.json (game n>1: "<stem>.g<n><ext>")
--p2p-names "A:B"               at most 31 bytes each, no ':' '"' '\'
--p2p-connect-seconds 45        appended by launcher.cpp
--p2p-games 1                   Slippi build (one game); the no-Slippi build passes 0 (no limit)
```

Exit codes (`H:port/app/source_p2p.cpp:session_done`, Slippi build): **0** after a game was played, **2** when no
game was played (connect/handshake failed, identity unusable, port busy). Any non-zero exit shows his "P2P Direct
match could not connect" box. The launcher then reads the result file(s) (`H:port/app/launcher_lobby.cpp:p2p_outcome`):
`has_outcome && !desync && !disconnected`, `local_slot`, `winner` (or `players[slot].stocks` when no winner) give
win / loss / incomplete for its match history. No status file other than the result file.

Option parsing rules worth matching (`H:source_p2p.cpp:option`, `check`): `--p2p-port` 0..65535, `--p2p-peer` IPv4
`a.b.c.d:port`, at most 6; chars `id[/colour]` ids 0..255; stage parsed with base 0; seed strtoul base 16 (<= 10
chars); delay 1..15; expect exactly 64 hex; connect-seconds 1..600; games 0..127. Half a request (port without peer
or the reverse) is an error. A port still busy is retried every 200 ms for 2 s.

## 2. Wire protocol (mu_net v1 rev 2)

All integers little-endian. BLAKE2b = 32-byte output. Ed25519 (SHA-512), X25519 (RFC 7748), XChaCha20-Poly1305
(24-byte nonce, 16-byte tag). Domain strings ASCII, no terminator. Frame 1 = first match frame.

### 2.1 Transport (`H:mu_net/peer_connector.cpp`)
- UDP + ENet 1.3 framing, one ENet host, **2 channels**, up to 8 peers (paths), connect data 0.
- Channel 0 reliable+ordered ("control"), channel 1 unsequenced ("input").
- Per connected ENet peer: `enet_peer_timeout(32, 4000, 8000)` and `enet_peer_ping_interval(250)`.
- Packets of size 0, > 1226 (protected) and channel >= 2 are dropped before parsing; inbox cap 512.
- One-byte datagrams (value 0) outside ENet are punch/keepalive bytes and ignored on receipt.

### 2.2 Connecting / NAT (`peer_connector.cpp`, spec sections 3 and 15)
- Dial an ENet connect to every candidate (<= 6); accept any incoming connect (any address).
- While no connection is up: a 1-byte datagram `00` to every candidate every **100 ms**; from **1 s** after start,
  every **300 ms** to <= 8 neighbouring ports of candidates on a **public** address (+1, +2, +3, -1).
- An unanswered ENet connect ends after ~15 s and is redialled while nothing is chosen.
- Limit: 8 s default, `--p2p-connect-seconds` (launcher: 45) overrides; covers connect + handshake + VERIFY.
- No relay, no server. STUN / NAT-PMP / UPnP are **launcher-side** (lobby socket, `nat_discovery.cpp`); the game
  only dials what it is given. We need none of that in the engine.

### 2.3 HELLO (unprotected, channel 0, exactly 160 bytes) (`session_security.cpp:begin`, `accept_hello`)
| off | field | type |
| --- | --- | --- |
| 0 | magic `MUNH` | 4 |
| 4 | version = 1 | u16 |
| 6 | flags = 0 | u16 |
| 8 | identity (Ed25519 pub) | 32 |
| 40 | ephemeral (X25519 pub, per session) | 32 |
| 72 | nonce (random) | 24 |
| 96 | Ed25519 signature over `"MeleeUnlockedNet1 hello"` + bytes 0..95 | 64 |

Sent first on every connection that comes up. Receiver checks in order: >= 6 bytes and magic; version equal (else
"different version", no negotiation); length 160 and flags 0; an already-accepted HELLO: identical bytes ignored,
different refused; ephemeral != own (reflection); identity == expected (`--p2p-expect`); signature; X25519 shared
secret not all zero.

### 2.4 Keys
```
shared     = X25519(own eph secret, peer eph pub)
E1 < E2    (the two ephemeral pubs sorted bytewise; H1, H2 = the HELLOs that carried them)
master     = BLAKE2b("MeleeUnlockedNet1 session key" || shared || E1 || E2)
transcript = BLAKE2b("MeleeUnlockedNet1 transcript" || H1 || H2)
key(E)     = BLAKE2b keyed(master) over ("MeleeUnlockedNet1 direction" || transcript || E)
```
Send key = key(own eph pub), receive key = key(peer eph pub). The holder of E1 is the **decider**.

### 2.5 Protected packet (every message after HELLO, both channels) (`session_security.cpp:seal`, `open`)
| off | field |
| --- | --- |
| 0 | packet version u8 = 1 |
| 1 | channel u8 (== ENet channel) |
| 2 | sequence u64 (per sender, one counter for both channels, from 0) |
| 10 | ciphertext (0..1200) |
| 10+n | tag 16 |

Nonce = seq u64 LE, channel u8, 15 zero bytes. AD = `MUN1` + version byte + channel byte (6 bytes).
Receive checks: 26 <= len <= 1226, version 1, channel byte = arrival channel, handshake done, replay rule, tag. Replay
state updated only after the tag verified. Replay: channel 0 strictly increasing; channel 1 a 64-wide sliding window.
First byte on channel 0: `M` = HELLO, 1 = protected, else drop.

### 2.6 Session descriptor `MUSD` (`session_types.cpp:serialize`, `digest`, `next_descriptor`)
Canonical bytes: `MUSD`, format u16=1, protocol u16=1, route u8 (1 direct), build id str8(<=64), content hash [32],
rules profile u32, match id [16], 4 x slot {present u8, character u8, colour u8, name str8(<=31), identity [32],
session key [32]}, stage u16, alt stage u8, seed u32, delay u8, rollback horizon u8 = 7, stocks u8 (4), timer u32
(480), features u32 (bit0 must be 0, bit1 pause allowed; ours 0).
`digest = BLAKE2b("MeleeUnlockedNet1 descriptor" || canonical)`, computed after each side writes both slots'
identity and ephemeral keys from the handshake (a non-zero preset key that differs fails the session).
His launcher-built descriptor (`source_p2p.cpp:build_descriptor`): slots 0/1 present with chars/colours/names from
the options, stage, seed, delay, defaults for the rest; content hash = ASCII `retail-1.02` zero-padded; rules
profile 1 (no-Slippi build) or 2 (Slippi build); build id = his port version string; match id = 16 bytes
`'c','l','i',0, seed BE32, c0, col0, c1, col1, stage BE16, delay, 0`.
**Ours**: same construction, build id `melee-rust <crate version> <build hash>` (so only identical Rust builds pair),
content hash and profile as his Slippi build (open question Q2).

### 2.7 VERIFY and readiness (control 0x01, 34 bytes: type, digest [32], slot u8)
- Decider, as soon as its handshake completes: chooses the connection its accepted HELLO arrived on, closes the
  others, resends its HELLO on it, sends VERIFY.
- Other peer, on the first VERIFY that decrypts: chooses that connection, closes the others, answers VERIFY.
- Digest or slot (the slot it expects the peer in) different: BYE reason 1, fail. Equal: **ready**.
- After ready: INFO (rev >= 1) once; PING/PONG every 250 ms until frame 1.

### 2.8 Control messages (channel 0)
| type | name | body | size |
| --- | --- | --- | --- |
| 0x01 | VERIFY | digest [32], slot u8 | 34 |
| 0x02 | BYE | reason u8 (0 left, 1 descriptor mismatch) | 2 |
| 0x03 | RESULT | frames i32, transcript digest [32] | 37 |
| 0x04 | INFO | revision u8 (2), games u16 (0 = no limit) | 4 |
| 0x05 | NEXT | game u16, next descriptor digest [32] | 35 |
Unknown types are counted and ignored.

### 2.9 Input messages (channel 1) (`input_transport.cpp:build_commit`, `on_commit`, `on_message`)
- **COMMIT 0x10**, 22 + 8n: frame i32 (newest), count u8 (1..128), checksum frame i32 (0 = none), checksum u32,
  ack i32 (highest F with all of the receiver's 1..F held), stamp u32 (low 32 bits of a microsecond clock),
  pads 8 x count newest first. One COMMIT per simulation tick (held ticks too). Carries every unacked frame (<=128),
  or the newest alone when all are acked. Pad sampled for game frame f is committed as f + delay; frames 1..delay
  are zero pads. Receiver refuses unless length exact, 1 <= count <= 128, frame >= count, checksum frame >= 0,
  ack >= 0, ack <= own newest committed. Oldest carried > held + 1: gap, take nothing. Held frames never replaced
  (different bytes = a conflict count). Frames > own finalized + 200 not taken. Game k: frame / checksum frame
  (non-zero) / ack carry base (k-1) x 2^24; out-of-game numbers drop the whole message silently.
- **ACK 0x11**, 9: ack i32, echo u32 (the COMMIT's stamp). Sent at once for a COMMIT that delivered >= 1 new frame.
  RTT = now32 - echo, discarded above 2 s.
- **PING 0x12 / PONG 0x13**, 5: stamp u32; PONG echoes. Every 250 ms between ready and frame 1.
- **SAMPLE 0x14**, 21 (rev 2): version u8 = 1, target i32 (wire frame), slot u8, pad [8], reserved [6] = 0. Early
  input; prediction only; never sent to a peer below rev 2. Same 13-byte payload as Slippi's early inputs, which our
  `melee_net::early` already models.
- The pad is the first 8 bytes of the game's 12-byte Slippi pad (buttons BE u16, stick x/y, c x/y, L, R): the same
  8 bytes our Slippi wire carries (`melee_net::slippi::pad_bytes`), minus Slippi's ninth seed byte.

### 2.10 Rollback rules (section 8; `rollback_session.cpp`)
Input window: at most 7 peer frames shown, anchored after the last confirmed frame. Stall: frame f may not run
while the window's newest < f - 7. First frame is answered with a neutral pad. Hold > 7 s = disconnected; BYE or a
lost connection = disconnected at once. Time offset per first arrival of a new peer frame F:
`(now - rtt/2) - T + 16683 x (L - F)` us (L, T = own latest COMMIT's frame and time); 30 samples, mean of the middle
third; positive = ahead. Every 30 frames: ahead > 10 ms in frames 1..120: hold up to 5 frames; later ahead > 10 ms
+ 2 frames: hold 1; behind > 10 ms + 1 frame after 120: up to 3 extra frames, one every 5th frame; otherwise speed
up to +1 % (behind >= 0.25 ms) / -0.5 % (ahead >= 8 ms), scaled over 3 frames of offset. These are the same numbers
as Slippi's time sync, which `RbSession` + `slippi.rs` already implement.
Desync: when the peer's (checksum frame, checksum) is for a frame we have: low 16 bits (signed) differ by > 1 =
desync; any other difference = logged once as a risk. Same rule as our Slippi path (`slippi_online::compare`).

### 2.11 Transcript, RESULT, result file
Transcript: BLAKE2b state started with `"MeleeUnlockedNet1 inputs" || descriptor digest`; per frame f in order, once
both pads are known: `f` u32, slot 0 pad [8], slot 1 pad [8]. On game end: RESULT (frames, digest); a peer that has
reached the other's count compares. Result file JSON `{"body":{...},"identity":"<64 hex>","signature":"<128 hex>"}`
+ `\n`, written via a `.part` file and renamed; signature Ed25519 over `"MeleeUnlockedNet1 result"` || exact body
bytes. Body key order (no whitespace): format 1, protocol 1, route `"p2p_direct"`, build_id, descriptor_digest,
match_id, local_slot, peer_identity, transcript_digest, transcript_frames, transcript_complete, peer_agreement
(`"match"`/`"mismatch"`/`"unknown"`), desync, disconnected, has_outcome, mode, frame_length, game_index,
tiebreak_index, winner, end_method, lras_initiator, synced_timer, players (4 x {slot_type, stocks, damage_done as
`%lld.%03lld` thousandths, synced_stocks, synced_damage}), game_info_hash. The outcome fields are the Slippi
game-end report (`kReportGame`); hex is lowercase (to check, Q4).

### 2.12 Several games (rev 1, `--p2p-games`)
INFO games: session plays min of both (0 = 127); a rev-0 peer = 1 game. After a game ends by itself (outcome, no
LRAS quit, no checksum disagreement, limit not reached, peer present): RESULT, then NEXT(game n+1, digest of the
next descriptor): `h = BLAKE2b("MeleeUnlockedNet1 next" || digest_n || (n+1) u32)`; match id = h[0..16], seed =
h[16..20] LE, stage = pool[h[20..24] LE mod 6], pool `0x1F, 0x20, 0x1C, 0x08, 0x02, 0x03`; all else carries. Equal
NEXTs: next game (frame 1 again, frame base (n) x 2^24, new transcript, PING/PONG until frame 1). Different: BYE 1.
No NEXT within 20 s after ours, BYE, or a closed connection: the session is over (no error).

### 2.13 Limits
Plain 1200, protected 1226, HELLO 160, 128 frames per COMMIT, 200 frames lead, replay window 64, 6 candidates,
8 neighbour ports, 8 paths, 256 events / 512 queued per pump, connect 8 s (1..600), input wait 7 s, RTT <= 2 s,
127 games, NEXT wait 20 s.

## 3. How it maps onto our stack

| piece | his | ours: reuse | ours: new |
| --- | --- | --- | --- |
| UDP socket | ENet host | `transport::UdpTransport` | raw 1-byte punch send through the same socket |
| ENet framing | ENet 1.3.13 | `melee_net::enet::Host` (connect both ways, reliable, unsequenced, ping) | per-peer timeout (4/8 s) and 250 ms ping interval knobs; tolerate 1-byte datagrams |
| Ed25519, X25519, ChaCha20-Poly1305 | monocypher | `ring` 0.17 (already in Cargo.lock via rustls; feature-gated) | HChaCha20 (for XChaCha), BLAKE2b plain + keyed: our own, tested against libsodium / hashlib vectors |
| HELLO, keys, seal/open, replay | `session_security.cpp` | | `melee_net::mu_net::sec` |
| descriptor, digest, next | `session_types.cpp` | | `mu_net::desc` |
| connector (dial, punch, path choice) | `peer_connector.cpp` | ENet host | `mu_net::conn` |
| COMMIT/ACK/PING/SAMPLE, time offset | `input_transport.cpp` | offset maths as `slippi.rs` | `mu_net::input` |
| rollback, stall, time sync, checksums | `rollback_session.cpp` + game ASM | `session::RbSession`, `time_sync::TimeSync` (the same section-8 numbers), `slippi_online` checksum + compare, `play_net` Slippi path | a `MuRun` beside `SlippiRun` in `play_net` |
| match setup | `source_p2p.cpp` | `NetDriver::connect_slippi` scenario building, `slippi_online::enable(seed)` | `NetDriver::connect_mu` from `--p2p-*` |
| identity file | `peer_identity.cpp` | | read `ed_seed` (or 32 raw bytes); create only when missing |
| result file | `result_outbox.cpp` | Slippi game-end report fields we already compute for ranked | JSON writer + signature |
| launcher | `launcher.cpp` | `launcher_compat.rs` (desktop) | one mapping line (below) |

Simulation: the match runs exactly as our Slippi-wire direct match (`play --slippi-mm`): Slippi online mode on with
the descriptor's seed, the online codes, the same per-frame checksum. Only the wire differs. Frame mapping is the
Slippi path's (`online frame 1 = session FIRST`).

Launcher mapping wanted from the desktop (`launcher_compat.rs::plan_env`): instead of refusing them, pass every
`--p2p-K V` through unchanged into `play_args` (play's own parser takes `--p2p-*` and routes to the mu path), e.g.
`f if f.starts_with("--p2p-") => { play.push(f.into()); play.push(next_value) }` (all of his `--p2p-*` take a value).
`play` then exits 2 when no game was played (connect / handshake / identity / port failure) and 0 after the game,
as his game does; the build needs `--features real-net` (the `mu-net` feature of melee_net).

## 3b. Status (2026-11-22, port/p2pdirect)

| milestone | state | proof |
| --- | --- | --- |
| M1 crypto + wire | done (`melee_net::mu_net::{blake2b,crypto,wire,desc,sec}`) | unit tests; vectors from libsodium / hashlib |
| M2 connector | done (`mu_net::conn`) | 6 tests over loopback UDP (both dial, one listens, build mismatch, stranger, nobody, neighbours) |
| M3 input session | done (`mu_net::{input,link}`) | unit tests + two Links, 1200 frames with 1/7 datagrams lost, RESULTs agree |
| M4 headless match | done (`melee mu-netplay`, `tools/rust-port/p2p/mu_pair.sh`) | 1200 frames 0 desyncs; 40 ms lag, 10 ms jitter, 5 % loss / reorder, 2 % dup: ~80 rollbacks, 0 desyncs, transcripts agree; build mismatch exits 2/2 |
| M5 `play --p2p-*` | done (`mu_play.rs`, `mu_result`, `play_pair.sh`, `check_results.py`) | two hidden windowed peers with his launcher's line: exit 0/0, 0 desyncs, both signed result files verify and agree; peer killed mid-match: exit 0, `disconnected: true`; no peer: exit 2 |
| M6 several games | done (`Link::{offer_next,begin_next_game}`, `NextGame`; headless `mu_netplay` loop; windowed `mu_next_match` + `Engine::next_vs`) | link tests (3 games over one connection, quit / desync / closed connection end the session); `mu_pair.sh --p2p-games 3`: 3 games, 0 desyncs, stages from the pool; `play_pair.sh --p2p-games 3`: 3 windowed games, all 3 result-file pairs agree; ours vs his reference harness, 3 games: the same chained descriptors (stage 8 then 32, same seeds and digests) and agreeing transcripts |
| M7 SAMPLE | received and counted / kept (`Input::samples`), never sent; not used for prediction yet | |
| M8 wire check vs his build | done (2026-11-22, his `d9448c66`) | see "M8 evidence" below |

Rust-vs-Rust pair tests (M4-M6) prove our two peers agree with each other, not that we match another client; the
cross-client evidence is M8's (his reference mu_net). The time sync is `melee_net::time_sync` from the timesync
merge, which is under suspicion for Slippi live play (port/livesync): re-run the pair tests when that lands.

Known gaps: the result file's `mode`, `tiebreak_index`, `lras_initiator`, `synced_timer`, `damage_done`,
`synced_damage` and `game_info_hash` are left at his defaults (0 / -1); `stocks` / `synced_stocks`, `winner`,
`has_outcome`, `end_method` (2 stocks, 1 time) are filled. Pause is on (as a Slippi Direct match). Failure
diagnosis is one sentence per stage reached, not his four-way NAT diagnosis.

## 4. Milestones (each with its own test; pushed when green)

- **M1 crypto + wire** (`melee_net::mu_net::{crypto,wire,sec,desc}`, feature `mu-net`, implied by `real-net`):
  BLAKE2b / keyed / HChaCha20 / XChaCha20-Poly1305 vs vectors made with libsodium + hashlib (independent code);
  HELLO build/accept (each refusal rule), key schedule symmetry, seal/open + both replay rules, descriptor
  serialize/parse round trip, digest, next-descriptor, message codecs with every length rule. `cargo test`.
- **M2 connector** (`mu_net::conn` on `enet::Host`): dial + accept + punch bytes + decider choice + VERIFY, BYE 1 on
  a mismatched descriptor, connect limit. Test: two in-process peers on 127.0.0.1 UDP (both dialing, one dialing,
  wrong expected identity, wrong build id, limit expiry).
- **M3 input session** (`mu_net::input` + transcript): COMMIT/ACK/PING/PONG with all receive rules, unacked history,
  RTT, offsets, transcript, RESULT compare. Test: unit tests + an in-process loopback "pad pump" with loss and
  reordering (a lossy transport wrapper) ending with equal transcripts.
- **M4 headless match** (`melee mu-netplay`, like `slippi-netplay`): two hidden engine processes on loopback
  (`game.sh 1` / `game.sh 2`), scripted pads, N frames, 0 desyncs, equal transcripts, checksums compared.
- **M5 `play --p2p-*`** (windowed retail flow, `NetDriver::connect_mu`, `MuRun`): option parsing as his, identity
  file, exit 0/2, result file per game. Test: two hidden `play` instances with his exact launcher line shape on
  loopback, result files parse and agree (same descriptor and transcript digest, signatures verify).
- **M6 several games** (INFO/NEXT, frame base, `--p2p-games 0/1`) and **M7 SAMPLE** (rev 2 early inputs, reusing
  `melee_net::early`). Tests: loopback 2-game session; a SAMPLE on/off pair playing identical frames.
- **M8 (optional)** wire check against his reference: a capture of his game's handshake/COMMIT framing only (not
  gameplay), to catch byte-order or nonce mistakes M1's self-consistency cannot.

## 5. Open questions and risks

- **R1 cross-engine**: a Rust peer vs his Source Port can never play: different simulations. Our build id differs
  from his, so the pair fails at VERIFY (digest mismatch, BYE 1) and both launchers show "could not connect". His
  lobby lets each player choose an engine independently; a mixed pair fails cleanly but without a specific message.
- **R2 self-consistency hides wire bugs**: M1-M5 are Rust vs Rust. A wrong byte order or nonce would still pass.
  Mitigation: vectors from libsodium for the crypto; spec-literal codec tests; M8 for framing.
- **R3 ENet compatibility**: our `enet::Host` was built for Slippi; his peers use 1.3.13 defaults plus per-peer
  timeouts. Needs per-peer timeout / ping knobs, 8 peers, and simultaneous connects resolving to one chosen path
  (Slippi prunes too; the rule here is different: decider's HELLO arrival path).
- **R4 port hand-over**: the lobby closes its socket just before the game starts; a busy port gets 2 s of retries.
- **R5 time**: the game connects only after the engine boots (several seconds; our pipeline warm-up). 45 s window
  covers it; the 8 s handshake rule ("within 8 s of start") applies to the host limit, which the launcher raises.
- **Q1** dependency: `ring` is already locked (rustls); we gate it behind a new `mu-net` feature so the default build
  is unchanged. HChaCha20 and BLAKE2b are small and written here. Alternative crates would need vendoring review.
- **Q2** content hash / rules profile values for Rust: use his `retail-1.02` + profile 2 (Slippi rules) and rely on
  the build id to separate engines, or a profile of our own? Proposed: his values, our build id.
- **Q3** what our identity looks like when `--p2p-identity` is missing (a game started by hand): create
  `p2p_identity.json` in the user dir in the same JSON form; never rewrite an existing file.
- **Q4** exact hex case and float formatting in the result body: confirm against his writer's helpers before M5.
- **Q5** game-end report fields (mode, end_method, lras_initiator, synced_*): which of them our engine exposes
  today; missing ones written as his defaults (0 / -1) with `has_outcome` from the match end.

## M8 evidence: our peer against his reference mu_net

Harness (local only, never committed: his code is GPL): his `port/runtime/mu_net/*.cpp` (session_security,
session_types, peer_identity, input_transport, peer_connector, rollback_session, result_outbox), his monocypher and
ENet (incl. `yg_enet_socket.c`), built with g++ against his `port/linux/compat` headers plus a 3-function shim
(BCryptGenRandom over getrandom, ASCII Multi/WideChar), and a small driver that builds the descriptor as his
`build_descriptor` does, runs `RollbackSession::start`, then feeds `on_inputs` 60 times a second with synthetic pads
(finalized = frame - 10, checksum 0), reports the game end (a zero 368-byte payload) and writes his result file.
Build and logs: `~/melee_out/p2pdirect/harness`. Our side: `melee mu-netplay` (real engine), on loopback.

1. His peer against itself: Ready, 300 frames, equal transcripts, agreement 1 (the harness works).
2. **Ours vs his, different build ids** (the real-world case): his log `handshake done with a09aa5f47a675980` (our
   identity), state Verifying, then `The two games do not agree on the match setup`; ours: `descriptor mismatch`, exit
   2. HELLO both ways, the key schedule and the encrypted VERIFY all worked; the digest compare is where it stops, as
   expected.
3. **Ours vs his, same build id** (`MELEE_P2P_TEST_BUILD_ID` = his `--build-id`): both compute descriptor digest
   `34f53c23...` (our canonical bytes and his are identical), `ready: both games hold match setup`, 400 frames of
   COMMIT/ACK both ways: his `rejected 0 malformed 0`, ours `gaps 0, conflicts 0`; both transcripts
   `e2288a4d19063e4b...` over 402 frames, RESULT agreement on both sides (his `agreement 1`, ours `peer agrees`). The
   HARD checksum mismatches on our side are his harness's zero checksums (no game behind it), not a wire fault.
4. Roles and a bad network: ours in slot 0, his in slot 1, 30 ms lag / 8 ms jitter / 5 % loss / 5 % reorder on our
   socket, three runs (ours decider twice, follower once): every run Ready, 600 frames, transcripts agree on both
   sides, his `rejected 0 malformed 0`. (A first try of run 3 failed only because his harness's 30 s connect window
   ran out while ours waited for a game slot.)
5. Result file: his written file verifies with our `verify_file` and with libsodium, has the same key sequence as
   ours, and `mu_net::result` test `body_bytes_match_a_reference_file` (`MU_RESULT_CHECK=<file>`, ignored by
   default) rebuilds its 1064-byte body byte for byte with our writer.

So the wire (framing, nonces, AD, replay, key schedule, descriptor canonical form, COMMIT/ACK/RESULT, transcript) and
the result signing interoperate with his implementation. What still separates a Rust peer from his Source Port in a
real match is the build id (by design) and, behind it, the simulation.

