# Shard 961–968: particle SRT and RObj constraints

Worktree: `worktrees/shard-961-968`, branch `port/shard-961-968`.

`tu/psappsrt.rs` exposes all **8** definitions from psappsrt.c through the existing
`AppSrtPool` implementation. `tu/robj.rs` exposes all **44** definitions from robj.c
(including the two static inline helpers). Existing logic is re-exported; new
adapters cover allocator metadata, standalone free, nullable parent lookup, and
the half-open `_HSD_RObjForgetMemory` address range. No new C-layout records were
handwritten and no type generator was changed by this task.

## Verification

Regenerated native C using the existing shipping flags, `-ftrivial-auto-var-init=zero`,
and `oracle/lower.py`:

- `bash rust/melee_runtime/oracle/build_robj.sh`: 22,960 rows.
- `bash rust/melee_runtime/oracle/build_generator.sh`: 32,400 rows.
- `bash rust/melee_runtime/oracle/build_constraints_961.sh`: 1,024 supplemental
  scenarios, 2,126,336 bytes. The helper includes the actual robj.c, psappsrt.c and
  aobj.c sources, linking real fobj/id/list/spline/gekko code. Inputs are generated;
  no disc data is involved.

`cargo test --manifest-path rust/Cargo.toml -p melee_runtime --test oracle_robj
--test oracle_generator --test tu_constraints_961` passes all three tests. Every
cargo invocation set `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
`CARGO_INCREMENTAL=0`. No workspace tests were run. Existing unrelated naming
warnings remain in dbeffect/state. Formatting used the installed nightly rustfmt
on the three owned Rust files only.

Supplemental scenarios compare actual return values and SRT fields (including
matrix and scratch floats), live/peak counters, release callback arguments,
RObj/AObj state, allocator metadata getter identity and initialization sizes,
nullable parent lookup, reference-release call order/targets, allocation liveness
before/after standalone free, SRT allocation size/alignment, and expression
buffer range invalidation. Animations use real AObj execution with empty FObj
chains; RObjUpdateFunc is additionally called directly at values below/equal/above
0.5 and unrelated track types. Existing RObj solver tests exercise expressions,
descriptor chains, reference lookup and matrix operations.

Mutation check: changing `_HSD_RObjForgetMemory`'s `low <= ptr` to `low < ptr`
fails supplemental scenario 0 (cargo exit 101). Restored source passes all three
tests. Two further mutations also fail: making HSD_RObjFree a no-op and changing
psInitAppSRT's allocator alignment from 4 to 8. Restoring both passes all tests.
robj.c has no fused arithmetic of its own; the two fmuladd instructions in
its compiled IR come from included math helpers. Their existing explicit fusion
ports are reused unchanged. The supplemental helper has one additional fused
math helper site from aobj.c.

## Per-function coverage

`S` means supplemental oracle, `R` existing regenerated RObj solver oracle, and
`G` existing regenerated generator oracle. Indirect means the real definition is
executed by the named caller; no replacement implementation is used in the C oracle.

| psappsrt.c definition (8) | Coverage |
|---|---|
| psInitAppSRT | S, G: return, reset counters, size metadata and allocator size/alignment |
| psAddGeneratorAppSRT | S: direct allocation, complete resulting SRT and counters |
| psAddParticleAppSRT_begin | S, G: returned allocation and particle slot |
| psAddGeneratorAppSRT_begin | S: returned allocation and generator slot |
| psAttachParticleAppSRT | S, G: null/object/already attached, count and slot |
| psAttachGeneratorAppSRT | S, G: null/object/already attached, count and slot |
| psRemoveParticleAppSRT | S, G: absent/shared/final reference, callback and counters |
| psRemoveGeneratorSRT | S, G: absent/shared/final reference, owner clearing and callback |

| robj.c definition (44) | Coverage |
|---|---|
| HSD_RObjInitAllocData | S: generated native record sizes and initializer calls |
| HSD_RObjGetAllocData | S: stable identity |
| HSD_RvalueObjGetAllocData | S: stable identity |
| HSD_RObjSetFlags | R: resulting flags |
| HSD_RObjGetByType | R: returned chain index |
| RObjUpdateFunc | S: flags, threshold and unrelated types |
| HSD_RObjAnim | S: actual AObj state |
| HSD_RObjAnimAll | S: complete three-node animation state |
| HSD_RObjRemoveAnimByFlags | S: selected/unselected removal |
| HSD_RObjRemoveAnimAllByFlags | S: selected/unselected chain removal |
| HSD_RObjRemoveAnimAll | S: all animation slots cleared |
| HSD_RObjReqAnimByFlags | S: frames/flags |
| HSD_RObjReqAnimAllByFlags | S: frames/flags across chain |
| HSD_RObjReqAnimAll | S: unconditional request |
| HSD_RObjAddAnim | S: descriptor load, old animation replacement |
| HSD_RObjAddAnimAll | S: three descriptors |
| HSD_RObjGetConstraintType | S, R: return |
| HSD_RObjGetGlobalPosition | R: return and output position |
| set_dirup_matrix | R: indirect through UpdateAll, resulting joint matrix |
| resolveCnsDirUp | R: indirect through UpdateAll, callback/state |
| jobj_parent | S: null, root and child; R indirect |
| inlined_HSD_RObjGetByType | S: direct returned match; R indirect |
| resolveCnsOrientation | R: indirect through UpdateAll, joint/callback state |
| resolveLimits | R: indirect through UpdateAll, joint fields |
| HSD_RObjUpdateAll | R: callbacks and all joint/RObj state |
| HSD_RObjResolveRefs | R: indirect from ResolveRefsAll, targets/refcounts |
| HSD_RObjResolveRefsAll | R: targets/refcounts |
| HSD_RObjLoadDesc | R: all descriptor kinds, resulting chain |
| HSD_RObjRemove | S: unref callback trace, ownership consumed |
| HSD_RObjRemoveAll | S: ordered unref callbacks, ownership consumed |
| HSD_RObjAlloc | S: zero initialized observable object fields |
| HSD_RObjFree | S: C allocation release versus native slot liveness; no-op mutation detected |
| expEvaluate | R: indirect from UpdateAll, arguments, expression callbacks and outputs |
| dummy_func | S: default descriptor function returns zero |
| HSD_RvalueAlloc | S: zero fields |
| HSD_RvalueRemove | S: actual unref-call trace |
| HSD_RvalueRemoveAll | S: ordered unref-call trace, including null targets |
| loadRvalue | R: indirect from descriptor load, resulting chains |
| expLoadDesc | S null/default, R expression chains |
| bcexpLoadDesc | S default, R bytecode chains |
| HSD_RvalueResolveRefs | R: indirect from ResolveRefsAll, reference targets/counts |
| HSD_RvalueResolveRefsAll | R: targets/counts |
| HSD_RObjSetConstraintObj | R: old/new targets/refcounts |
| _HSD_RObjForgetMemory | S: excluded and included lower address boundary, pointer/size |

## External interfaces and limits

The only new unresolved external callee is `HSD_ObjAllocInit`, exposed in both
`tu::robj::ext` and `tu::psappsrt::ext` with the exact C name and a default TODO
panic. Allocator records are generated
`melee_types::HSD_ObjAllocData`. Existing RObjHost/JObjHost and AppSrtHost interfaces
remain in use; existing expression evaluation calls already-ported bytecode/math
through the established host model.

These are the established native ownership APIs, not raw-pointer ABI replacements.
Chains are slices, references are arena IDs, free transfers ownership to Rust,
and pool initialization drops old native objects. The C allocator's failure paths,
free-list bytes, address reuse, leaks on reinitialization, and partial-size memset
behavior are not represented by that model and are not claimed oracle-equal.
Standalone HSD_RObjFree takes `&mut Option<HsdRObj>` and consumes the owned slot.
The oracle compares the C allocator's live-to-freed transition with the Rust
slot's owned-to-empty transition. The test therefore checks actual production
behavior, not a hardcoded expected destruction count; a no-op mutation fails.
Raw allocator bookkeeping beyond that ownership transition is not compared.
Removal's JObj unref effects are compared through actual recorded callbacks,
not calculated expected destruction counts. The existing model also documents
that arbitrary cross-member RObj union reinterpretation is unsupported.

Thus all 52 definitions are exposed and exercised across the oracles, but a claim
of complete raw allocator-state equivalence would require replacing/extending the
established owned object model. No Rust game boot/replay, shipping exe run or
screen verification was performed by this task.
