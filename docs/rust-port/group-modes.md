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

## Next round
- The 255 pointer-taking leaves: struct-in, struct-out helpers over the melee_types records. Check them with
  zeroed and randomised native structs; the C side needs those records laid out natively.
- gm_1601's large family of save-data getters (`gm_80162968`, `gm_GetPlayTime`, ...): each calls one
  gmMainLib accessor over the global save struct. They need a `GameSave` state object in Rust, with the
  accessors as `TODO(group)` stubs or ports.
- The non-const static tables (`lbl_803D7AC0`, `lbl_803D85F0`, ... in gm_17E4/gm_17EB) are probably
  never written. Check that, then treat them as const.
