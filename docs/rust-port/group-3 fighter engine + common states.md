# Group 3: fighter engine + common states (`rust/melee_fighter`)

Scope: `src/melee/ft/*.c` (67 TUs), `src/melee/ft/kinds/ftCommon/*.c` (142), `src/melee/pl/*.c`
(7): 216 TUs, ~76k lines, about 2,820 function definitions (regex count over the C).
State as of 2026-10-21.

## What exists

| layer | where | status |
|---|---|---|
| Native types + GC offsets | `src/types/generated.rs` (from `gen/gen_types.py`) | 569 `#[repr(C)]` records, all built |
| Mechanical translation of all 216 TUs | `src/c2r/` (from `gen/fix_raw.py`, feature `c2r`) | compiles (`--features c2r`), never run |
| Hand ports, checked bit-for-bit against the C | `src/neutral_hash.rs`, `src/ftcommon.rs`, `src/stale.rs`, `src/compat.rs`, `src/hooks.rs` | 57 C functions, tests pass |
| PowerPC `.s` files | none belong to this group (inventory §3) | nothing to do |

Functions: 57 of ~2,820 (about 2%) are hand-ported and verified against the C. All ~2,820 are
in the mechanical translation, which compiles; none of it has been linked or run.

### Native types (`gen/gen_types.py`)
- libclang (`C:\Program Files\LLVM\bin\libclang.dll`, Python `clang` bindings) parses
  `gen/types_tu.c` (every ft/, ftCommon/ and pl/ header) for `powerpc-unknown-eabi` with
  `-DTARGET_PC`, so offsets are the GameCube ones.
- It writes one `#[repr(C)]` struct or union per record: 254 defined in group-3 headers, and the
  rest embedded by value (Vec3, HSD_*, CollData...), marked `TODO(move-to-core)`. Records that
  are only pointed to become opaque `{ _opaque: [u8; 0] }`.
- Each record implements `melee_core::gc::GcLayout`: the GameCube size, alignment, and per field
  the offset, size and C type. Bitfields carry their MSB-first bit number (that per-struct record
  is the m-ex "GC offset -> native field" map).
- Bitfield runs are stored as `[u8; N]`, in the GameCube byte order. The accessors are
  `x()` / `set_x()`, MSB-first, sign-extended for signed C types, with the same names
  c2rust_bitfields generates.
- Anonymous members are `anonN` (`Fighter::anon1`...). Typedef names are aliases
  (`StaticPlayer = _StaticPlayer`, `Fighter_GObj = HSD_GObj`). Enums become `pub type E = i32/u32`
  plus one `pub const` per enumerator.
- Check: `Fighter` is 0x23EC on the GC. 274 of its fields were compared with the `/* fp+XXX */`
  comments in ft/types.h. All matched except 4, and those 4 are typos in the comments.
- Regenerate: `python rust/melee_fighter/gen/gen_types.py` (from the worktree root).

