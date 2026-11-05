# Rust sim performance (2026-11-01, branch port/perf)

Task: notes/bench.md showed the engine tick at 5.6-10.5 ms (C: 1.7-2.2 ms) and a 7-frame rollback at 34.5 ms. Everything
below is bit-identical: the 4 goldens (1801 / 1801 / 3601 / 3601, seed included), `gate.sh` (GATE OK) and
`melee synctest --depth 7` (0 mismatches, 4158 hash checks, both pad_falcon_fox_fd and cpu_fox_falco_dl) were re-run on
the final binary. No float code was touched (FMA sites unchanged).

## Profiler
No admin rights, so no ETW/WPR (`wpr` needs elevation) and no samply. `rust/melee/src/prof.rs` is an opt-in in-process
sampler: `MELEE_SAMPLE=<report.txt>` starts a helper thread that suspends the main thread every ~1 ms, unwinds it with the
PE unwind tables and counts frames; the report (self time + inclusive) is written at exit. Symbols need the PDB:
`MELEE_SAMPLE_PDB=<dir with melee.pdb>` (the release profile keeps `debug = 1`). `MELEE_SAMPLE_ROOT=<substring>` keeps only
samples whose stack contains the substring (e.g. `Engine::tick` to drop the boot). Windows only, no OS input.
`melee synctest` also prints per-tick ms for save / load / state_hash / game_hash / tick (the counters live in a leaked
heap block: a load restores every static, so a static counter array would be rolled back too).

## What the profile showed (before)
1. `melee golden` rewrote the whole growing CSV every frame (`std::fs::write` of the cumulative string, create + truncate):
   47-63% of all samples (O(n^2) bytes plus Windows file-open cost). That is harness cost, not sim cost, and it sat inside the
   "sim only" figures of bench.md.
2. Snapshot refresh: `GetWriteWatch` over the 4.8 GB arena (44% of a depth-7 synctest) on every save, load and hash
   (9 hashes per tick at depth 7), plus a 512 KB zeroed buffer per call.
3. `jobj::snapshot` (the native JObj matrix/IK bridge copies the whole connected joint tree into a `NativeTree` for every
   `HSD_JObjSetupMatrixSub` the render pass makes): linear `contains`/`position` scans over the joint list, i.e. quadratic
   in the tree size, 41% of a tick on cpu_fox_falco_dl; arena alloc/free another 16%.
4. Page hash: one serial multiply chain per 8 bytes (latency bound).

## Changes (all game-result neutral)
| change | where |
|---|---|
| golden CSV appended through one open handle (crash-safe as before), final full write kept | `melee/src/main.rs` |
| `snapshot()` joint index: scan for <=24 joints, stack open-addressing table up to 256, HashMap beyond; `NativeTree` pre-sized; same visit order so ids are unchanged | `melee_runtime/src/tu/jobj.rs` |
| `fast_matrix`: a joint with no robj, no aobj target, no IK / user matrix and a clean parent is set up through a two-node tree; only flags, `scl`, `mtx` of the joint are written back. `native_setup` returns before snapshotting when the joint is clean | same |
| dirty tracking: the arena part that existed at capture is copy-on-first-write, so its "ever written" bits already exist in the cow store (`cow::or_written`); `GetWriteWatch` only covers arena growth and the 64 MB host block. Dirty sets are bitmaps, load merges two sorted lists (no BTreeSet), stack buffer for the watch | `melee_engine/src/{snapshot,cow}.rs` |
| `page_hash`: 4 independent multiply lanes (the hash VALUE changed; it is only compared within one run, never against a golden) | `snapshot.rs` |
| sampler, per-tick synctest timers | `prof.rs`, `netmatch.rs` |
| `cargo build -p melee --profile perf`: fat LTO + codegen-units=1 (release is unchanged) | `rust/Cargo.toml` |

Not adopted: `panic = "abort"` (the crash handler and tests use unwinding; gain not measured), fat LTO in the default
release profile (build 9 min 18 s vs about 4 min; tick gain about 25%, see below).

## Numbers (medians of 3, Intel Core Ultra 7 265F, machine not idle: a game launcher was running, so absolute values wobble by up to 20%)
Before = `rust-port` at d373bba89 (release). After = port/perf HEAD (release profile).

