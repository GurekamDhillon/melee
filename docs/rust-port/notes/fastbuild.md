# Fast edit-build-test loop (profile `fast`, sccache)

Measured 2026-11-02 on the worktree `fastbuild` (branch `port/fastbuild`), Windows 11, same machine, nothing else running
(coverage runs and builds were never overlapped). Medians of 3 unless stated. Release is unchanged.

## Use
- Build: `bash tools/rust-port/cargo_fast.sh build -p melee --profile fast` (exe: `build/cargo/fast/melee.exe`).
  Any cargo arguments pass through. `NO_SCCACHE=1` = plain cargo; `FAST_NO_INCR=1` = `CARGO_INCREMENTAL=0` (see trade-off).
- Gate: `PROFILE=fast bash tools/rust-port/gate.sh` (builds through cargo_fast.sh, then runs `build/cargo/fast/melee.exe`).
  Default stays `release`. Coverage: `PROFILE=fast bash tools/rust-port/golden/coverage/run_rust.sh --all` (or `MELEE_EXE=`);
  `golden/check_slp.sh` honours `PROFILE` too.
- Do not set `CARGO_INCREMENTAL=0` for the fast profile unless you mean `FAST_NO_INCR=1` (the env var overrides the profile).

## What the profile is
`[profile.fast]` in rust/Cargo.toml: inherits release (so opt-level 3, overflow-checks off), `incremental = true`,
`codegen-units = 256`, `lto = false`, `debug = 0`. opt-level 2 was NOT measured: opt-level 3 is kept so the code generator
is the same as release's. Rust never contracts a*b+c, and the fmadds sites are explicit calls, so codegen-unit count,
incremental and LTO cannot change float results; the checks below confirm it.

## sccache and incremental: the trade-off
sccache does not cache a crate rustc compiles with `-C incremental`, and cargo only makes workspace (path) crates
incremental. So by default the third-party dependencies (wgpu, naga, lua-src, ...) come from sccache and the workspace
crates from each worktree's `build/cargo/fast/incremental`. `cargo_fast.sh` sets `RUSTC_WRAPPER=sccache`,
`SCCACHE_DIR=E:/Projects/Melee Workspace/_build/sccache`, `SCCACHE_CACHE_SIZE=30G` (all overridable by env).
`FAST_NO_INCR=1` makes the workspace crates cacheable too (a rebuild of an unchanged tree is then ~43 s), at the price that an
edit recompiles whole crates. Keep incremental (default) for the edit loop; use `FAST_NO_INCR=1` for a fresh target dir
of a tree sccache has already seen. The sccache key includes the source path, so a different worktree path should miss on the
workspace crates and hit on the dependencies (not measured; the table below is all in one worktree).

## Verification (fast exe)
- `PROFILE=fast gate.sh`: GATE OK. Goldens 1801 / 1801 / 3601 / 3601 (seed column included), synctest 240/240 with 0
  mismatches, audio hash equal to the C capture (6106 sub-frames), baselines 144, lint 398 (no new violations).
- Coverage `run_rust.sh --all -j 3`: release 53 PASS, 26 FAIL, 79 NOT_WIRED, 10 NO_GOLDEN; fast 52 PASS / 27 FAIL with identical rows except `char_luigi` (frame-0 mismatch, 0 frames) in
  that one run. `char_luigi` alone passed 3601/3601 on 3 more single runs and 8 parallel runs for each of fast and release, so
  that is a transient of the parallel batch (not reproduced, cause unknown; not shown to be profile specific), and the
  settled pass count is 53 for both.
- A `-j 3` full coverage run took 345 s (release) vs 406 s (fast); the fast exe is a little slower at run time (no LTO, 256 cgus).

## Timings (seconds)
| case | release | fast |
|---|---|---|
| no-op rebuild | 0.33 | 0.35 |
| touch-and-edit `melee_items/src/hash.rs` (appended a fn, so a real change) | 188 (174 / 188 / 188) | 161 (75 / 161 / 251: very noisy) |
| edit `melee_chara/src/tu/ftcaptain.rs` | 169 (152 / 169 / 180) | 48 (44 / 48 / 50) |
| clean build, new target dir, cold sccache | 284 (single, no sccache) | 337 (single; sccache fills) |
| clean build, deps warm in sccache, incremental workspace | n/a | 260 (243 / 260 / 263) |
| clean build, `FAST_NO_INCR=1`, everything warm | n/a | 43 (43 / 43 / 45) |
| clean build, `FAST_NO_INCR=1`, workspace cold | n/a | 211 (single) |

Reading it: the real win is the fighter-file edit (169 -> 48 s, 3.5x). `hash.rs` sits near the bottom of the crate order, so
nearly everything above it rebuilds anyway and fast helps little (and was noisy). The workspace crates dominate a clean build
(~210 s of ~260 s), so sccache on the dependencies alone saves only ~25% (337 cold -> 260 warm, and that cold number includes
the sccache write overhead; plain release clean is 284); caching the workspace crates too is what gets to 43 s.
Last sccache stats: 549 MiB for the whole session, cache max 30 GiB.