### Mechanical translation (`gen/fix_raw.py`, feature `c2r`)
- Input is the raw c2rust output (`src/raw/`, gitignored): `bash tools/rust-port/transpile.sh
  src/melee/ft src/melee/pl` (the ft dir includes ft/kinds/*; only ftCommon is used here).
- Output: `src/c2r/{ft,ftcommon,pl}/*.rs`, ~1.5M lines and 50 MB, which is why it is
  gitignored. Every TU keeps its own copy of the header types. Build with
  `cargo build -p melee_fighter --features c2r`, about 3 minutes.
- It is already native: c2rust read the C for x86_64 with `-DTARGET_PC`. It has no guest memory,
  no byte swaps and no `gw_r32`.
- Fixes `fix_raw.py` applies:
  - extern types become opaque structs;
  - C members and locals named `_` become `_pad` / `_unnamed_N`;
  - `f128` becomes `f64`;
  - gnu89 header `inline` functions (math_ppc.h `sqrtf`...) become module-local;
  - hook calls that c2rust left without a declaration get one (it declares a block-scope
    `extern` only inside its own block, so `Mex_Enabled_0` and others were undeclared);
  - the HitCapsule owner/flag union (see Traps).
- This layer is a scaffold. It still calls ~everything through `extern "C"` symbols, which other
  groups have to provide before a binary links. Its structs are c2rust's own per-TU copies, not
  `crate::types`. Switching it to `crate::types` is mechanical, because field and bitfield
  accessor names are the same. Anonymous members are the one difference: c2rust names them
  `c2rust_unnamed*`, the generator names them `anonN`.

### Hand ports (verified)
| module | C functions | test | cases |
|---|---|---|---|
| `neutral_hash.rs` | `nh_Mix`, `nh_CpuFlags`, `nh_Fighter` (fighter.c, TARGET_PC) + item/game word, `StateHash for Fighter` | `tests/nh_ref.rs` | 64 fighters, all fields + every CPU flag bitfield |
| `ftcommon.rs` | 33 functions of ftcommon.c (ground/air accel, drift, friction, KB friction, fall/ascend, CheckFallFast, facing, CalcHitlag) | `tests/ftcommon_ref.rs` | 6,600 (33 x 200) |
| `stale.rs` | plstale.c (5), plattack.c (3), ft_0881.c staling (5), player.c accessors (8) | `tests/stale_ref.rs` | 300 sequences x 40 ops |
| `compat.rs` | HitCapsule `hit_grabbed_victim_only` GC semantics | unit test | - |

How the C reference works: `gen/cref/run.py` compiles each `gen/cref/<name>_ref.c` natively
with clang (the same flags as `tools/rust-port/gen_cc.py`, plus `pc/gameworld/gekko_fp.c` for
`__frsqrte`) against the real headers. The game code under test is either `#include`d whole
(plstale.c, plattack.c) or cut verbatim out of the game's .c by start/stop markers (never
retyped). The programs print their outputs to `tests/data/<name>.txt`. The Rust test repeats the
same xorshift-driven inputs and compares every line.
Regenerate: `python rust/melee_fighter/gen/cref/run.py [name...]`.

`hooks.rs` holds the calls out of the group: `FT_SCRIPT_VALUE`/`ScriptGame_ModValue`,
`Geno_FallLimit`, `ft_PlaySFX`, `ft_GetGroundFrictionMultiplier`, and `p_ftCommonData`. Each is
a settable fn pointer whose default is vanilla behaviour.

## Commands
```
python rust/melee_fighter/gen/gen_types.py          # types (needs only LLVM + python clang)
python rust/melee_fighter/gen/cref/run.py           # C expected values (Windows, clang + MSVC link)
cd rust && cargo build -p melee_fighter && cargo test -p melee_fighter
# mechanical layer (needs WSL c2rust, see RUST_PORT.md "Translation toolchain"):
bash tools/rust-port/transpile.sh src/melee/ft src/melee/pl
python rust/melee_fighter/gen/fix_raw.py
cd rust && cargo build -p melee_fighter --features c2r
```

## Traps found (behaviour a plain native port would change)
1. **`HitCapsule::hit_grabbed_victim_only`** (lb/types.h +0x134) is a `u8 :1` that shares a union
   with `HSD_GObj* owner`.
   - On the GameCube the flag *is* the MSB of the owner's address, so any heap pointer
     (0x80xxxxxx) reads as 1. Natively that bit is 0.
   - c2rust also turns the `:1` into a whole u8.
   - `compat::gc_ptr_msb_flag{,_set}` stores the GC MSB in the native word: bit 63 means
     "cleared", and setting the flag on NULL gives 1. The c2r modules use those accessors. It is
     read in ftcoll.c:1896 and written in ftaction.c:411 and itsamusgrapple.c:228 (group 5).
2. **`ABS` is not `f32::abs`.** The C macro is `x < 0 ? -x : x`, so `ABS(-0.0)` is `-0.0`. The
   ports use `ftcommon::abs_c`.
3. **Float to int.** `(int)f` on PPC (`fctiwz`) saturates, as Rust `as` does. They differ only for
   NaN (PPC gives 0x80000000, Rust gives 0). This is noted in `ftcommon::trunc_i32`.
4. **`CpuFighter::csP` points into its own `buffer`.** Copying a native `Fighter` by value leaves
   it pointing at the old copy. The neutral hash uses the offset (`cpu_script_offset`), never
   the pointer. The real port should store an index.
5. **Fused multiply-add.** The C build (the oracle) uses `-ffp-contract=off`, so the ports never
   fuse `a*b+c`. The original MWCC GameCube code may fuse. Matching the C build is the target.
6. **Sibling crates did not build.** `melee_types`/`melee_runtime`/`melee_world` did not build
   while this was done (other agents generating them). `Cargo.toml` has them commented out, and
   the shared types this group needs are generated here with `TODO(move-to-core)`.

## Not done / blockers
- **The c2r layer has never linked or run.** Everything outside group 3 is an unresolved
  `extern "C"` (HSD, lb, mp, it, gm, PC hooks). It cannot be checked against the goldens until
  the other groups export those symbols and a frame driver exists.
- **No golden comparison yet.** `neutral_hash` matches the C function on synthetic fighters, but
  no Rust match has run.
- **Not verified:** `atan2f`/`sqrtf` users (`ftCo_GetLStickAngle`...). Their bit-exactness
  depends on group 1's MSL/math_ppc port (`__frsqrte`). They are left to the mechanical layer.
- **Next steps:** switch c2r modules onto `crate::types` TU by TU, then replace them with hand
  ports, verified the same way (cref harness first). The best candidates are the rest of
  ftcommon.c, ftlib.c and player.c, which are mostly field logic.