| measurement | before | after |
|---|---|---|
| sim tick, pad_falcon_fox_fd (golden wall minus `--frames 2` boot, per tick) | 7.05 ms | 0.20 ms |
| sim tick, cpu_fox_falco_dl | 9.16 ms | 0.53 ms |
| golden run end to end, pad / cpu | 18.5 s / 37.0 s | 5.4 s / 7.0 s (5 s of that was boot then; boot is about 0.2 s now, see the 2026-11-05 section) |
| synctest depth 7, whole tick (live frame + save + load + 7 resim + 9 hashes), pad / cpu | 65.5 / 71.5 ms | 7.4 / 9.5 ms (no longer true by 2026-11-05: 75 ms, see below) |
| synctest depth 1, pad | 28.2 ms | 3.9 ms |
| save (per call) | 5.4 ms | 0.97 ms |
| load (per call) | 5.5 ms | 1.03 ms |
| state hash (per call; 9 per tick at depth 7) | 4.6 ms | 0.17 ms |
| engine tick inside synctest (8 per iteration at depth 7, pad) | 12.5 ms (1.6 each) | 3.8 ms (0.47 each) |

The golden-minus-boot tick figures carry about +-0.2 ms of boot noise (boot is 4.6-6.1 ms of wall in the runs); the in-synctest
tick row is steadier but includes the neutral-row work and the resim bookkeeping. Rollback cost of a 7-frame resim now fits
a 16.7 ms frame budget by itself (7 x 0.47 = 3.3 ms of ticks).

Fat LTO + one codegen unit (`--profile perf`, separate target dir): all goldens identical, synctest total per tick 10.4 -> 7.6 ms
(pad depth 7) and tick 0.64 -> 0.48 ms (pad) / 0.76 -> 0.65 ms (cpu) measured on the counters-before-fix build, so only the
`total` and tick rows are trustworthy; about 25% on the tick for 2.3x the build time.

## Remaining hot spots
- Snapshot save/load: 2019 of the 2300 candidate pages per snapshot are the executable's `.data/.bss` (8 MB, no write
  tracking possible with GetWriteWatch), compared against the base copy on every save and load (about half of each ~1 ms).
  Next step: make the static sections copy-on-first-write like the arena (a VEH handler whose own state is off the protected pages).
  Also: share unchanged page boxes between consecutive snapshots (Rc), and a per-page hash cache for the host block.
- Tick: `jobj::snapshot` is still about a third of a cpu tick (joints with robj / IK / aobj targets, and `robj_update_all`
  re-snapshots); arena alloc/free about 6%, `Mtx` concat / inverse-transpose in the draw callbacks (the engine must still run
  the retail draw callbacks headless: they set up joint matrices the sim reads, notes/consolidation.md round 1).
  A restricted snapshot (ancestors plus robj targets, with flush leaving child/next alone) would remove most of it.
- Boot (about 5 s per run at the time; about 0.2 s by 2026-11-05, not part of a tick): `GcDecoder::issue` builds a `String` per non-relocated word (format, `rsplit`),
  `memchr`, `HashMap<u32,()>::contains_key` (SipHash) in `GcDecoder::target`. Making `issue` lazy/capped and using a fast hasher
  would cut most of it.
- `EngineGame::run_frame` formats the 16-word row into a String per frame and keeps it in a BTreeMap forever (synctest/netplay).
- Spin lock + class computation in `arena::alloc/free` (CAS per call).

## Method
Tick: `melee golden --engine tu --frames N` wall minus the same command with `--frames 2`, divided by N-2 (N = 1801 pad, 3601 cpu),
`MELEE_WINDOW_UNFOCUSED=1`, no window, no render. Rollback: `melee synctest --scenario S --depth K --frames 600`, the per-tick
counters it prints (boot excluded). Scripts are scratch only (not committed). Before binaries are rebuilt from d373bba89 with
only the timing counters added, so the snapshot/jobj code measured is the original.

