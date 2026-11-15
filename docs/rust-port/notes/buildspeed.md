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
