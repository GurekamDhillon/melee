# Group 1: runtime, math, HSD (core + render)

Crate: `rust/melee_runtime`. Status as of 2026-10-21 (first night).

## The rule every crate needs: fuse exactly where clang fused

The C build is **not** contraction-free. `_build/masstest/pipe_win.sh` compiles game TUs with
clang's default `-ffp-contract=on` (no `-ffp-contract=off` there), so every `a*b + c`, `a*b - c`
and `c - a*b` *inside one C expression* becomes `llvm.fmuladd`. gwtool's `lowerFMulAdd` then
turns an f32 one into `(f32)((f64)a*(f64)b + (f64)c)` (= `melee_core::fp::fmadds`) and an f64
one into a true `llvm.fma.f64`. (`docs/rust-port/inventory.md` says "no contraction anywhere
else"; that is wrong for the C build, and the c2rust run used `-ffp-contract=off`, so raw c2rust
output has lost every fusion.)

Clang's choice (`tryEmitFMulAdd`): in `A + B` / `A - B`, if `A` is a multiply it is fused, else if
`B` is. So `a*b + c*d` is `fmadds(a, b, c*d)`; `X - a*b` is `fmadds(-a, b, X)`; `a*b - X` is
`fmadds(a, b, -X)`; `a*b + c*d + e*f` is `fmadds(e, f, fmadds(a, b, c*d))`. Compound assignment
fuses too: `a.x += m.x * f` is `fmadds(m.x, f, a.x)`. Calls, casts and statement boundaries stop
it. Measured cost of getting it wrong: replacing ONE fused step in lbtrigf's Newton iteration with
plain arithmetic fails 99 of 31,900 checks. `sqrtf` in math_ppc.h is a double case:
`3.0 - guess*guess*x` is `(-(g*g)).mul_add(x, 3.0)`, a real f64 fma.

To see where clang fused in a C function: compile it with the oracle flags to IR and grep
`fmuladd` (`oracle/build.sh` leaves the `.ll` files in `rust/target/g1-oracle/`).

Other gwtool semantics a port must keep (`makeArithmeticSafe`; helpers in `src/cabi.rs`,
TODO(move-to-core)): float->int casts saturate (`(int)f` = Rust `as i32`, but `(u8)f` /
`(s16)f` saturate to **i32** and then truncate, unlike Rust `as u8`); `x/0`, `INT_MIN/-1`,
`x%0` return defined values; variable shifts >= 32 give 0 (or the sign fill). Uninitialised
locals are 0 (`-ftrivial-auto-var-init=zero`).

## Oracle (how the tests get C values)

`rust/melee_runtime/oracle/build.sh` (Git Bash, LLVM at `C:\Program Files\LLVM`) compiles the C
TUs natively to LLVM IR with the real build's flags (contraction on, auto-var-init=zero),
reapplies gwtool's two float rewrites to the IR text (`lower.py`: fmuladd lowering, saturating
fptosi/fptoui), links a driver (`driver.c`), and writes `tests/data/oracle_vectors.txt` (hex bit
patterns; no disc data). `tests/oracle.rs` replays every row and compares bit for bit. Currently
11,400 rows / ~50k checks, all equal. The oracle is natively compiled C with gwtool's lowering
reproduced, not the gwtool-built exe itself; it does not prove the in-game values (nothing in
this crate has run inside the game).

## Ported (native Rust, oracle-checked)

| C source | Rust | functions |
|---|---|---|
| `pc/gameworld/gekko_fp.c` | `gekko.rs` | `__frsqrte`, `__fres`, `sinf`, `cosf`, `tanf`, all helpers (14/14) |
| `src/MSL/math_ppc.h` | `gekko.rs` | `sqrtf`, `sqrtf_accurate` (2/2) |
| `pc/gameworld/mtx_pc.c` | `mtx.rs` | all 26 (PSMTX*, PSVEC*, C_MTXLookAt, MTXLight*) |
| `libs/dolphin/src/dolphin/mtx/mtx44.c` | `mtx.rs` | `MTXFrustum`, `MTXPerspective`, `MTXOrtho` (3/3) |
| `src/sysdolphin/baselib/random.c` (+ `random.h` `randi`) | `random.rs` | all 8, incl. the PC render stream |
| `src/melee/lb/lbtrigf.c` | `lbtrigf.rs` | `atanf` (PC), `atan2f`, `acosf`, `asinf`, `lb_sqrtf` (5/5) |
| `src/melee/lb/lbvector.c` + `lbvector.h` | `lbvector.rs` | 22 of 23 (not `lbVector_WorldToScreen`: needs HSD_CObj, GXProject) |
| `src/sysdolphin/baselib/mtx.c` (+ mtx.h `fabsf_bitwise`) | `hsd_mtx.rs` | 14 of 22: inverse (+ aliased form), inverse-concat, inverse-transpose, determinant, GetRotation/Translate/Scale, MkRotationMtx, MtxQuat, SRT, SRTQuat, ScaledAdd (port round 1) |
| `src/sysdolphin/baselib/quatlib.c` | `quatlib.rs` | 6/6 (port round 1) |
| `src/sysdolphin/baselib/spline.c` | `spline.rs` | 11/11 incl. statics (port round 1) |

