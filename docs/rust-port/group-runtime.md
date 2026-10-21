# Group 1 (runtime, math, HSD): port round 4 and later

Rounds 1-3 and the group's float rule, oracle method and blockers are in
`docs/rust-port/group-1 runtime, math, HSD (core + render).md`. This file carries the rounds
after that (the task for round 4 named this path).

## Port round 4 (night 3): AObj, the ID table, SList

Lowest level first: `list.c` and `id.c` have no callees outside alloc; `aobj.c` sits on the
round 2 FObj port.

| C TU | ported | module | checked against C |
|---|---|---|---|
| baselib/aobj.c | 18/27: GetFlags, SetFlags, ClearFlags, SetFObj, InitEndCallBack, InvokeCallBacks, ReqAnim, StopAnim, InterpretAnim, LoadDesc, Remove, Alloc, Free (drop), SetRate, SetRewindFrame, SetEndFrame, SetCurrentFrame, _HSD_AObjForgetMemory; plus MSL's inline `fmodf` | `aobj.rs` | ReqAnim, SetFlags, ClearFlags, SetRate, SetRewindFrame, SetEndFrame, SetCurrentFrame, StopAnim, InterpretAnim (with FObj callbacks), SetFObj, LoadDesc, GetFlags, InitEndCallBack |
| baselib/id.c | 5/7: IDSetup, InsertToTable, RemoveByIDFromTable, GetDataFromTable, _HSD_IDForgetMemory | `id.rs` | all 5 |
| baselib/list.c | 6/9: SListAlloc, AllocAndAppend, AllocAndPrepend, AppendList, PrependList, Remove | `list.rs` | AllocAndAppend, AllocAndPrepend, Remove (AppendList/PrependList through them) |

29 functions (+ `fmodf`).

