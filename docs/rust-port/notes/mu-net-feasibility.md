# mu_net (Melee Unlocked's P2P protocol): can the Rust engine speak it? (2026-11-03)

Read from `scratch/ext/melee-unlocked` (reference only; nothing copied): `docs/mu-net-protocol.md` (version 1,
revision 2), `port/runtime/mu_net/*` (about 3.7k lines of C++, crypto through monocypher), `port/app/source_p2p.cpp`
(the game's `--p2p-*` command line) and `port/app/launcher_lobby_p2p.*` (the launcher side).

## What the protocol is
- Transport: UDP with ENet 1.3.13 framing, two channels (0 reliable control, 1 unsequenced input). NAT work (one-byte
  punch datagrams to up to 6 candidates and neighbouring ports, STUN, NAT-PMP / UPnP) is in the launcher and the game;
  there is no relay and no server.
- Security: a signed `MUNH` HELLO (Ed25519 identity, X25519 ephemeral), BLAKE2b key schedule, every later packet
  XChaCha20-Poly1305 with a sequence nonce and a replay window.
- Session descriptor (`MUSD`, canonical bytes, BLAKE2b digest exchanged in VERIFY): route, **build id**, **content hash**,
  rules, match id, four player slots (character, colour, name, keys), stage, seed, delay, rollback horizon 7, stocks,
  timer, feature bits. Both digests must be equal before frame 1.
- Inputs: COMMIT messages carrying the 8-byte pad prefix of each frame (newest first, up to 128), ACK, PING/PONG, a
  checksum pair per COMMIT (desync = low 16 bits differ by more than 1), an input-transcript digest in RESULT.
  Revision 1 adds several games per connection (INFO, NEXT, a chained next descriptor that also draws the next stage
  from a fixed pool), revision 2 early-input SAMPLEs.
- The game is started by the launcher with `--p2p-port/--p2p-peer/--p2p-slot/--p2p-chars/--p2p-stage/--p2p-seed/
  --p2p-delay/--p2p-identity/--p2p-expect/--p2p-result/--p2p-names/--p2p-connect-seconds/--p2p-games`: the match is
  fully agreed in the launcher's lobby before the engine starts (no CSS over the wire).

## Fit with what the Rust port has
- The rollback core maps: `RbSession` already does delay, a window of 7, stall-on-window and checksum exchange; a
  mu_net transport would be a second `Net`-like adapter (COMMIT/ACK instead of gw_netplay's packets). The pad is the
  same 8 bytes (button u16, sticks, triggers). Frame numbering differs (mu_net frame 1 = the first match frame, ours
  `FIRST` = -123); a fixed offset.
- Launcher side: `launcher_compat.rs` already recognises the launcher's line and refuses `--p2p-*` (exit 3).
  Accepting it means building the descriptor from those options exactly as his game does (`build_descriptor`).
- Missing pieces, each self-contained but real work:
  1. ENet framing in Rust (or a binding to `extern/enet`, which the C tree already vendors): connect / verify /
     reliable ordered channel / unsequenced channel / pings. Roughly 1-2k lines to reimplement faithfully.
  2. Crypto: Ed25519, X25519, BLAKE2b (keyed), XChaCha20-Poly1305. The workspace has no Rust crates for these;
     adding them (ed25519-dalek, x25519-dalek, blake2, chacha20poly1305) needs vendoring and a licence check, or a
     binding to monocypher-equivalent C. Writing them by hand is not sensible.
  3. The descriptor, VERIFY, transcript, RESULT file (signed JSON with a fixed key order), INFO/NEXT for rematches.
  4. Identity key files (`--p2p-identity`) in his format.

## The blocker that is not code
The descriptor digest covers the **build id** and the **content hash** of the game both sides run, and rollback only
works when both peers simulate bit-identically. A Rust peer against a Melee Unlocked Source Port peer is therefore
not a protocol question: the two engines are different programs (GD's decomp-derived engine vs his source port), and
even GD's own C vs Rust builds diverged over the wire (net.md, "Rust vs C build over the wire": a desync at session
frame 89 from an effect RNG gap). Faking his build id to pass VERIFY would just produce desyncs. What mu_net could
give is Rust-vs-Rust matches launched from his lobby (both players choosing "Rust engine"), with our own build id.

## Verdict
Not implemented: not clearly tractable in this round. Rust-vs-Rust over mu_net is feasible (estimate: ENet +
crypto + descriptor/handshake + COMMIT adapter + launcher option translation, several days, plus the dependency
decision for the crypto), and cross-engine play with his source port is not achievable by protocol work at all.
Recommended order if pursued: (1) decide the crypto/ENet dependency route; (2) a loopback Rust-vs-Rust mu_net test
reusing `RbSession`; (3) map `--p2p-*` in `launcher_compat.rs`; (4) a check against his reference implementation
only for the handshake and COMMIT framing (a capture of his game talking to itself), never for gameplay.
Replays are the cheaper win for his users: the `.slp` export now lands in the launcher's `--replay-dir`.
