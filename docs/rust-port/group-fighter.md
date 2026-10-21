# Group 3: fighter engine + common states (`rust/melee_fighter`)

Scope: `src/melee/ft/*.c` (67 TUs), `src/melee/ft/kinds/ftCommon/*.c` (142), `src/melee/pl/*.c`
(7): 216 TUs, ~76k lines, 3,002 function definitions (recount, port round 9: a definition line
followed by a `{` line, static inline helpers included; the older "~2,820" left those out).
State as of 2026-10-21 (group 3 round 12).

## What exists

| layer | where | status |
|---|---|---|
| Native types + GC offsets | `src/types/generated.rs` (from `gen/gen_types.py`) | 569 `#[repr(C)]` records, all built |
| Mechanical translation of all 216 TUs | `src/c2r/` (from `gen/fix_raw.py`, feature `c2r`) | compiles (`--features c2r`), never run |
| Hand ports, checked bit-for-bit against the C | `src/ftaction.rs`, `src/fighter.rs`, `src/ftanim.rs`, `src/ftcoll.rs`, `src/neutral_hash.rs`, `src/ftcommon.rs`, `src/ftcommon2.rs`, `src/stale.rs`, `src/player.rs`, `src/ftlib.rs`, `src/plbonus.rs`, `src/ft0877.rs`, `src/ft0881.rs`, `src/ft0892.rs`, `src/pltrick.rs`, `src/ftcommon3.rs`, `src/plbonus2.rs`, `src/plbonus3.rs`, `src/ft084e.rs`, `src/compat.rs`, `src/hooks.rs` | 634 C functions (633 verified), tests pass |
| PowerPC `.s` files | none belong to this group (inventory §3) | nothing to do |

Functions: 634 of 3,002 (21.1%; 57 night 1, 39 in port round 1, 144 in port round 2, 96 in port round 3, 82 in port round 4, 34 in port round 5, 18 in port round 6, 19 in port round 7, 37 in port round 8, 50 in port round 9, 24 in round 10, 15 in round 11, 19 in round 12) are hand-ported; 633 are verified against the C (the one unverified port: `ftCommon_8007E2D0`, round 1, stores three fn pointers). All are
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
| `ftaction.rs`, `fighter.rs`, `ftanim.rs`, `ftcoll.rs`, `ftcommon3.rs` (angles) | 50 leaf functions (port round 9, below) | `tests/ftleaf_ref.rs` | 10,000 (50 ops x 200) |
| `neutral_hash.rs` | `nh_Mix`, `nh_CpuFlags`, `nh_Fighter` (fighter.c, TARGET_PC) + item/game word, `StateHash for Fighter` | `tests/nh_ref.rs` | 64 fighters, all fields + every CPU flag bitfield |
| `ftcommon3.rs` | 37 more of ftcommon.c (port round 8, below) | `tests/ftcommon3_ref.rs` | 10,200 (34 ops x 300) |
| `ft0881.rs` | 18 of ft/ft_0881.c (port round 6, below) | `tests/ft0881_ref.rs` | 5,100 (17 ops x 300) |
| `ft0877.rs` | all 34 of ft/ft_0877.c (port round 5, below) | `tests/ft0877_ref.rs` | 10,200 (34 ops x 300) |
| `plbonus.rs` | 82: all of pl/pl_040D.c (22) + plbonuslib.c pl_8003FBFC..EOF (59) + its inline `unk_cond` (port round 4, below) | `tests/plbonus_ref.rs` | 15,000 (300 tables x 50 ops, 81 op kinds) |
| `ftlib.rs` | 96 of ft/ftlib.c (port round 3, below) | `tests/ftlib_ref.rs` | 10,440 (87 ops x 120, 6 fighters + random p-link list) |
| `player.rs` | 144 of player.c: the player-table accessors (port round 2, below) | `tests/player_ref.rs` | 200 tables x 60 ops (144 op kinds, all exercised) |
| `ftcommon2.rs` | 39 more of ftcommon.c (port round 1, below) | `tests/ftcommon2_ref.rs` | 10,200 (34 ops x 300) |
| `ftcommon.rs` | 33 functions of ftcommon.c (ground/air accel, drift, friction, KB friction, fall/ascend, CheckFallFast, facing, CalcHitlag) | `tests/ftcommon_ref.rs` | 6,600 (33 x 200) |
| `stale.rs` | plstale.c (5), plattack.c (3), ft_0881.c staling (5), player.c accessors (8) | `tests/stale_ref.rs` | 300 sequences x 40 ops |
| `compat.rs` | HitCapsule `hit_grabbed_victim_only` GC semantics | unit test | - |

