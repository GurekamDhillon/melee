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
samples whose stack contains the substring (e.g. `Engine::tick` to drop the 5 s boot). Windows only, no OS input.
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
| golden run end to end, pad / cpu | 18.5 s / 37.0 s | 5.4 s / 7.0 s (5 s of that is boot) |
| synctest depth 7, whole tick (live frame + save + load + 7 resim + 9 hashes), pad / cpu | 65.5 / 71.5 ms | 7.4 / 9.5 ms |
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
- Boot (about 5 s per run, not part of a tick): `GcDecoder::issue` builds a `String` per non-relocated word (format, `rsplit`),
  `memchr`, `HashMap<u32,()>::contains_key` (SipHash) in `GcDecoder::target`. Making `issue` lazy/capped and using a fast hasher
  would cut most of it.
- `EngineGame::run_frame` formats the 16-word row into a String per frame and keeps it in a BTreeMap forever (synctest/netplay).
- Spin lock + class computation in `arena::alloc/free` (CAS per call).

## Method
Tick: `melee golden --engine tu --frames N` wall minus the same command with `--frames 2`, divided by N-2 (N = 1801 pad, 3601 cpu),
`MELEE_WINDOW_UNFOCUSED=1`, no window, no render. Rollback: `melee synctest --scenario S --depth K --frames 600`, the per-tick
counters it prints (boot excluded). Scripts are scratch only (not committed). Before binaries are rebuilt from d373bba89 with
only the timing counters added, so the snapshot/jobj code measured is the original.
