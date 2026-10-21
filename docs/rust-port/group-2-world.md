# Port group 2: world (mp, cm, ef, gr, if) -> `rust/melee_world`

The task named this file "group-2 world: map, camera, effects, stages, HUD.md". Windows does
not allow `:` in file names (NTFS reads it as an alternate data stream), so it is
`group-2-world.md` instead.

Status as of 2026-10-21 (first night). Only a small, tested slice is ported.

## Size

Function definitions in the C (counted as a `name(...)` line followed by `{`):

| dir | functions | ported |
|---|---|---|
| src/melee/mp | 226 | 105 (mplib line-graph queries, line geometry, segment checks, joint list, seam stitching + floor bounds; all of mpisland.c) |
| src/melee/cm | 182 | 0 |
| src/melee/ef | 67 | 0 |
| src/melee/gr | 2166 | 0 |
| src/melee/if | 372 | 0 |
| total | 3013 | 105 (about 3.5%) |

Group 2 has no PowerPC `.s` files (the repo's `.s` files are in libs/dolphin, MetroTRK and
Runtime). None of the group's structs are in the neutral hash field list
(docs/rust-port/neutral-hash.md says stage, camera and particle state are not hashed), so
no `StateHash` impls were needed yet.

## What is ported

- `mp::types`: native `MapLine`, `CollLine`, `CollVtx`, `MapLineRange`, `MapJoint`,
  `CollJoint`, `MapCollData`, `mpCollisionBox`, and the line/joint flag constants. Each one has
  a `gc_layout!` table with the GameCube offsets. The offsets come from the
  `/* +XX */` comments and the `ASSERT_SIZE`s in src/melee/mp/types.h, not from the clang ppc32
  record dump, because that tool is not written yet. Regenerate them once it is.
  - Natively, `CollLine::x0` is an index into the `MapLine` array instead of a `MapLine*`.
    `CollJoint` keeps raw `*mut c_void` for `HSD_JObj*` and `Ground*`, and an opaque fn pointer
    for the collision callbacks (gr/ft/it). Those stay stubbed until groups 1, 3 and 5 have the
    types.
- `mp::lines::GroundColl` replaces the mplib globals `groundCollVtx`, `groundCollLine` and
  `mpLib_804D64B4->line_count`. Ported, from src/melee/mp/mplib.c:
  mpLineGetNext, mpLineGetPrev, mpLine{Next,Prev}Non{Floor,Ceiling,LeftWall,RightWall} (8),
  mpLib_80053394/80053448/800534FC/800536CC_Floor, mpLib_8005389C/80053950/80053A04/80053BD4_Ceiling,
  mpLib_80053DA4/80053ECC_Floor, mpFloorGetRight/Left, mpCeilingGetRight/Left,
  mpLeftWallGetTop/Bottom, mpRightWallGetTop/Bottom, mpLineGetV0Pos/V1Pos, mpLineGetKind,
  mpLineGetFlags, mpLib_80054D68, mpLineGetNormal, mpLib_80054ED8, mpLinesConnected.
  - The PC build's `ScriptGame_StageLineActive` checks are a hook,
    `GroundColl::line_active` (the default treats every line as active). Each check sits exactly
    where the C has one, and is missing where the C is missing one: the `...Vtx` inlines and
    `mpLineGetNextInline` do not gate the `*_id0` fallback.
  - `LINEID_CHECK` and the OSReport-then-spin become panics.
- `gekko::frsqrte` / `force25` and `vec::ps_vec_normalize`: ports of pc/gameworld/gekko_fp.c
  and the PSVECNormalize in mtx_pc.c (one Newton step, which is not correctly rounded).
  `vec::Vec2`/`Vec3` are here too. All of these carry `TODO(move-to-core)`: they belong to
  group 0/1 or to melee_core.

