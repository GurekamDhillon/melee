# P2P Direct on the Rust engine: plan (not built)

2026-11-22. Melee Unlocked 05871a4a. `H:` = his repo, read through `git show 05871a4a:<path>`.

## What the launcher sends

His lobby's P2P Direct (`H:port/app/launcher.cpp:2841-2891`) starts the engine with
`launcher_lobby_p2p.cpp:p2p_arguments` (`:307-335`) plus `--p2p-connect-seconds 45 --p2p-games 1` (Slippi build) or
`--p2p-games 0` (no-Slippi build):

`--p2p-port P --p2p-peer IP:PORT (1..6) --p2p-slot 0|1 --p2p-chars C/c:C/c --p2p-stage S --p2p-seed 8hex
--p2p-delay 1..15 --p2p-identity "<file>" --p2p-expect <64 hex> --p2p-result "<file>" --p2p-names "A:B"`

The game binds the lobby's own UDP port (the lobby closes its socket first). The engine exits 2 when no game was
played, 0 after one (`source_p2p.cpp:session_done`). Non-zero gives his "P2P Direct match could not connect" box.

## Why it is not routed to our Slippi path

It is not Slippi wire. It is his own protocol, mu_net v1 rev 2 (`H:docs/mu-net-protocol.md`, reference
`H:port/runtime/mu_net/*`, about 210 KB of C++ with an 83 KB test file):

- ENet 1.3.13 framing on 2 channels, but a signed Ed25519 HELLO (`MUNH`), X25519 keys and XChaCha20-Poly1305 for every
  later packet. A canonical session descriptor (`MUSD`) is hashed with BLAKE2b and checked with VERIFY.
- Its own COMMIT/ACK/PING/SAMPLE input messages, rollback rules (7-frame window, time sync), a BLAKE2b input
  transcript, a signed result file, multi-game NEXT (section 14), and NAT punching (section 15).
- The descriptor carries a content hash and a **rules profile**: "retail-1.02", with profile 1 for his library
  without the Slippi layer and 2 for the one with it. A peer that does not match refuses the handshake.

Interop with his engine needs two things. The first is the protocol. The second is a simulation that stays
frame-identical to his under his profile's online rule codes. We have not shown that second part. Hero byte-match
(focus-hero-parity) is still open. A wire-perfect implementation would still desync a few seconds into a real
match against his Source Port.

## Plan (in order; each step has a proof before the next starts)

1. **Crypto + framing crate** (`rust/melee_net/src/mu_net/`): Ed25519 and X25519 (the vendored `monocypher` port
   or the `ed25519-dalek`/`x25519-dalek`/`chacha20poly1305`/`blake2` crates if vendoring allows), HELLO, keys
   (section 4.2), protected packets (4.3), descriptor + VERIFY (5). Proof: byte vectors produced by his
   `mu_net_test.cpp` cases, reproduced as Rust unit tests. This is read-only use of his test source as a spec.
2. **Connector** (section 3 + 15 without STUN/UPnP first): dial all candidates, 100 ms keepalive bytes, accept any.
   Proof: two Rust peers on loopback.
3. **Input session** (sections 7-9): COMMIT/ACK/PING/PONG/SAMPLE, the 7-frame window, stall rule, first neutral
   frame, time sync, checksums, transcript, RESULT. Plug it into `play_net::NetDriver` as a third transport beside
   our netplay and Slippi links. Proof: Rust-Rust loopback match, 0 desyncs, equal transcript digests.
4. **Launcher mapping**: `launcher_compat.rs` maps `--p2p-*` to `play --mu-p2p ...`. It reads the identity file
   (`lobby-peer-identity.json` beside the launcher; its own key, not a Slippi login) and writes the signed result
   file in section 11's exact key order. Exit codes 0/2. `--p2p-games 0/1` (section 14 NEXT; one game first).
5. **Cross-engine**: one hidden loopback pair, his `melee_source.exe` (his scratch build, never his repo) against
   ours, `MELEE_PAD_SCRIPT` both sides. It first needs the online rule codes of his profile 1 applied the same way
   in our engine. Expect desyncs until the Hero byte-match work lands; the transcript digests show whether the inputs
   agree and the checksums show where the simulations part.

Size: steps 1-4 are about 3-5k lines of Rust. Step 5 depends on parity work outside this lane.

Until then `--p2p-*` stays refused with exit 3 and a log line, so the launcher shows its "could not connect" box
instead of a different game starting.
