# Shard 945?952: lobj.c

## Scope and verification

All **64 definitions** in `src/sysdolphin/baselib/lobj.c` (including its **11 static helpers**)
are available under `melee_runtime::tu::lobj`. No definition is a TODO or smoke-only port.
Every C data structure is an existing generated `melee_types` record. No generator extension
was needed for this TU. Native pointers and numbers stay little-endian; no GC offset reads
are used by the native interface. The original renderer-facing `HsdRender` methods and
`stage_light_descs` API remain available.

The older `crate::lobj` implementation owns arena objects, whereas the complete TU interface
must operate on generated native pointer graphs and dispatch class methods. Its scalar union,
color-animation and attenuation logic is reused through a small generated-record adapter.
ID mapping and the already verified matrix/vector kernels are reused directly. Current-light
nodes use the shard's native `tu::list` functions, which reuse the existing SList ordering
implementation. Pointer/refcount/class orchestration follows the original C explicitly.

`oracle/build_lobj_complete.sh` includes the original `lobj.c` verbatim, compiles it with
`-ffp-contract=on` and the shipping initialization/integer flags, then applies `oracle/lower.py`.
It links the actual `mtx_pc.c` and `gekko_fp.c` arithmetic. The TU has **0 float contraction
sites and 4 saturated color conversions**; `mtx_pc` has 49 lowered contraction sites.
The new `oracle_lobj_complete` test replays **32 deterministic scenarios / 527,649 words**.
It compares ordered dependency calls and arguments, complete scalar light/GX object state,
WObj positions/flags/refcounts, native pointer identity, current/active lists, light masks,
class lookup/default transitions and return values. NaNs in vector/GX observations are
canonicalized identically on both sides. It checks scalar state immediately after each
animation update/setter, avoiding tests whose later write could hide an earlier error.

Coverage includes all four light types, raw and interpreted attenuation, hidden lights,
priority ordering, allocation/search fallback, null descriptor/animation paths, partial
animation chains, WObj replacement ownership, class release/destroy/amnesia, saturated
color updates, light-table limits, direct and second-pass specular allocation, dirty flags,
and both zero and nonzero specular IDs. The existing `oracle_lobj` test is preserved and
continues to cover 24,000 randomized logic operations. Initial missing native API checks
failed compilation before implementation; the native C transcript was generated before
writing the missing functions.

Commands run (each cargo command sets `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`):

```text
Git Bash: bash rust/melee_runtime/oracle/build_lobj_complete.sh
cargo test --manifest-path rust/Cargo.toml -p melee_runtime --test oracle_lobj_complete --test oracle_lobj
```

Mutation checks detected all three intentional errors: shininess multiplied by 0.25 instead
of 0.5 (seed 0, word 2861); selecting specular-only instead of diffuse+specular lights in
the second pass (seed 0, word 3781); reversing the point raw-attenuation bit check
(seed 0, word 241). The original source was restored and the test passed again.

## Function mapping

Both columns below refer to the original C definition and the exact-name native Rust export
in `rust/melee_runtime/src/tu/lobj.rs`. Static helpers are exposed for deterministic verification.

