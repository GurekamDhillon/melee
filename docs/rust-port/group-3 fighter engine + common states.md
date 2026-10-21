# Group 3: fighter engine + common states (`rust/melee_fighter`)

Scope: `src/melee/ft/*.c` (67 TUs), `src/melee/ft/kinds/ftCommon/*.c` (142), `src/melee/pl/*.c`
(7): 216 TUs, ~76k lines, about 2,820 function definitions (regex count over the C).
State as of 2026-10-21.

## What exists

| layer | where | status |
|---|---|---|
| Native types + GC offsets | `src/types/generated.rs` (from `gen/gen_types.py`) | 569 `#[repr(C)]` records, all built |
| Mechanical translation of all 216 TUs | `src/c2r/` (from `gen/fix_raw.py`, feature `c2r`) | compiles (`--features c2r`), never run |
| Hand ports, checked bit-for-bit against the C | `src/neutral_hash.rs`, `src/ftcommon.rs`, `src/ftcommon2.rs`, `src/stale.rs`, `src/player.rs`, `src/ftlib.rs`, `src/compat.rs`, `src/hooks.rs` | 336 C functions, tests pass |
| PowerPC `.s` files | none belong to this group (inventory §3) | nothing to do |

Functions: 336 of ~2,820 (about 11.9%; 57 night 1, 39 in port round 1, 144 in port round 2, 96 in port round 3; 335 verified) are hand-ported and verified against the C. All ~2,820 are
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
