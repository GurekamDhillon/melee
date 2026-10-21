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