| C definition | Rust export | Implementation and verification |
|---|---|---|
| `HSD_LObjGetFlags` | `HSD_LObjGetFlags` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetFlags` | `HSD_LObjSetFlags` | generated-record native interface; C state/effects checked |
| `HSD_LObjClearFlags` | `HSD_LObjClearFlags` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetLightMaskDiffuse` | `HSD_LObjGetLightMaskDiffuse` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetLightMaskAttnFunc` | `HSD_LObjGetLightMaskAttnFunc` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetLightMaskAlpha` | `HSD_LObjGetLightMaskAlpha` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetLightMaskSpecular` | `HSD_LObjGetLightMaskSpecular` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetType` | `HSD_LObjGetType` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetActive` | `HSD_LObjSetActive` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetNbActive` | `HSD_LObjGetNbActive` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetActiveByID` | `HSD_LObjGetActiveByID` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetActiveByIndex` | `HSD_LObjGetActiveByIndex` | generated-record native interface; C state/effects checked |
| `HSD_LObjClearActive` | `HSD_LObjClearActive` | generated-record native interface; C state/effects checked |
| `LObjUpdateFunc` | `LObjUpdateFunc` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjAnim` | `HSD_LObjAnim` | generated-record native interface; C state/effects checked |
| `HSD_LObjAnimAll` | `HSD_LObjAnimAll` | generated-record native interface; C state/effects checked |
| `HSD_LObjReqAnim` | `HSD_LObjReqAnim` | generated-record native interface; C state/effects checked |
| `HSD_LObjReqAnimAll` | `HSD_LObjReqAnimAll` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetLightVector` | `HSD_LObjGetLightVector` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `HSD_LObjSetup` | `HSD_LObjSetup` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetupSpecularInit` | `HSD_LObjSetupSpecularInit` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `setup_diffuse_lightobj` | `setup_diffuse_lightobj` | generated-record native interface; C state/effects checked |
| `setup_spec_lightobj` | `setup_spec_lightobj` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `setup_infinite_lightobj` | `setup_infinite_lightobj` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `setup_point_lightobj` | `setup_point_lightobj` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `setup_spot_lightobj` | `setup_spot_lightobj` | existing `mtx` arithmetic kernels + native orchestration; C state/effects checked |
| `HSD_LObjSetupInit` | `HSD_LObjSetupInit` | generated-record native interface; C state/effects checked |
| `HSD_LObjAddCurrent` | `HSD_LObjAddCurrent` | generated-record native interface; C state/effects checked |
| `HSD_LObjUnrefThis` | `HSD_LObjUnrefThis` | generated-record native interface; C state/effects checked |
| `HSD_LObjDeleteCurrent` | `HSD_LObjDeleteCurrent` | generated-record native interface; C state/effects checked |
| `LObjRemoveAll` | `LObjRemoveAll` | generated-record native interface; C state/effects checked |
| `HSD_LObjDeleteCurrentAll` | `HSD_LObjDeleteCurrentAll` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetCurrentAll` | `HSD_LObjSetCurrentAll` | generated-record native interface; C state/effects checked |
| `LObjReplaceAll` | `LObjReplaceAll` | generated-record native interface; C state/effects checked |
| `HSD_LObj_803668EC` | `HSD_LObj_803668EC` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetCurrentByType` | `HSD_LObjGetCurrentByType` | generated-record native interface; C state/effects checked |
| `HSD_LightID2Index` | `HSD_LightID2Index` | existing `crate::lobj` ID mapping; C state/effects checked |
| `HSD_Index2LightID` | `HSD_Index2LightID` | existing `crate::lobj` ID mapping; C state/effects checked |
| `HSD_LObjRemoveAll` | `HSD_LObjRemoveAll` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetColor` | `HSD_LObjSetColor` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetColor` | `HSD_LObjGetColor` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetSpot` | `HSD_LObjSetSpot` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjSetDistAttn` | `HSD_LObjSetDistAttn` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjSetAttnA` | `HSD_LObjSetAttnA` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjSetAttnK` | `HSD_LObjSetAttnK` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjSetAttn` | `HSD_LObjSetAttn` | existing `HsdLObj` scalar operation via generated-record adapter; C state/effects checked |
| `HSD_LObjSetPosition` | `HSD_LObjSetPosition` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetPosition` | `HSD_LObjGetPosition` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetInterest` | `HSD_LObjSetInterest` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetInterest` | `HSD_LObjGetInterest` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetDefaultClass` | `HSD_LObjSetDefaultClass` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetDefaultClass` | `HSD_LObjGetDefaultClass` | generated-record native interface; C state/effects checked |
| `HSD_LObjAlloc` | `HSD_LObjAlloc` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetPositionWObj` | `HSD_LObjGetPositionWObj` | generated-record native interface; C state/effects checked |
| `HSD_LObjGetInterestWObj` | `HSD_LObjGetInterestWObj` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetPositionWObj` | `HSD_LObjSetPositionWObj` | generated-record native interface; C state/effects checked |
| `HSD_LObjSetInterestWObj` | `HSD_LObjSetInterestWObj` | generated-record native interface; C state/effects checked |
| `LObjLoad` | `LObjLoad` | generated-record native interface; C state/effects checked |
| `HSD_LObjLoadDesc` | `HSD_LObjLoadDesc` | generated-record native interface; C state/effects checked |
| `HSD_LObjAddAnim` | `HSD_LObjAddAnim` | generated-record native interface; C state/effects checked |
| `HSD_LObjAddAnimAll` | `HSD_LObjAddAnimAll` | generated-record native interface; C state/effects checked |
| `LObjRelease` | `LObjRelease` | generated-record native interface; C state/effects checked |
| `LObjAmnesia` | `LObjAmnesia` | generated-record native interface; C state/effects checked |
| `LObjInfoInit` | `LObjInfoInit` | generated-record native interface; C state/effects checked |

## External boundaries and limitations

The complete interface has an explicit `ext` module. Its exact C-name functions invoke a
thread-local native host hook; an absent hook raises a named TODO instead of fabricating
behavior. These are **cross-TU dependencies, not unported lobj.c functions**:

- Class/object: `hsdInitClassInfo`, `hsdIsDescendantOf`, `hsdNew`, `hsdSearchClassInfo`;
  `hsdObj()` supplies the address of the external class-info global.
- Animation: `HSD_AObjInterpretAnim`, `HSD_AObjLoadDesc`, `HSD_AObjRemove`, `HSD_AObjReqAnim`.
- WObj: `HSD_WObjAlloc`, `HSD_WObjLoadDesc`, `HSD_WObjUnref`, `HSD_WObjSetPosition`,
  `HSD_WObjGetPosition`, `HSD_WObjInterpretAnim`, `HSD_WObjReqAnim`, `HSD_WObjAddAnim`.
- GX: `GXInitLightColor`, `GXInitLightPos`, `GXInitLightDir`, `GXInitLightAttn`,
  `GXInitLightDistAttn`, `GXInitLightSpot`, `GXLoadLightObjImm`.

There are **23 external function stubs plus one external-global accessor**. The C oracle
uses deterministic doubles for these dependencies; GX doubles store and emit every relevant
argument rather than claiming GPU validation. Class methods are real native function pointers;
WObj/AObj and allocator doubles retain deterministic identities and report lifecycle calls.
The external functions' own implementation is outside this TU's verification scope.

Native static state is thread-local (one light system per host thread) so independent tests
cannot corrupt each other. Class storage remains stable for that thread's lifetime. Hosts
must keep native objects alive according to the original reference-count and class rules.
Generated class `sizeof` values are native sizes (120/240 on x64), as in the native C oracle.
As in C, out-of-range active-light writes and malformed pointer graphs are invalid input;
the Rust active array additionally bounds-checks such writes.

The preserved DAT renderer adapter still approximates spot lights/attenuation and is not
silently redirected to the new class/GX boundary. The new complete interface is verified
against a native C oracle, not integrated game replays or an actual GPU; no in-game/golden
parity claim is made here.
