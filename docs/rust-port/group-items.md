# Group 5 (items `it`, trophies `ty`, vi): port rounds

Crate `rust/melee_items`. Rounds 1-11 (night 1 to night 3 round 3), the type notes, the decisions
and the traps are in `group-5 items, trophies, vi.md`. This file is the per-round log from round 12
on; where they disagree, this file wins.

## Round 12 (night 3, round 4): 15 hand ports, `src/itkinds12.rs`

**Recount first** (round 9's rule: unique `funcs()` names of gen_round8.py over
`src/melee/{it,ty,vi}/**/*.c`, ported when the name is defined in the oracle C under
`tests/c_oracle/`): **4966 total, 1497 ported** at the start (it 1493, ty 4, vi 0), matching
round 11. **1512 after** (it 1508, ty 4, vi 0).

**TODO stubs:** none replaceable. This crate's one stub (`TODO(group1)`: the item p-link list
taken as a raw head pointer, itlib3.rs / itkinds11.rs / now itkinds12.rs) still waits for items
to move onto melee_runtime's handle-based GObj model (`jobj.rs` / `gobj` there use ids, not the
GC-layout `HSD_GObj*` chain the item code walks). Other crates' `TODO(group5)` stubs are in their
paths; untouched.

**Batch.** A scan of every unported it/ty/vi function whose callees are all ported or native
elsewhere gave ~180 call-free names, many of them `order_data_*` / `sdata2_order` placeholders.
Ported the 15 with real logic that can be checked without new infrastructure:
- it_26B1.c: `it_8026B1D4` (hitbox damage + speed bonus; `my_sqrtf` is MSL `sqrtf` on PC, no
  MUST_MATCH), `it_8026C258` (nearest lock-on target on the p-link list), `it_8026C1D4` (L-Gun
  timer), `it_8026B7D8` / `it_8026B7E0` (counter getters);
- item.c: `Item_8026784C` (drop-class caps, `Item_804A0C64`);
- itcoll.c: `it_8026F9A0`; it_2725.c: `it_80272C90`, `it_802746F8`, `it_80274ED8`,
  `it_80274EE8`, `it_80274F28`;
- itmaplib.c: `it_80277544` (on round 10's `it_80277040`; PSVECAdd written out),
  `it_802775F0` (spin speed; `sqrtf_accurate_sp18` = `gekko::sqrtf_accurate`);
- it_279C.c: `it_8027B070` (attack id / staling fields from the owner Fighter).

Globals are parameters (`&ItemCommonData`, `&mut HSD_ObjAllocUnk`, the article table, the
counters, the p-link head).

**Fused sites (shipping contraction):** 6 f32: `it_8026B1D4` 3 (`vx*vx + vy*vy + vz*vz` ->
`fmadds(vz, vz, fmadds(vx, vx, vy*vy))`; `ret += speed * k5`), `it_8026C258` 1, `it_802775F0` 1,
`it_80277544` 1 (each `SQ(x) + SQ(y)` -> `fmadds(x, x, y*y)`).

**Verification** (`tests/oracle_round12.rs`, 21 tables x 48 cases, pass; oracle `round12.c`,
bodies `round12_bodies.h` by `gen_round12.py`, contraction on + lower.py). Tables 3216-3222 pick
inputs where a mutation is visible: the outer and inner `it_8026B1D4` fusions (and the inner
operand order), an `it_8026C258` pair at (a, b) / (b, a) where the fused sum makes the second item
strictly nearer plus an exact tie, `it_80277544` draws where `it_80277040` succeeds and the speed
lands exactly on `xCC` fused but below it unfused, or `xCC = FLT_MAX` so only `xD5C == 2` decides,
and every `Item_8026784C` drop class four times.
Mutations that failed the test: un-fusing each of the 6 sites, swapping the inner operands,
`<` -> `<=` in the nearest search, `>` -> `>=` on the facing side, hold kind 7 -> 8, the owner test,
`< 0` -> `<= 0` in the spin sign, the `x19 != 1` test, `xD5C == 2` -> `1`, `>` -> `>=` and `>=` ->
`>` in the drop caps, `xD08` value, `set_b5` -> `set_b4`, the child-walk bound.
**Not caught, unobservable:** `it_802775F0`'s `sqrtf_accurate` -> `sqrtf` (3 vs 4 Newton steps:
an exhaustive sweep of every positive finite f32 through `gekko::sqrtf` and `sqrtf_accurate`
found no difference); `it_8026B1D4`'s `ret <= 1.0` -> `< 1.0` (both return 1.0 at 1.0).

**Harness traps found:** round2's LCG has an alternating low bit, so `R() & 1` drawn at a fixed
position in a case is constant across cases with the same seed parity sequence (the first tie
table always took mode 0); use `(R() >> 16) & 1`. `xDD0_flag` is filled and hashed by value
in GC bit order on both sides (round2 `ufs_set` / `ufs_get`), so it needs no byte reversal (the
item-var bytes of round 4 do).

