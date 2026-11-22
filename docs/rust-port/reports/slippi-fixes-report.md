# Slippi fixes — 2026-09-27

Code only: no builds, game runs, or commits. Workspace sources were read only; their changes are delivered in `workspace-slippi.patch`.

## Early-ended negative controls

The runner rejected exit 3 before reading evidence; the comparer required the original fixture's full length. The workspace patch updates `two_client_replay.py`, `compare_finalized.py`, and their tests.

- Only negative controls accept paired exits `(3, 3)`. Mixed exits and other failures remain failures. Normal acceptance still requires full-length artifacts.
- Native evidence adds `exit_code` and `end_frame`. Both early-ended clients must report exit 3, the same actual end frame, and the same original fixture length. Recording-finalization failure becomes exit 2.
- Comparison covers the shorter finalized recording length, bounded by confirmation and the comparison fixture. Complete containers, contiguous frames, trace/input coverage, and checksum coverage remain required.
- Client consensus requires identical states, processed inputs, and checksums over that range. Negative controls additionally require actual gameplay divergence from the original fixture at or after the mutation. A missing tail alone cannot pass.
- Old early-end evidence lacking the new fields fails closed. Regenerate it using the updated native code. The frame-3282 game scenario was not rerun.

Apply from the workspace root when ready:

```text
git apply --check worktrees/codex-slippi-fixes/workspace-slippi.patch
git apply worktrees/codex-slippi-fixes/workspace-slippi.patch
```

## Peer loss

`MELEE_SLIPPI_TIMEOUT_MS` defaults to 5000 and accepts integers 1000..60000. The workspace runner exposes `--peer-timeout-ms` and forwards it through its sanitized environment. The setting is documented in `pc/docs/PORT_DEV_QUICKREF.md`.

`gw_slippi_peer.c` configures ENet timeout minimum and maximum equally, plus 250 ms ping intervals, on outgoing and accepted connections. Equal bounds prevent the retransmission/backoff condition from shortening outage grace. Terminal disconnect is exposed only when the final established connection is lost; losing a duplicate connection does not invalidate a surviving path.

`gw_SlippiMode_Tick` replaces the 60-second watchdog with the configured gameplay-progress timeout. Traffic without gameplay progress cannot reset it. Completed clients tolerate peer closure during the final ACK grace period.

Terminal loss logs the cause and displays either “Opponent disconnected. Match ended.” or “Connection lost: opponent stopped responding.” Simulation waits while the normal event/render loop remains available for a two-second toast, then the diagnostic finalizes evidence and exits 2. Detection is approximately five seconds; process exit includes the extra message interval. ENet scheduling/backoff can affect transport timeout timing; the gameplay watchdog independently bounds a stalled match.

Three seconds is below the default timeout, preserving the recovery window by design. Lower overrides reduce that grace. Actual game recovery and message rendering remain unverified.

## Verification

Python tests used local copies of workspace tools, synthetic replays, mocked launchers, and local UDP sockets. No game process was launched. The full suite printed PASS/FAIL lines.

```text
PASS compare_finalized: 31/31
PASS two_client_replay: 8/8
PASS alter_fixture: 3/3
PASS udp_relay: 6/7
FAIL test_udp_relay.UdpRelayTest.test_stall_holds_then_delivers
     elapsed 0.0 seconds; expected >= 0.02 seconds
PASS workspace patch: git apply --check
PASS game diff: git diff --check
```

Full Python suite: 48 passed, one failed (49 total). The failing relay test and relay implementation are unchanged. Its timing assertion failed consistently in this session; the suite is not reported as green. Initial runner tests also exposed missing timeout attributes in synthetic argument objects; default handling fixed those errors.

New synthetic tests cover unequal finalized tails, different end frames, missing/mismatched hashes, truncation without divergence, divergence relative to the mutation frame, and paired versus mixed exit codes.

Native tests added but **not built or run**:

- `pc/tests/slippi_mode_test.c`: default/custom timeout parsing, invalid values, three-second grace, exact expiry, progress reset, and unsigned clock wrap; existing `slippi-mode: PASS/FAIL` summary.
- `pc/tests/slippi_peer_test.c`: final established peer disconnect, silent raw-host disappearance, and a three-second endpoint polling pause followed by acknowledged PAD delivery. New `slippi-peer-disconnect`, `slippi-peer-silent-loss`, and `slippi-peer-3s-recovery` PASS/FAIL lines.

Remaining runtime checks: build through the workspace pipeline; confirm new strings in the executable; run native tests; repeat the early-end negative control, real three-second UDP blackout, permanent loss, and gameplay stall with live ENet; verify the toast and log/evidence. Native compilation, linking, timing, rendering, and in-game recovery remain unverified by instruction.

Local `.slippi-fix-work/` contains the tool copies used for Python verification. An automatic policy rejection blocked a combined report-writing/scratch-cleanup command. The report was subsequently written without cleanup; the scratch directory remains for inspection.
