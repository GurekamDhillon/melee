# Background asset service: design note (draft for the owner, 2026-11-11; not built yet)

## Determinism contract (overrides everything below)
1. The service's timing is never visible to the simulation. The sim only ever sees an asset at an **apply tick** that
   game state fixes: requested at tick T, applied at T + N, N a constant of the request kind (or a tick the sim
   computed from its own state). Nothing the sim reads depends on whether a worker finished early or late.
2. If the asset is not ready at its apply tick the game thread **blocks** until it is (the gate counts that as a
   readiness miss and fails). It never slips to a later tick, never applies early, never substitutes.
3. Preloading changes no snapshot, frame hash, replay or rollback state: everything held ahead lives outside the
   snapshot ranges (system heap / GPU), and the apply reproduces exactly the allocations the synchronous load makes,
   in the same order, from the same game-thread allocator.
4. Resimulation (rollback, synctest, replay seek) never triggers a load or a prefetch: requests are issued from
   confirmed ticks only (or are idempotent hints that the sim never waits on); an apply inside a resim reuses the
   already-held prepared image.

## What the code says (the two open questions)
- **Can parse output be built elsewhere and copied in unchanged? No — but it can be built elsewhere and rebased.**
  `melee_engine/src/archive.rs::host_parse_timed` does: `DatArchive::parse(file)` (pure, any thread), then decodes
  every known public root through `gcdecode` into a `GcArena` — a list of `Box` allocations from the global allocator,
  which on the game thread inside a `GameScope` lands in the game arena (`arena.rs`: `IN_GAME && GAME_THREAD &&
  !FOREIGN`; other threads go to the system heap). The decoded graph holds absolute pointers to its own objects and
  into the raw image (`set_raw_base(src)`: the C heap block the file was read into, address known only at apply), and
  the raw image is relocated in place with `(u32)data` (`Locate`). Then `fix_articles` / `fix_dynamics` touch statics.
  So a worker can produce: the parsed `DatArchive`, and a **relocatable decode plan** = the allocation sequence (size,
  align, bytes) in the exact order the game thread would make it + a fixup list (pointer slot -> object index+offset
  or raw-image offset). Apply on the game thread = same allocations in the same order (arena identical to a sync
  load), memcpy each object, patch fixups, copy the file bytes into the C heap block and run `Locate`, then the two
  fixups. That is linear memcpy work (~1 ms per MB order), replacing the 10-48 ms parse.
- **What NHV1 covers:** gameplay values only — seed, per fighter 12 words (motion id, positions, velocities, percent,
  facing, ground/air, kb vel, hitlag), item count and sum; "no pointers, no addresses, no raw memory"
  (`docs/rust-port/neutral-hash.md`). Snapshots/rollback, though, restore raw bytes of arena + OS-heap block +
  statics, so within a run addresses must also be reproduced — hence rule 3 (same allocation sequence), not just
  same values.

## Fixed retail heaps (the heap-1 crash: 386 KB free for a 2 MB animation bank)
- The C heaps (lbHeap 0/1, stage slot) are inside the snapshot; they cannot be pre-filled. The service keeps the
  bytes outside and does the heap allocation at the apply tick, like the sync path.
- Admission is decided **at request time from game state**: `lbHeap_LargestFree` (added by d90f99c2b) plus the
  service's ledger of what is already promised to that heap for pending applies. If it does not fit, the request is
  refused at T (deterministically; the sim takes its fallback then, e.g. the foe returns as itself), never at apply.
- `reserve(heap, bytes, until_tick)` books room for a known future apply so other loads cannot take it.
- Eviction of a retail-heap resident (an old fighter's animation bank) is a sim-visible free, so it happens at a
  sim-fixed tick too (the wave swap), through `unload`.

## API
Engine side (Rust), game thread unless noted:
- `prefetch(asset, priority, needed_by_tick) -> Handle` — queue read + parse/decode-plan on the worker.
- `ready(h) -> bool` (diagnostic / UI only; the sim must not branch on it).
- `acquire(h, apply_tick)` — at the apply tick: blocks if not ready (readiness miss), then performs the apply
  (allocations, copy, fixups) and returns what the sync load returns. `release(h)`.
- `unload(asset)` / evict under a memory budget (system-heap bytes held; LRU by priority; never evict an asset with
  a pending apply).
- `prewarm(gpu objects)` — pipelines/textures on the render side; never sim-visible.
- `reserve(heap|arena, bytes, until_tick)`; `cancel(h)`.
- Worker: one thread (marked foreign for the arena), a priority queue keyed by (needed_by_tick, priority); disc
  reads go through it too. Small finalise on the game thread is time-sliced only when it is not sim-visible (GPU),
  else done whole at the apply tick.
Assets: fighter data / costume / animation banks (PlXx, PlXxNr, PlXxAJ), stage archives (GrXx), commons (Ef*, ItCo,
LbRb...), menu/scene archives (GmTtAll, MnSlChr, MnMaAll...), audio banks, mod art and drive models, GPU objects.

Wasm host API (Envoy declares intent early): `gd_asset_intent(kind, id, apply_tick)` for pair, foes, wave-2
characters, candidate stages, boss, next room; `gd_asset_ready(kind, id)` for UI only; requests made from sim hooks
at confirmed ticks are part of sim state (the apply tick is), so rollback re-issues nothing. Engine-side hints at
retail transitions: CSS hover/lock -> fighter sets; SSS hover -> stage; title -> menu archives; results -> CSS.

## Profiler
Queue length, per-asset read/parse/apply ms, readiness misses (gate fails on any), bytes held, heap ledger, evictions,
per-frame apply ms. All in the frameprof jsonl.

## Hand-over from port/v3g ("free the old character's animation bank on a wave swap")
It should hand over: the call site and tick where the old bank is freed, the condition it uses (bank resident /
`lbHeap_LargestFree(1) >= 3 MB`), and `ftData_Table_Unk0[k].data` as the residency test. The service later owns
that free as `unload(bank)` at the swap tick and turns the 3 MB check into an admission check against its heap
ledger, with the new bank prefetched at wave-plan time.

## Treatments per offender (filled from the baseline list)
- already-loaded reuse: commons re-read per match (EfCoData, ItCo, LbRb, Kb sets).
- parse ahead on a worker + small apply: fighter sets, stage archives, menu archives (CSS MnSlChr 3.7 MB, MnMaAll 2 MB).
- time-sliced on the game thread (~1-2 ms/frame): GPU pipelines/texture uploads (menu 75 ms of pipelines).
- reserve up front: retail-heap room for wave respawns and stage switches.
