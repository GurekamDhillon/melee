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

## Port round 6: WObj and the DObj flags

`wobj.c` sits on round 5's JObj (`setup_matrix`), round 1's `mtx::mult_vec` and spline
`arc_length_point`, and round 4's AObj, so it went next; the flag half of `dobj.c` came with it.
`fma_sites.py`: no fused site in wobj.c or dobj.c.

| function(s) | module | checked against C |
|---|---|---|
| HSD_WObjSetPosition, SetPositionX/Y/Z, GetPosition, HSD_WObjClearFlags (inline), WObjUpdateFunc (path type 4 incl. clamp + spline, types 5-7) | `wobj.rs` | yes |
| HSD_WObjRemoveAnim, ReqAnim, AddAnim, InterpretAnim, WObjLoad / HSD_WObjInit, HSD_WObjLoadDesc, HSD_WObjAlloc | `wobj.rs` | no (thin wrappers over the checked parts, AObj and RObj stubs) |
| HSD_DObjSetCurrent, GetFlags, SetFlags, ClearFlags, ModifyFlags | `dobj.rs` | yes (SetCurrent: trivially, no observable output) |

18 functions (13 wobj.c incl. the static UpdateFunc and the header ClearFlags, 5 dobj.c).

- Native shape: `HsdWObj<R>` (`flags`, `pos`, `aobj: Option<HsdAObj<JObjId>>`, `robj: Vec<R>`),
  GcLayout delegated to `melee_types::HSD_WObj`. Position functions take the `JObjTree` and its
  host. InterpretAnim takes the AObj out while it runs so the update closure can write `flags`/
  `pos` (the C reads `wobj->aobj->hsd_obj` only, which is constant during the call).
  DObj: the native DObj record is not built yet, so the flag functions work on its `flags` word
  (`Option<&mut u32>`, None = NULL); `DObjGlobals` holds `current_dobj`.
- Stubs via `WObjHost` (TODO(group1 robj / jobj)): `jp->u.spline` (the native JObj keeps only the
  DObj member of its union), RObj RemoveAnimAll / ReqAnimAll / AddAnimAll / AnimAll /
  LoadDesc+ResolveRefsAll, and AObjLoadDesc's obj_id lookup. Path type 4 with no point from
  `splArcLengthPoint` gives (0,0,0) (the shipping build zero-initialises the local).
- Oracle: `oracle/build_wobj.sh` compiles wobj.c (included by `helper_wobj.c` so the static
  UpdateFunc is reachable), dobj.c, jobj.c, spline.c, mtx.c, mtx_pc.c, gekko_fp.c, lbtrigf.c with
  the shipping flags + `lower.py`; `driver_wobj.c` writes `tests/data/oracle_wobj.txt` (1.7 MB):
  150 scenarios (a 3-JObj chain with random SRT, a random spline of each type 0-3 on every JObj, a
  WObj with no AObj / a target-less AObj / an AObj targeting JObj 1 or 2, a DObj with random flags)
  x 100 random ops (positions, axis sets, GetPosition, ClearFlags, UpdateFunc types 0-8 with
  out-of-range and edge values, JObj SRT changes + SetMtxDirty, DObj flag ops incl. NULL). Each row
  checks the result, WObj flags/pos, DObj flags and an FNV-1a hash of the JObjs' flags and
  matrices. `tests/oracle_wobj.rs`: 15,000 rows, 0 failed, 1,197 path updates.
- Mutation: dropping the bit-0 set after a path update fails 9,784 rows. Dropping the upper clamp
  (`fv > 1 -> 1`) fails none: `splArcLengthGetParameter` already returns 1 for `s >= 1`, so the
  clamp is not observable; it is kept as in the C.
- Not ported: `HSD_WObjSetDefaultClass`, WObjRelease / Amnesia / InfoInit, `HSD_WObjUnref` (class
  system); the rest of dobj.c (anim functions need MObj/PObj, Load/ResolveRefs, Remove/Alloc,
  Disp).

## Port round 7: RObj (constraints, limits, expressions)

`robj.c` sits on round 5's JObj (`setup_matrix`, `make_matrix`), the round 1/2 matrix and PSVEC
ports and round 4's AObj, and it is what jobj.c's IK solvers and `HSD_RObjUpdateAll` stubs were
waiting for, so it went next. 38 robj.c definitions plus mtx.h's `HSD_MtxColMag` / `HSD_MtxColVec`
in `robj.rs`:

