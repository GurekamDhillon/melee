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
(Status: see "Integration, night 1" below.)

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

## Float contraction (group 1, 2026-10-21; corrects inventory.md section 3)
- The C build compiles with clang's default `-ffp-contract=on`: each `a*b+c` / `a*b-c` / `c-a*b`
  inside one expression is `llvm.fmuladd`, which gwtool lowers to `fmadds` (f32: f64 multiply,
  f64 add, then f32) or a true `fma` (f64). A port must fuse exactly those sites: LHS multiply
  first, compound `+=` included (`docs/rust-port/group-1 runtime, math, HSD (core + render).md`).
  c2rust ran with `-ffp-contract=off`, so raw output has lost every fusion; check the C IR.
- gwtool also defines float->int casts (saturate to i32, then truncate for narrow types), div/rem
  by zero and oversize shifts; uninitialised locals are 0. Helpers: `melee_runtime::cabi`.
- C oracle: `rust/melee_runtime/oracle/build.sh` (native IR + gwtool's float rewrites redone in
  `lower.py`) -> `tests/data/oracle_vectors.txt`, checked bit for bit by `tests/oracle.rs`.
- Ported and oracle-equal: gekko_fp.c, math_ppc.h sqrtf, mtx_pc.c, mtx44.c, baselib/random.c,
  lb/lbtrigf.c, lb/lbvector.c (not WorldToScreen). The rest of group 1 (HSD objects) is not ported.

## Group 0: melee_types (types + GC layout table)

Details: `docs/rust-port/group-0 types + GC layout table.md`.
- Types are GENERATED, not transpiled: `python rust/melee_types/gen/gen_types.py` (Windows Python
  + `pip install --user libclang==18.1.1`) parses every group-0 header with libclang as
  ppc32-none-eabi -DTARGET_PC (GC offsets, cross-checked against -fdump-record-layouts-complete)
  and as x86_64-msvc (native size asserts). Output `rust/melee_types/src/gen.rs` is committed.
- Every record implements `melee_core::gc::GcLayout`; `melee_types::GC_RECORDS` lists them all.
- Bitfields are `bf_<gcoff>: [u8; n]` in GC byte order with `name()`/`set_name()` accessors.
- Enums are int aliases + consts; `long`=i32, `bool`(MSL int)=i32, `size_t`=usize.
- Trap: the C typedef `Vec` shadows `std::vec::Vec` under `use melee_types::*`.
- `melee_types::neutral_hash` computes NHV1 from native structs; tested against the C functions
  compiled natively (gen/nh_oracle.py) and against all 10804 golden rows.

## Group 3: fighter engine + ftCommon + pl (2026-10-21; detail: docs/rust-port/group-3 fighter engine + common states.md)
- Native types with a GC offset record: `rust/melee_fighter/gen/gen_types.py` (libclang, ppc32 parse)
  -> `src/types/generated.rs`, 569 `#[repr(C)]` records each with a `GcLayout` table; Fighter 0x23EC.
- All 216 TUs compile as a mechanical native translation: `gen/fix_raw.py` over the raw c2rust
  output -> `src/c2r` (gitignored, 50 MB), `cargo build -p melee_fighter --features c2r`. Not linked/run.
- Hand ports verified bit-for-bit against the C (gen/cref/run.py compiles the game's own C natively,
  cut verbatim): NHV1 fighter hash, 33 ftcommon.c movement helpers, staling (plstale/plattack/ft_0881).
- Trap: `HitCapsule::hit_grabbed_victim_only` is a `:1` unioned with the owner pointer, i.e. the GC
  address MSB (1 for every heap pointer); see `melee_fighter::compat`. `ABS(-0.0)` is -0.0, not abs().

## Integration, night 1 (2026-10-21; supersedes the per-group notes where they disagree)

`cd rust && cargo build --workspace && cargo test --workspace` pass on stable 1.96 with no
warnings (25 test binaries, all green). Groups 2, 4 and 5 left no section of their own here; their
detail is in `docs/rust-port/group-2-world.md`, `group-4 fighter characters.md`,
`group-5 items, trophies, vi.md`. Group 6 (`melee_modes`) was not started.

### Status per group

"Verified" means checked bit for bit against the game's own C compiled natively (an oracle), not
against a running Rust game: no Rust simulation exists yet, so no golden has been reproduced by Rust.

