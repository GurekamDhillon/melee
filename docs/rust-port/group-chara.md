# Group 4 (fighter characters, `src/melee/ft/kinds/*` except ftCommon): port rounds

Crate `rust/melee_chara`. Night-1 notes (types, blockers, the old generator): `group-4 fighter characters.md`.
This file is the per-round log from round 1 on; where they disagree, this file wins.

## Round 1 (2026-10-21)

### What was ported
153 character functions in 68 TUs of 24 characters, hand-written in `src/chars/<Char>.rs`, one
`pub mod <tu>` per C file, C names kept (`melee_chara::chars::ftYoshi::ftyoshispecials::fn_8012EFF4`).
The list is `gen/ported.txt`. They are the lowest level of the character code: every one calls
nothing, or only the group-3 ftCommon movement helpers already ported in `melee_fighter::ftcommon`
(`ftCommon_Fall`, `FallBasic`, `CalcSelfAccel_*`, `CalcGroundAccel_*`, `ClampSelfVelX`,
`SetSelfMovementFromGroundedMovement`). Kinds:
- ~70 Phys/IASA/Coll callbacks (gravity, drift, friction, motion-var countdowns, Yoshi's Egg Roll
  deal-damage speed bleed `fn_8012EFF4`, Pikachu's `UpdateVel`, Sheik's hitbox reset, ...);
- ~45 `dat_attrs` / Fighter getters and small setters;
- ~25 motion-range checks (G&W item removal, Kirby copy checks, Samus/Link/Nana);
- 4 `UnkDemoCallbacks0` (Kirby, Mario, Luigi, Giga Bowser).

How they were chosen: a scan of every `ft/kinds/*/*.c` function whose callees are all ported
(`GET_FIGHTER` + the ftCommon list), minus callback-pointer setters (need typed fn pointers),
anything touching `gFtDataList`/`ft_data`/jobjs/items/effects, and the four
`*_GetMotionFileString` (string tables). 384 empty callbacks also qualify and were NOT ported
(no behaviour to port; they need the MotionState table typing first).

Signatures: `HSD_GObj*` used only for `GET_FIGHTER` becomes `&Fighter` / `&mut Fighter` (group 3's
convention); `ftSs_SpecialLw_80129158` / `ftSs_SpecialN_801291A8` test the gobj for NULL and take
`Option<&Fighter>`. `fp->dat_attrs` is read through `chars::da::<T>()` with the C's cast type.
`ftKb_SpecialN_800F597C` returns `i32` (`flags & 0x100`; MSL `bool` is int, not 0/1).

### FMA
`tools/rust-port/fma_sites.py` over all 68 TUs, filtered to the 153 functions: one fused site,
`ftKb_SpecialN_800F5B5C` (`output->x += off_x * facing_dir` -> `fmadds(off_x, facing_dir, x)`).
**This one is proven by a test**: with the unfused `+=` the oracle test fails on that function.

### Verification (bit-exact vs the C, shipping contraction)
`python rust/melee_chara/gen/chref.py` (Windows, LLVM clang) builds the oracle:
- a driver per TU `#include`s the game's own .c (so static helpers are callable) and dispatches
  to the listed functions; the ftCommon helpers are cut verbatim from ftcommon.c as group 3 does;
- every TU goes through the shipping front end (`-O2 -Xclang -disable-llvm-passes`, default
  `-ffp-contract=on`) and `melee_runtime/oracle/lower.py` (gwtool's fmuladd/fptosi lowering),
  then native x86-64; unresolved externs are left unresolved (`lld /FORCE:UNRESOLVED`, none is reached);
- `ScriptGame_CapsMaxJumps` (PC caps add-on) is the vanilla attribute; Rust: `chars::caps_max_jumps`
  hook, TODO(group6).
- `gen/chref_main.c` runs each function on 4000 random fighters (whole Fighter filled word by word
  with small ints / zeros / floats incl. +-0, motion_id biased to the character range 330-630,
  a 4 KiB random dat_attrs block, random ftCommonData, random args/out-params) and prints one
  FNV-1a hash per function over the whole Fighter, the attrs block, the common data, the return
  bits and out-params -> `tests/data/chars.txt` (computed from C source only, no disc data).
- `tests/chars_ref.rs` repeats the fill and compares. The header line also checks
  `sizeof(Fighter)` (11560) and `sizeof(ftCommonData)` against the Rust native layout.
- Result: **153/153 equal.** Debugging: `chref.py -v` writes per-case lines to
  `rust/target/chref/chars_verbose.txt`; `CHARS_VERBOSE=<index> cargo test -p melee_chara --test chars_ref -- --nocapture`
  prints the same format.
- Bitfields: native C packs them LSB-first, melee_types keeps them in GC order, so the same byte
  means different values. The 8 bitfields these functions (and the ftCommon helpers) read or
  write (`fall_fast`, `x2219_b5`, `x221F_b3`, `x2223_b4`, `throw_flags_b0`, `x2071_b6`,
  `mv.ca.specialhi.x2_b0/b1`) are set by value on both sides, compared by value, and their bytes
  zeroed before the byte hash. `mv.ca.specialhi.x2_*` shares its byte with every other
  character's motion vars (the `mv` union), so it is only treated as bitfields for the Captain
  function.
- Not covered by the vectors: values where the random fill never lands (e.g. motion ids outside
  0-999); `ftFx_SpecialS_CopyGhostPosIndexed` is only called with index 0-3 (the C does not
  bounds-check; Rust panics past 3).

### Counts
- Functions ported this round: 153 (all verified). Total hand-ported in the crate: 166 of ~4340
  (153 + the 13 night-1 pure helpers), about 3.8%.
- Stubs added: 1 (`chars::CAPS_MAX_JUMPS`, TODO(group6)).

### Next round
- The next tier needs, in order: `ftAnim_IsFramesRemaining` / `ftAnim_*`, `Fighter_ChangeMotionState`
  and `ftCommon_8007E2D0` (group 3), `ftCommon_ApplyFrictionAir` etc. Most Anim/Coll callbacks
  are one call to one of those plus a state change; stub them in this crate with TODO(group3)
  or wait for group 3.
- Typed `HSD_GObjEvent` callbacks (the ~40 callback setters skipped here) and the MotionState
  tables, which would also take the 384 empty callbacks.
