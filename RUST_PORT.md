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

## Group 3: fighter engine + ftCommon + pl (2026-10-21; detail: docs/rust-port/group-fighter.md)
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
| 1 runtime/HSD | melee_runtime | 2297 (recounted night 3: distinct IR definitions over baselib, lb, db, sfx, gekko_fp, mtx_pc, mtx44) | 516 = 447 checked vs C + 69 not (night 3 round 14: pobj.c logic 17 (GetFlags, Remove/Add/ReqAnim[All][ByFlags], PObjAnim[All], PObjUpdateFunc, ShapeSetSetAnimResult, Clear/Set/GetMtxMark, decode_u8/s8/u16/s16_xyz), all checked, oracle_pobj 20.5k rows over real aobj/fobj, 0 fused sites, 7 mutations caught; recount at start 499, no stub replaceable; night 3 round 13: mobj.c logic 7 (SetCurrent, Set/ClearFlags, MObjUpdateFunc, SetDiffuseColor, SetAlpha, GetTObj), all checked, oracle_mobj 20k rows, 4 mutations caught; recount at start 492, no stub replaceable; night 3 round 12: tobj.c logic 12 (9 GX id/index maps, DifferentTluts, MakeTextureMtx, TObjUpdateFunc), all checked, oracle_tobj 20k rows; recount at start 480 = 439 + the particle work efa84f09f committed: generator.c 17 + psappsrt.c 6 checked via oracle_generator, psAddGeneratorAppSRT[_begin] + 16 particle.c fns not checked per function (golden seed frames 0-63 only); night 3 round 11: fog.c logic 12, 10 checked (Alloc/AdjAlloc only via LoadDesc), oracle_fog 8k rows, recount at start 427; generator.rs/particle.rs/psappsrt.rs in the tree are another agent's uncommitted work, not counted; night 3 round 10: lobj.c logic 41, all checked, oracle_lobj 24k rows; recount at start 386, uncommitted particle/generator work in the tree not counted; night 3 recount: 316 before round 9; gekko_fp, sqrtf, mtx, mtx44, random, lbtrigf, lbvector; round 1: baselib mtx.c 14/22, quatlib 6/6, spline 11/11; round 2: fobj.c 24/30 keyframe interpreter, util.c 3/3, crypt.c 5/5; round 3: GObj system 36 (gobjplink/gxlink/proc/object/userdata/init all, gobj.c 10/16); round 4: aobj.c 18, id.c 5, list.c 6 = 29; round 5: jobj.c 27 (tree, flags, dirty propagation, MakeMatrix, SetupMatrixSub; IK solvers stubbed); round 6: wobj.c 13 + dobj.c flags 5 = 18; round 7: robj.c 38 + mtx.h ColMag/ColVec = 40; round 8: jobj.c 23 (JObjUpdateFunc, anim walkers, resolveIKJoint1/2, RObj wired into JObjHost via `jobj_rt::JObjRt`); round 9 (night 3): bytecode.c HSD_ByteCodeEval, lb_00CE expf/powf/powi, cobj.c logic 66 (62 checked), docs/rust-port/group-runtime.md) | all: 11400 rows + 33000 rows (oracle_hsd, 66k checks) + 65000 rows (oracle_anim, 135k checks) + GObj transcript (oracle_gobj: 40 seeds x 400 random ops, 55.8k lines, 16k state hashes); oracle_aobj (34.7k rows, 32k checks, fmodf fusion discriminated); oracle_jobj (22k random ops, state hash after each); oracle_wobj (15k random ops over WObj/JObj/spline/DObj flags, 1197 path updates); oracle_robj (400 scenarios, 16.6k checks: LoadDesc/ResolveRefs/UpdateAll constraint solver, 22 of 40 fns checked, ColMag fusion discriminated, NaN hashed canonical); oracle_ik (400 forests x 50 ops, 20.4k checks: IK solvers, effector, RObjUpdateAll + real JObjUpdateFunc; 7 of round 8's 23 fns checked, the IK1 FMA site not discriminable since 2*x is exact); oracle_bytecode (48k checks: 20k random stack programs + expf/powf/powi); oracle_cobj (500 cameras x 60 ops, 30k rows, state hash; roll2upvec fusion discriminated); oracle_tobj (20k rows, state hash per update, 0 fused sites, 5 mutations caught); oracle_mobj (20k ops, whole-MObj hash, 0 fused sites); oracle_fog (8000 ops: GX call log + whole fog/adj/AObj state, 0 fused sites, 6 mutations caught, range>640 clamp boundary not hit); oracle redoes gwtool fusion + safe shifts | Vec3/Quaternion; GObj/GObjProc/LibInitData/DelayedProcInfo delegate to melee_types; DAT reader `dat.rs` (night 2) | HSD objects (rest of JObj: load/disp/class, rest of DObj, PObj load/envelope/display, MObj anim/load/TEV, CObj display/anim/load, LObj GX/anim/load, Fog class/release, TObj anim/load/GX setup, ~1000; AObj ForeachAnim), gobj.c render callbacks, lb/db/sfx/THP |
| 2 world | melee_world | 3015 (night 3 r6 recount; r5's 3014 was one short in if/) | 1169 (night 3 r6: 18 if/textlib.c DevText fns (EraseFirstLine, SetCursorXY/X, 6 flag setters, SetScale, SetXY, StoreColorIndex, SetTextColor, SetBGColor, Erase, AdvanceLine, Print, PrintInt) + textlib_1 adjust + grpstadium mul_vec; night 3 r5: 17 ground.c / per-stage getters (MexParamStkind, 801C2AD8/2AE8, GetMapGObj, 801C2BBC/2BD4/2C8C/2CF4/2D0C, 801C20D0, fog colour 801C205C/2090, 801C498C, 801C1D38, grRCruise_80201918, grBigBlue_801EF7D8, grIceMt_801FA728) + 3 GrLibHooks TODO stubs replaced by them; night 3 r4: 220 single-call gr forwarders (`gr::forward`, generated: grZakoGenerator_801CAE04(NULL) / Ground_UpdateMapColl / StartMapAnim / InitMapCollAndAnim / UpdateWindAndMapColl; callees via `forward::CALLEES`, TODO panics); night 3 r3: 568 constant-return/empty per-stage gr callbacks (`gr::trivial`, generated) + efLib_SetFlags/PauseAll/ResumeAll (`ef::eflib`); 344 of 2447 excluding the trivial callbacks; before: mplib line queries, line geometry, segment checks + bounding, joint list edits + mpLib_80056C54, seam stitching + floor bounds; all of mpisland.c; round 5: 42 cm/camera.c framing/follow math, quakes, accessors; round 6: all 45 of gr/stage.c incl. slot select + ArenaBound; round 7: 85 gr/ground.c accessors/setters/SFX slots/forwarders + 801C4B50/801C5AEC math; round 8: all 24 of gr/grlib.c; night 3 r1: 24 more camera.c fns (x399/x39A accessors, 8002FE38, 8002EEC8, 8002BC78 + OrthonormalizeBasis, PC cm_script helpers); night 3 r2: 16 gr/if leaves (grGreens/grOldPupupu box tests, grStadium_ScaleColor, grMc_DistanceSquared, grKongo/grOldKongo barrel step, DevText_StrLen/NumToStr/Clamp, ifStock digit helpers, ifnametag inlineA1, ifstatus foo) + 3 hook stubs replaced) | 1053 vs C (night 3 r6: +20, textlib_r6_ref 84159 rows incl. full glyph buffer, no fused sites, mutation-checked; night 3 r5: +17, ground_r5_ref 21400 rows, no fused sites, mutation-checked; night 3 r4: +220, forward_r4_ref 1320 call traces over 6 map_ids, mutation-checked, no float math; night 3 r3: +571, trivial_r3_ref 1136 rows (zero + junk args) + eflib_r3_ref 3000 random list pairs, no fused sites; night 3 r2: +16 via leaves_r2_ref, 70269 rows, 2 fused sites mutation-checked; night 3 r1: +49 via camera_acc_ref, 8640 cases, incl. the 25 night-2 camera accessors; 8002BC78 NaN-sign caveat); night 2: 27231 + 50454 + 58522 + 36065 + 26629 + 43280 + 20616 + 60000 rows (oracles contraction on + gwtool lowering, night 2); 25 camera accessors + 10 stage hook-driven fns unit-tested only, Stage_80224E64 untested; round 7: 4 ground fns vs C, 81 unit-tested only; round 8: PointInsideColl vs C (40000 rows), the other 23 grlib fns unit-tested or untested hook forwarders | melee_types (camera structs), melee_runtime::gekko::tanf; mp types hand-written | ef, rest of gr (rest of ground.c, stages), if; rest of cm and mplib |
| 3 fighter engine | melee_fighter | 3002 (recount, round 9: incl. static inline helpers; unchanged round 10) | 684 (round 14: +10: ftCo_DamageFall.c ftCo_80090574/calcShift/80090594/80090690/80090718/DamageFall_Anim/DamageFall_Phys, ftParts_GetBoneIndex/ftPartsRemap/ftParts_8007506C, `src/ftdmgfall.rs`, both fused sites discriminated; FTCO_80090690 + FTPARTS_GET_BONE_INDEX hooks now default to the ports; recount at start 674 of 3002, unchanged; no other-group stub replaceable; round 13: +40: all of ft_0DF0.c (smash charge) + ft_0DF1.c (C-stick tests, incl. static inline canUseCstick), ftColl_8007AEE0/AEF8/AF10/AF28/B8A8/B8CC, ftCo_800C0658/0674/0694, ftParts_80074A4C/A74/A8C/ACC, `src/ft0df.rs`; FTCO_800DEEE8 hook now defaults to the port, new IT_8026B30C hook defaults to melee_items::itlib; DEEB8 fused site discriminated, DF608's not; recount at start 634 of 3002, unchanged; round 12: +19: all 14 of ft_084E.c, ft_80084CB0/DB0 (ft_081B.c), ft_8008A1FC/A244(+inline) (ft_08A1.c), `src/ft084e.rs`, 3 fused sites discriminated; new global hooks::FIGHTER_804D6520; no TODO hook replaceable; round 11: +15 plbonus.c primitives setFlag..resetBonuses, `src/plbonus3.rs`, PL_80038788/8824/8898/9418/86D8 hooks now default to them; round 10: +24 plbonuslib.c pl_8003DF44..pl_8003ED0C, `src/plbonus2.rs`; IT_80284FC4/85024 and HSD_JObjGetTranslation/MtxIsDirty hooks now default to melee_items::itkinds5 / melee_types::inline; nh_Fighter, ftcommon movement, staling; port round 1: +39 ftcommon.c field logic; port round 2: +144 player.c accessors; port round 3: +96 ftlib.c; port round 4: +82 pl_040D.c + plbonuslib.c tail; port round 5: +34 all of ft_0877.c; port round 6: +18 ft_0881.c voice channels; port round 7: +19 all of ft_0892.c + 7 pltrick.c attack-stats defs; port round 8: +37 ftcommon.c landing/nudge/item+parasol callbacks; port round 9: +50 leaves: 29 ftaction.c skip/flag handlers, 10 fighter.c incl. UCF pad ring, 4 ftanim.c, 4 ftcoll.c, 3 ftcommon.c atan2f stick angles; 7 IT_* hook stubs now default to melee_items::itlib) | 683, ftCommon_8007E2D0 unverified (ftdmgfall 15000 lines, round 14; ft0df 24000 lines, round 13; ft084e 7600 lines, round 12; plbonusc 15000 lines, round 11; 6600 + 12000 + 64 + 10200 + 33200-line player table + 10440 ftlib cases + 15000 plbonus ops + 10200 ft_0877 cases + 5100 ft_0881 cases + 5700 ft_0892/pltrick cases + 10200 ftcommon3 cases + 10000 ftleaf cases; plbonus now 104 ops / 15000 lines, ED0C's fmadds discriminated; cref built with shipping contraction + lower.py) | melee_types (re-exported as `crate::types`); melee_items (item hook defaults), melee_runtime (lbtrigf atan2f) since round 9 | everything else; c2r layer (feature `c2r`) compiles, never linked |
| 4 characters | melee_chara | 4605 (recounted night 3: definitions in the 232 non-ftCommon `ft/kinds` TUs) | 1612 (night 4 round 3: 8 functions (Fox 2, Seak 6) via `Fighter_SetDamageCallback(gobj, cb)` expanded in place and nested `{ Fighter* fp2 = GET_FIGHTER(gobj); ... }` / declaration-free bare blocks flattened, `src/chars/r14`, `gen/port_r14.py`, 1599/1599 equal, 0 fused sites; itIsHeavy stub now defaults to group 5's verified itlib port (via melee_fighter::hooks); night 4 round 2: 12 `_Phys` callbacks calling 9 group-3 ports newly reachable through `stub_or!` hooks (ftColl_8007AEE0/AF10, ft_80085004/5088/50B4/5154/51C0/51D0/5204), `src/chars/r13`, `gen/port_r13.py`, 1591/1591 equal, 0 fused sites; ftParts_80074A4C / ftColl_8007AEF8 stubs now default to group 3's verified ft0df ports; night 4 round 1: 101 functions via a (tu, name)-keyed ported filter (same-named statics such as ftDonkey `setCallbacks` were skipped since round 1), `fp = gobj->user_data` tops and header fp-store inlines expanded, `src/chars/r12`, `gen/port_r12.py`, 1579/1579 equal, 0 fused sites; ft_80084F3C/EEC/5134/DB0/FA8 stubs now default to group 3's verified ft084e ports; night 3 round 3: 79 functions via `static u32 const mf` masks inlined, `ftGetFacingDirInt(fp)` args, `fp->f & K` conditions, Fighter_SetEffectHitlagCallbacks / ftKb_SpecialN_set_cbs expanded, new ftCliffCommon_80081298 stub, `src/chars/r11`, `gen/port_r11.py`, 1478/1478 equal, 0 fused sites; Fighter_UnkSetFlag_8006CFBC stub now defaults to group 3's verified port; night 3 round 2: 10 functions unlocked by C-text rewrites (nested `{ Fighter* fp = GET_FIGHTER(gobj); }` blocks flattened, `T* x = x = e` initializers, `ca = &fp->co_attrs` locals, `a = b = 0` chains), `src/chars/r10`, `gen/port_r10.py`, 1399/1399 equal, 0 fused sites; 4 stubs (ftCommon_8007D7FC/DB24/D92C/EBAC) now default to group 3's verified ports; night 3 round 1: 16 Master/Crazy Hand functions reading `fp->ft_data->ext_attr` locals, `src/chars/r9`, `gen/port_r9.py`, oracle points ft_data at a reversed attrs block, 1389/1389 equal; 13 pure helpers + 153 Fighter functions, round 1: `src/chars`; 554 thin callbacks, round 2: `src/chars/r2`, `gen/port_r2.py`; 134 callbacks/setters with field writes, stored callbacks and calls into rounds 1-2, round 3: `src/chars/r3`, `gen/port_r3.py`; 110 with conditions on fields / dat_attrs reads / else-if, round 4: `src/chars/r4`, `gen/port_r4.py`; 191 calling 16 newly stubbed group-3 functions, group 3's ported ftCommon_8007E2F4/E2FC/UpdateFacing, and HSD_GObjEvent / s16 arguments, round 5: `src/chars/r5`, `gen/port_r5.py`; 108 OnItemVisible/Invisible hooks, ClampAirDrift/ApplyPartAnim/efLib callers and `fp->x--` countdowns, round 6: `src/chars/r6`, `gen/port_r6.py`; 37 with float field arguments, anonymous-union throw_flags and 3 more bitfields, round 7: `src/chars/r7`, `gen/port_r7.py`; 84 Master/Crazy Hand IASA `Player_GetPlayerSlotType == Human` checks + 2 PAD_STACK-blocked callbacks, round 8: `src/chars/r8`, `gen/port_r8.py`) | all 1612 (night 4 r3: 1599 functions + 13 pure helpers; night 4 r2: 1591 functions + 13 pure helpers; night 4 r1: 1579 functions + 13 pure helpers; night 3 r3: 1478 functions + 13 pure helpers, chars_ref): pure helpers + 153 functions by per-function hash over 4000 random fighters; the 554 round-2 callbacks over 256 fighters each, hashing fighter state plus a trace of every group-3 call (id + args) through recording stubs on both sides; round 3 (134, 256 fighters) also maps stored HSD_GObjEvent callbacks to their names on both sides; round 4 (110, 256 fighters) the same, 14 bitfields by value; round 5 (191, 256 fighters) also traces callback arguments by name; round 6 (108, 256 fighters) 7 more recorders; round 7 (37, 256 fighters) 17 bitfields by value, whole-word throw_flags writers hashed as bytes; round 8 (86, 256 fighters) Player_GetPlayerSlotType recorder; oracle = game TUs with shipping contraction + gwtool lowering (`gen/chref.py`, docs/rust-port/group-chara.md) | melee_types (re-exported as `types_gen`); 41 calls stubbed in `melee_chara::stubs` (night 3 round 3: +ftCliffCommon_80081298 TODO(group3), Fighter_UnkSetFlag_8006CFBC falls back to group 3; night 3 round 2: 4 of them fall back to group 3's verified ftcommon2/3 ports when unset; 38 TODO(group3), 1 TODO(group5) itIsHeavy, 1 TODO(ef) efLib_DestroyAll; panic when unset); 137 stored/passed callback entries in `chars::cb` (88 TODO(group4) panics, 49 call ports), TODO(group4), panic) | ~80%: callbacks with logic beyond stub calls, items/effects/HSD/jobj, callback setters, MotionState tables, 384 empty callbacks |
| 5 items/ty/vi | melee_items | 4966 (recount: unique function names defined in it/ty/vi `.c`, same regex as the scans) | **1567** (night 3 round 6: recount 1541 at start (it 1537, ty 4, vi 0), + 26 hand ports in `itkinds14.rs` (it_80285314, get_bone_by_id, 3 itanimlist cmd handlers it_80279768/720/744, it_8028B08C, itYoshistar_UnkMotion0_Phys, it_80291D38, it_802AB9C0, likelikeVelocity, itSamusbomb_UnkMotion2_Phys, itNesspkfirepillar_Logic24_DmgReceived, it_802B22B8_inline, itPikachutjoltair_UnkMotion0_Anim_inline, itPikachuthunder_UnkMotion2_ScaleCall, fn_80280974_inline, it_8026C47C, 2 empty fns, 7 getters), `tests/oracle_round14.rs`, 28 tables x 48, 3 f32 fused sites all mutation-caught; no TODO stub replaceable. Night 3 round 5: 1541, recount 1512 at start (it 1508, ty 4, vi 0), + 29 hand ports in `itkinds13.rs` (7 `*_Coll` forwarders, itSwapVelocity/swapVelocity/it_802D9B78, itFlipper_Repel, itFreeze_ClearLinkedItem/Logic17_Destroyed, itMball/itHeart/itTomato destroyed hooks, S-Scope/Lipstick/Blizzard/Sword/Kinoko/Dosei attribute readers, checkHitLag, it_ResetStageArticleTable, it_802BDA40, itZrshell_UnkMotion4_Anim, 2 getters), `tests/oracle_round13.rs`, 29 tables x 48, 0 fused sites, 17 mutations caught; sign-bit cancellation trap hit again and fixed. Night 3 round 4: 1512, recount 1497 at start (it 1493, ty 4, vi 0), + 15 hand ports in `itkinds12.rs` (it_8026B1D4, it_8026C258, Item_8026784C, it_8026C1D4, it_80272C90, it_802746F8, it_80274F28, it_80277544, it_802775F0, it_8027B070 + 5 global-counter accessors), `tests/oracle_round12.rs`, 21 tables x 48, 6 f32 fused sites all mutation-caught, 2 mutations proven/argued unobservable; log now in `docs/rust-port/group-items.md`. Night 3 round 3: recount 1491 at start, + 6 hand ports in `itkinds11.rs` on jobj.h getters / sqrtf_accurate / HSD_Randi (it_80272CC0, it_80274990, it_80271B60, it_80271F78, itLinkArrow_802A850C_inline_2, calc_dist_2d_accurate), `tests/oracle_round11.rs`, 7 tables x 48, 1 fused site mutation-caught, 2 boundaries by inspection; hash sign-bit cancellation trap found and fixed for round 11. Night 3 round 2: recount 1468 at start, + 23 hand ports in `itkinds10.rs` whose only unported callees were group 1's native lbVector_* / atan2f / sinf / cosf (+ RNG), `tests/oracle_round10.rs`, 25 tables x 48, 9 fused sites all mutation-caught; 2 boundary compares correct by inspection only). Before: 1468 by recount (it 1464, ty 4, vi 0; a name counts when its verbatim C copy is in the oracle; 1442 before round 9 on this rule, the earlier 1446 also counted header / duplicate inlines) = 1442 + round 9: 26 hand ports whose only unported callee was `HSD_Randi`/`HSD_Randf` (melee_runtime::random::HsdRand, passed in) or `__frsqrte` (`itkinds9.rs`, `tests/oracle_round9.rs`, 29 tables x 48, 12 new f32 fused sites + f64 Newton fmas, game random.c compiled into the oracle). Earlier rounds: 1446 (813 + 125 `return true;`/`return;` callbacks + round 8: 501 generated empty / `return false;` callbacks, `itfwd8.rs`, and 7 hand getters, `itkinds8.rs`): 9 Item-only + 29 leaf helpers (round 1, `helpers.rs`) + 103 Item accessors from it_26B1/it_2725/ithitbox/itmaplib/itcoll (round 2, `itlib.rs`) + 30 next-layer it_2725/it_26B1/it_279C/itanimlist/itcoll/item/ithitbox/itmaplib/itspawn/itzako functions (round 3, `itlib3.rs`) + 248 per-kind state callbacks from 112 `it/kinds` TUs (round 4, `itkinds.rs`; 138 generated from 5 shared one-statement shapes, 110 by hand) + 257 generated one-statement forwarders (round 5, `itfwd.rs`) + 79 hand-ported kind functions (round 5, `itkinds5.rs`) + 5 second-order/cast forwarders and 125 trivial callbacks (round 6, generated `itfwd6.rs`) + 40 hand-ported kind functions and static-inline helpers (round 6, `itkinds6.rs`) + 1 generated forwarder and 12 hand ports: hookshot vector normalisers, bisectValue, it_2F28 deceleration, SpawnItem fill, small inlines (round 7, `itfwd7.rs`, `itkinds7.rs`; first use of MSL `sqrtf` via melee_runtime::gekko) + item hash | 28 tests vs C oracle (contraction on + lower.py; 45 f32 + 2 f64 fused sites (+ math_ppc.h sqrtf's f64 sites, round 7); 2 (round 1) not exercised by a vector, 1 (round 5 `it_2E5A_UnkMotion1_Anim`, f64) proven unobservable by an exhaustive f32 sweep; rounds 2-5 hash 189 Item fields per case, 104x48 + 32x48 + 249x48 + 344x48 + 176x48 cases; rounds 4-6 also every item-var word; every round-6 and round-7 fused site caught by un-fusing it; round 8: 508x48 cases, no fused sites; round 7 also checks MSL sqrtf vs CRT) | melee_types (re-exported as `layout`); melee_runtime::gekko::sqrtf (round 7) | spawn/think/collision, kind callbacks that touch GObj/JObj/fighters/effects (~4,200 it functions), trophies, vi |
| 6 modes | melee_modes | **group total 3995 fns** in the 166 gm/mn/if/ty/lb TUs (night 3 count, incl. ~20 lbtrigf/lbvector fns group 1 owns); 346 leaf fns found (gm/mn/if/ty/lb) + 158 pointer-arg leaves + 39 round-4 callers (`oracle/scan_r4.py`; 9 already group 1's) | 251 fns: round 1 72 leaves (gm 32, mn 17, if+ty 8, lb 15); round 2 26 callers/const-table readers (gm 21, mn 2, lb 3); round 3 123 pointer-arg leaves over melee_types records (gm 52, if+ty 22, lb 29, mn 20; `*_ptr.rs`); round 4 30 callers of rounds 1-3, incl. objects behind pointers (gm 11, lb 14, mn 5; `*_r4.rs`; `lbShadow_8000E9F0` for group 4); night 3 round 1: 10 nested-pointer leaves (lb 3, if+ty 5, mn 1, gm 1; `*_r5.rs`); night 3 round 2: 8 (callers of round 5 + leftover leaves: hud 3, lb 3, gm 2; `*_r6.rs`); night 3 round 3: 6 memzero callers (hud 1, gm 5; `*_r7.rs`); night 3 round 4: 5 nested-pointer lb leaves (`lb_r8.rs`); night 3 round 5: 8 (lb_80011C18 + getters/setters over .c-defined records; `*_r9.rs`) ; night 3 round 6: 10 (lbrefract texel callbacks 4, lbaudio_ax fn_80025E38, gm leaves over MatchEnd/SObj/BracketEntry 5; `*_r10.rs`) -> **298 of 3995** (recounted: 288 at start) | all 261: 80030 scalar vectors + 30750 buffer vectors (`oracle_ptr`: seeded struct buffers, return + FNV hash of the bytes after; C cut for x86_64-windows-gnu -mno-ms-bitfields so bitfield-run records sit at melee_types' GC offsets; contraction on + lower.py; FMA discriminating for fe_ease, calc_dist_sq and 9 of round 3's 11 fused functions; NaN as any-NaN in rounds 1-2); round 4: 14300 vectors (`oracle_r4`: callees cut from the game C too, buffer links with pointer-normalised hashes; all 6 fused sites caught by unfused mutation, lb_8000D148 via generated boundary rows); night 3 r1: 2650 vectors (`oracle_r5`, r4 harness; no float math); night 3 r2: 3500 vectors (`oracle_r6`; my_fmodf's 1 fused site caught by unfused mutation, 912/1500); night 3 r3: 1800 vectors (`oracle_r7`; no float math); night 3 r4: 1950 vectors (`oracle_r8`; no float math; mutation-checked); night 3 r5: 2250 vectors (`oracle_r9`; no float math; mutation-checked); night 3 r6: 4950 vectors (`oracle_r10`; 0 fmuladd, saturating fptoi; mutation-checked) | melee_types; melee_runtime::lbtrigf::atanf (lb_8000D008); local `GXColor`, `UnlockableCharacterData`, `gmClassicMatchup`, `FigaTree` (gc_layout); round 4: melee_runtime::lbtrigf::acosf, gekko::sqrtf, melee_types::inline accessors; local `lbAudioAx_UserData` | 13 bitfield-touching + 12 .c-defined-type pointer leaves, callers of round 4, nested-pointer leaves, global-state functions; detail docs/rust-port/group-modes.md |
| platform (step 1/5) | melee_platform | shim_pad.c PADRead, gc_adapter.c (+decode.h), gw_script_pad.c txt script, Aurora PADRead/PADClamp | PADStatus pipeline, adapter (WinUSB + HID, reader + hotplug threads, focus suspend/resume, rumble), Aurora pad arithmetic, txt pad script, winit 60 Hz loop, headless runner | PADClamp/ClampCircle/ClampTrigger all inputs + adapter decode 4000-report stream vs the C/C++ compiled natively (oracle/build.sh); real WUP-028 read over WinUSB on this machine | PADStatus from melee_types (GC 0x10) | WinUSB-less Linux adapter, Lua/live/bot pads, ctl_poll remap, stick dead-zone setting, gamepad rumble, Aurora mapping files |
| platform (step 2/5) | melee_gx | Aurora's GX: lib/dolphin/gx encoders (~45 GX* calls + GXInit), lib/gx regs.cpp BP/CP/XF decode + command_processor.cpp, shader.cpp + shader_info.cpp (TEV/XF -> WGSL, uniforms), gx.cpp populate_pipeline_config/blend/depth, gfx/texture_convert.cpp | API -> FIFO -> decoded GXState, display-list record/call, draw resolution (index buffers, array snapshots, uniforms), WGSL generation, all GC texture formats + TLUTs, wgpu offscreen renderer (push constants for Aurora's immediates) | vs Aurora itself (oracle/build.py links the shipping aurora_gx.lib): 145 texture cases bit-exact, 400 random ShaderConfigs WGSL byte-identical, 300 random GX API sequences -> identical ShaderConfig/pipeline/uniform bytes/WGSL, test display list rendered offscreen = Aurora's screenshot (0 of 307200 pixels differ, wgpu Vulkan and DX12 vs Dawn D3D11) | GXTexObj/GXTlutObj/GXLightObj/GXColor carry GcLayout with the SDK's GC offsets | shim_gx.c + swapchain/present: night 3 platform step 1 (row below); EFB copies (GXCopyTex), palettes, frame interpolation, draw merging/caches, lines/points untested on GPU |
| platform (step 3/5) | melee_audio | pc/platform/shim_ax.c: 35 AX entry points, ADPCM/PCM16/PCM8 decoder, mixer (SRC, ve, 9 mix targets, ITD, de-pop), AXFX reverb-std + delay, native clips, frame driver, volume/audio.cfg, WAV dump | all of it: `adpcm.rs` decoder, `ax.rs` 64-voice AXVPB pool + mixer + clips, `fx.rs`, `output.rs` (cpal device on an SPSC ring, linear resample to the device rate, device-clock / wall-clock pacing) | vs shim_ax.c itself, copied verbatim and built as shipping shims are (clang i686-pc-windows-msvc -O2, contraction on, no FMA on i686): 4000 decoder states x 64 samples + 30000-step random AX script (23.8k calls, 6182 sub-frames, 6001 non-silent; every frame's output + VPB writeback hashed, 64 compared sample by sample, reverb/delay/clips exercised); mutation checks fail it | VPB/PB*/AXFX offsets cross-checked against melee_types GcLayout (`layout.rs` test) | device output only run once here (cpal WASAPI 48 kHz opened, 0 underruns; not listened to); not wired to a synth/game loop; reverb-hi/chorus unimplemented (as in C); AI shims (shim_misc.c) only as fields; predictor nibble >7 masked (UB in C) |
| platform (step 4/5) | melee_net | pc/platform/gw_net.c (+gw_net.h): UDP transport, HELLO/ACCEPT/REFUSE handshake (protocol 9: rules + Envoy words, mods_soft, guest_info/guest_hello, hold_start), READY/START/START_ACK start-time agreement, redundant INPUT exchange with acks, RTT + GGPO frame-advantage time sync, piggybacked checksums/desync, reliable lobby channel, interrupt/timeout/quit; gw_netplay.c np_encode/np_decode + np_cb_* glue; gw_rollback.h network-facing interface | all of gw_net.c (`net.rs`, line for line), std::net UDP transport, gw_net_tests.c's simnet (`sim.rs`), 11-byte pad codec (`pad.rs`), `RollbackSession` trait + `NetAdapter` (the np_cb_* glue) + a generic `RollbackDriver` over a save/load `Simulation` (delay, prediction, rollback, stall, waits, final checksums; NOT a port of gw_rollback.c) | wire-exact vs gw_net.c itself built as shipping shims are (clang i686-pc-windows-msvc -O2; oracle/build.sh): 24.9k-line transcript over 27 scenarios (clean/lossy/hostile/pull inputs, time sync on/off, desync push/pull, 11 refusal/agree cases, old-protocol peers, blackhole, timeout, quit, lobby+hold) = every datagram's bytes (FNV, session id masked), event, desync and lobby delivery and end state, line for line; mutations fail it (the EMA's x0.125 is exact, so fused vs unfused cannot differ there); Rust<->C over real UDP 127.0.0.1 both directions (oracle/net_peer.c, 300 frames each way + checksums + config); all 26 gw_net_tests.c tests ported and passing (incl. real-UDP loopback); rollback driver: two peers over the lossy simnet, every final checksum equal to each other and to an offline run (172 rollbacks at 60 ms) | gw_net/GwRbInput are native-only (exe memory, never MEM1): no GC layout | not wired to the game loop; gw_rollback.c itself (gmscene/gw_snap/replay coupling, epochs `_e` API, Slippi mode) not ported; gw_netplay.c lobby/app layer (pick-ban, modsync, netdir, stage runs) not ported; IPv4 only like the C; Linux EMSGSIZE mapped by errno 90 untested |
| platform (step 5/5) | mex_import (`tools/mex-import`, bin `mex-import`) | workspace `tools/mex_port/make_mod_from_disc.py` + `split_modpack.py` (+ `mex_hsd.py` GCM/HSD readers, `dump_ftfunction.py`/`dump_itfunction.py` MEXFunction parse) | all of the packer/splitter (GCM FST reader, byte-compare delta, MxDt.dat tables + fighter/stage claims, mod.json + digest, sha256); m-ex code inventory: ftFunction/grFunction/itFunction blobs relocated, register-type dataflow, every D-form access mapped through `melee_types::GC_RECORDS` to a native field path | Akaneia + ACE: output byte-identical to the Python tools on the same discs (477 / 1168 files incl. mod.json, sha256); code coverage 59.5% (Akaneia, 86k insns) / 61.4% (ACE, 112k insns) of struct-offset accesses mapped; types are inferred, not checked against execution | uses melee_types' GC tables + typedef aliases; vanilla `T *` returns from decomp headers + symbols.txt (build.rs) | mod PPC not translated/run natively; no DOL/Gecko patches; no RVZ/CISO (no nod); character `dat_attrs` structs unmapped (no vanilla record) |
| platform night 3 (step 1/4) | melee_gx (`shim`, `render`) + melee_platform (`gfx`, bin `gx_viewer`) | pc/platform/shim_gx.c (the gw_GX* layer: indirect-stage guard, GXProject, GXInitFogAdjTable, 3-arg GXSetArray, camera-shake projection, draw-done routing (shim_vi.c gw_gx_*draw_done*), draw suppression, stats, render modes, stubs); Aurora GXGetTexBufferSize/GXInitLightSpot/GXInitLightDistAttn/GXGetProjectionv/GXGetViewportv; swapchain + present | `melee_gx::shim::GxShim` (every gw_GX* entry point the game calls, native args since game state is native LE; DAT data stays BE), `Renderer::with_instance` (surface-compatible adapter), `render_frame` + `blit_to` (EFB -> swapchain), `melee_platform::gfx::Presenter` (wgpu surface on the winit window, resize/lost handling, EFB screenshot), `Game::present(&Arc<Window>)` + `Game::resized`; `gx_viewer` draws a stage from the user disc (map_head model groups -> joints -> DObj/MObj/TObj/PObj -> GxShim -> FIFO -> wgpu) or a synthetic stage | oracle/shim_build.sh cuts gw_xplat_GXProject + gw_GXInitFogAdjTable out of shim_gx.c and the 3 Aurora functions out of extern/aurora at build time, clang i686-msvc -O2 as shipping shims: 9000 cases (4000 GXProject persp/ortho, 1000 fog tables, 2000 tex sizes, 2000 light spot+dist) bit-identical; a fused GXProject term fails 335 of them; unit tests: indirect guard on the leaked-refraction sequence (stage 0 drawn direct, logged once, decoded draw has no indirect texture), suppressed display lists, draw-done callback, shake + getters, render-mode GC layout; gx_viewer run here (RTX 5070, Vulkan, Bgra8Unorm swapchain): GrNLa (FD) 37 PObjs/14 textures/306 draws, GrNBa (Battlefield) 466 draws, recognisable on screen and in the EFB screenshot (kept in rust/target/gx-viewer/, disc art); synthetic stage shot committed (docs/rust-port/shots/gx_viewer_synthetic.png) | GxRenderModeObj carries `_GXRenderModeObj` GcLayout (0x3C) | game code does not call GxShim yet (no hand-ported HSD disp path; c2r externs unbound); EFB copies (GXCopyTex logged/skipped); PC matrix palette + pal/env caches, TEMP DIAG logging, perf counters, overlay hook not ported; viewer is bind-pose, first texture only, simple lighting (FD top shows layered-texture artefacts); no frame interpolation; present runs at the display rate, not tied to VI retrace |
| platform night 3 (step 2/4) | melee_net (`session`) | pc/platform/gw_rollback.c (+ the SyncTest loop of gw_snap.c gw_SyncTest_Iterations/IterStart) | `session::RbSession<G: RbGame>`: rb_init config (RbConfig or the same MELEE_RB_* env), gw_RB_SceneBegin epochs, every network-facing function (submit remote/local, `_e` epochs, pre-open early buffer, ring refusal, confirmed frame, frame advantage, curated checksum ring + finality gate, checksum_full, request_wait), rb_same/rb_neutral_raw/rb_hash/rb_gen padgen, the fake network (rb_arrival, rb_fake_deliver, live samples in flight), gw_RB_PadLatch + rb_live_sample, gw_RB_Iterations (open, hash log, network clock, rollback plan, stall, wait), rb_prepare, gw_RB_IterStart (load / hash / save), InputFor/InputAny/PadGoverned/PadField; `run_tick` = the gmscene resimulation loop; snapshots = the game's native state (`RbGame::save/load`, ring of maxb+3); `SyncTest<G>` (save F, load F-k, resimulate, compare whole-state + curated hash at every resimulated frame); `RollbackSession` impl so `NetAdapter`/gw_net drive it | oracle/rb_oracle.sh cuts rb_hash/rb_neutral_raw/rb_gen/rb_arrival out of gw_rollback.c at build time (clang i686-msvc -O2): 27380 cases identical (tests/session_oracle.rs); tests/session.rs over a native match built from the PORTED fighter code (ftcommon ground accel/air drift/fall, HsdRand, stale player-table stocks, nh_fighter/nh_game neutral hash; snapshots are byte images restored in place): SyncTest k=1..7 x 900 frames 0 mismatches (~3000 checks per k), timeline = no-rollback reference, side effects gated; negative control (state outside the snapshot) caught; fake network (lat/jit/loss 1/0/0, 4/2/0, 6/3/15, 9/0/5, padgen) final curated hashes = all-local run over 1499 frames, up to 965 rollbacks, depth 7, 0 desyncs; two RbSessions over gw_net on the lossy simnet (live-pad source, 3 scenarios) both peers' finals equal each other and an offline replay of the sent inputs, up to 763 rollbacks; mutation (misprediction never flagged) fails both | GwRbInput native-only (no GC layout); the test match's Fighter/StaticPlayer are melee_types records with GC offsets | not wired to the real game loop (no Sim reads pads yet; vs_match.rs runs Entry frames 0-4 only, input-independent, and lives in the bin crate); test sim's attribute constants typed in and stick conversion a plain deadzone (not PlCo / the pad pipeline); not ported: Slippi fixture mode (gw_rb_slippi_*), Envoy NetBuildApply, Geno fault undo, Lab visualiser, profiler/wall-clock stats, MELEE_RB_PERTURB, rb_hash_detail, gw_replay.c's replay source/trace, gw_snap.c's MEM1 dirty-page machinery (replaced by native snapshots); C session behaviour beyond the pure functions not compared against the C itself (only by the contract tests) |
| platform night 3 (step 3/4) | mex_import (`ppc`, `remap`, `translate`) | pc/platform/gw_ppc.c execute core (gw_ppc_execute/_x/_cr/_fp, CR/XER/cond/rotate helpers, bridge tail) for m-ex mod code | `ppc::ir` PPC32 -> IR decoder (exactly gw_ppc.c's subset and quirks: L=1 cmp / OE forms refused, FP Rc ignored, fsel unrounded), `ppc::interp` IR interpreter over a `Mem` + `Bridge`; `remap::GuestSpace`: native little-endian Rust structs given guest addresses, every guest load/store rewritten through build.rs `native.rs` (GC offset -> `offset_of!` + native type per field, aliases like `Mtx` resolved): scalars byte-reversed per field, bools 0/1, byte arrays/bitfield units 1:1, pointers only as whole words via handles, padding shadowed; `translate`: per-blob/per-disc coverage in `mex-import code`/`import` output | interpreter vs gw_ppc.c itself (oracle/ppc_oracle.sh cuts the C out at build time, clang i686-msvc -O2): 6000 random programs (2985 return, 1356 mem fault, 317 bad opcode, 62 panic, 1279 budget, 1 fetch) identical exit/pc/FNV of all regs + 64 KiB memory + bridge log (tests/ppc_oracle.rs); a fused fmadd fails 5 of them; rewrite vs melee_types GcDecode on random GC bytes (Fighter, Item, HSD_JObj, HSD_GObj, ftCo_DatAttrs; 20k loads + stores each, tests/remap.rs, caught the `Mtx`-alias bug); a PPC program doubling an f32 and bumping a u32 in a native Fighter. Disc coverage (local, not committed): Akaneia decode 86318/86340 reached insns (99.97%), typed accesses rewritable 8744/8783 (99.56%), 5621 untyped resolved at run time, 558 bridge targets; ACE 111708/111741 (99.97%), 11686/11725 (99.67%), 6990 untyped, 577 bridge targets | rewrite tables are melee_types' GC records (GcLayout names) | not run against a live native game (no Sim hook calls mod callbacks; no Bridge to ported Rust functions, 558/577 vanilla targets unbound); no PPC -> Rust source emission (interpreter only); blockers: 29 byte accesses into `Ground` +0x4..0x9 and 8 into `HSD_GObj` +0x12..0x17 that our types call pointers, 1 `size_t`-width mismatch; untyped (Opaque/Unknown/Indexed) accesses only work if their target is a registered region or plain big-endian bytes; non-executable reached words (22/33) are cmpi L=1, sc, op0/op19 xo42 data words analyze reached |
| platform night 3 (step 4/4) | melee_audio (`synth`, `sound`, `sim`, `sem`, `disc`) | src/sysdolphin/baselib/synth.c (SSM banks, 64 SFX nodes, envelope/fades, pan law + ITD, HPS PStream), axdriver.c (96 HSD_SM virtual voices, smash2.sem script interpreter, StartParam handles, key-off/pan/volume/pitch/mix, pause channels, aux setup), lbaudio_ax.c fn_80023750 / lbAudioAx_800237A8 / 80023870 / 80024B94 / the BGM start | all of the above except reverb-hi/chorus (no AX processor), BankDeflag's async ARQ, the Lab/Snap rewind wrappers; DevCom/DVD reads complete synchronously; `sim`: process-wide Sound + plain-`fn` lbAudioAx hooks + `Output` (frame driver) + `virtual_frame` (10 sub-frames / 3 frames); `melee golden --audio` sets melee_fighter's LBAUDIOAX_800237A8/80023870/80024B94 hooks; `sfx_play` bin | **no C oracle** (synth.c/axdriver.c are game TUs, only reachable through the PPC retarget); tests/sound.rs on synthetic SSM/SEM/HPS: handle = (vidhigh<<7)|slot from the prepend-built free list (95, 94..), script PLAY/WAIT/END lifecycle back to the pool, key-off, paused channel refuses starts, a 6-block HPS plays exactly 420 sub-frames then ends; melee/tests/audio_sfx.rs (user disc, local): ft_PlaySFX and ft_800881D8 store real handles, audible mix, key-off via 0x83D61; `sfx_play` on the user disc: smash2.sem 55 banks / 4035 scripts, main.ssm 246 sounds, id 0xC1 peak 19851, 3 plays on the 48 kHz WASAPI device (0 underruns, WAV capture shows the 3 bursts), menu01.hps 10 s continuous across blocks (no gaps); `golden --audio` CSV byte-identical to without | HSD_SM (0x34) and HSD_SynthSFXNode (0x50) mirrored with GC offsets, checked against melee_types GcLayout; AXVPB.sync / AXPBITD shifts added to layout.rs | not listened to by me (user check); handles match the C only for the same interleaving of starts and 5 ms callbacks (the C paces AX by device/wall clock, so the C is not run-to-run reproducible either); ft_80087d0c's four lbAudioAx bank-table lookups (800233EC/80023130/800230C8/80023220, group 1) not ported (the test stubs them); per-stage/fighter SSM loading (lbAudioAx_80026F2C etc.) and bank sizes from offsets_arr_803BC4E4 not ported (fixed sizes); no mixer thread (driven from the game thread); HPS loop (x8 back-pointer) follows the C but only checked on synthetic data and 10 s of menu01 |
| platform-3 (m-ex fighter in the native match) | melee (`mods`, `mex`) + mex_import (`mexfn::parse_at_base`/`vanilla_target`, `remap` bytes-region pointers + `ptr_override`, `layout::vanilla_name`) | pc/platform/gw_mods.h resolution (enabled.txt, requires/conflicts, one base, mount order, later wins); gw_mex_ftfunction_runtime.c gw_mex_interp_run for slots 0 (onLoad, fighter.c Mex_OnLoadDispatch) and 23 (onFrame, Fighter_8006A360 Mex_OnFrameDispatch), gw_mex_shim_index_item | `melee golden --scenario pad_falcon_wolf_fd --mods DIR`: Akaneia's Wolf (Fox base kind, MxDt internal id 27 for ftPartsTable) from its gdmods, its ftFunction run on the native Fighter through the IR interpreter + field rewrite; memcpy/memset/MEX_IndexFighterItem bound | local, user discs: Wolf onLoad returns (memcpy of attrs into dat_attrs_backup, 2 articles indexed), onFrame 217/217; 216 frames; P1 column identical to falcon/fox on all frames, P2 diverges at 70; base-only mount keeps falcon/fox byte-identical; mods.rs unit tests | none new | other m-ex fighters/slots; 536/538 vanilla bridge targets unbound; r2/r13 unbacked; no C golden for Wolf; ppc_oracle fails 4/6000 in release only (pre-existing) |
| platform night 4 (audio wiring) | melee_audio (`lbaudio`, `sim`) + melee (`golden --audio`) | src/melee/lb/lbaudio_ax.c: tables (s32_arr_803BB5D0/803BB6B0/803BB8D4, offsets_arr_803BC4E4/803BC6A4, lbl_803BB3C0, ssm/hps files, unk_arr_803BC4A0, flags), lbAudioAx_8002305C/80023090/800230C8/80023130/80023220/800233EC, fn_80023254 + the boot bank sizes, 80026E84/80026EBC/80026F2C/8002702C/80027168/80027648 + fn_80026650/800267B0/800268B4 (bank manager), 8002785C VS branch, 800236DC, 80023F28 (BGM by id), 80024B58, 80024DC4/80024FDC/80024FF4/8002500C/80025038, 80027DF8 | all listed, retail only (LBAX_N 55; m-ex branches out); the fn_80026C04 ARQ chain runs to completion synchronously; every lbAudioAx hook melee_fighter declares is now set by `install_audio`; `golden --audio` loads the match banks (Captain, Fox, FD -> captain/fox/last.ssm), starts the picked BGM (78, sp_end.hps) and runs 80027DF8 each frame | no C oracle (game TU); unit tests on the tables (bank ranges, kirby remap both ways, fn_80023254 order, boot sizes 2045824/911456/...), melee/tests/audio_sfx.rs on the user disc with the real lookups instead of stubs: match banks committed (state 2) for exactly those 3, a captain.ssm sound audible via ft_PlaySFX, BGM 78 audible and not restarted when already playing; `golden --audio` CSV byte-identical to without (216 frames) | LbState is native-only (file statics, no GC record) | fn_80024654 volume ducking, the GObj positional sfx (fn_800251EC..., 80026510/800265C4 hooks still unset), boot-time bank-1 loads (nr_name/pokemon/1pend), 1P/other scene branches of 8002785C, language-dependent ssm dir (fixed /audio/us/), m-ex tables; no mixer thread or device output from the game loop (golden is headless, virtual_frame); not listened to |
| asm (10 items) | n/a | n/a | 0 | n/a | n/a | none belonged to groups 0-5; crash-screen register reads return 0 on PC |
| integration: native golden run (sim round 1) | melee (bin) | the frame loop of `pad_falcon_fox_fd` through the entry sequence: match start, Entry, ftCo_800C6408 EntryStart, ftCo_800C6B6C EntryEnd, ftCo_Fall_Enter/Anim/Phys, the hashed part of Fighter_ChangeMotionState, the airborne part of Fighter_procUpdate, the entry beam (efAsync_Spawn 0x43E -> efLib_Create_Attach 0x24 from EfCoData.dat, efLib_Update, efLib_Cb_DPtcl -> efLib_SpawnParticleEffect default), the HSD particle system (particle.c/generator.c/psappsrt.c) for the seed, FD background (grLast), bgm pick, CPU-init draws | `melee golden`: disc reads (GrNLa, PlCo, EfCoData, PlCa/PlFx + AJ for figatree lengths/loop flags, ftCo_DatAttrs), p_ftCommonData for the ported ftcommon code, proc order s_link 1 (anim step + anim_cb) / 4 (phys + integrate) / 9 (efAsync flush) / 15 (efLib_Update before the particle procs) | **frames 0-63 identical on every column incl. seed and hash** (64 of 1801); frames 64-66 differ by 1 ulp in Falcon's y (the C's mpColl_80043754 splits the Entry->Fall frame into 2 or 4 substeps because the ECB jumps; the ECB is not modelled), 67-74 identical again; run stops at 75 (Falcon below y=0: ftCo_Fall_Coll not ported) | uses melee_types Fighter / ftCo_DatAttrs / ftCommonData decoded with their GC layout | environment collision (ft_800831CC/ft_80083E64 -> mpColl_*) and the bone-driven ECB (needs the fighter JObj + figatree animation), Landing (42) onward, pad input (zero before GO here), Fall IASA checks, effect JObj transform tracks (bind pose used), the other three goldens (not wired) |
| integration: native golden run (sim round 2) | melee (bin) + melee_world mp/coll.rs | the fighter skeleton (melee/src/skeleton.rs: PlXxNr.dat joint tree, ftParts_SetupParts, the figatree attach of ftAnim_8006EBE8 with and without blending, ftAnim_8006E9B4 both paths with lb_8000C490 / lbCopyJObjSRT, the x4_jobj2 anim skeleton, the x8B0 hand poses, lb_8000B1CC), the bone ECB and environment collision (mpLibLoad, mpColl_LoadECB_JObj, mpColl_80043754 substeps, mpColl_80046904 air step with wall/ceiling/floor checks and ledge probes, mpColl_8004ACE4 grounded step, entry points 800471F8/8004730C/80047E14/8004B108/8004B2DC/8004B4B0), the fighter command-script interpreter (ftAction_80073240 / 80073354 + lbcommand Command_00..08), landing dust (ftCo_8009F834 -> efAsync -> efLib_Create_Attach_Pos 0x18), pad input (melee/src/input.rs: pad_fixed.lua cycles, the HSD pad layer, Fighter_Spaghetti_8006AD10 input half, the IfAll Ready-HUD GO unlock), states Landing, Turn, Wait, WalkSlow/Middle/Fast (enter/anim/IASA/phys/coll) | `melee golden` as in round 1 plus PlXxNr.dat, IfAll.usd, PlCo ftPartsTable; proc order s_link 0 (HUD GO) / 1 (anim + script + hand poses + anim_cb) / 3 (input + IASA) / 4 / 6 (procMap) / 9 / 15 | **frames 0-102 identical on every column incl. seed and hash** (103 of 1801); all 193 collision substeps (steps, cur_pos, env flags, ECB) and every animated joint's local SRT equal the C build's own trace; the pad layer equals HSD_PadGameStatus on all 1801 frames x 2 ports | Fighter, CollData (melee_types, GC layout) | stops at 103: Falcon B+side (ftCo_SpecialS -> Raptor Boost, character code + the TransN root-motion path x594_b0); next Fox jab at 105 (hitboxes). NotPorted stops (only on a hit): wall push-outs, squeezes, ceiling push, floor-end wall slides, edge teeter/walk-off, LoadECB_Fixed. Not modelled: dynamics bones (scarf/tail physics), DObj/material and colour animation, sound (only the RNG-relevant pitch draws are checked and stop the run), item checks, the other three goldens (other stages / fighters / CPU / items not wired) |

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

## Platform track, night 3 step 1: shim_gx + swapchain/present + gx_viewer (2026-10-21)
- `melee_gx::shim::GxShim` is pc/platform/shim_gx.c in front of `melee_gx::api::Gx`. The C shim's
  big-endian conversions (matrices, float arrays, GXColorS10, render-mode images) are gone: the
  native port keeps game state native LE, so the shim takes native values. File data the GPU reads
  (vertex arrays, display lists, textures, TLUTs) stays the DAT's BE bytes, as on the GC. What the
  shim does is ported: the indirect-stage guard (`IndGuard`, lbRefract leak), `gx_project` (one f32
  rounding per op, it reaches the sim via Camera_LogicToScreen), `init_fog_adj_table`, the 3-arg
  `set_array`, camera-shake projection offset (`shake_offset`, gw_Fx_ShakeOffset's output), draw-done
  routing to HSD's callback, `suppress_draws` (resimulation), stats, the three render modes
  (`GX_NTSC_480_*`, `GxRenderModeObj` with GC layout), stubs. GXCopyDisp ends a frame: the FIFO is
  decoded and `take_frame()` hands the draws to the presenter. Aurora functions melee_gx lacked:
  GXGetTexBufferSize, GXInitLightSpot/DistAttn, GXGetProjectionv/Viewportv.
- `melee_gx::render::Renderer::with_instance(instance, Some(&surface), 640, 480)` picks an adapter
  that can present; `render_frame` draws into the EFB without readback, `blit_to(view, format)`
  scales the EFB onto a swapchain image (alpha forced to 1). `melee_platform::gfx::Presenter` owns
  the wgpu surface on the winit window (non-sRGB format preferred, AutoVsync, resize/lost
  handling) and takes EFB screenshots. `Game::present` now gets `&Arc<Window>`; `Game::resized`.
- Trap: `std::cos(float)` in Aurora on x86 MSVC (and clang i686-msvc) is `(float)cos((double)x)`,
  not a single-precision cosf; `init_light_spot` does the same (single cosf differed by an ulp in
  2 of 2000 cases, magnified by 1 - cr). And C's `(u32)f` on x86 leaves NaN/inf/>=2^32 with low bits
  0 where Rust's `as` saturates (fog table entries `000` vs `fff`): `x86_f32_to_u32`.
- Verified: `cargo test -p melee_gx` (shim_oracle: 9000 cases bit-identical vs the C cut verbatim
  from shim_gx.c/Aurora by oracle/shim_build.sh and built as shipping shims; a fused GXProject term
  fails 335; shim unit tests; the existing render test still 0-diff). `gx_viewer` on this machine:
  `cargo run -p melee_platform --bin gx_viewer -- --stage GrNLa.dat` (disc from GW_ISO_VANILLA)
  opens a window, presents FD (and `--stage GrNBa.dat` Battlefield) through GxShim -> FIFO ->
  decode -> WGSL -> wgpu -> swapchain, and writes the frame-60 EFB to rust/target/gx-viewer/
  (disc art, never committed). `--synthetic` draws a generated stage with no disc; its shot is
  docs/rust-port/shots/gx_viewer_synthetic.png. Checked by looking at the screenshots, no oracle.
- Not done: no game code calls GxShim yet (HSD's JObj/DObj/MObj/PObj disp is group 1's; the
  c2r `extern "C" GX*` declarations are not bound to it); EFB->texture copies; the PC matrix
  palette and the palette/envelope caches; frame pacing against VI; the viewer's material model is
  a stand-in (bind pose, first texture, one light, opaque alpha test at 128).

## Platform track, night 3 step 2: gw_rollback.c session + SyncTest (2026-10-21)
- `melee_net::session` is gw_rollback.c: `RbSession<G: RbGame>` holds the C's file-static `rb`
  (used/truth rings of 64, conf_slot, first_wrong, plan, hring, early buffers, fake network,
  latches). The C's externs are `RbGame`: `frame` (gw_Replay_Frame), `peek_input`
  (gw_Replay_PeekInput), `save`/`load` (gw_snap_save/load: the native game state; the session keeps
  a ring of maxb+3), `state_hash` (gw_Snap_Checksum), `game_hash` (gw_RB_GameHashForFrame, the
  neutral hash), `run_frame(inputs, resim)`. `run_tick` is the scene loop: `iterations` ->
  per iteration `iter_start` / `run_frame` / `iter_end`. Netplay: `RbConfig { net: true }` and
  `NetAdapter::tick(&mut net, &session)` before `run_tick` (the C calls gw_Netplay_Tick inside
  gw_RB_Iterations; the transport's callbacks borrow the session, so the poll is hoisted).
- `SyncTest<G>`: gw_snap.c's SyncTest on native snapshots (whole-state and curated hash compared at
  the start of each resimulated frame and at the end of the window).
- Verified: `cargo test -p melee_net --test session_oracle` (rb_hash/rb_gen/rb_arrival vs the C cut
  from gw_rollback.c, `oracle/rb_oracle.sh`), `--test session` (see the row). Snapshot lesson: the
  state is restored in place as bytes, so pointers into the fighters (e.g. CpuFighter.csP) stay valid;
  a snapshot that moved the Fighter would break them.
- Not done: wiring into a game loop (needs a Sim that reads pads), the Slippi mode, and comparing
  the stateful session against the C session itself (only its pure functions are oracle-checked).

## Platform track, night 3 step 4: melee_audio connected (2026-10-21)
- `melee_audio::Sound` = AX + 16 MB ARAM + the synth.c and axdriver.c state; `Sound::boot(fs, banks)`
  is lbAudioAx's audio init (HSD_AudioInitMultiPStream, reverb-std 1.88 on aux A, delay on aux B,
  three banks, stereo, `/audio/us/smash2.sem`, `/audio/us/main.ssm` into bank 0). `sub_frame()` =
  HSD_SynthCallback (8-node sweep, envelope, AXDriverCallback, PStream clock) + the AX mix.
- Sim side: `melee_audio::sim::{install, lbaudioax_800237a8, lbaudioax_80023870, lbaudioax_80024b94,
  virtual_frame, Output}`. `melee golden --audio` installs it and sets the fighter hooks (minimal
  call-site change in rust/melee: Cargo.toml dep, main.rs). The golden's frames never reach a sfx
  call yet, so its CSV is unchanged.
- Audible check: `cargo run -p melee_audio --release --bin sfx_play -- --iso <disc> [--id 0xC1]
  [--bgm menu01.hps] [--wav out.wav] [--scan N]`. `--scan` lists each main.ssm script's handle,
  length and peak headless.
- Floats: axdriver's sqrtf/powf are the game's (melee_runtime gekko::sqrtf, lb_00ce::powf),
  synth's sqrtf_accurate is gekko's; no fused site in either file.
- Left: a C oracle for synth/axdriver (needs the game-TU retarget pipeline), group 1's lbAudioAx
  tables (bank lookup, SSM-per-scene loading, fighter voice limiter 80026510/800265C4), a mixer
  thread, user listening test.

## Platform track, step 5: mex-import (2026-10-21)
- `tools/mex-import` (workspace member `../tools/mex-import`, bin `mex-import`; README there).
  `import --vanilla V --mod M --name N --out DIR` writes gdmods = the mods-folder entries the C
  build mounts (`gw_mods.h`: `<id>/mod.json` + `<id>/files/<disc path>`), one per m-ex fighter and
  stage plus `<name>-base`, and `<name>.mex-code.{json,tsv}`: every m-ex code blob's loads/stores
  mapped to native field paths (`Fighter.self_vel.y`, `Fighter.bf_2101`, `Fighter.anon_594`).
- Verified locally on the user's discs (nothing disc-derived committed; `--out` refuses a
  non-ignored path inside a git tree): Akaneia and ACE outputs byte-identical to the workspace's
  Python packer+splitter. Code coverage: 59.5% / 61.4% of non-stack, non-global accesses map to a
  native field; the rest are Opaque (pointers from `void *` fields: `dat_attrs`, other GObjs'
  user_data), Unknown, indexed, or ArrayElem candidates. Types are inferred from assumed callback
  seeds (r3 = owner GObj), not proven per slot.
- Not done: translating or running mod PPC natively, DOL/Gecko patches, RVZ/WIA images.

## Sim round 2: pad_falcon_fox_fd to frame 103 (2026-10-21)
- `melee golden` now matches the golden on **every column (seed and hash included) for frames 0-102**
  (103 of 1801; round 1 ended at 64). It stops at frame 103, the first frame that needs
  character code: Falcon's B + side (Raptor Boost, msid 357, with TransN root motion).
- What closed round 1's two gaps: the fighter skeleton animated from its figatree
  (`melee/src/skeleton.rs`) gives the six ECB joints, and `melee_world/src/mp/coll.rs` ports the
  mpcoll.c air and grounded steps over the group 2 mplib queries. Frame 64's 1-ulp miss was the
  4-substep split of the Entry->Fall ECB jump; landing happens at frame 77 as in the C.
- Then, in order of the frames they unlocked: Landing (77), the command-script interpreter and the
  landing dust (the seed at 77-80), Fox's hand pose (79, ftAnim_ApplyPartAnim), pad input and the
  GO unlock (84: the IfAll Ready HUD's animation ends at frame 85, its end callback
  `fn_8016B7F8 -> ftLib_800868A4` runs in golden frame 84's s_link 0), Turn (84), animation
  blending + Wait + Walk (95-102).
- Ground truth: the C build itself, instrumented with temporary logs (per-substep collision, ECB
  joints and every joint's local SRT, motion changes, the pad status and fighter input, IASA
  entry) written by `MELEE_ECB_LOG=<file>`, plus `MELEE_RAND_TRACE` + `MELEE_RAND_TRACE_STACK`
  resolved against melee-pc.map. The instrumentation is reverted in the tree; it is kept as
  `tools/rust-port/golden/c_trace.patch` (`git apply` it on a scratch build root to use it again;
  it also contains round 1's uncommitted RAND_TRACE_STACK hack in gw_replay.c).
- Facts found: the pad script's sample for golden frame g is the one set at g-1 (n = g - 1);
  the PC pad shim passes raw sticks through (100 -> 80/80 = 1.0); fighter input is zeroed by
  `x221D_b4` until GO, which also keeps x67F/x680 at 255; footsteps on FD's floor material draw no
  RNG (no footstep sound in the C trace); `x594` must be stored in GameCube byte order for the
  generated bitfield accessors (x596_bits.x7 = Turn's 2 drives a part copy at the Wait blend).
- Next: Raptor Boost (ftcaptainspecials.c, the TransN path ftAnim_8006E054 / x6A4), Fox's jab
  (hitbox commands ftAction_8007121C, ftColl hit detection), then the run off the stage (frame
  142+), death and rebirth. The other three goldens need their stages, fighters, CPU and items.

## Sim round 1: pad_falcon_fox_fd through the entry sequence (2026-10-21)
- `melee golden` matches the golden on **every column, seed and hash included, for frames 0-63**;
  frames 64-66 differ in s0/hash only (Falcon's y by 1 ulp), 67-74 are identical again, and the run
  stops before frame 75. Checked with `compare_golden.py ... --allow-short` against the committed CSV.
- Ground truth came from the C build itself: a temporary `OSReport` of every hashed fighter field in
  `nh_Fighter` (under MELEE_LOG_MOTION, reverted, never committed) and `MELEE_RAND_TRACE` (+ the
  uncommitted `MELEE_RAND_TRACE_STACK` hack in gw_replay.c) resolved against melee-pc.map. Every draw
  in golden frames 5-63 comes from the particle procs (`hsd_8039EE24`/`8039DAD4`/`8039930C`/`8039F05C`).
- What was needed past frame 5: EntryStart (timer x6BC=30, y = x4 + x20*t, x20 = (f32)(1.497345 *
  trophy_scale)); EntryStart's figatree length from `ftData->xC[238]` + PlXxAJ.dat (10 for both);
  EntryEnd (x6C0, no animation: frame_count keeps 10 on the change frame, then -1); the entry beam
  effect 0x24 (EfCoData desc: lifetime 60, two tracks spawning bank-0 particles 445/449 at frame 0 and
  448 at frame 10) created at s_link 9 and animated at s_link 15 ahead of the particle procs; Fall with
  the ported ftcommon fall/drift and a looping AObj (Fall row: 8 frames, x594 bit 1 = loop).
- Frame 64's 1-ulp miss: `mpColl_80043754` recomputes `cur_pos = last_pos + vel/steps` (steps times)
  with steps = |largest ECB or position delta| / 6 + 1. On the Entry->Fall frame the ECB jumps from the
  entry box to the bone-driven one, so the C takes 2 (or 4) substeps; the native run takes the
  position as is. Matching it needs the ECB, i.e. the fighter skeleton animated from its figatree.
- Next: ECB (ftColl ECB from bones: JObj tree + figatree DObj/AObj animation), mpColl air/ground
  collision over the ported mplib lines (landing at golden frame 77), Landing/Wait, then input.

## Native golden run: pad_falcon_fox_fd (2026-10-21)
- `melee golden --iso <disc> --out C.csv` (`rust/melee/src/vs_match.rs`, `stage.rs`, `disc.rs`) is
  the first real `Sim`: it reads `GrNLa.dat` and `PlCo.dat` from the user's disc (local only, not
  written anywhere), starts the VS match and steps frames until the next frame needs unported code.
  `compare_golden.py docs/rust-port/golden/pad_falcon_fox_fd.csv C.csv --allow-short`.
- Ground truth for frame 0 came from the C build itself (a scratch Lua pad script that logged
  `gd.player()` each frame, hashes unchanged): both fighters are in Entry (322), airborne, at
  (-60, 10, 0) and (60, 10, 0), facing +1/-1, shield 60, cmd frame_count -1, jumps used 1,
  x67F = x680 = 255, stocks 3. A brute force over the NHV1 word confirmed exactly that tuple.
- **Result: 5 frames (0-4) written. First diverging frame: 0. First diverging field: `seed`
  (golden 9D38619F, native 00003039 = MELEE_TEST_SEED 12345 undrawn), hence `hash`.** `s0` and `s2`
  (every fighter field) are equal on all 5 frames; with `--seed-from <golden>` (diagnostic: copies
  the golden seed column) `compare_golden.py` reports the 5 frames identical, so the frame combine
  and the fighter words are right and the seed is the only difference there.
- The seed: the C build draws 54 times (12345 -> 9D38619F) before frame 0, then 19, 25, 19, 7, 22,
  42, 56, ... per frame (MELEE_RAND_TRACE + melee-pc.map). Every draw in logic frames 29-34 (golden
  0-5) comes from the HSD particle generator for the entry effects: `hsd_8039930C` (particle.c),
  `hsd_8039DAD4`, `hsd_8039EE24`, `hsd_8039F05C` (generator.c). Matching `seed` needs
  generator.c/particle.c ported (group 1) plus the ef/ entry-effect spawns that drive them. (So
  particle state, excluded from the hash, still reaches it through the RNG.)
- Where the run stops: frame 5, Falcon's Entry timer (Player_GetUnk4C = 5; Fox's is 10) reaches 0
  and `ftCo_Entry_Anim` calls `ftCo_800C6408` (EntryStart: Fighter_ChangeMotionState, the entry
  animation; the C then moves y to 0x1.438386p+3 by frame 6). Not ported: Fighter_ChangeMotionState,
  the fighter animation/figatree path (group 1 DObj/AObj walkers are stubs) and the per-frame
  fighter update (fighter.c Fighter_UnkProcess*); Fox diverges the same way at frame 10.
- Approximations in what runs: the spawn point's world translation is composed from the joint
  descriptors (exact only with no rotation/scale above it; FD's are, `SpawnPoint::exact`), not
  through the JObj system; the Entry timer lives beside the Fighter, not in `fp->mv`; fighter
  defaults are only the fields the hash reads, not all of Fighter_UnkInitReset; stocks 3 is the
  scene's, not read from the VS rules struct. `MELEE_GOLDEN_DUMP=1` prints the start state.
- Other scenarios are not wired (only falcon/fox and FD).

## End of night 2: summary (2026-10-21)
Functions ported (all bit-exact vs the shipping-contraction C unless noted in the row):
group 1 ~236 of ~2000 (~12%), group 2 301 of 3013 (10%), group 3 526 of ~2820 (19%),
group 4 1386 of ~4340 (32%), group 5 1446 of ~5000 (29%) [night 3 recount: 1468 of 4966 after round 9], group 6 251 (of 543 scanned
leaves/callers; gm/mn/if/ty/lb total not counted). Totals are estimates from each group's doc.
`cargo build --workspace` clean, `cargo test --workspace --no-fail-fast` all green at 95307a264.

## Night 3: plan, in order
1. **Golden seed (group 1):** port particle.c / generator.c (hsd_8039930C, hsd_8039DAD4,
   hsd_8039EE24, hsd_8039F05C) and the entry-effect spawns, so `seed` matches pad_falcon_fox_fd
   from frame 0 (C draws 54 before frame 0, then 19/25/19/7/22).
2. **Past frame 5 (groups 1+3):** Fighter_ChangeMotionState, ftCo_800C6408 EntryStart, DObj/AObj
   anim walkers (remove the `JObjExt` stubs), the per-frame fighter update; target frame 10 (Fox).
3. **JObj load/ResolveRefs + DAT-driven spawn** so stage spawns go through the JObj system
   (drops the "no rotated parent" approximation), then wire the other three goldens.
4. **Groups 2/4/5:** keep porting the callers of what the golden needs first (ground.c rest,
   mplib, item spawn/think, chara callbacks behind `chars::cb`); shrink the TODO(groupN) panics.
5. **Platform step 3 (present):** join melee_gx to melee_platform's winit loop (swapchain),
   port shim_gx.c, EFB copies; connect melee_audio to the synth; wire melee_net to a rollback
   port of gw_rollback.c once the sim can save/load.
6. **User checks** (cannot be done unattended): visuals of a real scene, audio by ear, the
   adapter's stick rest offset / trigger 255 reading vs the C build, HID adapter, a gamepad,
   online play against another machine.

## melee_modes::tu: whole-TU ports with a stateful C oracle (shard 465-472)
`rust/melee_modes/src/tu/<stem>.rs` ports every function of one gm `.c` file (gmcameramode,
gmclassic, gmdebugmode, gmevent, gmfixedcamera, gmgiant, gmgover, gmhanyucss: 147 functions,
all checked). Callees of unported TUs are `ext` hooks (`ext::NAME`, settable, unset panics
"TODO") named like the C; data owned by other TUs is a `pub static mut` in `ext`; the block is
generated from the C by `gen/cref/gen_tu.py` (libclang, the native flags of `oracle/build_r10.sh`).
- Oracle: `python rust/melee_modes/gen/cref/run_tu.py <stem>` compiles `gen/cref/tu/<stem>.c`,
  which `#include`s the game's `.c` verbatim (shipping contraction + gwtool lowering), defines a
  recording stub for every callee (generated: trace hash of call order, arguments, and return
  values from a shared xorshift), fills the synthetic game state (generated `fill_<T>` per record,
  bitfields by value, pointers linked by the driver), runs each function on 300-1000 seeded
  states, and writes one line per case (return, state hash, trace hash) to `tests/data/mtu_<stem>.txt`.
  `tests/tu_<stem>.rs` makes the same draws, installs the same recording hooks and compares.
  State hashes cover every field of the records (generated `hash_<T>`: scalars, bitfields by
  value, pointers as region ids, function pointers as table ids), so a wrong store anywhere shows.
- Debugging a mismatch: `rust/melee_modes/gen/cref/dbg.sh <stem> <test> <op:case>` prints the
  first differing trace / state words of one case; `ct.sh <test>` runs one test.
- Caveats found: records the `.c` defines are hand-written with `gc_layout!` (melee_types does not
  generate them); an anonymous union / struct member is compared as raw bytes (bitfield overlays
  such as `MatchPlayerData.x3` are not compared by value: melee_types keeps bitfields in GameCube
  bit order, native C does not); `gmclassic.c` reaches `gm_804908A0` through
  `gmClassicIntroDataBuffer` (adjacent on the console), so the oracle patches the two declarations
  into one object (`specs.py`); `onEnterVs` with `player_init[4]->c_kind == ChKind_None` writes the
  character into the bitfield byte `EventData+0xB` (never in the real level data; the oracle avoids
  it); `gm_8016A404((s32) fnptr)` passes a function address as a 32-bit int (PPC-ism: a native
  callee cannot turn it back into a function; the trace names the function by its low 32 bits).

## melee_world::tu: whole-TU gr ports with a stateful C oracle (shard 553-560)
`rust/melee_world/src/tu/{grdisplay,grdynamicattr,grfigure1,grfigure2,grfigure3,grfigureget,grflatzone,grfourside}.rs`
port every function of those `src/melee/gr` files (187 incl. the static-inline helpers; 44 constant-return
callbacks are re-exports of `gr::trivial`). Callees of unported TUs are `ext` hooks (unset: panics "TODO"), the
block is generated from the C by `gen/mtu/gen_tu.py` (a copy of melee_modes' `gen/cref` oracle, retargeted).
- Oracle: `bash rust/melee_world/gen/mtu/regen_gr.sh` (or `python gen/mtu/run_tu.py <stem>`) compiles the game's
  `.c` verbatim (shipping contraction + gwtool lowering), with a recording stub per callee, runs every function on
  500-2500 seeded synthetic states per op and writes `tests/data/mtu_<stem>.txt` (return, field-by-field state hash,
  call-trace hash); `tests/tu_<stem>.rs` makes the same draws. The stage drivers are written from one description by
  `gen/mtu/mk_stage_driver.py` + `mk_stage_cfgs.py` (C and Rust halves cannot drift); grdisplay / grdynamicattr
  have hand-written drivers (`gen/mtu/tu/<stem>.c`, `tests/tu_<stem>.rs`).
- Traps found: (1) melee_types keeps `uintptr_t`/`HSD_IDKey` at 4 bytes (GC) but native C has 8: the state walker
  compares such fields as 32 bits; (2) a top-level anonymous union member must be addressed as the record, not as a
  reference to it (generator bug fixed); (3) `x = (u64) fv() << 32 | fv()` has no defined evaluation order: use
  `fv64()`; (4) unions that hold pointers must be hashed by region id (`PIB`), never as raw bytes (addresses differ
  between the two executables), and a pointer slot another view overwrites byte-wise only counts at an exact base;
  (5) bitfield bytes differ between native (LSB-first) and GC order (MSB-first): compared by value (`foursideCrane.x1`);
  (6) div/rem by zero is not rewritten by `lower.py` (the native C traps), so fixtures keep divisors non-zero; the
  port uses `cabi::srem` for the gwtool result (`x % 0 = x`), which the oracle therefore does not exercise.
- GC union aliasing: `grflatzone.c` names one union word through `flatzone.xD0`/`unk.xD0`/`flatzone2.xD0` and
  `flatzone.xD4`/`flatzone2.timer`; with 8-byte native pointers (`flatzone2.xCC`) these stop aliasing. The port keeps
  the GameCube aliasing (uses `flatzone2.xD0` / `.timer` everywhere) and the oracle compiles a copy of the TU with
  the same substitution (`patch` in `gen/mtu/specs.py`). Mutation checks: flipping a fused multiply-add site, a
  boundary constant or a state-machine constant in grdisplay / grflatzone / grfourside fails the test.
- Not compared: `OSReport` diagnostics; `atan2f` (lbtrigf.c) is an `ext` hook in grfourside (ported and checked in
  melee_runtime); the crane/stage init chains of `OnInit` / `setup_gobj` ops compare the callback trace, not the
  union bytes the chain overwrites (each init function has its own exact op).

## melee_world::tu: gr/grt*.c set B (shard 601-608)
`grtkirby grtkoopa grtlink grtluigi grtmario grtmars grtmewtwo grtness` ported by the same
generator as set A (`GRT_SET=b bash rust/melee_world/cref/gen_grt_stages_ref.sh`; test
`tests/tu_grt_stages_b.rs`, 60 seeded cases per function). New fixtures: Generator/AppSRT pair
(mario particle scale), 8-slot yakumono struct (mewtwo), `ypick` biased ftLib positions so the
mario floor-crossing branch is exercised.

## melee_world::tu: gr/grt*.c set C (shard 609-616)
`grtpeach grtpichu grtpikachu grtpurin grtsamus grtseak grtyoshi grtzelda`, by the set-A/B generator
(`GRT_SET=c bash rust/melee_world/cref/gen_grt_stages_ref.sh`; test `tests/tu_grt_stages_c.rs`, 60
seeded cases per function, all 20 functions of each TU incl. the re-exported constant ones). New:
`grtseak.c`'s TARGET_PC creator (`Mex_GrCallbacks` / `Mex_GrTrace`, fixture = a fake m-ex
`StageCallbacks[4]` whose entries record their call; the "grtrace" OSReports carry `%p` args and are
the separate ext `OSReport_grtrace`, traced by format only), `grtpurin.c`'s loop over the
joint table (`lb_8000B1CC` recorder now returns x in 37.5..162.5 so both sides of `x < 130` run) and
its `yakumono_param` reader; the TU-local `YakumonoParam` carries a `gc_layout!` GameCube table.

## melee_items::tu: it/kinds item-kind ports with a C state oracle (shard 657-664)
`rust/melee_items/src/tu/{itarwinglaser,itbat,itbombhei,itbox,itcapsule,itcerebi,itchicorita,itclimbersblizzard}.rs` port
every function of those `src/melee/it/kinds` files (234, plus the static-inline helpers from `itbombhei.static.h`,
`it/inlines.h` and `jobj.h`, which live in the file or in `tu/kinds_inl.rs`) with the C names and signatures; callees of
unported TUs are `ext` hooks (unset: panics "TODO"). The old snake_case forwarders in `itfwd*.rs` / `itkinds*.rs` take
`&mut Item` and are not re-exported here (different signatures).
- Oracle: `bash rust/melee_items/gen/kinds/regen.sh [stem...]` (a copy of melee_world's `gen/mtu`, retargeted; own file
  names so it does not collide with the other melee_items TU oracles): `mk_driver.py` writes `gen/kinds/tu/<stem>.c` and
  `tests/kinds_<stem>.rs` from one description (`specs_cfg.py`: typed `x4_specialAttributes` buffer, fixture tweaks,
  argument hints, extra ops), `run_tu.py` compiles the game's `.c` verbatim (shipping contraction + gwtool lowering)
  and writes `tests/data/kinds_<stem>.txt`. The world is 3 gobjs / items / articles / attributes, 48 jobjs (14-bone
  dynamic bone tables), the common item data and 4 Vec3, redrawn field by field every case (400 per function);
  every callee is a recording stub (call order, arguments, `Vec3*` / `SpawnItem*` pointees by value, random returns,
  `lb_8000B1CC`-style out-parameters written from the shared stream). After each call the whole world, the return
  value, the trace and (once) the `ItemStateTable` arrays are hashed.
- Walker changes vs melee_world: a union whose views all cover the same storage and one of which holds bitfields
  (`UnkFlagStruct`, `flag32`) is compared by bitfield value (native C is LSB-first, melee_types MSB-first); a union
  of differently sized variants (`Item_ItemVars`) stays raw bytes, so a TU's pointer slots / bitfield bytes are hashed
  by value and cleared in the per-TU `ptrs` snippet; stubs are defined as `(name)(...)` so macro-named callees
  (`HSD_JObjSetMtxDirty`) work; only file-scope variables count as extern data; fill / hash bodies are `unsafe`.
- Fused sites (checked in the IR): `itChicorita` leaf `prev_pos.x += x8 * facing_dir` and Blizzard
  `(x10 - xC) * rand + xC` are `fmadds`; `itBox_UnkMotion8_Phys` has two f64 sites (`mul_add`, indistinguishable after the f32
  store except in rare double-rounding cases, so a mutation of them is not caught). `sinf` / `cosf` are the real
  `melee_runtime::gekko` ports (gekko_fp.c is linked into the oracle); `atan2f` and the variadic `efSync_Spawn` are
  `ext` hooks (fixed-arity typed hook per call shape).
- `it_802E7654` has no `return` in the C; the port returns the spawned gobj (what PPC leaves in r3), the oracle does
  not compare that return value. Ints that the C wraps (`-= 1`, sums of weights) use `wrapping_*` ops. Mutation checks
  (flipping fused sites, constants, comparison boundaries, state ids) fail the tests.

## melee_items::tu: it/kinds itpeach*, itpikachu*, itpippi (shard 769-776)
`rust/melee_items/src/tu/{itpeachparasol,itpeachtoad,itpeachtoadspore,itpeachturnip,itpikachuthunder,itpikachutjoltair,itpikachutjoltground,itpippi}.rs`
port every function (131 incl. statics) with the shard 657-664 oracle (`bash rust/melee_items/gen/kinds/regen.sh <stem>`,
`cargo test -p melee_items --test kinds_<stem>`, 400 seeded states per op). Fused sites: toadspore `speed` and `angle`
(`fmadds`). Notes: the turnip odds table is read by raw offset (the C reads one entry past `x8[length-1]`);
`peachturnip.xDD4.b0` is hashed by value (native bitfield order differs); `it_802BDA40` with state row -1 would index
`it_804D5518[-1]` (UB, unknown sdata neighbour): the port returns 0 and the oracle only asks rows 1 and 2; `bool` is
MSL `int`, so the ports return Rust `bool` and the oracle's stubs for int-returning callees used as booleans return 0/1.

## melee_items::tu: it/kinds set (shard 673-680)
`rust/melee_items/src/tu/{itegg,itentei,itevyoshiegg,itfflower,itfflowerflame,itfire,itflipper,itfoods}.rs`
port every function of those `src/melee/it/kinds` files (220 incl. the static-inline helpers) plus their
`ItemStateTable` data. Callees of other TUs are `ext` hooks (unset: panics "TODO").
- Oracle: `bash rust/melee_items/gen/mtu/go.sh <stem>` (`gen_it.py` = melee_world's `gen_tu.py` retargeted, plus
  `UnkFlagStruct` filled / hashed bit by bit by value; `mk_it_driver.py` + `cfg_it.py` write the C driver and the
  Rust test from one description; `run_it.py` compiles the game's `.c` verbatim with shipping contraction + gwtool
  lowering). One fixture item (gobj, Item, Article + 1 KiB attribute buffer, ItemAttr, 4 joints, bone table, second
  item for spawns, owner, ICD `it_804D6D28`), 150 seeded states per function, state hashed field by field, every
  call traced. Spawn records are hashed inside the `Item_80268B18` stub. Mutation checks: fma sites (foods
  `fmadds`, flipper `sqrtf(vx*vx+vy*vy)`), a boundary compare and a swapped spawn parent each fail the test.
- Variadic `efSync_Spawn`: the compiled copy of the TU counts the arguments (`patch` in specs_it.py), the port
  calls `ext::efSync_Spawn_N(gfx, gobj, &[args])`; pointees of the stack args (`&val`, `&v`) are not compared.
- Traps: (1) `UnkFlagStruct` raw bytes differ between native (LSB-first) and GC bit order; compare by value.
  (2) the flipper's `xDEC_stageAnchor` pointer lives in the item-var union: normalised to its region id before the
  hash. (3) `itFoodsAttributes[]` is indexed with the native 24-byte stride (GC: 16); the port mirrors the native
  struct like the rest of melee_types. (4) `ABS(x)` is `x < 0 ? -x : x`; `-1.0f * y + c` is a (non-discriminating)
  fma site, kept as `fmadds`.

## melee_items::tu: it TUs with a stateful C oracle (shard 649-656)
`rust/melee_items/src/tu/{itspawn,itmaterial,itzako,it_26B1,it_2725,it_279C}.rs` (271 functions incl. 18 static inlines, all
present; it_3F14 / it_3F2F hold only data tables and have no functions; their tables are not ported). Oracle (generator copy kept in gen/mtu649, since gen/mtu was taken by another shard's variant; the generated gen/mtu/tu files stay where they are): `python
rust/melee_items/gen/mtu649/{gen_tu,mk_driver,run_tu}.py <stem>` (same design as melee_world's gen/mtu: the game `.c` verbatim,
shipping contraction, recording stubs, seeded synthetic state, field-by-field hash) -> `tests/data/mtu_<stem>.txt`,
`tests/tu_<stem>.rs`. Per-TU fixtures live in `gen/mtu/stems.py` / `specs.py`; the shared world is `tests/mtu_w/world.rs`.
New generator features: `xpools` (stem-private record pools), `gen_extra`, `paren_stubs`, `skip_freg`, `post_all`, `noret`.
- Bugs the oracle found in the first-pass ports: invented RENDER_* constants (itmaterial); missed fused multiply-adds
  (itzako product_xyz/sqrt structure, it_26B1 it_8026B344); `M_PI_2` is a double (it_2725); `1 << n` with n >= 32 needs
  `cabi::shl`; C `bool` is int (parameters and pass-through returns are i32); dropped TARGET_PC paths (Mex_ItRecord);
  argument evaluation order (itspawn: HSD_Randf before gm_8016AE80); calls into sibling TUs must go through `ext`;
  header inlines must be local, not `ext`; 27 functions the first pass left to itlib were missing from the TU files.
- Oracle traps: unions with pointers or bitfield views (xDD4_itemVar, xD90) alias differently native vs GC bit order, so
  fixtures normalise them (`post_all`); `HM_LINE` (MTU_DBG builds) prints the gen.h line of each hashed word.
- Not measured: branch coverage (llvm-cov profile versions do not match rustc's); `chars_ref` (melee_chara) does not
  terminate in this checkout, unrelated.
## melee_items::tu: it/kinds set (shard 681-688)
`itfoxblaster itfoxillusion itfoxlaser itfreeze itfreezer itfushigibana itgamewatchbreath itgamewatchchef`
(`rust/melee_items/src/tu/`, 163 functions incl. the static-inline helpers and the TU's `ItemStateTable` /
float / int data tables; header inlines the TUs use are copied into each file). Callees of unported TUs are
`ext` hooks (unset: panic "TODO"); `cosf` / `sinf` are `melee_runtime::gekko` (the oracle links the real
`gekko_fp.c`), `atan2f` is an ext hook.
- Oracle: `python rust/melee_world/gen/mtu/mk_itkinds.py` writes, from the one description in
  `gen/mtu/specs_it.py` (`CFG`), the C driver `gen/mtu/tu/<stem>.c` and the Rust test
  `rust/melee_items/tests/tu_<stem>.rs`; `python rust/melee_world/gen/mtu/run_tu.py <stem>` builds the C
  (shipping contraction + gwtool lowering) and writes `tests/data/mtu_<stem>.txt`. Each case (300 per
  function, seeded) builds a synthetic world (item gobj + Item, owner / reference / parent / spawn-result
  gobjs and Items, a JObj chain, article / attribute / dynamic-bone records, the kind's item-var union
  member) and hashes every field of every record after the call (generated `fill_` / `hash_` walkers,
  `skip_fields` + by-value bitfield views added to `gen_tu.py`), the return value, and the call trace of
  every stub (the `SpawnItem` a spawn stub receives is hashed at the call).
- Traps found: (1) `-1.0f * x` is folded to a negation by LLVM, which flips a NaN's sign; the console
  `fmuls` keeps it (`itfreeze.rs` `mul_m1`); (2) `attrs->entries[i]` on a one-element array type: index
  through a raw pointer; (3) a float local read before init (`it_8028EB88` `ip`) is 0 in the game build;
  (4) `HSD_ASSERT(jobj)` in the jobj.h setters hangs on NULL: the oracle never passes NULL, the port asserts;
  (5) `it_8029CD78` loops forever when the ghost jobj exists (kept as `loop {}`).
- Mutation checks (flip a fused site, a table value, a state constant, a flag, an index) are caught in all
  eight TUs; equivalent mutants only at float-equality boundaries.

## melee_items::tu: it/kinds set (shard 705-712)
`ithinoarashi ithitodeman ithouou itkabigon itkamex itkinoko itkirbycutterbeam itkirbygamewatchchefpan`
(`rust/melee_items/src/tu/`, 133 functions incl. the static-inline helpers and the 8 state tables; the older leaf-style
ports in itkinds*.rs / itfwd*.rs keep their own signatures). State oracle: `python rust/melee_world/gen/mtu/mk_itk.py all <stem>`
generates the C driver (the game `.c` included verbatim, shipping contraction + gwtool lowering, `gekko_fp.c`, `lb/lbtrigf.c`,
`lb/lbvector.c` linked for sinf/cosf/atan2f/lbVector_*), the recording stubs for every other callee, and the Rust test
`tests/tu_<stem>.rs`; 400 seeded cases per function, each hashing every field of Item + the spawned Item + JObjs + attribute
records. Bitfield unions (UnkFlagStruct, flag32) are hashed by value; the item-var union is walked as the TU's variant;
hinoarashi's Vec3-over-flags pun (`it_802D61C8` / `it_802D6310`) is compared as raw bytes. `efSync_Spawn` (variadic) is an
ext taking `&[*const c_void]`, its extras traced by gfx id. Mutation checks (flipping fused sites, boundaries, constants) fail
the tests; `-1.0 * x + z` fused vs unfused is bit-equal, so that site is not distinguishable.
## melee_items::tu: it/kinds set 721-728 (itkyasarinegg itleadead itlgun itlgunbeam itlgunray itlikelike itlinkarrow itlinkbomb)
`rust/melee_items/src/tu/<stem>.rs` ports all 344 functions of those `src/melee/it/kinds` files (static inlines included; header inlines are
local `#[inline]` copies) with the C names and signatures, checked by the shard 657-664 kinds oracle (`bash rust/melee_items/gen/kinds/regen.sh
<stem>`, tests `kinds_<stem>.rs`, 400 seeded cases per function, whole world hashed after every call, call trace hashed).
- Raw-int `bool`s: `bool` is the MSL `int`, so a callback that `return`s a callee's int (`return it_8027C794(gobj);`) must return that
  value unnormalised: those ports return `i32` (state-table slot `Some(f)` instead of `pred_b(f)`), found by the oracle (random stub returns).
- Fused sites (checked in the IR, every one caught by an unfused mutation unless noted): `itlgunbeam` `diff * r + lo` x2; `itlinkarrow`
  `arg4 * ((x8-x4)/arg5) + x4` x2, the `lookup[8] * rand + lookup[0]` terms (cases 0/2/4/6 and 1/3/5, both in `802A81C4` and `Motion4_Anim`),
  `xD4 * cosf/sinf + xC8/xCC` (`Motion2_Phys`, `Logic98_HitShield`); `itlinkbomb` `Motion4_Phys`; `itleadead` `neg_dot` is
  `fmuladd(-ax, bx, -(ay*by))` then `fmuladd(-az, bz, .)` (clang folds the negated products; IR-checked, the branch outcome it feeds is
  not sensitive to the rounding in the random cases, so no mutation of it is caught). Use `melee_core::fp::fmadds` (f64 multiply-add), never `mul_add`.
- Oracle additions: `all3()` in `specs_cfg.py` fills the special-attribute block of all three items (spawned items read `ART[2]`);
  `deep_ecb` spec flag traces the pointee of `itECB*` stub arguments (`TECB` / `tecb`; off for the older TUs so their data files stay valid);
  bitfield words inside an item-var union are zeroed before their bits are set and hashed by value (native C is LSB-first, melee_types MSB-first).
- No `return` in the C (UB, PPC leaves r3): `itKyasarinegg_UnkMotion4_Anim` (returns the `it_802751D8` int), `itLinkArrow_802A81C4`
  (returns the new `x9C`) and the fall-off path of `itLinkArrow_802A850C` (returns 0); the oracle skips those return values (state, trace
  and the defined path of 802A850C are compared). Boundary constants that random stub returns never straddle (for example `dx <= x10`,
  `ad > 3.0` in leadead, `var_f1 < 15.0` in likelike) are matched by reading, not by mutation.
- 274 callee hooks across the eight files are `ext` TODO stubs (unset: panic "TODO: name is not ported").
## melee_items::tu: it/kinds itluigifireball, itmaril, itmariocape, itmariofireball, itmarumine, itmasterhandbullet, itmasterhandlaser, itmatadogas (shard 737-744)
Same layout and oracle (`bash rust/melee_items/gen/kinds/regen.sh <stem>...`) as the 657-664 shard: every function of the
eight files (150 incl. statics, e.g. `calc_dist_2d_accurate`, `sqrtf_store`) in `rust/melee_items/src/tu/<stem>.rs`, 400 seeded cases each.
- Driver additions: `double` returns are hashed (`mk_driver.py`); variadic `efAsync_Spawn` / `efSync_Spawn` are fixed-arity custom stubs;
  `itmasterhandlaser` adds a fake `Fighter` (`parts` -> `FighterBone[8]`) as GO[1]'s user data, a 2-entry `HSD_Joint` chain for `xC8_joint`,
  and `mpCheckMultiple` writes its `Vec3` out-parameter; bool-returning stubs of the bullet return 0/1.
- Fused sites (IR-checked): f32 `fmadds` in `calc_dist_2d_accurate` (luigi / mario fireball) and 4 in `it_802F063C`; f64 Newton steps of
  `sqrtf_accurate` / `sqrtf_store` (`3.0 - g*g*x` as fma; un-fusing them is not caught, the f32 rounding hides it). Mutations of the f32 sites,
  the `M_PI` negation in `itmaril` and the `MTXDegToRad` constant in `itmatadogas` fail the tests.
## melee_items::tu: it/kinds nokonoko ... peachexplode (shard 761-768)
`rust/melee_items/src/tu/{itnokonoko,itoctarock,itoctarockstone,itoldkuri,itoldottosea,itparasol,itpatapata,itpeachexplode}.rs`
port every function of those `src/melee/it/kinds` files (257, statics and the two `*_inline` helpers of itnokonoko included),
checked with the same whole-TU C state oracle as shard 657-664 (`bash rust/melee_items/gen/kinds/regen.sh [stem...]`,
400 seeded cases per function, tests `kinds_<stem>`). New specs live in `specs.py` (`MY_RET` / `MY_EFF`: pass-through `bool`
callees return 0/1, `it_80276308` returns 0/4/8, `ftLib_8008627C` returns an owner gobj or NULL, `Vec3` out-parameters are
written) and `specs_cfg.py` (typed `x4_specialAttributes` with a pointee block at +64, the `it_804D6D40` data, the
ptr slot of `oldottosea.x20` cleared after hashing, bones 14..18 of the dynamic bone table for patapata).
- Fused sites (checked in the IR): parasol `deg_to_rad * fm + ry`, octarockstone `Randf * ((pos.y - y) / 10) + vel.y`,
  oldottosea `pos.x += x14 * facing` and `vel.x += -facing * xC`; octarock / oldkuri / patapata / nokonoko / peachexplode have none.
  Mutations of each fused site, constants and state ids fail the tests.
- `x40_vel.y <= 0.2` (oldkuri, nokonoko) is a double comparison in the C (`0.2` unsuffixed); the port compares as f64.
- `itParasol it_8028B08C(item, statenum)` indexes a 8-entry table by `anim_id`, which is -1 for states 0 and 3 (C reads out
  of bounds); the oracle only draws the states 1, 2, 4..10 that the game uses. `bool` parameters of the C are `int`: ports take `i32`.
- Not in these files (itwhitebea.c): `itOldottosea_UnkMotion8..11`, `it_802E3400/3528/35CC/37A4`; the state table names the former
  through `ext::cb`.
### Shard 745-752: it/kinds itmato, itmball, itmetalb, itmetamon, itmew, itmewtwodisable, itmewtwoshadowball, itmsbomb
`rust/melee_items/src/tu/{itmato,itmball,itmetalb,itmetamon,itmew,itmewtwodisable,itmewtwoshadowball,itmsbomb}.rs` port every
function of those `src/melee/it/kinds` files (171, incl. the static inlines) with the same `gen/kinds` state oracle (168 ops x
400 cases, hashed world + return + trace; the state tables too). The TARGET_PC `ScriptGame_*` branches of itmato are ported
(`ext` hooks). Notes: `Item_ItemVars` bitfield slots (mball b0/b1, msbomb xDD4_b0) are native-LSB-first in C vs MSB-first in
melee_types, so the cfg tweaks draw them by value and the hash clears the slot; variadic `efAsync_Spawn` / `efSync_Spawn` are
fixed-arity custom stubs; `it_802C4D10` has no `return` in the C (port returns 0, oracle skips it). Fused sites (checked with
`fma_sites.py`): msbomb `pos.y += scl*(top-bottom)`; shadowball `x30.y/z += (x40*x50)*sin/cos`, the five `charge*((hi-lo)/max)+lo`
lerps and `0.5*(x18/x1C)+0.5` (`fmadds`), and `x3C += (M_PI/4)*(2*(rand-0.5))` (f64 `fma`). Mutation checks (unfusing the
msbomb and sinf sites, a constant, a comparison boundary) fail the tests; the f64 `x3C` fma and the exact `0.5*q+0.5` lerp are
indistinguishable after the f32 store, as with itbox's f64 sites. Regenerate: `bash rust/melee_items/gen/kinds/regen.sh itmato ...`.

## melee_items::tu: it/kinds set 2 (shard 777-784)
`itporygon2 itrabbitc itraikou itrshell itsamusbomb itsamuschargeshot itsamusgrapple itsamusmissile` ported with the
shard 657-664 generator (`bash rust/melee_items/gen/kinds/regen.sh [stem...]`; tests `tests/kinds_<stem>.rs`, 400
seeded cases per function, every function of each TU incl. the state tables).
- Traps: (1) `bool` parameters are C `int` (`typedef int bool`), so extern fns take `i32`; (2) `typedef`-less local
  records (`itRShell_Attrs`) are plain `#[repr(C)]` in the port; (3) raw-byte comparison of unions that hold a pointer
  or a bitfield differs between the two executables (address bytes, LSB- vs MSB-first): `itrshell` clears the item
  var union / flag byte after hashing the fields by value, `itsamusgrapple` does the same for `Fighter.u` and
  `HitCapsule.owner`/`hit_grabbed_victim_only`; (4) `itSamusChargeshot` item vars `xDEC/xDF0/xDF8/xDFC` are ints, so
  float values are converted when stored; (5) `ftSs_DatAttrs.xA0/xB0/xC0` are ints compared to a float.
- `itsamusgrapple.c` reads two packed bitfields of the hitbox command by byte offset (`*(u16*)(data+2) & 0x3FF`,
  `(flags+3)->hit_grounded`). On the GameCube (MSB-first) those are the `damage` and `hit_grounded` fields; the port
  reads them by name and the oracle compiles a copy of the TU with the same two substitutions (`patch` in
  `gen/kinds/specs.py`). The world is custom (`gen/kinds/specs_cfg_grapple.py`): a 2-6 link chain wired into
  `ItemLink` pool entries, an owner fighter with `ftSs_DatAttrs`, a bone table, the beam attributes and a hitbox
  record; the pool stubs (`GObj_Create`, `HSD_ObjAlloc`, `HSD_JObjLoadJoint`) hand out pool entries.
- Fusion: each contracted `a*b + c` is `fmadds` (f32) or `mul_add` (f64); mutating the sites in missile/bomb/
  chargeshot/rshell/grapple fails the tests except where the product is exact (`1.0f * x`).
## melee_items::tu: it/kinds set 2 (shard 785-792)
`rust/melee_items/src/tu/{itscball,itseakchain,itseakneedleheld,itseakneedlethrown,itseakvanish,itsonans,itspycloak,itsscope}.rs`
port every function of those `src/melee/it/kinds` files (static inlines included) with the C names and signatures, using the
shard 657-664 oracle (`bash rust/melee_items/gen/kinds/regen.sh itscball ...`, tests `tests/kinds_<stem>.rs`, 400 seeded cases
per function). New fixtures: a 16-link `ItemLink` pool + `Fighter` behind `GO[2]` for itseakchain (`ptrargs` in `mk_driver.py`
maps `ItemLink*` / attribute-buffer parameters; pool allocators `GObj_Create` / `HSD_ObjAlloc` / `HSD_JObjLoadJoint` are stubs
returning pool entries; `PSMTX*` stubs have matrix effects). Notes: `ABS(table[HSD_Randi(8)])` evaluates its argument twice
(two draws); the `it_802BAF2C` result is `(int) pointer` and is not compared; float-to-int of `xC[]` in itsscope follows
gwtool's saturating cast. A mutation of the fused comparison `v < -x18 * s - x28` in `it_802BC080` is not caught.
## melee_items::tu: it/kinds set B (shard 793-800)
`rust/melee_items/src/tu/{itsscopebeam,itstar,itstarrod,itstarrodstar,itsuikun,itsword,ittaru,ittarucann}.rs` port every
function of those `src/melee/it/kinds` files (the same pipeline and `ext` convention as shard 657-664; `gen/kinds/splice.py <stem>`
pastes the generated `ext` block into a port). State oracle per file (`bash rust/melee_items/gen/kinds/regen.sh <stem>`, then
`cargo test -p melee_items --test kinds_<stem>`), 400 cases per function plus the state tables.
- MSL `bool` is `int`: a C function that `return`s a callee's int gives the raw value (e.g. 12), the port a Rust `bool`; mk_driver
  now compares `(call) != 0` for functions declared `bool`.
- `ittarucann` / `ittaru` / `itsscopebeam` fused sites (checked by mutation): `+=` of a product, `a*b + c` (`fmadds`); the product stored
  to `temp` first in the `*_Spawned` functions is NOT fused. `8.0f * cosf/sinf + x` and `2.0f * angle + y` are exact either way.
  `itTarucann_UnkMotion7_Phys` returns a `float` (state-table slot is cast); `itTarucann` stubs `sinf`/`cosf` are the real gekko ports.
- Oracle additions: `facing_dir` also drawn as an arbitrary float; a fake `ItemModelDesc`/`HSD_Joint` (`MD`, `JNT`) for the
  `x10_modelDesc->x0_joint` reads; `ftCo_800C92E4`'s `lbColl_80008D30_arg1` pointee is traced.
## melee_items::tu: it/kinds set 2 (shard 809-816)
`itwhitebea itwstar ityaku ityoshiegglay ityoshieggthrow ityoshistar ityoshitongue itzeldadinfire` (165 fns, all with a
state-oracle op each) ported with the shard 657-664 generator (`gen/kinds`; descriptions in `cfg_809.py`, specs in `specs.py`).
New in the oracle: `efAsync_Spawn` modelled as a 6-argument hook (the variadic scale is read only for spawn kind 4);
`__frsqrte`/`sinf`/`cosf` are the real linked ports; callback fixtures (ityaku) are registered functions.
Fused sites (IR): yoshistar `vel.x += accel*dir`, yoshiegglay timer `-= xCA0*x0`, zeldadinfire scale and steering,
whitebea distance. The test catches the yoshistar mutation; the whitebea distance (only compared against thresholds)
and zelda steering (rarely reached) are not discriminated by the current inputs.
`x28_entries` (itwstar) and the yaku `while (true)` hang are handled as noted in the file headers.
## Shard 881-888: pl/player.c plattack.c plbonus.c plbonuslib.c, mn/mn_22EC.c, mp/mpisland.c mplib.c mpcoll.c
No oracle was run for this shard (no C oracle, no cargo test): the ports are checked by construction.
- `melee_fighter::tu::{player,plattack,plbonus,plbonuslib}` and `melee_modes::tu::mn_22EC` are hand ports with the C
  names and signatures (`bool` is MSL `int`, so `bool` parameters that the C compares with `== 1` are `i32`). `player_slots` /
  `ftMapping_list` / `unk_804D6480` are the existing statics (`crate::stale`, `crate::player`) re-exported under the C
  names. `pl_800386D8` / `pl_8003E70C` / `pl_8003E7D4` index past the member they name (into `x504..`, `x848[]`), so
  they use byte-offset pointer reads. Fused sites (`fma_sites.py`): plbonuslib `pl_8003ED0C` (1), `fn_8003F294` (1),
  `fn_8003F654` (3), `pl_8003FAA8` (1); everything else in these files has none.
- `melee_world::tu::{mplib,mpcoll,mpisland}` are generated: `python rust/melee_world/gen/mtu/c2rs.py` translates the
  functions of one C file with libclang (the AST carries every implicit conversion, so the integer / float / pointer
  semantics are the C ones; ints use plain ops because the workspace builds with overflow-checks off (-fwrapv),
  `/ % <<` with variable operands go through `melee_runtime::cabi`, float->int uses the saturating `as`). Contraction
  follows clang's `-ffp-contract=on` rule (a `+`/`-` whose left, else right, operand is a same-type multiply, or the
  negation of one, becomes `fmadds` / `mul_add`); `--check` compares the per-function count with `tools/rust-port/fma_sites.py`
  (all translated functions agree; the hand ports and the `sqrtf` alias are covered by hand), and spot checks against
  the LLVM IR (`-Xclang -disable-llvm-passes`) agreed on operand order and negation. `bash rust/melee_world/gen/mtu/regen_mp.sh`
  regenerates the three files (heads in `gen/mtu/mp/`, goto state machines the tool cannot structure in `gen/mtu/hand/`:
  `mpIsland_8005AE1C`, `mpLib_80053DA4_Floor`, `mpLib_80053ECC_Floor`, `mpLib_80056C54`). Forward / backward / loop-entry
  gotos (`block_8`, `again`, `after0`...) are turned into labeled blocks / loops by the tool.
- Traps: (1) `(CollLine*) ((int) groundCollLine + line_offset)` is a PPC-ism (pointer through an `int`); it is a byte offset
  from the base pointer here, and `line - groundCollLine` is `offset_from`. (2) `mpLib_80056C54` passes `sqrt_tmp - 4` as the
  volatile store slot of `sqrtf_store` (stack layout hack): a scratch float here. (3) `HSD_ASSERT` / `__assert` panic;
  `OSReport` diagnostics go to `ext::OSReport(&str)`. (4) The TARGET_PC script hooks (`ScriptGame_Stage*`) are ext hooks
  like every other callee outside the three files (unset: panic "TODO"); `ScriptGame_ModValue` (player.c) falls back to
  the crate's neutral `SCRIPT_MOD_VALUE` hook. (5) `lbVector_Add/Diff`, `PSMTXMultVec`, `sinf`, `cosf`, `sqrtf` and
  `PSVECNormalize` call the existing `melee_runtime` ports.

## Status recount and next plan (2026-10-24, after shard merge, platform nights 3/4 and resolve passes 1-3)

State: `rust-port` has ~1000 shard merges, platform-2/3 (audio wiring, Akaneia Wolf over gdmods) and three stub-resolve
passes (4041 + 155 + 50 stub sites rewired to existing ports, compile-checked). `port/shard-665-672` is the only
unmerged shard branch (conflicts in `melee_items/gen/mtu/stems.py` and `src/tu/mod.rs`); rebase it onto rust-port and
redo the generated parts with `gen/mtu`. The earlier "unmerged" shard list was a stale merge-in-progress that no longer exists.

### Recount of ported functions per group (name-level, supersedes the older table rows)
Method (`recount.py`, scratchpad): distinct C function-definition names in the group's `src/` dirs (regex) intersected with
distinct Rust `fn` names in the crate's `src/` (excluding `raw/`, `c2r/`, and bodies containing `todo!`/`TODO(`). A name
match is an upper bound: it counts hook-wrapper stubs that forward to a port or to an unset hook, and duplicate ports
count once. "Verified" (bit-for-bit vs the C via a per-shard oracle) is not recounted here; every `tu/` shard port has a
stateful oracle except shards 881-888 (checked by construction) and the resolve rewires (no test run, per their rule).

| group | crate | C defs | name-matched ports | share |
|---|---|---:|---:|---:|
| 1 runtime/HSD/lb/db/sfx | melee_runtime | 2175 | 1778 | 82% |
| 2 world (mp cm ef gr if) | melee_world | 3511 | 3208 | 91% |
| 3 fighter engine + ftCommon + pl | melee_fighter | 3242 | 2135 | 66% |
| 4 characters (ft/kinds) | melee_chara | 6075 | 4202 | 69% |
| 5 items/ty/vi | melee_items | 4966 | 4704 | 95% |
| 6 modes (gm mn sc) | melee_modes | 2802 | 2647 | 94% |

Caveats: g3/g4 definition counts include static inlines and per-char duplicates, so their share is soft; ~1,420 hook/ext
stubs still panic when called (about 930 `hook!`, 440 `ext_fns!` TODO(port), 58 TODO(pc script)); the ported-name figures
above include some stubs forwarding to those. Resolve passes did NOT dedupe duplicate ports (e.g. `Ground_*` in `tu_ext` and
`tu::ground`) and did not run tests or goldens, so behaviour of the 4,200+ rewired sites is unconfirmed until the
checklist below is done.

### Build/test state at this commit
`cargo build --workspace` passes. `cargo test --workspace --no-fail-fast`: ~791 test binaries, ~990 tests; fixed here:
(1) every `melee_modes` test binary and the `melee` bin exited 255 before main: `tu/script_game.rs` (c2rust leftover) put its
static initializer in `.CRT$XIB` (an int-returning C initializer list on MSVC), now `.CRT$XCU`; (2) three `lbAudioAx`/`gmMainLib`/`lbMthp`
hooks in gmmovieend/gmomake15/gmopening/gmmultiman/grvenom had become safe `fn` types after the resolve rewire (back to
`unsafe extern "C" fn`); (3) shard 985-992 left `thp_state/`, `tu_card.rs` and its oracle scripts untracked (copied in);
(4) `tu_ftco_0c60`/`0dc2` should_panic strings updated (rewired stubs now stop at `p_ftCommonData not loaded` /
`PcTraceMotionEnabled`); (5) `lbtu_lbspdisplay` disabled with `#![cfg(any())]` (its expected file was never generated; `run_tu.py` fails on `__builtin_va_arg`).
Still failing (checked: not caused by the resolve passes, `tu_grfourside` fails identically at 7d3fb0087; the rest not bisected):
`melee_modes` `oracle_ptr` and `oracle_r5` (native sizeof of `un_804D6EF4_t` 184 vs C 176, `TyDisplayData` 688 vs 672: oracle C
uses 4-byte pointer slots where the generated type has `*mut HSD_Text`), `tu_gm_1798`, `tu_lb_020A`, `mngallery_vs_c`;
`melee_world` `tu_grfourside`, `tuo_grvenom` (388/5736), `tuo_grzakogenerator`; `melee_runtime` `tu_sobjlib`. Fix these first.

### Platform status
Audio runs inside the native sim (banks + BGM by id, `golden --audio`); mods mount over the disc and Akaneia Wolf runs as P2
(onLoad/onFrame only). Golden `pad_falcon_fox_fd` and `pad_falcon_wolf_fd` write 216 frames then stop at `ftCo_Walk_IASA`
with a button (not ported). Known issue: `mex_import` `tests/ppc_oracle.rs` fails 4/6000 cases in `--release` (passes debug);
not confirmed whether pre-existing.

### Next plan, in order
1. Run the checks the resolve passes skipped: `cargo test --workspace` (this commit fixes the one compile break found:
   three `lbAudioAx_80024E50`/`gmMainLib`/`lbMthp` hooks in `melee_modes::tu::{gmmovieend,gmomake15,gmopening}` had become
   safe `fn` types), then `melee golden` on all scenarios against `docs/rust-port/golden/*.csv`. Bisect any divergence
   against the rewire commits `fa3889307`, `455fb1b42`, `4f50a2dd3`.
2. Recursion/loop audit of rewired sites (27 same-name candidates found in pass 1) and dedupe duplicate ports
   (pick one per C name, delete the rest).
3. Port `ftCo_Walk_IASA` and what follows it so the golden runs past frame 216; extend to the other three goldens.
4. Unblock the ~300 wrong-arity and ~95 cross-crate stubs: global `HSD_Randi/Randf` state, `GObj_Create` host API,
   `efSync_Spawn`; the dependency cycle fighter/chara/world/modes needs a trait/hook layer or merged crate.
5. Merge `port/shard-665-672`.
6. Mod track: bind the 536 unbound vanilla calls and 6 m-ex helpers, give r2/r13 backing memory, wire Wolf's remaining
   callbacks; fix the GrSv/GrBx/GrMVb/PlSn type-guess blockers; investigate the release-mode ppc_oracle diff.
7. Audio leftovers (ducking, positional SFX, boot banks, 1P branches) and the user-only checks (visuals, audio by ear, adapter,
   netplay).
## melee_items::tu: it/kinds set (shard 665-672)
`itclimbersice itclimbersstring itclinkmilk itcoin itcrazyhandbomb itdkinoko itdosei itdrmariopill`: every function
(incl. static inlines) checked by the mtu state oracle (`python rust/melee_items/gen/mtu/run_tu.py <stem>`, driver from
`mk_driver.py`, per-TU fixtures in `gen/mtu/stems.py`, stub effects in `specs.py`). Header inlines (HSD_JObj setters,
spawn helpers) are local ports via the `jobj_inl!`/`spawn_inl!` macros in `tu/mod.rs`; every out-of-TU callee goes through
`ext`. Traps: C `bool` is `int` (pass-through returns are `i32`), `ext` callbacks need `ext::cb::*` (C ABI), itemVar unions
holding pointers or aliasing bitfield members are hashed by field and cleared after each op, `HSD_ASSERT` is an infinite
loop in the C driver so fixtures clear JOBJ_USE_QUATERNION. The double-precision Newton steps of `my_sqrtf` cannot be
told apart from unfused ones through the f32 result.

## Merge of port/shard-665-672, failing-target fixes, stub audit (2026-10-24)

- `port/shard-665-672` merged (conflicts in `melee_items/gen/mtu/{mk_driver,specs,stems}.py` and `src/tu/mod.rs` resolved by keeping both sides).
- The nine failing targets pass. Causes: (1) oracles compiled the game C with 4-byte slots where melee_types widens
  trophy slots to native pointers; `melee_types/gen/native_hdr.py` now writes shadow headers with the same widened slots
  (`-include`d by the `oracle_ptr` / `oracle_r5` builds); (2) `tu_mngallery` expected data lacked `float_order_helper` rows (regenerated);
  (3) `tu_lb_020A`: the C driver still masked `HSD_IDKey` to u32 (now pointer-width) and `(u64) fv() << 32 | fv()` was
  evaluated right to left (`fv64`); (4) `tu_gm_1798`: tuoracle drew pointer-width ints as 32 bits in C but 64 in Rust;
  (5) `tu_grfourside`: a stale half pointer (low half overwritten by a 32-bit view) depends on the image base, now null on both sides;
  (6) `tuo_grvenom` / `tuo_grzakogenerator`: pool objects now `LOWMEM` (same addresses on both sides; the result changed from run to run),
  and `PTR_MASK` ignores bits 0/7 of a pointer's low byte aliased by a bitfield (b0 is bit 7 on the GameCube, bit 0 native);
  (7) `tu_sobjlib` found a real port bug: `origin_x` divided by 3.0 instead of 2.0.
- Stub audit: no infinite recursion among the 2,552 `None => port` fallbacks (cycle search over wrapper edges and over the call graph;
  recursive targets are genuine tree walks). 149 rewired names also have another port; the clear pure duplicates (7 legacy melee_modes
  ports, 1 melee_fighter port) now forward to the TU port; the rest differ in signature (`efSync_Spawn`, `efAsync_Spawn`, ...) and stay.
- `docs/rust-port/missing.md` lists every remaining panicking stub with its NO_PORT / SIGNATURE / CYCLE tag.

## melee_chara stub fill (port/fill-melee_chara, 2026-10-24)

`docs/rust-port/missing.md` lists no panicking stub for `melee_chara` any more (was 319 sites / 188 names).

- Adapters: `melee_chara::adapt` converts the call shapes the character TUs declare to the ports below them; `port_hook!` (lib.rs) is a settable hook that runs the adapter when unset. Rust-`fn` callbacks go through a trampoline slot (the ports call them before returning).
- New ports: `melee_runtime::tu::jobj_ptr` (jobj.c on raw `HSD_JObj*`: dirty/flags/DObj list/JObjUpdateFunc/anim walkers/inlines), `tu::mex_data` (m-ex/skins/replay data services; vanilla answers until the host installs `MEX`/`SKINS`), `tu::random::GLOBAL` (the C's one `seed` for callers with no state argument; the sim must use it, or sync it, to stay on one stream), `tu::os_report`; `melee_fighter::tu::{ftcoll, ftmaterial, ftCo_09f4, ftparts_render}`; `melee_world::cm::camera_boss` + `NativeEfHost`; six chara TUs (Samus x3, Crazy Hand x3).
- Moved below `melee_modes` (re-exported there): the hit-rule capsule table, the movement-caps jump limit, the VS scene controller global.
- Verified against the C (shipping contraction): `melee_fighter/tests/ftcollset_ref.rs` (ftcoll setters + hit-rule table, 4400 cases), `melee_world/tests/camera_boss_vs_c.rs` (5500), `melee_runtime/tests/jobjptr_vs_c.rs` (6000; found a wrong RObj type constant), six state oracles in `melee_chara/tests/oracle_ft{samus,samusspeciallw1,samusspecialn,crazyhandcancel,crazyhandcapturewaitcrazyhand,crazyhandfingerbeam}.rs`, `melee_runtime/tests/mex_data.rs` (the C's branches, no in-process C). Not verified: GX-bound code (`ftparts_render`, `ftmaterial`, light overlay), the jobj anim walkers' RObj/AObj/DObj legs, `NativeEfHost`, `script_caps`, the adapters (type conversions only).
- Traps: C evaluates call arguments right to left on the MSVC target (draw into temporaries in a C reference); a union aliasing a bitfield differs natively from the GC (hash only the members the function wrote); a pointer stored in a union occupies 8 bytes natively.

## melee_fighter stub fill (port/fill-melee_fighter, 2026-10-24)

Stub names in `docs/rust-port/missing.md` for melee_fighter: NO_PORT 276 -> ~71, SIGNATURE 302 -> ~55, CYCLE 188 -> ~151 left.
- New ports with C oracles: `tu/ftcoll.rs` (ftcoll.c; ops 0-9 = ftColl_80079AB0/80079EA8/80079C70/80076528/800765F0/80076640/8007AC68/8007A06C/8007699C/80076CBC,
  20000 cases, `tests/tu_ftcoll.rs`; ftColl_80076ED8 is ported but its C driver crashes on the stubbed callees, so unverified), `tu/ftCo_0A01.rs`
  (CPU stick/trigger/buttons, IsCpuControlled, 800A0DA4; 18000 cases).
- `platform.rs`: the pc/platform layer (replay, snap, rollback, m-ex dispatch tables, Geno registry, UCF history, settings) in its
  default configuration, hook-overridable; no oracle (read off the C). `bridge.rs`: native-pointer adapters over runtime baselib math and
  C-callback trampolines. `chardata.rs`: kirby/koopa/yoshi attribute accessors moved down from melee_chara (chara still has its copies).
  `rng.rs`: global HSD rand state.
- Left: GX calls (ftafterimage), ftParts/ftCo_09F4/ftCo_800B3900 (CPU AI, JObj class setup), Camera/efAsync/GObj world calls, ScriptGame_*/Geno_*/gm_* (state
  lives in melee_modes), ftMh/ftCh/ftSs/ftKb chara functions that need chara internals.
- Oracle drivers: `__assert` no longer hangs (`asrt_hit`), `-Wl,/force:unresolved` link; clang evaluates call arguments right to left in some cases
  (see tests/tu_ftcoll.rs), mirror it in test fills.

## Golden round 1: pad_falcon_fox_fd matches in full (2026-10-24)
- `melee golden --scenario pad_falcon_fox_fd` is identical to `docs/rust-port/golden/pad_falcon_fox_fd.csv` on all
  1801 frames (0..1800; `compare_golden.py ... --allow-short` exit 0). The sim then stops on the first unported
  state the script reaches past the golden (`ftCo_RunBrake_Enter`, frame 1802). Start of the round: 216 frames.
- Ported in `rust/melee/src/vs_match.rs` and `vs_match/{rebirth,fox,attackair,dashrun}.rs`: blast-zone KO and DeadDown;
  respawn (Rebirth / RebirthWait, the platform path of `Fighter_UnkProcessDeath`, `fn_8016758C` spot countdown, the
  spawn probe); shield (GuardOn / Guard / GuardOff, power shield, upkeep); jumps (KneeBend, Jump, JumpAerial,
  FallAerial); Fox neutral-B in the air (gun and laser items, muzzle flash); aerial attacks (stick / C-stick
  selection, Anim / IASA / Phys / Coll, landing through `landing_enter_basic`); Dash and Run; the player nudge
  (`ftCommon_8007E0E4`, radii `*ftData->x50` into `fp->x2C4`).
- Traps found (each cost a bisect):
  - 1-ulp position differences are almost always a collision substep in the C (ECB Clear / locked ECB, or an
    item spawn moving > 6 units: `it_80276100` -> `mpColl_800471F8` adds `vel / steps` per substep). Compare the
    value rounded to the coarser exponent grid before suspecting velocity.
  - `ftCo_800CC988` runs after `ftCo_Fall_Anim_Inner` every frame the fall blend is nonzero; poses feed the ECB
    (and the Rebirth spawn probe's locked ECB bottom), so a wrong pose shows up only at a landing or a respawn.
  - `Fighter_ChangeMotionState` clears `x221D_b5` (the "no nudge" flag Rebirth sets).
  - Debugging the C: temporary `OSReport` in a game TU, `tools/port/build.sh` (about 1 min incremental), `crun.sh`
    with `dump.lua` (Lua `n` = golden frame + 1 but `gd.player` x reads mid-frame; trust the hash columns).
    OSReport lines of per-frame call sites can be dropped by the log rate limit: print only a rare condition
    (an item kind) instead. Revert the TU afterwards (`git checkout src`).
- Not ported (the next stops): RunBrake / TurnRun / smash turn, Z grab and every hit (`hit_guard` stops the run),
  Fox ground neutral-B, Falcon's other specials, platforms and ledges, shield break, `x2135 != -1` respawn stages.
- Next: the other three goldens are not wired (`VsMatch::new` supports `GrNLa.dat` only; `golden()` has no Marth,
  Sheik, Battlefield `grbattle.c`, or CPU input). Order of attack: Battlefield stage init and its materials ->
  Marth / Sheik file and effect-bank loaders -> the states `pad_fixed.lua` reaches first.

## State snapshots (resume the golden sim mid-match)
`MELEE_SNAP_DUMP=<frame>[,..]:<file>` makes the C build dump its gameplay state after a match frame;
`melee golden --from-snapshot FILE --start N` resumes the native sim from it and compares the hash from N
on. Dump/resume/proof commands, what is loaded from where, and the model differences found:
`tools/rust-port/snap/README.md`. Proven on `pad_falcon_fox_fd` at frames 0 and 300.

## melee_world::tu::efn: ef/eflib.c, efasync.c, efdata.c via c2rs + state oracle (pre-melee_world)
- `bash rust/melee_world/gen/mtu/regen_ef.sh` regenerates `src/tu/efn/{eflib,efasync,efdata}.rs` (c2rs `--all-extern`: every
  function is `extern "C"` so a test can store any of them as a callback; then `gen/rewire/rewire.py --skip=lbArchive_80017040`).
  c2rs learned: variadic definitions (`args: &[*const c_void]`, `crate::tu::va::Va` for `va_list`; `va_arg` steps it like the
  `__va_arg` shim), goto-into-a-later-case-label inside a switch (flag + block after the match), `&arr[i]` as a wrapping
  pointer offset (C forms out-of-range addresses before checking them), a NULL `%s` in OSReport, pointer args of `%08X`.
- Oracle: `python rust/melee_world/gen/mtu/mk_ef.py all` (`dbg_ef.sh <op>:<case>` bisects): the three TUs as one native C TU
  (shipping contraction, `efSync_Spawn` call sites counted by a patch macro), 78 ops x 400 cases (every eflib/efasync function,
  Dispatch over all 144 case ids with the argument pointers each case reads, the fused `0.04f * x + 0.3f` site of case 0x3E8
  discriminated). Fighter is filled/hashed by the 14 fields ef reads (the full record has bitfield overlays the walker cannot map).
  Traps: asserts inside `extern "C"` ports abort the test (no unwinding), so the fixture never reaches an assert (GObj_Create
  never NULL, EffectCount < 64); `any` Dispatch ids avoid the case ids because their argument kinds are pointers of the wrong type.
- Camera stubs (Camera_Init/Create/ForgetGameCamera/RefreshViewingMtx/8002A4AC/8002F3AC/PCMainCObj/SetQuakeScale/LogicToScreen) of
  world, modes, items and fighter now default to `tu::cmcamera`.

## melee_chara trace verification (port/pre-melee_chara, 2026-10-24)
`docs/rust-port/trace/pad_falcon_fox_fd.csv` listed no stub or missing `melee_chara` function; the 10 `ported-unverified`
rows now have C oracles (all equal, rows flipped to `ported+verified`): `ftPr_Init_8013C2F8`, `mpColl_8004730C`,
`ftCa_SpecialHi_800E3EAC`, `ftCa_SpecialLw_Coll/Anim`, `ftCa_SpecialLwEndAir_Phys`, `ftFx_SpecialAirNStart_Anim/IASA`,
`ftFx_SpecialN_CreateBlasterShot`, `ftFx_SpecialAirNLoop_Anim`.
- Build: `python rust/melee_chara/gen/turef/tu_pre.py [stem]` (same machinery as `tu_shard89.py`: the game `.c` is `#include`d
  verbatim, shipping contraction, recording stubs for callees, 4000-6000 seeded cases to `tests/data/tu_<stem>.txt`);
  tests are `tests/tu_{ftcaptainspeciallw,ftfoxspecialn,ftpurin,mpcoll}_c.rs`.
- `tu_mpcoll_ref.c` includes `mplib.c` + `mpisland.c` + `mpcoll.c` together and loads a synthetic stage with `mpLibLoad`; it is
  the first oracle of the generated `melee_world::tu::{mplib,mpisland,mpcoll}` (it also exercises mpCollPrev,
  mpColl_80042C58, the whole 80043754 loop, floor / wall / ceiling passes and the joint callbacks). The image is linked
  below 4 GB because those files route pointers through `int`. Trap: a group named `LeftWall` has normal -x (hit while
  moving right), i.e. it is the stage's right-hand wall.
- Write C driver edits with the Edit tool, not a Python string: `\n` inside a Python literal puts a real newline in the C.

## melee_world trace verification (port/pre-melee_world, 2026-10-24)
All 93 `melee_world` rows of `docs/rust-port/trace/pad_falcon_fox_fd.csv` that were stub, missing or ported-unverified are now
`ported+verified`: camera.c (`tu::cmcamera`, `tests/tu_cm.rs`), eflib/efasync/efdata (`tu::efn`, `tests/tu_ef.rs`) and
mplib/mpcoll/mpisland (`tu::{mplib,mpcoll,mpisland}`, `tests/tu_mp.rs` plus melee_chara's `tu_mpcoll_c`) each have a state oracle against the
shipping-contraction C. `Camera_SetQuakeOffset` / `Camera_SetBackgroundColor` of grlib, grlast, ground and grpura now default to
`tu::cmcamera` (they wrote the hand-port `cm::camera::world()` state, a different `game_camera` from the one tu::cmcamera renders).
Rows that are static helpers (`update_bounds`, `cm_script_update`, `fn_801C8EF8_inline`, ...) are verified through their callers' ops.

## melee_fighter trace verification (port/pre-melee_fighter, 2026-10-25)
`docs/rust-port/trace/pad_falcon_fox_fd.csv` listed 225 `melee_fighter` rows as stub / missing / ported-unverified. 176 were already
covered by the committed ref/state oracles (fighter, ftanim, ftaction, ftcoll, ftdynamics, ft_081b/0c31/0c88/0c8c/0d31/0d4d, ftparts,
ftCo_09f4, ftCo_Turn, player/stale, ftcommon) and were only never flipped; this pass flipped them and closed most of the rest:
- `tests/tu_ftcpuattack.rs` (ftCo_800B9704), `tu_player_vs_c.rs` (ops 144-146 on `gen/cref/player2_ref.c`; `player_ref.c` / `player.txt`
  stay at 144 ops for the crate-level `player_ref.rs`), `ftleaf_ref.rs` op 50 (mpClearFloorSkip), `Camera_ScriptState` seam of fighter.rs now
  defaults to melee_world's oracle-checked `tu::cmcamera` port.
- Unity oracles (`MTU_UNITY=1`, `gen/cref/tu/*_unity.c`): `tu_plbonus_vs_c.rs` (player.c + plattack.c + plbonus.c + plbonuslib.c + ft_0892.c vs
  `tu::plbonus` / `tu::plbonuslib`), `tu_ftdrawcommon_vs_c.rs` (ftdrawcommon.c + mtx_pc.c; `script_motion_draw_begin` is now a real port),
  `tu_crowdsfx_vs_c.rs`. `gen/mtu/stubs.py` takes several port modules (`a.rs,b.rs,c.rs` -> `m`, `m2`, `m3`), the `=>` ext form and
  `[f32; 4]` return types; `gen_tu.py` treats every #included game `.c` as the TU under `MTU_UNITY`.
- `tu_ftdata_vs_c.rs` (spec from `gen/mtu/mk_ftdata_spec.py`): ft_800852B0, ftData_800855C8, ftData_80085B10, ftData_80085CD8 and the
  pc/geno `GenoDefine_InitKinds / CaptureBase / RestPoseTree / BindFighter` / `ResetDescriptors`, which are now real ports in `tu::ftdata`
  (the platform hooks default to them; the registry queries `Geno_Define*` are `ext` hooks that default to "no define registered").
  Every per-kind table of the TU is filled and hashed (pointer-sized words by pool id). Not covered: the `Geno_DefineThrownAs` stand-in
  legs of ftData_80085B10 (`as != kind` calls ftData_ThrownStandInPrepare -> ftData_8008572C; the oracle returns -1 or the kind).
- `tu_ftparts_class.rs` (JObjInfoInit, IntpJObjInfoInit, PObjInfoInit, ftParts_80073758, IntpJObjLoad, JObjMakePositionMtx against the real
  class.c / mtx.c / mtx_pc.c; parent classes synthetic), `tu_ftparts_pobj.rs` (ftParts_PObjSetupMtx z-scale paths).
- Traps: a Python string with `\n` written through the tool's shell lands as a real newline in the C (use the Edit tool); the oracle C is
  native, so `size_t` is the 4-byte `gc_size_t` while Rust `usize` is 8 (hash `(u32)`); native x86 bitfields are LSB-first, so raw-byte
  readers (`FT_DRAW_FLAG` reads `x21FC_flag.byte`) need the byte set on both sides after the field-wise fill; fused sites of
  `fn_8003F654` / `fn_8003F294` / `script_motion_draw_begin` mutation checks: only the last is discriminated by the generated inputs.
## Trace status regenerated after the pre-melee_* merges (2026-10-25)
`rust_status`/`rust_crate` of `docs/rust-port/trace/pad_falcon_fox_fd.csv` were re-derived against the current tree with the
`calltrace.py` rules (`index_rust` + `tested_names` + `missing.md`; the function list and counters are unchanged, 2625 rows).
Status is name-level: "verified" = a real-bodied Rust fn plus a `rust/*/tests` file naming it, so it also counts
delegating wrappers; the per-agent notes above are the finer-grained claim.

| crate | ported+verified | ported-unverified | stub | missing | total |
|---|---:|---:|---:|---:|---:|
| melee | 13 | 2 | 0 | 0 | 15 |
| melee_audio | 1 | 0 | 0 | 0 | 1 |
| melee_chara | 88 | 0 | 0 | 0 | 88 |
| melee_fighter | 524 | 137 | 2 | 6 | 669 |
| melee_items | 160 | 1 | 0 | 0 | 161 |
| melee_modes | 447 | 0 | 0 | 0 | 447 |
| melee_runtime | 723 | 41 | 1 | 0 | 765 |
| melee_types | 1 | 0 | 0 | 0 | 1 |
| melee_world | 475 | 3 | 0 | 0 | 478 |
| **total** | **2432** | **184** | **3** | **6** | **2625** |

Before: 1989 verified, 525 unverified, 55 stub, 56 missing (melee_runtime 378/314/39/37, melee_fighter 444/190/16/19). The
remaining 9 stub/missing rows are melee_fighter (8) and melee_runtime (1). Note the earlier melee_runtime report counted 41 unported audio/devcom rows; the CSV
now reads them as ported-unverified because same-name Rust fns with real bodies exist (name-level, not a test claim).

### melee_fighter trace rows, last six (port/pre-melee_fighter)
`ftParts_PObjSetupMtx` (`tests/tu_ftparts_pobj.rs`, 4000 cases; the C reference evaluates the mark arguments in a fixed order: the
MS-ABI argument order is unspecified, so a call with two `rnd()` arguments diverged), `ftParts_8007482C` (op 6 of `tu_ftparts_class`;
`src/tu/ftparts_render.rs` `ext` hooks for HSD_JObjSetDefaultClass / HSD_JObjLoadJoint, stand-ins record the default-class bracket) and
`ftMaterial_800BF260 / 2B8 / 534 / 6BC` (mtu state oracle `tests/tu_ftmaterial_vs_c.rs`, spec `gen/mtu/specs/ftmaterial.json`, 4 ops x 600
cases over the real ftmaterial.c; the TU's callees are recording stubs and `src/tu/ftmaterial.rs` gained an `ext` seam; `ftMaterial_800BF2B8`
is `extern "C-unwind"` so the C `__assert` longjmp has a Rust panic counterpart; `x61A_controller_index` is held to 0..6 because the
C reads `p_ftCommonData->x6D8[idx]` unchecked). Not covered: the lbRefract / LObj / TExp / GX callees themselves (verified in their own TUs).

## Trace status after all six pre-melee_* merges (2026-10-30)
`rust_status`/`rust_crate` of `docs/rust-port/trace/pad_falcon_fox_fd.csv` re-derived with the `calltrace.py` rules (2625 rows). Rows already
`ported+verified` are never downgraded: the name-level rules would drop 12 rows that agents verified with C oracles (static inlines such as
`ftCo_GuardOn_Anim_inline`, `ftDrawCommon_80080E18_surface_impl`, and wrappers without a test naming them). Only the crate of 9 rows changed.
Open: melee_runtime 41 ported-unverified (axdriver, lbaudio_ax, devcom, lobj, hsd_3933, hsd_3A76, particle `ref_INC`), melee_world 3,
melee 2, melee_items 1; stub 1 (`HSD_AudioSFXSetPitchFid`, macro-defined body, actually ported); missing 0.

| crate | ported+verified | ported-unverified | stub | missing | total |
|---|---:|---:|---:|---:|---:|
| melee | 12 | 2 | 0 | 0 | 14 |
| melee_audio | 1 | 0 | 0 | 0 | 1 |
| melee_chara | 86 | 0 | 0 | 0 | 86 |
| melee_fighter | 677 | 0 | 0 | 0 | 677 |
| melee_items | 159 | 1 | 0 | 0 | 160 |
| melee_modes | 447 | 0 | 0 | 0 | 447 |
| melee_runtime | 720 | 41 | 1 | 0 | 762 |
| melee_types | 1 | 0 | 0 | 0 | 1 |
| melee_world | 474 | 3 | 0 | 0 | 477 |
| **total** | **2577** | **47** | **1** | **0** | **2625** |