| group | crate | C functions | hand-ported | verified vs C | types + GC offsets | stubbed / missing |
|---|---|---|---|---|---|---|
| 0 types | melee_types | 110 header inlines | 82 | NHV1 words (8 oracle cases) + frame combine on all 10804 golden rows | 1618 records, the ONE generator (libclang ppc32), covers groups 0/3/4/5 headers | 28 inlines need other groups |
| 1 runtime/HSD | melee_runtime | ~2000 | ~235 (gekko_fp, sqrtf, mtx, mtx44, random, lbtrigf, lbvector; round 1: baselib mtx.c 14/22, quatlib 6/6, spline 11/11; round 2: fobj.c 24/30 keyframe interpreter, util.c 3/3, crypt.c 5/5; round 3: GObj system 36 (gobjplink/gxlink/proc/object/userdata/init all, gobj.c 10/16); round 4: aobj.c 18, id.c 5, list.c 6 = 29; round 5: jobj.c 27 (tree, flags, dirty propagation, MakeMatrix, SetupMatrixSub; IK solvers stubbed); round 6: wobj.c 13 + dobj.c flags 5 = 18; round 7: robj.c 38 + mtx.h ColMag/ColVec = 40, docs/rust-port/group-runtime.md) | all: 11400 rows + 33000 rows (oracle_hsd, 66k checks) + 65000 rows (oracle_anim, 135k checks) + GObj transcript (oracle_gobj: 40 seeds x 400 random ops, 55.8k lines, 16k state hashes); oracle_aobj (34.7k rows, 32k checks, fmodf fusion discriminated); oracle_jobj (22k random ops, state hash after each); oracle_wobj (15k random ops over WObj/JObj/spline/DObj flags, 1197 path updates); oracle_robj (400 scenarios, 16.6k checks: LoadDesc/ResolveRefs/UpdateAll constraint solver, 22 of 40 fns checked, ColMag fusion discriminated, NaN hashed canonical); oracle redoes gwtool fusion + safe shifts | Vec3/Quaternion; GObj/GObjProc/LibInitData/DelayedProcInfo delegate to melee_types; DAT reader `dat.rs` (night 2) | HSD objects (rest of JObj incl. IK/anim/load/disp, rest of DObj, MObj/CObj/LObj/Fog...; RObj not yet wired into JObjHost, ~1070; AObj ForeachAnim), gobj.c render callbacks, lb/db/sfx/THP |
| 2 world | melee_world | 3013 | 301 (mplib line queries, line geometry, segment checks + bounding, joint list edits + mpLib_80056C54, seam stitching + floor bounds; all of mpisland.c; round 5: 42 cm/camera.c framing/follow math, quakes, accessors; round 6: all 45 of gr/stage.c incl. slot select + ArenaBound; round 7: 85 gr/ground.c accessors/setters/SFX slots/forwarders + 801C4B50/801C5AEC math; round 8: all 24 of gr/grlib.c) | 160 vs C: 27231 + 50454 + 58522 + 36065 + 26629 + 43280 + 20616 + 60000 rows (oracles contraction on + gwtool lowering, night 2); 25 camera accessors + 10 stage hook-driven fns unit-tested only, Stage_80224E64 untested; round 7: 4 ground fns vs C, 81 unit-tested only; round 8: PointInsideColl vs C (40000 rows), the other 23 grlib fns unit-tested or untested hook forwarders | melee_types (camera structs), melee_runtime::gekko::tanf; mp types hand-written | ef, rest of gr (rest of ground.c, stages), if; rest of cm and mplib |
| 3 fighter engine | melee_fighter | ~2820 | 489 (nh_Fighter, ftcommon movement, staling; port round 1: +39 ftcommon.c field logic; port round 2: +144 player.c accessors; port round 3: +96 ftlib.c; port round 4: +82 pl_040D.c + plbonuslib.c tail; port round 5: +34 all of ft_0877.c; port round 6: +18 ft_0881.c voice channels; port round 7: +19 all of ft_0892.c + 7 pltrick.c attack-stats defs) | 488 (6600 + 12000 + 64 + 10200 + 33200-line player table + 10440 ftlib cases + 15000 plbonus ops + 10200 ft_0877 cases + 5100 ft_0881 cases + 5700 ft_0892/pltrick cases; cref built with shipping contraction + lower.py) | melee_types (re-exported as `crate::types`) | everything else; c2r layer (feature `c2r`) compiles, never linked |
| 4 characters | melee_chara | ~4340 | 964 (13 pure helpers + 153 Fighter functions, round 1: `src/chars`; 554 thin callbacks, round 2: `src/chars/r2`, `gen/port_r2.py`; 134 callbacks/setters with field writes, stored callbacks and calls into rounds 1-2, round 3: `src/chars/r3`, `gen/port_r3.py`; 110 with conditions on fields / dat_attrs reads / else-if, round 4: `src/chars/r4`, `gen/port_r4.py`) | all 964: pure helpers + 153 functions by per-function hash over 4000 random fighters; the 554 round-2 callbacks over 256 fighters each, hashing fighter state plus a trace of every group-3 call (id + args) through recording stubs on both sides; round 3 (134, 256 fighters) also maps stored HSD_GObjEvent callbacks to their names on both sides; round 4 (110, 256 fighters) the same, 14 bitfields by value; oracle = game TUs with shipping contraction + gwtool lowering (`gen/chref.py`, docs/rust-port/group-chara.md) | melee_types (re-exported as `types_gen`); 16 group-3 calls stubbed in `melee_chara::stubs` (TODO(group3), panic when unset); 55 stored callback targets unported (`chars::cb`, TODO(group4), panic) | ~80%: callbacks with logic beyond stub calls, items/effects/HSD/jobj, callback setters, MotionState tables, 384 empty callbacks |
| 5 items/ty/vi | melee_items | ~5000 | 755: 9 Item-only + 29 leaf helpers (round 1, `helpers.rs`) + 103 Item accessors from it_26B1/it_2725/ithitbox/itmaplib/itcoll (round 2, `itlib.rs`) + 30 next-layer it_2725/it_26B1/it_279C/itanimlist/itcoll/item/ithitbox/itmaplib/itspawn/itzako functions (round 3, `itlib3.rs`) + 248 per-kind state callbacks from 112 `it/kinds` TUs (round 4, `itkinds.rs`; 138 generated from 5 shared one-statement shapes, 110 by hand) + 257 generated one-statement forwarders (round 5, `itfwd.rs`) + 79 hand-ported kind functions (round 5, `itkinds5.rs`) + item hash | 24 tests vs C oracle (contraction on + lower.py; 27 f32 + 2 f64 fused sites; 2 (round 1) not exercised by a vector, 1 (round 5 `it_2E5A_UnkMotion1_Anim`, f64) proven unobservable by an exhaustive f32 sweep; rounds 2-5 hash 189 Item fields per case, 104x48 + 32x48 + 249x48 + 344x48 cases; rounds 4-5 also every item-var word) | melee_types (re-exported as `layout`) | spawn/think/collision, kind callbacks that touch GObj/JObj/fighters/effects (~4,200 it functions), trophies, vi |
| 6 modes | melee_modes | 346 leaf fns found (gm/mn/if/ty/lb) + 158 pointer-arg leaves + 39 round-4 callers (`oracle/scan_r4.py`; 9 already group 1's) | 251 fns: round 1 72 leaves (gm 32, mn 17, if+ty 8, lb 15); round 2 26 callers/const-table readers (gm 21, mn 2, lb 3); round 3 123 pointer-arg leaves over melee_types records (gm 52, if+ty 22, lb 29, mn 20; `*_ptr.rs`); round 4 30 callers of rounds 1-3, incl. objects behind pointers (gm 11, lb 14, mn 5; `*_r4.rs`; `lbShadow_8000E9F0` for group 4) | all 251: 80030 scalar vectors + 30750 buffer vectors (`oracle_ptr`: seeded struct buffers, return + FNV hash of the bytes after; C cut for x86_64-windows-gnu -mno-ms-bitfields so bitfield-run records sit at melee_types' GC offsets; contraction on + lower.py; FMA discriminating for fe_ease, calc_dist_sq and 9 of round 3's 11 fused functions; NaN as any-NaN in rounds 1-2); round 4: 14300 vectors (`oracle_r4`: callees cut from the game C too, buffer links with pointer-normalised hashes; all 6 fused sites caught by unfused mutation, lb_8000D148 via generated boundary rows) | melee_types; melee_runtime::lbtrigf::atanf (lb_8000D008); local `GXColor`, `UnlockableCharacterData`, `gmClassicMatchup`, `FigaTree` (gc_layout); round 4: melee_runtime::lbtrigf::acosf, gekko::sqrtf, melee_types::inline accessors; local `lbAudioAx_UserData` | 13 bitfield-touching + 12 .c-defined-type pointer leaves, callers of round 4, nested-pointer leaves, global-state functions; detail docs/rust-port/group-modes.md |
| platform (step 1/5) | melee_platform | shim_pad.c PADRead, gc_adapter.c (+decode.h), gw_script_pad.c txt script, Aurora PADRead/PADClamp | PADStatus pipeline, adapter (WinUSB + HID, reader + hotplug threads, focus suspend/resume, rumble), Aurora pad arithmetic, txt pad script, winit 60 Hz loop, headless runner | PADClamp/ClampCircle/ClampTrigger all inputs + adapter decode 4000-report stream vs the C/C++ compiled natively (oracle/build.sh); real WUP-028 read over WinUSB on this machine | PADStatus from melee_types (GC 0x10) | WinUSB-less Linux adapter, Lua/live/bot pads, ctl_poll remap, stick dead-zone setting, gamepad rumble, Aurora mapping files |
| platform (step 2/5) | melee_gx | Aurora's GX: lib/dolphin/gx encoders (~45 GX* calls + GXInit), lib/gx regs.cpp BP/CP/XF decode + command_processor.cpp, shader.cpp + shader_info.cpp (TEV/XF -> WGSL, uniforms), gx.cpp populate_pipeline_config/blend/depth, gfx/texture_convert.cpp | API -> FIFO -> decoded GXState, display-list record/call, draw resolution (index buffers, array snapshots, uniforms), WGSL generation, all GC texture formats + TLUTs, wgpu offscreen renderer (push constants for Aurora's immediates) | vs Aurora itself (oracle/build.py links the shipping aurora_gx.lib): 145 texture cases bit-exact, 400 random ShaderConfigs WGSL byte-identical, 300 random GX API sequences -> identical ShaderConfig/pipeline/uniform bytes/WGSL, test display list rendered offscreen = Aurora's screenshot (0 of 307200 pixels differ, wgpu Vulkan and DX12 vs Dawn D3D11) | GXTexObj/GXTlutObj/GXLightObj/GXColor carry GcLayout with the SDK's GC offsets | pc/platform/shim_gx.c layer (indirect-stage guard, GXInitFogAdjTable, fog scope, GXProject, VI draw-done), EFB copies (GXCopyTex/Disp), palettes, frame interpolation, draw merging/caches, lines/points untested on GPU, swapchain/present |
| platform (step 3/5) | melee_audio | pc/platform/shim_ax.c: 35 AX entry points, ADPCM/PCM16/PCM8 decoder, mixer (SRC, ve, 9 mix targets, ITD, de-pop), AXFX reverb-std + delay, native clips, frame driver, volume/audio.cfg, WAV dump | all of it: `adpcm.rs` decoder, `ax.rs` 64-voice AXVPB pool + mixer + clips, `fx.rs`, `output.rs` (cpal device on an SPSC ring, linear resample to the device rate, device-clock / wall-clock pacing) | vs shim_ax.c itself, copied verbatim and built as shipping shims are (clang i686-pc-windows-msvc -O2, contraction on, no FMA on i686): 4000 decoder states x 64 samples + 30000-step random AX script (23.8k calls, 6182 sub-frames, 6001 non-silent; every frame's output + VPB writeback hashed, 64 compared sample by sample, reverb/delay/clips exercised); mutation checks fail it | VPB/PB*/AXFX offsets cross-checked against melee_types GcLayout (`layout.rs` test) | device output only run once here (cpal WASAPI 48 kHz opened, 0 underruns; not listened to); not wired to a synth/game loop; reverb-hi/chorus unimplemented (as in C); AI shims (shim_misc.c) only as fields; predictor nibble >7 masked (UB in C) |
| asm (10 items) | n/a | n/a | 0 | n/a | n/a | none belonged to groups 0-5; crash-screen register reads return 0 on PC |

### Done in integration
- Deduped the hand-written float/vector helpers into `melee_core`: `fp::{frsqrte, fres, force25}`
  (from melee_runtime::gekko; identical copies deleted from melee_world::gekko and
  melee_chara::pure) and `vec::{Vec2, Vec3}` (from melee_runtime::mtx and melee_world::vec, with
  GC layout + StateHash). Old paths re-export, so every crate's oracle tests now run on the moved
  code, and pass.
- `melee` bin: a headless frame driver (`Sim` trait -> `nh_game` -> NHV1 CSV writer).
  `melee hashlog --words G.csv --out C.csv` replays the words of an existing CSV through it; all 4
  goldens round-trip and `compare_golden.py` reports identical (3601/3601/1801/1801 frames). This
  checks the frame combine and the CSV format only. **It is not a simulation.**
- `Cargo.lock` committed in its current consistent state.

### Not done in integration
- (Fixed night 2, step 1: see "Night 2" below.) Types were in four generated copies and NHV1 in three.
- Sibling crates still do not depend on each other (deps were removed while other agents were
  mid-generation). They all build now; re-add deps as each group starts calling another.
- `PSVECNormalize` exists twice (melee_runtime::mtx::vec_normalize, melee_world::vec::ps_vec_normalize),
  each verified by its own oracle; not merged because the world oracle ran without contraction.

### Risks found while integrating
1. The golden check proves only `nh_game`'s combine; the per-fighter words still come from the C.
2. **Oracle flag mismatch.** Groups 2, 4 and 5 built their C oracles with `-ffp-contract=off`; group 1
   showed the real build contracts. Those results are bit-exact against uncontracted C, not
   necessarily against the game. Concrete case: `melee_chara::pure::my_sqrtf` computes
   `3.0 - g*g*x` unfused, which group 1 found is a double fma in the real build (math_ppc.h sqrtf).
   Re-run those oracles through `melee_runtime/oracle/lower.py` (contract on + gwtool lowering).
3. Float->int saturation, div-by-zero and oversize-shift rules live in `melee_runtime::cabi`, not core yet.

## Night 2: next steps, in order
1. **One type generator** (above), so Fighter/Item/HSD types exist once in melee_types with the
   GC offset table (the m-ex offset map). Point groups 3-5 at it and delete their copies.
2. **Re-run the group 2/4/5 oracles with contraction on** and fix any fused sites.
3. **Big-endian DAT/archive loader** driven by `GcLayout` (relocation table -> native structs with
   real pointers or indices). Without it nothing data-driven (fighters, stages) can run.
4. **HSD core** (GObj/JObj/AObj, process lists) in melee_runtime, then the frame loop: replace
   `WordReplay` with a real `Sim`; first target is `melee hashlog` matching `pad_falcon_fox_fd` frame 0.
5. **Platform layer** (new crate, e.g. `melee_platform`): winit window, wgpu for GX (port the GX/TEV
   state model the C build gets from Aurora), cpal for AX audio, gilrs for PAD, std::fs for DVD.
   Rendering is checked visually + render_test; logic stays on the hash oracle.
6. **m-ex importer** (offline tool, not in the game): read the user's m-ex disc once and write each
   asset to a gdmod file in the native format; mod PPC code is recompiled or interpreted, with field
   accesses remapped through the `GcLayout` "GC offset -> native field" tables. The game then boots
   from a vanilla disc plus those gdmods. No disc-derived output is ever committed.

## Night 2 (2026-10-21)

### Step 1: one type generator (done)
- `rust/melee_types/gen/gen_types.py` now writes its probe `gen/all_types.c` itself (`write_probe`):
  every baselib header, every `melee/*/forward.h` + `types.h`, all of `ft/*.h`, `ft/kinds/**`,
  `pl/*.h`, `it/**`, `ty/**`, `vi/**` (799 headers) -> 1618 records, 0 missing from the
  `-fdump-record-layouts-complete` cross-check. Every record also gets `X::zeroed()`.
- Deleted: `melee_fighter/gen/gen_types.py` + `types_tu.c` + `src/types/generated.rs`,
  `melee_chara/gen/gen_types.py` + `run_gen.sh` + `src/types_gen.rs`, `melee_items/tools/gen_layout.*`
  + `src/gen/`. Old paths (`melee_fighter::types`, `melee_chara::types_gen`, `melee_items::layout`)
  are `pub use melee_types::*` shims. Every record those three generated exists in melee_types
  (checked by name; only anonymous-member record names differ, e.g. `plActionStats.anon_358`).
- NHV1 is one implementation (`melee_types::neutral_hash`); `melee_fighter::neutral_hash` and
  `melee_items::hash` are thin wrappers, so group 3's `nh_ref` and group 5's oracle tests now check
  that one copy against the C. The fighter/items StateHash impls were dropped (melee_types has them).
- Changes in convention for callers: bitfields are `bf_<gcoff>: [u8; n]` in GC byte order (items'
  old one-integer form is gone), enums are int aliases (`GroundOrAir` = u32), fn pointers typed.
- Group 4's generator laid out GC offsets WITHOUT `-DTARGET_PC`; melee_types uses `-DTARGET_PC` for
  both parses. No `TARGET_PC` appears in `ft/kinds/*/types.h`, so character records are unaffected.
- Not done: the gitignored `melee_fighter` `c2r` feature (fix_raw.py output) still carries per-TU
  type copies; it was not rebuilt.

### Step 2: FMA contraction, groups 2/4/5 re-checked (done)
- Shipping build, verified from `_build/masstest/pipe_win.sh` (what `build_objects.py` runs per game
  TU): `_toolchains/llvm` clang 23.1.1, `--target=ppc32-none-eabi -O2 -Xclang -disable-llvm-passes`,
  no `-ffp-contract` flag, so clang's default `on`: every `a*b+c` within one expression is
  `llvm.fmuladd`. gwtool runs with `GW_GWTOOL_FLAGS` empty, so `--no-exact-fmuladd` is off and
  `lowerFMulAdd` turns every f32 site into fmadds (f64 mul+add, one f32 rounding), f64 into `fma`.
- Tool: `python tools/rust-port/fma_sites.py [-v] [--func RE] <tu.c>...` runs that exact front end
  and prints per-function fmuladd sites. Use it before porting any float code.
- Oracles rebuilt with contraction on + `melee_runtime/oracle/lower.py`:
  `melee_world/cref/gen_mplib_lines_ref.sh` (15 f32 sites lowered), `melee_chara/gen/run_ref.sh`
  (6 f64 fma), `melee_items/tests/c_oracle/gen_expected.sh` (0 sites). All three outputs came out
  bit-identical to the uncontracted ones: the existing inputs do not discriminate.
- Ports fixed to fuse where the shipping IR fuses:
  `melee_world::mp::lines::check_link` (`SQ(dx)+SQ(dy) < 4.0` in mpLineGetNext/Prev and the
  CheckInline variants: `fmadds(dx, dx, dy*dy)`); `melee_chara::pure::my_sqrtf` (all 7 copies:
  `(-(g*g)).mul_add(x, 3.0)`, true f64 fma); `melee_fighter::ftcommon::calc_hitlag`
  (`ftCommon_CalcHitlag`: `fmadds(dmg, x198, x19C)`; group 3, found by running fma_sites.py over
  its ported functions). Group 5's 9 ported item functions have 0 sites (checked).
- Not proven: since the oracle rows did not change, no test currently fails on the unfused form
  of those three sites; they are right by the IR, not by a discriminating vector.
  A 20M-input Rust experiment found no f32 result difference for my_sqrtf fused vs unfused.
- Still uncontracted: group 0's `gen/nh_oracle.py` (-ffp-contract=off). (Group 3's `gen/cref/run.py`
  was switched to contraction on + lower.py in group 3 port round 1; nh/stale/ftcommon unchanged.)
  NHV1 has no float arithmetic; group 3's ftcommon has 12 fused sites in ftcommon.c functions
  (`fma_sites.py src/melee/ft/ftcommon.c`), only CalcHitlag among the ported ones.