Left out: `it_80279720` / `it_80279744` / `it_80279768` (item command parsers: `CommandInfo`
script bitfields are big-endian data; needs the command-stream reader), `Item_802674AC` (PC-only
Geno / m-ex branches call shims), `Item_8026A810` / `Item_8026ADC0` (logic-table callbacks: need
typed state tables). Not done: no caller wired; no frame-level check.

**Workspace check:** `cargo build --workspace` clean; `cargo test --workspace --exclude
melee_chara` all pass. melee_chara's `chars_ref` fails (`character_functions_match_c`) on that
crate's uncommitted work in progress by another agent (its gen/chref.py, chars/cb.rs, tests/data
are modified in the tree); not touched here.

## Round 13 (night 3, round 5): 29 hand ports, `src/itkinds13.rs`

**Recount first** (same rule as round 12): **4966 total, 1512 ported** at the start (it 1508,
ty 4, vi 0), matching round 12. **1541 after** (it 1537, ty 4, vi 0).

**TODO stubs:** none replaceable. This crate's one stub (`TODO(group1)`, the item p-link list as a
raw head pointer) still waits for items to move onto melee_runtime's handle-based GObj model.
Other crates' `TODO(group5)` stubs (melee_chara `itIsHeavy`, melee_world `Toy_*` / item plink)
are in their paths; untouched.

