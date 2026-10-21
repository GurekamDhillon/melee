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

## Round 2 (night 2): 103 Item accessors, `src/itlib.rs`

The `Item`-only getters, setters and small helpers of five TUs, all callees-free (a regex scan
for bodies that call nothing, then read by hand):

- it_26B1.c (49): `it_8026B294`, `itIsHeavy`, `it_8026B2D8`, `itGetKind`, `it_8026B30C`,
  `itGetHoldKind`, `itGetDamageMultiplier`, `it_8026B344`, `itGetGrabRangeX/Y`, `it_8026B390`,
  `it_8026B3A8`, `it_8026B424`, `it_8026B47C`, `it_8026B4F0`, `it_8026B54C/560/574`,
  `it_8026B588`, `it_8026B594`, `it_8026B6C8`, `it_8026B718`, `it_8026B724`, `it_8026B73C`,
  `it_8026B774`, `itGetMotionId`, `itGetTeamId`, `it_8026B7BC/7CC/7E8`, `it_8026B894`,
  `it_8026B924`, `it_8026B960`, `it_8026BB88`, `it_8026BBCC`, `it_8026BC68`, `itGetAttackId`,
  `it_8026BCF4/D0C/D24/D3C/D54/D6C/D84/D9C`, `it_8026BDCC`, `it_8026BE28`, `it_8026C1E8`,
  `it_8026C334`
- it_2725.c (24): `it_80273030`, `it_80273130`, `it_8027346C/484/49C`, `it_802734B4`,
  `it_80274C60/C78/C88`, `it_80274D6C`, `it_80274DAC`, `it_80274DFC`, `it_80274E44`,
  `it_80274ECC`, `it_80274EF8`, `it_80274F10`, `it_802750E8`, `it_802751D8`,
  `it_80275210/228/240/258/270`, `it_802753BC`
- ithitbox.c (19): `it_802753DC` .. `it_802758D4` (hitbox flags, scale, state, max damage)
- itmaplib.c (5): `it_802762D8`, `it_80276308`, `it_80276348`, `it_802763B8`, `it_80276408`
- itcoll.c (6): `it_802714C0`, `it_80271508`, `it_80271534`, `it_80271590`, `it_80272280`,
  `it_80272298`

Fused sites (`fma_sites.py` on each TU): `it_8026B344` 1 and `it_8026B424` 1 (f32); every other
function 0. The oracle's lowering reports 10 fmuladd.f32 (8 from round 1 + these 2).

**Verification** (`tests/oracle_round2.rs`, 104 tables x 48 cases, all pass): `round2_bodies.h`
holds the C bodies extracted unchanged from the decomp files; `round2.c` calls them through a
real `HSD_GObj` whose `user_data` is the Item. Both sides fill 189 Item fields (plus the
ItemAttr / Article / itUnkAttributes / ItemCommonData the functions follow) from the same LCG
(`c_oracle/gen_round2_fields.py` emits the C and Rust fill/snapshot code from one list), run the
function, and compare a hash of all 189 fields + return value + an output Vec3. Bitfield units
are compared as their GameCube big-endian value, built field by field in C and read from the
Rust `bf_*` bytes, so this also checks melee_types' bit numbering for flag32, UnkFlagStruct,
CameraBoxFlags, the `xDCC` flags, HitCapsule +0x40 and HurtCapsule +0x24. Inputs never contain
inf/NaN; pointers are drawn from {NULL, gobj A, gobj B} and hashed as an index.
Mutation checks (each made the test fail): un-fusing `it_8026B344` (only after adding a
moderate-magnitude input set; random exponents alone never hit it) and `it_8026B424`, wrong flag
bit, `<` for `<=` in `it_802758D4` (only the signed-zero table hits it), bool-normalising
`it_802750E8` / `it_80274ECC`, wrong `xCB0_source_ply`, `x0_78` vs `x0_hold_kind`.

