# Frame-time tail in real combat — 2026-09-21

**Result: the 4-CPU fighting tail is gone.** Vanilla Battlefield, fighting AI: p99 34–47 ms → **17.3 ms**,
max 58–61 → **19 ms**. Akaneia (three interpreted m-ex fighters): p99 63–78 → **18–21 ms**, max 79–111 → **31 ms**.

## The problem

`_research/perf-baseline-2026-09-21.md` (and its aurora fix, `78aceaf1a`) was measured with
scene-launched CPUs that **stood still** (`cpu_kind` 0, fixed in `32f87b5ec`). With the fighting AI
(`cpu_kind` 4) the same scene had p50 16.6 but p95 ~30, p99 34–48, max 58–79 (14% of frames over
20 ms). With `/kind0` (standing still): p95 17.0, p99 18–19.

## Method (all env-gated, off by default; `pc/platform/shim_vi.c`, `shim_pad.c`)

* `MELEE_PROFILE_SPIKE=<ms>` (with `MELEE_PROFILE_SAMPLE=<s>`): the sampler records the frame
  thread's samples with a timestamp, and a frame whose game time exceeds `<ms>` has its window
  copied into `melee-pc.spike.samples` (same format as `melee-pc.samples`, so
  `tools/port/prof_report.py` reads it) plus `melee-pc.spike.chains` (return-address chains found by
  scanning the stack, validated by a preceding call instruction). An ordinary sampled profile is an
  average over all frames and hides a tail that is 5–14% of them.
* `MELEE_PROFILE_FRAMES=<file>`: one CSV row per presented frame: split, GXInitTexObj count, GX
  prims/dlists, aurora queued/created/urgent pipelines, draw calls, buffer sizes, `gw_PADRead`
  segment times, and `gw_wait_idle` calls.

## Findings

Spike frames (>20 ms, 14% of frames; per-frame means, normal → spike):

| column | normal | spike |
|---|---|---|
| total ms | 15.8 | 31.7 |
| **game ms** (game thread, excl. present) | **2.2** | **21.4** |
| present ms | 13.5 | 10.3 |
| GXInitTexObj / frame | 310 | 311 |
| prims / draw calls | 121k / 583 | 124k / 583 |
| pipelines queued / urgent | 4435 / 3.2 | 4432 / 3.2 |
| texture upload KB | 1.2 | 0.9 |
| `gw_PADRead` ms (aurora, adapter, rest) | 0.00 / 0.00 / 0.02 | 0.00 / 0.00 / 0.02 |
| **`gw_wait_idle` calls / frame** | **0.39 M** | **9.9 M** (worst frames 20–45 M) |

Everything the earlier baseline suspected is **innocent** in these frames: texture inits, prims,
draw calls, pipeline compiles, texture uploads, the aurora FIFO thread, and `PADRead` (0.01 ms)
are all equal between normal and spike frames. The slow frames are the **game thread spinning in
`gw_wait_idle`** for 20–90 ms.

Spike-frame call chains (return-address chains, top of the profile):

```
22.7%  wait_idle <- ARQPostRequest <- lbArq_80014BD0 <- lbArq_80014BD0 <- ftData_80085CD8
11.1%  wait_idle <- lbArq_80014BD0 <- lbArq_80014BD0 <- ftData_80085CD8 <- Fighter_ChangeMotionState
 7.7%  lbDvd_GetPreloadedArchive <- lbDvd_8001819C <- lbFile_800168A0 <- ftData_80085A14 <- ftData_80085B10
 ... (ftData_80085CD8 <- Fighter_ChangeMotionState in nearly every top chain)
```

## Root cause

Every fighter **action-state change** (`Fighter_ChangeMotionState` → `ftData_80085CD8`) loads that
animation from ARAM through `lbArq_80014BD0`, whose callback-less path is a synchronous wait:

```c
while (lbArq_80014ABC(rp) != LB_ARQ_STATE_DONE) { wait_idle(); }
```