# Rollback re-measure (2026-11-05, branch port/hero-fix)
Hero measured depth-7 synctest at 110-138 ms per iteration (74 % in snapshot hashing) on a busy PC, against the 7.4 / 9.5 ms
above. Re-measured here on `rust-port` fc427c98b, profile `fast` (inherits release: opt-level 3, no LTO, 256 codegen units;
release was not built for lack of disk), `melee synctest --scenario S --depth 7 --frames 600`, the per-tick counters it prints.
The machine was NOT quiet (other agents' builds and games, CPU load 83-93 %), so absolute values are high; one run each.

| per tick, depth 7 | before (fc427c98b) pad | after pad | after cpu |
|---|---|---|---|
| total | 75.3 ms | 20.0 ms | 20.9 ms |
| state_hash (9 calls) | 55.9 ms | 1.6 ms | 1.7 ms |
| save | 7.4 ms | 7.7 ms | 7.3 ms |
| load | 7.2 ms | 7.5 ms | 7.2 ms |
| run_frame (8 ticks) | 4.7 ms | 3.2 ms | 4.5 ms |

The regression was real, not only the busy PC: the host block's ever-written page set had grown to most of the block, and
`page_hash` compared every one of those pages against the base and rehashed it on each of the 9 calls per tick. Fix
(`melee_engine/src/snapshot.rs`): a per-page digest cache for the hashed range. The write-watch delta of each `refresh` is ORed
into a `stale` bitmap; `page_hash` recomputes the digest (or the equal-to-base mark) only for stale or never-seen pages and folds
the cached digests in page order. A load's restores are writes, so they mark pages stale too. The hash value changed (it is only
compared within one run: rollback snapshot checks and synctest, never across peers or builds). `MELEE_HASH_VERIFY=1` recomputes
every page on every call and panics on a stale cached digest: cpu_fox_falco_dl depth 7, 600 frames, no panic, 0 mismatches.
Both scenarios: 594 rollback passes, 4158 hash checks, 0 mismatches.