**Batch.** From the same callee scan, functions whose callees are all ported here or that call
nothing:
- item.c `checkHitLag`; it_26B1.c `it_ResetStageArticleTable`;
- 7 `*_Coll` forwarders to `it_8027C794` (Kyasarin 9, Leadead 16, Likelike 21, Octarock 8,
  Oldkuri 11 / 7, Whitebea 11); `itZrshell_UnkMotion4_Anim` (on round 6's `it_802DDBE8`);
- itlikelike.c `itSwapVelocity` / `swapVelocity` / `it_802D9B78`; itflipper.c `itFlipper_Repel`
  (on round 10's inline); itfreeze.c `itFreeze_ClearLinkedItem` / `itFreeze_Logic17_Destroyed`
  (on round 5's `it_802E37A4`);
- attribute readers: `itClimbersBlizzard_UnkMotion0_Phys`, `it_80291CCC`, `it_80291CF4`,
  `it_80295748`, `it_80285300`, `it_80293660`, `itDosei_UnkMotion4_GetAttrs`;
- globals: `itMball_Destroyed` (`Item_804A0C64.x1C`), `itHeart_Logic8_Destroyed` /
  `itTomato_Logic9_Destroyed` (`gm_80473A18.x90[i]`, written through a struct-based pointer so an
  index past `x90[4]` lands where C's does), `it_802BDA40` (`it_803F74F8` as a pointer);
- getters `itKyasarin_GetCollData`, `it_80299FB4_get`.

**Fused sites:** none (every f32 expression is one operation or a compare; no body has an
`a * b + c` shape that contraction could fuse).

**Verification** (`tests/oracle_round13.rs`, 29 tables x 48 cases, pass; oracle `round13.c`,
bodies `round13_bodies.h` by `gen_round13.py`, which also copies each source file's own
`#define GET_ATTRS` ahead of its bodies). Mutations that failed the test: `<=` -> `<` and
`<` -> `<=` and `/ 8` -> `>> 3` in `it_80291CF4`, dropping the facing flip in `itSwapVelocity`,
`x8` -> `xC` in Blizzard, `<` -> `>` in `checkHitLag`, `b0` -> `b1` (heart), inverted tomato test,
`b1` -> `b0` (M-ball), `idx + 2` -> `+ 1` (Kinoko), the swapped `{15, 16}` table, `x1C` -> `x18`
(sword), the missing link clear (freeze), `y` -> `x` (S-Scope), a partial lipstick copy, the
dropped Repel call, the dropped `it_802DDBE8` call.

**Harness traps:** the sign-bit cancellation of round 11 hit again: `itSwapVelocity` flips the sign
of both `facing_dir` and `vel.x`, which `r10_h` hashes, so dropping the flip passed until
`r13_sh` mixed both sign bits in as bit 0. round12.c's `r12_arts[20]` is too small for
`It_Kind_Kinoko` (26), so round 13 uses its own 40-slot table. Bitfields set by name in the
item-var union (heart / tomato at +4, M-ball at +0) need `r4_rev` around them on the Rust side,
since `fields::snap` hashes the overlapping words as raw bytes.

Left out: `itSamusgrapple_UnkMotion0..8_Phys` / `itLinkhookshot_UnkMotion0..8_Phys` (they store
`fn_802B7E34` / `fn_802A2E4C`, not ported), `itKyasarinegg_UnkMotion4_Anim` (falls off the end of
a `bool` function), the `order_*` placeholders. Not done: no caller wired; no frame-level check.

## Round 14 (night 3, round 6): 26 hand ports, `src/itkinds14.rs`

**Recount first** (same rule as rounds 12-13): **4966 total, 1541 ported** at the start (it 1537,
ty 4, vi 0), matching round 13. **1567 after** (it 1563, ty 4, vi 0).

**TODO stubs:** none replaceable. The crate's one stub (`TODO(group1)`, the item p-link list as a
raw head pointer; now also `it_8026C47C`) still waits: melee_runtime's `gobj.rs` keeps
`plink_head: Vec<Option<GObjId>>`, ids rather than the GC-layout `HSD_GObj*` chain the item code
walks. Other crates' `TODO(group5)` stubs (melee_chara `itIsHeavy`) are in their paths; untouched.

**Batch** (the callee scan again: 102 unported names whose callees are all in the oracle):
- it_2725.c `get_bone_by_id`; itsword.c `it_80285314` (on round 11's `it_80272CC0`);
- itanimlist.c `it_80279768` / `it_80279720` / `it_80279744` (`CommandInfo` step + `set_throw_flags.hit_idx`);
- itspawn.c `it_8026C47C` (kind bitmap over the p-link list, on round 3's `it_80272828`);
- kinds: `itYoshistar_UnkMotion0_Phys`, `itSamusbomb_UnkMotion2_Phys` (4-step `my_sqrtf` =
  `my_sqrtf_accurate`, melee_runtime `lbvector::normalize_xy`), `itNesspkfirepillar_Logic24_DmgReceived`,
  `likelikeVelocity` (on round 13's `swapVelocity`), `it_802AB9C0` (on `it_802AC5D8`),
  `itPikachutjoltair_UnkMotion0_Anim_inline` (on `it_802B3368`), `itPikachuthunder_UnkMotion2_ScaleCall`,
  `it_802B22B8_inline`, `fn_80280974_inline` (BombHei), `it_80291D38` (S-Scope, charge 0..9),
  `it_8028B08C` (Parasol; `it_803F5AB0` passed as a slice);
- empty `it_802D14D0`, `it_802F1340`; getters `it_802D472C_inline`, `fn_802A33A0_GetFighter`,
  itzgshell `get_attrs`, `itDosei_GetJObj`, itparasol `jobj_child`, `it_802C248C_joint24/28`.

**Fused sites:** 3 f32: `vel.x += accel * facing_dir` (Yoshi star), `VEC2_SQ_LEN` ->
`fmadds(x, x, y*y)` (Samus bomb), `-((xC9C * x4) - life)` -> `-fmsubs` (PK Fire pillar).

**Verification** (`tests/oracle_round14.rs`, 28 tables x 48 cases, pass; oracle `round14.c`, bodies
`round14_bodies.h` by `gen_round14.py`; round14.c supplies ityoshistar.c's file-local `StarAttrs`
verbatim and `#define my_sqrtf my_sqrtf_accurate` for itsamusbomb.c). Two extra edge tables
(3427 Samus bomb with `xC` = the contracted length or one ulp under; 3428 Like-Like at
`|d| == limit`). Mutations caught: un-fusing each of the 3 sites, swapping the fused operands,
`<=` -> `<` (pillar), `x48 >= 4` -> `>= 5` / `> 4`, `<` -> `<=` on both Like-Like axes, `&&` -> `||`,
`> xC` -> `>=`, the kind loop bound and the `& 31` word step, bone 5 -> 4, bone table index, the
pika_scale operand swap, dropped `xD5C = 0` / `it_80275444` / `it_8026B3A8` / owner / x13 / link
calls, the parasol table order, accel -> speed, x24 vs x28 joint, x0 -> x1 flag, hit_idx -> opcode.
**Survived (unobservable by these inputs):** 4-step -> 3-step `my_sqrtf` in the Samus bomb (the
fourth Newton step does not change the f32 result on any drawn input); the port follows the source.

**Harness traps (new):** (1) pointers stored into the item-var union alias fields `fields::snap`
hashes (heal amounts at +0 etc.), and pointer values differ between the sides: clear them before
the hash. (2) `R() & 1` / `R() & 3` on the raw LCG cycles with period 2 / 4 (the low bits of an
LCG mod 2^32), so a coin like `(R() & 1) ? a : b` is fixed per draw position: round 14 draws
`(R() >> 16) & n`. (3) C's `ABS(RM())` evaluates `RM()` twice; take it into a temporary.

Left out: `it_802E6380_tier` (reads `tiers[1].threshold` through `(s32*)&tiers[0].ecb + 9`, a GC
offset alias that lands in a pointer on the native layout), `itKyasarinegg_UnkMotion4_Anim`, the
`order_*` placeholders, the hookshot `ItemLink` inlines, the ty/vi call-free helpers (next round).
Not done: no caller wired; no frame-level check.