Traps found:
- `bool` is `int` in the shipping build (`-isystem src/MSL`, `src/MSL/stdbool.h`), but `_Bool`
  in this oracle (clang's stdbool). `it_802750E8` returns `xDC0 & mask` unnormalised and
  `it_80274ECC` stores an int argument to a u8, so the ports return/take `i32` and `round2.c`
  compiles its copies with `#define bool int`. Round 1's oracle copies still get `_Bool` (none of
  them depends on it).
- C `bool` return values that are `true`/`false` literals are ported as Rust `bool`.

Signatures: the GObj becomes `&Item`/`&mut Item`; a followed C pointer (`xCC_item_attr`,
`xC4_article_data`) makes the function `unsafe` (documented precondition); the `it_804D6D28`
global becomes `&ItemCommonData`; an out-of-range hitbox/hurtbox index panics where the C would
write past the array. No other-group stubs were needed. Not done: no caller uses these yet, and
`it_80272818`/`it_80272D1C` (Fighter / GObj classifier) were left out.

## Round 3 (night 3): 30 next-layer functions, `src/itlib3.rs`

Chosen by a scan of it/ty/vi for bodies whose every call is to an already-ported function (or
none), then read by hand, library TUs first:

- it_2725.c (11): `it_80272818`, `it_80272828`, `it_80272860`, `it_80272D1C`,
  `itColl_BounceOffVictim`, `it_80273500`, `it_80274658`, `it_80274D04`, `it_80275174`,
  `it_80275328`, `it_80275390`
- it_26B1.c (3): `it_8026B3C0`, `it_8026C3FC`, `it_8026C42C` (item p-link list walks)
- it_279C.c (4): `it_8027A13C`, `it_8027A344`, `it_8027A364`, `it_8027AAA0` (pokemon spawn)
- itanimlist.c (2): `it_802795EC`, `it_802796FC` (item script commands)
- itcoll.c (1): `it_80270CD8` (knockback); item.c (4): `Item_80268E40`, `Item_80269CC4`,
  `Item_8026AE60`, `Item_IsGrabbable`; ithitbox.c: `it_802756E0`; itmaplib.c: `it_80276CEC`;
  itspawn.c: `it_8026CA4C`; itzako.c: `it_8027B730`, `it_8027CBFC`

Fused sites (`fma_sites.py`): `itColl_BounceOffVictim` 1, `it_80270CD8` 6 (clang turns
`a*b + c*d` into `fmuladd(a, b, c*d)`), `it_8026CA4C` 1; all f32 (`fmadds`). Every other one 0.

