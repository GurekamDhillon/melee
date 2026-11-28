# Build speed and memory (branch port/buildspeed, 2026-11-14)

Machine: 32 GB, 20 threads, 3 build slots, other lanes building at the same time. Every number below is therefore noisy
(the same operation measured 247 s and 417 s). `cargo build --release -p melee -j 3`, `CARGO_INCREMENTAL=0`, no wrapper, private
target dir, release profile as committed (debug = 1). Raw data: `_build/bs/` (timings html, per-process peak working set).

## Before (measured)
| case | wall |
|---|---|
| clean release build of the game | 7 m 16 s (cargo timings: 436 s critical path, 212 units) |
| rebuild after editing `melee_data` (a mid-graph crate: melee_modes, melee_engine, melee relink) | 3 m 39 s .. 6 m 57 s (4 runs, load dependent) |
| rebuild after editing `melee_world` | 4 m 20 s |
| exe link alone | under 1 minute of the `melee` bin's 43 s unit; link.exe peak 0.9 GB |
| mods only (`build_mods.sh`, 13 wasm mods, serial) | 12 s |

Top 10 by time (s): melee_link 155, melee_engine 103, melee_modes 77, naga 55, melee_types 53, melee_items 51,
melee_fighter 49, melee bin (codegen + link) 43, wgpu-core 40, melee_world 38.

Top by peak memory of one rustc (MB): **melee_link 9127**, melee_engine 3866, melee 3555, melee_items 2996, melee_fighter 2555,
melee_modes 1873, melee_chara 1730, melee_world 1194, melee_types 1170, melee_runtime 1011.
`melee_link` is the outlier: one crate of 63k lines whose `gen_table.rs` (47k generated lines, 5 MB) is a few giant
`install_*` functions holding thousands of closures; rustc's per-function passes make it 9 GB and 155 s, serial at the point where
nothing else can run.

