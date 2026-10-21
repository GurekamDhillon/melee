# Group 6: game modes, menus, HUD, trophies, lb leftovers (`rust/melee_modes`)

## Round 1 (night 2, 2026-10-21): leaf functions

**Picking the batch.** I ran libclang (melee_types' `gen_types.py` setup, native flags) over every gm, mn,
if, ty and lb TU in `docs/rust-port/game_tus.txt`. It lists each function defined in the TU whose body has
no call and no reference to a file-scope variable. That gives 346 functions: 91 take only scalar or
by-value-struct arguments, and 255 take a pointer. These have no callees, so they are the lowest level.
Round 1 ports 72 of the 91 scalar ones.

**Ported: 72 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle`, 64,787 vectors):

| module | C TUs | functions |
|---|---|---|
| `gm.rs` | gmevent, gmmain_lib, gmfrontend, gmregcommon, gmresult, gmresultplayer, gmtoulib, gmtou_0, gmvs, gm_1601, gm_16F1, gm_17AD, gm_17C0, gm_1A3F | 32 |
| `mn.rs` | mncharsel, mncount, mndiagram2, mnmain, mnname, mnvibration | 17 |
| `hud.rs` (if + ty) | ifnametag, ifstock, soundtest, textlib, textlib_1, tydisplay | 8 |
| `lb.rs` | lbaudio_ax, lbcardnew, lbcollision, lbdvd, lbsnap, lbtime, lb_00CE, lb_00F9 | 15 (lb leftovers group 1 did not port; TODO(group1) move into melee_runtime) |

Each Rust function keeps the C name. `GXColor` is defined in `melee_modes` with a `gc_layout!` table, because
melee_types does not generate it (TODO(group0)). Nothing calls into another group yet, so there are no
stubs.

**Oracle** (`rust/melee_modes/oracle/`):
- `gen_oracle.py` cuts each function verbatim out of its TU using libclang extents. It renames each one to
  `o_<name>` with internal linkage, because `expf` and `powi` clash with MSL. It wraps each TU's cut in a
  file that has that TU's `.h` includes and its one-line `#define`s.
- It also writes a stdin dispatcher and the input vectors. Inputs are exhaustive for u8/s8, edge values
  plus sweeps plus seeded random values for ints and floats, and at most 1,500 rows per function.
- `build.sh` compiles each file with the shipping build's front end: clang, contraction on,
  `-O2 -Xclang -disable-llvm-passes`. It then runs `melee_runtime/oracle/lower.py` for gwtool's fmadds
  lowering and float->int saturation, links, runs, and writes `tests/data/vectors.txt` and `sigs.txt`.
- Rerun it with `bash rust/melee_modes/oracle/build.sh` from Git Bash on Windows (LLVM, Python libclang).

**FMA.** `tools/rust-port/fma_sites.py` finds three fused sites in this batch:
- `fe_ease`: `fmadds(-(u*u), u, 1)`
- `fe_row_y`: `fmadds(34, slot, 114)`
- `_tyDisplay_80318CB4_calc_dist_sq`: `fmadds(dx, dx, dz*dz)`

All other ported functions have 0 sites. Here the vectors DO tell fused and unfused apart. With the unfused
form, the oracle test fails 20/1500 rows for `fe_ease` and 64/1500 for `calc_dist_sq`. `fe_row_y` is exact
either way for these inputs, so it is fused by the IR only.

**Limits, stated honestly:**
- **NaN results are compared as "any NaN".** The natively compiled C and rustc pick different mulsd operand
  orders, so `fe_ease(NaN)` gives `0x7fc00000` in C and `0xffc00000` in Rust. Neither is necessarily the
  Gekko's NaN. Every non-NaN bit is compared.
- **Domains are restricted where the C is undefined or never returns.** lower.py does not model gwtool's
  integer rules, so these were kept off the vectors:
  - `bitset64_mask`: bit < 64.
  - `selkind_mask`: kind < 32. The Rust uses `cabi::shl` for the rest.
  - `ifStock_802F89F8_divisor` / `ifStock_802F89F8_digit`: position <= 12.
  - `mn_8022C010`: menu_kind <= 33. Higher kinds fall off the end of the C function, and the Rust returns 0
    there, which is unverified.
  - `expf`: |x| <= 12, plus 20, -40 and +-inf. Its series never terminates for 68 multiples of 1/16 with
    12.625 <= |x| <= 14.6875, or for NaN, in the C as in the Rust.

**Not ported from the 91 scalar leaves:**
- 10 empty `void` functions (e.g. `gm_Mode_Vs_OnUnload`, `un_802FD704`).
- `gm_1832_sdata2_order` / `gm_1884_sdata2_order`, which only order float literals.
- `fn_800269AC_delay` and `inline_pad`, which have no effect.
- `ifAll_GetArchive`, which returns a static's address.
- `fn_8024FC48_inline`, an int-to-pointer cast.
- `gm_801891F4_GetTickRate`: `OS_TIMER_CLOCK` reads `*(u32*)0x800000F8`, so it is not pure, though the
  scan missed this.
- `lbvector_sin/cos` and `lbTrigf_Fmadds/Fnmsubs`, which group 1 already ported.

## Round 2 (night 2, 2026-10-21): callers of round 1 and const-table readers

**Picking the batch.** I re-ran the libclang scan over the same gm, mn, if, ty and lb TUs. This time
it kept every function with scalar arguments and a scalar return whose calls all go to already-ported
functions, and whose file-scope reads are all `const` tables. It repeated that until nothing new
qualified. That gives 31 functions. 5 belong to group 1 (lbtrigf, lbvector), 1 is
`gm_801891F4_GetTickRate` (not pure), and 2 are tydisplay functions that return a string's address
cast to `s32`, which has no native meaning. Round 2 ports the other 26.

**Ported: 26 functions, all checked bit for bit against the C.** `cargo test -p melee_modes --test oracle`
now covers 98 functions with 80,030 vectors.

| module | C TU | functions |
|---|---|---|
| `gm.rs` | gm_1601 | 20: `gm_801601C4_inner/_noinline`, `gm_80160244_inner/_noinline`, `gm_SelKindToUnlockIndex`, `gm_CKindToUnlockIndex`, `get_unlockable_selkind_by_bit_index`, `gm_GetCKindByUnlockIndex`, `fn_80160710`, `gm_GetChallengerStKind`, `fn_801607A8`, `fn_801607F4`, `gm_80160968`, `gm_SelKindToCKind`, `gm_CKindToSelKind`, `fn_8016419C`, `fn_801641B4`, `gm_801641CC`, `getStageUnlockIndex`, `getStageUnlockNotifyId` |
| `gm.rs` | gmmain_lib | 1: `selkind_bit` |
| `mn.rs` | mnmain | 2: `mn_GetDigitAt`, `mn_GetDigitCount` (over `powi`) |
| `lb.rs` | lb_00CE, lbcardnew | 3: `powf` (over `expf`), `lb_8000D008` (over group 1's `melee_runtime::lbtrigf::atanf`), `convertHsdResult` |

- The gm_1601.static.h tables these functions read are in `gm.rs` as `pub static`s:
  `LBL_803B7808`, `LBL_803B7864`, `SELKIND_TO_CKIND_MAP`, `CKIND_TO_SELKIND_MAP`, `LBL_803B78C8`,
  `LBL_803B790C` and `LBL_803B7930`.
- `UnlockableCharacterData` has a `gc_layout!` table (size 6).
- The values are typed in from the header and confirmed by the oracle.
- Watch the comments in `ckind_to_selkind_map`: they number the selkinds as if Zelda and Sheik each had
  one, but the real enum (mn/types.h) puts FALCO at 0x13.
- `gm_CKindToSelKind` is the TARGET_PC `GM_CK_TO_SELKIND`: an m-ex kind (0x22..0x7F) reads Popo's row.
- **FMA:** `fma_sites.py` finds no fused site in any of the 26.
- **No stubs were needed.** The one other-group callee, `atanf`, was already ported by group 1, so it is
  called directly.

**Oracle changes** (`gen_oracle.py`):
- Every cut function is now renamed `o_<name>` with external linkage, and calls inside the cuts are
  renamed too, so callers reach the cut callee. Prototypes, with enums as `int`, are emitted for every
  cut function a file calls.
- `HELPERS` cuts other-group callees without testing them: lbtrigf's `atanf`, `lbTrigf_Fmadds` and
  `lbTrigf_Fnmsubs`.
- File-scope variables defined in the `.c` itself (e.g. `atanf_lookup`) are cut along with the
  functions.
- `EXTRA` adds macros that sit under `#if defined(TARGET_PC)`, which the one-line `#define` scan
  cannot pick up: `GM_IS_MEX_CK`, `GM_CK_TO_SELKIND`, and lbtrigf's `BITWISE`.
- Because the random draws now come in a different order, round 1's sampled vectors changed. All 72
  round-1 functions still pass on the new set.

**Domains restricted** (the C reads out of bounds or never returns):
- Table indices stay inside their tables: `gm_80160968` (u8) < 9; `gm_SelKindToCKind` < 0x1C;
  `gm_CKindToSelKind` / `gm_CKindToUnlockIndex` skip 0x21 and >= 0x80 (after the u8 truncation);
  `fn_8016419C/801641B4` 1..9; `gm_801641CC` < 30.
- `selkind_bit`: kind < 32.
- `mn_GetDigitAt`: place 0..31, because `powi` is 0 for a negative place, which would divide by zero.
- `powf`: x > 0 (log-uniform over e^±4.5) with |y ln x| <= 12, plus 0, -0 and a NaN y at x = 0. Like
  `expf`, its series need not terminate.
- Out of these domains the Rust panics on a table index where the C reads past the table.

## Round 3 (night 2, 2026-10-21): leaf functions over game structs

**Picking the batch.** I re-ran the libclang scan over the gm, mn, if, ty and lb TUs for functions that
take a pointer, have no call and no file-scope variable, and return a non-pointer. I dropped any whose
body dereferences a pointer it did not get as an argument (`p->a->b`, pointer locals). That leaves 158.
Round 3 ports 123 of them.

**Ported: 123 functions, all checked bit for bit against the C.** `cargo test -p melee_modes --test oracle_ptr`
runs 30,750 vectors, 250 per function.

| module | C TUs | functions |
|---|---|---|
| `gm_ptr.rs` | gmcamera, gmclassic, gmgiant, gmlightning, gmmain_lib, gmregcommon, gmregenddisp, gmresult, gmslomo, gmstaffroll, gmsupersudden, gmtiny, gmtoulib, gmtou_0, gmtou_1, gmvs, gmvsmelee, gm_1601, gm_1798, gm_17C0, gm_181A, gm_1884, gm_1A3F, gm_1B03 | 52 |
| `hud_ptr.rs` (if + ty) | ifprize, ifstatus, ifstock, soundtest, textlib, textlib_1, tydisplay, tyfigupon | 22 |
| `lb_ptr.rs` | lbanim, lbcardnew, lbcollision, lbcommand, lbshadow, lbsnap, lbspdisplay, lbvector, lb_00B0, lb_00F9, lb_013B, lb_0146 | 29 C functions, 28 Rust (`checkJObjFlags` is one Rust function for the two identical static copies, tested under both names) |
| `mn_ptr.rs` | mncharsel, mndiagram, mndiagram3, mngallery, mninfo, mninfobonus, mnmain, mnname, mnnamenew, mnsound, mnstagesw, mn_22EC | 20 |

The structs are the melee_types records. `gmClassicMatchup` and `FigaTree` are opaque there, because
they are defined in a `.c` and behind an unscanned header, so they are defined locally with `gc_layout!`
(TODO(group0)). No function needed another group's code, so there are no stubs.

Pointer conventions:
- A pointer the C dereferences becomes a reference.
- It becomes a slice when the C indexes it, and an `Option` when the C checks it for NULL.
- A pointer the C only stores or compares stays a raw pointer. Examples are `lbColl_800089B8`'s victim,
  `lb_8000CE30`'s next and `fn_80191240_save_root_jobj`'s jobj.

**Oracle** (`oracle/gen_oracle_ptr.py`, `oracle/build_ptr.sh` -> `tests/data/vectors_ptr.txt`):
- Each function is cut verbatim and renamed `o_<name>`, as in rounds 1 and 2. Types that the `.c` itself
  declares and the cut uses are cut too (e.g. `gmClassicMatchup`).
- Each pointer argument the body dereferences gets a buffer of max(512, sizeof) bytes, or N elements
  for the array readers listed in `BUFN`.
- The buffer is filled from a seed by a generator implemented in both main.c and the test (splitmix64
  words: zeros, small ints, bytes, floats in +-500 and +-2, and raw bits with bit 30 clear, so never
  NaN or inf).
- The C views the bytes as its own struct, the Rust as the melee_types record. The test asserts
  `size_of::<T>()` equals the C `sizeof` for every view.
- Both sides print the return value and an FNV-1a 64 hash of every buffer afterwards.
- Pointers the body only stores are passed as plain 64-bit values (0, small, and random).
- **Bitfield packing.** The cut C is compiled for `x86_64-pc-windows-gnu -mno-ms-bitfields`, not the
  msvc target that the round 1-2 oracle and melee_types' native check use.
  - The two targets have the same LLP64 data layout. Itanium bitfield packing puts the fields after a
    bitfield run at the GameCube offsets melee_types uses. MSVC packing moves them:
    `StartMeleeRules.x6` is at 8 under msvc, but at 6 on the GC and in melee_types.
  - On the msvc target the 6 functions over StartMeleeData/CSSData failed for exactly this reason.
    melee_types only size-asserts records without bitfields against msvc, so its records with
    bitfields are GC-offset, not msvc-native.
  - clang 23 refuses `-mno-ms-bitfields` on the msvc target ("Itanium-compatible layout for the Microsoft
    C++ ABI is not yet supported").
- **Functions that read or write a bitfield are refused** (3: `initVsPlayer`, `gmMultiman_InitTimedRules`,
  `fn_801BA1B4`). melee_types stores bitfield bytes in GC bit order, so the byte images differ. They
  need a field-level comparison.
- **Fixups** (`f:` tokens) store values after the fill, so loops terminate and indices stay in bounds:
  - terminators for `gmClassic_GetMatchupCount` (0x148), `un_80303FD4_first/count` (9) and
    `firstState` (0xFF)
  - `Command_06`'s loop_count, 1..3
  - `gm_801B0730`'s 1-based slot, 1..4
  - `tyDisplay_SetGridSize`'s count, <= 2540; beyond that the s8 row counter wraps and the C loops forever
  - small row caps for `mnCharSel_MexPackRows`; a random cap near -10^9 makes its fill loop run 10^9
    times
  - runs of full-width spaces for `checkStringRest`
- Rerun with `bash rust/melee_modes/oracle/build_ptr.sh`.

**FMA.** `fma_sites.py` finds fused sites in 11 of the 123. Each `fmadds` in the Rust is one of them,
and every other `a*b+c` stays unfused:

| function | fused sites |
|---|---|
| `lbColl_80005EBC` | 9 |
| `lbColl_80005FC0` | 5 |
| `end` | 2 |
| `sqrDistance` | 2 |
| `splGetCardinalTangent` | 14 |
| `splGetBSplineTangent` | 11 |
| `splGetBezierTangent` | 12 |
| `lbVector_Mirror` | 3 |
| `lb_800103D8` | 1 |
| `lb_80014638` | 2 |
| `getVolumePosition` | 1 |

I swapped `fmadds` for an unfused `a*c+b` in a scratch copy. The vectors then catch 9 of the 11
(11 to 125 failing rows each). They do not catch `end` or `lb_80014638`, which only compare the
fused value, so those two are fused by the IR alone.

**Things the C does that the port keeps:**
- `firstState` (gm_1A3F.c): its `break` only leaves a `do {} while (0)`, so it always returns the
  first state's id, or 0 when the table starts with 0xFF. `next_id` has no effect.
- `gm_80166CCC`: the second team loop runs to 6 over the 5-entry `team_standings`, so it reads
  `team_standings[5].active`, a byte inside `player_standings[0]`. The Rust reads that byte by offset
  from the MatchEnd base. MatchEnd has no pointers before it, so this is the same byte on the GC.
- `checkStringRest` (mnname.c): the full-width space literal is compiled from a UTF-8 source, so the
  shipping build compares E3 80, not the GameCube's Shift-JIS 81 40 (`mn_ptr::FULLWIDTH_SPACE`).
- `un_802FE918_update_x3`: `new_x3` is uninitialised in the C when nothing matches. The shipping
  build's `-ftrivial-auto-var-init=zero` makes it 0.
- `lbColl_80008D30`: `damage` is a u32 -> f32 conversion.

**Not ported from the 158:**
- 13 bitfield functions: the 3 the generator refused (above), plus 10 left out of `FUNCS` by hand
  for the same reason: `gmMultiman_InitScoreRules`, `lbColl_80008688/80008820` (HitCapsule
  `x40_b4`), `lb_80013BE4`, `lb_80013FF0`, `lb_80014234`, `lb_80014498`, `lb_800144C8`
  (ColorOverlay `x7C_*`), and `DevText_StoreColorIndex` and `DevText_SetTextColor`
  (`current_color:2`).
- 7 empty functions (`fn_801A4BD0`, the OnExit stubs, `fn_8019BA04`, `fn_80021C18`, ...).
- 12 functions over types melee_types does not generate because they are defined in `.c` files or
  unscanned headers: `gm_801BC00C_GetCharacterKind`, `fn_80180630_GetX118`, `gm_801AB200_HasCheck`,
  `getStKind`, `lbArq_80014ABC`, `lbHeap_ResetHeap`, `lbSnap_GetSaveDataOffset`,
  `mnCount_InitUserData`, `mnGallery_80259604`, `mn_80231634`, `mnSnap_GetLoadPhotoIdx` and
  `mnVibration_GetCursorRow`.
- `fn_801B9FB8`, which stores a function's address (`gm_80165290`).
- `fn_801661E0`, which returns its argument cast to `s32`.
- `fn_80166A8C`, whose body is MWCC inline asm only, so it has no PC body.

**Domains restricted:** index arguments stay inside their arrays (`DOMAIN` in gen_oracle_ptr.py), for
example `fn_801656A8` arg1 < 6, `ifStatus_GetStoredTranslationX` < 4 and `is_character_unlocked` n < 64.
Out of range, the Rust panics on the index, where the C reads past the array.

## Round 4 (night 2, 2026-10-21): callers of rounds 1-3, objects behind pointers

**Picking the batch.** `oracle/scan_r4.py` scans the gm, mn, if, ty and lb TUs for functions with
no indirect call, no bitfield, no non-const file-scope variable, no asm and a non-pointer return,
whose every callee is already ported (in melee_modes, or another crate's `pub fn` of that name),
and repeats to a fixed point. It finds 39. Round 4 ports 30 of them. The other 9 are
lbvector.c's `lbVector_CosAngle`, `_Rotate`, `_RotateAboutUnitAxis`, `_NormalizeXY`,
`_CreateEulerMatrix`, `_sqrtf_accurate`, `_Len_xy_accurate` and `_AngleXY`, and lbtrigf's `atan2f`.
Group 1 already ported them in `melee_runtime::{lbvector, lbtrigf}`.

**Ported: 30 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r4`,
14,300 vectors).

| module | C TUs | functions |
|---|---|---|
| `gm_r4.rs` | gmmain_lib, gm_1B03, gm_1601, gmtrainingmode, gmvsmelee, gmresult, textlib | 11: `bitset_set/clear/test/test_word`, `gm_SetupHumanPlayer`, `gm_SetupCpuPlayer`, `gm_MatchHasMultipleWinners`, `gm_801B07E8_layer`, `gmVsMelee_UpdateKOCounts`, `fn_801771C0`, `DevText_SetCursorX` |
| `lb_r4.rs` | lb_00CE, lb_00F9, lb_020A, lbshadow, lbaudio_ax, lbspdisplay | 14: `lb_8000D148`, `groundHeight`, `calc_acos`, `lbShadow_8000E9F0`, `fn_800253D8`, `fn_800256BC`, `fn_80025B44`, `fn_80025CBC`, `fn_800259EC`, `fn_80025A98`, `lbAudioAx_800264E4`, `lb_800138CC`, `lb_800138D8`, `lb_8001204C` |
| `mn_r4.rs` | mndiagram3, mnname, mnvibration | 5: `mnDiagram3_GetPopupSpacing`, `mnDiagram3_GetPopupY`, `mnDiagram3_GetRowSpacing`, `mnName_TextWidth`, `mnVibration_GetCursorYSpacing` |

- Callees from other groups are called directly, no stubs: `melee_runtime::lbtrigf::acosf`,
  `melee_runtime::gekko::sqrtf` (MSL math_ppc.h), and melee_types' `inline.rs` jobj/gobj accessors.
- `lbShadow_8000E9F0` is the one group 4 has as `unimplemented!("TODO(group1)")` in
  `melee_world/src/gr/ground.rs`; it can now call `melee_modes::lb_r4::lbShadow_8000E9F0`.
- `lbAudioAx_UserData` comes from `lbaudio_ax.static.h`, which melee_types does not scan, so it
  is defined in `lb_r4.rs` with a `gc_layout!` table (size 0x48; TODO(group0)).
- A gobj's user data, a spline's `cv` and the jobj tree are reached through raw pointers with
  `unsafe` and a SAFETY note, as the C dereferences them. `HSD_JObjGetTranslation*` asserts its
  jobj; the Rust panics on NULL there.

**Oracle** (`oracle/gen_oracle_r4.py`, `oracle/build_r4.sh` -> `tests/data/vectors_r4.txt`), built
on round 3's buffers and generator:
- Every game function a cut function calls is cut too, transitively, and renamed, so the C side
  runs the game's own callees (`calcPan`, `gm_801B05F4`, `acosf`/`atanf`, the spline tangents...),
  not the Rust ports. 17 callees are cut this way. `pc/gameworld/gekko_fp.c` is linked for
  `__frsqrte`, and main.c defines MSL float.c's NaN/inf words and an aborting `__assert`.
- Links: `l:<from>:<off>:<to>` stores buffer `to`'s address in buffer `from` (`ffff` = NULL).
  Buffers after the argument ones are reachable only through links: a gobj's user data, a
  spline's control points, `Diagram3.jobjs[8/9]`, a random jobj tree of 1-8 nodes with
  consistent child/next/parent links and random `JOBJ_INSTANCE` flags.
- Before hashing, both sides rewrite every 8-byte word that points into a buffer as
  (buffer, offset), so stored pointers (`lb_8001204C`'s results, the links) compare by target.
- Rerun with `bash rust/melee_modes/oracle/build_r4.sh`.

**FMA.** `fma_sites.py` finds fused sites in two of the 30: `lb_8000D148` (5) and
`mnDiagram3_GetPopupY` (1). I swapped each `fmadds` for an unfused `a*b + c` in a scratch
copy, one at a time. All 6 are caught: 15/250 rows for `GetPopupY` and 6 to 40 of 6,000 rows
for each `lb_8000D148` site. `lb_8000D148` returns a bool, so random inputs almost never see the
last bit. The generator adds boundary rows: it emulates the f32 arithmetic in Python and puts
the threshold where the fused and unfused intermediate values disagree.

**Domains restricted:** `bitset_*` bit < 4096 (the 512-byte buffer); `lbShadow_8000E9F0` numcv >= 2
(numcv 1 with u = 1 and type 0 reads `cv[-1]`); `lb_800138CC/D8` always get a user-data buffer
(the C does not check it). The sound functions get NULL gobjs and NULL user data.

**Not verified:** the test runs on a snapshot of HEAD plus these files, because other agents'
uncommitted work in `melee_chara` stopped the live worktree from compiling at the time
(see the commit message).

## Night 3 round 1 (2026-10-21): group total, nested-pointer leaves

**Group total.** libclang over the 166 gm, mn, if, ty and lb TUs in `game_tus.txt` (scan_r4.py's
parse, counting every function defined in the TU itself, static inlines included): **3,995
functions**. That includes lbtrigf.c / lbvector.c, which group 1 ports (`melee_runtime`), about
20 functions. Ported before this round: 251 (250 distinct C names; `checkJObjFlags` is two
static copies). After it: **261 of 3,995 (6.5%)**.

**TODO stubs.** melee_modes has no stubs to replace: every callee in rounds 1-4 was already
ported. Other crates hold `TODO(group6)` hooks (melee_fighter `hooks.rs`: `gm_8016B168`,
`gm_GetFrameCount`, `ifStatus_802F6*`...; melee_world `gm_8016B238/3A0`, `gm_IsCurrently1PMode`;
melee_chara `CAPS_MAX_JUMPS`). All of them read the global game state (`lbl_8046B6A0` match
state, save data), which is not ported yet, so none could be wired this round.

**Ported: 10 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r5`,
2,650 vectors). They are the round-3 near misses whose only obstacle was a pointer inside a pointee,
now testable with round 4's links.

| module | C TU | functions |
|---|---|---|
| `lb_r5.rs` | lbcommand | `Command_04`, `Command_05`, `Command_07` |
| `hud_r5.rs` | textdraw, ifstock, toy | `DevText_StrLen`, `DevText_NumToStr`, `DevText_AddToList`, `copyColor`, `_Toy_8030FE48_link_entries` |
| `mn_r5.rs` | mnnamenew | `GetAutoNameCharacter` |
| `gm_r5.rs` | gmmain_lib | `SetDefaultHandicaps` |

- **FMA:** none of the 10 does float arithmetic (lower.py: 0 fmuladd in every cut).
- `Command_04` keeps the C's `((u32*) info)[loop_count + 3] -= 1` "hack to match": on the native
  layout (u at 8, loop_count at 16, event_return at 24) it decrements whatever 4-byte word sits
  at byte 4*(loop_count+3), so the Rust does the same on the record's bytes. `loop_count - 1` is
  u32 arithmetic: with loop_count 1 the decrement hits loop_count itself and the C then reads
  `event_return[0xFFFFFFFF]` and faults natively (on the GC it wrapped to -1). Domain: 2..4.
- Mutation check: `>` for `>=` in `DevText_AddToList` fails 54/250 rows; a wrong ring head in
  `_Toy_8030FE48_link_entries` fails 214/250.

**Oracle** (`oracle/gen_oracle_r5.py` over `gen_oracle_r4.py`, `oracle/build_r5.sh` ->
`tests/data/vectors_r5.txt`): only the function list, domains and row builders are new.
`gen_oracle_r4.py`'s callee renamer no longer renames member names after `.`/`->`
(`u->Command_05.ptr`); its round-4 vectors regenerate byte-identical.

**Domains restricted:** `Command_04` loop_count 2..4, and event_return[0] is never a buffer link
(loop_count 3/4 decrement one half of it, and a moved address no longer compares by target);
`Command_05` loop_count 0..2; `GetAutoNameCharacter` index 0..511 (the name buffer);
`_Toy_8030FE48_link_entries` count 0..13 (13 entries); `DevText_StrLen` strings NUL-terminated
inside the buffer.

**Not ported / skipped this round:** `gm_80172DD4` (scan_r4 listed it because another crate has a
`pub fn inline1`, a name collision; gm_16F1's `inline1` reads the save data), `Locate`
(lbarchive: adds a native pointer into u32 relocation slots, no native meaning) and
`_tyDisplay_8031BBF4` (returns a string's address as s32). The big tydisplay sorters
(`_tyDisplay_8031830C/80318714`, ~780 tokens each) are next.

## Next round
- Bitfield-touching functions: compare field values, not bytes, or add a native<->GC bitfield bridge.
- Generate the records that `.c` files define (TODO(group0)), then port the 12 functions above.
- gm_1601's save-data getters (`gm_80162968`, `gm_GetPlayTime`, ...) need a `GameSave` state object.
- The non-const static tables in gm_17E4/gm_17EB (`lbl_803D7AC0`, ...): check whether they are ever
  written, then treat them as const.
- Callers of round 4 (rerun `oracle/scan_r4.py`; it lists near misses with one unported callee).
- Leaf functions with nested pointers that round 3 skipped (`Command_04/05/07`, `DevText_NumToStr`,
  `_tyDisplay_8031830C`...): round 4's links make them testable.

## Night 3 round 2 (2026-10-21): recount, callers of round 5, leftover leaves

**Recount.** Group total unchanged: **3,995** functions in the 166 gm/mn/if/ty/lb TUs (same
libclang count as round 1; ~20 of them lbtrigf/lbvector, group 1's). Ported at the start: 261
(all verified). After this round: **269 of 3,995 (6.7%)**, all verified.

**TODO stubs.** Still none inside melee_modes. Rerunning `oracle/scan_r4.py` over the current
workspace: the other crates' `TODO(group6)` hooks all still need the global game state
(`lbl_8046B6A0`, save data), and the scan's only newly unlocked callers are this round's two
textlib_1 functions plus group 1's lbvector/lbtrigf functions (`lbVector_*`, `atan2f`), left to
group 1.

**Ported: 8 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r6`,
3,500 vectors; `oracle/gen_oracle_r6.py` over gen_oracle_r4.py, `oracle/build_r6.sh`):

| module | C TU | functions |
|---|---|---|
| `hud_r6.rs` | textlib_1, soundtest | `un_80302EA4`, `un_80302FFC_maxlen` (callers of round 5's `DevText_StrLen`; the C side runs the cut `DevText_StrLen`), `un_802FFCD0` |
| `lb_r6.rs` | lbrefract, lb_00B0 | `my_fmodf`, `lb_8000B074`, `memzero` |
| `gm_r6.rs` | gmmain_lib, gmtou_0 | `gmMainLib_AdjustNameTags`, `fn_80194658_get_value` |

- **FMA:** one fused site, `my_fmodf`'s `a - b * (float) q` = fmuladd(-b, q, a) (lower.py: 1
  lowered). The unfused form fails 912/1,500 rows, so the vectors discriminate.
- `un_80302EA4`'s `(int) x18` is a float->int conversion lower.py saturates (1 site); the
  vectors use 0, 0.5, 1, 1.9, 2, 2.5, 3 and -1. Mutation `<= 1` -> `< 1` on x0 fails 29/400
  rows; rounding instead of truncating x18 is NOT caught (the extra string rarely changes the
  maximum), so the truncation is checked by reading only.
- The dispatcher defines `fabsf` as a sign-bit clear (the fabs instruction): `-fno-builtin`
  leaves my_fmodf's call unresolved otherwise.

**Domains restricted:** `my_fmodf` finite a in +-1000, b in +-50 and nonzero (|a/b| stays far
inside s64: lower.py does not model gwtool's float->s64 conversion, so an out-of-range or NaN
quotient is unverified; the Rust saturates); `memzero` size 0..512 (a negative size loops
forever in the C); `un_802FFCD0` count -3..127 (the 512-byte buffer); `un_80302EA4` strings are
NUL-terminated inside their buffers and xC holds at least (int) x18 entries.

**Not ported (looked at this round):** `gm_801891F4_GetTickRate` (reads `__OSBusClock`, a
global), `tyDisplay_8031BB34` (returns a table pointer as s32, like `_tyDisplay_8031BBF4`).

## Night 3 round 3 (2026-10-21): recount, callers of memzero

**Recount.** Group total unchanged: **3,995** functions in the 166 gm/mn/if/ty/lb TUs (same
libclang count; ~20 lbtrigf/lbvector, group 1's). Ported at the start: 269 (268 distinct
`pub fn` names in melee_modes + the second static `checkJObjFlags` copy), all verified. After
this round: **275 of 3,995 (6.9%)**, all verified.

**TODO stubs.** None inside melee_modes. `oracle/scan_r4.py` over the current workspace: the
only newly unlocked callers are the six memzero callers below (round 6 ported memzero) plus
group 1's lbvector/lbtrigf functions, left to group 1. The other crates' `TODO(group6)` hooks
still all read the global game state (`lbl_8046B6A0`, save data), not ported yet.

**Ported: 6 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r7`,
1,800 vectors; `oracle/gen_oracle_r7.py` over gen_oracle_r4.py, `oracle/build_r7.sh`; the C side
runs the game's own memzero / fn_801771C0 / fn_80165418 / fn_801654A0 from the callee cut):

| module | C TU | functions |
|---|---|---|
| `hud_r7.rs` | textlib | `DevText_Erase` |
| `gm_r7.rs` | gm_1601, gm_16A2, gmresult, gmresultplayer, gm_17C0 | `fn_80168A6C`, `fn_80169574`, `gmResult_PcWinner`, `gm_80177724`, `gm_8017C984` |

- **FMA:** none (lower.py: 0 fmuladd in every cut).
- `fn_80168A6C` reads a record the function itself defines; it is `gm_r7::Fn80168A6CSrc` with
  `gc_layout!` GC offsets (0x0 s32**, 0x4 s32*, 0x8, 0xC; 0x10 bytes).
- `gmResult_PcWinner`'s scratch is a function-local static the C zeroes on every call; the Rust
  uses a zeroed local (nothing else reads the static).
- Mutation check: `-1` for `-2` in fn_80169574 fails 400/400; xC -> x8 in fn_80168A6C 369/400;
  `1 *` for `2 *` in DevText_Erase 185/250.

**Domains restricted:** `fn_80169574` size 0..511 (the 512-byte buffer); `DevText_Erase` 2*w*h <= 512;
`fn_80168A6C` idx 0..7 (an 8-pointer array), entries and x4 NULL 30% of the time.

**Unverified ports:** none.

**Next:** the 12 functions over `.c`-defined records (local records with `gc_layout!`, as
`Fn80168A6CSrc` here), the bitfield functions (compare by value), and the tydisplay sorters.

## Night 3 round 4 (2026-10-21): recount, leftover lb leaves over nested pointers

**Recount.** Group total unchanged: **3,995** functions in the 166 gm/mn/if/ty/lb TUs (libclang
definition count from `oracle/scan_r4.py`; ~20 lbtrigf/lbvector, group 1's). Ported at the start:
**275** (274 distinct C names among melee_modes' `pub fn`/`pub unsafe fn` + the second static
`checkJObjFlags` copy), all verified. After this round: **280 of 3,995 (7.0%)**, all verified.
`scan_r4.py` now also counts `pub unsafe fn` as ported (it missed them before, so its "mine" set
under-counted by a few; the counts above were always taken by name, not from the scan).

**TODO stubs.** None inside melee_modes. Rerunning `scan_r4.py`: no newly unlocked callers
besides group 1's lbvector/lbtrigf functions; the other crates' `TODO(group6)` hooks still read
the global game state. The c2r externs in melee_fighter (`ftcoll.rs`, `ftcpuattack.rs`,
`ft_07C1.rs`) for `lbColl_CopyHitCapsule` / `lbColl_8000ACFC` can now bind to `melee_modes::lb_r8`
(not touched: group 4's crate).

**Ported: 5 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r8`,
1,950 vectors; `oracle/gen_oracle_r8.py` over gen_oracle_r4.py, `oracle/build_r8.sh`):

| module | C TU | functions |
|---|---|---|
| `lb_r8.rs` | lbcollision, lb_00B0, lbspdisplay | `lbColl_CopyHitCapsule`, `lbColl_8000ACFC`, `lb_8000B09C`, `lb_8000B134`, `lb_80011B74` |

- These are the "missing []" near-misses of scan_r4 (no callee, no global, no bitfield) that round 3
  skipped for nested pointers. Rows: random JObj trees (1-9 joints, INSTANCE flag, AObjs with
  NO_ANIM/REWINDED combos), DObj chains of 1-6 with MObjs, and victim arrays seeded with the
  victim argument's value 15% of the time (true in 182/400, 102/500, 116/500 rows).
- **FMA:** none (lower.py: 0 fmuladd in every cut).
- `lbColl_CopyHitCapsule`: the C struct assignment copies HitVictim's 4 tail-padding bytes; a
  field-wise Rust copy failed 250/250 rows on those bytes, so the Rust copies each element
  bytewise.
- Mutation check: REWINDED bit 26 -> 25 fails 50/500; dropping the JOBJ_INSTANCE test fails
  81/500 and 117/500; victims_1 -> victims_2 165/400; `|=` -> `^=` 257/300.

**Unverified ports:** none.

**Workspace tests:** `cargo build --workspace` green; `cargo test --workspace --no-fail-fast`
fails only `melee_chara` chars_ref and `melee_fighter` plbonus_ref, both crates with other agents'
uncommitted edits in the worktree at the time; every melee_modes test passes.

**Next:** gm/mn inline getters over `.c`-defined records (`getStKind`, `gm_801AB200_HasCheck`,
`gm_801BC00C_GetCharacterKind`, `mnVibration_GetCursorRow`, `mnSnap_GetLoadPhotoIdx`, ...), the
bitfield functions, and the tydisplay sorters.

## Night 3 round 5 (2026-10-21): recount, lb_80011C18, getters over .c-defined records

**Recount.** Group total unchanged: **3,995** functions in the 166 gm/mn/if/ty/lb TUs (libclang
definition count from `oracle/scan_r4.py`; ~20 lbtrigf/lbvector, group 1's). Ported at the start:
**280** (279 distinct C names among melee_modes' `pub fn`/`pub unsafe fn`, besides the
`GXColor::to_u32`/`from_u32` helpers, + the second static `checkJObjFlags` copy), all verified.
After this round: **288 of 3,995 (7.2%)**, all verified.

**TODO stubs.** None inside melee_modes to replace: `FigaTree` and `gmClassicMatchup` are still
opaque in melee_types and `GXColor` is still not generated. `scan_r4.py` rerun: the only newly
ready caller in this group is `lb_80011C18` (ported below); the rest of its ready list is group 1's
lbvector/lbtrigf.

**Ported: 8 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r9`,
2,250 vectors; `oracle/gen_oracle_r9.py` over gen_oracle_r4.py, `oracle/build_r9.sh`):

| module | C TU | functions |
|---|---|---|
| `lb_r9.rs` | lbspdisplay, lb_0219, lbsnap | `lb_80011C18`, `fn_80021C18`, `lbSnap_GetSaveDataOffset` |
| `gm_r9.rs` | gmtoulib, gm_1879, gmstaffroll, gmevent | `fn_8018FBD8`, `getStKind`, `gm_801AB200_HasCheck`, `gm_801BC00C_GetCharacterKind` |
| `mn_r9.rs` | mnvibration | `mnVibration_GetCursorRow` |

- New local records with `gc_layout!` GC offsets: `Unk80433380_0` (0x3C), `gm_80187F48_EnterData`
  (0x4), `StaffEntryData` (0x14 GC / 0x20 native), `gm_801BAB40_src` (0x1C), `MnVibrationData`
  (0x98 GC / 0x128 native). The C side views the same buffers as its own .c-defined struct; the
  test asserts the Rust sizes equal the C sizeof.
- `lb_80011C18` rows: random JObj trees of 1-5 joints (child/next/parent), flags with JOBJ_PTCL,
  JOBJ_SPLINE, both, or neither; u.dobj a 1-2 DObj chain (each with an MObj) or NULL (20%). The C
  oracle's main.c holds 32 buffers, so trees are kept small (a 9-joint, 4-DObj row overflowed it).
- `lbSnap_GetSaveDataOffset` is `xC + offsetof(x38)` (0x38 on GC and native: no pointers in the
  record); `fn_8018FBD8` sign-extends the s32 into `user_data`, as the native int-to-pointer cast.
- gen_oracle_r9 adds the `StaffEntryCheck` typedef by hand (`EXTRA`): the cut copies only types the
  function references directly.
- **FMA:** none (lower.py: 0 fmuladd in every cut).
- Mutation check: dropping the `jobj->next` block of lb_80011C18 fails 29/500, `|=` -> `^=` 177/500;
  zero-extending the s8 in gm_801BC00C fails 69/250; zero-extending fn_8018FBD8's s32 85/250.

**Unverified ports:** none.

**Not ported (looked at):** `mnSnap_GetLoadPhotoIdx` and `fn_80180630_GetX118` (one-line getters,
but over 0x2A8/0x158-byte .c records with ~80/~50 fields to lay out; next round),
`fn_801B9FB8` (stores the address of `gm_80165290`: a function-pointer value the harness cannot
compare across C and Rust).

**Workspace tests:** `cargo build --workspace` green. `cargo test --workspace` does not build at
this moment because of other agents' uncommitted work in the worktree (melee_runtime
`tests/oracle_generator.rs` and `melee/src/stage.rs` use `melee_runtime::generator`/`particle`,
not yet declared), plus LNK1104 file locks from concurrent test runs; every melee_modes test passes
(run with a private CARGO_TARGET_DIR).

**Next:** the two large-record getters above, the bitfield functions (compare by value), the
tydisplay sorters.

## Night 3 round 6 (2026-10-21): lbrefract texel callbacks, fn_80025E38, gm leaves

**Recount.** Group total still **3,995** (no gm/mn/if/ty/lb TU changed since the night-3 count,
`git log` over src/melee/{gm,mn,if,ty,lb}). Ported at the start: **288**, all verified.
After this round: **298 of 3,995 (7.5%)**, all verified.

**TODO stubs.** None replaceable: `GXColor` is still not generated, `FigaTree` and
`gmClassicMatchup` are still opaque in melee_types.

**Ported: 10 functions, all checked bit for bit against the C** (`cargo test -p melee_modes --test oracle_r10`,
4,950 vectors; `oracle/gen_oracle_r10.py` over gen_oracle_r4.py, `oracle/build_r10.sh`):

| module | C TU | functions |
|---|---|---|
| `lb_r10.rs` | lbrefract, lbaudio_ax | `fn_80021F70`, `fn_80021FB4`, `fn_80021FF8`, `fn_8002206C`, `fn_80025E38` |
| `gm_r10.rs` | gm_1601, gm_17C0, gm_1832, gmtoulib | `fn_80161004`, `gm_801688AC`, `fn_8017DE54`, `fn_801859C8`, `gmTournament_SetTripleRightCoords` |

- **FMA:** none (lower.py: 0 fmuladd); fn_80025E38's float->int conversion is the saturating one
  gwtool's lowering produces (2 fptoi saturated in the cut).
- `fn_801859C8` dereferences `gobj` unchecked, so its gobj argument is never NULL in the rows (an
  earlier generator run with a NULL gobj segfaulted the C oracle and left a truncated vector file).
- Mutation check (each fails its function only): x40 mask 177->205/400 (fn_801859C8), pkind 0->1
  231/300, x18/3->/2 168/250, RGB565 b/8->b/4 543/600, out 0xFF->0xFE 589+499/600,
  VOL_MAX 0x7E 193/600, 3-count->2-count 171/600, pkind!=3->!=2 65/400, arg9->arg8 599/600.

**Unverified ports:** none.

**Workspace:** `cargo build --workspace` green; every melee_modes test passes (private target dir).

**Next:** `mnSnap_GetLoadPhotoIdx`, `fn_80180630_GetX118` (large .c records), bitfield functions,
tydisplay sorters.
