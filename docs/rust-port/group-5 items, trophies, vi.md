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

## Round 1 (night 2): 29 leaf functions, `src/helpers.rs`

Chosen by scanning it/ty/vi for function bodies that call nothing (a regex pass, so a candidate
list, not a call graph) and taking the ones that are pure math or touch only `Item` /
`ItemLink` fields. Ported, each verified against its verbatim C copy in
`tests/c_oracle/round1.c` (compiled by `gen_expected.sh`: x86_64, `-ffp-contract=on` + group 1's
`lower.py`, i.e. the shipping build's contraction and gwtool lowering):

- vector: `product_xy` (itmaplib), `product_xyz` (itzako), `neg_dot` (itleadead),
  `_tyDisplay_80318CB4_calc_dist_sq`, `it_link_lerp` (itlinkhookshot)
- angles: `it_8027B798_CalcAngle` (itzako), `clamp_tau`, `clamp_pi_tau`, `clamp_angle_pi`,
  `norm_xF74_from_angle` (itlinkboomerang), `itDosei_FacingAngle`
- scalar: `itLinkboomerang_UnkMotion3_Phys_add_clamp`, `fsign_inline`, `float_sign_int_inline`
  (itlinkbomb), `inline_fabsf` (ittaru), `lt_zero` (itnokonoko), `getX` (itoctarockstone),
  `pika_scale` (itpikachuthunder), `tyFigupon_StoreDigits`
- items: `itColl_chkECBOverlap`, `it_802763E0` (ceiling flag), `decrease_lifetimer`
  (itpeachturnip), `it_80275158` (lifetime; the global `it_804D6D28` is a parameter),
  `itRshell_ClampVel`, `itRshell_UM5_Accel`, `itRshell_UM5_AddVelAndCheck`,
  `itRshell_UM5C_Reverse`, `itSeakChain_clamp_x10`, `itSeakChain_clamp_x14`
- plus the file-local `itRShell_Attrs` as a native struct with its own `gc_layout!` (0x58).

Fused sites (`tools/rust-port/fma_sites.py` on each TU): product_xy 1, product_xyz 2, neg_dot 2
(clang folds the negations: `fmadds(-ax, bx, -(ay*by))`), calc_dist_sq 1, it_link_lerp 1,
CalcAngle 1 (f32); itDosei_FacingAngle 1 f64 (true fma). All others 0. The oracle lowered
exactly these 8 f32 sites. Mutation check: un-fusing product_xy/xyz, neg_dot, dist_sq or lerp makes
the tests fail; un-fusing **CalcAngle or FacingAngle does not** (no vector hits a case where it
matters), so those two are correct by the compiled IR only.

Inputs: 96 rows per table (32/40 for the integer ones) from a pool of edge values (signed zeros,
+-1, pi/tau neighbours, inf, NaN, denormal) plus random bits and scaled randoms. NaN results
are compared as "both NaN", not by bits: the sign/payload of an x86 NaN depends on operand order
in the instruction each compiler picks (it showed up in neg_dot and UM5_Accel) and is not
modelled as game state. Every non-NaN result is bit-compared.

Signature changes vs the C: an `Item_GObj*` used only through `GET_ITEM` becomes `&mut Item`;
`ip->xC4_article_data->x4_specialAttributes` becomes an `&itRShell_Attrs` parameter; the
`it_804D6D28` global becomes `&ItemCommonData`. No other-group stubs were needed this round.
`cargo test -p melee_items`: 18 tests pass. Not done: nothing above calls these yet (their C
callers need GObj/JObj/collision), so no frame-level comparison.

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
