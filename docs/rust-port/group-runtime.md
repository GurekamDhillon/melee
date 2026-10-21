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

## Port round 8: JObj animation, JObjUpdateFunc, the IK solvers

Round 7's RObj port was what jobj.c's IK solvers and its `JObjHost` stubs were waiting for, so the
rest of jobj.c's logic went next. 23 jobj.c definitions in `jobj_rt.rs`:

| function(s) | checked against C |
|---|---|
| resolveIKJoint1, resolveIKJoint2, jobj_get_effector_checked, jobj_get_joint2 / jobj_get_effector (static inlines) | yes (through HSD_JObjSetupMatrix) |
| JObjUpdateFunc (every type but PATH and the SETBYTE / SETFLOAT callback list), HSD_JObjSetDPtclCallback (type 0x28) | yes |
| HSD_JObjRemoveAnimByFlags / AllByFlags / RemoveAnim / RemoveAnimAll, ReqAnimByFlags / AllByFlags / ReqAnimAll / ReqAnim, JObjSortAnim, HSD_JObjAddAnim / AddAnimAll, HSD_JObjAnim, JObjAnimAll, HSD_JObjAnimAll, HSD_JObjSetCurrent / GetCurrent | no (walkers over the checked AObj / RObj ports and DObj stubs) |

