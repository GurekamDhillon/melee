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

## Round 4 (2026-10-21)

### What was ported
110 more character functions in 48 TUs, the next level up from round 3. The list is
`gen/ported_r4.txt`; the code is `src/chars/r4/<Char>.rs`, path `melee_chara::chars::r4::<Char>::<tu>::<fn>`,
signature `fn(gobj: *mut HSD_GObj)` like rounds 2-3. Kinds: Anim/Coll callbacks that branch on a
field (`if (fp->motion_id == ftMh_MS_Wait1_0 || ...)`, `if (fp->cmd_vars[0] != 0)`, Kirby
`u.kb.x64`), state Enter / SetVars functions that copy `dat_attrs` values into motion vars
(`ftKb_SetNsFlashAttr`, Ness PK Flash, ...), callback setters that store new callbacks, and
`else if` chains.

How: `gen/port_r4.py` reuses port_r3.py and widens its C subset:
- `if` conditions `fp->a.b OP <const>` (== != < > <= >=), `fp->a.b`, `!fp->a.b`, joined with
  `&&` / `||` / parentheses, mixed with round 2's call conditions. Each comparison follows C's
  usual arithmetic conversions, decided from the types clang reports for the TU: int-or-narrower
  vs int compares as i32, anything with `unsigned int` as u32 (field converted `as u32`, as C
  does), f32 vs float as f32, f32 vs double as f64, f32 vs int as f32 (constant converted).
  The constant's bits come from clang in its own type. Pointer fields `== / != NULL` -> `is_null()`;
- `else if`;
- `fp->arr[<int>]` constant indices (`cmd_vars[0]`);
- `T* da = fp->dat_attrs;` / `getFtSpecialAttrs(fp)` locals, read only: `da->x` becomes
  `(*((*fp).dat_attrs as *const T)).x` (writes through `da` are rejected);
- the `getFighter(gobj)` / `Fighter* ft = GET_FIGHTER(gobj)` / `Fighter* fp; fp = GET_FIGHTER(gobj);`
  spellings; struct copies `fp->a = fp->b;` (Vec3) when rustc accepts the types;
- calls into round-3 ports and this round's (fixpoint), `X_cb = &fn;`.
Bitfields only where the test compares them by value: `x2218_b4` was added to that list
(14 now; chref_main.c and chars_ref.rs). rustc-rejected functions are dropped and the scan repeated
(45 dropped in all, mostly enum-typed fields, anonymous member names, `mv` union bitfields).

Stored callbacks: port_r4 rewrites `src/chars/cb.rs` over rounds 3+4 (port_r3.write_cb) and lists
the 28 new keys in `gen/callbacks_r4.txt` (`callbacks_r3.txt` unchanged; chref.py reads both).
55 entries now, all TODO(group4) panics (none of their targets is a round-4 port).

### FMA
`tools/rust-port/fma_sites.py --func <the 110>` over the 48 TUs: 0 fused sites in all 110 (no float
multiplies; float work is compares and copies).

### Verification
Same oracle (`gen/chref.py`), now reading `ported_r4.txt` and both callback lists; round-4
functions run 256 random fighters each. `tests/data/chars.txt` regenerated (the 14th bitfield
changes the stream for all rounds).
- Result: **951/951 equal** (153 + 554 + 134 + 110).
- Negative check: changing a motion id in `ftCh_Init_80156018`, `== 0` -> `!= 0` in
  `ftKb_SpecialAirS_Enter` and swapping one `dat_attrs` field in `ftKb_SetNsFlashAttr` made those
  fail (the test reported 4 functions differing; my output filter showed only those three, so the fourth was not identified).
- Not proven: branch coverage. 256 random fighters (motion_id biased to 330-630, fields often 0)
  reach most `== 0` / motion-id branches but nothing guarantees every branch of every function
  is taken; the stubbed group-3 calls and the 55 unported callback targets.

### Counts
- Ported this round: 110 (all verified). Total in the crate: 964 of ~4340 (22.2%).
- Stubs added: 28 callback entries in `chars::cb` (TODO(group4)); no new group-3 stub.

### Next round
- The biggest blockers in the remaining scan: group-3 calls not stubbed yet (`ft_80082C74`,
  `ft_8008403C`, `ftCommon_8007E2F4`, `ftParts_80074A4C`, `Fighter_OnItemVisible/Invisible`,
  `Fighter_OnKnockbackEnter/Exit`, `ft_CheckGroundAndLedge`, `Fighter_ClearCmdVars`;
  ~280 functions), `Player_GetPlayerSlotType` (group 6, 42), `static MotionFlags const mf`
  locals (49), non-constant right-hand sides (arithmetic, 40), `COPY_ATTRS` (26).

## Round 5 (2026-10-21)