## Tried, measured (not kept unless noted)
| change | result | decision |
|---|---|---|
| rust-lld as linker (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld`) | 404 s vs 247 / 417 s for link.exe on the same edit: no signal above the noise; the link is not where the time is | not kept |
| `debug = 0` in release | 6 m 21 s vs 7 m 16 s clean; exe 42.36 MB vs 42.37 MB (debuginfo goes to the pdb) | not kept: release must not change and crash logs use the pdb. Fine for non-release (`fast`/test already have it) |
| sccache on the release build | cold 5 m 51 s, same-tree warm 5 m 40 s (0 hits both times in the big build, unexplained; 4 c-in-3 proc-macro/bin units are never cacheable) | not kept for release. A small test (`-p png`) shows dependency crates ARE shared across different target dirs (10 hits in a new dir), so sccache stays as it is in `cargo_fast.sh` (deps only, workspace incremental) |
| build-override opt-level | already 0 by default for build scripts and proc macros | nothing to do |
| `lto`/codegen-units for release | untouched on purpose | - |

## Kept
1. **One target dir per worktree (the stale `melee_fighter` rlib bug).** Cargo names workspace crates by a hash of their
   workspace-relative path, so two worktrees pointed at one `CARGO_TARGET_DIR` (`_build/cargo-shared`) get the same file names, and
   one lane's rlib replaces another's while fingerprints still look fresh. `cargo_fast.sh` and `gate.sh` now ignore a
   `CARGO_TARGET_DIR` outside the worktree (`ALLOW_SHARED_TARGET=1` to keep it); the default is `build/cargo` from
   `.cargo/config.toml`. Cost: disk, about 3 to 9 GB per worktree release target: delete it on hand-back. Sharing between
   lanes then comes from sccache (third-party crates only). `scratch/gbspawn-test.sh` still exports the shared dir: remove that line.
2. **Mods-only path:** `tools/rust-port/modhost/build_mods.sh [mod ...]` builds only the named wasm mods, 4 at a time, each in its
   own target dir. All 13 clean: 12 s before, 4 s after; one mod after an edit: 1.4 s. The wasm files are byte-identical to the old
   script's (sha256 of all 13). The exe need not be rebuilt: the host loads `build/mods/*.wasm` at run time.
3. **`buildlock.sh` (also copied to `tools/rust-port/buildlock.sh`; the live one in `scratch/` is installed from it):**
   - `run|take|release --light <owner>`: separate pool (`scratch/.light-slots`, default 2) for `cargo check` and single-crate
     tests; needs 4 GB free; does not use the heavy slots.
   - Lanes: `genoboss` and `genoboss-b` (suffix `-b`, `-2`, `-b2`, `_b`) are one lane; a second name of a lane that holds a heavy
     slot shares it instead of taking another (no deadlock if a script nests). Old CLI unchanged; tested on a copy
     (`BUILDLOCK_DIR=<tmp>`) first.

## Proof the game is unchanged
`tools/rust-port/gate.sh` on this branch (release rebuilt from scratch in the worktree): GATE OK (goldens, synctest, audio,
baselines 144, lint, semantics lint 0, snapguard within baseline). No game source or profile changed.

## Plan, not done
### Split the giants (cut peak memory and the serial tail)
1. `melee_link`: change `gen_link.py` to emit chunks of at most about 150 hooks per `fn` (`#[inline(never)] fn install_N`) in
   several files (`gen_table_00.rs` ...), with a small `install_all` calling them. Expect peak 9 GB to about 2 GB, and
   parallel codegen units. This is the single biggest win and touches only the generator.
2. `melee_engine` (3.9 GB, 103 s, 38k lines): split the lowest-level pieces (arena/cow/lowmem, snapshot) from the rest.
3. `melee_modes` (196k lines, last in the chain): split `tu/script_game` (36k lines) and `geno` (and its 145k-line tests) into
   `melee_script_game` / `melee_geno` crates; the integration tests are the biggest compile units in `cargo test`.
4. `melee_items` (115k) does not depend on `melee_fighter`: it already builds in parallel with it. `melee_chara` (86k, 235 files)
   depends on both; split by character group only if `cargo timings` shows it on the critical path after 1 to 3.
5. Always re-run the gate after each split (symbol order and fp sites must not change).

### Cross-build the Windows exe on the Linux laptop
Target `x86_64-pc-windows-msvc` with `cargo-xwin` (downloads the MSVC CRT and Windows SDK, needs `clang`, `lld`, `llvm`, and the
Microsoft SDK licence accepted) or `x86_64-pc-windows-gnu` with mingw (different ABI, would not be bit-identical to the shipped
build). Blockers: the three build scripts (`melee/build.rs`, `melee_modes/build.rs`, `melee_world/build.rs`) must not assume a
Windows host; `webgpu_dawn`/SDL3 DLL packaging stays on the desktop; float code is Rust-defined so output should match, but the exe
must pass the gate on Windows before it is trusted; msvc `link.exe` vs lld-link produces a different binary (pdb, ordering). Do a
spike: `cargo xwin build --release -p melee` on the laptop, then run `gate.sh` against the result on the desktop.

## melee_link split into install crates (Nov 15)

Chunking alone (150 hooks per `fn`, 10 files) cut melee_link's peak rustc only from 8.9 to 6.2 GB: the peak is
`late_resolve_crate` resolving all 14,947 hook statements (29,877 paths) in one crate. `gen_link.py` now emits the installs
as six workspace crates `rust/melee_link_p0..p5` (consecutive 150-hook chunks, cut at the nearest equal share: 2,400-2,564
hooks each, `NPARTS` in the script) plus `melee_link_abi` (`cast`, `va_word(s)`, `cabi`, re-exported at each crate root so
the hook lines are unchanged). `melee_link::install()` calls `melee_link_pN::install()` in order. Grouping by dependency crate
does not help: every leaf group's hooks reach five or six of the leaf crates. The committed tables are a mechanical re-split
of the previous ones; `tools/rust-port/link/check_split.py <old single-file gen_table.rs> rust` shows the ordered install
statements and HOOKS table identical.

Measured on the desktop, release, `-j 3`, `CARGO_INCREMENTAL=0`, own target dir, peak working set per rustc:

| | before (chunked, one crate) | after (6 crates) |
|---|---|---|
| melee_link peak | 6,209 MB | 2,724 MB (retail_ms + HOOKS floor) |
| largest part crate | - | 1,884 MB (p2) |
| clean `cargo build -p melee` | 350 s | 350 s |
| touch `melee_link/src/lib.rs`, rebuild melee | 157 s | 137 s |
| edit `melee_modes/src/lib.rs`, rebuild melee | 160 s | 190 s (six part crates rebuild at -j 3) |

The largest rustc of a build is now melee_engine (3.9 GB) / melee (3.6 GB).

## Agent check builds use `--profile fast` (branch port/fastprof, 2026-11-15)
Measured on this machine with other lanes building (noisy), `cargo build -p melee -j 3`, no sccache, private target dirs, same
one-line edit (an unused `pub fn`) in each crate for both profiles. Release as the gate builds it (`CARGO_INCREMENTAL=0`); fast
as committed (incremental, 256 cgus, no LTO, debug 0).

| case | release | fast |
|---|---|---|
| clean build | 432 s | 391 s |
| rebuild after a one-line edit in `melee_world` | 224 s | 94 s |
| rebuild after a one-line edit in `melee_modes` | 226 s | 78 s |

Peak rustc memory was sampled over all rustc processes on the machine, so other lanes' compiles are in it: release clean 9.1 GB
single / 16.7 GB total (melee_link, as before), fast clean 7.7 / 11.5 GB. The fast edit rows (10.2 GB single) are another lane's
melee_link; read them as "no worse". Fast is not a memory fix: melee_link still needs its 12 GB wait until the generator split lands.

### Hashes are byte-identical between profiles
Same source, two exes (release and fast), compared with `cmp`:
- goldens `pad_falcon_fox_fd`, `pad_marth_sheik_bf`, `cpu_fox_falco_dl`, `cpu_peach_samus_ys_items`, 3601 frames each, the whole
  NHV1 CSV (hash columns included): identical;
- `synctest --engine tu` depth 4 and 7 (600 frames, golden-checked): identical result lines, 0 mismatches;
- a boss row on a tree with the Geno VM (commit 4a8b6a1a9, not part of this branch): Ridley through `geno_vm_check.sh`
  fight / sync 4 and 7 / nhv1, 1500 frames: fight.csv, both nhv1 runs and both synctest lines identical; last NHV1 row
  `1499,338BA078,C3874CD2` in both.

### What changed
- `cargo_fast.sh build|run` with no `--release`/`--profile` now means `--profile fast`; `RELEASE=1` means `--release`.
- `gate.sh`, `golden/coverage/run_rust.sh`, `golden/check_slp.sh`, `modhost/determinism.sh` and the Envoy/quickswap/sfx/tag/netplay/slippi
  check scripts default to `build/cargo/fast/melee.exe`. `PROFILE=<p>` still picks one; `RELEASE=1` picks release.
- The gate's snapshot perf guard is release-only, so a fast gate skips it: run `RELEASE=1 bash tools/rust-port/gate.sh` when
  perf matters.
- Stay on release, on purpose: the exe handed to the owner (candidate builds), `launcher/package.sh`, `yougame/package.sh`,
  `perf/zero_hitch_gate.sh`, `perf/snapguard.sh`, and any frame-time measurement. A fast exe runs a little slower (no LTO).
  Say "release" in the report of any such build.

## melee_link compile memory re-measured on port/buildspeed2 (Nov 15): no regression
A sampler once recorded 9,117 MB for "melee_link" on this branch. Re-measured at da42db877 (six part crates, 15,088 hooks, HOOKS
17,590 rows) merged with genbloat 8559eac89, own target dir, `-j 3`, peak working set of each rustc filtered to this worktree's
path (the earlier sample matched every rustc on the machine, so another lane's unsplit melee_link or melee_engine can show up under
the same name). `gen_link.py` already keeps only HOOKS, `install_all` (six calls) and the retail code in melee_link: no generator
change was needed.

| crate | release (MB) | fast (MB) |
|---|---|---|
| melee_link alone (touch, rebuild) | 2,730 (55 s) | 3,207 (6 s, incremental) |
| melee_link_p0..p5 | 1,066 / 1,437 / 2,162 / 1,521 / 1,260 / 988 | - |
| melee_engine | 3,453 | - |
| melee | 3,536 | - |
| melee_items / melee_fighter / melee_modes | 2,784 / 2,553 / 1,654 | - |

Release `cargo build -p melee`: clean 350 s; edit melee_modes 152 s; edit melee_engine 152 s (incl. the engine's 29 s rebuild after
genbloat; was 130 s). `check_split.py` against linkdrift's single-file table: IDENTICAL. `gate.sh` and `RELEASE=1 gate.sh`: GATE OK.

## Nightly toolchain pin (branch port/nightly, Nov 15)
The repo is pinned to `nightly-2026-11-13` (rustc 1.102.0-nightly) by `rust-toolchain.toml` at the repo root (rust/ and the wasm mods
inherit it): components `rust-src`, `llvm-tools`, target `wasm32-unknown-unknown`, profile minimal. `rustup` installs it on first use.
The global default toolchain is untouched; `RUSTUP_TOOLCHAIN=stable` overrides the pin for a comparison build. Cargo.lock and the
game crates are unchanged (no `#![feature]` is used: the gain, where there is one, comes from the compiler).
`semantics_lint.py` keeps its own `SEMLINT_TOOLCHAIN` (nightly-2026-02-14, needs clippy).

### Determinism: stable 1.96 vs the pin, same tree (port/buildspeed2), release, byte for byte
- four golden CSVs (3601 frames, whole NHV1 files): identical; audio wav and mix hash `6106 0xf4956676e79a4037`: identical;
- `synctest --engine tu` depth 4 and 7: identical result lines, 0 mismatches;
- `run_modes_check.sh` camp route, 6000 ticks, headless and window: tick.log (6000 lines) and hash.log (4904 lines) identical
  between the two toolchains (each used the wasm mods built by its own toolchain; the wasm files themselves differ in bytes);
- Geno VM on a throwaway merge of port/geno-vm4 + buildspeed2 (not pushed): Ridley fight csv identical, NHV1 last row
  `2999,9D2E0FF4,FAD1A1DD` in both, synctest d4/d7 0 mismatches; Tabuu/Bucculus (`geno_grab_check.sh` hold/chase/sync/nhv1):
  every csv identical, last rows `623146CA,DE51755F` and `D656E509,072E830F`. (`geno_vm_check.sh` with BOSS=tabuu does not run
  the VM: Tabuu is a grab actor there, so its VM rows FAIL on both toolchains alike.)

### Desktop timings, `-j 3`, other lanes building (noisy), own target dir per toolchain, no sccache
| case | stable 1.96 | nightly pin |
|---|---|---|
| clean release `cargo build -p melee --release` | 303 s (324 s on the geno merge) | 300 s (307 s) |
| clean fast | 367 s | 398 s / 310 s (second run, quieter) |
| fast, edit `melee_modes` (2 runs) | 66 / 57 s | 58 / 60 s and 42 / 43 s |
Result: on the desktop the pin is no faster than 1.96 (the laptop's 30% was against Arch's packaged 1.99). Exe size 42.00 MB vs 42.13 MB.

### Faster options tried on the pin (fast profile only; release unchanged)
| option | result | decision |
|---|---|---|
| `-Zshare-generics=y` | clean 588 s, edit 85 / 79 s vs 398 / 58 / 60 s | slower; not kept |
| Cranelift (`-Zcodegen-backend=cranelift`, component `rustc-codegen-cranelift-preview`) | compiles in 290 s (195 s with `-Zshare-generics=n`) but the exe does not link: LNK1120, 8,601 unresolved externals (the generic `cabi` closure instances of `melee_link_abi` used by `melee_link` and its part crates). Hashes therefore not testable | not adopted |
| `rust-lld` (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld`) | edit rebuild 43 s vs 42 / 43 s with link.exe | no signal; not kept |
| `-Zthreads=8` (laptop) | no gain | not kept |

## Fresh timings and the verdict on steps 2 and 3 (port/modessplit, Nov 28, at 96519221a)
Desktop, `-j 8`, own target dir, no sccache, other lanes possibly building (noisy; one run per clean case, three per edit case).
Raw timing html: `build/cargo/cargo-timings/` in the worktree (deleted on hand-back).

| case | result |
|---|---|
| clean `cargo build --release -p melee` | 3 m 36 s (246 units, 217 s of unit time on the critical path) |
| clean `--profile fast` | 3 m 47 s |
| fast, edit `melee_modes/src/tu/gmtitle.rs` (3 runs) | 55 / 43 / 42 s, median 43 s |
| fast, edit `melee_chara/src/adapt.rs` (3 runs) | 51 / 45 / 44 s, median 45 s |

Release, top units by time (s): melee_link 66, melee_types 56, naga 53, melee_fighter 46, melee_items 45, wgpu-core 45,
melee_modes 45, melee_world 39, melee_runtime 35, melee (bin) 34. Fast: melee_link 77, melee_types 59, melee_fighter 51,
naga 51, melee_modes 51, melee_items 45, wgpu-core 45, mex_import 41, melee_world 39, melee 36.
Peak rustc working set (fast clean, MB): melee 4,198, melee_engine 4,064, melee_fighter 3,004, melee_items 2,970,
melee_link_p2 2,764, melee_link 2,381, melee_modes 1,957, melee_chara 1,732, p3 1,658, p1 1,635.

Critical path (release): melee_types/fighter/chara -> melee_link_p0..p5 (finish about 120 s) -> **melee_link (116 -> 183 s, 66 s)** ->
melee (183 -> 217 s). melee_modes (86 -> 131 s) and melee_engine (153 -> 174 s) both finish before melee_link does: neither is on
the critical path. Same shape in fast (melee_link 115 -> 192 s of 228).

**Step 3 (melee_script_game / melee_geno) was not done**, for two reasons:
1. melee_modes is off the critical path; it only matters for the edit loop (43 s median), and the exe link plus melee_link's
   relink are a good part of that already.
2. The split is not a move of files but a cycle break. `tu/script_game.rs` (36k lines) and `geno_*.rs` use `crate::tu::adapt`
   (203 and 153 references), `crate::oracle::Va` and about 370 `ext_fns!` declarations whose defaults are
   `crate::tu::adapt::<fn>__<hash>`; in the other direction `adapt.rs`, `envoy.rs`, `gmscene.rs`, `gmvs.rs`, `gm_1601.rs`,
   `petey.rs`, `stage_hop.rs`, `ext_hook.rs` and others call into `script_game` / `geno_game` / `geno_lab_mode`. A new crate
   needs the adapt hub, `ext_fns!` and `oracle` pulled down into a lower crate and the reverse calls turned into registered
   hooks; hook defaults are function pointers compared by identity (`unpredictable_function_pointer_comparisons = deny`), so
   this needs its own gate round. 80k lines of churn for no critical-path gain.
Where the time is now: the serial tail is **melee_link itself (66 s, 36% of the build)**. Its `retail*.rs` (33k lines,
`retail_ms` 12.9k) do not use the part crates; only `gen_table.rs::install_all` does. Moving `manual/retail/retail_ms/retail_it/strs`
into a `melee_link_retail` crate would let them compile alongside p0..p5 instead of after them (the gain is bounded by the
retail share of the 66 s; measure `-Z time-passes` on melee_link first). That is the cheapest next step, ahead of steps 2 and 3.
Step 2 (melee_engine, 21 s, off the path in release) also has no critical-path gain; it is only worth it for the 4.1 GB peak.