**Verification** (`tests/oracle_round3.rs`, 32 tables x 48 cases, pass): `round3_bodies.h` is
extracted unchanged from the decomp; `round3.c` reuses round 2's LCG / 189-field fill+snap and
additionally hashes every field a function touches outside that list (pokemon/zako vars, the
`Item_80269CC4` reset set, `xC2C`, `xDAC`, spin speed, ItemLink, CommandInfo advance, side items of
a 0..3-long GObj list). Globals are redirected to statics in C and are parameters in Rust.
Script commands are written through the C bitfields on one side and the generated GC-order
setters on the other, so this also checks `set_hitbox_scale` / `set_throw_flags` bit numbering.
Mutation checks (each made the test fail): un-fusing each of the 8 sites. Two of them
(`it_80270CD8`'s no-`x28` inner site, `it_8026CA4C`) were only hit after adding targeted tables
231/232 (small `x80_float[9]`; `arg4 = (n - 0.99) / w` +-1 ulp). Swapped flag bit, dropped case
label, wrong constant also fail. `fmadds` vs a single-rounding f32 fma is not distinguished.

Decisions: `(s32) f` ports as Rust `as i32` (gwtool lowers `fptosi` to saturating
`fptosi.sat`, NaN -> 0, same semantics). `it_80276CEC`'s uninitialised `coll_index` is 0
(`-ftrivial-auto-var-init=zero`). `it_8027AAA0` takes the spawner and the spawned `Item`
separately. The p-link head (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]`) is a parameter:
TODO(group1) take it from the native GObj runtime. Script pointers advance by
`size_of::<CmdUnion>()` (native), not 4 GC bytes: correct for a natively decoded script only.
Left out: `it_8027B070` (Fighter `Struct2070` bitfield copy), the per-kind state callbacks
(~680 one-liners and ~250 small bodies on kind-specific vars; next round), `it_ResetStageArticleTable`.
No other-group stubs were needed. Not done: no caller uses these yet; no frame-level comparison.

## Round 4 (night 3): 248 per-kind state callbacks, `src/itkinds.rs`

Chosen by a scan of `src/melee/it/kinds/*.c` (112 TUs) for bodies of 4-40 lines whose every
call is to an already-ported function, that touch no GObj/JObj, fighter, effect, HSD or global
data, then read by hand. 138 of them share five one-statement shapes and are emitted from
templates: fall (`it_80272860` with the item's `x10_fall_speed`/`x14_fall_speed_max`, 86), fall
while airborne (10), `itColl_BounceOffVictim; return false` (23), `vel.y -= fall_speed` (10,
airborne-only 6), `it_80273030; return false` (3). The other 110 are hand ported: life-timer
ticks, item-var copies/counters, special-attribute physics (Kinoko, Dosei, Leadead, Freeze,
Freezer, Houou, Kireihana, Kamex, Linkbomb, Samus missile, Sonans, Gshell, Flipper, Chicorita,
it_2E5A, Lugia, Matadogas, ...), reflect/bounce variants, owner/msid getters.

Fused sites (`fma_sites.py` on all 112 TUs, every ported function checked): 1 f32 site each in
`itHitodeman_UnkMotion1_Phys` (`vel.x = -fmadds(-facing_dir, x4C, -vel.x)`; clang folds the
outer subtraction), `itKamex_UnkMotion1_Phys`, `itLinkbomb_UnkMotion4_Phys`,
`itSamusmissile_UnkMotion1_Phys`; every other one 0.

**Verification** (`tests/oracle_round4.rs`, 249 tables x 48 cases, pass): `round4_bodies.h` is
extracted unchanged from the decomp; `round4.c` reuses round 2's LCG / 189-field fill+snap and
adds `r4_pre`: ground_or_air (0..2), the four item-command vars, anim frame (1/4 exactly 16.0) and
speed, xCCC, xCA8, the ItemAttr fall speeds, the ItemCommonData floats the callees read, a
256-word special-attributes block, and every 32-bit word of the 504-byte item-var union (zero,
0..3, or a float). The hash adds all of those plus every item-var word after the call, so any
write anywhere in the union is compared. `sizeof(Item_ItemVars)` is checked equal (504) on both
sides. Item-var bitfields (`gshell.xDEC_*`, `linkbomb.x0`, `it_2E5A.x18`) are LSB-first in the x86
C and GC-numbered in melee_types, so the Rust side bit-reverses that byte around the call.
Mutation checks (each made the test fail): un-fusing each of the 4 sites (Samus missile only
after adding targeted table 600: moderate inputs, clamp disabled; the random table clamps every
case where fusing matters), dropping the assignment-as-condition in
`itNesspkfirepillar_UnkMotion0_Phys`, wrong bit in the gshell and linkbomb flags.

Trap found: round1.c's `it_80275158` copy takes the Item (its `GET_ITEM` is a cast), so
`round4_bodies.h` adds a verbatim GObj-taking copy under `#define it_80275158 it_80275158_g` for
`itHammerHead_Logic40_Spawned`. `it_802762BC` (ported in round 0) was missing from the oracle TU
and is copied there too.

Signatures: as round 3; functions that read `xCC_item_attr` or the special attributes are
`unsafe`. `itGamewatchchef_UnkMotion0_Phys` indexes the C's `entries[1]` past its declared
length, ported as pointer arithmetic. `itGShell_Attrs` (file-local) is a native struct with its
own `gc_layout!` (0x40). `it_8028D4E4` reuses round 1's `itRshell_ClampVel` (same body). No
other-group stubs were needed. Left out: kind callbacks that touch GObj/JObj, fighters, effects,
random numbers or other items (e.g. `it_802E79C8`, item-to-item links); ~680 bodies that are only
`return false;`/empty were not counted or ported. Not done: no frame-level comparison; the state
tables that would call these are still untyped function pointers.

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
