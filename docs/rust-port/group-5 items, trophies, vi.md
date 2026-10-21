# Port group 5: items (it), trophies (ty), vi -> `rust/melee_items`

Status 2026-10-21 (first session). C: 199 TUs, ~88.8k lines, about 5,000 function definitions
(it 4,674, ty 221, vi 101; counted with a regex, so approximate). No `.s` files or inline asm
belong to this group (inventory.md section 3); the `asm` hits in item.c/toy.c/iteffect.c
are comments citing m-ex patches.

## What exists

- **Raw transpile**: all 199 TUs are in `rust/melee_items/src/raw/` (gitignored, regenerate with
  `bash tools/rust-port/transpile.sh src/melee/it src/melee/ty src/melee/vi`). That is the
  toolchain agent's full run; this session did not rerun it. Reference only, not compiled.
- **Native structs + GC offset record** (`src/gen/layout.rs`, generated):
  `tools/gen_layout.sh` includes every group-5 header in a probe TU, dumps clang's
  `--target=powerpc-none-eabi -fdump-record-layouts-complete` plus the typedef/record decls,
  and `tools/gen_layout.py` emits for 438 records (every record defined under it/ty/vi and
  everything they embed by value, e.g. Vec3, CommandInfo, ColorOverlay, HurtCapsule, ECBSource,
  HSD and ft records):
  - a native `#[repr(C)] #[derive(Clone, Copy)]` struct/union: little-endian, native pointers,
    C enums as `i32` (`-fno-short-enums`), every C function pointer as
    `FnPtr = Option<unsafe extern "C" fn()>` (signature in the doc comment; TODO typed);
  - `melee_core::gc_layout!` per record: each C field's GameCube offset/size/C type, bitfields
    with PPC MSB-first bit numbers (the m-ex "GC offset -> native field" map);
  - bitfields: one integer per storage unit (`bf_<gc offset>`) that holds the bits **in GC
    numbering**, so a union's byte view (`UnkFlagStruct.byte`) equals the GC byte; getters,
    `set_*`, and `*_signed` for signed fields. C anonymous members become `anon_<off>`;
    C field `_` becomes `unk`; keywords get a `_` suffix; `Vec` is renamed `Vec3`.
  - pointers to records not generated here are opaque `[u8; 0]` markers (20).
  - Self-check: the generator recomputes every layout by ppc32 rules and fails on any
    disagreement with clang. Result: 0 mismatches. sizeof(Item) on GC = 0xFD0.
  - `Item::zeroed()` etc.: all-zero is a valid value for every generated record.
- **Neutral hash, items** (`src/hash.rs`): `mix` (= `nh_Mix`), `item_word`, `ItemSum`
  (count + wrapping sum, `mix_into` for the game word) and `StateHash for Item` (NHV1 field set).
  `Item` is the only group-5 struct in the neutral hash field list.
- **Ported functions** (`src/item.rs`, 9): `Item_80269CA0`, `checkHitLag`, `it_802762B0`,
  `it_802762BC`, `it_80274F28`, `decelerateItemX`, `itSamusMissile_ClampTurn`, `norm_xF74`,
  `itNesspkthunderball_ShiftAngles`. Chosen because they touch only `Item` fields.
- **Tests** (`tests/oracle.rs`, 8, pass): item_word over 64 vectors (incl. -0.0), damage clamp
  (incl. s32 wrap), decelerateItemX (48), ClampTurn (32), norm_xF74 (32, f64 `- M_PI` then f32)
  all bit-compared with `tests/c_oracle/expected.rs`, which `tests/c_oracle/gen_expected.sh`
  produces by compiling `oracle.c` natively with clang (x86_64, `-ffp-contract=off`) against
  the real headers, with the C bodies copied verbatim. Plus GC offsets of the hash fields vs
  neutral-hash.md, and bitfield numbering.

`cargo build -p melee_items` and `cargo test -p melee_items` pass with no warnings.

## Decisions / traps

- `Cargo.toml` no longer depends on melee_types/runtime/world/fighter: nothing calls them yet,
  and their crates were mid-generation by other agents (melee_types and melee_fighter did not
  build when checked), which broke this crate's build. Re-add when code calls into them.
- melee_types and melee_fighter also generate Vec3/HSD/ft records. The copies here are
  TODO(move-to-core): when group 0's tables settle, delete ours and `use` theirs (the field
  names may differ; check `Anon_*` naming).
- `Anon_<file>_<line>_<col>` names come from the C source position: they change when a header
  is edited. Regenerate rather than hand-edit.
- The raw c2rust output uses x86_64 layouts; never take offsets from it. GC offsets come only
  from the ppc32 dump.

## Not done / blockers

- About 0.2% of the group's functions (9 of ~5,000) are ported. The rest is coupled to HSD_GObj
  / HSD_JObj (group 1), the ground/collision code (group 2) and Fighter (group 3): 148 ft TUs
  include it and 74 it TUs include ft. Porting item.c's spawn/think/collision needs those
  crates' native APIs first, or a shared `extern "C"`-style call table agreed across groups.
- Item state tables (`ItemStateTable`, per-kind `it_803F...[]` arrays) are function-pointer
  tables; they wait for typed callbacks.
- DAT-loaded data (ItemAttr, ToyTable, article data) needs the big-endian loader that the
  `GcLayout` tables are meant to drive; not written.
- No frame driver, so the item columns of the goldens (`items`, `itemsum`) are not compared
  yet; only the per-item word is checked against C.
