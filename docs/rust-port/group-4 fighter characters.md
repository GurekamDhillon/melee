# Port group 4: fighter characters (`src/melee/ft/kinds/*` except `ftCommon`)

Crate: `rust/melee_chara`. Status as of 2026-10-21 (first night).

## Scope
- 232 TUs, 79,558 C lines, about 4,340 function definitions (regex count over the .c files).
- No PowerPC `.s` or inline-asm files belong to this group (checked: none under `ft/kinds`).
- Every TU is in `docs/rust-port/game_tus.txt` under `src/melee/ft/kinds/` (the inventory's
  `ft/chara` is this directory).

## What was done
1. **Transpile:** `bash tools/rust-port/transpile.sh --crate melee_chara -j 12 src/melee/ft/kinds`
   -> 374 ok, 0 failed (the 142 ftCommon TUs come along with the directory; their output was
   deleted from `melee_chara/src/raw/`, they are group 3's). Raw output is gitignored; rerun the
   command to get it back.
2. **Native types (generated):** `rust/melee_chara/gen/run_gen.sh` (needs clang 23 + Python 3):
   - ppc32 `-fdump-record-layouts-complete` without `TARGET_PC` -> original GameCube offsets.
   - native x86_64 `-ast-dump=json` with `TARGET_PC` -> the field set and C types.
   - `gen/gen_types.py` -> `src/types_gen.rs`: a `#[repr(C)] #[derive(Clone, Copy)]` struct or
     union per record declared in `ft/kinds/*/types.h` (172: each character's `*_DatAttrs`,
     `*_FighterVars`, `*_MotionVars` and the per-state structs), plus `melee_core::gc_layout!`
     with every field's GC offset/size/C type (bitfields with PPC MSB-first bit numbers).
   - Native rules: little-endian, 8-byte pointers, C bitfields become one integer field each
     (the GC bit position lives only in the table), MSL `bool` (int) -> `i32`, enums -> `i32`,
     function pointers -> `FnPtr = Option<unsafe extern "C" fn()>` (untyped for now).
   - Records owned by other groups: 9 used by value (`Vec` = Vec3, `Vec2`, `_GXColor`,
     `ReflectDesc`, `AbsorbDesc`, `DObjList`, `FtPartsVis`, `CostumeTObjList`, `ftCollisionBox`)
     are generated in full at the top of the file; 8 used only behind pointers (`HSD_GObj`,
     `HSD_JObj`, `HSD_DObj`, `HSD_TObj`, `HSD_Joint`, `HSD_AnimJoint`, `HSD_MatAnimJoint`,
     `FtPartsVisLookup`) are opaque `[u8; 0]` placeholders. All marked TODO(move-to-core): switch to melee_types /
     melee_fighter once they build.
   - `StateHash` is generated for the 121 plain structs (no pointers, no unions inside), field
     values in declaration order. None of these records is in the neutral-hash field list
     (`docs/rust-port/neutral-hash.md` hashes only Fighter/CpuFighter/Item fields), so nothing
     in the golden hash depends on them yet.
3. **Pure functions (hand port, `src/pure.rs`):** `frsqrte` (local copy of gekko_fp.c's
   `__frsqrte`, TODO(move-to-core): group 1 owns it), `my_sqrtf` (one function for the 7 identical
   static-inline copies in ftCrazyHand/ftMasterHand/ftSeak), `sumOfSquares` (ftSeak),
   `ftFox_AppealS_GetLR`, ftNess `returnStateVar`, ftYoshi `perm_neg_inline`/`perm_mul_inline`.
4. **Tests:** `cargo test -p melee_chara`: 5 tests, all pass.
   - `frsqrte` and `my_sqrtf` bit-equal to the C on 220 inputs (zero, -0, denormals, max, inf,
     NaN, negatives, 200 spread normals); `sumOfSquares` on 64 pairs. Expected values come from
     `gen/ref_pure.c` compiled natively (`clang -O2 -ffp-contract=off`, x86-64 baseline, no FMA)
     by `gen/run_ref.sh` into `src/pure_ref.rs`.
   - Layout: Fox records checked against the offsets in the header comments; every generated
     table is in bounds of its GC sizeof.

## Percentage ported
- Functions: about 13 of ~4,340 (0.3%), counting the 7 `my_sqrtf` copies. The other ~99.7% are
  **not ported**: no stubs were written for them either; they exist only as raw c2rust output.
- Types: 172/172 records of the group's `types.h` files (100%) generated and building.

## What builds
`cargo build -p melee_chara` and `cargo test -p melee_chara` (stable 1.96, no warnings).

## Blockers
1. **Fighter / ft core types are group 3's and did not build tonight.** Nearly every character
   function takes `HSD_GObj*` and reads `Fighter` (`fp->mv.fx.SpecialS.gravityDelay`,
   `fp->dat_attrs`, `ftCommon_*`, `ftAnim_*`, `ftColl_*`, `ef*`, `it*`). Until `melee_fighter`
   exports a native `Fighter` and the ft-core functions, the bodies can't be ported in a way that
   compiles. When this was written, `melee_types/src/gen.rs` and
   `melee_fighter/src/types/generated.rs` (another agent's, still being written) failed to
   compile (a `fn type` keyword, an empty union, `-0x1` offsets, missing `FigaTree` etc.), so
   `melee_chara` depends on `melee_core` only. Put the deps back in `rust/melee_chara/Cargo.toml`
   when they build.
2. **Duplicate type generators.** Groups 0 and 3 are generating the full type set as well. Once
   one generator wins, `types_gen.rs` should keep only the `ft/kinds` records and import the rest.
   The native rules above (bitfield expansion, bool -> i32, enum -> i32) need to match across
   generators, or the `Fighter.mv` union will not line up with these records.
3. **Function pointers are untyped** (`FnPtr`). The `MotionState` tables (one per character)
   carry anim/IASA/phys/coll callbacks and need typed `fn(*mut HSD_GObj)` once `HSD_GObj` is real.
4. **FMA question for the oracle (affects every group):** the C build (`_build/masstest/pipe_win.sh`)
   emits PPC LLVM IR with clang's default `-ffp-contract=on` (no `-ffp-contract=off`), so
   `a*b+c` in C becomes `llvm.fmuladd`. Whether that fuses depends on gwtool's x86 backend target
   features: with no FMA feature it lowers to separate mul+add, which is what the Rust port and
   these tests assume. Not checked tonight. Someone should confirm in the gwtool target setup
   (or the exe disassembly) that no `vfmadd` is emitted for game TUs.

## Next steps
- Once `Fighter` exists: port per character, starting with the smallest TUs (`ftSandbag`,
  `ftZakoBoy`/`ftZakoGirl`, `ftGigaKoopa`), then the clones that share code with others
  (Falco/Fox, Young Link/Link, Pichu/Pikachu, Dr. Mario/Mario, Ganon/Captain), working through
  `src/raw/` file by file and checking each against the goldens.
- Typed function pointers, then the `MotionState` tables.
- Move `frsqrte` to melee_runtime and the external by-value records to melee_types.
