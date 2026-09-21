# Rollback netcode transport (`gw_net`) - protocol spec and evaluation

Branch `agent/netcode`. Code: `melee/pc/platform/gw_net.{c,h}` (transport), `gw_net_tests.c` (18 headless
tests, registered from `gw_tests_core.c`). Companion: `_research/rollback-netcode.md` (design),
`pc/platform/gw_rollback.h` on `agent/rbsession` (the session layer this talks to).

## STATE / NEXT STEPS (read this first)

**Works, tested:** the whole transport, standalone. Headless suite **94/94** (76 existing + 18 `net_*`),
~4.5 s total, stable over repeated runs. Build: shims `gw_net.c`, `gw_net_tests.c` (+ the edited
`gw_tests_core.c`); `ws2_32` is linked by `#pragma comment(lib, "ws2_32.lib")`, so **no link-list change
beyond the two object lines** is needed.

**To merge into pc-port:** add to the ROOT `_build/melee_link_objects.rsp`, after the `gw_snap.obj` line:
`../masstest/shimobj/gw_net.obj` and `../masstest/shimobj/gw_net_tests.obj`. Then
`bash tools/port/build.sh --shim gw_net.c --shim gw_net_tests.c --shim gw_tests_core.c`. A fresh worktree may
also need `--shim gw_runtime.c --shim gw_replay.c --shim gw_snap.c` (stale hardlinked objects).

**Not done (the game-side integration - nothing in gw_net touches game memory):**
1. The ~40-line adapter between `gw_net` and `gw_rollback.h` (mapping in the header comment of `gw_net.h`;
   exact calls below). rbsession's header exists on `agent/rbsession` (`3b85de51b`).
