# Rust port (branch `rust-port`)

Goal: the whole codebase (decomp C, the 10 PowerPC `.s` files, `pc/` platform layer) in Rust,
with game logic bit-identical to the C build so rollback netplay and Slippi replays still agree.

## Oracle (already in the C build)
- `MELEE_TURBO=1` + `MELEE_PAD_SCRIPT` / `MELEE_SLP=<replay>`: headless, fast simulation.
- `MELEE_TURBO_HASHLOG=<csv>`: per-frame full snapshot hash.
- `MELEE_SYNCTEST_CURATED=1`: gameplay-only hash (the full byte hash drifts in HSD particle state).
- `MELEE_RB_HASH_DETAIL=<csv>` + `pc/tests/compare_rb_hash_detail.py`: per-region diff to localise a mismatch.
- `MELEE_RAND_TRACE`: RNG draw trace, diffable.

A Rust module is "done" when golden replays produce the same per-frame hashes as the C build.

## Phases
1. Golden corpus: record C-build hash CSVs for a fixed replay/pad-script set.
2. c2rust transpile of `src/` + `pc/` C sources; link and boot.
3. Parallel per-module rewrite to idiomatic Rust, gated on the hash oracle.
4. Platform layer (Aurora/Dawn/borealis -> wgpu/winit/cpal/gilrs), checked visually + render_test.

## Toolchain
- LLVM 23.1.4 at `C:\Program Files\LLVM` (`LIBCLANG_PATH` = its `bin`).
- c2rust via `cargo install --locked c2rust`.

## Inventory (docs/rust-port/inventory.md)
- 993 game TUs (~557k .c lines) from `_build/masstest/files.txt`; MetroTRK/Runtime dead; MSL is
  headers + `math_ppc.h` (Newton-Raphson on bit-exact `__frsqrte`, must be ported, not std).
- Native x86_64 parse: fails only on MSL `size_t` and 104 `sizeof` layout asserts (8-byte pointers);
  masked, 0/993 fail. Real native work: big-endian relocated DAT files, 1,274 bitfields (MSB-first
  on PPC), punning unions, 684 pointer<->int casts.
- The "10 asm" = 5 `.s` + 5 inline-asm C files; none carry game logic on PC.
- GC offsets: generate from `clang --target=ppc32-none-eabi -Xclang -fdump-record-layouts-complete`.
- `gk_fmadds` is double-then-float, not `f32::mul_add`.

## Translation toolchain (2026-10-21; supersedes "c2rust via cargo install" above)
- `cargo install c2rust` fails on Windows; it works in **WSL Debian** (no sudo):
  `wsl -d Debian -- bash "/mnt/e/Projects/Melee Workspace/worktrees/rust-port/tools/rust-port/setup_wsl.sh"`
  installs LLVM/clang 18.1.8 prebuilt + libtinfo.so.5 into `~/opt`, then c2rust 0.22.1, and writes
  `~/opt/c2rust-env.sh`. (Docker daemon was not running; not used.)
- `bash tools/rust-port/transpile.sh [--crate NAME] [-j N] <paths...>` (from Git Bash; re-execs in WSL).
  Files/dirs/globs, filtered to `docs/rust-port/game_tus.txt`. One c2rust run per TU, in parallel.
  Output: `rust/<crate>/src/raw/<path minus src/>.rs`, e.g. `src/melee/cm/camera.c` ->
  `rust/melee_world/src/raw/melee/cm/camera.rs`. Logs: `rust/target/transpile-logs/`.
  `src/raw/` is gitignored and not `mod`-declared, so `cargo build` never sees it.
- Flags (`tools/rust-port/gen_cc.py`): `--target=x86_64-pc-windows-msvc` (LLP64: C `long` stays 32-bit
  like PPC; `s32`/`u32` come out as c_int/c_uint), `-DTARGET_PC -ffp-contract=off`, plus masks:
  `-D_Static_assert(a,b)=` (104 GC sizeof asserts), `-Dsize_t=gc_size_t` (MSL size_t clash, so raw
  `gc_size_t` is c_ulong = u32 on Windows: change to usize when porting), `-mlong-double-64`, and
  `tools/rust-port/include/stdarg.h` (plain `__builtin_va_arg`; MSL's TARGET_PC va_arg uses
  `__builtin_types_compatible_p` which c2rust panics on). Font .inc files come read-only from
  `../../melee/build/GALE01/include` (override: `GALE_INCLUDE`).
- Measured: **all 993 TUs transpile** (984 first pass; the 9 failures were all va_arg, fixed by the
  stdarg override), ~5.3M raw lines because every TU repeats its headers' types.
- Raw output is not compilable as-is: c2rust emits `extern type` (nightly), `use ::c2rust_bitfields`
  (884 files), `pub _:` fields where C names a field `_` (rustfmt rejects; c2rust still writes it), and
  `::f128::f128` for `long double` literals like `0.0L` in 3 files (camera, ground, jobj: on the GC
  long double is f64, use f64). `raw/pc/gameworld/gekko_fp.rs` does compile standalone on stable.
  Raw code is reference input; ports go into each crate's real modules.

## Cargo workspace (`rust/`)
`cargo build` / `cargo test` in `rust/` are clean (stable 1.96, no warnings). Crates, in dependency
order: `melee_core` (fp: `fmadds`/`fmsubs` as f64-then-f32; gc: `GcField`, `GcLayout` trait,
`gc_layout!` macro for the generated GameCube offset tables; hash: `StateHash`/`StateHasher`, FNV-1a
64 over big-endian canonical bytes so a C walker over guest memory can match), `melee_types` (g0),
`melee_runtime` (g1), `melee_world` (g2), `melee_fighter` (g3, incl. ftCommon + pl), `melee_chara`
(g4), `melee_items` (g5), `melee_modes` (g6), and the `melee` bin that links all (prints a self-test).
Group crates are empty stubs. Not built yet: the ppc32 record-layout dump -> `gc_layout!` generator.

## Golden corpus (phase 1)
- `MELEE_NEUTRAL_HASHLOG=<csv>`: byte-order-neutral gameplay hash (`RB_NeutralHash`, fighter.c),
  RB_GameHash's field set by logical value, minus PC-only add-ons. Spec + GC offsets:
  `docs/rust-port/neutral-hash.md`. The Rust port must reproduce this exactly.
- Native code must not pass a pointer into game code to receive results (game stores are
  big-endian); results come back one u32 at a time (`RB_NeutralHashWord`).
- Record: `tools/rust-port/golden/record.sh` (runs each scenario twice, requires identical CSVs);
  goldens in `docs/rust-port/golden/*.csv`; compare with `tools/rust-port/compare_golden.py`.
- Build root: `_build/agents/rustgold` (agent_new.sh's root half, run against this worktree).
- Traps found recording it (2026-10-21): (1) MSVC `std::chrono::current_zone()` throws for the
  Windows zone "British Columbia Standard Time" (ICU lacks it) and every exe died at boot frame 2,
  the shared baseline too; `extern/aurora/lib/dolphin/os/OSTime.cpp` now falls back to the CRT.
  (2) `gd.input(port, spec, 1)` from `on_frame_pre` is never read in turbo (the hold is consumed
  before the next PADRead); use frames=2. `pc/tests/gdlp_whole/pad.lua` has the same bug (its
  fighters stand still). (3) A fresh lane's Aurora dep copy can fail on read-only `.git` packs
  (`dawn-src.pending`): `attrib -R /S`, delete, robocopy the dep in by hand.