### What was ported
191 more character functions in 62 TUs of 29 characters (the list is `gen/ported_r5.txt`; code
`src/chars/r5/<Char>.rs`, path `melee_chara::chars::r5::<Char>::<tu>::<fn>`, signature
`fn(gobj: *mut HSD_GObj)`). Mostly Coll callbacks (`ft_80082C74(gobj, <edge cb>)`,
`ft_8008403C(gobj, <landing cb>)`), OnDeath / OnKnockbackEnter/Exit hooks (`ftParts_80074A4C`,
`Fighter_OnKnockbackEnter`), Anim callbacks ending in `ftCommon_8007D92C`, landing-lag Coll
callbacks (`ftCo_LandingFallSpecial_Enter(gobj, false, da->x)`), and Enter functions using
`Fighter_ClearCmdVars` / `ftCommon_8007E2F4` / `ftAnim_SetAnimRate`.

How: `gen/port_r5.py` = port_r4's translator plus
- 16 new group-3 stubs in `src/stubs.rs` (TODO(group3), panic unset): `ft_80082C74`, `ft_8008403C`,
  `ft_800841B8`, `ft_80084104`, `ftParts_80074A4C`, `ftParts_80074B0C`, `ft_CheckGroundAndLedge`,
  `ftCommon_8007DB24`, `ftCommon_8007D92C`, `ftCo_Wait_IASA`, `ftCommon_8007EBAC`,
  `ftAnim_SetAnimRate`, `ftAnim_800704F0`, `ftCo_LandingFallSpecial_Enter`,
  `Fighter_UnkSetFlag_8006CFBC`, `ftCo_800957F4`. (Group 3 already has ports of 8007D92C,
  8007DB24, 8007EBAC in melee_fighter; they call melee_fighter hooks the chars test does not
  record yet, so they stay stubs here. Switching them is the next cheap step.)
- three calls to group 3's existing ports: `ftCommon_8007E2F4`, `ftCommon_8007E2FC`
  (melee_fighter::ftcommon2), `ftCommon_UpdateFacing` (melee_fighter::ftcommon); the oracle runs
  their C bodies (new cut range in `chref.py` FTCOMMON);
- `Fighter_OnKnockbackEnter/Exit`, `Fighter_ClearCmdVars` (ft/inlines.h) written out in stubs.rs;
- argument kinds: HSD_GObjEvent (function name -> `chars::cb` entry, NULL -> None) and s16/u8
  constants (clang evaluates `(s16)(expr)`).
60 functions were dropped by the rustc-reject loop (mostly int-to-float argument conversions
such as `ftCo_80096900(..., 1, da->x)` and anonymous member names).

Callbacks: `cb.rs` now covers rounds 3-5 (133 entries; 79 new keys in `gen/callbacks_r5.txt`,
including callbacks passed as arguments). 91 are TODO(group4) panics.

### FMA
`fma_sites.py --func <the 191>` over the 62 TUs: 0 fused sites (no float arithmetic).

