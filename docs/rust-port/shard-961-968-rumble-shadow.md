# Shard 961–968: rumble.c and shadow.c

33/33 C function definitions are implemented in `rust/melee_runtime/src/tu/{rumble,shadow}.rs`. All records are generated `melee_types` records. No raw module is compiled. Both focused native-C oracle tests pass; this is not an in-game/replay verification.

## Coverage

`rumble.c` (15): HSD_PadRumbleOn, HSD_PadRumbleOffN, HSD_PadRumbleFree, HSD_PadRumbleRemove, HSD_PadRumbleRemoveAll, HSD_PadRumbleRemoveId, HSD_PadRumblePause, HSD_PadRumblePauseAll, HSD_PadRumbleUnpauseAll, func_80378430_inline, HSD_PadRumbleAdd, HSD_Rumble_80378524, HSD_PadRumbleInterpret1, HSD_PadRumbleInterpret, HSD_PadRumbleInit.

20,165 generated oracle rows compare return values, every field of the four rumble channels, rumble allocator metadata, every field of all twelve pool nodes (pointer indices normalized), and ordered OS interrupt/PAD motor calls. The 20,000 seeded operations cover every operation category (guarded in the test); Free, the priority insertion helper, and Interpret1 are exercised through callers. Four deterministic 40-frame sequences additionally check -1 replay, -2 stream-end deletion, positive frame expiry, zero-frame decrement, waits, command opcodes 0–5, and loop stack rewinds. Generated command bitfields are populated by field value on each side, accounting for native-C versus generated-GC bitfield order.

`shadow.c` (18): HSD_ShadowGetAllocData, HSD_ShadowInitAllocData, makeShadowTObj, HSD_ShadowAlloc, HSD_ShadowRemove, HSD_ShadowInit, HSD_ShadowSetSize, drawBackgroundRect, HSD_ShadowStartRender, HSD_ShadowEndRender, HSD_ShadowSetActive, HSD_ShadowAddObject, HSD_ShadowDeleteObject, makeMatrix, HSD_ShadowSetViewingRect, HSD_ViewingRectInit, HSD_ViewingRectCheck, HSD_ViewingRectAddRect.

1,800 generated scenarios check allocation-return identity, shadow scalar/pointer state, affected texture/image fields, camera reference counts, object links, all resulting matrix elements, all viewing-rectangle fields and its return value. Ordered external traces include scalar arguments, known fixture pointer identities, identity-matrix contents, and channel/TEV/PE descriptor payloads. Inputs vary projection type, active state, image allocation, object-list presence, refcount, intensity and rectangle positions. The render path executes with both empty and populated lists. C definition-count guards in both build scripts fail when the source function set changes or a Rust entry point is absent.

## Native oracle and reuse

Regenerate with:

```
python rust/melee_runtime/oracle/shard_rumble_build.py
python rust/melee_runtime/oracle/shard_shadow_gen.py
python rust/melee_runtime/oracle/shard_shadow_build.py
```

The native Windows clang compile uses shipping contraction and auto-init flags, then existing `oracle/lower.py`. Both TUs have zero direct fused sites. Shadow links the real native C `mtx_pc.c`, `gekko_fp.c`, and `lbtrigf.c`; lower.py reports 49 matrix and 8 trig f32 fused sites. Rust reuses existing `mtx::{light_perspective,light_frustum,light_ortho,concat,vec_*}` and `lbtrigf::atan2f`; small value adapters bridge generated Vec3 to the existing math Vec3.

Verification command (environment variables mandatory):

```
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'
cargo test --manifest-path rust/Cargo.toml -p melee_runtime --test shard_rumble --test shard_shadow
```

Both tests pass. Nine pre-existing naming warnings are unrelated. Mutation checks fail as expected: priority insertion `<=` changed to `<` fails rumble step 7665; channel GX_DF_CLAMP changed to GX_DF_SIGN fails shadow case 26. Mutations were restored and focused tests rerun successfully.

## Hooks and limits

Rumble's `ext` requires HSD_PadLibData, OSDisableInterrupts, OSRestoreInterrupts and PADControlMotor. Shadow's exact-C-name `ext` slots cover allocation/class lifecycle, camera operations, list/reference operations, render state, GX and perf calls; unset function hooks panic with TODO and the exact name. Header-inline callees are also isolated at this boundary by the C oracle's recording wrappers.

Existing `cobj.rs` cameras are owned `HsdCObj<R>` values, with tree/host arguments; existing `list.rs` uses a Vec-backed `SList<T>`. They are already ported logic, but cannot directly satisfy the generated `HSD_CObj*`/`HSD_SList*` ABI. Their hooks specifically require integration adapters, not missing C algorithms. The oracle intentionally records these boundaries instead of validating the underlying allocators, camera system, graphics driver, class deletion, references, or profiler implementation.

The tests check valid finite generated inputs. Invalid-pointer/assertion paths, hardware execution, allocation failure, malformed/nonterminating rumble bytecode, and exhaustive floating exceptional values are not covered. Shadow hashes the fields its functions affect, not padding or the entire unrelated HSD camera/texture object internals. No disc data is used; no full workspace tests, game build, replay run, commit, merge or push was performed by this worker.
