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
