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

## Next round
- The 255 pointer-taking leaves: struct-in, struct-out helpers over the melee_types records. Check them with
  zeroed and randomised native structs; the C side needs those records laid out natively.
- Then the callers of round 1's functions (e.g. `gm_801602C0` over `gm_801601C4` / `gm_80160244`), with
  other-group callees as TODO(group) stubs.