- (port round 1, night 2) `mp::geom`, from src/melee/mp/mplib.c: mpRemap2d, mpLib_8004DD90_Floor,
  mpLib_8004E090_Ceiling, mpLib_8004E398_LeftWall, mpLib_8004E684_RightWall, mpLineIntersection,
  mpLineIntersectionH, mpLineIntersectionV, mpLib_8004ED5C, mpVtxGetPos, mpVtxSetPos,
  mpLineSetPos, mpLib_80056758, mpGetSpeed (14), plus `geom::sqrtf` (src/MSL/math_ppc.h sqrtf;
  `TODO(move-to-core)`, it duplicates melee_chara::pure::my_sqrtf).
  - The C's out-params become a return value (`Option<LineHit>`, `Option<(f32, f32)>`); `None`
    is exactly the C's -1 / false, where the C writes no out-param.
  - FMA sites follow `fma_sites.py`: mpRemap2d 6 f64, mpLineIntersection 7 f64,
    mpLineIntersectionH/V 1 f64 each, mpLib_8004ED5C 2 f32 (`fmadds(dx, dx, dy*dy)`), sqrtf
    `fma(-(g*g), x, 3.0)`; the four point queries, the setters and mpGetSpeed have none.
  - mpGetSpeed's `DbLevel >= DbLKind_DebugRom` range assert (debug ROM only) is not ported.

- (port round 2, night 2) `mp::check`, from src/melee/mp/mplib.c (21): mpScriptInvalidateBounding,
  mpCheckedBounding, mpBoundingCheck, mpBoundingCheck2, mpBoundingCheck3, mpUncheckBounding,
  mpCheckFloor, mpCheckFloorRemap, mpCheckCeiling, mpCheckCeilingRemap, mpCheckLeftWall,
  mpCheckLeftWallRemap, mpCheckRightWall, mpCheckRightWallRemap, mpLib_800511A4_RightWall,
  mpLib_800515A0_LeftWall, mpLib_8005199C_Floor, mpLib_80051BA8_Floor, mpCheckMultiple, mpCheckAll,
  mpCheckAllRemap.
  - The joint list (`groundCollJoint`, `jointListStart`, `didCheckBounding`) is now in
    `GroundColl` as `joints: Vec<CollJointRt>` (index links, -1 = NULL; `gc_layout!` with the
    CollJoint offsets of the fields it keeps), `map_joints`, `joint_list_start`,
    `did_check_bounding`. The PC `ScriptGame_StageJointActive` checks are the `joint_active` hook.
    The raw-pointer `types::CollJoint` stays for the jobj/callback fields (not used yet).
  - The C's `goto block_8` into the line loop (the Dynamic range) runs the body once before testing
    the count; `GroundColl::joint_lines` keeps that.
  - Out-params become `Option<SegHit>` / `Option<i32>` / `(i32, Option<Vec3>)`; `None` is exactly
    where the C writes nothing. mpCheckFloor's `cb(gobj, line)` is a closure (`LineFilter`).
    mpLib_80051BA8's `dir == 0` HSD_ASSERT is a panic.
  - FMA per fma_sites.py: mpCheckFloor/Ceiling 2 f32 (`SQ+SQ`), the 4 Remap variants 2 f32 (the
    direction dot product only; their dist2 is split over statements), 800511A4/800515A0 6 f32,
    8005199C 1 f32 (`dy/dx*(x-x0)+y0`); the rest none.