Struct records: `Vec3` and `Quaternion` carry `gc_layout!` GameCube offsets; `Mtx`/`Mtx44` are
plain `[[f32; 4]; N]` (same element order as the GC). `Vec3` implements `StateHash`. These
should move to a shared crate (TODO(move-to-core)).

API shape: C routines that allow `dst == src` take borrowed inputs and return by value, which is
what the C computes through its temporary in the aliased case. `PSMTXInverse` returns `Option`.
`HsdRand` holds random.c's statics; a stage seed redirect is `SeedSlot::Redirected(*mut u32)`
until stage state exists natively.

## Port round 1 (night 2): baselib math, lowest level first

mtx.c / quatlib.c / spline.c are the leaves under JObj (HSD_MtxSRT/SRTQuat build every joint
matrix; spline.c feeds JObj path animation). 31 C definitions ported; every fused site matches
`fma_sites.py` (mtx.c 118 f32 sites, quatlib 38, spline 45).

- Oracle: `oracle/build_hsd.sh` (same flags + `lower.py` as build.sh; links mtx_pc, gekko_fp,
  lbtrigf, wrap.c and `stubs_hsd.c` for sqrtf/fabsf/ObjAlloc) -> `tests/data/oracle_hsd.txt`
  (33,000 rows, generated inputs only), replayed by `tests/oracle_hsd.rs`: 66,000 checks, all
  bit-identical. `lower.py` now also lowers narrow `fptosi` (s16/s8: saturate to i32, then
  truncate), needed by splGetSplinePoint's `(s16)t`.
- Mutation check (unfuse one site, rerun): detected for every function except two sites:
  (1) the Simpson accumulation `var += 4.0F * f(t)` in splArcLengthGetParameter (30,000
  single-segment bisection rows still give the same result; the bisection only flips on rare
  comparisons) and (2) slerp's `1.0F - 2.0F*t` (2t is exact, so fused == unfused; harmless).
  (1) is right by the IR, not by a failing vector.
- API: matrices returned by value; `HSD_MtxInverse(m, m)` is a separate `inverse_in_place`
  because the C reads `src[r][3]` after writing `dest[0..r][3]` (different result from the
  non-aliased call, reproduced). Spline takes a borrowed `Spline<'_>` view (slices for cv,
  segLength, segPoly) instead of the raw `HSD_Spline` pointers. `None` where the C leaves the
  output untouched.
- Not ported in mtx.c: HSD_VecAlloc/Free, HSD_MtxAlloc/Free, the four `*AllocData` functions
  (HSD_ObjAlloc pool wrappers; they wait for objalloc.c / the object system).
- Not started this round: the object system itself (HSD_Obj/class.c, GObj, JObj, AObj, DObj).
  These are stateful (no pure oracle); they need a decision on native object ownership
  (arena + indices vs Box) before porting.

## Not ported

About 111 of the group's ~2,000 C function definitions are ported (~4%). Everything below is
untouched:

- `src/sysdolphin/baselib` (~1,200 functions, 48k lines): the HSD object system (JObj, DObj, MObj,
  TObj, AObj/FObj animation, CObj, LObj, particles, archives, memory/heap, GObj/process lists,
  card, debug console, psdisp, `hsd_397E.c`'s crash-screen register reads, which on PC are
  `return 0`). It needs the shared struct definitions and a decision on how big-endian
  DAT/archive data becomes native structs (relocation + per-field byte swap driven by the
  `GcLayout` tables, which are not generated yet).
- `src/melee/lb` apart from lbtrigf/lbvector (~520 functions), `src/melee/db` (81),
  `src/melee/sfx` (23), `libs/dolphin/src/dolphin/thp/THPDec.c` (45).
- No PowerPC `.s` file belongs to this group (inventory section 3); the only asm-adjacent item is
  `hsd_397E.c` (see above).

Raw c2rust output for all 125 of the group's TUs is in `rust/melee_runtime/src/raw/` (local only,
gitignored), from the toolchain run.

## Blockers for the rest of the group

1. Raw c2rust output was made with `-ffp-contract=off`: every float expression in it must be
   rechecked against the C IR for fusions. Regenerating with contraction on does not help:
   c2rust does not emit fmuladd. Hand-porting with the IR open is what worked here.
2. The `GcLayout` table generator (ppc32 record-layout dump -> `gc_layout!`) does not exist yet;
   the HSD structs are large and must not be hand-typed.
3. Shared types (`Vec3`, `Mtx`, HSD structs) need one home crate; `melee_types` currently
   does not compile (another agent's work in progress), so this crate no longer depends on it.
