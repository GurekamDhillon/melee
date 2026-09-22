# Rollback session layer — design and STATE / NEXT STEPS

Branch `agent/rbsession` (melee). Code: `pc/platform/gw_rollback.c` + `.h`, hooks in
`pc/platform/gw_replay.c` (input source + per-iteration trace buffering) and
`src/melee/gm/gmscene.c` (`RB_Enabled`/`RB_Iterations`/`RB_IterStart`/`RB_SceneBegin`), entry points
appended to `pc/platform/gw_snap.c` (`gw_Snap_OpenSession`, `_SessionResim`, `_HasFrame`, `_Checksum`).
**On merge add `gw_rollback.obj` to the root `_build/melee_link_objects.rsp`** (after `gw_snap.obj`).

## Design as built

- **Time** is Slippi's logic frame (-123 first), owned by `gw_replay.c`; a rollback rewinds it with the
  snapshot cursor.
- **Inputs**: per slot (port*2+follower) two rings of 64 frames: `truth` (confirmed inputs delivered or
  sampled) and `used` (what the sim consumed, with a confirmed/predicted flag). Fighters read them through
  `gw_replay.c`'s accessors, which ask `gw_RB_InputFor` once a session is active (same injection point
  as .slp playback, so UCF/pad processing is untouched); `RawStickBack` reads `used` too, so predicted
  raw bytes — not the future truth — feed UCF's pad buffer.
- **Prediction** = the last contiguous confirmed input of the slot, repeated (neutral before any).
- **Rollback**: a delivered remote input that differs from `used[frame]` sets `first_wrong`; the next tick
  plans k = next - first resimulated iterations + the new one (`gw_RB_Iterations`); the first iteration
  loads S[first] (`gw_RB_IterStart`); every iteration re-saves its snapshot (`gw_snap_save(next)`), so
  snapshots always belong to the current timeline.
- **Stall**: the new frame is skipped (rollback iterations still run) when `next - confirmed > MAX`
  (MELEE_RB_MAX, default 7); at the replay's end the "past the last frame" tick waits for full
  confirmation.
- **Fake network** (`MELEE_RB_FAKE=lat[,jitter[,loss]]`): remote ports (`MELEE_RB_REMOTE`, default port 1)
  get the replay's recorded inputs at tick `F+123-D+lat+j` (D = `MELEE_RB_DELAY`, default 2); loss adds a
  retransmit timeout; jitter/loss are hashed from `MELEE_RB_SEED`, so a run is reproducible. The network
  clock `tk` counts render ticks from the first simulated frame (stalls advance it).
- **Trace** (`MELEE_STATE_TRACE`): rows (trace, vel, seed, rand) are buffered per logic iteration
  (`gw_Replay_TraceBeginIter`) and written only when the iteration is final
  (`gw_Replay_TraceFlushUpTo`), so the file equals a plain playback's.
- **Sound**: resimulated iterations set `gw_Snap_SessionResim(1, frame)`, which gates SFX starts to the
  first pass's handle cache (SyncTest's `SfxPut/Take`). Not yet done: let a genuinely NEW sound of the
  corrected timeline play (`SfxTake` miss returns -1 today = silent), stop sounds of the abandoned timeline,
  music. Rumble is reset on every load (gw_snap).
- Env: `MELEE_RB_LOG=<n>` logs the first n mismatches/rollbacks.

## STATE (when paused)

- Builds and runs. `MELEE_RB_FAKE=4,2` plays RustyJuicyElephant with real rollbacks (74 by frame 265,
  average depth 2.7, max 4; no stalls, no desyncs; save 2.5-3.2 ms, load 2.4 ms, resim iteration
  1.2-1.6 ms).
- **Acceptance NOT passing yet**: the session trace differs from the plain-playback baseline
  (`_build/runs/rb_base/trace.csv`, full replay to frame 9522) first at frame -28 (Falcon's first dash,
  baseline action 20, session 42).

## THE BUG, and the NEXT STEP (do this first)

`gw_RB_Iterations` calls `gw_Replay_TraceFlushUpTo(min(conf, frame))` AFTER computing the rollback plan
but BEFORE the resimulation runs, so it flushes rows of iterations that are about to be re-simulated —
the OLD timeline's rows (log: "rollback to frame -28 (k=4) ... confirmed -26" flushed -28..-26 first).
Fix: flush at the START of `gw_RB_Iterations`, before the deliveries, using the confirmed frame left by
the previous tick's execution (frames <= that were compared at delivery and, if wrong, already
resimulated). Then rerun `MELEE_RB_FAKE=4,2` (60-100 s) and diff with
`python tools/replay/det_diff.py <base>/trace.csv <run>/trace.csv`; expect identical through the last
confirmed frame. Then latency 0, 2, 4, 7 (also D=0 with 7), jitter, loss, whole replay.

Also noticed: many mismatch logs print identical lx/ly/buttons for used vs true (`pred=1`): the difference
is in `trigger` or the raw stick bytes. If it is the raw bytes only and the processed fields match, the
resimulation is wasted work — consider comparing only what the sim reads (for UCF that includes raw x/y).

## Remaining after the acceptance test

Rollback cost at depth 7 (measure `rb_extra_ms_*` — the per-depth stats are collected but not yet logged);
audio (see above); live pads through `gw_rb_submit_local_input` at the raw PADStatus pop
(HSD_PadRenewMasterStatus) with UCF gating for live matches (MELEE_SLIPPI_CODES); scene-end deferral so
the last frames of a match are confirmed before results; `gw_rb_frame_advantage` should use peer-reported
frames once a transport exists.
