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


## 2026-10-05 (engine-3): why the bench SyncTest mismatches in sustained combat

Setup: `MELEE_SYNCTEST=12 MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1`, Fox v Fox, Final Destination, two `MELEE_PAD_BOT`
programs, `MELEE_TEST_SEED=12345`, turbo (render pass on or with `MELEE_TURBO_RENDER=0`: no difference). The first
mismatch lands anywhere from frame 1.4k to 5.9k and, once it starts, most later checks mismatch. New tool:
`MELEE_SYNCTEST_CURATED_DIFF=1` compares every fighter's whole struct, its GObj and its joint tree against the first pass at the
start of every resimulated frame and logs the first frame each word differs (`snap: REGION-DIFF new word: frame F tag T
+0xOFF`; tags 1-12 fighter, 100+ GObj, 1000+player*256+n joint n). What it shows, in the order the differences appear:

1. Frame ~35 (match start). `fp+0x20A4` (LbShadow flag byte) and `fp+0x2224` bit 0; every joint's `flags` word (`+0x14`): the
   rendered first pass has the joints' world matrices set up (flag `JOBJ_MTX_DIRTY` 0x40 clear, `mtx` at `+0x44` holds the
   matrix), the logic-only resimulation has them dirty with an identity cache until logic asks for one joint
   (`lb_8000B1CC` -> `HSD_JObjSetupMatrix`). Render-owned state, harmless by itself.
2. Frame ~104: `fp+0x2144/0x214C/0x2160` AXDriver voice ids (a resimulation does not play sound).
3. Frame ~161, then every few hundred frames: ULP-LEVEL float differences in the hurtbox capsules (`fp+0x11A0..`, a_pos/b_pos), the
   hitboxes' `hurt_coll_pos` (`fp+0x978`, `+0xAB0`) and `dmg.x1854_collpos`. First pass values are identical in every run
   (also identical with render off), resimulation values are identical in every run: deterministic, but the resimulation's
   lazily computed world matrices are not bit-equal to the ones the render pass computed. `MELEE_SYNCTEST_CURATED_SETUP=1`
   (`Fighter_BenchSetupMatrices`, run at the top of each curated iteration) removes classes 1 and 3 from the log.
4. Heap ADDRESS differences from frame ~800 on: the accessory JObj (`fp+0x20A0`), `jobj->aobj` of many joints, a hitbox's victim
   list entry (`fp+0x988`), `dmg.x1868_source` (the damage-source GObj), `fp+0x1974`. The same logical object sits at a different
   pool slot in the resimulation: the first pass's render pass allocates and frees from the same HSD pools (the matrix pass
   allocates `HSD_VecAlloc` scale vectors) and the resimulation does not, so the free lists are threaded differently from the
   first resimulated allocation on. This is the "allocator free-list threading" of the uncurated mode's particle family.
5. The first BEHAVIOURAL difference follows those by hundreds to thousands of frames: a fighter's velocity, hitlag, motion or an
   animation counter (`fp+0x1ABC`, `+0x1B04`), then everything. Not isolated to one field.

Note: runs with `MELEE_TURBO_RENDER=0` show the same first-pass values as rendered runs, so the pass that sets the matrices up is the first pass's draw phase (GObj draw callbacks), which that mode still executes; "render pass" below means that phase, not GX submission.

Verdict. The curated resimulation is not equivalent to the first pass because it skips the render pass, and the game's own
logic reads state the render pass writes (the world-matrix cache, with ulp-level different results when it is filled lazily; the
allocator pools the draw callbacks churn). Netplay's rollback resimulates `logic + render` per iteration (`gw_rollback.c`:
"resim iters (logic+render inside the loop)"), so both peers run the same sequence, which is why two real clients ran an hour
without a desync. Not proven: the one remaining class (heap addresses) as the cause of the first behavioural difference; with the
matrix setup on, a clean long run was not demonstrated. The fix that would settle it: make the bench's resimulation render, as
rollback does, and accept the particle-state mask; the SyncTest then becomes the strict mode with its known particle family.
Risk this puts on netplay: a peer that does NOT run the render pass (headless, a render-throttled window, `MELEE_TURBO_RENDER=0`)
is not equivalent to one that does, in the sim's low-order float bits and in allocation order; nothing prevents it today.

## The rollback desync checksum (RB_GameHash), widened 2026-10-05

