# Rollback: savestates + SyncTest — status

Branch `agent/synctest`. Code: `pc/platform/gw_snap.c`, hooks in `src/melee/gm/gmscene.c`,
`src/sysdolphin/baselib/{axdriver,objalloc,memory}.c`, `src/melee/cm/camera.c`.
Add `gw_snap.obj` to the root `_build/melee_link_objects.rsp` when merging.

## What a snapshot is

MEM1 (24 MB at 0x80000000) + every game-owned `.data`/`.bss` symbol found by walking
`melee-pc.map` (`src_melee_*`, `src_sysdolphin_*`, `libs_dolphin_*`, `<common>` `_gw_*`) + the replay
cursor. About 26 MB per slot, ring of k+2 slots.

Measured (RustyJuicyElephant, MEM1 memcpy): save 2.2 ms, load 2.2 ms, compare 5.0 ms per frame.
The compare is a byte diff; a real rollback needs a hash (or nothing) instead.

## Usage

`MELEE_SLP=<file.slp> MELEE_SYNCTEST=<k>`: every logic frame, save S[F], load S[F-k], resimulate
F-k..F-1 and compare each resimulated frame with the first pass. `MELEE_SYNCTEST_DUMP=<prefix>`
writes both MEM1 images of the first mismatch. `MELEE_SYNCTEST_ALLOC=1` traces every pool/heap
operation (tagged frame / in-render / resim). `MELEE_SYNCTEST_SFX=1` lets sounds play during resim.

## Results (whole replay, frames -123..2498)

| k | rollbacks | checks | mismatching |
|---|-----------|--------|-------------|
| 1 | 2600 | 2600 | 0 |
| 3 | 2600 | 7800 | 0 |
| 7 | see bottom | | |

## The design lesson: a resimulated frame must render too