### Verification
`gen/chref.py` (shipping front end + lower.py), now reading ported_r5 / callbacks_r5. New C
recorders in `chref_main.c` and identical ones in `tests/chars_ref.rs`; a callback argument is
traced as its non-NULL bit + FNV-1a of its `<tu>:<name>` key. Round-5 functions run 256 fighters.
- Result: **1142/1142 equal** (951 + 191). `cargo test --workspace` green (excluding another
  agent's uncommitted `mex_import` crate, whose own tests fail).
- Negative check: changing `ftParts_80074A4C(gobj, 0, 0)` to `(gobj, 0, 1)` in `ftCa_Init_OnDeath`
  made exactly that function fail (1 of 1142).
- Not proven: branch coverage (random fill), the 32 stubbed group-3 calls (16 + 16), the 91
  unported callback targets.

### Counts
- Ported this round: 191 (all verified). Total in the crate: 1155 of ~4340 (26.6%).
- Stubs added: 16 TODO(group3); 79 new callback keys (TODO(group4) where the target is unported).

## Round 6 (2026-10-21)

### What was ported
108 more character functions in 49 TUs (list `gen/ported_r6.txt`; code `src/chars/r6/<Char>.rs`,
path `melee_chara::chars::r6::<Char>::<tu>::<fn>`, signature `fn(gobj: *mut HSD_GObj)`). Mostly the
per-character `OnItemVisible` / `OnItemInvisible` hooks (53), Phys/Coll callbacks calling
`ftCommon_ClampAirDrift`, `ftAnim_ApplyPartAnim`, `ftColl_8007AEF8`, `ftCo_AirCatchHit_Coll`,
OnDeath hooks calling `efLib_DestroyAll`, and Phys callbacks with motion-var countdowns
(`fp->mv.x.y--`).

How: `gen/port_r6.py` = port_r5's translator plus
- 7 new stubs in `src/stubs.rs` (panic unset): TODO(group3) `ftColl_8007AEF8`, `ftCo_AirCatchHit_Coll`,
  `ftAnim_ApplyPartAnim`, `ftAnim_80070C48`, `ftAnim_80070CC4`; TODO(group5) `itIsHeavy`;
  TODO(ef) `efLib_DestroyAll` (no group owns ef/ yet);
- `Fighter_OnItemVisible` / `Fighter_OnItemInvisible` (ft/inlines.h) written out in stubs.rs over
  `itIsHeavy` + `ftAnim_80070C48` / `80070CC4`;
- group 3's existing ports `ftCommon_Fall` and `ftCommon_ClampAirDrift` (melee_fighter::ftcommon;
  the oracle's ftcommon.c cut already contains their C). In practice no `ftCommon_Fall` caller
  survived (all take `da->x` arguments rustc rejected); 13 functions call `ClampAirDrift`;
- statement `fp->f++;` / `fp->f--;` (prefix too) rewritten to `+= 1` / `-= 1` before translation.
62 candidates were dropped by the rustc-reject loop (the same int-to-float / anonymous-member
cases as round 5, plus `ftCommon_Fall(fp, da->x, da->y)` argument conversions).
1 new callback key (`gen/callbacks_r6.txt`); 6 cb.rs entries that panicked TODO(group4) now call
round-6 ports.

### FMA
`fma_sites.py --func <the 108>` over the 49 TUs: 0 fused sites. (`ftCommon_Fall` /
`ClampAirDrift` are group 3's; their C runs in the oracle with shipping contraction.)

### Verification
`gen/chref.py` now reads ported_r6 / callbacks_r6; 7 new C recorders in `chref_main.c` (ids 38-44;
`itIsHeavy` traces its item gobj as NULL / non-NULL and answers from the LCG) and identical ones in
`tests/chars_ref.rs`. Round-6 functions run 256 fighters each.
- Result: **1250/1250 equal** (1142 + 108); the 1142 earlier hashes are unchanged.
- Negative check: `gravityDelay - 1` -> `- 2` in `ftFx_SpecialSStart_Phys` and the flag `1` -> `0`
  in `ftCl_Init_OnItemVisible` made exactly those two fail (2 of 1250).
- Not proven: branch coverage (random fill), the 39 stubbed calls, unported callback targets.

### Counts
- Ported this round: 108 (all verified). Total in the crate: 1263 of ~4340 (29.1%).
- Stubs added: 7 (5 TODO(group3), 1 TODO(group5), 1 TODO(ef)); 1 new callback key.
- Next: typed `da->x` float arguments (int->float conversion in the translator) would take most of
  the 62 rejected; local declarations (`Vec3 v;`, `s32 i;`) and `Player_GetPlayerSlotType(...) == k`
  conditions (85 uses, Master/Crazy Hand) block the largest remaining groups.

## Round 7 (2026-10-21)

### What was ported
37 more character functions in 25 TUs (list `gen/ported_r7.txt`; code `src/chars/r7/<Char>.rs`,
path `melee_chara::chars::r7::<Char>::<tu>::<fn>`, signature `fn(gobj: *mut HSD_GObj)`). Mostly
Up-B / Down-B / Neutral-B `_Enter`s that clear `fp->throw_flags`, `_Anim` / `_Coll` callbacks ending
in `ftCo_80096900(gobj, ..., da->mobility, da->landing_lag)` (FallSpecial) with float attributes, and
Master/Crazy Hand capture callbacks writing `invisible` / `x2220_b3`.

How: `gen/port_r7.py` = port_r6's translator plus
- a `float` call argument that is a Fighter or dat_attrs field becomes `(field as f32)` (C's implicit
  conversion; int/double -> float round to nearest like Rust `as`, f32 -> f32 is a no-op);
- `fp->throw_flags` / `fp->x594_s32` (members of anonymous unions) map to melee_types'
  `anon_2210` / `anon_594`;
- bitfields `x2222_b2`, `x2220_b3`, `x221E_b6` join the by-value set (`KNOWN_BF`, oracle bf 14-16);
- the "nothing new" filter is now "uses a new stub, assigns, or has a float field argument" (the
  384 empty callbacks still stay out);
- emit-time skip reasons go to `port_r7.WHY`, the last cargo stderr to `rust/target/port_r7/cargo_stderr.txt`.
No new stubs. 4 new callback keys (`gen/callbacks_r7.txt`); cb.rs now has 137 entries, 88 still panic TODO(group4).
Still rejected (37 candidates): ~21 bitfield writes/compares outside the by-value set (mostly `da`
bitfields and `x2_b0`-style motion-var bits), 6 non-GObjEvent pointer args, 3 type mismatches,
pointer assignments from calls (`fp->x1A5C = ftBossLib_GetFighterGObj(...)`), and their callers.

### FMA
`fma_sites.py --func <the 37>` over the 25 TUs: 0 fused sites.

### Verification
`gen/chref.py` reads ported_r7 / callbacks_r7. Oracle changes (both sides):
- NBF 14 -> 17 (the three new bitfields). This changes the random stream, so all of
  `tests/data/chars.txt` was regenerated.
- `THROW_WORD`: the 20 ported functions that write the whole `throw_flags` word (closed over
  callers) hash the byte of `throw_flags_b0` as bytes instead of comparing b0 by value. Without
  this a wrong word value was invisible (found by the negative check: `throw_flags = 1` in
  `ftMr_SpecialHi_Enter` passed).
- Result: **1287/1287 equal** (1250 + 37).
- Negative checks: `.x28 as f32` -> `.x28 as i32 as f32` in `ftMs_SpecialHi_Anim` and
  `throw_flags = 0` -> `1` in `ftMr_SpecialHi_Enter` each made exactly that function fail.
- Not proven: branch coverage (random fill), the stubbed calls, unported callback targets.

### Counts
- Ported this round: 37 (all verified). Total in the crate: 1300 of ~4340 (30.0%).
- Stubs added: 0. Callback keys added: 4.
- Next: pointer-valued assignments from calls (`ftBossLib_GetFighterGObj`), dat_attrs bitfields in
  the by-value set, local declarations, `Player_GetPlayerSlotType(...) == k` conditions.

## Round 8 (2026-10-21)

### What was ported
86 more character functions in 55 TUs (list `gen/ported_r8.txt`; code `src/chars/r8/<Char>.rs`,
path `melee_chara::chars::r8::<Char>::<tu>::<fn>`, signature `fn(gobj: *mut HSD_GObj)`): 84 Master /
Crazy Hand `_IASA` callbacks (`if (Player_GetPlayerSlotType(fp->player_id) == Gm_PKind_Human)
ftBossLib_8015BD20(gobj);`) plus `ftPp_SpecialS1_Anim` and `ftSk_SpecialHi_Anim` (bodies that only
failed on a `PAD_STACK(8 * 4)`-style padding statement).

How: `gen/port_r8.py` = port_r7's translator plus
- `PAD_STACK(<expr>);`, `FORCE_PAD_STACK(<expr>);`, `FORCE_PAD_STACK_<n>;` are dropped before translation
  (unused stack padding); a function whose only new feature is that padding passes the "nothing new"
  filter when it emits at least one statement;
- 1 new stub in `src/stubs.rs`: TODO(group3) `Player_GetPlayerSlotType(slot) -> i32` (pl/player.c);
- `ftBossLib_8015BD20` (ft/ftbosslib.c, C body `return;`) ported as a no-op helper in `src/stubs.rs`.
38 candidates were dropped by the rustc-reject loop. No new callback keys (`gen/callbacks_r8.txt` is empty).

### FMA
`fma_sites.py --func <the 86>` over the 55 TUs: 0 fused sites.

### Verification
`gen/chref.py` reads ported_r8 / callbacks_r8. New recorder id 45 `Player_GetPlayerSlotType`
(traces the slot, answers 0 = Human / 1 from the LCG) on both sides; the C oracle defines
`ftBossLib_8015BD20` with its real empty body.
- Result: **1373/1373 equal** (1287 + 86); `tests/data/chars.txt` regenerated.
- Negative checks: `player_id as i32` -> `+ 1` in `ftCh_BackAirplane1_IASA` and `== 0` -> `!= 0` in
  `ftPp_SpecialS1_Anim` each made exactly that function fail. Flipping the Human test in a boss IASA is
  not observable (both branches do nothing visible: the callee is empty), so it passes, correctly.
- Not proven: branch coverage (random fill), the stubbed calls, unported callback targets.

### Counts
- Ported this round: 86 (all verified). Total in the crate: 1386 of ~4340 (31.9%).
- Stubs added: 1 TODO(group3) (+1 ported no-op helper). Callback keys added: 0.
- Next: `T* da = fp->ft_data->ext_attr;` dat-attr locals (37 Master Hand + 31 Crazy Hand), nested
  `{ Fighter* fp = ...; }` blocks (23), local `Vec3` declarations (60), `static u32 const mf` masks (47).

## Night 3, round 1 (= round 9 of the generator, 2026-10-21)

### Recount at the start
- Ported before this round: 1386 = 13 pure helpers + 1373 functions in `gen/ported*.txt` (153 + 554 +
  134 + 110 + 191 + 108 + 37 + 86; list lines minus headers, no duplicates). Matches the night-2 row.
- Total recounted: **4605** function definitions in the 232 `src/melee/ft/kinds/*/*.c` files outside
  ftCommon (top-level `name(...) {` at brace depth 0, comments/preprocessor stripped; 4549 distinct
  names, statics repeat across TUs). The old "~4340" was an estimate. 1386/4605 = 30.1%.

### Stub replacement
None possible. Every one of the 40 `melee_chara::stubs` targets (ft_0877, ftcoll, ftanim,
Fighter_ChangeMotionState, ftParts, efLib_DestroyAll, itIsHeavy, Player_GetPlayerSlotType, ...) exists
elsewhere only in group 3's mechanical `c2r`/`raw` translation (feature `c2r`, never run, not verified),
not as a verified port; the same holds for the 87 distinct TODO(group4) targets in `chars::cb`.
Wiring a stub to unverified c2r code would break the "verified" rule, so they stay.

### What was ported
16 functions (Master Hand 8, Crazy Hand 8; list `gen/ported_r9.txt`, code `src/chars/r9/<Char>.rs`,
path `melee_chara::chars::r9::<Char>::<tu>::<fn>`): `_Init` / motion-enter helpers and Anim/Phys
callbacks that read `T* da = fp->ft_data->ext_attr;` (positions, timers) and call already-stubbed
group-3 functions or earlier rounds' ports.

How: `gen/port_r9.py` = port_r8's translator plus
- `T* x = fp->ft_data->ext_attr;` locals become read-only pseudo fields (round 4's dat_attrs
  mechanism, separate index per type in `tu.ea`), emitted as
  `(*((*(*fp).ft_data).ext_attr as *const melee_types::T)).f`;
- a function using such a local passes the "nothing new" filter.
40 candidates were dropped by the rustc-reject loop (38 the same as round 8's). Of the ~88
ext_attr users, most also call unported boss helpers (`ftBossLib_*`, `ftCh_UpdateBossMotion`,
`ftMh_*` inlines) and stay out. No new stubs, no new callback keys.

### FMA
`fma_sites.py --func '^(<the 16>)$'` over the 10 TUs: 0 fused sites.

### Verification
Oracle (both sides, `gen/chref_main.c` and `tests/chars_ref.rs`): `fp->ft_data` points at a zeroed
`ftData` whose `ext_attr` is a second attrs block (the dat_attrs block reversed, so a port reading
the wrong block differs); the random `ft_data` word is restored before hashing, so the random stream
and all 1373 earlier hashes are unchanged (diff of `tests/data/chars.txt`: 16 lines added only).
- Result: **1389/1389 equal** (1373 + 16).
- Negative checks: `ext_attr` -> `dat_attrs` in `ftCh_Init_80158F34` and `.x20` -> `.x1C` in
  `fn_80156F6C` made exactly those two fail (2 of 1389).
- Not proven: branch coverage (random fill), the stubbed calls.

### Counts
- Ported this round: 16 (all verified; none unverified). Total: 1402 of 4605 (30.4%).
- Unverified: none counted. Stubs replaced: 0 (no verified target exists).
- Next: the boss helpers (`ftBossLib_*`, `ftCh_UpdateBossMotion`, `ftMh` inlines) would open most of
  the remaining ~70 ext_attr users; nested `{ Fighter* fp = ...; }` blocks; local `Vec3`s.

## Night 3, round 2 (= round 10 of the generator, 2026-10-21)

### Recount at the start
- Ported before this round: 1402 = 13 pure helpers + 1389 functions in `gen/ported*.txt` (rounds 1-9,
  no duplicates). Total: 4605 definitions (round 9's recount, unchanged). 1402/4605 = 30.4%.

### Stub replacement
Group 3 has since ported and verified (ftcommon2.rs / ftcommon3.rs, group-fighter.md rounds 7-8)
four of our stub targets: `ftCommon_8007D7FC`, `ftCommon_8007DB24`, `ftCommon_8007D92C`,
`ftCommon_8007EBAC`. They now use `stub_or!` in `src/stubs.rs`: the hook stays (the oracle installs
recorders through it), and an unset hook calls group 3's port instead of panicking. The others (ft_0877
physics/collision, ftanim, Fighter_ChangeMotionState, ftParts, ftCo_Fall_Enter, ft_8008A2BC,
Player_GetPlayerSlotType, efLib/itIsHeavy, ...) still have no verified port outside c2r and stay
panicking stubs. Note: group 3's 8007D7FC/DB24/D92C call group 3's own hooks (mpLib, efLib,
ftCo_Fall_Enter, ft_8008A2BC), some of which still panic until installed.

### What was ported
10 functions (Crazy Hand 3, Master Hand 4, Koopa 1, Seak 2; list `gen/ported_r10.txt`, code
`src/chars/r10/<Char>.rs`). How: `gen/port_r10.py` = port_r9's translator plus semantics-preserving
rewrites of the stripped C body before translation:
- `Fighter* fp = fp = GET_FIGHTER(gobj)` and any `T* x = x = e` initializer -> `T* x = e`;
- a bare nested block `{ Fighter* fp = GET_FIGHTER(gobj); ... }` at depth 0 of a function whose
  top-level fp is the same `GET_FIGHTER(gobj)`: the inner declaration and braces are removed, only
  when no other name declared in the block appears outside it;
- `ftCo_DatAttrs* ca = &fp->co_attrs;` / `= getFtAttrs(fp);` (inlines.h: `return &fp->co_attrs;`)
  -> `fp->co_attrs.` when the local is only used as `ca->`;
- `a = b = ... = 0;` over `fp->` lvalues and a zero literal -> separate `= 0` stores;
- a function changed by a rewrite passes the "nothing new" filter if it emits a statement.
40 candidates were dropped by the rustc-reject loop (mostly round 9's). The rewrites unlock few
functions by themselves: most rewritten bodies still hit locals (`Vec3`, `int`), item calls or
preprocessor blocks (skip-reason histogram via a scan of port_r10.rewrite + Fn7.block).

### FMA
`tools/rust-port/fma_sites.py --func '^(<the 10>)$'` over the 9 TUs: 0 fused sites.

### Verification
`gen/chref.py` reads ported_r10 / callbacks_r10 (module path `chars::r10`). Oracle unchanged.
- Result: **1399/1399 equal** (1389 + 10); `tests/data/chars.txt` regenerated, earlier 1389 hashes
  unchanged. (A first run forgot R10 in chref.py's `N_R1` count, so the 10 functions after round 1's
  block ran 4000 cases instead of 256 and their hashes moved; fixed, both sides had agreed anyway.)
- Negative checks: `self_vel.z = 0` -> `1.0` in `ftCh_FingerBeamEnd_Anim` and msid 361 -> 362 in
  `ftKp_SpecialLw_Enter` made exactly those two fail (2 of 1399).
- Not proven: branch coverage (random fill), the stubbed calls.

### Counts
- Ported this round: 10 (all verified; none unverified). Total: 1412 of 4605 (30.7%).
- Stubs replaced: 4 (fall back to group 3's verified ports). Stubs added: 0. Callback keys added: 1.
- Next: local `Vec3`/`int` declarations, `static u32 const mf` masks, `ftGetFacingDirInt(fp)`
  (inline: `facing_dir < 0 ? -1 : +1`), the boss helpers (`ftBossLib_*`, `ftCh_GrabUnk1_8015BC88`,
  `ftMh_MS_389_80151018`).

## Night 3, round 3 (= round 11 of the generator, 2026-10-21)

### Recount at the start
- Ported before this round: 1412 = 13 pure helpers + 1399 (tu, function) lines in `gen/ported*.txt`
  (153 + 554 + 134 + 110 + 191 + 108 + 37 + 86 + 16 + 10; no duplicate (tu, name) pairs; three names,
  `doAnim0` / `doAnim1` / `setCallbacks`, are distinct statics in different TUs). Total: 4605
  definitions (round 9's recount; no C changed under `ft/kinds`). 1412/4605 = 30.7%, as round 10 said.

### Stub replacement
- `Fighter_UnkSetFlag_8006CFBC` now uses `stub_or!`: group 3 ported and verified it (ft/fighter.c,
  `melee_fighter::fighter::fighter_unk_set_flag_8006cfbc`, group-fighter.md round 10), so an unset hook
  calls that port instead of panicking. 5 stubs fall back to group 3 now.
- Every other `stubs.rs` target checked against the workspace (outside `c2r`/`raw`): none has a
  verified port yet (ft_0877 physics, ftcoll, ftanim, Fighter_ChangeMotionState, ftParts,
  Player_GetPlayerSlotType: group 3's player.rs has GetPlayerState, not GetPlayerSlotType).
- `efLib_PauseAll` / `efLib_ResumeAll` (new `ext__` entries in `chars::cb`, stored by this round's
  `Fighter_SetEffectHitlagCallbacks` expansions) have verified ports in `melee_world::ef::eflib`,
  but those take the effect plink heads as an argument (the GObj list owner is not settled) and
  melee_chara does not depend on melee_world, so the entries still panic (the label says
  TODO(group3), the generator's default for ext targets; the owner is group 2 / ef). Storing the
  pointer is what the ports do and what the oracle checks.

### What was ported
79 functions (Kirby 45, Mars 8, Captain 6, Fox 5, Popo 4, Nana 3, Mewtwo 2, Yoshi 2, Zelda 2,
Pikachu 1, Seak 1; list `gen/ported_r11.txt`, code `src/chars/r11/<Char>.rs`). How: `gen/port_r11.py`
= port_r10's translator plus
- `static u32 const mf = <constant expr>;` locals: declaration removed, uses replaced by
  `((u32) (expr))`, the value evaluated by clang from the TU like every constant argument;
  skipped when the name is address-taken or assigned;
- `ftGetFacingDirInt(fp)` int arguments -> `(if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })`
  (ft/inlines.h; NaN gives +1 on both sides);
- conditions `fp->f & K` / `!(fp->f & K)` (integer field, constant K: held/pressed buttons) ->
  `(((*fp).f as u32) & K) != 0` / `== 0` (C's usual arithmetic conversion to unsigned);
- header inlines expanded in place: `Fighter_SetEffectHitlagCallbacks(fp)` (two hitlag callback
  stores) and Kirby's `ftKb_SpecialN_set_cbs` / `_set_cbs2(gobj)` (death2_cb / take_dmg_cb, in their
  order; only where the caller's top-level fp is `GET_FIGHTER(gobj)`);
- `ftCliffCommon_80081298(gobj)` (ledge-grab check, returns bool) through a new TODO(group3) stub
  (`stubs::FTCLIFFCOMMON_80081298`, recorder id 46 on both sides of the oracle).
46 candidates were dropped by the rustc-reject loop (round 10's 40 + 6). 4 new callback keys
(`callbacks_r11.txt`). The mask rule alone opened few functions: most mask users also have nested
`{ ftKb_DatAttrs* da; ... }` blocks calling ftColl (Kirby copy abilities).

### FMA
`tools/rust-port/fma_sites.py --func '^(<the 79>)$'` over the 18 TUs: 0 fused sites.

### Verification
`gen/chref.py` reads ported_r11 / callbacks_r11 (module path `chars::r11`); chref_main.c and
chars_ref.rs gain the ftCliffCommon_80081298 recorder.
- Result: **1478/1478 equal** (1399 + 79); `tests/data/chars.txt`: 79 lines added, none changed.
- Negative checks: `-1i32` -> `1i32` in `ftFx_SpecialHiHoldAir_Coll`'s facing argument, `== 0` ->
  `!= 0` in a Kirby `held_buttons & HSD_PAD_B` condition, mask 0xC4C508C -> 0xC4C508D in
  `ftKb_SpecialNGw_8010CC6C`: the test failed with 4 of 1478 differing (the 3 mutated functions plus, presumably, an r11 caller of the mutated Kirby function; the name list was not captured because a concurrent group-3 edit then broke the melee_fighter build).
- Not proven: branch coverage (random fill), the stubbed calls (traced by id + args only).

### Counts
- Ported this round: 79 (all verified; none unverified). Total: 1491 of 4605 (32.4%).
- Stubs replaced: 1 (Fighter_UnkSetFlag_8006CFBC -> group 3's port). Stubs added: 1
  (ftCliffCommon_80081298, TODO(group3)). Callback keys added: 4.
- Next: nested `{ ftKb_DatAttrs* da; ... }` blocks calling ftColl (Kirby copy abilities), local
  `Vec3`s, `efSync_Spawn`, the boss helpers.

## Night 4, round 1 (= round 12 of the generator, 2026-10-21)

### Recount at the start
- 1491 = 13 pure helpers + 1478 (tu, function) lines in `gen/ported*.txt` (rounds 1-11; no duplicate
  pairs). Total 4605 definitions (no C under `ft/kinds` changed). 32.4%, as round 11 said.

### Stub replacement
- `ft_80084F3C`, `ft_80084EEC`, `ft_80085134`, `ft_80084DB0`, `ft_80084FA8` now use `stub_or!`: group 3
  ported and verified them (`melee_fighter::ft084e`, group-fighter.md round 12). An unset hook calls
  the port; the oracle still installs recorders. 10 stubs fall back to group 3 now.
- Every other `stubs.rs` target checked: no committed, verified port (the `c2r`/`raw` mechanical
  translations do not count; group 3's `ft0df.rs` with ftParts_80074A4C / ftColl_8007AEF8 is
  uncommitted work in the tree, not used).

### What was ported
101 functions (Fox 21, Donkey 16, Zelda 14, Mewtwo 13, Nana 10, Ness 8, GameWatch 7, Samus 6,
Seak 5, Pikachu 1; list `gen/ported_r12.txt`, code `src/chars/r12/<Char>.rs`). How: `gen/port_r12.py`
= port_r11's translator plus
- the "already ported" filter keyed by (tu, function) instead of by name. Rounds 1-11 skipped
  every static whose name matched a ported function in another TU (ftDonkey's two `setCallbacks`,
  because ftCaptain's and ftMario's were ported), and with them all their callers;
- top-level `Fighter* fp = gobj->user_data;` read as `GET_FIGHTER(gobj)` (same value: ft/inlines.h);
- header `static inline void f(gobj)` helpers whose body is `fp = GET_FIGHTER(gobj)` + declaration-free
  statements (ftSamus_updateDamageDeathCBs, ftSamus_SetAttrx2334, Fighter_UnsetCmdVar0,
  ftFox_SpecialN_CheckLoopInput, ...) expanded in place where the caller's top-level fp is GET_FIGHTER(gobj);
- the "nothing new" filter dropped (the pair-keyed filter already excludes earlier rounds); empty
  bodies are still skipped. 47 candidates dropped by the rustc-reject loop. 9 new callback keys.
`gen/chref.py` reads ported_r12 keyed by (tu, name) (module path `chars::r12`), and `N_R1` now
excludes round 12 (first run counted them as round 1, which gave 101 round-2 functions 4000 cases
instead of 256 and changed their hashes; fixed, `tests/data/chars.txt` then only gained 101 lines).

### FMA
`tools/rust-port/fma_sites.py --func '^(<the 101>)$'` over their 24 TUs: 0 fused sites.

### Verification
- **1579/1579 equal** (1478 + 101), `cargo test -p melee_chara --test chars_ref`.
- Negative check: ftDonkey specialn `setCallbacks` storing ftDk_Init_8010D774 into take_dmg_2_cb
  instead of ftDk_SpecialN_DestroyAllEffects: 14 of 1579 differ (it and its 13 round-12 callers).
- Not proven: branch coverage (random fill), the stubbed calls (traced by id + args only).
- `cargo test --workspace` (debug) green. Under `--release`, melee_world leaves_r2 / textlib_r6 and
  mex_import ppc_oracle fail on NaN sign bits (not this crate; noted for their owners).

### Counts
- Ported this round: 101 (all verified; none unverified). Total: 1592 of 4605 (34.6%).
- Stubs replaced: 5 (ft_084E.c physics -> group 3's ports). Stubs added: 0.
- Next: nested `{ Vec3 ...; }` blocks, `Vec3` locals, the Kirby hat helpers (HSD_JObj), item calls.

## Night 4, round 2 (= round 13 of the generator, 2026-10-21)

### Recount at the start
- 1592 = 13 pure helpers + 1579 (tu, function) lines in `gen/ported*.txt` (rounds 1-12; no duplicate
  pairs). Total 4605 definitions (no C under `ft/kinds` changed). 34.6%, as round 12 said.

### Stub replacement
- `ftParts_80074A4C` and `ftColl_8007AEF8` now use `stub_or!`: group 3 committed and verified them in
  its round 13 (`melee_fighter::ft0df`, group-fighter.md). 12 stubs fall back to group 3 now.
- Every other `stubs.rs` target re-checked (committed, non-`c2r`/`raw` ports): none. `ftAnim_SetAnimRate`
  is only a hook in group 3 too; `ftParts_80074B0C`, ftcoll ground/air, ftanim, ChangeMotionState,
  itIsHeavy, Player_GetPlayerSlotType: no port.
- 9 new `stub_or!` hooks, each defaulting to a verified group-3 port the character code calls:
  `ftColl_8007AEE0`, `ftColl_8007AF10` (ft0df.rs) and `ft_80085004`, `ft_80085088`, `ft_800850B4`,
  `ft_80085154`, `ft_800851C0`, `ft_800851D0`, `ft_80085204` (ft084e.rs). Oracle recorders 47-55
  (`gen/chref_main.c`, `tests/chars_ref.rs`).

### What was ported
12 functions, all `_Phys` callbacks (GameWatch 3, Koopa 3, Mars 3, Kirby 1, Mario 1, Peach 1; list
`gen/ported_r13.txt`, code `src/chars/r13/<Char>.rs`). How: `gen/port_r13.py` = port_r12's translator
with the 9 calls above known and calls into round-12 ports. 48 candidates dropped by the rustc-reject
loop (round 12's 47 + 1). No new callback keys.

### FMA
`tools/rust-port/fma_sites.py --func '^(<the 12>)$'` over their 8 TUs: 0 fused sites.

### Verification
- **1591/1591 equal** (1579 + 12), `cargo test -p melee_chara --test chars_ref`; `tests/data/chars.txt`
  gained 12 lines, none changed.
- Negative check: `st::ft_800850B4` -> `st::ft_80085088` in a round-13 ftKoopa function: 1 of 1591 differ.
- Not proven: branch coverage (random fill), the stubbed calls (traced by id + args only; the 9 new
  hooks run group 3's ports in the game, which group 3 verified separately).
- `cargo build --workspace` clean; `cargo test --workspace --no-fail-fast` green except one run where
  the melee_chara doctest could not find `melee_fighter` (E0463) while another agent was rebuilding;
  `cargo test -p melee_chara --doc` alone passes.

### Counts
- Ported this round: 12 (all verified; none unverified). Total: 1604 of 4605 (34.8%).
- Stubs replaced: 2. Stubs added: 9 (all `stub_or!` to verified group-3 ports).
- Next: nested `{ Vec3 ...; }` blocks, `Vec3` locals, the Kirby hat helpers (HSD_JObj), item calls;
  the cheap "new callee" path is nearly exhausted.