How the C reference works: `gen/cref/run.py` compiles each `gen/cref/<name>_ref.c` natively
with clang (the same flags as `tools/rust-port/gen_cc.py`, plus `pc/gameworld/gekko_fp.c` for
`__frsqrte`) against the real headers. Since port round 1 it uses the SHIPPING contraction:
`-ffp-contract=on -O2 -Xclang -disable-llvm-passes -ftrivial-auto-var-init=zero` to LLVM IR,
then `rust/melee_runtime/oracle/lower.py` (gwtool's fmadds / fma / saturating float->int), then
link. Re-running nh, stale and ftcommon this way gave byte-identical expected files. A driver
must contain no `a*b+c` of its own (it would be fused too and the inputs would differ from the
Rust test's); check with `awk '/^define/{f=$0} /xp = fmul double/{print f}' rust/target/cref/<n>__<n>_ref.lo.ll`. The game code under test is either `#include`d whole
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
5. **Fused multiply-add** (corrected night 2). The shipping C build fuses every `a*b+c` clang
   contracts (`tools/rust-port/fma_sites.py`); the ports fuse exactly those sites with
   `melee_core::fp::fmadds`, and the cref oracle is built the same way (above).
6. **Sibling crates did not build.** `melee_types`/`melee_runtime`/`melee_world` did not build
   while this was done (other agents generating them). `Cargo.toml` has them commented out, and
   the shared types this group needs are generated here with `TODO(move-to-core)`.

## Port round 1 (2026-10-21): `src/ftcommon2.rs`
39 C definitions of ftcommon.c, all field logic, verified on 10,200 generated cases against the C
cut verbatim (`gen/cref/ftcommon2_ref.c`, 13 extract ranges in run.py):
SandbagGetKnockbackDeaccelX/Y, UnlockECB, 8007D5D4, 8007D60C, UseAllJumps, 8007D780, InitGrab,
GrabMash, 8007E2A4, 8007E2D0 (setter; not exercised by the test: fn-pointer args), 8007E2F4,
8007E2FC, 8007EBAC, 8007ECD4, 8007ED2C, 8007ED50, 8007EE0C, 8007EFC0, GetModelScale, 8007F76C,
8007F7B4, 8007F804, 8007F8B4, my_fminf, 8007FDA0, 8007FF74, 8007FFD8, 80080144, 80080174,
8008021C, 8008031C, 80080460, 80080474, 80080484, 800804A0, 800804EC, 800804FC
That is 39 C definitions: 37 functions plus the static inlines `_func_8007E2FC_inline` and
`my_fminf`. 38 are verified; `ftCommon_8007E2D0` (stores three fn pointers) is ported but not in
the test.
- Fused sites (fma_sites.py): 8007ED50, 8007EE0C, 8007FDA0, 80080174, one f32 each -> fmadds.
  Discriminated: un-fusing `x710*x2024+x708` (8007FDA0/80080174) fails the test (op 21 case
  34). Un-fusing 8007ED50/8007EE0C still passes: the int truncation hides the difference on
  these inputs, so those two are right by the IR only.
- New hooks (`hooks.rs`): Geno_GroundEdge (vanilla no-op), pl_8003FC44 / pl_800402D0 /
  pl_80040460 / Player_8003544C / Player_UpdateJoystickCountByIndex (TODO(group3 pl)),
  lb_80014574 / HSD_PadRumbleRemoveId / lb_800145C0 (TODO(group1), rumble, no-op default),
  un_803224DC (TODO(group6)), it_80294E78 / it_8029A89C / it_8029A8F4 (TODO(group5)),
  ftCo_80090690 / ftCo_800DEEE8 (TODO(group3 ftCo states)), `Fighter_GrabMashShake`.
  Hooks with no neutral default panic when reached unset (never a silent wrong value).
- Not ported from ftcommon.c yet: 8007D6A4/8007D7FC (PC floor guard calls mpLib + Lua event),
  the nudge pass 8007DD7C/8007DFD0/8007E0E4 (GObj list + mpLineGetNext/Prev), JObj users
  (8007E358, 8007E3EC, SetAccessory, 8007F6A4...), item callers (8007E6DC..., ftGetParasolStatus,
  8007EEC8/EF5C), the m-ex table dispatches (8007F578..8007F9B4), atan2f users.

## Port round 2 (2026-10-21): `src/player.rs`
144 C definitions of src/melee/pl/player.c, verified against the C cut verbatim
(`gen/cref/player_ref.c`; run.py extracts `ftMapping_list`/`player_slots`, Player_GetPtrForSlot..
Player_80031900, Player_80031DA8..Player_80036978, and the TARGET_PC Player_MexSetMapping /
MexSetPartner / ForgetEntities). 200 random player tables x 60 random ops; after each op the
return value, the out-vector, the stub trace word and the whole touched slot are printed; at the
end every slot and the first 40 mapping rows. 33,200 lines compared, every one of the 144 op kinds
exercised (the test asserts it). Mutation check: dropping the clamp in GetRemainingHPByIndex or
changing UpdateMatchFrameCount's 0xFFFF test fails the test.
- Ported: hasExtraFighterId, Player_80031790, 80031848, func_8008688C_wrapper, 80031900,
  800319C4, 80031DA8, 80031DC8, 80031EBC, 80031FB0, 80032070, 8003219C, 8003221C,
  Get/SetPlayerState/Character, 800325C8, 80032610, LoadPlayerCoords, 80032768, 80032828,
  800328D4, 80032A04, SetPlayerAndEntityFacingDirection, 80032BB0, SetScale, spawn/some pos
  get/set, 80032F30/FA4, facing get/set/conditional, costume, controller, team, player id, cpu
  level/type (+entity), handicap, unk50, attack/defense ratio (TARGET_PC ScriptGame_ModValue),
  model scale, 80033BB8, GetP1Stock, LoseStock, coins, total coins, unk98/9C, GetEntity(AtIndex),
  SwapTransformedStates, damage/HUD damage/HP by index, other stamina, remaining HP (+by index),
  every flags / more_flags / flagsAE bit get/set, falls (4), KOs (2), match frame count (2),
  self destructs (3), 800353BC, 8003544C, nametag, unk4C/4D/45, 80036058, 800360D8,
  SetStructFunc, GetUnk6A8Ptr, GetStaleMoveTableIndexPtr2, 80036394, 80036428, joystick count
  (2), 800366DC, 80036790, 80036844, 800368F8, 80036978, MexSetMapping, MexSetPartner,
  ForgetEntities. `ftMapping_list` is `player::FT_MAPPING_LIST` (ChKind_Cap rows, as TARGET_PC).
  `Player_8003248C` now has a table-reading wrapper (`player::player_8003248c`; the night-1
  `stale::slot_type_for` is not counted again).
- Fused sites: none in player.c (`fma_sites.py src/melee/pl/player.c`: 0 in every function).
- Hooks: `hooks::PLAYER_8003544C` / `PLAYER_UPDATE_JOYSTICK_COUNT` now default to the ports.
  33 new panicking hooks (`needed_hooks!` in hooks.rs): ftLib_* / ft_8008* (TODO(group3 ftlib)),
  ftCo_800D4F24 / 800D4FF4 (TODO(group3 ftCo)), ftDemo_SetFacingDirection (TODO(group3 ftdemo)),
  ftKb_SpecialN_800EED50 (TODO(group4)), HSD_GObjFree (TODO(group1)), ifStatus_* and gm_*
  (TODO(group6)). `hooks::script_mod_value(slot, ..)` is the slot-keyed form of SCRIPT_MOD_VALUE.
- Differences kept on purpose: `transformed[i]` >= 2 panics (C reads the neighbouring field);
  `Player_SetFlagsAEBit1` returns () (C: declared u8, returns nothing). Neither is reachable with
  retail data.
- Not ported from player.c: Player_80031AD0 (Fighter_Create), 80031CB0 / 80031D2C (ftData
  preload, Geno, lbDvd), Player_InitOrResetPlayer and the rest (allocators, archives, JObj).

## Port round 3 (2026-10-21): `src/ftlib.rs`
96 C definitions of src/melee/ft/ftlib.c (93 functions plus the static inlines `sgn`,
`vector_add`, `helper`), verified against the C cut verbatim (`gen/cref/ftlib_ref.c`; run.py cuts
ftlib.c 800860C4..800871A8, 80087284..80087508, 800876B4..8008777C, 800877D4..EOF, plus the
already-ported ftcommon.c 8007ED2C, GetModelScale, 80080144, 80080174 so SetScale / 80087140 /
8008746C run the real C). Six fighters with ftData / attrs / sfx / CmSubject / bone tables, a
random fighter p-link list (0-6 entries, random order), 87 ops x 120 cases = 10,440 lines; each
prints the return value (gobj/joint/pointer results as an identity index), the out-params, a hash
of every mutable field of all six fighters, and the stub trace word. All 87 ops are exercised
(the test asserts it). Mutation check: dropping `!x221F_b1` from 80086F4C fails (op 55 case 11).
- Signature shape: `g: *mut HSD_GObj` like the C (`gobj->user_data` is the Fighter); the p-link
  list comes from `hooks::plink_head` (TODO(group1 HSD)).
- TARGET_PC paths ported: 80086A8C returns true + UpdateLogicOffCamera; Slippi Whispy fix in
  800864A8; Geno template kinds in GetKind; m-ex relative crowd sfx in 8008746C.
- Fused sites: 8008627C and 80086368, `dx*dx + dy*dy` -> `fmadds(dx, dx, dy*dy)`. **Not
  discriminated**: un-fusing both still passes (the result is an argmin over gobjs and these
  inputs never flip it). Right by the IR (fma_sites.py) only.
- `dmg.x1830_percent` is a float: 800870F0 converts int->float, 80087120 and 80086198 float->int
  (saturating, same as fctiwz except NaN, see Traps 3; inputs contain no NaN).
- `ftLib_800870BC` reads `p_ftCommonData->x6D8[controller_index]`, a 1-element array read past
  its end (into `x6DC_colorsByPlayer`), kept as raw pointer arithmetic.
- The 16 `ftLib_*` hooks player.c calls (`FTLIB_*`) now default to these ports (setting one still
  overrides it); `PLAYER_GET_MORE_FLAGS_BIT4` defaults to `player::get_more_flags_bit4`.
  New panicking hooks: HSD_GObjPLinkHead / HSD_Randi (group1 HSD), lb_8000B1CC (group1 lb),
  Camera_80030A50 / LogicToScreen / 80031154 (group2 cm), Slippi_Codes (pc), Geno_DefineBaseKind,
  Mex_SsmForPortKind (pc), ftParts_GetBoneIndex, Fighter_UpdateModelScale / ResetInputData /
  UnkProcessDeath, ftCommon_8007E6DC / 8007E82C / 8007D92C, ftData_80085560, pl_8003E4A4,
  ftAnim_IsFramesRemaining (group3, not yet ported), ftGw/ftKb Init (group4).
  `GENO_DEFINE_TEMPLATE_SKELETON` defaults to 0 (vanilla: no Geno defines).
- Not ported from ftlib.c: 800871A8 (metal box: ftCo/it/camera/sfx), 80087508 / 80087574 /
  80087610 (archive preloading), 8008777C (atan2f, waits on group 1's MSL).

## Port round 4 (2026-10-21): `src/plbonus.rs`
82 C definitions, verified against the C cut verbatim (`gen/cref/plbonus_ref.c`; run.py cuts the
same player.c ranges as player_ref plus `pl_040D.c` from pl_80040DDC to EOF, plbonuslib.c
`unk_cond`..`pl_8003D644` and `pl_8003FBFC`..EOF):
- pl_040D.c, all 22: pl_80040DDC, 80040ED4, 80040FBC, 800410F4, 800411C4, 80041280, 800412D0,
  80041300, 8004134C, fn_8004138C, 800414C0, 80041524, fn_800415B0, 80041720, 80041744,
  fn_80041770, 8004182C, 800418F4, 800419AC, 80041B08, fn_80041BC8, 80041BFC.
- plbonuslib.c, 59 + static inline `unk_cond`: pl_8003FBFC, FC20, FC44, FC88, FDA0, FDC8, FDF4,
  FE1C, FE40, FE64, FED0, FF44, FFDC, 80040048, 0120, 01F0, 0270, 02D0, 0330, 0374, 03C0, 03FC,
  0460, 049C, 0614, 065C, 0688, 07C8, 0870, 0894, 08B8, 08DC, 0900, 0924, 0948, 0A04, 0A30, 0A54,
  0A78, 0A9C, 0AF0, 0B18, 0B3C, 0B64, 0B8C, 0BD8, 0BFC, 0C24, 0C48, 0C6C, 0C90, 0CB4, 0CD8, 0CFC,
  0D20, 0D44, 0D68, 0D8C, 0DB8. The header inlines `pl_CalculateAverage` (plbonusinline.h) and
  `pl_CheckIfSameTeam` (pl/inlines.h) are ported too but not counted (group 0 header inlines).
- Test: 300 random tables (6 slots: stale-table extension fields, x5EC / x8C0 / x8D8 rings,
  action stats, entities, team, state) + random pl_804D6470 constants, 50 ops each; per op the
  return value, the stub trace word and a hash of every field these functions write, all 6 slots.
  15,000 lines, all 81 op kinds exercised (asserted). Mutation check: reading
  `by_attack_counts[DownAttackD + k]` instead of `+ k + 1` in pl_80040A9C fails (case 0 step 41);
  wrapping the 8C0 ring at 3 instead of 4 fails (case 3 step 48).
- Fused sites: none (`fma_sites.py src/melee/pl/pl_040D.c src/melee/pl/plbonuslib.c`: the only
  sites in plbonuslib.c are in pl_8003ED0C, fn_8003F294, fn_8003F654, pl_8003FAA8, not ported).
- Bitfields: the GC (MSB-first) and native-C (LSB-first) bit orders differ, so the test hashes
  fields through accessors, never raw bytes. `xDD0` / `xDD1` are unions; bits go through `anon_0`.
- float -> int / u32 returns (800412D0, 80041300, 80040B3C, 80040B64, 80040BFC) saturate as
  gwtool's lowering does (`as`; NaN not generated).
- Hooks: `PL_8003FC44` / `PL_800402D0` / `PL_80040460` (ftcommon2's calls) now default to these
  ports. New: `PL_804D6470` (constants table pointer, TODO(group3 plbonus) loader) and panicking
  hooks pl_80038788 / 80038824 / 80038898 / 80039418 / 800386D8 (TODO(group3 plbonus): plbonus.c),
  ft_80087A8C / A98 / AA4 / AB4 (TODO(group3 ft_0877)), ft_800898B4 (TODO(group3 ft_0892)),
  gm_8016F2F8 / gm_8016B6E8 (TODO(group6 gm)).
- C quirks kept: ring indices (`x8C0.x0`, `x8D8.x0`) and the `x5EC.x0` slot are unchecked in C;
  out-of-range values panic here. pl_8004049C's HSD_ASSERTREPORT on a bad player panics.
- Not ported from plbonuslib.c: plBonusLib_8003D514 .. pl_8003FAA8 (ftLib/item/stage callers,
  the sqrtf/fused-site functions).

## Port round 5 (2026-10-21): `src/ft0877.rs`
All 34 C definitions of src/melee/ft/ft_0877.c, verified against the C cut verbatim
(`gen/cref/ft0877_ref.c`; run.py cuts ft_0877.c from `#define TEST` to EOF plus ftcommon.c
`ftCommon_80080144`): ft_800877F8, 80087818, 80087838, 80087858, 80087878, 800878BC, 80087900,
80087944, 80087988, 800879D8, 800879F8, 80087A18, 80087A80, 80087A8C, 80087A98, 80087AA4,
80087AB4, 80087AC0, 80087AEC, 80087B34, 80087BAC, 80087BC0, 80087BEC, ft_GetFtKindMask,
80087C58, 80087C64, 80087C70, 80087D0C, 8008805C, 80088080, 800880AC, 800880D8, 80088110,
ft_PlaySFX. One op per function.
- Test: six fighters, random p-link list, random fields; the trace word starts from a random value
  per case so stub results vary. 34 ops x 300 = 10,200 lines (return value, hash of
  invisible/x2228_b5/x1988/x2164/x2168/x2160 of all six fighters, trace word). All ops exercised.
- Fused sites: ft_80087B34 only, five f32 fmuladd (`x*x + y*y` -> `fmadds(x, x, y*y)`).
  **Discriminated**: the driver sometimes makes two velocity vectors swapped copies (equal in exact
  math), and un-fusing `v1` fails (op 19 case 164).
- Mutation checks that fail: dropping `|| v == 0x80` in 80087A18 (op 11), the case-13 fallthrough
  in 80087D0C (op 27), case 2 of 80087C70 (op 26), `|=` -> `^=` in GetFtKindMask (op 23).
  Not discriminated: PlaySFX's random-pan range end (370 vs 369; no stub returns 370).
- TARGET_PC paths ported: m-ex relative sfx ids (5000..9999) in 80087D0C, Geno voice owner in
  ft_PlaySFX (new hook `GENO_DEFINE_VOICE_PLAY_OWNER`, unset = 0 = vanilla path).
- Hooks: `PLAY_SFX` (ftcommon's CheckFallFast) now defaults to the port (it used to be a no-op);
  the eight `FT_80087*` hooks player.c / plbonus call now default to the ports. New panicking hooks:
  itIsHeavy / itGetKind (TODO(group5 it)), ftGetParasolStatus (TODO(group3 ftcommon)),
  ftCo_800C06B4 / ftCo_800A101C (TODO(group3 ftCo)), lbAudioAx_800233EC / 80023130 / 800230C8 /
  80023220 / 80024DC4 / 80024FDC / 80024FF4 / 8002500C / 80025038 / 800237A8 / 80024B94
  (TODO(group1 lb)). Sound is therefore loud-unported: ft_PlaySFX panics unless those are set.
- `Geno_DefineVoicePlayOwner` takes the fighter pointer (the C passes `(u32) fp`).

## Port round 6 (2026-10-21): `src/ft0881.rs`
The 18 remaining C definitions of src/melee/ft/ft_0881.c (its staling tail is in `stale.rs`):
17 functions ft_800881D8, 80088328, 80088478, 80088510, 800885A8, 80088640, ftCo_800886D8,
80088770, 800887CC, 80088828, 80088884, 800888E0, 8008893C, 80088998, 800889F4, 80088A50,
80088C5C, plus the static inline `inline0` (folded into ftCo_800886D8).
Verified against the C cut verbatim (`gen/cref/ft0881_ref.c`; run.py cuts ft_0881.c from its
TARGET_PC `FT_GENO_VOICE` block to `ft_800890BC`, plus all of ft_0877.c and ftCommon_80080144,
because every play path goes through ft_80087D0C).
- Test: six fighters (kinds incl. Samus, Game & Watch, m-ex 33/34 for the Geno voice path),
  random player_id / is_sub_fighter / x2225_b6, random channel handles (a third are -1), sfx arg
  0x83D60 / 0x83D61 / m-ex relative / random. 17 ops x 300 = 5,100 lines (hash of all ten sfx
  words of all six fighters + trace word). All ops exercised.
- Mutation checks that fail: 80088328's stop calling 80088770 instead of 800887CC (op 1 case 1);
  dropping Samus from the x2225_b6 allow-list (op 0 case 12).
- Fused sites: none (`fma_sites.py src/melee/ft/ft_0881.c`).
- Driver trap found: clang for x86_64-pc-windows-msvc evaluates call arguments right to left,
  so a driver must never pass two `rnd()` calls in one argument list (now drawn into locals).
- `lbAudioAx_800265C4` returns `bool` but 80088C5C tests `== 1`; the stub returns 0/1/2 in C and
  the Rust hook (bool) models it as `== 1`. If group 1's port returns >1 values this hook shape
  must change to i32.
- New panicking hooks: lbAudioAx_80023870 / 80026510 / 800265C4 (TODO(group1 lb)).

## Port round 7 (2026-10-21): `src/ft0892.rs`, `src/pltrick.rs`
All 12 C definitions of src/melee/ft/ft_0892.c (ft_800892D4, 80089460, 800895E0, 80089768,
80089824, 80089884, 80089890, 8008989C, 800898A8, 800898B4, 800898C0, 80089914) plus
`ftGetTemplateKind` (ft/inlines.h, as `ft0892::template_kind`), and 7 of pltrick.c's 10
definitions: the table pl_803BCE70, pl_80037B2C, 80037BC0, 80037C60, 80037DF4, 80037ECC,
800384DC, 80038628. Not ported: fn_80037F00, pl_80038144 (need ifMagnify, gm, more plbonuslib).
Verified against the C cut verbatim (`gen/cref/ft0892_ref.c`; run.py cuts all of ft_0892.c, the
two pltrick.c ranges, plAttack_80037B08 and Player_GetActionStats).
- Test: one fighter with random Struct2070/2074/x18d4/x18d8 fields, random action stats for its
  slot, random attack counter (incl. 0xFFFF wrap). 19 ops x 300 = 5,700 lines (return value, hash
  of every touched field by name + the whole plActionStats + counter, trace word). All ops hit.
- Fused sites: none (`fma_sites.py` over ft_0892.c and pltrick.c).
- **Byte order (game is big-endian).** `union Struct2070 { struct {s8; bits; u8 x2073}; int }`:
  the port keeps GC byte order in the struct view, and the int view is
  `ft0892::s2070_gc_int` / `s2070_from_gc_int` (big-endian). Int arguments that the C puns
  (`ft_800895E0(fp, int)`, `pl_80037C60(g, s32)`, `pl_800384DC(g, int, void*)`) take a
  `Struct2070`; callers convert with `s2070_from_gc_int` (it_279C passes a bare x2073 value as
  that int: in the game that lands in x2073). The native C oracle lays the two literals of
  ft_800895E0 (0x240063 Luigi, 0x44003D item) out little-endian, so both sides hash them as tags
  A1/A2 instead of fields: the literal paths are checked as taken, not their field contents.
- `by_attack_hi[id]` (pl_80037C60, id >= 100) and `by_attack_counts[id]` (pl_80037BC0) are
  unchecked in C. The port increments u32 fields as the C for ids that land on u32 fields
  (tested up to id 115), does a big-endian byte increment for by_attack_hi 116/117 (x5B8/x5BC;
  untested, the native C differs) and panics for by_attack_hi id >= 118 (past plActionStats into
  the GC StaticPlayer). pl_80037B2C returns 0 for idx >= 100 (C has no return there).
- Mutation checks that fail: either literal (op 2), Mewtwo exclusion (op 11), 113 in 800898C0
  (op 10), x2084_b6 -> b5 (op 1), dropping x2072_b2 in 80037C60 (op 4), id 0x63 -> 0x64 and the
  x10_b7 test in 800384DC (op 17).
- Hooks: `FT_800898B4` now defaults to the port; new ported hooks `PL_80037C60` (pltrick),
  `PL_8003FE40` (plbonus). New panicking hooks: it_8026B6C8 (TODO(group5 it)),
  ftFx_AppealS_CheckIfUsed (TODO(group4 ftFx)), pl_8003DFF4 (TODO(group3 plbonuslib)),
  `FTDATA_80085FD4_X8` = `ftData_80085FD4(fp, i)->x8` (TODO(group3 ftdata)).

## Port round 8 (2026-10-21): `src/ftcommon3.rs`
37 more C definitions of src/melee/ft/ftcommon.c: ftCommon_8007D6A4 (with the TARGET_PC statics
`ftCommon_PcFloorGuard` / `ftCommon_PcFloorValid`), 8007D7FC, 8007D92C, 8007DB24, 8007DB58,
8007DD7C, 8007DFD0, 8007E0E4, 8007E358, 8007E3EC, SetAccessory, 8007E690, 8007E6DC, 8007E79C,
8007E7E4, 8007E82C, 8007E83C, ftGetParasolStatus, 8007EA90, 8007EC30, 8007EEC8, 8007EF5C,
8007F578, 8007F5CC, 8007F6A4, 8007F824, 8007F86C, 8007F8E8, 8007F948 + `_func_8007F948_inline`,
8007F9B4, 8007FA00, 8007FA58, 8007FC7C, 8007FE84.
Verified against the C cut verbatim (`gen/cref/ftcommon3_ref.c`; run.py cuts 13 ftcommon.c ranges
plus ft_0881.c's ft_80088770 / ft_800887CC; run.py's start marker now takes a leading `^` to begin
one line early, at the `#if defined(TARGET_PC)` opening the floor-guard block).
- Test: six fighters, random p-link list, three items with random kind/motion, random fields,
  trace word seeded per case. 34 ops x 300 = 10,200 lines (return value, hash of every written
  field of all six fighters, trace word). The HSD_JObj header inlines are redirected to stubs by
  macro in the C, hooks in Rust.
- Fused sites: 8007DD7C / 8007DFD0 two each (`x2C4.x * facing + v.x`), 8007E3EC three
  (`x1A7C += d * x1A6C`); all `fmadds`. **Discriminated**: odd nudge cases make the arg side
  cancel exactly unfused (other side exactly 0), so un-fusing the arg side fails (op 5 case 47);
  un-fusing x1A7C.z fails (op 9 case 0). Not discriminated: the other-fighter side of the nudge
  (zeroed in those cases), and 8007DFD0's own fused pair beyond what op 6 hits.
- Other mutation checks that fail: the f64 `phi > M_PI` compare done in f32 (op 18; the angle
  stub returns `(float) M_PI` 1 in 8), dropping Player_SetUnk98 (op 18), parasol table-3 NULL
  check (op 16), x700 clamp in 8007FC7C (op 32), `(float) x2024 + arg8` truncation (op 32),
  `x221E_b3 = arg1` keeping only bit 0 (op 23), phi_r28 in 8007DD7C (op 5), 8007DFD0's x45C
  (op 6), the mpLineGetPrev adjacency (op 5). Not discriminated now: PcFloorGuard's every-120th
  report (needs >120 misses for one player; it was hit before the trace seed was randomised).
- TARGET_PC paths ported: the missing-floor guard (falls through ftCo_Fall_Enter instead of
  landing; `PC_FLOOR_MISSES` is a never-reset static like the C), Script_GameEvent on landing
  (hook, default no-op), the m-ex `Mex_On*Dispatch` calls (hooks; they also stand for the
  ftData_* per-kind tables, TODO(pc mex + group3 ftdata)), the parasol table-3 bounds check.
- `it_8026B54C` returns float (it_26B1.h): 8007FA58 truncates it into x2014 / 8007F948's arg.
- Hooks: `FTCOMMON_8007E6DC` / `8007E82C` / `8007D92C` and `FT_GET_PARASOL_STATUS` now default to
  these ports. New ported overrides: FTLIB_80086FD4, FTLIB_800866DC, PLAYER_GET_ENTITY,
  PLAYER_GET_UNK98 / SET_UNK98. New panicking hooks: mpLib_8004D164 / mpLineGetNext / Prev
  (TODO(group2 mp)), ft_80084A18 / ft_80081C88 (TODO(group3 ft_081B)), ftCo_Fall_Enter,
  ftCo_800C5240 / 800C5500 / 800D105C / ftCo_Damage_CalcAngle (TODO(group3 ftCo)), ft_8008A2BC,
  efLib_DestroyAll, the HSD_JObj ops (TODO(group1 HSD)), pl_8003EA08 (TODO(group3 plbonuslib)),
  the nine m-ex dispatchers, gm_8016B0B4 (TODO(group6 gm)), and 17 item calls (TODO(group5 it)).
- Not ported from ftcommon.c: the atan2f users (ftCo_GetLStickAngle, GetCStickAngle, 8007D9D4,
  8007E5AC; need group 1's MSL), 8007EFC8 (transform swap, ~25 callees), PcFloorGuardTest.

## Port round 9 (2026-10-21): leaves of ftaction / fighter / ftanim / ftcoll + stick angles
Recount first: 526 ported before this round (525 verified), of 3,002 definitions.

Stubs replaced by ports that now exist elsewhere: the seven group-5 item hooks group 3 calls
(`IT_IS_HEAVY`, `IT_GET_KIND`, `IT_8026B6C8`, `IT_GET_MOTION_ID`, `IT_8026B54C`, `IT_8026BD0C`,
`IT_8026BCF4`) now default to `melee_items::itlib` (verified by group 5) through `gobj->user_data`
instead of panicking; still overridable (the ref tests install tracing stubs). Wiring test:
`tests/hooks_items.rs`. melee_fighter now depends on melee_items and melee_runtime (no cycle).
Not replaced, and why: `HSD_RANDI` / `PLINK_HEAD` (group 1 has them, but as methods on a
`Random` / GObj-system instance, no global to default to), `MP_LINE_GET_NEXT/PREV` (group 2's
are methods on a map-lines instance), the lbAudioAx, gm, if, cm hooks (no port exists yet).

50 C definitions ported, all verified against the C cut verbatim (`gen/cref/ftleaf_ref.c`;
run.py gained a `fn:NAME` extract that cuts one definition through its closing `}`):
- ft/ftaction.c (29, `src/ftaction.rs`): the 16 `SKIP_CMD` handlers (800711DC, 800715EC, 8007168C,
  800716F8, 80071774, 800717C8, 80071810, 80071CA4, 80071D30, 80071F0C, 800722C8, 800726C0,
  80072E24, 80072FE0, 8007309C, 8007320C), the 9 `NEXT_CMD`-only ones (8007296C, 800729C4,
  80072A4C, 80072AAC, 80072B04, 80072B84, 80072BE4, 80072C5C, 80073108) and the 4 flag setters
  (80071908 / 8007192C / 80071974 throw_flags_b1 / b2 / b0, 80071950 allow_interrupt).
  Handlers that read `CmdUnion` bitfields are left: the C build declares them LSB-first (x86) over
  command words whose in-memory byte order on the native build is not decided; that needs one
  decision shared with groups 1/4/5 (lbcommand, item commands) before porting.
- ft/fighter.c (10, `src/fighter.rs`): Fighter_NewSpawn_80068E40 (static counter
  `Fighter_804D64F8`), Fighter_UnkSetFlag_8006CFBC, setBit, Fighter_GetPosX / GetPosY, and the
  TARGET_PC UCF 0.84 block: ftUcf_PadBufferPush, ftUcf_PadRaw, ftUcf_FlickCounter,
  ftUcf_RimUnits, ftUcf_FlickCount (static `ftUcf_Pad[4]`).
- ft/ftanim.c (4, `src/ftanim.rs`): scale_inline, get_part_joint, ftAnim_8007077C, ftAnim_80070FB4.
- ft/ftcoll.c (4, `src/ftcoll.rs`): ftColl_800765E0 (statics `dmg_log0_idx/1_idx`), getEnvDmg,
  inlineB1, getUnkVal.
- ft/ftcommon.c (3, appended to `src/ftcommon3.rs`): ftCo_GetLStickAngle, ftCo_GetCStickAngle,
  ftCommon_8007D9D4, through group 1's `melee_runtime::lbtrigf::atan2f`; the C side links the real
  lbtrigf.c atan2f + TARGET_PC atanf (cut verbatim), so the test checks the composition too.
- Test: one fighter with random flags / sticks / x8B0 slots, random statics (spawn counter near
  wrap, pad rings with out-of-range idx), 50 ops x 200 = 10,000 lines (return bits, hash of every
  written field and static, command-pointer delta). Floats include +-0, NaN, random bit patterns.
- Fused sites: ftUcf_RimUnits' f64 `|v| * 80.0 - 1e-4` (contracted to fma by the shipping build;
  ported as `mul_add`). Not discriminated and cannot be: an f32 times 80 is exact in f64, so fused
  and unfused agree. No other a*b+c in the batch.
- Mutation checks that fail: a skip count (op 10), `> 1936` -> `>=` (op 38; the driver forces
  raw-y deltas of 43..45 in half the cases), the flick counter increment / the -0.609375 test /
  the timer `<= 1` (op 38), RimUnits' `+ 2` (op 37), getEnvDmg's `return 1` (op 44),
  scale_inline's multiply-by-reciprocal vs divide (op 39), the spawn counter's wrap to 1 (op 29),
  inlineB1's `<` (op 45), the throw-flag bit (op 25), x8B0 x11 (op 41), x221A_b1 (op 30).
  Not discriminated: `ABS(x)` vs `x.abs()` in the angle getters: atan2f gives the same result for
  x = +0 and -0 here (only a negative-sign NaN would differ; ABS keeps it, abs() clears it).

## Round 10 (2026-10-21, night 3): `src/plbonus2.rs` + four stubs replaced
Recount first: 576 ported before this round (575 verified: all but `ftCommon_8007E2D0`), of
3,002 definitions (the round-9 rule; a quick re-scan with a slightly different regex finds 2,989,
the difference is in what counts as a definition line, not in the ported set).

Stubs replaced by ports that now exist elsewhere (wiring test: `tests/hooks_items.rs`
`round10_hooks_default_to_ports`): `IT_80284FC4` / `IT_80285024` -> group 5's
`melee_items::itkinds5` (itsword.c, verified there); `HSD_JOBJ_GET_TRANSLATION` /
`HSD_JOBJ_MTX_IS_DIRTY` -> `melee_types::inline` (the jobj.h inlines). Also two group-3 hooks now
default to this round's ports: `PL_8003DFF4`, `PL_8003EA08`. Checked and not replaceable: the
rest of the TODO hooks have only c2r `raw/` translations elsewhere (unverified), or a port that
is a method on an instance (HSD_GObj / Random / map lines / JObj tree) with no global to default to.

24 C definitions of src/melee/pl/plbonuslib.c ported, all verified against the C cut verbatim
(the `plbonus` cref grew a `pl_8003DF44`..`fn_8003EE2C` cut): pl_8003DF44, pl_8003DFF4,
pl_8003E058, pl_8003E0E8, pl_8003E114, pl_8003E150, pl_8003E17C, pl_8003E2CC, pl_8003E334,
pl_8003E39C, pl_8003E420, match_item_kind (static inline), pl_8003E4A4, pl_8003E854,
pl_8003E978, fn_8003E998, pl_8003EA08, pl_8003EA40, pl_8003EA74, pl_8003EAAC, pl_8003EB30,
pl_8003EC30, pl_8003EC9C, pl_8003ED0C.
- Test: tests/plbonus_ref.rs, now 104 ops x 50 steps x 300 cases = 15,000 lines (was 81 ops);
  the slot hash gained every field these write (x674/x710/x7AC, xC60..xCA8, xCD8, xD08, xD10,
  xD44, xD54, xD6C, Struct2074.x207C.y). New tracing hooks: IT_8026B7E8 / B774 / B7BC / B960 /
  B7CC / IT_GET_OWNER (default to melee_items::itlib), FTLIB_80086960 / 80086BE0 / 800874BC,
  FT_800898A8 (default to group 3's ports), IT_8026C1D4 (TODO(group5 it), panics unset).
  The ftLib_80087300 stub now returns 0..6 (a slot or 6), since pl_8003DF44 indexes with it; the
  C's `__assert` stub now prints and exits instead of spinning.
- Fused site: pl_8003ED0C `xCA0 * (n - 1) + arg2` (fmadds). **Discriminated** (un-fusing fails
  case 44 step 47 op 103).
- Mutation checks that fail: `arg5 - 0x5C <= 1U` (op 100), 0x34 test (op 100), xC78 accumulate
  (op 100), the x138 reset in pl_8003E4A4 (op 92), `it_8026B7E8 == 1` (op 87), xD6C store in
  pl_8003E854 (op 93), x207C.y store (op 94), xD54 store (op 81), case 3 in pl_8003EC30 (op 101),
  xD10 reset (op 102). Not discriminated: `<=` vs `<` against `x180` in pl_8003E854 (equality
  with a random float never happens).
- C quirks: pl_8003E4A4 with a kind match_item_kind does not map gives -1, which passes the C's
  signed `< Pl_ItemLog_Terminate` and stores to `seen[-1]` (it hung the oracle by clobbering the
  loop counter); the port skips it, the test only feeds mapped kinds. `Player_GetEntityAtIndex`
  with index 2 reads past `player_entity[2]` in C; the port panics, the test drives index 0/1.
- Not ported in that range: pl_8003E70C / pl_8003E7D4 (index the 8-entry `x598` with item kinds
  161..190, i.e. into neighbouring fields; needs a decision on reproducing that overflow).

## Round 11 (2026-10-21, night 3): `src/plbonus3.rs`
Recount first: 600 ported (599 verified, all but `ftCommon_8007E2D0`) of 3,002, as left by round
10 (commit 02d4fb29d; no group-3 commit since). An untracked `src/ftcpu.rs` (ftCo_800A101C /
ftCo_800B9704) and its `pub mod ftcpu;` line in lib.rs appeared in the worktree from another agent;
it has no C reference test, is not counted here and was not committed by this round.

Stubs: re-checked every `TODO(groupN)` hook against the workspace. Still none replaceable: the
ones with a port elsewhere (`mpLineGetNext/Prev` in melee_world::mp::lines, `HSD_Randi` in
melee_world::gr) are methods on an instance (map lines / stage RNG state), not globals a hook can
default to; the rest have no port outside c2r/raw. Five group-3 hooks now default to this round's
ports: `PL_80038788`, `PL_80038824`, `PL_80038898`, `PL_80039418`, `PL_800386D8` (they were
`TODO(group3 plbonus)` panics; plbonus_ref still overrides them with tracing stubs). New needed hook:
`GM_DECISION_GET_TYPE` (TODO(group6 gm) gmDecisionGetType; only the HSD_ASSERTs read it).

15 C definitions of src/melee/pl/plbonus.c ported, all verified against the C cut verbatim
(`gen/cref/plbonusc_ref.c`, run.py cuts `setFlag`..`pl_8003891C` and `pl_80038F10`..`pl_80039450`):
setFlag (static), setPointValue (static inline), pl_800386D8, pl_800386E8, fn_80038700,
pl_80038788, pl_80038824, pl_80038898, pl_80038914, pl_80038F10, fn_80038FB8, pl_8003906C,
pl_80039238, pl_80039418, resetBonuses (static inline).
- Test: tests/plbonusc_ref.rs, 300 cases x 50 steps = 15,000 lines (15 op kinds, all exercised):
  return value, both out-params, trace word (gm stubs), hash of all 6 slots' x904[215], xCAC,
  xD50/54/60/64/6C. Frame count drawn from {0, -2, 1..40, random} so both pl_Verify branches and
  both `<=`/`%` arms run.
- HSD_ASSERT is live in the C (debug.h -> `__assert`); the port panics in the same places.
- No fused site (no a*b+c). Mutation checks that fail: pl_80038898's `== 0` (op 5), the xCAC reset in
  pl_80038F10 (op 7), `< arg4` -> `<=` and `>= arg4` -> `>` in pl_8003906C (op 9), fn_80038700's
  `<=` (op 2), the `!= -2` test (op 10). Not discriminated: `arg7 - *arg4 < arg6` vs `<=` in
  pl_80039238 (equal floats never occur on these inputs).
- Workspace: `cargo build --workspace` clean; `cargo test --workspace` green except
  melee_chara chars_ref, which failed to link (LNK1104, the exe locked by a concurrent agent's run),
  not a test failure.
- Not ported from plbonus.c yet: pl_8003891C (reads an uninitialized local in the decomp),
  pl_80039450 and the fn_80039618..fn_8003D2EC bonus evaluators (calls into ftLib/it/gm).

## Round 12 (2026-10-21, night 3): `src/ft084e.rs`
Recount first: 615 ported (614 verified, all but `ftCommon_8007E2D0`) of 3,002, as left by round
11 (commit b38dd4848; no group-3 commit since). A name scan of the 216 TUs against the hand-port
sources + this doc agrees (611 exact names + 76 suffix matches, the extra being name collisions).
The untracked `src/ftcpu.rs` (ftCo_800A101C / ftCo_800B9704, another agent's, no C reference
test) and its `pub mod ftcpu;` line are still in the worktree; not counted, not committed.

Stubs: re-checked every `TODO(groupN)` hook. Still none replaceable by a verified global port:
`HSD_JObjSetTranslate/SetScale` exist in melee_types::inline only with a `JObjDirty` handle
(HSD_JObjSetMtxDirtySub is not a global port), `mpLineGetNext/Prev` and `HSD_Randi` are instance
methods (melee_world), lbAudioAx / Camera / ifStatus / gm / HSD_GObjFree have no port outside
c2r/raw. New global: `hooks::FIGHTER_804D6520` (bunny-hood multipliers, PlCo `pData[13]`;
TODO(move-to-core) with p_ftCommonData).

19 C definitions ported, all verified against the C cut verbatim (`gen/cref/ft084e_ref.c`):
all of src/melee/ft/ft_084E.c (ft_80084E1C, ft_80084EEC, ft_80084F3C, ft_80084FA8,
ft_80085004, ft_80085030, ft_80085088, ft_800850B4, ft_800850E0, ft_80085134, ft_80085154,
ft_800851C0, ft_800851D0, ft_80085204), ft_081B.c's ft_80084CB0 and ft_80084DB0, ft_08A1.c's
ft_8008A1FC, ft_8008A244 and its static inline ft_8008A244_inline.
- Test: tests/ft084e_ref.rs, 19 ops x 400 random fighters = 7,600 lines: gr_vel, xE4, self_vel,
  x74_self_accel, fall_fast, lstick timer, sfx trace, the ECB box, return value, the ft_8008A2BC
  call count (C stub / hook recorder). The cut includes the ftcommon.c helpers they call
  (CalcGroundAccel_Deaccel..UnlockECB) and MSL sinf/cosf from gekko_fp.c (Rust:
  melee_runtime::gekko).
- Fused sites (fma_sites.py): ft_80085030 `z * dir - gr_vel`, ft_80085154 `a*b - c*d` and
  `a*b + c*d` (left product fused). All three **discriminated** (un-fusing fails op 3 / op 10;
  fusing the right product instead also fails op 10).
- Mutation checks that fail: the bunny-hood `x14` factor (op 12), `v < 0` abs in ft_80084FA8
  (op 3), `< 0` -> `<=` in both stick tests (op 16, 17), `<` -> `<=` against walk_stick_threshold.
- ft_8008A244 calls `ft_8008A2BC` (Wait entry) through the existing `TODO(group3 ft_08A1)` hook;
  unset it panics.
- Workspace: `cargo build --workspace` clean; `cargo test --workspace --exclude melee_chara` green
  (one transient doctest "can't find crate melee_items" during a concurrent rebuild, green on
  rerun). melee_chara does not compile at the moment: group 4's uncommitted `src/chars/r11`
  work in progress (missing ftDonkey/ftFox/ftKoopa functions), not this round's change.

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
  ftcommon.c and ftlib.c (player.c's accessors are done; port round 2), which are mostly field logic.
