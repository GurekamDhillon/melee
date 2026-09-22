# Rollback session layer — design, results, what remains

Branch `agent/rbsession` (melee; includes `agent/snapcost`). Code: `pc/platform/gw_rollback.c` + `.h`;
hooks in `pc/platform/gw_replay.c` (input source, per-iteration trace buffering, UCF raw bytes),
`src/melee/gm/gmscene.c` (`RB_Enabled/RB_Iterations/RB_IterStart/RB_TickEnd/RB_SceneBegin`),
`src/sysdolphin/baselib/controller.c` (live-pad latch, pad status from the ring),
`src/sysdolphin/baselib/axdriver.c` (sound claim/new/release), `src/melee/ft/fighter.c` (curated hash),
entry points appended to `pc/platform/gw_snap.c`.
**On merge add `gw_rollback.obj` to the root `_build/melee_link_objects.rsp`** (after `gw_snap.obj`).

## Design as built

- **Time**: Slippi's logic frame (-123 first), owned by `gw_replay.c`; a load rewinds it. Frames are
  numbered per scene and each scene is a new **epoch** (`gw_rb_epoch`); labels from another epoch are
  dropped (older) or held until reached (newer) — `gw_rb_submit_remote_input_e`, `gw_rb_checksum_e`.
- **Inputs**: per slot (port*2+follower) two 64-frame rings: `truth` (confirmed: delivered or sampled)
  and `used` (what the sim consumed, confirmed/predicted flag). Two entry kinds:
  - *processed* (replay-style: sticks/trigger/buttons after the game's processing) — fighters read them
    at the .slp injection point (`Fighter_Spaghetti_8006AD10`);
  - *raw controller* (`is_raw`: a whole PADStatus) — fed to `HSD_PadRenewMasterStatus` in place of the
    pad queue, also during resimulation, so deadzones/calibration/edges/UCF raw reads run as on a console.
  UCF's raw-byte reads (`gw_Replay_RawStick/RawStickBack`) come from `used`, so a prediction's raw
  bytes — not the future truth — feed UCF's pad buffer.
- **Sources** (`MELEE_RB_INPUT`): `replay` (the .slp, processed), `padgen` (deterministic raw pad
  generator, hash of seed/port/frame, for automated tests of the raw path), `live` (real pads).
- **Live pads**: every `PADRead` is latched per port (`gw_RB_PadLatch`): buttons OR'd between logic
  frames (a tap that starts and ends between two logic frames is kept), trigger/analog **peaks**, newest
  stick. Sampled once per new frame for frame `next + D`; the latch then restarts from what is held.
- **Delay** D (`MELEE_RB_DELAY`, default 2). **Prediction**: the slot's last contiguous confirmed input.
- **Rollback**: a delivered input differing from `used[f]` sets `first_wrong`; the tick plans
  `k = next - first` resimulated iterations + the new one; the first iteration loads S[first]; every
  iteration re-saves its snapshot. A resimulated frame also runs the render calls (minus present), with
  draws suppressed (snapcost).
- **Stall** when `next - confirmed > MAX` (`MELEE_RB_MAX`, default 7) and at the match end until fully
  confirmed. **Wait** (`gw_rb_request_wait(n)`): the time-sync actuator — skips new-frame iterations,
  rollbacks and rendering continue; not counted as stalls.
- **Ring safety**: inputs more than 56 frames ahead of the sim are refused (a conforming peer stalls at
  MAX ahead of what it has confirmed from us).
- **Checksums (two tiers)**: `gw_rb_checksum(frame)` = CURATED gameplay hash (RNG seed + per fighter:
  motion, position, velocity, damage, facing) of the state at the frame's start, published only for final
  frames (all inputs before it confirmed, no pending correction). `gw_rb_checksum_full` = snapcost's
  whole-state hash, SyncTest only (two machines legitimately differ in render-owned bytes).
- **Sound**: a resimulated frame's sound starts are matched to the first pass's **by id**: a repeat gets
  the existing handle (already playing); a sound only the corrected timeline plays is played and
  remembered; at the end of each resimulated frame, first-pass sounds nothing re-emitted are released
  (`HSD_AudioSFXKeyOff`). Rumble is reset to idle on every load (gw_snap).
- **Fake network** (`MELEE_RB_FAKE=lat[,jitter[,loss%]]`, remote ports `MELEE_RB_REMOTE` hex mask,
  default port 1): the remote slot's input for frame F arrives at clock tick `F+123-D+lat±j` (+timeout
  when lost); the peer is capped at `next+MAX+D` ahead. The clock advances only when a NEW frame is
  simulated or on the session's own stall/wait ticks, so holds (the loading hold freezes the match) and
  rollbacks during holds do not give it a lead. Reproducible (`MELEE_RB_SEED`).