### Step 3: DAT/HSD archive reader (done, synthetic-tested)
- `melee_runtime::dat`: `DatArchive::parse` (port of `HSD_ArchiveParse`: header, data block,
  relocation table, public roots, externs, symbol strings; the C's file_size "byte-order
  mismatch" check; bad relocations rejected), `public_offset` / `extern_name`
  (`HSD_ArchiveGetPublicAddress` / `HSD_ArchiveGetExtern`), `load::<T>(root)` -> `Loaded<T>`
  (native root + `GcArena` owning every reached object + issue list). `DatBuilder` writes
  synthetic archives. The file is never patched in place (the C's `Locate`).
- `melee_core::gcdecode`: `GcDecode` / `GcPointee` + `GcDecoder`. Generated for all 1618 records
  by gen_types.py: each field decoded at its GC offset (scalars byte-swapped, bitfield bytes
  copied as-is, nested records recursive, pointers followed only through the relocation set,
  each (offset, type) decoded once, so cycles share objects; a work queue, not recursion).
- Unions: `HSD_Joint.u` (JOBJ_SPLINE / JOBJ_PTCL / dobj) and `HSD_RObjDesc.u` (ROBJ_TYPE_MASK) are
  hand-written in `melee_types/src/decode.rs` (gen `HAND_DECODE`). Other unions: pointer member
  if the first word is relocated, else first scalar member (heuristic).
- Limits (left zero + reported in `issues`): `void*` targets (e.g. HSD_ImageDesc pixel data,
  display lists: need a typed hand decoder), function pointers, and "pointer to first element"
  arrays (only element 0 decodes; `GcDecoder::array(target, n)` when the count is known, e.g.
  `MtxPtr` = `f32 (*)[4]` decodes one row).
- Tests: `cargo test -p melee_runtime --test dat` (5 synthetic: header/tables, HSD_Joint graph
  with cycle/strings/dobj/matrix, shared decoder, malformed files, unrelocated word).
  Opt-in disc check, run locally on the user's vanilla ISO (nothing written or committed):
  `GW_ISO_VANILLA=... cargo test -p melee_runtime --test dat -- --ignored --nocapture`:
  PlFxNr.dat/PlCaNr.dat parse, joint trees decode (Fox 73 joints / 1300 objects, Falcon 63 /
  1290), no out-of-bounds; ~350 issues each, mostly void* image data and words that some
  pointer-to-array or mis-typed pointer reaches as the wrong type (e.g. an HSD_ExpDesc reached
  as an HSD_Joint). Values were not compared against the C loader's output: no C-vs-Rust oracle
  for loaded data yet. GrNBa/GrNLa parse but have no `*_joint` roots (stage roots are map_head etc).

## Platform track, step 1: melee_platform (2026-10-21)
- Crate `rust/melee_platform`: `pad` (PADStatus = `melee_types::PADStatus`, constants, Aurora's
  PADClamp/PADClampCircle), `adapter` (port of `pc/platform/gc_adapter.c`: reader thread holding the
  newest report, hotplug scanner, focus-following ownership, `MELEE_PAD_IGNORE_ADAPTER`/`MELEE_UNATTENDED`,
  recalibrate, rumble-on-change; transports WinUSB then HID like the C; `decode` = gc_adapter_decode.h),
  `aurora` (Aurora's PADRead per-port arithmetic: keyboard bindings, standard button map, default
  axis map + dead zones, trigger emulation), `gamepads` (gilrs -> SDL-convention raw values,
  connection-order ports), `script` (`MELEE_PAD_SCRIPT` .txt + `MELEE_PAD_CHANNELS`), `pad_read`
  (`PadSystem::read` = shim_pad.c `gw_PADRead` order: Aurora layer, mode-NONE/suspended disconnect,
  F9 recal, adapter override, idle port 0, script, L+R+Y+X+Start combo -> Aurora's no-op
  PADRecalibrate), `main_loop` (winit 0.30 window; exact 60 Hz fixed step in integer ns*60, max 5
  catch-up ticks; present gets alpha; 100 ms focus-loss debounce; `run_headless`).
- The game calls `PadSystem::read` itself inside `Game::tick` (as HSD_PadRenewRawStatus calls PADRead).
- `button` is native-order u16: the C's bswap16/gw_w16 exist only for big-endian game memory.
- Verified: `cargo test -p melee_platform` (oracle: 3 clamp functions over every input, decode over a
  4000-report LCG stream with recal, equal to `oracle/oracle.cpp` built from the shipping sources;
  pipeline/adapter-lifecycle/script/clock tests with a fake device). `cargo run -p melee_platform
  --example window -- 4` opened a window, ran 240 ticks in 4.03 s with 0 dropped, and read a real
  WUP-028 (Zadig/WinUSB) controller in port 1. Not verified: the HID transport on hardware (this
  machine's adapter is WinUSB-bound), gilrs gamepads and keyboard on hardware (no gamepad attached;
  keyboard map is unit-tested only), Aurora gamepad arithmetic against the C++ (hand-checked values
  only; SDL axis values vs gilrs floats are a conversion, not identical device readings).
- Not ported: Linux adapter (`gc_adapter_linux.inc`), Lua/live/bot pads and gd.input claims,
  ctl_poll remapping, stick dead-zone setting, gamepad rumble, Aurora per-device mapping files and
  per-type default maps, diagnostics/overlay/profiling. No rendering yet (step 2: wgpu).

## Platform track, step 2: melee_gx (2026-10-21)
- Crate `rust/melee_gx`: a port of the GX the C build gets from Aurora (`extern/aurora`), same
  architecture: `api::Gx` = the GX API encoders of `lib/dolphin/gx/*.cpp` + the `__GXData` shadow
  registers, writing BP/CP/XF/`GX_AURORA` commands into a big-endian FIFO (or a display list);
  `state::GxState::process` = `regs.cpp` + `command_processor.cpp` decode (BP mask, register dedupe,
  cull swap, tref channel remap, VAT/VCD, XF matrices/lights/viewport/projection); draws become
  `draw::DrawCmd` (raw vertex bytes, Aurora's index buffers, array snapshots, uniform bytes).
  `shader` = `shader.cpp`/`shader_info.cpp`/`attr_fmt.cpp`/`populate_pipeline_config`, generating
  Aurora's WGSL text byte for byte; `texture` = `texture_convert.cpp` (I4/I8/IA4/IA8/RGB565/RGB5A3/
  RGBA8/CMPR, C4/C8/C14X2 + IA8/RGB565/RGB5A3 TLUTs, arbitrary-mip check); `render::Renderer` =
  wgpu 27 offscreen target with Aurora's resource model (vbuf/abuf storage, 256-aligned dynamic
  uniforms, 64-byte immediates as push constants, reversed Z, CW front faces, Aurora's blend/depth
  mapping). Pointers in the FIFO are handles (`resources`); game data stays GC big-endian bytes.
- Verified (`cargo test -p melee_gx`), against Aurora's own code, not a re-implementation:
  `oracle/build.py` compiles `oracle/oracle.cpp` with the flags CMake used for aurora_gx and links
  the shipping Aurora build (`_build/agents/rustgold/ax86m`, MSVC x86), then writes tests/data:
  `tex_oracle.txt` (145 decode cases, FNV of the output), `shader_oracle.txt` (400 random configs,
  WGSL length + FNV + uniform size), `state_oracle.txt` (GXInit state + 300 random sequences of 36
  kinds of GX call; ShaderConfig, pipeline state, uniform-byte and WGSL hashes) and
  `render_aurora.png` (Aurora's screenshot of `oracle/scene.inc`: a 2-stage-TEV textured quad /
  strip / fan from an INDEX8 S16 position array, and a per-vertex-lit alpha-blended pair of
  triangles, both via GXCallDisplayList). All match exactly; tests/render.rs draws the same scene
  and gets 0 differing pixels on wgpu Vulkan and DX12 (RTX 5070) vs Dawn D3D11. Negative check: a
  one-character change in the generator fails 251/400 WGSL cases and the state test.
- WGSL for wgpu (`render::adapt_wgsl`, applied after generation, so the oracle compares Aurora's
  text): `var<immediate>` -> `var<push_constant>`; naga rejects `ptr<storage, ...>` function
  parameters, so they become a buffer selector (0 = vbuf, 1 = abuf); naga does not fold builtins
  over abstract floats (`mix(0.0, 0.0, 0.0)` in an all-constant alpha combiner), so literals on alpha
  combiner lines are typed `f32(...)` (exact values).
- Traps: the game calls GXInit (aurora_initialize does not); Aurora skips a draw while its pipeline
  compiles asynchronously (the oracle screenshots at frame 60); this Dawn build has no Vulkan and its
  D3D12 swapchain faults in `WaitForQueueSerial` in the oracle harness on this machine (oracle uses
  D3D11); `aurora_shutdown` faults after the device is destroyed (oracle `_Exit`s); Aurora writes its
  pipeline-coverage db to %LOCALAPPDATA%\GD Melee while the oracle runs.
- Not ported / not verified: the `pc/platform/shim_gx.c` layer above Aurora (indirect-stage guard,
  GXInitFogAdjTable, fog scope, GXProject/GXGet*, VI draw-done routing, draw suppression during
  resimulation), EFB copies (GXCopyTex/GXCopyDisp; logged), PC palette / surface programs / frame
  interpolation / motion capture, Aurora's draw merging and pipeline/uniform/texture caches (every
  draw recomputes; the renderer caches pipelines, samplers and decoded textures), texture
  replacements, swapchain/present (step 3 joins this to melee_platform's winit loop), MSAA.
  Lines/points, fog, indirect texturing and emboss are WGSL-verified but not rendered by a test.
