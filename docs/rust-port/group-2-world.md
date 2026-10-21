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
| src/melee/cm | 182 | 66 (night 3 count; see the night-3 section) |
| src/melee/ef | 67 | 3 (night 3 r3: efLib_SetFlags/PauseAll/ResumeAll) |
| src/melee/gr | 2167 | see the round sections (stage.c, ground.c, grlib.c, night-3 r2 leaves, night-3 r3 568 constant/empty callbacks) |
| src/melee/if | 373 | 8 (night 3 r2) |
| total | 3015 | 912 after night 3 round 3 (30.2%; 568 of them are constant-return/empty callbacks); 796 verified vs C |

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

## Round 7: gr/ground.c accessors and helpers (85 functions)

`src/gr/ground.rs` ports the self-contained part of src/melee/gr/ground.c as methods on
`StageWorld` (stage_info) plus free functions: the GroundParam y/x4 getters and SetParamY, the 18
GroundParam colour setters/getters (801C052C..801C06A4), the stage_info flag/field accessors
(801C1D6C..801C1E84, 801C5740..801C57F0, 801C5A84..801C5ABC, 801C4D70/4DA0), the 16 camera and
blast-zone setters (801C3880..801C39B0), 801C3D44/3DB4, 801C4338/4368/438C, the per-Ground SFX
slot helpers (801C5440/54DC/5544/55AC/5630/5694, 801C53EC/5414), the one-line forwarders
(801C43A4, 801C4DD0/4E20, EnableMatchCamera, 801C5840, GetP1Fighter(2), 801C5A28/5A60), the
orientation math of 801C4B50 and 801C5AEC. `stage_info.param` stays the raw pointer the C has.
Callees in other groups are `GroundHooks` methods whose default bodies panic with
TODO(group1: lbaudio_ax, random, spline, lbshadow), TODO(group2: grzebes/grkongo/groldkongo,
camera modes), TODO(group3: Player_GetEntity*), TODO(group5: Toy_*). FMA: fma_sites.py reports
no fused site in any of these functions.

`tests/ground_vs_c.rs`: `cref/gen_ground_ref.sh` cuts 801C4B50/5630/5694/5AEC verbatim into
`cref/ground_tu.c` (spline/lbShadow/audio stand-ins), builds it with the game flags together
with lbvector.c, lbtrigf.c, gekko_fp.c, mtx_pc.c, mtx44.c (contraction on + lower.py, the
melee_runtime oracle recipe) and writes `tests/data/ground_ref.txt` (60000 rows: 20000 tangents
for 4B50 incl. near-vertical and zero-y, 20000 ONB triples for 5AEC incl. b.z = +-1 and NaN
asin, 20000 values for the 5630 volume / 5694 pan arguments). All match bit for bit. Mutation
check: the f32-PI variant of 4B50's `M_PI - x` and dropping 5AEC's y clamp both fail.
Unit-tested only (`gr::ground::tests::{accessors, sfx_slots}`): the accessors, setters and
forwarders (call order through a recording hook). Not ported: the rest of ground.c (loading,
JObj/GObj, fog/lights, map-coll, Ground_801C5940, 801C5AD0 which needs stage_datas).

Regenerate: `bash rust/melee_world/cref/gen_ground_ref.sh`.

## Round 8: gr/grlib.c (all 24 functions)

`src/gr/grlib.rs` ports every function in src/melee/gr/grlib.c: the generator helpers
(801C96F8 incl. the AppSRT scale by Ground_801C0498, 801C97DC/9808/9834/9854/9874, 801C98A0 and
the recursive 801C9908 that kills every generator on a joint subtree), 801C99C0, the four-point
matrix 801C9A10/9A70 (`GrLib::mtx` is the static `grLib_8049EF58`), the DynamicsDesc forwarders
801C9B20/9B6C/9B8C, the quake GObj 801C9CEC plus its two static procs 801C9BC8/9C40, x708
get/set 801C9E40/9E50, 801C9E60, PointInsideColl and 801C9EE8. Callees go through `GrLibHooks`
(defaults panic: TODO(group1: particle/psappsrt/jobj/gobj/lb_00F9), TODO(group2: ground.c
Ground_801C2D24, grrcruise/grbigblue/gricemt, camera wiring), TODO(group3/5: fighter and item
plinks, given as CollData lists)).

