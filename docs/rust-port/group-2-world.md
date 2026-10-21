# Port group 2: world (mp, cm, ef, gr, if) -> `rust/melee_world`

The task named this file "group-2 world: map, camera, effects, stages, HUD.md". Windows does
not allow `:` in file names (NTFS reads it as an alternate data stream), so it is
`group-2-world.md` instead.

Status as of 2026-10-21 (first night). Only a small, tested slice is ported.

## Size

Function definitions in the C (counted as a `name(...)` line followed by `{`):

| dir | functions | ported |
|---|---|---|
| src/melee/mp | 226 | 36 (mplib line-graph queries) |
| src/melee/cm | 182 | 0 |
| src/melee/ef | 67 | 0 |
| src/melee/gr | 2166 | 0 |
| src/melee/if | 372 | 0 |
| total | 3013 | 36 (about 1.2%) |

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

Regenerate: `bash rust/melee_world/cref/gen_mplib_lines_ref.sh`, then
`cd rust && cargo test -p melee_world`.

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

1. The rest of mplib: mpLineIntersection{,H,V}, mpRemap2d, and the floor, ceiling and wall
   point queries (mpLib_8004DD90_Floor etc.). These are pure float code over the same state, so
   the same C-reference harness can be extended to them.
2. mpcoll.c, the collision driver. It calls into fighter and item callbacks, which get stubbed.
3. cm/camera.c. `long double` literals become f64.
4. gr/*: 77 stage TUs, mostly callback tables plus per-stage logic. Port the shared grlib parts
   first.
5. if/* (HUD) and ef/* (effects). These are mostly render-side; whatever touches the RNG has to
   keep the draw order, or the golden seed column breaks.
