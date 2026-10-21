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

## Round 2 (2026-10-21)

### What was ported
554 more character functions in 135 TUs of 26 characters (all of `ft/kinds/*` except ftCommon;
ftCLink, ftDrMario and ftSandbag are new this round). They are the thin callbacks one level above
round 1: every one calls group-3 code that is not ported yet, or the ported
`ftCommon_8007D5D4` / `ftCommon_8007D60C`. Kinds, by body:
- 362 Phys/Coll callbacks that are one call: `ft_80084F3C` (166), `ft_80084EEC` (97),
  `ft_80085134` (56), `ft_80084DB0` (24), `ft_80084FA8` (12), `ft_80083F88` (7);
- 72 Anim callbacks `if (!ftAnim_IsFramesRemaining(gobj)) ft_8008A2BC / ftCo_Fall_Enter / ...`;
- 89 motion-state transitions: `Fighter_ChangeMotionState(...)` with constant arguments
  (+ `ftAnim_8006EBA4`), `ftCommon_GroundToAirStateChange` / `AirToGroundStateChange`
  (ftCommon/inlines.h, ported as `stubs::ftCommon_*StateChange`), `8007D60C` + state change;
- 31 Coll callbacks guarding a state change on `ft_80082708` / `ft_80081D0C` / `ft_800827A0`.

The list is `gen/ported_r2.txt`; the code is `src/chars/r2/<Char>.rs`, path
`melee_chara::chars::r2::<Char>::<tu>::<fn>` (C names kept). Signature: `fn(gobj: *mut HSD_GObj)`
(the raw gobj: these pass it on to group 3), `fp` is `(*gobj).user_data` as in ftlib.

How: `gen/port_r2.py` translates a C subset mechanically (GET_FIGHTER, PAD_STACK, calls to the
known group-3 functions with `gobj` / `fp` / `NULL` / constant / `fp->field` arguments, `if/else`
on a call result, `return`). Anything outside the subset is skipped and left for hand porting.
Every constant argument (motion ids, `MotionFlags` statics such as `transition_flags0`,
float literals) is evaluated by clang from the TU itself, so the Rust carries the exact bits with
the C spelling in a comment. The output is checked in and is not hand-edited; the translator is
a porting aid, the check below is what verifies it.

### Stubs (TODO(group3))
`src/stubs.rs`: 16 settable hooks, unset = panic `TODO(group3): <name> is not ported`:
`ft_80084F3C`, `ft_80084EEC`, `ft_80085134`, `ft_80084DB0`, `ft_80084FA8`, `ft_80083F88`,
`ftAnim_IsFramesRemaining`, `ftAnim_8006EBA4`, `Fighter_ChangeMotionState`, `ft_8008A2BC`,
`ftCo_Fall_Enter`, `ft_80081D0C`, `ft_80082708`, `ft_800827A0`, `ftCommon_8007D7FC`,
`ftCo_80096900`. `bool`/`GroundOrAir` returns are `i32`. `ftCommon_8007D5D4/D60C` call group
3's ports (`melee_fighter::ftcommon2`). When group 3 ports one of the 16, its hook default
should become that port.

### FMA
`tools/rust-port/fma_sites.py --func <the 554 names>` over the 135 TUs: no fused site in any
of them (they do no float arithmetic; float arguments are constants or field reads).

### Verification
Same oracle as round 1 (`gen/chref.py`, shipping front end + `lower.py`), extended:
- `gen/chref_main.c` defines the 16 group-3 functions as recorders: each mixes its id and
  every non-pointer argument (a gobj argument as NULL / own / other) into a trace word, and a
  value-returning one answers from an LCG seeded per case. `ftCommon_8007D5D4/D60C` run for real
  (cut verbatim from ftcommon.c with the round-1 helpers); their callees `Geno_GroundEdge` /
  `pl_8003FC44` are recorders too. `tests/chars_ref.rs` installs identical recorders in
  `melee_chara::stubs` and `melee_fighter::hooks`.
- The per-function hash now also covers the trace word. Round-2 functions run 256 random
  fighters each (round 1 keeps 4000).
- Bitfields compared by value: +`is_sub_fighter`, `x2227_b0` (read by `ftCommon_8007D60C`), 10 in all.
  This changed the random stream, so all of `tests/data/chars.txt` was regenerated (round 1 included).
- Result: **707/707 equal** (153 round 1 + 554 round 2). Negative check: changing one motion id
  in `ftMh_MS_365_80153730` and one `== 0` in `ftZd_SpecialHi_Anim` made exactly those two fail.
- What this proves: each callback's own logic (which calls, in which order, with which
  arguments, under which branch outcomes) matches the C. It does not prove anything about the
  stubbed group-3 functions; the 554 functions do nothing useful in a game until those exist.