FMA: fma_sites.py reports 4 fused sites, all in PointInsideColl (`0.5f*h + offset`,
`0.5f*(t+b) + pos`, x and y); the port uses mul_add there. `tests/grlib_vs_c.rs`:
`cref/gen_grlib_ref.sh` cuts PointInsideColl verbatim into a TU over a stand-in CollData,
compiled with the game flags, contraction on + lower.py (4 fmuladd lowered), and writes
`tests/data/grlib_ref.txt` (40000 rows, a third placed exactly on an edge +-2 ulp, one NaN).
All match. No mutation check is possible: every fused multiply is by 0.5f, which is exact, so
fused and unfused agree on all normal inputs (an unfused variant passed every row).
Unit-tested only (`gr::grlib::tests`): 801C98A0/9908 (kill order, instance stop, null),
801C9A10/9A70, 801C9E40/9E50/9E60, 801C9CEC's early returns, 801C9EE8 (fighter/item lists,
It_PKind_Random skipped). Not tested: 801C96F8, 801C9CEC's create path, 801C9BC8/9C40, and the
one-line forwarders.

Regenerate: `bash rust/melee_world/cref/gen_grlib_ref.sh`.

## Night 3, round 1: camera accessors vs C, 24 more camera.c functions

Recount at the start (from the night-2 rows): 301 of 3013 ported, 160 of them checked against
the C; 141 unit-tested or untested (25 camera accessors, 10 stage hook-driven + Stage_80224E64,
81 ground.c, 23 grlib.c; that list sums to 140, one is unaccounted for in the night-2 notes).

New, all in `src/cm/camera.rs` (24): Camera_80030A60, 80030A78, 80030A8C, SetStageVisible,
80030AC4, 80030AE0, 80030AF8, 80030B0C, 80030B24, 80030B38, 80030B50, 80030B64, 80030B7C,
80030B90, 80030BA8 (the x399/x39A bit accessors), Camera_8002FE38 (TARGET_PC body),
Camera_8002EEC8, Camera_8002BC78 and its static inline OrthonormalizeBasis (PSVECCrossProduct /
lbVector_Normalize from melee_runtime), and the PC script helpers cm_script_float, cm_script_bits,
cm_script_lerp, cm_script_blend, cm_script_ease (`CmScriptPose` is PC-only, no GC layout).
FMA: the shipping-flag IR of the cut bodies has 2 fused f32 sites, cm_script_lerp
(`(b-a)*t + a`) and cm_script_ease case 3 (`1 - 2(1-t)*(1-t)`); the port uses fmadds at both.

`tests/camera_acc_vs_c.rs`: `cref/gen_camera_acc_ref.sh` cuts these 24 bodies plus the 25
accessors that night 2 had only unit-tested (8002F260, SetModeToStandard/Fixed, 800300F0,
8003010C/0130/0154/0178, 80030730, Set/GetBackgroundColor, GetTransformPosition/Interest,
80030DE4/DF8/E10, 80031060/1074/108C/10A0, 800310E8, 80031144, 800311CC/11DC,
StageSlotClipGet) verbatim out of camera.c into `cref/camera_acc_tu.c` (a `game_camera`
stand-in with the cm/types.h member names, `cref/camera_acc.h`), compiled with the game flags,
contraction on + lower.py, linked with mtx_pc.c, mtx44.c, lbvector.c, lbtrigf.c, gekko_fp.c.
8640 cases (random full state incl. NaN/denormal floats and out-of-range bitfield arguments;
1500 each for 8002BC78 near the +-0.999 clamps, lerp, ease, blend). All match bit for bit and
state for state, with one exception: Camera_8002BC78 with a NaN `pause_up` gives a NaN of the
other sign (1 row; the sign of a propagated NaN through melee_runtime's vec_cross / fmadds is not
pinned down by LLVM). The test treats NaN-vs-NaN as equal for that op only. Mutation check:
unfusing cm_script_lerp fails 1592 rows, unfusing ease case 3 fails 8.