The ARQ shim (`shim_ar.c`) does the transfer inline and only *defers* the completion callback
(`gw_defer`) so it runs off the starter's stack. Nothing but `gw_wait_idle` can therefore set
`DONE`, and `gw_wait_idle` ran the deferred queue **only when `GetTickCount64()` had changed since
the last call**. So every load waited for a clock tick: ~1 ms at best, up to a whole 15.6 ms system
tick when the timer resolution is coarse. In a fight, several fighters change motion state per
frame and the waits stack into 20–90 ms spins (~10 M `wait_idle` calls); a fighter standing still
almost never changes state, which is why the earlier baseline never saw it.

## Fix (`200c54f31`)

`gw_wait_idle` runs the deferred queue immediately when it is non-empty (a queued completion is
already due); the clock gate stays for `gw_os_run_alarms`, which is genuinely time-driven.

## Numbers (`MELEE_PROFILE=1`, 600-frame windows, three runs each, alternating before/after)

Vanilla, `mode=vs;p1=fox/cpu9;p2=falco/cpu9;p3=marth/cpu9;p4=ganondorf/cpu9;time=180;stage=battlefield`:

| run | p50 | p95 | p99 | max |
|---|---|---|---|---|
| before 1 | 16.65 | 32.48 | 47.51 | 61.48 |
| before 2 | 16.65 | 31.48 | 44.13 | 58.59 |
| before 3 | 16.66 | 28.01 | 34.54 | 58.71 |
| **after 1** | 16.67 | **16.89** | **17.41** | **19.43** |
| **after 2** | 16.66 | **17.05** | **17.31** | **19.00** |
| **after 3** | 16.67 | **17.06** | **17.35** | **18.78** |

Akaneia, `mode=vs;p1=ck:38/cpu9;p2=ck:39/cpu9;p3=ck:36/cpu9;p4=fox/cpu9;time=180;stage=battlefield`:

| run | p50 | p95 | p99 | max |
|---|---|---|---|---|
| before 1 | 16.64 | 45.22 | 63.40 | 110.93 |
| before 2 | 16.47 | 45.09 | 63.46 | 78.85 |
| before 3 | 16.63 | 39.25 | 77.68 | 79.40 |
| **after 1** | 16.67 | **17.25** | **17.83** | **30.67** |
| **after 2** | 16.67 | **19.38** | **21.01** | **31.12** |
| **after 3** | 16.67 | **18.35** | **20.28** | **31.27** |

## Side effects checked

* Headless `--test`: 76/76, `FATAL 0` on vanilla, Akaneia and ACE.
* `.slp` playback (`RustyJuicyElephant.slp`, 3.19 online): unchanged — no percent/stock divergence,
  first difference is still the one-ULP `x` at frame 4056, and the whole replay (9522 frames) plays
  through.
* Completions now run at the wait itself instead of at a wall-clock tick, which is also *more*
  deterministic (nothing about their timing depends on the clock any more).

## What remains

* Akaneia still has a small tail (p99 up to 21 ms, max ~31 ms): a handful of frames per run. Not
  yet attributed; run `MELEE_PROFILE_SPIKE=19` with the Akaneia scene to profile them. Candidates:
  the m-ex interpreter on the frame that a fighter's code first runs, first-sight pipeline
  compiles, or the remaining `lbDvd_GetPreloadedArchive` path (7.7% of spike samples before the
  fix: `lbFile_800168A0` resolves a path through `DVDConvertPathToEntrynum`, a linear FST scan plus
  the mod lookup, ~4 times a frame — `IfAll.usd`).
* The earlier baseline's open items are unchanged: `texture::sweep_object_caches` (~2 ms/frame on
  the game thread), and the seed warm-up's compiler thread holding a core for minutes.
* The `GetTickCount64` gate is still used for alarms; if a load-bearing alarm wait exists anywhere
  (e.g. lbMemory's chunked-copy alarm 3 ms out) it would still be paced by the clock resolution.