- Native shape: `JObjRt<X>` is the `JObjHost` for a `JObjTree<D, u32>`: `R` is an index into its
  RObj arena `robjs` (each JObj's `robj` Vec is its chain), so the round 5 effector branch,
  `HSD_RObjUpdateAll(jobj->robj, jobj, JObjUpdateFunc)` and the IK solvers now run for real.
  `JObjGlobals` holds jobj.c's statics (`ufc_callbacks`, the three particle / sound callbacks,
  `current_jobj`). `HSD_ObjData` is `ObjData` (3 words). `HsdAnimJoint` and
  `HsdSubAnimJoint<A>` (Mat/ShapeAnimJoint, payload = the DObj port's handle) are owned trees.
- Stubs via `JObjExt` (TODO(group1 dobj)): the four DObj anim walkers; `splArcLengthPoint` on the
  AObj target's spline (TODO(group1 jobj spline)); AObjLoadDesc's obj_id lookup (TODO(group1 jobj
  load)); `HSD_JObjRef` / `Unref` (TODO(group1 class)). `ext` is `Rc<RefCell<X>>` so robj.c's
  `update_all` can take it as its `RObjHost` while `JObjRt` is its `JObjHost`.
- Ordering difference, inside `HSD_RObjUpdateAll` only: an IK-hint `rotate_x` write (ROTX on a
  JOINT1) and the SETBYTE / SETFLOAT / 0x28-0x2A callbacks raised by the update are applied after
  it rather than during it. Nothing in RObjUpdateAll reads the hint; the callbacks need an
  expression RObj of type >= 20. A PATH update from RObjUpdateAll panics (needs the spline).
- Uninitialised locals: resolveIKJoint1's short-path branch (`|eff - origin|^2 <= 1e-8`) reads
  `var_f28`, `var_f27`, `sp80`, `sp74` unset; the shipping build zero-initialises them and so does
  the port. The oracle hits that branch (zero-length path: effector target = J1's parent), and a
  non-zero value there fails 3,440 checks.
- FMA: `fma_sites.py` finds one fused site, resolveIKJoint1's `2*(a+b) - d`, ported with
  `fmadds`. It cannot be discriminated: `2*x` is exact, so fused and unfused agree (unfused: 0
  checks fail). JObjUpdateFunc, resolveIKJoint2 and the walkers have none.
- Oracle: `oracle/build_ik.sh` compiles jobj.c, robj.c, mtx.c, mtx_pc.c, gekko_fp.c, lbtrigf.c,
  `helper_robj.c` and `helper_ik.c` with the shipping flags + `lower.py`; `driver_ik.c` writes
  `tests/data/oracle_ik.txt` (29k rows, 3.1 MB): 400 forests of anchors, plain JObjs with JObj /
  limit constraints, and 1-3 IK chains JOINT1 [-> JOINT2] -> EFFECTOR (IK hints with and without
  the flip bit 4 and rotate_x, a pole on JOINT1, a limit on JOINT1 so RObjUpdateAll + remake runs,
  limits 5 / 6 on JOINT2, an active / inactive effector target), x 50 random ops (SetupMatrix,
  SRT change + SetMtxDirty, JObjUpdateFunc with 31 type values incl. scale-clamp and 0/1 values,
  SetMtxDirty). Each row checks a hash of every JObj and RObj (and the dptcl callback calls) and
  the written-back value. `tests/oracle_ik.rs`: 20,400 checks, 0 failed, debug and release.
- Mutation: IK2's limit clamp fails 2,948 checks, IK1's flip (bit 4) 9,080, the pole rotation
  4,467, the RObjUpdateAll call in IK1 1,878 (after the JOINT1 limit was added; 0 before), the
  scale clamp 1,482.
- Not ported in jobj.c: HSD_JObjDispAll (render), load (JObjLoadJointSub, JObjLoad,
  HSD_JObjLoadJoint, ResolveRefs(+All)), the class system (SetDefaultClass, Unref, UnrefThis,
  JObjInit as a class method, ReleaseChild, Release, Amnesia, InfoInit).

## Night 3, port round 1 (round 9): recount, bytecode, lb_00CE math, CObj logic

### Recount (start of round)
Total: **2,297** distinct function definitions in group 1's TUs (`src/sysdolphin/baselib/*.c`
1,502, `src/melee/lb/*.c` 655, `db` 92, `sfx` 23, plus gekko_fp.c, mtx_pc.c, mtx44.c; IR
`define`s from `tools/rust-port/fma_sites.py` over every TU, header inlines counted once; MSL's
math_ppc inlines are among them). The old "~2000" was an estimate.
Ported before this round (from each round's table above and in the night-1 doc): **316**, of which
**271** are checked against the C and **45** are not (aobj 5, wobj 6, robj 18, round-8 jobj 16;
each listed in its round). The RUST_PORT.md row said "~235"; that total predated rounds 6-8.

### Stubs whose target now exists
- `RObjHost::byte_code_eval` (robj.rs): `HSD_ByteCodeEval` is ported (`bytecode.rs`); the trait
  method stays (the implementation needs the host's `HsdRand`), its TODO is replaced by the
  pointer to `bytecode::byte_code_eval`.
- jobj.rs `JObjHost` docs: the RObj / IK entries were still marked TODO(group1 robj) although
  `jobj_rt::JObjRt` implements them for real since round 8; comments corrected.
- Not done: wobj.rs `WObjHost`'s RObj methods (`robj_*_anim_all`, `robj_load_desc`) could be
  backed by robj.rs, but `HsdWObj<R>` is generic over the RObj handle and the robj.rs walkers take
  `&mut [HsdRObj]`; wiring needs a concrete `R` (as `JObjRt` did for JObj). Left as is.

### Ported this round

| C TU | function(s) | module | checked against C |
|---|---|---|---|
| baselib/bytecode.c | HSD_ByteCodeEval (every opcode) | `bytecode.rs` | yes |
| lb/lb_00CE.c | expf, powf, powi | `lb_00ce.rs` | yes |
| baselib/cobj.c | SetMtxDirty, MtxIsDirty, Get/Set/ClearFlags, Get/SetInterest, Get/SetEyePosition, GetEyeVector, GetEyeDistance, upvec2roll, roll2upvec, Get/SetUpVector, GetLeftVector, SetRoll, SetupViewingMtx, GetViewingMtx, GetViewingMtxPtr(+Direct), GetInvViewingMtxPtr(+Direct), Get/SetFov, Get/SetAspect, GetAuthoredAspect, UpdateView (TARGET_PC), Get/SetTop, Bottom, Left, Right, Get/SetNear, Far, Get/SetScissor, SetScissorx4, GetViewportf, SetViewport, SetViewportf, SetViewportfx4, Get/SetProjectionType, SetPerspective, SetFrustum, SetOrtho, GetPerspective, GetOrtho, makeProjectionMtx; the static inlines vec_get_x, vec_get_abs_y, cobj_get_up_x, get_up_vector_for_viewing_mtx(_inner); util.h vec_normalize_check | `cobj.rs` | yes (62) |
| baselib/cobj.c | Get/SetInterestWObj, Get/SetEyePositionWObj | `cobj.rs` | no (field accessors; the driver never calls them) |

70 functions: 66 checked against the C, 4 not.

- bytecode.c: the `HSD_SList` of `void*` is a `Vec<u32>` (top at the end; only the low word of a
  stack slot is ever read). `logf` (opcode 0x13) is the host CRT's in the shipping build (MSL
  math.c is not compiled, nothing else defines it); the port uses `f32::ln`, which on Windows is the
  same CRT `logf`, and the oracle links that CRT, so 0x13 is checked only CRT against CRT. Integer
  `/` and `%` use `cabi::sdiv` / `srem` (gwtool's safe division); the oracle has no division
  lowering (native idiv traps), so it only divides by non-zero, non -1 constants.
- lb_00CE.c: both series loop until the partial sum stops changing. In f32 `expf(|x| >= ~13)`
  reaches `inf/inf` (term and factorial both overflow) and **never terminates, in the C too**, as
  does `expf(NaN)` and `powf(x, y)` when `y * 2*atanh(..)` is `0 * inf`. The port keeps that.
  The oracle keeps to |x| <= 10. (`lb_8000D008` / `lb_8000D148` from the same TU were ported by
  group 6 in `melee_modes::lb`; not duplicated.)
- cobj.c native shape: `HsdCObj<R>` owns its eye / interest WObjs; the `u {roll; up}` union is one
  `Vec3` (`roll` = `u.x`); `projection_param` is `[f32; 4]`; `proj_mtx` is `Option<Mtx>`, a new
  one zeroed (the C's is uninitialised object memory, read only when the view matrix is
  singular). `HSD_CObjUpdateView` reads the platform's `View_ProjectionScaleBits()`; the callers
  take its float as `proj_scale`. GcLayout delegated to `melee_types::HSD_CObj` (0x94 with the
  TARGET_PC tail). The TARGET_PC `diag_cobj_view` call in SetupViewingMtx (logging only) is not
  ported.
- FMA (`fma_sites.py`): bytecode.c none of its own (inlined MSL fmodf, math_ppc sqrtf); lb_00CE's
  three none; cobj.c: roll2upvec's two `a*a + b*b` (ported with `fmadds`), the setup*Camera ones
  are not ported. Discriminated: unfusing roll2upvec's |eye.y| ~ 1 branch fails 3 rows (after a dedicated
  near-vertical op was added), the other branch reaches the test's 50-failure stop by row 360.
- Oracles: `oracle/build_bytecode.sh` (bytecode.c, list.c, random.c, lb_00CE.c, lbtrigf.c,
  gekko_fp.c) -> `tests/data/oracle_bytecode.txt` (2.5 MB): 3000 expf, 3000 powf, 2000 powi rows
  and 20,000 random stack programs (tracked depth, branches / jumps over net-zero blocks, every
  opcode, random seed in and out). `tests/oracle_bytecode.rs`: 48,000 checks, 0 failed, debug and
  release. Mutations: 0x37 `==` -> `!=` fails 91, swapped atan2f args 92, `Randi(2)` -> 3 206,
  expf's factorial off by one 6,198.
  `oracle/build_cobj.sh` (cobj.c through `helper_cobj.c`, over the real wobj.c, jobj.c, mtx.c,
  mtx_pc.c, mtx44.c, gekko_fp.c, lbtrigf.c) -> `tests/data/oracle_cobj.txt` (4.1 MB): 500 cameras
  x 60 random ops (all 53 op kinds incl. NULL cameras, vertical eyes, up vectors along the eye,
  WObj moves); each row checks the results and an FNV-1a hash of the whole camera + WObj state.
  `tests/oracle_cobj.rs`: 30,000 rows, 0 failed, debug and release. NaN is canonicalised (the
  C's view matrices from degenerate look-ats carry 7fc/ffc NaNs; same crate-level NaN-sign
  property as round 7). Mutations (the test stops at 50 failures): dropping UpdateView's scale hits 50 by row 1,391,
  the interest dirty bit by row 1,138, DegToRad in f64 by row 3,034, `upvec2roll`'s +-pi/2
  constant by row 959.
  Not discriminated: upvec2roll's `1 - |dot| < FLT_MIN` vs `< 0` (an up vector exactly along
  the eye gives `dot == 1` either way).
- WObjs in the oracle have no AObj target (the local-space path is wobj.c's, checked in round 6).
- Not ported in cobj.c: the display half (EraseScreen, SetCurrent / EndCurrent, setupOffscreen /
  Normal / TopHalf / BottomHalf camera: GX), the anim functions, load / init / class, GetCurrent.

### Count after this round
Ported **386 of 2,297** (16.8%): **337** checked against the C, **49** not (the 45 above + the
4 CObj WObj accessors).

## Night 3, port round 2 (round 10): LObj logic

### Recount (start of round)
No group 1 commit since round 9 (`git log -- rust/melee_runtime`), so the committed count is
round 9's: **386 of 2,297** (337 checked against the C, 49 not). Not counted: the uncommitted
`particle.rs` / `generator.rs` / `psappsrt.rs` (+ `oracle_generator`) in the working tree, another
agent's work in progress (not touched or committed by this round; `lib.rs`'s three `pub mod` lines
for them are left unstaged).

### Stubs whose target now exists
None newly available. Checked: the TODO(group1 class / jobj load / jobj spline / dobj) stubs in
jobj.rs, jobj_rt.rs, robj.rs, aobj.rs, wobj.rs and particle.rs still point at unported code
(class system, JObj load, DObj/MObj/PObj anim). wobj.rs's RObj methods stay as described in round 9.

### Ported this round
`lobj.c` sits on round 6's WObj (`set_position`, `get_position`), round 4's AObj (type only) and
the round 1 PSVEC ports, and nothing else ported depends on what it still lacks, so it went next.
`lobj.rs`, 41 definitions, **all checked against the C**:

| function(s) | checked against C |
|---|---|
| HSD_LObjGetFlags, SetFlags, ClearFlags, GetType, the header inline HSD_LObjGetPriority (u8 of the u16 field) | yes |
| HSD_LObjGetLightMaskDiffuse / AttnFunc / Alpha / Specular | yes, but only at 0 (their statics are written by HSD_LObjSetupInit, GX, not ported) |
| HSD_LObjSetActive, GetNbActive, GetActiveByID, GetActiveByIndex, ClearActive, HSD_LightID2Index, HSD_Index2LightID | yes |
| LObjUpdateFunc (every track type 0-25, raw / spot params, colour channels) | yes |
| HSD_LObjAddCurrent, HSD_LObjUnrefThis, DeleteCurrent, the static inline LObjRemoveAll, DeleteCurrentAll, SetCurrentAll, the static inline LObjReplaceAll, HSD_LObj_803668EC, GetCurrentByType, HSD_LObjRemoveAll | yes |
| HSD_LObjSetColor, GetColor, SetSpot, SetDistAttn, SetAttnA, SetAttnK, SetAttn | yes |
| HSD_LObjSetPosition, GetPosition, SetInterest, GetInterest, GetPositionWObj, GetInterestWObj, GetLightVector | yes |

- Native shape: `LObjSystem<R>` is an arena of `Option<HsdLObj<R>>` (None = destroyed by
  UnrefThis) plus lobj.c's statics; `current_lights` is a `Vec` in list order; `next` is an arena
  id. `HsdLObj::ref_count` keeps `ref_INC` / `ref_DEC` exactly (post-decrement: a light is
  destroyed only when its count is 0 on entry, and the count wraps to 0xFFFF). The union `u` is six
  raw words (point / spot / attn members at the C offsets). GcLayout delegated to
  `melee_types::HSD_LObj`. The GX light objects are not kept.
- Float->u8: `LObjUpdateFunc`'s `color.r = 255.0 * fv` is `fptoui double to i8`, which gwtool
  lowers to `llvm.fptoui.sat.i32` + truncate (unsigned saturation). **`cabi::f2u8` saturates
  signed** and differs for negative values (e.g. fv = -0.5: C 0, `f2u8` 0x81). lobj.rs uses its own
  `f2u8_unsigned`; with `cabi::f2u8` 563 rows fail. `cabi::f2u8` is left as is (its one user,
  melee_world `gr/leaves.rs`, should check whether its C target is unsigned).
- FMA: `fma_sites.py` finds no fused site in lobj.c's ported functions.
- Oracle: `oracle/build_lobj.sh` compiles lobj.c (through `helper_lobj.c`, statics reachable),
  wobj.c, list.c, jobj.c, mtx.c, spline.c, dobj.c, mtx_pc.c, gekko_fp.c, lbtrigf.c, mtx44.c with
  the shipping flags + `lower.py` (4 fptoi sites saturated in lobj.c); the class methods are a
  fake HSD_ClassInfo whose release counts and destroy marks the light dead (memory kept).
  `driver_lobj.c` writes `tests/data/oracle_lobj.txt` (24.3k rows, 4.1 MB): 300 scenarios of 8
  lights (random refcounts 0-2, flags, priorities incl. > 0xFF, colours, WObjs present or not,
  union words, acyclic `next` chains) x 80 random ops over all 32 op kinds (NULL lights, colour
  values < 0 and > 1, VIS at exactly 0.5, IDs and indices out of range). Each row checks the
  results and an FNV-1a hash of the whole state (active table, current list, release count, every
  light). `tests/oracle_lobj.rs`: 24,000 rows, 0 failed, debug and release; every op kind > 700
  rows.
- Mutation: signed f2u8 fails 563 rows, a u16 priority compare 84, `>=` for `>` in AddCurrent's
  insertion 758, the spot ref_dist word (3) written as attn a1's (1) 832; `ref_DEC` testing 1
  instead of 0 panics (destroyed light used).
- The driver keeps to what the C keeps valid: SetActive only while a non-ambient slot is left
  (`nb_active_lights++` past 9 writes out of bounds), chain walks only over live lights, IDs
  passed to HSD_LightID2Index only from the nine valid ones (others assert; the port panics).
- Not ported in lobj.c: the GX half (HSD_LObjSetup, SetupSpecularInit, setup_*_lightobj,
  SetupInit), the anim functions (Anim / AnimAll / ReqAnim / ReqAnimAll / AddAnim / AddAnimAll),
  LObjLoad / LoadDesc / Alloc, the class functions (Set/GetDefaultClass, LObjRelease, Amnesia,
  InfoInit), Set{Position,Interest}WObj (`HSD_WObjUnref`'s class release).

### Count after this round
Ported **427 of 2,297** (18.6%): **378** checked against the C, **49** not (unchanged list).

## Night 3, port round 3 (round 11): fog.c logic

### Recount (start of round)
No group 1 commit since round 10 (`git log -- rust/melee_runtime`): **427 of 2,297** (378 checked
against the C, 49 not). Still not counted: `generator.rs` / `particle.rs` / `psappsrt.rs` (+
`oracle_generator`, which passes) in the working tree. They are another agent's uncommitted work
(the golden-seed track; `rust/melee/src/{effect,stage,vs_match}.rs` were being edited at the same
time and use `particle.rs`). This round leaves them alone and does not commit their three
`pub mod` lines in `lib.rs`.

### Stubs whose target now exists
None. The TODO(group1 class / jobj load / jobj spline / dobj / robj) stubs still point at code
nobody has ported. wobj.rs's RObj methods are as described in round 9.

### Ported this round
`fog.rs`, from `fog.c`. It depends only on round 4's AObj, so it was the lowest-level file left:

| function | checked against C |
|---|---|
| FogUpdateFunc (tracks 1, 2, 5-8, 20; every other type is a no-op) | yes |
| HSD_FogInit (desc / viewport), HSD_FogAdjInit (desc / viewport) | yes |
| HSD_FogSet (NULL fog, adj width != 0, projection types 0 / 1 / other, range clamp) through a `FogGx` trait | yes, except that no row has range exactly 641 |
| HSD_FogReqAnim, HSD_FogReqAnimByFlags, HSD_FogInterpretAnim, HSD_Fog_8037DE7C | yes, but only with AObjs that have no FObj tracks |
| HSD_FogLoadDesc, HSD_FogAdjLoadDesc | yes |
| HSD_FogAlloc, HSD_FogAdjAlloc (a zeroed native record) | no: exercised only through LoadDesc |

- Conversions: the colour bytes and the `u16` widths are `fptoui` saturated to 32 bits and then
  truncated. `cabi::f2u8` and `f2u16` saturate as signed and would be wrong here. The centre is
  `cabi::f2i16`. `PSMTXCopy` and `PSMTXIdentity` write only rows 0-2 of the Mtx44.
- FMA: helper_fog.ll has 0 fmuladds; lower.py reports 9 fptoi sites, all of them fog.c's.
- Oracle: `oracle/build_fog.sh` compiles fog.c through `helper_fog.c` (GX recorded, CObj faked)
  over the real aobj.c, fobj.c, id.c, list.c, spline.c, mtx_pc.c and gekko_fp.c, with the shipping
  flags + lower.py. `tests/data/oracle_fog.txt` has 8,000 ops (2.9 MB, every op kind > 680). Each
  row checks the GX call log (args as bits) and the whole fog state. `tests/oracle_fog.rs`: 0
  failed.
- Mutations: these fail: signed u8 colour (row 344), copying all 4 matrix rows (row 1), signed u16
  width (row 67), Rust `as i16` centre (row 1192), +1 for -1 in the type-0 projection (row 15).
  The clamp `> 641` is **not** caught.
- Not ported: FogRelease, FogInfoInit, FogAdjInfoInit (class system).

### Count after this round
Ported **439 of 2,297** (19.1%): **388** checked against the C, **51** not (the 49 before + the
two Alloc functions).

## Night 3, port round 4 (round 12): tobj.c logic

### Recount (start of round)
Since round 11 (8cc2cf4d8) the sim agent's efa84f09f committed the particle work into this crate
(`generator.rs`, `particle.rs`, `psappsrt.rs`, `oracle_generator`). Counted now:
- **checked vs C (23)**, through `oracle_generator` (32.4k rows; direct op or reached from one):
  generator.c HSD_StageSlotParticlesClear, hsd_8039D1E4, D1EC, D214, D354, D3AC, D4DC, D580,
  D5DC, D688, D71C, D9C8, DAD4, EE24, EFAC, F05C, F6CC (17); psappsrt.c psInitAppSRT,
  psAddParticleAppSRT_begin, psAttachGeneratorAppSRT, psAttachParticleAppSRT,
  psRemoveGeneratorSRT, psRemoveParticleAppSRT (6).
- **not checked vs a C oracle (18)**: psAddGeneratorAppSRT, psAddGeneratorAppSRT_begin (only
  behind F05C's kind 0x20000, not shown to be hit); particle.c HSD_StageSlotParticleLinkClear,
  hsd_803983A4, 80398A08, 80398F0C, 80398F8C, 803991D8, 8039930C, 8039CEAC, 8039CF4C, 8039D048,
  8039D0A0, psEnableTexture, psGenerateParticle0, psInitDataBankLoad, psInitDataBankLocate,
  psReadFloat/psSpawnChild (16; evidence is only the synthetic `particle/tests.rs` and the
  pad_falcon_fox_fd golden seed matching frames 0-63, which is end-to-end, not per function).
Recount: **480 of 2,297** (411 checked, 69 not).

### Stubs whose target now exists
None found: the TODO(group1 ...) stubs (class, jobj load, jobj spline, dobj, robj) still point at
unported code.

### Ported this round
`tobj.rs`, from `tobj.c` (needs only round 1's mtx/HSD_MkRotationMtx):

| function | checked against C |
|---|---|
| HSD_TexMapID2PTTexMtx, HSD_TGTex2Index, HSD_TexCoordID2TexGenSrc, HSD_TexCoord2Index, HSD_Index2TexCoord, HSD_TexMtx2Index, HSD_Index2TexMtx, HSD_Index2TexMap, HSD_TexMap2Index | yes, every valid input (out-of-range inputs panic in both; not compared) |
| DifferentTluts (non-MUST_MATCH: entry counts only) | yes |
| MakeTextureMtx (epsilon 1e-10 per tobj.c, GX_MIRROR offset) | yes |
| TObjUpdateFunc (all 24 track types + unknown 0/25/26; PC `s32` tlut_no) | yes |

- Native shape: `HsdTObj<I>` (I = image-desc handle), owned `lod` / `tev`, `has_tluttbl` flag.
  GcLayout delegated to `melee_types::HSD_TObj`, `_HSD_TObjTev`, `_HSD_Tlut`, `_HSD_TexLODDesc`.
- `HSD_TexCoordID2TexGenSrc(GX_TEXCOORD7)` asserts in the C (case falls into default); kept.
- Conversions: TEV/konst bytes are `fptoui` saturated to 32 bits then truncated (not
  `cabi::f2u8`); TIMG index and tlut_no are `cabi::f2i32`.
- FMA: 0 fmuladd in MakeTextureMtx / TObjUpdateFunc (callees' sites are the ported mtx code's).
- Oracle: `oracle/build_tobj.sh` (helper_tobj.c includes tobj.c; real mtx.c, mtx_pc.c,
  gekko_fp.c, lbtrigf.c; shipping flags + lower.py; links with lld `/FORCE:UNRESOLVED` since the
  GX/TEV half is never called). `tests/data/oracle_tobj.txt`: 20,000 rows, ~5000 per op;
  update rows hash the whole TObj state after each op on a persistent TObj. 0 failed, debug and
  release.
- Mutations caught: signed `f2u8` (2548 rows), mirror term rewritten as `scale/repeat` (49),
  `(u8)` tlut_no (668), `repeat * (1/scale)` (1009), rot.z not negated (3899).
- Not ported: TObj anim entry points (need AObj per TObj), load/alloc/class, GX setup / TExp.

### Count after this round
Ported **492 of 2,297** (21.4%): **423** checked against the C, **69** not (the 51 before + the 18
particle/psappsrt functions above).