Since fixed (see "Rollback snapshots" below). Still open then: save and load were about 7 ms each, against about 1 ms in the table above. They compare every ever-written host
page against the base (and the snapshot's own pages) for the same reason; a since-last-save page reuse for the rollback
ring (like the Lab's `save_after`) would remove most of it. Not done here.

Boot: `melee golden --engine tu --scenario pad_falcon_fox_fd --frames 2` wall is 185-197 ms (3 runs, same busy machine), the
full 1801-frame golden 0.6 s; the "5 s boot" figures above are from before the boot work.

# Rollback snapshots (2026-11-05, branches port/snapfix-1, port/snapfix)
`melee synctest --engine tu --scenario S --depth 7 --frames 600`, release, the per-tick counters it prints (1 save, 1 load,
9 `state_hash`, 8 ticks per tick). The machine was busy (other agents' builds and games); CPU load is given per row.

| per tick, depth 7, cpu_fox_falco_dl | save | load | state_hash | run_frame | total | CPU load |
|---|---|---|---|---|---|---|
| 7bf535a19 (before) | 6.86 ms | 6.87 ms | 1.52 ms | 4.08 ms | 19.4 ms | 99 % |
| d23f7fc5b (since set cleared at capture) | 1.19 ms | 1.31 ms | 1.21 ms | 3.83 ms | 7.6 ms | 97 % |
| port/snapfix (epochs, watched statics) | 0.29 ms | 0.44 ms | 1.20 ms | 3.58 ms | 5.6 ms | 90 % |
| same, pad_falcon_fox_fd | 0.24 ms | 0.35 ms | 1.05 ms | 2.33 ms | 4.0 ms | 99 % |

1. The 7 ms regression (f82738ca3, the Lab's incremental ring) was a set left full: `Base::capture` cleared the cumulative
   dirty set but not the new since-last one, so every later refresh ORed the whole host block (16384 pages, everything written
   since process start) back in and every save and load compared all 64 MB. The page pool was not involved.
   `MELEE_SNAP_STATS=1` prints the page counters (host-block candidates per save 16384 -> 324).
2. Then every page carries the epoch it was last written in (an epoch ends at each save and load). A load restores only the
   pages written after its snapshot; a plain save shares the previous snapshot's unwritten pages and copies the written ones
   (about 450 pages looked at, 430 copied per save; 450 looked at, 290 copied per load). The writes come from the OS write
   watch (host block, arena) and, for the exe's 9.3 MB of statics that the watch cannot see, `melee_engine/src/pagewatch.rs`:
   read-only pages and a vectored handler that records the first write; a faulted page stays writable for 32 epochs.
   Trap found on the way: the base copies must be taken after tracking starts, or a page written in between differs from
   its base without being dirty (it showed as an arena free-list corruption and a hang).
3. `page_hash` (host block only) refreshes just the host watch, returns the previous value when nothing was written, and
   digests with an xxh3-style 8-lane accumulate against cached base-page digests. About 230 written pages are digested per
   call; that and the 64 MB `GetWriteWatch` (about 50 us a call) are what is left of `state_hash`.

Checks: `MELEE_SNAP_CHECK=N` compares the live state after every Nth save and load with the snapshot, page by page (host block
and statics in full, every ever-written arena page): pad and cpu, 600 frames, 1194 checks, clean; `lab-check` with it, ALL OK.
`MELEE_HASH_VERIFY=1` clean, 0 mismatches over 4158 checks in both scenarios. `MELEE_SNAP_NOINC=1` (every dirty page
compared) and `MELEE_SNAP_STATIC_SCAN=1` (statics compared in full) are the fallbacks. The Lab ring save went from
1314-1427 us to 333 us each (`lab-check` lab perf line).

## Snapshot perf guard
`tools/rust-port/perf/snapguard.sh` (run by `gate.sh` for the release profile; `SNAPGUARD=0` skips it) runs the cpu_fox_falco_dl
depth-7 synctest 3 times (`--runs N`), takes the minimum of save, load and total ms per tick, and fails when one is more than
25 % (and more than 0.05 ms) over `tools/rust-port/perf/snap_baseline.tsv`. The minimum of N runs is what keeps it usable
while other builds run; on a quiet machine the numbers are lower, never higher. `--update` rewrites the baseline (min of
`--runs`): do it only when a change is meant to move these numbers, and say so in the commit. Every run has
`synctest --timeout 300`, so a hang fails the guard instead of leaving a process behind.

# Play polish (2026-11-04, branch port/play-perf)
Medians of 3, Temple 4 CPUs (fox/marth/falco/peach cpu9, 3603 frames, vsync off, interp on, scale 2, the bench-live
method), visible unfocused window; load times headless. Before = rust-port 66d48cc1b, release.

| measurement | before | after |
|---|---|---|
| archive parse in the VS load (`MELEE_BOOT_TIMES`) | 47.2 ms | 35.1 ms |
| engine boot incl. the VS load | 92 ms | 78 ms |
| load tick (max tick in the bench) | 98-108 ms | 85-91 ms |
| frame interval p99, match frames (ft.csv) | 3.76 ms | 3.05 ms |
| frame p99 / 1% low (MELEE_SHOW_FPS line) | 3.78 ms / 228 fps | 3.32 ms / 252 fps |
| Stadium shots at ages 400-1000, 3 runs | 3 different | identical (= MELEE_INTERP=0) |

- Frame-time tail: `MELEE_PRESENT_SPIKE=<ms>` logs each slow present with prepare / draw / swap / adopt. Every fresh-frame
  present spent 1.0-1.5 ms dropping the retired frame (~2500 DrawCmds); `Presenter::retire` now frees it on a helper
  thread when the draw list is not arena memory (`offthread_drop`; the async GX decode thread's frames are system heap).
  What is left of the tail is one fresh frame's prepare + draw + swap (~3 ms), plus occasional driver stalls in draw/swap.
- Archive parse: `MELEE_ARCHIVE_LOG` prints per-file phases. itPublicData (ItCo, every VS load) was 29 of 49 ms: 93 PObj
  `verts` lists with no GX_VA_NULL for thousands of records (records the item tables reach with another type) and the
  garbage-count TLUT tables. Long until-word lists now use the shared per-phase table, repeated terminator scans are
  remembered, shared tables come from `arena::alloc_zeroed` (no memset for fresh bump memory while the arena was never
  snapshotted), and the decoder tests relocations in a bitmap. Decoded values are unchanged (gate, 212 coverage pass).
  Not done: moving the parse off the game thread (its allocations are arena state the sim's later allocations depend on).
- Stadium variance: re-posed in-between presents re-ran the frame's EFB copies at a wall-clock phase; the screen samples the
  previous frame's copy, so a shot depended on timing. A screenshot present now first redraws the previous sim frame
  exactly (`Presenter::shot`). Live in-between presents still copy at their phase (by design phase-dependent).