Stubs replaced: GroundHooks::camera_set_mode_to_fixed / _standard and
GrLibHooks::camera_set_quake_offset are implemented for `CameraWorld` (the ported camera.c
bodies); the trait defaults now say to forward to it. Still stubs: the rest (group 1 HSD, group 3
Player_GetEntity, group 5 Toy, other gr TUs); `StageHooks::hsd_randi` is a required method, and
melee_runtime::random exists, but the RNG state owner is not in this crate.

Not ported (aliasing): Camera_8002EB5C / 8002EC7C / 8002ED9C write `x35C.bits` / `x35C.vec`,
a union whose byte overlap differs between the GC layout and a native one; left for when the
camera struct's union handling is decided.

Counts after this round: 325 of 3013 ported (10.8%); 209 verified against the C; 116 not
(81 ground.c accessors, 23 grlib.c, 10 stage.c hook-driven fns, Stage_80224E64, +1 as above).

Regenerate: `bash rust/melee_world/cref/gen_camera_acc_ref.sh`.

## Night 3, round 2: recount, 16 gr/if leaves vs C, 3 stubs replaced

Recount at the start: the function-definition count (a `name(...)` line followed by `{`) is now
3014 (gr/ has 2167, one more than night 1's 2166); ported 325, verified against the C 209, as the
round-1 notes say. 116 ported but not checked against the C (listed in round 1).

New, all verified bit for bit (`tests/leaves_r2_vs_c.rs`, 70269 rows from
`cref/gen_leaves_r2_ref.sh`: the bodies cut verbatim out of the game C, `static`/`inline`
dropped, game flags, contraction on + lower.py, stand-in Ground / HSD_GObj with the members the
bodies touch):
- `gr::leaves` (8): grGreens_80213AB4, grOldPupupu_8021128C, grStadium_ScaleColor,
  grMc_DistanceSquared, grZebes_Subtract, grInishie1 fabsf_inline, grKongo_801D8270,
  grOldKongo_802105AC.
- `hud::leaves` (8, the first if/ functions; module `hud` because `if` is a keyword):
  DevText_StrLen, DevText_NumToStr, DevText_Clamp, ifStock_802F89F8_count / _divisor / _digit,
  ifnametag inlineA1, ifstatus foo (its s8 toggle `ifStatus_804D6D61` is passed in).
FMA: the IR has 2 fused f32 sites, both in grMc_DistanceSquared; the port uses `fmadds`
(gwtool's double-rounded lowering). Mutation check: an unfused version fails rows
(`distance_squared_fusion_is_observable`). Inputs include NaN, +-inf, +-0, denormals, i32::MIN
(NumToStr's wrapped `-num`), and float->int out of range (inlineA1, saturating as gwtool makes
it). Not covered: ifStock divisor/digit with position >= 32 (10^32 wraps to 0, a divide by zero
the oracle cannot run natively; the port uses cabi::sdiv, the C build's x/0 semantics).

Stubs replaced (targets now exist):
- `GroundHooks::grkongo_801d8270` / `groldkongo_802105ac` default to the ported bodies.
- `GrLibHooks::jobj_translation_xy` reads `j->translate` through melee_types::inline
  (HSD_JObjGetTranslationX/Y), with the inlines' null assert.
Not replaced, and why: splGetSplinePoint exists in melee_runtime::spline, but the hook returns a
Vec3 while the C leaves `*p` untouched for u outside [0, 1]; changing that needs the hook to take
the out-param, left for the Ground_801C4B50 caller rework. lbShadow_8000E9F0 is in melee_modes,
which depends on melee_world (cycle). HSD_Randi / GObj / JObj / particle hooks: melee_runtime's
versions are arena/id based, the hooks take raw pointers; they wait for one object model.

Counts after this round: 341 of 3014 ported (11.3%); 225 verified against the C; 116 not.

Regenerate: `bash rust/melee_world/cref/gen_leaves_r2_ref.sh`.

## Night 3, round 3: recount, 568 constant/empty gr callbacks + 3 efLib walkers vs C

Recount at the start (same rule: a `name(...)` line followed by `{`): mp 226, cm 182, ef 67,
gr 2167, if 373 (one more than round 2's 372) = **3015**. Ported 341, verified 225, as round 2
left them (the ported tally is carried from the round notes, not re-derived from the Rust).

Stubs whose target now exists elsewhere: none replaceable this round. Checked: the GroundHooks /
GrLibHooks / StageHooks TODOs point at group 1 HSD (GObj/JObj/particle/lb), group 3 Player,
group 5 Toy, group 6 gm, or at gr TUs still unported (grZebes_801DA3F4, grKongo_801D828C,
grOldKongo_802105C8 need grMaterial_801C95C4 -> lb_80014498, group 1). StageHooks'
Ground_801C5A84/5AA4 etc. are ported on `GroundWorld`, but StageHooks is a required trait
with no default, so there is no stub to replace; whoever owns both worlds implements it.
CameraStubLog's grLib_801C9CEC record stays: grlib's version needs a `GrLibHooks` (GObj_Create
etc., group 1) that CameraWorld does not hold.

New, all verified against the C:
- `gr::trivial` (568, generated by `cref/gen_trivial_r3.py`): every function in the per-stage
  gr TUs (all gr/*.c but stage.c, ground.c, grlib.c) whose body is exactly `return false;`,
  `return true;`, `return 0;`, `return 1;`, `return NULL;`, `return;` or empty: 436 bool
  predicates (366 `return false`-style), 72 void, 60 DynamicsDesc* (NULL). One module per TU
  (5 TUs share static `stageGObjN_Callback1` names). `extern "C"` with the C ABI types
  (`bool` -> i32), so they drop into `StageCallbacks` / `HSD_GObjPredicate` as they are.
  Oracle: the bodies verbatim, renamed `<tu>__<name>`, game flags + lower.py, each called with
  zero and with junk arguments (1136 rows, `tests/trivial_r3_vs_c.rs`, which also checks all
  568 are present). This verification is shallow by nature: the bodies are constants.
- `ef::eflib` (3): efLib_SetFlags, efLib_PauseAll, efLib_ResumeAll, the first ef/ functions.
  The two plink heads (`HSD_GObjPLinkHead[11]`, `[12]`) are passed in as a slice. Oracle: 3000
  random list pairs (0-8 nodes each, NULL user_data, 3 candidate parents, random s8/u8 flags,
  s32 expire args incl. values that truncate), `tests/eflib_r3_vs_c.rs`. No float math, no
  fused sites (the TU's IR has 0 f32 fmuladd after lowering).

Counts after this round: 912 of 3015 ported (30.2%); 796 verified against the C; 116 not
(unchanged from round 1: 81 ground.c accessors, 23 grlib.c, 10 stage.c hook-driven fns,
Stage_80224E64, +1). Excluding the 568 trivial callbacks: 344 of 2447 (14.1%).

Regenerate: `bash rust/melee_world/cref/gen_trivial_r3_ref.sh` (also regenerates
`src/gr/trivial.rs`).

## Night 3, round 4: recount, 220 single-call gr forwarders vs C

Recount at the start (same rule, `name(...)` line then `{`, per file): mp 226, cm 182, ef 67,
gr 2167, if 372 = **3014**. Round 3 wrote if 373 / 3015, but src/melee/if has not changed
since (last commit afa8c0e0d) and the rule gives 372 both ways, so 3014 is right. Ported 912,
verified 796, as round 3 left them (tally carried from the round notes).

Stubs whose target now exists elsewhere: none replaceable. Re-checked every
`unimplemented!("TODO(groupN): X")` in the crate against the workspace: the only hits are
melee_fighter's transpiled `c2r` externs (not ports), melee_runtime's arena/id-based GObj (the
hooks take raw pointers, as before) and melee_modes' lbShadow_8000E9F0 (dependency cycle).

New, all verified against the C:
- `gr::forward` (220, generated by `cref/gen_forward_r4.py`): every function in the per-stage
  gr TUs whose body is exactly one call: `grZakoGenerator_801CAE04(NULL)` (58),
  `Ground_UpdateMapColl(gobj)` (32), or the gr/inlines.h inlines `Ground_InitMapCollAndAnim`
  (55: Ground_InitMapColl(jobj, gp->map_id); grAnime_801C8138(gobj, map_id, 0)),
  `Ground_StartMapAnim` (50: grAnime_801C8138(gobj, gp->map_id, 0)) and
  `Ground_UpdateWindAndMapColl` (25: lb_800115F4(); Ground_UpdateMapColl(gobj)), expanded.
  The callees are not ported; they go through `gr::forward::CALLEES` (a table of fn pointers
  whose defaults panic with TODO(group2) / TODO(group1: lb_800115F4)). "Ported" here means the
  forwarder's own logic (which callee, in which order, with which arguments, read from which
  field); the callees themselves are still TODO, so these do nothing useful until they land.
  Oracle: the bodies cut verbatim (renamed `<tu>__<name>`), the three inlines cut verbatim from
  inlines.h, game flags + lower.py, stand-in HSD_GObj/Ground (map_id at 0x14); recorders print
  each callee call with symbolic pointers (gobj / its hsd_obj / NULL) and int args. Each
  function runs for map_id 0, 1, 31, 70, -1, INT_MAX: 1320 traces, `tests/forward_r4_vs_c.rs`
  (also checks all 220 are present). No float math (0 fmuladd). Mutation check: passing 1
  instead of 0 as grAnime's bool fails the test.
- Not taken this round: the 21 `Ground_InitTargetStage(create)` forwarders (they write
  `stage_info.unk8C` bitfields and need the global stage_info / StageWorld wiring) and the
  other ~200 one-statement bodies (mixed callees).

Counts after this round: 1132 of 3014 ported (37.6%); 1016 verified against the C; 116 not
(unchanged: 81 ground.c accessors, 23 grlib.c, 10 stage.c hook-driven fns, Stage_80224E64, +1).
Excluding the 568 trivial callbacks and the 220 forwarders: 344 of 2226 (15.5%).

Workspace: `cargo test --workspace --no-fail-fast` green on HEAD plus this round's files
(checked in a clean worktree). The shared working tree did not build at the time because of
another agent's uncommitted melee_runtime/src/generator.rs (E0506), not touched here. Another
agent also has uncommitted gr/grlast.rs and gr/bgm.rs in this crate; not part of this round.

Regenerate: `bash rust/melee_world/cref/gen_forward_r4_ref.sh` (also regenerates
`src/gr/forward.rs`).

## Night 3, round 5: recount, 17 ground.c / per-stage getters vs C, 3 grlib stubs replaced

Recount at the start (same rule, `name(...)` line then `{`): mp 226, cm 182, ef 67, gr 2167,
if 372 = **3014** (unchanged). Ported 1132, verified 1016 (tally carried from round 4).

Stubs whose target now exists: `GrLibHooks::grrcruise_80201918` / `grbigblue_801ef7d8` /
`gricemt_801fa728` (TODO(group2) panics) are gone; grLib_801C9E60 now calls the ported
StageWorld getters (melee_types `Vec` <-> `Vec3` copy). The grlib unit test now checks the
real grBigBlue result (zero vector with no map gobj 34). Nothing outside the crate became
replaceable: the other TODO(group1) names exist only as transpiled `raw/` modules, not ports.

New, all verified against the C (`tests/ground_r5_vs_c.rs`, `cref/gen_ground_r5_ref.sh`,
21400 rows; bodies cut verbatim, game flags + lower.py; 0 f32 fmuladd after lowering, so no
fused sites; mutation-checked: count==1 fallback, the 0x4C stkind, the 2AE8 product grouping,
the classifier compare, the fog-colour copy, the 2090 return, and a doubled callback all fail):
- `StageWorld`: Ground_MexParamStkind (TARGET_PC), Ground_801C2AD8, Ground_801C2AE8 (stage
  param scale; the C's OSReport + spin when no row matches is a panic here, rows predicted by
  the driver, not run), Ground_GetMapGObj, Ground_801C2BBC, Ground_801C2BD4 (x694 add, assert
  when full -> panic), Ground_801C2C8C, Ground_801C2CF4, Ground_801C2D0C, Ground_801C20D0,
  Ground_801C205C / Ground_801C2090 (stage fog colour set/get through `stage_info.x12C`),
  grRCruise_80201918, grBigBlue_801EF7D8, grIceMt_801FA728.
- free fns: Ground_801C498C (first GROUND-class gobj on the light plink list, head passed in),
  Ground_801C1D38 (the Ground's xC_callback).
The first twelve were written by an earlier, interrupted run of this round and left uncommitted
with a broken driver (`__builtin_setjmp` across the loop crashed after a few thousand rows, and
the file only held M rows); the driver now uses an assert flag and predicts the spin case.

Counts after this round: 1149 of 3014 ported (38.1%); 1033 verified against the C; 116 not
(unchanged: 81 ground.c accessors, 23 grlib.c, 10 stage.c hook-driven fns, Stage_80224E64, +1).

Workspace: `cargo test -p melee_world` green. `cargo test --workspace` currently fails to build
melee_chara (`E0583: file not found for module r11`, another agent's uncommitted
chars/mod.rs); not touched here. gr/grlast.rs, gr/bgm.rs and their two gr/mod.rs lines are
another agent's uncommitted work and are left out of this commit.

Regenerate: `bash rust/melee_world/cref/gen_ground_r5_ref.sh`.

## Night 3, round 6: recount, 20 DevText / HUD and stage leaves vs C

Recount at the start (rule: a line starting with an identifier, containing `(` and no `;`,
followed by a line starting with `{`; `awk` over src/melee/{mp,cm,ef,gr,if}/*.c): mp 226,
cm 182, ef 67, gr 2167, if 373 = **3015**. Round 5 reported if 372 / 3014; src/melee/if has
not changed since that commit, so the earlier tally was one short (not traced further).
Ported 1149, verified 1033 carried from round 5.

Stubs whose target now exists elsewhere: none found. The remaining TODO(group*) panics in
this crate (camera.rs, grlib.rs, ground.rs, stage.rs, mp/*) still name HSD JObj/CObj/GObj,
particle and fighter functions that exist only as transpiled `raw/` modules or not at all.
`ifStock_802F8298_step_anim` is already ported by melee_modes (hud_ptr.rs); not re-counted.

New, all verified against the C (`tests/textlib_r6_vs_c.rs`, `cref/gen_textlib_r6_ref.sh`,
84159 rows: 60000 random DevText op rows with full state + glyph buffer before/after, 4096
colours, 20000 vectors; bodies cut verbatim, game flags + lower.py; 0 f32 fmuladd (the 3 f64
sites are math_ppc.h's unused sqrtf inline), so no fused sites):
- `hud::textlib` (if/textlib.c): DevText_EraseFirstLine, DevText_SetCursorXY,
  DevText_SetCursorX, DevText_HideCursor, DevText_80302AC0, DevText_ShowBackground,
  DevText_HideBackground, DevText_ShowText, DevText_HideText, DevText_SetScale, DevText_SetXY,
  DevText_StoreColorIndex, DevText_SetTextColor, DevText_SetBGColor, DevText_Erase,
  DevText_AdvanceLine (static inline), DevText_Print, DevText_PrintInt; textlib_1.c `adjust`.
- `gr::leaves::grstadium_mul_vec` (grpstadium.c `mul_vec`).
Conventions: the glyph buffer is a slice (not `DevText.buf`); a cell is the native C's
`DevTextGlyph` (byte 1 bits 0-1 = colour). DevText's +0x27 bitfield stays in melee_types' GC
byte order; the test converts from the PC layout the C dumps. Mutation-checked: the
EraseFirstLine loop bound, the glyph colour mask, the SetXY minimum and the NOWRAP test all
fail; `c*50/100` -> `c/2` in `adjust` passes (equivalent for u8).
Not taken: DevText_Create / find_by_id / DevText_Printf (pool + draw list, vsnprintf).

Counts after this round: 1169 of 3015 ported (38.8%); 1053 verified against the C; 116 not
(unchanged: 81 ground.c accessors, 23 grlib.c, 10 stage.c hook-driven fns, Stage_80224E64, +1).

Workspace: `cargo test -p melee_world` green. `cargo test --workspace` fails to build the
`melee` bin (E0063 `appsrt_hook_lists`, E0609 `entry_timer`) from other agents' uncommitted
rust/melee/src changes; not touched here.

Regenerate: `bash rust/melee_world/cref/gen_textlib_r6_ref.sh`.

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
4. gr/*: the per-stage TUs, mostly callback tables plus per-stage logic (grlib is done).
5. if/* (HUD) and ef/* (effects). These are mostly render-side; whatever touches the RNG has to
   keep the draw order, or the golden seed column breaks.