- (port round 3, night 2) `mp::joint`, from src/melee/mp/mplib.c (17): mpJointHide, mpJointUnhide,
  mpJointUpdateDynamics, mpLib_80055E24, mpJointUpdateBounding, mpLib_8005667C, mpJointFromLine,
  mpLib_80056C54, mpLib_80057424, mpLib_80057528, mpLib_800575B0, mpJointListAdd,
  mpJointListUnlink, mpLib_80057BC0, mpLib_80057FDC, mpLib_80058044, mpJointSetB10.
  - `GroundColl` gains `joint_list_end` (`jointListEnd`) and `CollJointRt` gains `x_e` (`xE`,
    +0xE). mpJointListUnlink takes a joint index instead of a `CollJoint*`; mpJointFromLine scans
    `joints.len()` (= `mpLib_804D64B4->joint_count`).
  - `mpScriptIslandRefresh` (mpIsland_8005B334, or the PC script-batch dirty mark) is a stub:
    every call is queued in `GroundColl::island_refresh` in order. TODO(group2: mp/mpisland.c and
    the PC script batch).
  - mpJointUpdateDynamics' HSD_ASSERT on a zero-length Dynamic line is a panic.
  - mpLib_80056C54 returns `Option<FloorWalk>` (`None` = the early `return false` that writes
    nothing). Its possibly-uninitialised `total_dist_f27` starts at 0, as the shipping build's
    `-ftrivial-auto-var-init=zero` makes it. Its `sqrtf_store(.., sqrt_tmp - 4)` writes out of
    bounds; in the shipping build's optimised IR (clang -O2 ppc32, SHIP flags from fma_sites.py,
    without -disable-llvm-passes) that store and the sqrt_tmp alloca are gone, so the port has
    no side effect there and the oracle patches the pointer in bounds (see the gen script).
  - FMA per fma_sites.py: only mpLib_80056C54 fuses (3 f32 `t*(b-a)+a`, plus sqrtf_store's three
    f64 Newton steps = `geom::sqrtf`). Not ported (need HSD_JObj / other groups' callbacks):
    mpLib_800552B0, mpLib_80055E9C, mpJointSet/Get/ClearCb1/2, mpLib_8005811C.

- (port round 4, night 2) `mp::stitch`, from src/melee/mp/mplib.c (5): mpLib_800581DC,
  mpLib_80058560, mpLib_80058614_Floor, mpLib_800587FC, mpLib_80058820; and `mp::island`, all of
  src/melee/mp/mpisland.c (12): mpIsland_8005A6F8, 8005A728, 8005AB54, 8005AC14, 8005AC8C,
  8005ACE8, 8005AE1C, 8005B004, 8005B334, and the PC-only StageSlotFree, ScriptRecycle,
  ScriptBatch.
  - `GroundColl` gains `stage_joint` (the PC `ScriptGame_StageJoint` hook, default false),
    `coll_box` (`mpLib_80458868[2]`), `map_ranges` (`mpLib_804D64B4->ranges`) and `islands`.
  - Islands (`mp_UnkStruct0`, `HSD_MemAlloc(0x2C)`) live in an arena, `Islands::segs`, in
    allocation order; every link is an index (-1 = NULL) and `HSD_Free` only marks `freed`, so
    an index equals the C's allocation ordinal. `x2A`/`ptr` are never written by mpisland.c.
    The C's pointer-to-pointer list params become values returned as `(list, free)` pairs.
  - `GroundColl::mp_island_drain_refresh` runs the queued `mpScriptIslandRefresh` calls through
    mpIsland_8005B334, as the console build does at each call site. The PC build's routing of a
    script joint's refresh to the batch dirty mark (scriptBatchDepth) is not ported:
    TODO(group2: PC script batch).
  - Stubs: `grDynamicAttr_801CA224` (TODO(group2: gr/grdynamicattr.c)) and the GObj_Create /
    HSD_GObj_SetupProc of mpLib_80058820 (TODO(group1: baselib/gobj.c)) are no-ops.
  - FMA: fma_sites.py finds no fused site in any of these 17 functions.

- (port round 5, night 2) `cm::camera`, from src/melee/cm/camera.c (42): the framing and
  follow math Camera_80029124, 8002928C, 800293E0, 8002958C, 80029AAC, 80029BC4, 80029C88,
  80029CF8, Camera_ApplyQuake, 8002E158; the quake bookkeeping Camera_SetQuakeOffset,
  UpdateQuakes, RequestQuake, StopQuake, SetQuakeScale; and the `game_camera` accessors
  8002F260, SetModeToStandard, SetModeToFixed, 800300F0, 8003010C, 80030130, 80030154, 80030178,
  80030730, Set/GetBackgroundColor, GetTransformPosition/Interest, 80030DE4, 80030DF8,
  80030E10, 80031060, 80031074, 8003108C, 800310A0, 800310E8, 80031144, 80031154, 8003118C,
  800311CC, 800311DC and the PC Camera_StageSlotClipGet.
  - First use of melee_types in this crate: `Camera`, `CmSubject`, `CameraBounds`,
    `CameraTransformState`, `CameraUnkGlobals` (GC offsets kept). melee_world now depends on
    melee_types and melee_runtime (MSL `tanf` from `melee_runtime::gekko`).
  - Globals live in `CameraWorld`: `game_camera`, `cm_803BCCA0` (`globals`, from the C
    initializer), the used part of `cm_803BCB64` (`desc`), and the `cm_804D6468` list as
    `subjects` in `->prev` walk order (a subject is its index).
  - Stage getters (`Stage_GetCam*`, `Stage_UnkSetVec3TCam_Offset`, `Ground_801C4368`) are plain
    inputs in `CamStage`: TODO(group2: gr/stage.c, gr/ground.c). The PC `cm_fly_*` wrappers are
    ported (`geno_fly_any`, `script_lift_bounds`, per-subject `geno_fly_subject`).
  - Stubs: `HSD_GObjFree` in UpdateQuakes (TODO(group1: baselib/gobj.c)) and `grLib_801C9CEC` in
    RequestQuake (TODO(group2: gr/grlib.c)) are counted in `CameraWorld::stubs`.
  - ApplyQuake takes the PC gate `gm_IsCurrently1PMode_inline() && !Mex_Enabled(..)` as a bool
    (TODO(group6)); the PC `diag_cam_translate` print is not ported. The C's HSD_ASSERTs in
    80029CF8 panic.
  - Uninitialised locals follow the shipping build's `-ftrivial-auto-var-init=zero`: 8002E158's
    `var_f4` (case 3), RequestQuake's `quake_length` (unknown kind), GetBackgroundColor's alpha.
  - FMA per fma_sites.py: 8002958C 5, 80029AAC 2 + get_follow_speed 1, 80029C88 3, 80029CF8 7 +
    get_y_bias 1, ApplyQuake 2, 8002E158 1 (all f32 -> `fmadds`); the rest none.

## Tests

`tests/mplib_lines_vs_c.rs` checks the port against the real C. `cref/gen_mplib_lines_ref.sh`
cuts the function bodies verbatim out of src/melee/mp/mplib.c (between marker lines) and
compiles them with clang for **i686**. 32-bit is needed because the C assumes 4-byte pointers:
`(int) groundCollLine + offset`, and `flags_base[id * 2]` over `CollLine`. It also builds with
SSE and `-ffp-contract=off`. The harness runs the functions on 24 random line graphs with random
link, flag and active tables, and writes `tests/data/mplib_lines_ref.txt`, about 27k rows with
no disc data. Every integer result and every float bit matches, including 8 cases where the C
walk never ends: a budget on the active-hook catches those in both languages.

Caveat: NaN results are compared only as "is NaN". On a zero-length line, PSVECNormalize gives
NaN. An x86 build makes a NaN with a different sign than a Gekko does, so the native clang
reference cannot pin the NaN's bits.

`tests/mplib_geom_vs_c.rs` does the same for `mp::geom`: `cref/gen_mplib_geom_ref.sh` cuts
those bodies out of mplib.c (plus the line-query bodies they call), builds them i686 with
contraction on and `melee_runtime/oracle/lower.py` (22 f32 + 21 f64 sites lowered), and writes
`tests/data/mplib_geom_ref.txt` (50454 rows: 32 random graphs x 4 query points per line for the
four projections and mpGetSpeed, mpLib_8004ED5C per line, replayed setters, then 6000 random
float tuples for mpRemap2d / the three intersections / sqrtf). All match bit for bit, including
8 endless walks. Mutation check: unfusing mpLib_8004ED5C's fmadds fails the test. Unfusing any
f64 site (remap, intersections, sqrtf) does NOT: their multiplies are mostly f32 x f32 (exact in
f64, so fma equals unfused), and the rest round to f32 afterwards. Those f64 fusions are right
by the compiled IR, not proven by a vector.

`tests/mplib_check_vs_c.rs` does the same for `mp::check`: `cref/gen_mplib_check_ref.sh` cuts the
mpCheck*/mpLib_8005xxxx and bounding bodies out of mplib.c (plus the lines/geom bodies), builds
them i686 with contraction on + lower.py (49 f32 sites), and writes `tests/data/mplib_check_ref.txt`
(58522 rows: 48 random graphs, each with 1-6 random joints (ranges, flags, boxes, list order, active
table), 72 segment queries each through all 15 query functions, half of them aimed through a line's
midpoint, a quarter run with the bounding box pre-checked). Every result, every float bit, and
`didCheckBounding` plus every joint's flags after each call match. Mutation check: breaking the
flat-floor normal or dropping the goto's once-through Dynamic body fails the test. Unfusing any of
the f32 sites does NOT: they only feed min-distance comparisons and a dot-product sign, and no
vector lands on the tie where fusing flips the result. Those fusions are right by the compiled IR
(fma_sites.py), not proven by a vector. The script takes `LOWER_PY=` (another agent was mid-edit
on lower.py; the data was generated with the committed HEAD copy).

`tests/mplib_joint_vs_c.rs` does the same for `mp::joint`: `cref/gen_mplib_joint_ref.sh` cuts the
joint bodies out of mplib.c (plus lines/geom), builds them i686 with contraction on + lower.py
(25 f32 sites lowered), and writes `tests/data/mplib_joint_ref.txt` (36065 rows: 64 random graphs
with 1-7 joints carrying vertex ranges; 80 random joint/line edits each, after every one the full
state is compared: list start/end, each joint's next/flags/xE/bounds, every line's flags, every
MapLine lo_flags, every vertex x10/x14 and the ordered island-refresh calls; mpJointFromLine for
every line; 160 mpLib_80056C54 walks per graph, 463 returning true). All match bit for bit.
Mutation check: unfusing the x lerp in mpLib_80056C54 fails the test, so that fusion is proven by
a vector.

`tests/mpisland_vs_c.rs` does the same for `mp::island` + `mp::stitch`: `cref/gen_mpisland_ref.sh`
cuts the lines/geom/joint/check bodies plus the stitch range of mplib.c and all of mpisland.c
(with `mpScriptIslandRefresh` = mpIsland_8005B334, HSD_MemAlloc a numbered pool), builds them
i686 with contraction on + lower.py, and writes `tests/data/mpisland_ref.txt` (26629 rows: 128
random graphs, a third of them with ring chains, 70 random ops each over island builds/refreshes,
script batches, slot frees, stitching, the floor-bounds pass, joint add/hide/bounding and flag
edits, and 4 query kinds). After every op the whole island state (every list, each segment's
allocation index and fields), every MapLine `*_id1`, every joint's flags/xE and both boxes must
match; 6376 ops and 1707 query results match, plus 10 endless walks caught by the budget in both.
Mutation check: flipping mpIsland_8005AE1C's enabled test, never reusing the free list in
8005B004, dropping 800581DC's one-way unlink, and dropping 80058614's script-joint xE clear or
its floor-flag test each fail the test. Thin spots: mpIsland_8005AC14 finds an island in only 4
queries and AB54 in 31; the free list is non-empty in only a few states.

Regenerate: `bash rust/melee_world/cref/gen_mplib_lines_ref.sh` (and `gen_mplib_geom_ref.sh`, `gen_mplib_check_ref.sh`, `gen_mplib_joint_ref.sh`, `gen_mpisland_ref.sh`), then
`cd rust && cargo test -p melee_world`.

`tests/camera_vs_c.rs` does the same for `cm::camera`: `cref/gen_camera_ref.sh` cuts the
struct definitions out of cm/types.h and the bodies (plus the PC `cm_fly_*` block) out of
camera.c, builds them i686 with contraction on + lower.py (22 f32 sites lowered) and gekko_fp.c's
tanf, and writes `tests/data/camera_ref.txt` (43280 rows: 3000 random framing cases with 0-6
subjects of random state/timers/extents, random stage camera values, fly/script-lift flags and
1P gate, each running 80029124/80031154/8003118C per subject, then 800293E0, 8002958C,
80029AAC, 80029BC4, 80029C88, 80029CF8 and ApplyQuake in sequence; 2000 8002E158 cases over all
four modes; 300 random RequestQuake/StopQuake/UpdateQuakes sequences, full quake state after
each op). All match bit for bit. Mutation check: unfusing any one of seven sites (8002958C,
80029AAC, 80029CF8 base and target_interest.x, 80029C88, ApplyQuake, 8002E158) fails the test.
Not covered: no case reaches 80029CF8's asserts; 8003118C only with in-range distances; the 25
accessors are checked by a unit test (`cm::camera::tests::accessors`), not against the C.

Regenerate: `bash rust/melee_world/cref/gen_camera_ref.sh`.

## Round 6: gr/stage.c (45 functions)

`src/gr/stage.rs` ports all of src/melee/gr/stage.c plus `Stage_StageSlotSelect`
(stage_stage_slot_retail.inc) and the read side of `ScriptGame_ArenaBound`
(pc/gameworld/script_arena.inc, `ScriptArenaView::bound`), TARGET_PC bodies throughout: the 24
camera / blast-zone / pause-cam / angle getters, CalcUnkCamY(Bounds), UnkSetVec3TCam_Offset,
SetVecToFixedCamPos, the fixed-camera floor projection 80224CAC (lbVector_ApplyEulerRotation from
melee_runtime), 80224DC8, the spawn helpers 80224E38/80224E64/80224FDC, 80225074, the StKind ->
GrKind map (286 vanilla rows generated from the C initializer, 384 with the m-ex tail) with
Stage_MexInitIds/8022519C/802251B4, and the selected-stage forwarders 802251E8/8022524C/80225298/
802252E4/8022532C/80225194. Globals live in `StageWorld`; Ground_*, HSD_Randi, gm_*, lbAudioAx
and the Mex_* queries are `StageHooks` (TODO(group2: gr/ground.c), TODO(group1), TODO(group6),
TODO(pc: mex)). FMA: fma_sites.py reports only Stage_80224CAC's two lines; both fused.

`tests/stage_vs_c.rs`: `cref/gen_stage_ref.sh` cuts the getter range, the map + MexInitIds +
8022519C, the slot-select .inc, ScriptGame_ArenaBound and lbvector's sin/cos/Rotate/
ApplyEulerRotation, builds them i686 with contraction on + lower.py (12 f32 sites), and writes
`tests/data/stage_ref.txt` (20616 rows: 20000 random stage_info + script-arena states, a quarter
with an arena owner, every getter + the three Vec3 outputs; 80224DC8 over -20..119; the full
384-entry map after the m-ex fill with a fixed synthetic m-ex config; StageSlotSelect for -2..89).
All match bit for bit (34 functions). Mutation check: unfusing 80224CAC's x line fails the test.
Unit-tested only (`gr::stage::tests::hook_driven`, call order through a recording hook):
80225074, 80224FDC, 80224E38 and the seven forwarders. Not tested at all: 80224E64 (its kind-4
path runs mp_check_multiple, itself verified; the composition is not).

Regenerate: `bash rust/melee_world/cref/gen_stage_ref.sh`.

## Build notes

- `cargo build -p melee_world` and `cargo test -p melee_world` pass. I ran them with
  `CARGO_TARGET_DIR=target/g2` to stay out of other agents' way.
- I removed `melee_types` and `melee_runtime` from melee_world's dependencies. Nothing used them,
  and melee_types did not compile (46 errors in its generated gen.rs, which group 0 was still
  writing). Add them back when group 0/1 types are used. That change touches the shared
  `rust/Cargo.lock`, and I left the lockfile uncommitted.
- RUST_PORT.md is not updated. The task limited edits to this crate and this doc, so whoever
  merges the groups should copy this summary into it.

## Next (in order)

1. The rest of mplib: mpLibLoad/mpPruneEmptyLines, the PC script batch around
   mpScriptIslandRefresh, then the HSD_JObj-driven mpLib_80055E9C once
   group 1's JObj is usable.
2. mpcoll.c, the collision driver. It calls into fighter and item callbacks, which get stubbed.
3. The rest of cm/camera.c (subject list, mode callbacks, pause/debug cameras, CObj setup),
   once HSD_CObj exists; gr/stage.c getters are now ported (wire CamStage to StageWorld). `long double` literals become f64.
4. gr/*: 77 stage TUs, mostly callback tables plus per-stage logic. Port the shared grlib parts
   first.
5. if/* (HUD) and ef/* (effects). These are mostly render-side; whatever touches the RNG has to
   keep the draw order, or the golden seed column breaks.
