# melee_types: types, GC layout table, the one generator

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 99-112 -->
## Group 0: melee_types (types + GC layout table)

Details: `docs/rust-port/group-0 types + GC layout table.md`.
- Types are GENERATED, not transpiled: `python rust/melee_types/gen/gen_types.py` (Windows Python
  + `pip install --user libclang==18.1.1`) parses every group-0 header with libclang as
  ppc32-none-eabi -DTARGET_PC (GC offsets, cross-checked against -fdump-record-layouts-complete)
  and as x86_64-msvc (native size asserts). Output `rust/melee_types/src/gen.rs` is committed.
- Every record implements `melee_core::gc::GcLayout`; `melee_types::GC_RECORDS` lists them all.
- Bitfields are `bf_<gcoff>: [u8; n]` in GC byte order with `name()`/`set_name()` accessors.
- Enums are int aliases + consts; `long`=i32, `bool`(MSL int)=i32, `size_t`=usize.
- Trap: the C typedef `Vec` shadows `std::vec::Vec` under `use melee_types::*`.
- `melee_types::neutral_hash` computes NHV1 from native structs; tested against the C functions
  compiled natively (gen/nh_oracle.py) and against all 10804 golden rows.


<!-- RUST_PORT.md lines 205-224 -->
## Step 1: one type generator (done)
- `rust/melee_types/gen/gen_types.py` now writes its probe `gen/all_types.c` itself (`write_probe`):
  every baselib header, every `melee/*/forward.h` + `types.h`, all of `ft/*.h`, `ft/kinds/**`,
  `pl/*.h`, `it/**`, `ty/**`, `vi/**` (799 headers) -> 1618 records, 0 missing from the
  `-fdump-record-layouts-complete` cross-check. Every record also gets `X::zeroed()`.
- Deleted: `melee_fighter/gen/gen_types.py` + `types_tu.c` + `src/types/generated.rs`,
  `melee_chara/gen/gen_types.py` + `run_gen.sh` + `src/types_gen.rs`, `melee_items/tools/gen_layout.*`
  + `src/gen/`. Old paths (`melee_fighter::types`, `melee_chara::types_gen`, `melee_items::layout`)
  are `pub use melee_types::*` shims. Every record those three generated exists in melee_types
  (checked by name; only anonymous-member record names differ, e.g. `plActionStats.anon_358`).
- NHV1 is one implementation (`melee_types::neutral_hash`); `melee_fighter::neutral_hash` and
  `melee_items::hash` are thin wrappers, so group 3's `nh_ref` and group 5's oracle tests now check
  that one copy against the C. The fighter/items StateHash impls were dropped (melee_types has them).
- Changes in convention for callers: bitfields are `bf_<gcoff>: [u8; n]` in GC byte order (items'
  old one-integer form is gone), enums are int aliases (`GroundOrAir` = u32), fn pointers typed.
- Group 4's generator laid out GC offsets WITHOUT `-DTARGET_PC`; melee_types uses `-DTARGET_PC` for
  both parses. No `TARGET_PC` appears in `ft/kinds/*/types.h`, so character records are unaffected.
- Not done: the gitignored `melee_fighter` `c2r` feature (fix_raw.py output) still carries per-TU
  type copies; it was not rebuilt.