- Native shapes: `HsdAObj<O>` owns its tracks as `Vec<HsdFObj>` and its target as `Option<O>`
  (dropping `O` stands for `HSD_JObjUnref`); `AObjGlobals` holds the two file counters and
  `endcallback_list`. `IdTable<T>` (101 buckets, last element of a bucket = chain head, so
  lookup order is the C's); the C's `default_table` is a table the caller owns. `SList<T>` is a
  `Vec` in list order; note `AppendList` puts the new node **second**, not last.
  `HsdAObj`, `HsdAObjDesc`, `IdTable`, `SList` delegate `GcLayout` to melee_types.
- FMA: aobj.c, id.c and list.c have no fused site of their own (`fma_sites.py`). The one fused
  site on this path is MSL's inline `fmodf` (`a - b*q` -> `fmuladd(-b, q, a)`), called by
  InterpretAnim's loop rewind; ported with `fmadds`. The oracle discriminates it: unfused, 4
  vectors fail (3 InterpretAnim, 1 SetFObj state after a rewind).
- Oracle: `oracle/build_aobj.sh` compiles aobj.c, fobj.c, id.c, list.c, spline.c, gekko_fp.c
  with the shipping flags + `lower.py`, links `driver_aobj.c` + `stubs_aobj.c`, and writes
  `tests/data/oracle_aobj.txt` (34,720 rows, 2.7 MB): 120 AObjs from generated FObj streams x 80
  random operations with full AObj + track state after each; 20,000 ID-table operations over two
  tables with colliding keys; 5,000 SList operations. `tests/oracle_aobj.rs`: 32,220 checks, 0
  failed. Mutation: dropping the REWINDED clear fails 1,497 checks.
- Not verified against C: the two end-callback counters (C file statics; their only effect,
  `InvokeCallBacks` walking `endcallback_list`, is a no-op because nothing in the game adds to
  that list); `LoadDesc` with `obj_id != 0` (needs JObj: TODO(group1 jobj), callers pass the
  lookup as a closure); `fmodf` with NaN or `|a/b| >= 2^63` (the C's `fptosi i64` is not
  rewritten by gwtool; Rust saturates, x86 gives i64::MIN; aobj.c never gets there with finite
  frames). The C keeps `fobj->ad` NULL before the first ReqAnim and the Rust an offset of 0; the
  test skips that one slot while the C prints NULL.
- Not ported: `HSD_ForeachAnim` and its ten per-type walkers (need JObj/DObj/MObj/PObj/TObj/
  LObj/CObj/WObj/RObj/Fog), the alloc-data getters/initialisers (native allocation).

## Port round 5: JObj tree, flags and matrix build

`jobj.c` sits on the round 1 matrix ports (`hsd_mtx::srt`/`srt_quat`, `mtx::concat`/`mult_vec`)
and the round 4 AObj, so it went next. 27 jobj.c definitions in `jobj.rs`, plus the header inlines
`HSD_JObjMtxIsDirty`, `HSD_JObjSetupMatrix`, `HSD_JObjSetMtxDirty`, `GetChild/GetNext/GetParent`:

| function(s) | checked against C |
|---|---|
| HSD_JObjCheckDepend, JObjResetRST, HSD_JObjResetRST, HSD_JObjWalkTree(+WalkTree0) | yes |
| HSD_JObjMakeMatrix (incl. the TARGET_PC m-ex scale-compensation block, bit 26, and the AObj-target translate), HSD_JObjSetupMatrixSub, HSD_JObjSetMtxDirtySub | yes (IK branches: no, see below) |
| HSD_JObjRemove, HSD_JObjRemoveAll, RecalcParentTrspBits, UpdateParentTrspBits, HSD_JObjAddChild, HSD_JObjReparent, HSD_JObjAddNext, HSD_JObjGetPrev | yes |
| HSD_JObjGetDObj, HSD_JObjAddDObj, HSD_JObjPrependRObj (+robj_set_next), HSD_JObjDeleteRObj | yes |
| HSD_JObjGetFlags, SetFlags, SetFlagsAll, ClearFlags, ClearFlagsAll | yes |
| HSD_JObjAlloc + JObjInit (`HsdJObj::new`) | the driver builds its JObjs with the same init (the real JObjInit runs through hsdNew/class.c, stubbed) |

- Native shape: `JObjTree<D, R>` arena, `JObjId` links; DObj / RObj chains are `Vec<D>` / `Vec<R>`
  (front = head); `aobj: Option<HsdAObj<JObjId>>`; `scl: Option<Vec3>`. `HsdJObj` delegates
  `GcLayout` to `melee_types::HSD_JObj`. `HSD_Joint` input is any `JointSource`.
- Stubs via `JObjHost` (TODO(group1 robj / class)): `HSD_RObjGetByType(.., IKHINT, 0)` bone
  length, `HSD_RObjUpdateAll`, `resolveIKJoint1/2` (need `HSD_RObjGetGlobalPosition`), and
  `HSD_JObjUnref`.
- FMA: `fma_sites.py` finds no fused site in any ported function (resolveIKJoint1's one site is
  not ported); the effector branch uses math_ppc `sqrtf` (double fma in `gekko::sqrtf`).
- Oracle: `oracle/build_jobj.sh` compiles jobj.c, mtx.c, mtx_pc.c, gekko_fp.c, lbtrigf.c and
  `helper_jobj.c` (accessors compiled with the game flags so the driver sees the real layouts, plus
  the RObj stubs) with the shipping flags + `lower.py`, and `driver_jobj.c` writes
  `tests/data/oracle_jobj.txt` (22k rows, 2.8 MB): 60 forests of 20 JObjs + 4 anchor JObjs
  (AObj targets), 16 RObjs (half IK hints), 16 DObjs, a 48-joint HSD_Joint forest, 400 random
  operations each (all functions above, random SRT, random flag masks incl. INSTANCE / USER_DEF /
  INDEP / quaternion / bit 26 / effector). Each row has the op, its result and an FNV-1a hash of
  every JObj's links, flags, SRT, mtx, scl, RObj and DObj chains (+ WalkTree and RObjUpdateAll
  call logs). `tests/oracle_jobj.rs` replays the rows: 22,020 rows, 0 failed (260 RObjUpdateAll
  calls, 28 effector IK-hint hits).
- Mutation check: dropping the bit-26 compensation concat, the `<< 10` in either trsp-bit
  function, the effector's `x_scale`, the flag-8 scl copy, or the INDEP_PARENT test in
  SetMtxDirtySub each fail thousands of rows. (The first data set did not discriminate the `<< 10`
  in RecalcParentTrspBits; flags 18-20 were added to the masks.)
- The generator keeps to what the C keeps valid: never JOINT1/JOINT2 (the IK solvers are not
  ported), no cycles, AObj targets that are standalone, RObj/DObj prepends only of objects in no
  chain, `Remove` only with a single child, `AddChild` honouring its asserts.
- Not ported: resolveIKJoint1/2, jobj_get_effector_checked, the anim functions (ReqAnim*,
  RemoveAnim*, AddAnim*, JObjSortAnim, JObjUpdateFunc, Anim/AnimAll), load / ResolveRefs, Unref /
  UnrefThis / ReleaseChild / Release / Amnesia / InfoInit (class system), DispAll, SetCurrent /
  GetCurrent, the callback setters.
