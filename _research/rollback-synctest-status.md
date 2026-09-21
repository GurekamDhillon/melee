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