| function(s) | checked against C |
|---|---|
| HSD_RObjSetFlags, HSD_RObjGetByType (+ the static inlined copy), HSD_RObjGetConstraintType, HSD_RObjGetGlobalPosition | yes |
| HSD_RObjUpdateAll with set_dirup_matrix, resolveCnsDirUp, resolveCnsOrientation (both branches), resolveLimits, expEvaluate; HSD_MtxColMag, HSD_MtxColVec | yes |
| HSD_RObjLoadDesc, expLoadDesc, bcexpLoadDesc, loadRvalue, dummy_func, HSD_RObjResolveRefs(+All), HSD_RvalueResolveRefs(+All), HSD_RObjSetConstraintObj | yes |
| RObjUpdateFunc, HSD_RObjAnim(+All), RemoveAnimByFlags / AllByFlags / All, ReqAnimByFlags / AllByFlags / All, AddAnim(+All) | no (thin wrappers over the checked AObj port) |
| HSD_RObjRemove(+All), HSD_RObjAlloc, HSD_RvalueAlloc / Remove / RemoveAll, _HSD_RObjForgetMemory | no (native allocation; their only effect is the ref/unref host calls) |

- Native shape: an RObj chain is a `Vec<HsdRObj>` / slice in list order; the C union is `RObjU`
  (one field per member, read by the type bits as in the C; reading a member after `SetFlags`
  ORs in new type bits is not modelled, the game never does it). `HsdRObjDesc::u` is the enum
  `RObjDescU`; `HSD_RvalueList` arrays are slices without the NULL terminator; the update
  callback is `RObjUpdate` (tree, obj, type, `Option<RObjVal>`). `HsdRObj` / `HsdRObjDesc`
  delegate `GcLayout` to `melee_types::HSD_RObj` / `HSD_RObjDesc`. `RObjGlobals` holds
  expEvaluate's `arg_buf` (zeroed natively; the C's HSD_MemAlloc buffer is uninitialised).
- Stubs via `RObjHost`: `HSD_JObjRefThis` / `HSD_JObjUnrefThis` (TODO(group1 class)),
  `HSD_ByteCodeEval` (TODO(group1 bytecode)). ID lookups (`HSD_IDGetData`) and AObjLoadDesc's
  obj_id lookup are closures.
- FMA: `fma_sites.py` finds no fused site in robj.c's own functions; `HSD_MtxColMag` has two
  (`fmadds(m2, m2, fmadds(m0, m0, m1*m1))`), the rest are math_ppc `sqrtf` and the PSVEC/HSD_Mtx
  ports. The oracle discriminates ColMag's fusion: unfused, 927 checks fail.
- Oracle: `oracle/build_robj.sh` compiles robj.c, jobj.c, mtx.c, mtx_pc.c, gekko_fp.c, lbtrigf.c
  and `helper_robj.c` with the shipping flags + `lower.py`; `driver_robj.c` writes
  `tests/data/oracle_robj.txt` (23k rows, 2.4 MB): 400 scenarios, each a 3-12 JObj forest
  (random SRT, CLASSICAL_SCALE, scl) and a 1-10 RObj chain built by LoadDesc + ResolveRefsAll
  (+ random SetConstraintObj) from random descs of every type (JObj constraints of subtype 1-5,
  limits 0-13, expressions with fn0 / fn1 / NULL func, bytecode with and without code, IK
  hints, rvalue lists over all 26 argument bits incl. unhandled ones), then 40 random ops
  (UpdateAll on a random JObj, GetGlobalPosition types 0-5, GetByType, SetFlags, SRT change +
  SetMtxDirty, SetupMatrix). The callback hashes every call and writes the orientation /
  position vectors into the object's matrix (so later reads see them, as JObjUpdateFunc's
  would). `tests/oracle_robj.rs`: 16,619 checks, 0 failed, 27,018 callback calls, debug and
  release.
- NaN: the hashes canonicalise NaN. Degenerate matrices (zero columns written by the
  orientation callback) make get_scale / vec_normalize produce NaN; the C (x86 SSE) gives
  0xFFC00000 and the Rust port gives 0x7FC00000 in a debug build but 0xFFC00000 in release (the
  f64 path in `vec_normalize`/`frsqrte` is not sign-stable across opt levels). 27 checks
  differed only by that sign before canonicalising. This is a crate-level property of round 1's
  `mtx::vec_normalize`, not of robj.c; it matters only if a game NaN's bits are stored and read.
- Mutation: dropping the parent-chain concat in resolveCnsOrientation fails 770 checks, the
  expEvaluate deg->rad scale 1,352, LoadDesc's limit conversion for subtype 6 1,342, the obj
  column magnitude 2,898, resolveLimits case 11's translate.y (vs z) 1,160. Not discriminated:
  resolveCnsDirUp's `up = (0,0,1)` fallback (needs `|1 - dot| < 1e-10`; the random data never
  hits it).
- Not ported: the alloc-data getters / initialiser (native allocation), `HSD_RObjFree` (drop).
  Not wired yet: jobj.rs's `JObjHost::ik_hint_bone_length` / `robj_update_all` still stub;
  wiring them needs JObjUpdateFunc (jobj.c anim, not ported) and an RObj handle on the JObj.