`RB_GameHash` (`src/melee/ft/fighter.c`) is what two netplay peers compare every frame (`gw_rollback.c` hashes at the start of each
iteration; `gw_net.c` exchanges and compares). It used to cover the RNG seed and, per fighter, motion, position, velocity x/y, percent,
facing, the Turbo window word and the Geno define word. It now also covers, per fighter (`RB_FighterHash(h, fp, slot)`): `ground_or_air`,
knockback velocity x/y, ground velocity, hitlag frames (`dmg.x195c`), the hitlag flag `x2219_b5`, shield health, the action/subaction
frame counter (`x3E4_fighterCmdScript.frame_count`), `x1968_jumpsUsed`, the L/R timers `x67F` and `x680`, and stocks; and per match one
item word (`RB_ItemHash`: count, then the SUM over the item plink list of a hash of kind, state, position and velocity, so list order cannot
raise a false desync). Floats go in by bit pattern. Nothing the draw phase writes (joint matrix caches and flags, shadow flags, AX voice ids,
capsule positions) and no pointer is hashed; `RB_GameHashTest` (`rb_game_hash`, native suite) pins that.

- **Legacy mode**: `MELEE_RB_HASH_LEGACY=1` (set on BOTH peers) hashes exactly the old fields. Test and negative control only: the mode is not
  in the handshake, so peers in different modes report a desync at frame -123.
- **Negative control**: `MELEE_RB_PERTURB=<field>` on ONE peer perturbs one hashed value at the start of frame `MELEE_RB_PERTURB_FRAME` (default 600),
  live and on every resimulation of that frame: `hitlag jumps shield x680 x67f kbvel groundvel cmdframe b5 stocks item pos` (`pos` is a legacy
  field). The widened hash reports `netplay: DESYNC at frame N` at once; legacy mode does not for the added fields.
- **Verifying a field before it is hashed**: `MELEE_SYNCTEST_CURATED=1` records the same words per frame (fighter records now 29 words, plus one record
  per item and an item-count record, `Snap_CuratedItems`), and `snap: curated word changes` in the log says how often each word changed in the first
  pass, so a word that never moved is visible as unexercised. Items need a VS scene with `items=<n>` (LAB-style `items=4`).
- **Cost**: the log line `rb: tick ... hash X/call` is the mean time inside `gw_RB_GameHash` (about 2 microseconds for two fighters).
- **Startup fix found on the way** (`gw_rollback.c`, `rb_early`): remote inputs delivered before the session OPENS (the first logic tick of the match)
  were wiped by the open while the transport counted them delivered, a permanent hole; with `MELEE_NET_SIM` set (even `lag=0`) both peers stalled
  at frame about -115 for ever, on the unmodified base too. They are now buffered and submitted right after the open.

### Evidence for the widening (2026-10-05, worktree checksum, build root _build/agents/checksum; logs and scripts in _build/audit-20261003/checksum/)

- **Bench SyncTest** (resimulates with the render block, curated record + item records, Turbo on and off, `items=4`): 110,400 compared / 0
  mismatching (Turbo word 1) and 110,400 / 0 (Turbo word 0, seed 777), items on the list up to 22-23 at once; plus 2 x 43,200 / 0 and 2 x 36,000 / 0
  with the same record. Every word of the record in the hash was bit-equal. **Exercised**: ground_or_air, knockback velocity, ground velocity, hitlag,
  `x2219_b5`, `jumpsUsed`, the action frame counter and the item words moved thousands of times (`snap: curated word changes`). **Not exercised by the
  bench**: shield health, `x67F`, `x680` and stocks stayed constant (60.0, 255, 255, 99) in every bench run (the pad bot never shields or presses L/R, no
  KO happens); they never differed, and an attempt to force L/R/shield presses through the bench input hook did not reach them (the pad pass's
  `HSD_PAD_LR` edge is not produced there), so they are in the hash on code review (plain counters/floats advanced from the synchronised input) and
  flagged as unexercised.
- **Netplay soak** (two real clients over loopback, `MELEE_PAD_BOT` both sides, Fox v Fox, FD, 600 s, about 40,000 frames each): clean and
  lag 50 / jitter 20 / loss 3 %, delay 2 and 0, Turbo on and off: 0 desyncs in all 8 configurations on both peers (rollbacks per run up to
  4,064, max depth 7).
- **Negative control** (`MELEE_RB_PERTURB` on the host only, frame 600, clean network, delay 2, Turbo on): the widened hash reported
  `DESYNC at frame 600` on both peers for `jumps`, `x680`, `hitlag` and `pos`; with `MELEE_RB_HASH_LEGACY=1` on both peers the added fields
  (`jumps`, `x680`, `hitlag`) were NOT reported (the match ran on to frame 6,200-7,000 with the peers silently different), while `pos`, a
  legacy field, was reported at frame 600 in both modes.
- **Cost**: `hash 0.0017-0.0020 ms/call` legacy, `0.0015-0.0024 ms/call` widened (timer noise dominates), about 0.01 % of a frame.