- **Trace** (`MELEE_STATE_TRACE`): rows buffered per iteration, written when final — equals plain playback.
- Diagnostics: `MELEE_RB_LOG=<n>`, `MELEE_RB_HASHLOG=<file>` (final curated hashes), `MELEE_RB_WAITTEST=1`
  (2 frames of wait every 40 ticks), per-depth tick cost table every 1200 ticks.

## Acceptance results (confirmed-frame trace vs zero-latency playback, `tools/replay/det_diff.py`)

RustyJuicyElephant.slp (3.19 online, Fountain of Dreams), whole replay -123..9522:
- `MELEE_RB_FAKE=4,2`: `trace identical through frame 9522` (2597 rollbacks, avg depth 2.6, max 4)
- `MELEE_RB_FAKE=7`: `trace identical through frame 9522` (3753 rollbacks, depth 5, 31 stalls)
- `MELEE_RB_FAKE=5,3,10`: `trace identical through frame 9522` (362 rollbacks, max depth 7, 33 stalls)
- latency 0 / 2 (covered by delay 2, no rollbacks): identical through 6060 / 5545 (runs cut by time)
- latency 6 / 7 with delay 0 (depth 6 / 7 every frame): identical through 1652 / 1457 (cut by time; the
  pre-snapcost build ran ~5 fps then)
Gang-Steals/11/Game_20190308T171303.slp (1.7.1, Battlefield, remote port 2), whole replay -123..6107:
- latency 6, delay 0: `trace identical through frame 6107` — 3367 rollbacks, depth 6 every frame
- latency 7, delay 0: `trace identical through frame 6107` — 3351 rollbacks, depth 7 every frame
- latency 5±3, 10% loss: `trace identical through frame 6107` — 374 rollbacks, max 7, 12 stalls
Raw pad path (`padgen`, through the game's pad pipeline, match to stock-out -123..1720): latency 4±2
(1074 rollbacks), 6 with delay 0 (1833, depth 6), 5±3 + 10% loss — all `identical through frame 1720`.
Curated checksum: 1955/1957 final-frame hashes at 4±2 and 5±3+10% identical to latency 0 (0 mismatches).
Wait actuator: `MELEE_RB_WAITTEST=1` identical to the same run without waits (after the ring-safety fix).
Desyncs: 0 in every run. SyncTest k=3 still passes (1700 rollbacks, 0 mismatching). Headless 76/76.

## Cost (after snapcost's dirty-page snapshots)

Per tick, Battlefield, `MELEE_PROFILE` frame time: depth 6 every frame p50 16.67 / p99 17.35 ms (fits);
depth 7 every frame p99 21.35 ms. Components on a depth-7 tick: save 2.5, load 0.37, resimulated
iterations ~0.55-0.9 ms each, new iteration ~13-19 ms (stage dependent). Fountain of Dreams is slower
regardless of rollback: plain playback p50 19.0 ms, session without rollbacks 19.5, latency 4±2 20.0 —
FoD's rendering (the water reflection re-renders the scene), not the session.

## yampp findings (coordinator's list)

1. Live-pad latch (edges OR'd, trigger peaks, newest stick) — **adopted** (`gw_RB_PadLatch`).
2. Two-tier hashing — **adopted** (`gw_rb_checksum` curated, `gw_rb_checksum_full` whole-state).
3. Scene epochs — **adopted** (per-scene frame numbering, `_e` API). Menus/CSS/results are not simulated
   by the session yet (VS matches only), so today only matches carry frames.
4. Time sync — the session side is **adopted** (`gw_rb_request_wait`, and `gw_rb_frame_advantage`); the
   "ahead peer gives back half the gap" rule and the two-clock-rate drift test belong to the transport
   (gw_net already implements GGPO's rule and a slow-peer test). Verified here: waiting never changes
   the confirmed result.

## What remains

- Live play with two humans (MELEE_RB_INPUT=live): smoke-tested with a pad script (both ports act, one
  early rollback); not yet played by people. UCF for live matches: the raw path already feeds UCF's raw
  reads, but UCF's gates (`gw_Replay_UcfVersion/UcfCardinals`) only switch on during .slp playback —
  add a MELEE_SLIPPI_CODES-driven gate for live matches.
- Menus/CSS/SSS are outside the session (the fake mode starts at the match); netplay needs a lockstep or
  session path there too.
- Sound: music start/stop and looping SFX kept alive by per-frame calls are not deduplicated; a sound
  the corrected timeline starts late is played late (no seek).
- `gw_rb_frame_advantage` uses the local confirmed frame; with a transport it should use peer-reported
  frames (gw_net computes its own).
- FoD frame time (rendering) exceeds 16.7 ms even without rollback.
- The trace/hash buffers are diagnostics-only overhead; off unless the env vars are set.