The render pass is part of a frame. It grows and drains HSD object pools (which call
`HSD_MemAlloc`), fills matrix caches, clears JObj dirty flags. A resimulation that skips it lands in a
different heap state; ~a dozen rounds of masking (render-written bytes, particles, pool arenas,
pool top-up at end of logic) each fixed one symptom and exposed the next. What worked: each
resimulated iteration runs the same render calls as the real one, minus `HSD_VICopyXFBAsync`
(gmscene.c). The picture is unaffected (aurora discards the extra commands with the frame; a
resimulated frame costs a render's CPU time).

`MELEE_SYNCTEST_RENDER=undo` (put the render pass's writes back instead) crashes in `ftParts_80074D7C`
after the first load: something logic reads is only correct after render has run. It is kept as an
opt-in experiment and is the list of render->logic couplings if anyone wants it
(`=report` lists them).

## Exclusions and why (evidence)

Not snapshotted at all: audio (`lbaudio_ax`, `synth`, `axdriver`, `audio`), `video` / `_gw_HSD_VIData`,
`perf`, `devcom`, card, movies (`lbmthp`, `THPDec`), `lbsnap`, `memory.c` (heap-usage diagnostics only:
`caller_hits` differed every frame), `state.c` (shadow of the HOST GX state; restoring it desyncs from
the GPU - first mismatch after render-in-resim), `_gw_HSD_PadLibData` / `_gmMain_8046B108` (raw pad
queue), `_gw_start_time` (wall clock).

Saved and restored but not compared: `psdisp.c` (particle display), the scene loop's render counter
(`_gm_80479D58+4`), pad statuses, `rumble` (see below), plus bytes measured as written between
logic frames by the render pass / deferred callbacks (`sn_mark_window`).

Rumble: `rumble.c`, `_gw_HSD_Rumble_804C22E0`, `_gmMain_8046B1F8` are reset to zero on every load.
Leaving it stale crashed in `HSD_PadRumbleInterpret1` (script pointers into freed heap); restoring it
hung k=7 in the same function (cursor rolled back into a loop). Idle is always safe.

Async world, not rolled back: devcom's `HSD_DevCom` request nodes (lifted out before a load and
written back), and a fixed range `0x80171160+0x70` (the streaming-music DVD command blocks; override with
`MELEE_SNAP_ASYNC="hexva+hexlen,..."`). It is a boot-time allocation, stable for one build/disc, but
it is an address, not a symbol.

SFX: `HSD_AudioSFXStartParam` records (sound, handle) per frame on the first pass; a resimulated
frame gets the same handles back without touching a voice.

## Remaining issues

* Compare masks are *measured* (bytes the render pass or deferred callbacks wrote between logic frames
  are skipped). That hides a real render->logic coupling whose bytes render also writes. `x221F_b0` (fighter
  off-camera flag) is one; fixed in parallel elsewhere. To audit: run with `MELEE_SYNCTEST_RENDER=report`.
* The fixed async range is address-based.
* Save/load are full 24 MB copies; a real rollback wants dirty-page tracking or a smaller MEM1
  footprint. The compare is 5 ms and would be a hash.
* Only one replay was used for the pass. The other Gang-Steals replays (different stages/characters/items)
  are not run; expect item/stage-specific state (grounds, effects) to show up there.

## Snapshot cost, dirty-page mode (agent/snapcost)

**Results, RustyJuicyElephant, WHOLE replay (-123..9522, ~9,600 rollbacks per run), dirty-page mode:**

| run | rollbacks | checks | mismatching |
|---|---|---|---|
| strict byte compare, k=1, `MELEE_SNAP_VERIFY=1` | 9600 | 9600 | 0 (verify fails 0) |
| strict, k=3 | 9600 | 28800 | 0 |
| strict, k=7 | 9600 | 67200 | 0 |
| **curated hash, no resim render**, k=1 / k=3 / k=7 | 9700 / 9600 / 9600 | 9700 / 28800 / 67200 | 0 / 0 / 0 |
| curated + poison negative control (40 s) | 1300 | 3900 | 3900 (as intended) |

(An earlier "k=7 passes" only covered the first ~2,600 frames. Over the whole replay k=7 found one more
input-derived global, `_controller_map` (gm_1A36.c, rebuilt every frame from the live pad), now skipped in
the compare like the pad statuses. Headless tests 76/76 on vanilla, Akaneia and ACE on the final tree.)

**What each piece costs, ms per call (2 fighters, Fountain; full-copy -> dirty-page):**

| | full copy | dirty-page |
|---|---|---|
| `gw_snap_save` | 2.46 | 0.22 |
| `gw_snap_load` | 2.34 | 0.30 |
| SyncTest compare (SyncTest only) | 5.15 | 0.74 |
| write-watch poll (`GetWriteWatch`+reset) | - | 0.05 (105-170 dirty pages/frame) |
| `gw_snap_hash()` (peer checksum) | 5+ (whole slot) | 0.25 (235 pages + 0.86 MB globals) |
| resimulated frame, strict: logic + render | 0.34 + 0.90 | 0.34 + 0.63 |
| resimulated frame, curated: logic only | - | **0.18-0.33** |

Worst case per tick at k=7: strict = save 0.22 + hash 0.25 + load 0.30 + 7 x ~1.0 = ~7.8 ms; curated =
0.77 + 7 x ~0.2 = ~2.2 ms, both well inside 16.7 ms for this match (4 fighters and item-heavy stages will cost
more: logic scales with fighters).

**Where the resim render time goes** (`MELEE_SNAP_CBTIME=1`): the GObj draw walk is 0.66 of the 0.75 ms.
`fn_800301D0` (camera.c:4071, the main game camera's render callback: the whole world draw, ground, fog;
0.42 ms) cannot be skipped - it also refreshes the camera and clears dirty flags. `grIzumi_801CCEA0`
(Fountain's water reflection: a second scene render from a mirrored camera into a texture; 0.23 ms) only
feeds the picture: **skipped in resimulated frames by default** (`MELEE_SNAP_SKIP_CB=none` to disable);
SyncTest k=3 passed 5,600 rollbacks with it. Suppressing display-list submission (`gw_Gx_SuppressDraws`)
saved only ~0.06 ms (aurora's FIFO is not the cost; the CPU is the callbacks' own JObj/matrix walk).

**Curated mode (`MELEE_SYNCTEST_CURATED=1`, the coordinator's experiment from yampp-comparison.md).**
Compares only a curated set - the RNG seed plus per fighter: player/kind/motion id, ground-or-air, stocks,
held buttons, position x/y/z, facing, percent, self and knockback velocity, ground velocity, hitlag, shield
health, both sticks, the motion script frame counter - and the resimulated frame runs NO render calls at all.
It passes the whole replay at k=1/3/7 (above) and cuts a resim iteration from ~1.0 to ~0.2 ms. What it does
NOT cover: items, stage state (Fountain platforms), projectiles, camera, effects, heap/pool state - all
covered by the strict mode. So: **keep strict as the default for development and CI**; use curated as the
in-game rollback path (where cost matters) only alongside the periodic incremental state hash
(`gw_snap_hash`, 0.25 ms) that catches drift the curated set misses. Caveat: this replay has 2 fighters, no
items, and a stage without moving hazards; an item- or Stadium-heavy replay is the next thing to try, and
the curated set should grow (item core fields, stage/timer) before trusting it there.

## API (pc/platform/gw_snap.c)

```c
int      gw_snap_open(int k);          /* ring of k+2 slots; idempotent; 0 on success. MELEE_SYNCTEST=<k> calls it. */
void     gw_snap_save(int frame);      /* the state at the START of `frame` into a slot (oldest slot reused) */
int      gw_snap_load(int frame);      /* restore it; -1 if that frame is not in the ring */
uint64_t gw_snap_hash(void);           /* 64-bit hash of the LIVE state; incremental (~0.25 ms) */
uint64_t gw_snap_frame_hash(int frame);/* the hash taken at that slot's save (0 in full mode / MELEE_SNAP_HASH=0) */
/* rbsession's entry points, kept compatible: */
int      gw_Snap_OpenSession(int k);   uint32_t gw_Snap_Checksum(int frame);  /* folds gw_snap_frame_hash in dirty mode */
void     gw_Snap_SessionResim(int on, int frame);   int gw_Snap_HasFrame(int frame);
```
Env: `MELEE_SNAP_MODE=full|dirty` (default dirty; full copies all of MEM1 and is the cross-check),
`MELEE_SNAP_VERIFY=1` (memcmp live vs slot after every dirty save/load), `MELEE_SNAP_HASH=0`,
`MELEE_SNAP_SKIP_CB=none`, `MELEE_SNAP_RESIM_DRAWS=1`, `MELEE_SNAP_CBTIME=1`,
`MELEE_SYNCTEST_CURATED=1` (+ `_POISON=1`, a negative control). Dirty mode needs MEM1 allocated with
`MEM_WRITE_WATCH` (gw_runtime.c; falls back to full copy if that allocation fails).

**Design of dirty mode:** each slot keeps a superset bitmap of the 4 KB pages where live MEM1 may differ from
its copy. `sn_poll` ORs `GetWriteWatch` results into every slot's set. Save copies only the chosen slot's
pages; load copies the target's pages back and folds its set into the others'. The hash keeps a hash per page
and rehashes only pages written since the last call; the disc's asynchronous state (streaming DVD block,
devcom request nodes) is hashed as zero, and pad / rumble / particle-display / controller-map globals are left
out. Globals (~0.86 MB) are copied/hashed every call.

## Remaining

* Curated set only tried on one 2-fighter, item-free replay. Grow it (items, stage, timer) and try a 4-player
  and a Stadium/item replay.
* The strict compare masks are still measured (render-written bytes between logic frames), and the fixed async
  range `0x80171160+0x70` is an address, not a symbol.
* Dirty-page tracking assumes nothing but the main thread writes MEM1 between polls except via the kernel
  (ReadFile into MEM1 is tracked by write-watch too); the audio thread reads it only. `MELEE_SNAP_VERIFY=1`
  is the check if that ever changes.
* Rollback needs the input-delay / prediction session (rbsession, merged here) and the transport (netcode).