2. A way to start a netplay match: host/join UI or env (`MELEE_NET=host:PORT|join:IP:PORT`), building
   `gw_net_config` (exe hash of `melee-pc.exe`, ISO hash via `gw_net_hash_file(iso, header+FST bytes)`,
   mods hash, `StartMeleeData` as `match_blob`, seed, slots, delay), and entering the match from the blob
   on both peers (the replay module's `gw_Replay_ApplyMatch` path is the model).
3. Session stalls should also consult `gw_net_recommend_wait`, `gw_net_unacked`, `gw_net_silent_ms`.
4. Real two-machine test (the UDP path is only tested over 127.0.0.1); NAT traversal/relay is out of scope.

**Decision on the design question:** hand-rolled, confirmed (see Evaluation).
**Known limits:** one guest per host; UDP payloads <= 1200 B (max redundancy window is
`min(40, 1000 / (payload * local_slots))` frames - 17 frames for two 28-byte slots); the time-sync
constants (EMA 1/8, cooldown `2w+6`) are tuned against the simulator only.

## Evaluation: library vs hand-rolled

Facts checked (GitHub READMEs fetched 2026-09-21):
- **GekkoNet** - "C/C++ ... SDK", C++20, ASIO optional (`NO_ASIO_BUILD`), MSVC/GCC/Clang, **no mention of
  32-bit/x86**, event-driven session that advances frames itself, abstracted socket manager, BSD-2-Clause.
- **GGPO** - Windows-only build (VS2019 + CMake), MIT, ~115 commits. Callback model
  (`save_game_state/load_game_state/advance_frame`) from the previous research doc (not re-read).

Reasons for hand-rolled (the research doc's recommendation stands):
1. **The game is a 32-bit `i686-pc-windows-msvc` exe with a scene loop we do not own.** Both libraries want
   to own "advance frame"; ours is `gm_801A4D34`'s loop plus `gw_snap`/`gw_rollback` (26 MB MEM1 snapshots,
   resimulate-with-render). A library's save/load callbacks would just forward to `gw_snap` - the
   transport is the only part of a library we would use, and it is ~900 lines.
2. **No verified 32-bit build for GekkoNet (C++20)**; GGPO is old, Windows-only, and hides its socket loop.
3. **Testability.** A pluggable transport + clock lets every protocol path (loss, dup, reorder, clock wrap,
   silence) run in deterministic virtual time in the headless suite. Neither library offers that seam for us.
4. **Wire control.** Opaque per-slot payload of negotiated size (9 B raw pad ... 28 B `GwRbInput`), slot
   masks over `port*2+follower`, signed frame numbers (-123), checksum piggyback - all cheap to have.
Cost: we own the bugs. Mitigation: 18 tests including a 10k-frame 50% loss / 150 ms jitter run.

## Model

- Single-threaded: everything happens inside `gw_net_poll(net, local_frame)`, called once per render tick.
- **Slots** = `port*2 + follower` (0..7); each peer owns a slot mask; one payload (`payload_bytes`,
  1..32, negotiated) per owned slot per frame. **Frames** are the session's signed numbers; on the wire
  they are u32 indexes from `first_frame` (-123).
- Input delivery to the session: `cb.remote_input(frame, slot, payload)` **exactly once, strictly
  increasing frame order** (all slots of frame f, ascending, before f+1). It may fire before the local
  `EV_STARTED` (jitter): the session's ring must accept early inputs.
- Local inputs: **push** (`gw_net_submit_local`, contiguous from first_frame) or **pull**
  (`cb.local_input(frame, slot, out)` asked for consecutive frames until it returns 0 - the shape of
  `gw_rb_local_input_for_send`).
- Checksums: **push** (`gw_net_report_checksum`) or **pull** (`cb.checksum(frame, &h)`, asked for frames
  up to `remote_next`, i.e. the newest whose remote inputs are all in - the shape of `gw_rb_checksum`).
- Clock: `cfg.now_ms` (NULL = QueryPerformanceCounter). All comparisons are wrap-safe uint32 (a test wraps
  the guest clock mid-run; the two peers' clocks are unrelated).

## Wire format (little-endian, explicit field encoding)

Header, every packet (18 bytes):

| off | size | field |
|---|---|---|
| 0 | u16 | magic `0x4E47` |
| 2 | u8 | type: 1 HELLO, 2 ACCEPT, 3 REFUSE, 4 READY, 5 START, 6 START_ACK, 7 INPUT, 8 QUIT |
| 3 | u8 | flags (bit0: `t_echo` valid) |
| 4 | u32 | session id (0 in HELLO/REFUSE) |
| 8 | u32 | `t_send`: sender's clock, ms |
| 12 | u32 | `t_echo`: last `t_send` received from the peer |
| 16 | u16 | `echo_delay`: ms the echo was held |

RTT sample = `now - t_echo - echo_delay` on every packet (TCP-timestamp style), smoothed 7/8.

Bodies:
- **HELLO** (guest->host): u16 protocol version (2) | u64 exe_hash | u64 iso_hash | u64 mods_hash |
  u8 payload_bytes | i32 first_frame.
- **ACCEPT** (host->guest): u16 version | u32 seed | u8 input_delay | u8 host_slots | u8 guest_slots |
  u16 blob_len | blob (<= 900 B; the match config, e.g. `StartMeleeData`).
- **REFUSE**: u8 code | u8 len | text (<= 90). Reasons: protocol version, `different melee-pc.exe build`,
  `different disc image`, `different mod pack`, `different netplay settings (input size / first frame)`,
  `session full` (a second guest).
- **READY / START_ACK / QUIT**: empty. **START**: u32 ms until start on the host's clock (remaining, so a
  retransmit stays exact).
- **INPUT**: u32 local_frame idx | s8 adv (sender's smoothed frame advantage) | u32 ack_next (all remote
  frame idx below this are delivered) | u32 first idx | u8 count | count x (nl x payload) | u8 nchk |
  nchk x (u32 frame idx, u32 hash). `first` = the oldest frame the peer has not acked, so **every
  unacked frame is resent in every packet** (GGPO-style redundancy, no retransmit timers); nchk <= 8
  most-recent reported checksums.

## Handshake state machine

```
guest: CONNECTING --HELLO every 250 ms--> host
host : LISTENING  --valid HELLO--> ACCEPTED (locks the peer, picks session id) --ACCEPT--> guest
       invalid HELLO -> REFUSE, stays LISTENING (a refused guest does not use the slot)
       HELLO from another address while locked -> REFUSE "session full"
       repeated HELLO in ACCEPTED (our ACCEPT was lost) -> ACCEPT again
guest: ACCEPTED (config received; EV_ACCEPTED)  --READY every 150 ms-->
host : on READY -> STARTING; start = now + max(300 ms, 2*RTT + 150); START (remaining) every 100 ms
       until START_ACK or any INPUT arrives (INPUT also acknowledges START)
guest: on START -> STARTING; start = now + remaining - RTT/2; START_ACK
both : at their start time -> RUNNING (EV_STARTED). Frames begin; INPUT flows.
       start times agree to ~1 ms in real time (tested with unrelated clocks).
timeouts: guest handshake 10 s -> DEAD "handshake timed out"; silence >= notify_timeout (1 s) ->
       EV_INTERRUPTED, >= disconnect_timeout (5 s) -> DEAD "peer timed out"; a packet after INTERRUPTED
       -> EV_RESUMED; QUIT x3 on free -> peer DEAD "peer quit".
```

## Time sync (GGPO's frame advantage)

On each INPUT: `remote_now = pkt.local_frame + (RTT/2)/frame_ms`; `adv = local_frame - remote_now`
(positive = we are ahead); EMA (1/8) of our `adv` and of the peer's reported `adv`. The peer that is ahead
waits `w = floor((adv_local - adv_remote)/2)` frames (capped 8): `gw_net_recommend_wait()` returns `w`
and starts a cooldown of `2w+6` polls so the estimate can reflect the stall. Test: one peer at 0.85x
speed, 50 ms + jitter + 5% loss: gap stays <= 6 frames with sync vs >= 200 without.

## Desync detection

Both peers report the checksum of each frame once its remote inputs are in; the last 8 ride on every
INPUT packet. On a mismatch for a frame both have: `cb.desync(frame, local, remote)` once, and
`gw_net_desync_frame()` keeps the smallest mismatching frame. No false alarms at 30% loss (tested).

## The game-side adapter (for `gw_rollback.h`)

```
cfg.first_frame = -123;  cfg.payload_bytes = 28 (GwRbInput minus seed; or 9 = raw pad - see below)
cb.local_input  = { gw_rb_local_input_for_send(slot, frame, &in) ? (encode(in, out), 1) : 0 }
cb.remote_input = { decode(payload, &in); gw_rb_submit_remote_input(slot, frame, &in) }
cb.checksum     = { h = gw_rb_checksum(frame); *out = h; return h != 0 }
per render tick : gw_net_poll(net, gw_rb_current_frame());  stall += gw_net_recommend_wait(net);
```
**Payload choice.** `GwRbInput` is post-processing (floats + processed buttons + raw bytes) = 28 B/slot.
Sending the **raw pad** (9 B) and running the game's own deadzone/UCF processing on both peers is smaller
and desync-proof by construction, but needs the session to inject at the pad master rather than at the
fighter's pad read - a decision for the session/adapter author; `payload_bytes` supports either.

## Tests (`gw_net_tests.c`, all in the headless suite)

Simulated network = deterministic virtual time, loss/dup/jitter(reorder)/blackhole, per-peer clock
offset (guest clock wraps uint32 mid-run), slots 0,1,4 vs 2,6.
`net_hash_addr`, `net_handshake` (config intact, start times agree, RTT), `net_refuse_{exe,iso,mods,settings}`
(+ slot survives a refusal, + "session full"), `net_inputs_clean` (10k frames), `net_inputs_lossy`
(20% loss, 50 ms jitter, 10% dup, 28-byte payload), `net_inputs_hostile` (50% loss, 150 ms jitter),
`net_inputs_pull` (callbacks supply inputs, 28 B), `net_frame_sync` (with control), `net_interrupt_resume`,
`net_peer_timeout` (~300 ticks after the cut), `net_desync` / `net_desync_pull`, `net_quit`, `net_api_misc`
(sequence/window/config validation), `net_udp_loopback` (real Winsock, real clock, 200 frames each way).
Every delivery is checked for exactly-once, in-order, byte-exact.