### Counts
- Ported this round: 554 (all verified). Total in the crate: 720 of ~4340 (16.6%).
- Stubs added: 16 (TODO(group3)).

### Next round
- `gen/port_r2.py` skips: bodies with field writes/arithmetic, calls to round-1 functions,
  callback-pointer setters, attrs reads (`da`), items/effects/jobj. Extending the subset to
  `fp->x = <const or field>` assignments and calls into round-1 ports would take the next
  ~150 (`gen` scan: ~685 candidates call only these stubs + ported code).
- Group 3: porting the 16 stubbed functions (ft_0877 physics steps, ftcoll ground/air,
  ftAnim, Fighter_ChangeMotionState) makes these 554 callbacks live.

## Round 3 (2026-10-21)

### What was ported
134 more character functions in 54 TUs of 21 characters, one level above round 2. The list is
`gen/ported_r3.txt`; the code is `src/chars/r3/<Char>.rs`, path `melee_chara::chars::r3::<Char>::<tu>::<fn>`,
signature `fn(gobj: *mut HSD_GObj)` like round 2. Kinds:
- Coll/Anim callbacks that branch into a round-1/2/3 port (`if (!ft_80082708(gobj)) ftFx_SpecialHiHold_GroundToAir(gobj);`);
- GroundToAir / AirToGround / Enter functions that change state and then set fields or callbacks
  (`fp->accessory4_cb = ftFx_SpecialHi_CreateLaunchGFX; fp->x2223_b4 = true;`);
- callback setters / clearers (`setCallbacks`, `clearCallbacks`, `unsetCallbacks`, `ftPp_set_cbs`, ...);
- small field setters (`ftMewtwo_SpecialHiLost_SetVars`, `ftGw_SpecialHi_ItemRescueSetNULL`, ...).

How: `gen/port_r3.py` reuses port_r2.py and widens its C subset:
- calls to round-1 ports taking only the fighter, to round-2 ports and to functions accepted this
  round (fixpoint over the scan; a static of the caller's TU wins over a same-named function elsewhere);
- `fp->a.b = <constant>;`, `fp->a.b = fp->c.d;` (same scalar C type), `fp->a.b += / -= <constant>;`
  (ints wrap as under -fwrapv; floats only for a `float` constant, so no double promotion);
- `fp->X_cb = NULL;` and `fp->X_cb = <function>;` for Fighter's HSD_GObjEvent fields;
- `fp->flag = true/false;` for Fighter-level one-bit bitfields.
Every field's C type and each constant cast to it is evaluated by clang from the TU (a file-scope
static initialiser, so only real constants pass). The emitted Rust is built; any function rustc
rejects (Rust name differs, e.g. anonymous members, or another field type) is dropped and the
scan repeated (17 dropped). `fp` is a raw `*mut Fighter` in this round's output.

Stored callbacks: `src/chars/cb.rs` (generated) has one `unsafe extern "C"` entry per stored C
function, keyed by its defining TU (`<tu stem>__<name>`, `ext__<name>` outside ft/kinds); it calls
the target's port when there is one, else panics. All 27 targets are unported today: **27
TODO(group4) entries** (GFX/hitlag/damage callbacks of the characters themselves; none is group 3's).

### FMA
`fma_sites.py --func <the 134>` over the 54 TUs: no fused site (no float multiplies).

### Verification
Same oracle (`gen/chref.py`), extended:
- round-3 functions run 256 random fighters each, like round 2;
- bitfields compared by value: +`allow_interrupt`, `invisible`, `reflecting` (13 in all);
  this changed the random stream, so `tests/data/chars.txt` was regenerated for all rounds;
- callbacks: after each call both sides replace every Fighter HSD_GObjEvent field whose value is
  a known callback by FNV-1a of `"<defining tu>:<name>"` (C: per-driver address tables, plus weak
  definitions for callbacks no built TU defines, so each has its own address; Rust: `cb::CALLBACKS`).
  So the test checks which function was stored, not its address.
- Result: **841/841 equal** (153 + 554 + 134). Negative check: changing one float constant in
  `ftCh_Init_80157B58` and storing the wrong Kirby callback in `fn_80109A08` made exactly those two fail.
- What it does not prove: the stubbed group-3 calls and the 27 unported callback targets.

### Counts
- Ported this round: 134 (all verified). Total in the crate: 854 of ~4340 (19.7%).
- Stubs added: 27 callback entries in `chars::cb` (TODO(group4)); no new group-3 stub.

### Next round
- 17 functions port_r3 dropped (anonymous member names such as `ftKb_PrSetCallbacks`,
  `ftSamus_ClearThrowFlagsUnk`) and the 27 callback targets, by hand.
- port_r3 still skips: `da` (dat_attrs) reads, conditions on fields, arithmetic beyond `+=` const,
  item/effect/jobj calls; extending conditions to `fp->field ==/!= const` is the next cheap step.
