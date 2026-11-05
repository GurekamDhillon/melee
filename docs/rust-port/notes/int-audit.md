# Integer-semantics audit (2026-11-05, branch port/audit-int)

Goal: no place where the port's integer behaviour differs from Gekko. Reference: the retail DOL's compiled PPC (symbols.txt +
splits.txt), cross-checked with the C shipping IR (the gwtool input) and the Rust MIR. Scanner: `tools/rust-port/int_audit.py`
(usage in its docstring). No disc bytes are printed or committed: the table below holds addresses, symbol names and mnemonics.

## What Gekko does, and the helpers that copy it
`melee_core::ppc` (Rust) and gwtool's `makeArithmeticSafe` (C, pc/tools/gwtool/gwtool.cpp) implement the same rules:

| op | Gekko / MWCC | Rust before | C build before |
|---|---|---|---|
| `divw` by 0, `INT_MIN / -1` | -1 if the dividend is negative, else 0 (Dolphin's hardware-tested model) | panic | same as Gekko |
| `divwu` by 0 | 0 | panic | same |
| `%` (MWCC: divw, mullw, subf) | `x % 0 = x`; `INT_MIN % -1 = INT_MIN - 1 = INT_MAX` | panic | `x % 0 = x`, but `INT_MIN % -1 = 0`: **fixed** to INT_MAX |
| `slw`/`srw`/`sraw` amount 32..63 | 0 / 0 / all sign bits | amount masked to 5 bits | same as Gekko |
| `fctiwz` (`(int) f`) | saturates; NaN -> 0x80000000 | saturates; NaN -> 0 | `fptosi.sat`, NaN -> 0: **fixed** (select on `fcmp uno`) |
| `(s16) f`, `(u8) f`, ... | fctiwz then truncate | `as i16` saturates to the narrow type; `(f as u32) as u8` floors negatives at 0 | fctiwz-then-truncate; NaN fixed as above |
| `(u32) f` (`__cvt_fp2unsigned`) | saturates, negative and NaN -> 0 | same (`as u32`) | same (`fptoui.sat`) |
| `(s64) f` | `__cvt_fp2signed`-style runtime call | `as i64` (saturating, NaN 0) | `fptosi.sat.i64`, same as Rust: left, 8 Rust sites, not checked against the console |
| add/sub/mul overflow | wraps | wraps (overflow-checks = false in dev, release, fast, perf, test) | wraps (-fwrapv) |
| int -> int casts | truncate / extend by type | `as` truncates / extends by type | same |
| int -> float | exact in f64 then one rounding (frsp) | one rounding (`as f32`) | one rounding: same result |

i8/i16/u8/u16 operands of `/ % << >>` are promoted to int first (`Ppc` impls for the narrow types go through i32).
Host code (renderer, netplay, scripting, launcher, generated layout accessors) is native in both builds, so x86 semantics
are right there and it was left alone (scope `host` in the scanner: 83 div, 49 rem, 184 shl, 85 shr, 117 float->int sites).

## Method
1. `dol`: capstone over every .text symbol of the DOL. Classes: cmp-s / cmp-u (cmpw[i] / cmplw[i] whose CR field feeds a
   relational branch), shift (slw/srw/sraw), div (divw/divwu), fptoi (fctiwz/fctiw), sext8 (extsb), sext16 (extsh/lha*),
   zext8 (lbz*), zext16 (lhz*).
2. `cir`: the same classes in the shipping C IR, with line numbers (each DOL site gets the k-th C line of its class in
   that function; approximate where MWCC and clang emit different counts).
3. `rmir`: every Rust Div/Rem/Shl/Shr with a non-literal operand and every FloatToInt (except to u32/u64/usize) from MIR
   spans. `fix` rewrote all 1254 game-scope sites to `melee_core::ppc` (0 left; 8 `as i64` sites kept, see above; 3 const
   items in granime.rs use literal-range shifts and stay `<<`). A MIR re-scan after the fix leaves 262 game-scope rows, none open: 251 whose operand is a literal MIR did not fold (`x % 6 as c_int`, `1 << 23` in const items; exact), the 8 `as i64` and the 3 granime const shifts.
4. `cmp`: 174 functions where compare signedness, extension or `== 1` (bool truthiness) differ in count between the C IR and
   the Rust MIR. All 174 were checked by hand: 0 mismatches. The flags come from `-enum int` enums (signed in C),
   u8/u16 promotion, match range patterns and literal copies of C `== 1` tests. Notes: `pl_8003E4A4` (Rust skips a C
   `seen[-1]` store that is undefined behaviour), `Ground_StageSlot*` are PC-only.
   Bool: the generated types make every C `BOOL`/`bool` an i32 and no Rust code reinterprets game memory as `bool`.
   Extension: field loads come from the generated layout (signedness from the headers), so lbz/lha/lhz/extsb/extsh
   follow the types; the cmp check found no swapped extension.

## Results
- Rust: 1254 sites changed (div 150, rem 187, shl 187, shr 20, float->int 718 incl. 25 `(f as u32) as u8/u16`).
- C (mirrored, frozen-C rule: a shared bug): gwtool NaN -> 0x80000000 for fptosi, `INT_MIN % -1` -> INT_MAX.
- Goldens: not re-recorded. No golden changes: the Rust engine now carries the new semantics and still matches every C golden
  (gate, coverage 212/10), so no recorded run hits a NaN conversion or `INT_MIN % -1`; a C build with the new gwtool gives
  the same CSVs. The shared `_build/gwtool/gwtool.exe` is NOT rebuilt (built and checked privately: the new IR has the selects).

Measures: slpcheck 86 of 125 frame-exact (86 before); gate OK; coverage 212 pass / 10 no golden.

## Site table (DOL div / shift / fptoi; compare and extension classes as counts)

Status: fixed = the Rust function had a raw op of the class, now a melee_core::ppc call; ok = the Rust function has none left (it calls a helper that is fixed, or the op is constant); ok-const = no non-constant op of the class in the C either (MWCC emitted it for a constant or a runtime routine); no-port = no Rust function of that name (SDK / MSL / runtime: native shims in both builds).

| class | status | sites |
|---|---|---|
| cmp-s | - | 7908 |
| cmp-u | - | 2002 |
| div | fixed | 74 |
| div | no-port | 31 |
| div | ok | 82 |
| div | ok-const | 152 |
| fptoi | fixed | 480 |
| fptoi | no-port | 62 |
| fptoi | ok | 473 |
| sext16 | - | 2275 |
| sext8 | - | 1509 |
| shift | fixed | 34 |
| shift | no-port | 211 |
| shift | ok | 42 |
| shift | ok-const | 159 |
| zext16 | - | 3301 |
| zext8 | - | 19281 |

| address | function | C line | class | insn | Rust location | status |
|---|---|---|---|---|---|---|
| 8000EA44 | lbShadow_8000E9F0 | melee/lb/lbshadow.c:90 | fptoi | fctiwz | melee_runtime/src/tu/lbshadow.rs:159 | fixed |
| 80012354 | lb_800122F0 | melee/lb/lbspdisplay.c:278 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012360 | lb_800122F0 | melee/lb/lbspdisplay.c:279 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012368 | lb_800122F0 | melee/lb/lbspdisplay.c:280 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012378 | lb_800122F0 | melee/lb/lbspdisplay.c:282 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 8001237C | lb_800122F0 | melee/lb/lbspdisplay.c:283 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012380 | lb_800122F0 | melee/lb/lbspdisplay.c:284 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 8001238C | lb_800122F0 | melee/lb/lbspdisplay.c:286 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012390 | lb_800122F0 | melee/lb/lbspdisplay.c:287 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80012394 | lb_800122F0 | melee/lb/lbspdisplay.c:288 | fptoi | fctiwz | melee_runtime/src/tu/lbspdisplay.rs:491 | ok |
| 80014370 | lb_80014258 | melee/lb/lb_013B.c:191 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80014384 | lb_80014258 | melee/lb/lb_013B.c:192 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80014398 | lb_80014258 | melee/lb/lb_013B.c:193 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 800143AC | lb_80014258 | melee/lb/lb_013B.c:194 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 8001440C | lb_80014258 | melee/lb/lb_013B.c:201 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80014420 | lb_80014258 | melee/lb/lb_013B.c:202 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80014434 | lb_80014258 | melee/lb/lb_013B.c:203 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80014448 | lb_80014258 | melee/lb/lb_013B.c:204 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:195 | ok |
| 80019D1C | executeNextTask | melee/lb/lbcardnew.c:205 | shift | slw | melee_runtime/src/tu/lbcardnew.rs:335 | ok |
| 8001F2F0 | fn_8001F2A4 | melee/lb/lbmthp.c | div | divwu | melee_runtime/src/tu/lbmthp.rs:454 | ok-const |
| 8001F354 | fn_8001F2A4 | melee/lb/lbmthp.c | div | divwu | melee_runtime/src/tu/lbmthp.rs:454 | ok-const |
| 8001F3D8 | fn_8001F2A4 | melee/lb/lbmthp.c | div | divwu | melee_runtime/src/tu/lbmthp.rs:454 | ok-const |
| 80020510 | fn_800204C8 | melee/lb/lbbgflash.c:444 | fptoi | fctiwz | melee_runtime/src/tu/lbbgflash.rs:247 | fixed |
| 80020524 | fn_800204C8 | melee/lb/lbbgflash.c:445 | fptoi | fctiwz | melee_runtime/src/tu/lbbgflash.rs:248 | fixed |
| 80020538 | fn_800204C8 | melee/lb/lbbgflash.c:446 | fptoi | fctiwz | melee_runtime/src/tu/lbbgflash.rs:249 | fixed |
| 8002054C | fn_800204C8 | melee/lb/lbbgflash.c:447 | fptoi | fctiwz | melee_runtime/src/tu/lbbgflash.rs:250 | fixed |
| 80021B64 | lbBgFlash_Proc | melee/lb/lb_0219.c:85 | fptoi | fctiwz | melee_modes/src/tu/lb_0219.rs:110 | ok |
| 80021B8C | lbBgFlash_Proc | melee/lb/lb_0219.c:87 | fptoi | fctiwz | melee_modes/src/tu/lb_0219.rs:110 | ok |
| 80021BB4 | lbBgFlash_Proc | melee/lb/lb_0219.c:88 | fptoi | fctiwz | melee_modes/src/tu/lb_0219.rs:110 | ok |
| 80024740 | fn_80024654 | melee/lb/lbaudio_ax.c:1136 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2153 | ok |
| 80024814 | fn_80024654 | melee/lb/lbaudio_ax.c:1147 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2153 | ok |
| 80024934 | fn_80024654 | melee/lb/lbaudio_ax.c:1162 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2153 | ok |
| 800249E0 | fn_80024654 | melee/lb/lbaudio_ax.c:1171 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2153 | ok |
| 800251D0 | calcPan | melee/lb/lbaudio_ax.c:1472 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2462 | ok |
| 800253A4 | fn_800251EC | melee/lb/lbaudio_ax.c:1538 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2514 | ok |
| 8002553C | fn_800253D8 | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2556 | ok |
| 80025694 | fn_800253D8 | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2556 | ok |
| 80025820 | fn_800256BC | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2570 | ok |
| 80025978 | fn_800256BC | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2570 | ok |
| 80025C98 | fn_80025B44 | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2604 | ok |
| 80025E10 | fn_80025CBC | melee/lb/lbaudio_ax.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2614 | ok |
| 80025EEC | fn_80025E38 | melee/lb/lbaudio_ax.c:1657 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2624 | ok |
| 80025F80 | fn_80025E38 | melee/lb/lbaudio_ax.c:1662 | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:2624 | ok |
| 8002B190 | Camera_8002B0E0 | melee/cm/camera.c:1604 | fptoi | fctiwz | melee_world/src/tu/camera.rs:1578 | ok |
| 8002EE08 | Camera_8002ED9C | melee/cm/camera.c:3457 | fptoi | fctiwz | melee_chara/src/adapt.rs:322 | ok |
| 8002F220 | Camera_8002F0E4 | melee/cm/camera.c:3588 | fptoi | fctiwz | melee_chara/src/adapt.rs:331 | ok |
| 80030C48 | Camera_80030BBC | melee/cm/camera.c | fptoi | fctiwz | melee_world/src/tu/camera.rs:4720 | ok |
| 80030C50 | Camera_80030BBC | melee/cm/camera.c | fptoi | fctiwz | melee_world/src/tu/camera.rs:4720 | ok |
| 80031190 | Camera_8003118C | melee/cm/camera.c:5155 | fptoi | fctiwz | melee_world/src/tu/camera.rs:4925 | fixed |
| 80037B9C | pl_80037B2C | melee/pl/pltrick.c:26 | shift | slw | melee_fighter/src/tu/plbonus.rs:63 | ok |
| 80038038 | fn_80037F00 | melee/pl/pltrick.c:178 | fptoi | fctiwz | melee_fighter/src/tu/pltrick.rs:98 | fixed |
| 80038414 | pl_80038144 | melee/pl/pltrick.c:343 | shift | slw | melee_fighter/src/tu/pltrick.rs:228 | fixed |
| 80039168 | pl_8003906C | melee/pl/plbonus.c:275 | div | divwu | melee_fighter/src/tu/plbonus.rs:357 | fixed |
| 80039340 | pl_80039238 | melee/pl/plbonus.c:321 | div | divwu | melee_fighter/src/tu/plbonus.rs:393 | fixed |
| 8003A794 | fn_80039618 | melee/pl/plbonus.c:603 | fptoi | fctiwz | melee_fighter/src/tu/plbonus.rs:667 | fixed |
| 800412E4 | pl_800412D0 | melee/pl/pl_040D.c:147 | fptoi | fctiwz | melee_modes/src/tu_ext.rs:1131 | ok |
| 8004132C | pl_80041300 | melee/pl/pl_040D.c:153 | fptoi | fctiwz | melee_fighter/src/tu/pl_040d.rs:145 | fixed |
| 800432EC | mpColl_80043268 | melee/mp/mpcoll.c:772 | fptoi | fctiwz | melee_world/src/tu/mpcoll.rs:732 | fixed |
| 80043428 | mpCollEnd | melee/mp/mpcoll.c | fptoi | fctiwz | melee_world/src/tu/mpcoll.rs:758 | ok |
| 800435CC | mpColl_80043558 | melee/mp/mpcoll.c:865 | fptoi | fctiwz | melee_world/src/tu/mpcoll.rs:801 | fixed |
| 8004363C | mpColl_80043558 | melee/mp/mpcoll.c:875 | fptoi | fctiwz | melee_world/src/tu/mpcoll.rs:811 | fixed |
| 800438A4 | mpColl_80043754 | melee/mp/mpcoll.c:980 | fptoi | fctiwz | melee_world/src/tu/mpcoll.rs:901 | fixed |
| 8005817C | mpLib_8005811C | melee/mp/mplib.c:5690 | fptoi | fctiwz | melee_world/src/tu/mplib.rs:4935 | fixed |
| 8006A944 | Fighter_procAnim | melee/ft/fighter.c:2078 | fptoi | fctiwz |  | no-port |
| 8006CC50 | Fighter_UnkTakeDamage_8006CC30 | melee/ft/fighter.c:3556 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:2610 | ok |
| 8006CCE8 | Fighter_TakeDamage_8006CC7C | melee/ft/fighter.c:3569 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:2618 | ok |
| 8006CD28 | Fighter_TakeDamage_8006CC7C | melee/ft/fighter.c:3573 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:2618 | ok |
| 8006CD54 | Fighter_TakeDamage_8006CC7C | melee/ft/fighter.c:3580 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:2618 | ok |
| 8006D380 | Fighter_procCollResolve | melee/ft/fighter.c:3841 | fptoi | fctiwz |  | no-port |
| 8006D468 | Fighter_procCollResolve | melee/ft/fighter.c | fptoi | fctiwz |  | no-port |
| 8006D6C4 | Fighter_procCollResolve | melee/ft/fighter.c | fptoi | fctiwz |  | no-port |
| 80073070 | ftAction_80073008 | melee/ft/ftaction.c:1348 | fptoi | fctiwz | melee_fighter/src/tu/ftaction.rs:812 | fixed |
| 800750AC | ftParts_8007506C | melee/ft/ftparts.c:770 | shift | slw | melee_fighter/src/tu/ftanim.rs:51 | ok |
| 80076670 | ftColl_80076640 | melee/ft/ftcoll.c | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:244 | ok |
| 80076A08 | ftColl_8007699C | melee/ft/ftcoll.c:276 | fptoi | fctiwz |  | no-port |
| 80076A10 | ftColl_8007699C | melee/ft/ftcoll.c:276 | fptoi | fctiwz |  | no-port |
| 80076B58 | ftColl_8007699C | melee/ft/ftcoll.c:286 | fptoi | fctiwz |  | no-port |
| 80076B60 | ftColl_8007699C | melee/ft/ftcoll.c:286 | fptoi | fctiwz |  | no-port |
| 80076D2C | ftColl_80076CBC | melee/ft/ftcoll.c | fptoi | fctiwz |  | no-port |
| 8007703C | ftColl_80076ED8 | melee/ft/ftcoll.c:474 | fptoi | fctiwz |  | no-port |
| 800770F4 | ftColl_80076ED8 | melee/ft/ftcoll.c:489 | fptoi | fctiwz |  | no-port |
| 800771D0 | ftColl_80076ED8 | melee/ft/ftcoll.c:490 | fptoi | fctiwz |  | no-port |
| 80077294 | ftColl_80076ED8 | melee/ft/ftcoll.c:513 | fptoi | fctiwz |  | no-port |
| 800774A8 | ftColl_80077464 | melee/ft/ftcoll.c:677 | fptoi | fctiwz |  | no-port |
| 800776EC | ftColl_80077688 | melee/ft/ftcoll.c:793 | fptoi | fctiwz |  | no-port |
| 800779DC | ftColl_80077970 | melee/ft/ftcoll.c:960 | fptoi | fctiwz |  | no-port |
| 800779E4 | ftColl_80077970 | melee/ft/ftcoll.c:960 | fptoi | fctiwz |  | no-port |
| 80077B2C | ftColl_80077970 | melee/ft/ftcoll.c:970 | fptoi | fctiwz |  | no-port |
| 80077B34 | ftColl_80077970 | melee/ft/ftcoll.c:970 | fptoi | fctiwz |  | no-port |
| 80077F38 | ftColl_80077C60 | melee/ft/ftcoll.c:1057 | fptoi | fctiwz |  | no-port |
| 80077F40 | ftColl_80077C60 | melee/ft/ftcoll.c:1058 | fptoi | fctiwz |  | no-port |
| 80077F7C | ftColl_80077C60 | melee/ft/ftcoll.c:1064 | fptoi | fctiwz |  | no-port |
| 80077FEC | ftColl_80077C60 | melee/ft/ftcoll.c:1075 | fptoi | fctiwz |  | no-port |
| 80078108 | ftColl_80077C60 | melee/ft/ftcoll.c:1076 | fptoi | fctiwz |  | no-port |
| 800781C4 | ftColl_80077C60 | melee/ft/ftcoll.c:1095 | fptoi | fctiwz |  | no-port |
| 800795D4 | ftColl_8007925C | melee/ft/ftcoll.c | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:416 | ok |
| 80079B94 | ftColl_80079AB0 | melee/ft/ftcoll.c | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:245 | ok |
| 80079DA8 | ftColl_80079C70 | melee/ft/ftcoll.c | fptoi | fctiwz |  | no-port |
| 80079F90 | ftColl_80079EA8 | melee/ft/ftcoll.c | fptoi | fctiwz |  | no-port |
| 8007A35C | ftColl_8007A06C | melee/ft/ftcoll.c:2451 | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:360 | ok |
| 8007A5E4 | ftColl_8007A06C | melee/ft/ftcoll.c:2479 | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:360 | ok |
| 8007A8FC | ftColl_8007A06C | melee/ft/ftcoll.c:2592 | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:360 | ok |
| 8007A924 | ftColl_8007A06C | melee/ft/ftcoll.c:2608 | fptoi | fctiwz | melee_modes/src/tu/script_game.rs:360 | ok |
| 8007AC2C | ftColl_8007ABD0 | melee/ft/ftcoll.c:2697 | fptoi | fctiwz | melee_fighter/src/tu/ftcoll.rs:2748 | fixed |
| 8007BE70 | ftColl_8007BE3C | melee/ft/ftcoll.c:3352 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:438 | ok |
| 8007DAAC | ftCommon_CalcHitlag | melee/ft/ftcommon.c:749 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:459 | ok |
| 8007DAD0 | ftCommon_CalcHitlag | melee/ft/ftcommon.c:750 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:459 | ok |
| 8007DAFC | ftCommon_CalcHitlag | melee/ft/ftcommon.c:752 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:459 | ok |
| 8007ED94 | ftCommon_8007ED50 | melee/ft/ftcommon.c:1340 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:449 | ok |
| 8007EE50 | ftCommon_8007EE0C | melee/ft/ftcommon.c:1350 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:454 | ok |
| 8007F0F4 | ftCommon_8007EFC8 | melee/ft/ftcommon.c:1406 | fptoi | fctiwz | melee_chara/src/adapt.rs:65 | ok |
| 8007FA94 | ftCommon_8007FA58 | melee/ft/ftcommon.c:1758 | fptoi | fctiwz |  | no-port |
| 8007FB60 | ftCommon_8007FA58 | melee/ft/ftcommon.c:1770 | fptoi | fctiwz |  | no-port |
| 8007FCBC | ftCommon_8007FC7C | melee/ft/ftcommon.c:1788 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:422 | ok |
| 8007FD18 | ftCommon_8007FC7C | melee/ft/ftcommon.c:1795 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:422 | ok |
| 8007FE88 | ftCommon_8007FE84 | melee/ft/ftcommon.c:1849 | fptoi | fctiwz |  | no-port |
| 80080028 | ftCommon_8007FFD8 | melee/ft/ftcommon.c:1870 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:361 | ok |
| 80080108 | ftCommon_8007FFD8 | melee/ft/ftcommon.c:1888 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:361 | ok |
| 80086244 | ftLib_FindLowestPercentOpponent | melee/ft/ftlib.c:112 | fptoi | fctiwz |  | no-port |
| 8008712C | ftLib_GetPercent | melee/ft/ftlib.c:826 | fptoi | fctiwz |  | no-port |
| 800871F8 | ftLib_ApplyMetalBox | melee/ft/ftlib.c:845 | fptoi | fctiwz |  | no-port |
| 8008720C | ftLib_ApplyMetalBox | melee/ft/ftlib.c:845 | fptoi | fctiwz |  | no-port |
| 80087C40 | ft_GetFtKindMask | melee/ft/ft_0877.c:254 | shift | slw | melee_runtime/src/tu/lbsnap.rs:49 | ok |
| 80089520 | ft_80089460 | melee/ft/ft_0892.c:103 | fptoi | fctiwz | melee_fighter/src/tu/ft_0892.rs:25 | ok |
| 80089698 | ft_800895E0 | melee/ft/ft_0892.c:148 | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:266 | ok |
| 8008B594 | ftCo_AttackDash_SetMv0 | melee/ft/kinds/ftCommon/ftCo_AttackDash.c:81 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_AttackDash.rs:108 | fixed |
| 8008D6A8 | ftCo_LandingAir_EnterWithLag | melee/ft/kinds/ftCommon/ftCo_LandingAir.c:60 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_LandingAir.rs:68 | fixed |
| 8008DD34 | ftCo_8008DCE0 | melee/ft/kinds/ftCommon/ftCo_Damage.c:293 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyheavylanding.rs:49 | ok |
| 8008DD70 | ftCo_8008DCE0 | melee/ft/kinds/ftCommon/ftCo_Damage.c:297 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyheavylanding.rs:49 | ok |
| 8009060C | ftCo_80090594 | melee/ft/kinds/ftCommon/ftCo_DamageFall.c | fptoi | fctiwz | melee_fighter/src/tu/fighter.rs:448 | ok |
| 80091BAC | ftCo_80091B9C | melee/ft/kinds/ftCommon/ftCo_Guard.c:146 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:233 | ok |
| 80092100 | ftCo_80091E78 | melee/ft/kinds/ftCommon/ftCo_Guard.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:381 | ok |
| 80092124 | ftCo_80091E78 | melee/ft/kinds/ftCommon/ftCo_Guard.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:381 | ok |
| 80093744 | ftCo_8009370C | melee/ft/kinds/ftCommon/ftCo_Guard.c:1003 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:859 | ok |
| 80093960 | ftCo_8009388C | melee/ft/kinds/ftCommon/ftCo_Guard.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:893 | ok |
| 80093B60 | ftCo_80093A50 | melee/ft/kinds/ftCommon/ftCo_Guard.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Guard.rs:924 | ok |
| 8009C688 | ftCo_8009C640 | melee/ft/kinds/ftCommon/ftCo_Shouldered.c:49 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Shouldered.rs:85 | fixed |
| 8009ED24 | ftCo_8009EC70 | melee/ft/kinds/ftCommon/ftCo_BarrelWait.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_BarrelWait.rs:103 | ok |
| 800A1258 | ftCo_800A101C | melee/ft/kinds/ftCommon/ftCo_0A01.c:751 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:573 | fixed |
| 800A399C | ftCo_800A3908 | melee/ft/kinds/ftCommon/ftCo_0A01.c:2026 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:1798 | fixed |
| 800A3AA0 | ftCo_800A3908 | melee/ft/kinds/ftCommon/ftCo_0A01.c:2061 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:1828 | fixed |
| 800A40CC | ftCo_800A4038 | melee/ft/kinds/ftCommon/ftCo_0A01.c:2199 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:1938 | fixed |
| 800A41D0 | ftCo_800A4038 | melee/ft/kinds/ftCommon/ftCo_0A01.c:2232 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:1968 | fixed |
| 800A4E54 | ftCo_800A4BEC | melee/ft/kinds/ftCommon/ftCo_0A01.c:2519 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:2233 | fixed |
| 800A8F5C | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A8F88 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A8FC8 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A8FF4 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9034 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9060 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A90A0 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A90CC | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9110 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A913C | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A917C | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A91A8 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A91E8 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9214 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9254 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9280 | ftCo_800A8EB0 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:3938 | ok |
| 800A9BAC | ftCo_800A9904 | melee/ft/kinds/ftCommon/ftCo_0A01.c:4604 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4176 | fixed |
| 800AA2A8 | ftCo_800A9CB4 | melee/ft/kinds/ftCommon/ftCo_0A01.c:4792 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4354 | fixed |
| 800AA73C | ftCo_800AA42C | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4480 | ok |
| 800AA7A4 | ftCo_800AA42C | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4480 | ok |
| 800AA7D4 | ftCo_800AA42C | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4480 | ok |
| 800ABEC0 | ftCo_800ABBA8 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5487 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4986 | fixed |
| 800ABF00 | ftCo_800ABBA8 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5495 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:4994 | fixed |
| 800AC4BC | ftCo_800AC434 | melee/ft/kinds/ftCommon/ftCo_0A01.c | div | divw | melee_fighter/src/tu/ftCo_0A01.rs:5143 | ok-const |
| 800AC700 | ftCo_800AC5A0 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5705 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:5168 | ok |
| 800AC704 | ftCo_800AC5A0 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5706 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:5168 | ok |
| 800AC72C | ftCo_800AC5A0 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5708 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:5168 | ok |
| 800AC730 | ftCo_800AC5A0 | melee/ft/kinds/ftCommon/ftCo_0A01.c:5709 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:5168 | ok |
| 800B0984 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c:7360 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B099C | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c:7361 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B09CC | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B09E4 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0A14 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0A2C | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0A5C | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0A74 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0A90 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B0AA8 | ftCo_800B0918 | melee/ft/kinds/ftCommon/ftCo_0A01.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6755 | ok |
| 800B10E4 | ftCo_800B101C | melee/ft/kinds/ftCommon/ftCo_0A01.c:7607 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:6993 | fixed |
| 800B3688 | ftCo_800B33B0 | melee/ft/kinds/ftCommon/ftCo_0A01.c:8566 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_0A01.rs:7975 | fixed |
| 800B4214 | ftCo_800B3E04 | melee/ft/ftcmdscript.c:187 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B423C | ftCo_800B3E04 | melee/ft/ftcmdscript.c:188 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B42D8 | ftCo_800B3E04 | melee/ft/ftcmdscript.c:206 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B4300 | ftCo_800B3E04 | melee/ft/ftcmdscript.c:207 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B4408 | ftCo_800B3E04 | melee/ft/ftcmdscript.c:250 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B4438 | ftCo_800B3E04 | melee/ft/ftcmdscript.c:251 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B449C | ftCo_800B3E04 | melee/ft/ftcmdscript.c:258 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B44CC | ftCo_800B3E04 | melee/ft/ftcmdscript.c:259 | fptoi | fctiwz | melee_fighter/src/tu/ftcmdscript.rs:302 | ok |
| 800B501C | ftCo_800B4AB0 | melee/ft/ftcpuattack.c:340 | div | divw | melee_fighter/src/tu/ftcpuattack.rs:387 | ok |
| 800B581C | ftCo_800B52AC | melee/ft/ftcpuattack.c:551 | div | divw | melee_fighter/src/tu/ftcpuattack.rs:393 | ok |
| 800B5FD4 | ftCo_800B5AB0 | melee/ft/ftcpuattack.c:736 | div | divw | melee_fighter/src/tu/ftcpuattack.rs:399 | ok |
| 800B6424 | ftCo_800B63D8 | melee/ft/ftcpuattack.c:860 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B644C | ftCo_800B63D8 | melee/ft/ftcpuattack.c:863 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B6474 | ftCo_800B63D8 | melee/ft/ftcpuattack.c:866 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B649C | ftCo_800B63D8 | melee/ft/ftcpuattack.c:869 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B64C4 | ftCo_800B63D8 | melee/ft/ftcpuattack.c:872 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B64EC | ftCo_800B63D8 | melee/ft/ftcpuattack.c:875 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B6514 | ftCo_800B63D8 | melee/ft/ftcpuattack.c:878 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B653C | ftCo_800B63D8 | melee/ft/ftcpuattack.c:881 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B6564 | ftCo_800B63D8 | melee/ft/ftcpuattack.c:884 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:508 | ok |
| 800B9750 | ftCo_800B9704 | melee/ft/ftcpuattack.c:2091 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1304 | ok |
| 800B9D68 | ftCo_800B9CBC | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1427 | ok |
| 800BA008 | ftCo_800B9F90 | melee/ft/ftcpuattack.c:2353 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1489 | ok |
| 800BAA78 | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BABCC | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BADEC | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BAE74 | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BAF34 | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BB014 | ftCo_800BA9A0 | melee/ft/ftcpuattack.c | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1613 | ok |
| 800BB2E8 | ftCo_800BB220 | melee/ft/ftcpuattack.c:2702 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1718 | ok |
| 800BB314 | ftCo_800BB220 | melee/ft/ftcpuattack.c:2704 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1718 | ok |
| 800BB33C | ftCo_800BB220 | melee/ft/ftcpuattack.c:2706 | fptoi | fctiwz | melee_fighter/src/tu/ftcpuattack.rs:1718 | ok |
| 800BF7E8 | ftMaterial_800BF6BC | melee/ft/ftmaterial.c:262 | div | divw | melee_fighter/src/tu/ftmaterial.rs:282 | fixed |
| 800BF7FC | ftMaterial_800BF6BC | melee/ft/ftmaterial.c:264 | div | divw | melee_fighter/src/tu/ftmaterial.rs:284 | fixed |
| 800BF860 | ftMaterial_800BF6BC | melee/ft/ftmaterial.c:274 | div | divw | melee_fighter/src/tu/ftmaterial.rs:289 | fixed |
| 800BF898 | ftMaterial_800BF6BC | melee/ft/ftmaterial.c:283 | div | divw | melee_fighter/src/tu/ftmaterial.rs:291 | fixed |
| 800C1A64 | ftCo_800C18A8 | melee/ft/kinds/ftCommon/ftCo_FlyReflect.c:203 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_FlyReflect.rs:163 | fixed |
| 800C2A54 | ftCo_800C2600 | melee/ft/ftafterimage.c:234 | fptoi | fctiwz | melee_fighter/src/tu/ftafterimage.rs:262 | ok |
| 800C2B2C | ftCo_800C2600 | melee/ft/ftafterimage.c:272 | fptoi | fctiwz | melee_fighter/src/tu/ftafterimage.rs:262 | ok |
| 800C2C40 | ftCo_800C2600 | melee/ft/ftafterimage.c:301 | fptoi | fctiwz | melee_fighter/src/tu/ftafterimage.rs:262 | ok |
| 800C7BE4 | ftCo_800C7B0C | melee/ft/ftCo_800C78B0.c:111 | fptoi | fctiwz | melee_items/src/tu/itlikelike.rs:147 | ok |
| 800C93E0 | ftCo_800C92E4 | melee/ft/kinds/ftCommon/ftCo_Barrel.c | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Barrel.rs:170 | ok |
| 800CA3F8 | ftCo_Dash_IASA | melee/ft/kinds/ftCommon/ftCo_Dash.c:119 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Dash.rs:132 | ok |
| 800CE820 | ftCo_800CE650 | melee/ft/ft_0CE3.c:156 | div | divw |  | no-port |
| 800CEA58 | ftCo_800CE650 | melee/ft/ft_0CE3.c:213 | div | divw |  | no-port |
| 800CEF94 | ft_800CEF08 | melee/ft/kinds/ftCommon/ftCo_ItemParasolOpen.c:58 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_ItemParasolOpen.rs:101 | ok |
| 800CF06C | ftCo_800CEFE0 | melee/ft/kinds/ftCommon/ftCo_ItemParasolOpen.c:72 | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialhi.rs:144 | ok |
| 800D16A8 | ftCo_800D15D0 | melee/ft/kinds/ftCommon/ftCo_KinokoGiantStart.c:66 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_KinokoGiantStart.rs:109 | fixed |
| 800D7964 | fn_800D7938 | melee/ft/kinds/ftCommon/ftCo_ItemScopeStart.c:127 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_ItemScopeStart.rs:130 | ok |
| 800D7A24 | fn_800D79B4 | melee/ft/kinds/ftCommon/ftCo_ItemScopeStart.c:170 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_ItemScopeStart.rs:147 | ok |
| 800D7A98 | fn_800D79B4 | melee/ft/kinds/ftCommon/ftCo_ItemScopeStart.c:189 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_ItemScopeStart.rs:147 | ok |
| 800D9D74 | fn_800D9CE8 | melee/ft/kinds/ftCommon/ftCo_CatchPull.c:37 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_CatchPull.rs:76 | fixed |
| 800DD4C0 | ftCo_800DD4B0 | melee/ft/kinds/ftCommon/ftCo_Throw.c:170 | shift | slw | melee_fighter/src/tu/ftCo_Throw.rs:253 | ok |
| 800DDF68 | ftCo_800DDDE4 | melee/ft/kinds/ftCommon/ftCo_Throw.c:579 | fptoi | fctiwz | melee_fighter/src/tu/ftCo_Throw.rs:537 | ok |
| 800DFFF0 | ftWalkCommon_800DFEC8 | melee/ft/ftwalkcommon.c:165 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyheavywalk.rs:40 | ok |
| 800E001C | ftWalkCommon_800DFEC8 | melee/ft/ftwalkcommon.c:167 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyheavywalk.rs:40 | ok |
| 800E2CA4 | ftCa_SpecialN_Anim | melee/ft/kinds/ftCaptain/ftcaptainspecialn.c | fptoi | fctiwz | melee_chara/src/tu/ftcaptainspecialn.rs:108 | ok |
| 800E2D80 | ftCa_SpecialAirN_Anim | melee/ft/kinds/ftCaptain/ftcaptainspecialn.c | fptoi | fctiwz | melee_chara/src/tu/ftcaptainspecialn.rs:116 | ok |
| 800E7268 | ftFx_SpecialHi_Enter | melee/ft/kinds/ftFox/ftfoxspecialhi.c:82 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecialhi.rs:170 | fixed |
| 800E72F4 | ftFx_SpecialAirHiStart_Enter | melee/ft/kinds/ftFox/ftfoxspecialhi.c:97 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecialhi.rs:186 | fixed |
| 800E7BE0 | ftFx_SpecialAirHi_AirToGround | melee/ft/kinds/ftFox/ftfoxspecialhi.c:467 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecialhi.rs:410 | fixed |
| 800E7D8C | ftFx_SpecialAirHi_Enter | melee/ft/kinds/ftFox/ftfoxspecialhi.c:529 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecialhi.rs:459 | fixed |
| 800E85B4 | ftFx_SpecialLw_Enter | melee/ft/kinds/ftFox/ftfoxspeciallw.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspeciallw.rs:172 | ok |
| 800E865C | ftFx_SpecialAirLw_Enter | melee/ft/kinds/ftFox/ftfoxspeciallw.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspeciallw.rs:179 | ok |
| 800E94B0 | ftFx_SpecialLwTurn_Check | melee/ft/kinds/ftFox/ftfoxspeciallw.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspeciallw.rs:469 | ok |
| 800E951C | ftFx_SpecialLwTurn_Check | melee/ft/kinds/ftFox/ftfoxspeciallw.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspeciallw.rs:469 | ok |
| 800E9F1C | ftFx_SpecialSStart_Enter | melee/ft/kinds/ftFox/ftfoxspecials.c:103 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecials.rs:146 | fixed |
| 800E9FA4 | ftFx_SpecialAirSStart_Enter | melee/ft/kinds/ftFox/ftfoxspecials.c:125 | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecials.rs:163 | fixed |
| 800EACA8 | ftFx_SpecialSEnd_Enter | melee/ft/kinds/ftFox/ftfoxspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecials.rs:359 | ok |
| 800EAD34 | ftFx_SpecialAirSEnd_Enter | melee/ft/kinds/ftFox/ftfoxspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftfoxspecials.rs:372 | ok |
| 800EF0A4 | ftKb_SpecialN_800EF040 | melee/ft/kinds/ftKirby/ftkirby.c:2962 | shift | slw | melee_chara/src/tu/ftkirby.rs:1602 | fixed |
| 800EF82C | ftKb_SpecialN_800EF69C | melee/ft/kinds/ftKirby/ftkirby.c | shift | slw | melee_chara/src/tu/ftkirby.rs:1826 | ok-const |
| 800F5BFC | ftKb_SpecialN_800F5BA4 | melee/ft/kinds/ftKirby/ftkirbyspecialn.c:253 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialn.rs:438 | fixed |
| 800F5C80 | ftKb_SpecialN_800F5C34 | melee/ft/kinds/ftKirby/ftkirbyspecialn.c:265 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialn.rs:453 | fixed |
| 800FA6A0 | ftKb_SpecialNKp_800FA588 | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:66 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialkoopa.rs:145 | fixed |
| 800FA6A8 | ftKb_SpecialNKp_800FA588 | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:66 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialkoopa.rs:146 | fixed |
| 800FA85C | ftKb_SpecialNKp_800FA83C | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:114 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialkoopa.rs:197 | fixed |
| 800FA898 | ftKb_SpecialNKp_800FA878 | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:126 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialkoopa.rs:210 | fixed |
| 800FAAF0 | ftKb_KpSpecialN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:220 | div | divw | melee_chara/src/tu/ftkirbyspecialkoopa.rs:279 | ok |
| 800FAC48 | ftKb_KpSpecialAirN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c:271 | div | divw | melee_chara/src/tu/ftkirbyspecialkoopa.rs:294 | ok |
| 800FCC4C | ftKb_SpecialNSs_800FCC14 | melee/ft/kinds/ftKirby/ftkirbyspecialsamus.c:48 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialsamus.rs:80 | fixed |
| 800FD2E0 | ftKb_SsSpecialNHold_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialsamus.c:294 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialsamus.rs:276 | fixed |
| 800FD378 | ftKb_SsSpecialNHold_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialsamus.c:303 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialsamus.rs:276 | fixed |
| 800FFD64 | ftKb_DkSpecialN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c:172 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialdonkey.rs:215 | ok |
| 800FFDA4 | ftKb_DkSpecialN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c:180 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialdonkey.rs:215 | ok |
| 801001E4 | ftKb_DkSpecialAirN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c:320 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialdonkey.rs:292 | ok |
| 80100224 | ftKb_DkSpecialAirN_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialdonkey.c:328 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialdonkey.rs:292 | ok |
| 80101060 | ftKb_SpecialNPr_80100F94 | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:142 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:391 | fixed |
| 80101094 | ftKb_SpecialNPr_80100F94 | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:145 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:393 | fixed |
| 801013D4 | ftKb_SpecialNPr_8010131C | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:229 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:450 | fixed |
| 8010180C | ftKb_PrSpecialNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:338 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:606 | ok |
| 80101844 | ftKb_PrSpecialNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:340 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:606 | ok |
| 801019BC | ftKb_PrSpecialNFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:362 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:611 | ok |
| 801019F4 | ftKb_PrSpecialNFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:364 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:611 | ok |
| 80102128 | ftKb_PrSpecialNTurn_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:455 | div | divw | melee_chara/src/tu/ftkirbyspecialpurin.rs:721 | fixed |
| 801025A8 | ftKb_PrSpecialAirNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:504 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:616 | ok |
| 801025E0 | ftKb_PrSpecialAirNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:506 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:616 | ok |
| 80102758 | ftKb_PrSpecialAirNFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:528 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:621 | ok |
| 80102790 | ftKb_PrSpecialAirNFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:530 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:621 | ok |
| 80103F40 | ftKb_PrSpecialN1_Phys | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:858 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:967 | fixed |
| 80104490 | ftKb_PrSpecialAirN_Phys | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:991 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:1043 | fixed |
| 80104B9C | ftKb_PrSpecialN1_Coll | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:1119 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:1145 | fixed |
| 80105380 | ftKb_PrSpecialAirN_Coll | melee/ft/kinds/ftKirby/ftkirbyspecialpurin.c:1252 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpurin.rs:1244 | fixed |
| 80106F7C | ftKb_SpecialNMt_80106F44 | melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c:47 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialmewtwo.rs:126 | fixed |
| 80107A18 | ftKb_MtSpecialNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c:445 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialmewtwo.rs:452 | ok |
| 80107A9C | ftKb_MtSpecialNLoopFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c:463 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialmewtwo.rs:468 | fixed |
| 80107EA0 | ftKb_MtSpecialAirNLoop_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c:569 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialmewtwo.rs:457 | ok |
| 80107F24 | ftKb_MtSpecialAirNLoopFull_Anim | melee/ft/kinds/ftKirby/ftkirbyspecialmewtwo.c:587 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialmewtwo.rs:479 | fixed |
| 8010C538 | ftKb_SpecialNPe_8010C4D4 | melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpeach.rs:233 | ok |
| 8010C5D4 | ftKb_SpecialNPe_8010C560 | melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpeach.rs:242 | ok |
| 8010CDA8 | fn_8010CD88 | melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c:372 | fptoi | fctiwz | melee_chara/src/tu/ftkirbyspecialpeach.rs:403 | fixed |
| 8010EA00 | ftDk_SpecialN_Anim | melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c:166 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyspecialn.rs:163 | ok |
| 8010EA40 | ftDk_SpecialN_Anim | melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c:174 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyspecialn.rs:163 | ok |
| 8010EE94 | ftDk_SpecialAirN_Anim | melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c:284 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyspecialn.rs:235 | ok |
| 8010EED4 | ftDk_SpecialAirN_Anim | melee/ft/kinds/ftDonkey/ftdonkeyspecialn.c:292 | fptoi | fctiwz | melee_chara/src/tu/ftdonkeyspecialn.rs:235 | ok |
| 80110D98 | ftSk_SpecialS_80110BCC | melee/ft/kinds/ftSeak/ftseakspecials.c:375 | fptoi | fctiwz | melee_chara/src/tu/ftseakspecials.rs:603 | fixed |
| 801111B4 | ftSk_SpecialS_CheckInitChain | melee/ft/kinds/ftSeak/ftseakspecials.c:544 | fptoi | fctiwz | melee_chara/src/tu/ftseakspecials.rs:766 | fixed |
| 80115C84 | ftNs_AttackHi4_YoyoStartTimedRehit | melee/ft/kinds/ftNess/ftnessattackhi4.c:616 | fptoi | fctiwz | melee_chara/src/tu/ftnessattackhi4.rs:849 | fixed |
| 80119E44 | ftNs_SpecialLwStart_Enter | melee/ft/kinds/ftNess/ftnessspeciallw.c:47 | fptoi | fctiwz | melee_chara/src/tu/ftnessspeciallw.rs:124 | ok |
| 80119EC0 | ftNs_SpecialAirLwStart_Enter | melee/ft/kinds/ftNess/ftnessspeciallw.c:69 | fptoi | fctiwz | melee_chara/src/tu/ftnessspeciallw.rs:134 | ok |
| 8011B150 | ftNs_AbsorbThink_DecideAction | melee/ft/kinds/ftNess/ftnessspeciallw.c:874 | fptoi | fctiwz | melee_chara/src/tu/ftnessspeciallw.rs:545 | fixed |
| 8011B198 | ftNs_AbsorbThink_DecideAction | melee/ft/kinds/ftNess/ftnessspeciallw.c:889 | fptoi | fctiwz | melee_chara/src/tu/ftnessspeciallw.rs:554 | fixed |
| 8011B1B0 | ftNs_AbsorbThink_DecideAction | melee/ft/kinds/ftNess/ftnessspeciallw.c:890 | fptoi | fctiwz | melee_chara/src/tu/ftnessspeciallw.rs:555 | fixed |
| 8011D944 | ftPe_SpecialHiStart_Anim | melee/ft/kinds/ftPeach/ftpeachspecialhi.c:206 | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialhi.rs:384 | fixed |
| 8011D9F4 | ftPe_SpecialAirHiStart_Anim | melee/ft/kinds/ftPeach/ftpeachspecialhi.c | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialhi.rs:393 | ok |
| 8011E434 | ftPe_SpecialN_Enter | melee/ft/kinds/ftPeach/ftpeachspecialn.c | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialn.rs:256 | ok |
| 8011E4D0 | ftPe_SpecialAirN_Enter | melee/ft/kinds/ftPeach/ftpeachspecialn.c | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialn.rs:266 | ok |
| 8011EC8C | onUnkHit | melee/ft/kinds/ftPeach/ftpeachspecialn.c:382 | fptoi | fctiwz | melee_chara/src/tu/ftpeachspecialn.rs:429 | fixed |
| 80122DB8 | fn_80122D2C | melee/ft/kinds/ftPopo/ftpopospeciallw.c:187 | fptoi | fctiwz | melee_chara/src/tu/ftpopospeciallw.rs:226 | ok |
| 801239E0 | ftNn_Init_80123954 | melee/ft/kinds/ftNana/ftnanaspecials.c:89 | fptoi | fctiwz | melee_chara/src/tu/ftnanaspecials.rs:163 | fixed |
| 80124E0C | ftPk_SpecialN_80124DC8 | melee/ft/kinds/ftPikachu/ftpikachuspecialn.c:216 | fptoi | fctiwz | melee_chara/src/tu/ftpikachuspecialn.rs:181 | fixed |
| 801273F8 | ftPk_SpecialAirHiEnd_Coll | melee/ft/kinds/ftPikachu/ftpikachuspecialhi.c:760 | fptoi | fctiwz | melee_chara/src/tu/ftpikachuspecialhi.rs:665 | fixed |
| 80129138 | ftSs_SpecialLw_80129100 | melee/ft/kinds/ftSamus/ftsamusspeciallw0.c:281 | fptoi | fctiwz | melee_chara/src/tu/ftsamusspeciallw0.rs:359 | fixed |
| 80129804 | ftSs_SpecialNHold_Anim | melee/ft/kinds/ftSamus/ftsamusspecialn.c:220 | fptoi | fctiwz | melee_chara/src/tu/ftsamusspecialn.rs:314 | fixed |
| 8012989C | ftSs_SpecialNHold_Anim | melee/ft/kinds/ftSamus/ftsamusspecialn.c:230 | fptoi | fctiwz | melee_chara/src/tu/ftsamusspecialn.rs:323 | fixed |
| 8012F998 | ftYs_SpecialAirSLoop_0_Anim | melee/ft/kinds/ftYoshi/ftyoshispecials.c:465 | fptoi | fctiwz | melee_chara/src/tu/ftyoshispecials.rs:510 | fixed |
| 8012F9CC | ftYs_SpecialAirSLoop_0_Anim | melee/ft/kinds/ftYoshi/ftyoshispecials.c:467 | fptoi | fctiwz | melee_chara/src/tu/ftyoshispecials.rs:512 | fixed |
| 8012FF70 | ftYs_SpecialAirSLoop_1_Anim | melee/ft/kinds/ftYoshi/ftyoshispecials.c:584 | div | divw | melee_chara/src/tu/ftyoshispecials.rs:586 | ok |
| 801308F8 | ftYs_SpecialAirSLoop_3_Anim | melee/ft/kinds/ftYoshi/ftyoshispecials.c:692 | fptoi | fctiwz | melee_chara/src/tu/ftyoshispecials.rs:709 | fixed |
| 8013092C | ftYs_SpecialAirSLoop_3_Anim | melee/ft/kinds/ftYoshi/ftyoshispecials.c:698 | fptoi | fctiwz | melee_chara/src/tu/ftyoshispecials.rs:715 | fixed |
| 80133C88 | ftKp_SpecialSHit_IASA | melee/ft/kinds/ftKoopa/ftkoopaspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecials.rs:468 | ok |
| 80133DB4 | ftKp_SpecialAirSHit_IASA | melee/ft/kinds/ftKoopa/ftkoopaspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecials.rs:482 | ok |
| 80133EEC | ftKp_SpecialSWait_IASA | melee/ft/kinds/ftKoopa/ftkoopaspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecials.rs:496 | ok |
| 801340D8 | ftKp_SpecialAirSWait_IASA | melee/ft/kinds/ftKoopa/ftkoopaspecials.c | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecials.rs:517 | ok |
| 80134BCC | ftKp_SpecialLw_80134ACC | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c:70 | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecialn.rs:155 | fixed |
| 80134BD4 | ftKp_SpecialLw_80134ACC | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c:70 | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecialn.rs:156 | fixed |
| 80134E00 | ftKp_SpecialLw_80134DE0 | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c:124 | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecialn.rs:199 | fixed |
| 80134E3C | ftKp_SpecialLw_80134E1C | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c:136 | fptoi | fctiwz | melee_chara/src/tu/ftkoopaspecialn.rs:212 | fixed |
| 80135018 | ftKp_SpecialN_Anim | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c | div | divw | melee_chara/src/tu/ftkoopaspecialn.rs:261 | ok-const |
| 8013514C | ftKp_SpecialAirN_Anim | melee/ft/kinds/ftKoopa/ftkoopaspecialn.c | div | divw | melee_chara/src/tu/ftkoopaspecialn.rs:269 | ok-const |
| 80139198 | ftMs_SpecialLw_80139140 | melee/ft/kinds/ftMars/ftmarsspeciallw.c:329 | fptoi | fctiwz | melee_chara/src/tu/ftmarsspeciallw.rs:243 | fixed |
| 8013D9B0 | ftPr_SpecialS_8013D8E4 | melee/ft/kinds/ftPurin/ftpurinspecialn.c:169 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:214 | fixed |
| 8013D9E4 | ftPr_SpecialS_8013D8E4 | melee/ft/kinds/ftPurin/ftpurinspecialn.c:175 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:214 | fixed |
| 8013DD1C | ftPr_SpecialS_8013DC64 | melee/ft/kinds/ftPurin/ftpurinspecialn.c:238 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:270 | fixed |
| 8013E14C | ftPr_SpecialNLoop_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:382 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:403 | ok |
| 8013E184 | ftPr_SpecialNLoop_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:384 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:403 | ok |
| 8013E2FC | ftPr_SpecialNFull_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:404 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:408 | ok |
| 8013E334 | ftPr_SpecialNFull_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:406 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:408 | ok |
| 8013EA68 | ftPr_SpecialNTurn_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:479 | div | divw | melee_chara/src/tu/ftpurinspecialn.rs:498 | ok |
| 8013EEE0 | ftPr_SpecialAirNChargeLoop_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:527 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:413 | ok |
| 8013EF18 | ftPr_SpecialAirNChargeLoop_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:529 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:413 | ok |
| 8013F090 | ftPr_SpecialAirNChargeFull_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:549 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:418 | ok |
| 8013F0C8 | ftPr_SpecialAirNChargeFull_Anim | melee/ft/kinds/ftPurin/ftpurinspecialn.c:551 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:418 | ok |
| 80140850 | ftPr_SpecialNRelease_Phys | melee/ft/kinds/ftPurin/ftpurinspecialn.c:873 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:747 | fixed |
| 80140D90 | ftPr_SpecialAirNChargeRelease_Phys | melee/ft/kinds/ftPurin/ftpurinspecialn.c:990 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:832 | fixed |
| 80141484 | ftPr_SpecialNRelease_Coll | melee/ft/kinds/ftPurin/ftpurinspecialn.c:1150 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:935 | fixed |
| 80141C40 | ftPr_SpecialAirNChargeRelease_Coll | melee/ft/kinds/ftPurin/ftpurinspecialn.c:1318 | fptoi | fctiwz | melee_chara/src/tu/ftpurinspecialn.rs:1019 | fixed |
| 80142AAC | ftLg_SpecialS_SetVars | melee/ft/kinds/ftLuigi/ftluigispecials.c:46 | fptoi | fctiwz | melee_chara/src/tu/ftluigispecials.rs:157 | fixed |
| 80142AD4 | ftLg_SpecialS_SetVars | melee/ft/kinds/ftLuigi/ftluigispecials.c:52 | fptoi | fctiwz | melee_chara/src/tu/ftluigispecials.rs:162 | fixed |
| 80146D04 | ftMt_SpecialN_GetChargeLevel | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c:54 | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:129 | ok |
| 801477E4 | ftMt_SpecialNLoop_Anim | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c:567 | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:552 | ok |
| 80147868 | ftMt_SpecialNLoopFull_Anim | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c:588 | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:557 | ok |
| 80147C84 | ftMt_SpecialAirNLoop_Anim | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c:710 | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:623 | ok |
| 80147D08 | ftMt_SpecialAirNLoopFull_Anim | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c:732 | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:628 | ok |
| 80148C1C | ftMt_SpecialN_Shoot | melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c | fptoi | fctiwz | melee_chara/src/tu/ftmewtwospecialn.rs:789 | ok |
| 8014D224 | ftGw_SpecialLw_IASA | melee/ft/kinds/ftGameWatch/ftgamewatchspeciallw.c:278 | fptoi | fctiwz | melee_chara/src/tu/ftgamewatchspeciallw.rs:338 | ok |
| 8014D2DC | ftGw_SpecialAirLw_IASA | melee/ft/kinds/ftGameWatch/ftgamewatchspeciallw.c:314 | fptoi | fctiwz | melee_chara/src/tu/ftgamewatchspeciallw.rs:344 | ok |
| 80151544 | ftMh_MS_343_80151484 | melee/ft/kinds/ftMasterHand/ftmasterhanddamage0.c | fptoi | fctiwz | melee_chara/src/tu/ftmasterhanddamage0.rs:152 | ok |
| 801530FC | ftMh_MS_363_801530A4 | melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c:55 | fptoi | fctiwz | melee_chara/src/tu/ftmasterhandfingergun.rs:89 | fixed |
| 80154334 | ftMh_MS_372_801542E0 | melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c:51 | fptoi | fctiwz | melee_chara/src/tu/ftmasterhandbackdisappear.rs:138 | fixed |
| 80158B90 | ftCh_Init_80158B3C | melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c:57 | fptoi | fctiwz | melee_chara/src/tu/ftcrazyhandfingerbeam.rs:31 | fixed |
| 8015AE90 | ftCh_GrabUnk1_8015ADD0 | melee/ft/kinds/ftCrazyHand/ftcrazyhandtaggrab.c | fptoi | fctiwz | melee_chara/src/tu/ftcrazyhandtaggrab.rs:88 | ok |
| 8015BD60 | ftBossLib_8015BD24 | melee/ft/ftbosslib.c:39 | div | divw | melee_fighter/src/tu/ftbosslib.rs:52 | ok |
| 8015CFF0 | gmMainLib_8015CFCC | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:142 | ok-const |
| 8015D048 | gmMainLib_8015D00C | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gmmain_lib.rs:371 | ok-const |
| 8015D118 | gmMainLib_8015D0F4 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:143 | ok-const |
| 8015D170 | gmMainLib_8015D134 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_17c0.rs:171 | ok-const |
| 8015D240 | gmMainLib_8015D21C | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:146 | ok-const |
| 8015D298 | gmMainLib_8015D25C | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_17c0.rs:175 | ok-const |
| 8015D368 | gmMainLib_8015D344 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:149 | ok-const |
| 8015D3C0 | gmMainLib_8015D384 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_17c0.rs:179 | ok-const |
| 8015D4CC | gmMainLib_8015D4A8 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:151 | ok-const |
| 8015D604 | gmMainLib_8015D5DC | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:153 | ok-const |
| 8015D668 | gmMainLib_8015D640 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_1736.rs:45 | ok-const |
| 8015D8A0 | gmMainLib_8015D888 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gmmain_lib.rs:626 | ok-const |
| 8015D8C8 | gmMainLib_8015D8B0 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:156 | ok-const |
| 8015D8F0 | gmMainLib_8015D8D8 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:157 | ok-const |
| 8015D914 | gmMainLib_8015D8FC | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:158 | ok-const |
| 8015D93C | gmMainLib_8015D924 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:159 | ok-const |
| 8015D964 | gmMainLib_8015D94C | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:160 | ok-const |
| 8015DA0C | gmMainLib_8015D9F4 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gmmain_lib.rs:674 | ok-const |
| 8015DA34 | gmMainLib_8015DA1C | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:162 | ok-const |
| 8015DA58 | gmMainLib_8015DA40 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_1736.rs:47 | ok-const |
| 8015DA80 | gmMainLib_8015DA68 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_16f1.rs:163 | ok-const |
| 8015DAA8 | gmMainLib_8015DA90 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_1BFA.rs:237 | ok-const |
| 8015DACC | gmMainLib_8015DAB4 | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gmmain_lib.rs:699 | ok-const |
| 8015DAF4 | gmMainLib_8015DADC | melee/gm/gmmain_lib.c | shift | slw | melee_modes/src/tu/gm_1736.rs:49 | ok-const |
| 80160228 | gm_801601C4 | melee/gm/gm_1601.c:638 | fptoi | fctiwz | melee_modes/src/tu/mnsoundtest.rs:74 | ok |
| 801602AC | gm_80160244 | melee/gm/gm_1601.c:655 | fptoi | fctiwz | melee_modes/src/tu/mnsoundtest.rs:75 | ok |
| 80160334 | gm_801602C0 | melee/gm/gm_1601.c | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:558 | ok |
| 80160378 | gm_801602C0 | melee/gm/gm_1601.c | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:558 | ok |
| 801634B0 | gm_80163374 | melee/gm/gm_1601.c:2271 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1566 | ok |
| 80163634 | gm_801634D4 | melee/gm/gm_1601.c:2302 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1619 | ok |
| 80163814 | gm_801636D8 | melee/gm/gm_1601.c:2342 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1640 | ok |
| 80163998 | gm_80163838 | melee/gm/gm_1601.c:2373 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1648 | ok |
| 80163B78 | gm_80163A3C | melee/gm/gm_1601.c:2413 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1669 | ok |
| 80163CFC | gm_80163B9C | melee/gm/gm_1601.c:2444 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:1677 | ok |
| 80164210 | gm_801641E4 | melee/gm/gm_1601.c:2599 | shift | slw | melee_modes/src/tu/gm_1601.rs:1796 | ok |
| 80164230 | gm_801641E4 | melee/gm/gm_1601.c:2601 | shift | slw | melee_modes/src/tu/gm_1601.rs:1796 | ok |
| 80164274 | gm_IsStageUnlocked | melee/gm/gm_1601.c:2607 | shift | slw | melee_modes/src/tu/gm_1601.rs:1805 | ok |
| 801642D0 | fn_801642A0 | melee/gm/gm_1601.c:2621 | shift | slw | melee_modes/src/tu/gm_1601.rs:1813 | ok |
| 801643A4 | gm_80164330 | melee/gm/gm_1601.c:2654 | shift | slw | melee_modes/src/tu/gm_1601.rs:1829 | ok |
| 801643FC | gm_80164330 | melee/gm/gm_1601.c:2666 | shift | slw | melee_modes/src/tu/gm_1601.rs:1829 | ok |
| 80166550 | gm_80166378 | melee/gm/gm_1601.c:3440 | fptoi | fctiwz | melee_modes/src/tu/gm_1601.rs:2339 | fixed |
| 80169718 | fn_801695BC | melee/gm/gm_16A2.c:180 | div | divw | melee_modes/src/tu/gm_16A2.rs:209 | fixed |
| 80169738 | fn_801695BC | melee/gm/gm_16A2.c:183 | div | divw | melee_modes/src/tu/gm_16A2.rs:212 | fixed |
| 80169854 | fn_801697FC | melee/gm/gm_16A2.c:225 | div | divw | melee_modes/src/tu/gm_16A2.rs:248 | fixed |
| 8016AF50 | gm_8016AF0C | melee/gm/gmvs.c:128 | fptoi | fctiwz | melee_modes/src/tu/gmscene.rs:159 | ok |
| 80172AEC | gm_801729EC | melee/gm/gm_16F1.c:2078 | shift | slw | melee_modes/src/tu/gm_16f1.rs:1478 | fixed |
| 80174A20 | fn_801749B8 | melee/gm/gmresult.c:356 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:565 | fixed |
| 80174A24 | fn_801749B8 | melee/gm/gmresult.c:357 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:567 | fixed |
| 80174C70 | fn_80174B4C | melee/gm/gmresult.c:540 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:630 | fixed |
| 80175174 | fn_8017507C | melee/gm/gmresult.c:672 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:750 | fixed |
| 801751BC | fn_8017507C | melee/gm/gmresult.c:678 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:756 | fixed |
| 80175204 | fn_8017507C | melee/gm/gmresult.c:684 | fptoi | fctiwz | melee_modes/src/tu/gmresult.rs:762 | fixed |
| 80178C70 | fn_80178BB4 | melee/gm/gmresultplayer.c:1405 | fptoi | fctiwz | melee_modes/src/tu/gmresultplayer.rs:1499 | fixed |
| 80178D84 | fn_80178BB4 | melee/gm/gmresultplayer.c:1419 | fptoi | fctiwz | melee_modes/src/tu/gmresultplayer.rs:1499 | fixed |
| 80178EE0 | fn_80178BB4 | melee/gm/gmresultplayer.c | fptoi | fctiwz | melee_modes/src/tu/gmresultplayer.rs:1499 | fixed |
| 80178F78 | fn_80178BB4 | melee/gm/gmresultplayer.c | fptoi | fctiwz | melee_modes/src/tu/gmresultplayer.rs:1499 | fixed |
| 80178FDC | fn_80178BB4 | melee/gm/gmresultplayer.c | fptoi | fctiwz | melee_modes/src/tu/gmresultplayer.rs:1499 | fixed |
| 8017CDF4 | gm_8017CD94 | melee/gm/gm_17C0.c:700 | div | divw | melee_modes/src/tu/gm_17c0.rs:823 | fixed |
| 8017D210 | gm_8017CE34 | melee/gm/gm_17C0.c | div | divw | melee_modes/src/tu/gm_17c0.rs:862 | ok-const |
| 8017F7E0 | fn_8017F608 | melee/gm/gmregclear.c:384 | div | divw | melee_modes/src/tu/gmregclear.rs:424 | ok |
| 8017F8C0 | fn_8017F608 | melee/gm/gmregclear.c:405 | div | divw | melee_modes/src/tu/gmregclear.rs:424 | ok |
| 8017FE98 | fn_8017FE54 | melee/gm/gmregclear.c:610 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:679 | fixed |
| 80180230 | fn_8017FF1C | melee/gm/gmregclear.c:719 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:772 | fixed |
| 80180C98 | fn_80180C60 | melee/gm/gm_180A.c:115 | fptoi | fctiwz | melee_modes/src/tu/gm_180A.rs:126 | fixed |
| 80180F4C | fn_80180C60 | melee/gm/gm_180A.c:187 | fptoi | fctiwz | melee_modes/src/tu/gm_180A.rs:196 | fixed |
| 801858A4 | fn_801857C4 | melee/gm/gm_1832.c:726 | fptoi | fctiwz | melee_modes/src/tu/gm_1832.rs:894 | fixed |
| 801858BC | fn_801857C4 | melee/gm/gm_1832.c:727 | div | divw | melee_modes/src/tu/gm_1832.rs:895 | fixed |
| 801858C4 | fn_801857C4 | melee/gm/gm_1832.c:731 | div | divw | melee_modes/src/tu/gm_1832.rs:899 | fixed |
| 80185B68 | fn_80185A0C | melee/gm/gm_1832.c | div | divw | melee_modes/src/tu/gm_1832.rs:963 | fixed |
| 80185BB0 | fn_80185A0C | melee/gm/gm_1832.c | div | divw | melee_modes/src/tu/gm_1832.rs:964 | fixed |
| 80185BF8 | fn_80185A0C | melee/gm/gm_1832.c | div | divw | melee_modes/src/tu/gm_1832.rs:966 | fixed |
| 80185C40 | fn_80185A0C | melee/gm/gm_1832.c | div | divw | melee_modes/src/tu/gm_1832.rs:966 | fixed |
| 80185C88 | fn_80185A0C | melee/gm/gm_1832.c | div | divw | melee_modes/src/tu/gm_1832.rs:966 | fixed |
| 80187974 | fn_80187910 | melee/gm/gm_1879.c:91 | fptoi | fctiwz | melee_modes/src/tu/gm_1879.rs:112 | fixed |
| 8018AE00 | fn_8018AA74 | melee/gm/gmtoulib.c | div | divw | melee_modes/src/tu/gmtoulib.rs:548 | ok-const |
| 8018E51C | fn_8018E46C | melee/gm/gmtoulib.c:1714 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1327 | ok |
| 8018E560 | fn_8018E46C | melee/gm/gmtoulib.c:1718 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1327 | ok |
| 8018E5A4 | fn_8018E46C | melee/gm/gmtoulib.c:1722 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1327 | ok |
| 8018E5E8 | fn_8018E46C | melee/gm/gmtoulib.c:1726 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1327 | ok |
| 8018FDF0 | fn_8018FDC4 | melee/gm/gmtoulib.c:2407 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1866 | ok |
| 8018FE74 | fn_8018FDC4 | melee/gm/gmtoulib.c:2410 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1866 | ok |
| 8018FEF8 | fn_8018FDC4 | melee/gm/gmtoulib.c:2413 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1866 | ok |
| 8018FFC8 | fn_8018FF9C | melee/gm/gmtoulib.c:2420 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1879 | ok |
| 8019004C | fn_8018FF9C | melee/gm/gmtoulib.c:2423 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1879 | ok |
| 801900D0 | fn_8018FF9C | melee/gm/gmtoulib.c:2426 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1879 | ok |
| 8019048C | fn_80190480 | melee/gm/gmtoulib.c:2503 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1966 | ok |
| 80190524 | fn_80190520 | melee/gm/gmtoulib.c:2521 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1982 | ok |
| 80190558 | fn_80190520 | melee/gm/gmtoulib.c:2521 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1982 | ok |
| 8019056C | fn_80190520 | melee/gm/gmtoulib.c:2521 | fptoi | fctiwz | melee_modes/src/tu/gmtoulib.rs:1982 | ok |
| 801A0C30 | fn_801A0B60 | melee/gm/gm_19EF.c:714 | fptoi | fctiwz | melee_modes/src/tu/gm_19EF.rs:663 | fixed |
| 801A0C34 | fn_801A0B60 | melee/gm/gm_19EF.c:715 | fptoi | fctiwz | melee_modes/src/tu/gm_19EF.rs:663 | fixed |
| 801A317C | gmCamera_801A30E4 | melee/gm/gmcamera.c:541 | shift | slw | melee_modes/src/tu/gmcamera.rs:876 | fixed |
| 801A48E0 | gm_801A48A4 | melee/gm/gmscene.c | shift | sraw | melee_modes/src/tu/gmscene.rs:289 | ok-const |
| 801A4904 | gm_801A48A4 | melee/gm/gmscene.c | shift | sraw | melee_modes/src/tu/gmscene.rs:289 | ok-const |
| 801A492C | gm_801A48A4 | melee/gm/gmscene.c | shift | sraw | melee_modes/src/tu/gmscene.rs:289 | ok-const |
| 801A4954 | gm_801A48A4 | melee/gm/gmscene.c | shift | sraw | melee_modes/src/tu/gmscene.rs:289 | ok-const |
| 801B1D1C | gm_801B1C24 | melee/gm/gmtrainingmode.c:137 | div | divw | melee_modes/src/tu/gmtrainingmode.rs:176 | fixed |
| 801B1D50 | gm_801B1C24 | melee/gm/gmtrainingmode.c:141 | div | divw | melee_modes/src/tu/gmtrainingmode.rs:179 | fixed |
| 801B3C5C | gmClassic_801B3B40 | melee/gm/gmclassic.c:1101 | shift | slw | melee_modes/src/tu/gmclassic.rs:930 | ok |
| 801B6670 | gm_801B65D4 | melee/gm/gmmultiman.c:378 | shift | slw | melee_modes/src/tu/gmmultiman.rs:841 | ok |
| 801C5674 | Ground_801C5630 | melee/gr/ground.c:4057 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:90 | ok |
| 801C56E0 | Ground_801C5694 | melee/gr/ground.c:4067 | fptoi | fctiwz | melee_world/src/tu/grcastle.rs:87 | ok |
| 801CC3C0 | grIzumi_801CC358 | melee/gr/grizumi.c:443 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:594 | fixed |
| 801CC3C4 | grIzumi_801CC358 | melee/gr/grizumi.c:443 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:636 | fixed |
| 801CC5A0 | grIzumi_801CC358 | melee/gr/grizumi.c:491 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:668 | fixed |
| 801CC5A4 | grIzumi_801CC358 | melee/gr/grizumi.c:491 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:668 | fixed |
| 801CC6D4 | grIzumi_801CC358 | melee/gr/grizumi.c:529 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:668 | fixed |
| 801CC6D8 | grIzumi_801CC358 | melee/gr/grizumi.c:529 | fptoi | fctiwz | melee_world/src/tu/grizumi.rs:668 | fixed |
| 801CDB18 | grCastle_801CDA0C | melee/gr/grcastle.c:487 | fptoi | fctiwz | melee_world/src/tu/grcastle.rs:670 | fixed |
| 801CDBF8 | grCastle_801CDA0C | melee/gr/grcastle.c:504 | fptoi | fctiwz | melee_world/src/tu/grcastle.rs:686 | fixed |
| 801CF478 | grCastle_801CF308 | melee/gr/grcastle.c:1277 | fptoi | fctiwz | melee_world/src/tu/grcastle.rs:1377 | fixed |
| 801CF98C | grCastle_801CF868 | melee/gr/grcastle.c:1478 | div | divw | melee_world/src/tu/grcastle.rs:1525 | ok |
| 801D33F4 | grStadium_801D32D0 | melee/gr/grpstadium.c:1367 | fptoi | fctiwz | melee_world/src/tu/grpstadium.rs:1362 | ok |
| 801D3410 | grStadium_801D32D0 | melee/gr/grpstadium.c:1368 | fptoi | fctiwz | melee_world/src/tu/grpstadium.rs:1362 | ok |
| 801D3E40 | grStadium_801D3BBC | melee/gr/grpstadium.c | fptoi | fctiwz | melee_world/src/tu/grpstadium.rs:1642 | ok |
| 801D3E74 | grStadium_801D3BBC | melee/gr/grpstadium.c | fptoi | fctiwz | melee_world/src/tu/grpstadium.rs:1642 | ok |
| 801D3EA8 | grStadium_801D3BBC | melee/gr/grpstadium.c | fptoi | fctiwz | melee_world/src/tu/grpstadium.rs:1642 | ok |
| 801D546C | fn_801D542C | melee/gr/grkongo.c:226 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:528 | ok |
| 801D5664 | grKongo_801D55D8 | melee/gr/grkongo.c:295 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:586 | fixed |
| 801D5668 | grKongo_801D55D8 | melee/gr/grkongo.c:295 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:586 | fixed |
| 801D5984 | grKongo_801D577C | melee/gr/grkongo.c:370 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:643 | fixed |
| 801D5988 | grKongo_801D577C | melee/gr/grkongo.c:371 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:654 | fixed |
| 801D5A68 | grKongo_801D577C | melee/gr/grkongo.c:384 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:654 | fixed |
| 801D5A6C | grKongo_801D577C | melee/gr/grkongo.c:385 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:654 | fixed |
| 801D5DDC | grKongo_801D577C | melee/gr/grkongo.c:440 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:654 | fixed |
| 801D7774 | fn_801D7700 | melee/gr/grkongo.c:1085 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:1133 | fixed |
| 801D7C84 | grKongo_801D7BBC | melee/gr/grkongo.c:1196 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:1203 | ok |
| 801D81F4 | fn_801D8134 | melee/gr/grkongo.c:1412 | fptoi | fctiwz | melee_world/src/tu/grkongo.rs:1405 | ok |
| 801D8914 | grZebes_801D881C | melee/gr/grzebes.c:448 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1038 | fixed |
| 801D8A14 | grZebes_801D881C | melee/gr/grzebes.c:478 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1065 | fixed |
| 801D8AC8 | grZebes_801D881C | melee/gr/grzebes.c:500 | div | divw | melee_world/src/tu/grzebes.rs:1084 | fixed |
| 801D8D70 | grZebes_801D881C | melee/gr/grzebes.c | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1065 | fixed |
| 801D8DE0 | grZebes_801D881C | melee/gr/grzebes.c | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1065 | fixed |
| 801DA118 | grZebes_801DA0C4 | melee/gr/grzebes.c:1135 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1637 | fixed |
| 801DA12C | grZebes_801DA0C4 | melee/gr/grzebes.c:1136 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1638 | fixed |
| 801DA140 | grZebes_801DA0C4 | melee/gr/grzebes.c:1137 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1639 | fixed |
| 801DA168 | grZebes_801DA0C4 | melee/gr/grzebes.c:1143 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1645 | fixed |
| 801DA17C | grZebes_801DA0C4 | melee/gr/grzebes.c:1144 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1646 | fixed |
| 801DA190 | grZebes_801DA0C4 | melee/gr/grzebes.c:1145 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1647 | fixed |
| 801DA1E8 | grZebes_801DA0C4 | melee/gr/grzebes.c:1156 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1658 | fixed |
| 801DA208 | grZebes_801DA0C4 | melee/gr/grzebes.c:1159 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1661 | fixed |
| 801DA228 | grZebes_801DA0C4 | melee/gr/grzebes.c:1163 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1665 | fixed |
| 801DA374 | grZebes_801DA254 | melee/gr/grzebes.c:1206 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1697 | fixed |
| 801DA3A0 | grZebes_801DA254 | melee/gr/grzebes.c:1207 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1698 | fixed |
| 801DA3B0 | grZebes_801DA254 | melee/gr/grzebes.c:1208 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1699 | fixed |
| 801DA5FC | grZebes_801DA528 | melee/gr/grzebes.c:1265 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:1755 | fixed |
| 801DC69C | grZebes_801DC408 | melee/gr/grzebes.c:2224 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:2681 | fixed |
| 801DC6A0 | grZebes_801DC408 | melee/gr/grzebes.c:2225 | fptoi | fctiwz | melee_world/src/tu/grzebes.rs:2682 | fixed |
| 801DCD94 | grCorneria_801DCCFC | melee/gr/grcorneria.c:263 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:572 | fixed |
| 801DCD98 | grCorneria_801DCCFC | melee/gr/grcorneria.c:264 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:573 | fixed |
| 801DCFAC | grCorneria_801DCE1C | melee/gr/grcorneria.c:350 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:644 | fixed |
| 801DCFB0 | grCorneria_801DCE1C | melee/gr/grcorneria.c:351 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:644 | fixed |
| 801DEBC8 | grCorneria_801DE8E4 | melee/gr/grcorneria.c:1053 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:1341 | fixed |
| 801DF0D0 | grCorneria_801DED50 | melee/gr/grcorneria.c:1206 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:1455 | fixed |
| 801DF2FC | grCorneria_801DED50 | melee/gr/grcorneria.c:1264 | div | divw | melee_world/src/tu/grcorneria.rs:1513 | fixed |
| 801DF350 | grCorneria_801DED50 | melee/gr/grcorneria.c:1272 | div | divw | melee_world/src/tu/grcorneria.rs:1518 | fixed |
| 801DF56C | grCorneria_801DED50 | melee/gr/grcorneria.c:1314 | div | divw | melee_world/src/tu/grcorneria.rs:1550 | fixed |
| 801DF5C0 | grCorneria_801DED50 | melee/gr/grcorneria.c:1322 | div | divw | melee_world/src/tu/grcorneria.rs:1555 | fixed |
| 801DFB94 | grCorneria_801DF8D0 | melee/gr/grcorneria.c:1460 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:1674 | fixed |
| 801E13EC | grCorneria_801E1348 | melee/gr/grcorneria.c:2057 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2214 | fixed |
| 801E140C | grCorneria_801E1348 | melee/gr/grcorneria.c:2061 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2217 | fixed |
| 801E1410 | grCorneria_801E1348 | melee/gr/grcorneria.c:2062 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2218 | fixed |
| 801E1498 | grCorneria_801E1348 | melee/gr/grcorneria.c:2072 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2228 | fixed |
| 801E159C | grCorneria_801E1348 | melee/gr/grcorneria.c:2095 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2251 | fixed |
| 801E1750 | grCorneria_801E1348 | melee/gr/grcorneria.c:2153 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2308 | fixed |
| 801E17A4 | grCorneria_801E1348 | melee/gr/grcorneria.c:2154 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2309 | fixed |
| 801E19B0 | grCorneria_801E1970 | melee/gr/grcorneria.c:2210 | fptoi | fctiwz | melee_world/src/tu/grcorneria.rs:2357 | fixed |
| 801E329C | grStory_801E3234 | melee/gr/grstory.c | fptoi | fctiwz | melee_world/src/tu/grstory.rs:107 | ok |
| 801E32F4 | grStory_801E3234 | melee/gr/grstory.c | fptoi | fctiwz | melee_world/src/tu/grstory.rs:107 | ok |
| 801E3470 | grStory_801E3418 | melee/gr/grstory.c:272 | fptoi | fctiwz | melee_world/src/tu/grstory.rs:183 | fixed |
| 801E34C4 | grStory_801E3418 | melee/gr/grstory.c:278 | fptoi | fctiwz | melee_world/src/tu/grstory.rs:187 | fixed |
| 801E3544 | grStory_801E3418 | melee/gr/grstory.c | fptoi | fctiwz | melee_world/src/tu/grstory.rs:187 | fixed |
| 801E3600 | grStory_801E3418 | melee/gr/grstory.c | fptoi | fctiwz | melee_world/src/tu/grstory.rs:187 | fixed |
| 801E3B40 | grOnett_801E3A34 | melee/gr/gronett.c:206 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:310 | ok |
| 801E3B44 | grOnett_801E3A34 | melee/gr/gronett.c:206 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:310 | ok |
| 801E3F60 | grOnett_801E3DA0 | melee/gr/gronett.c:319 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:406 | ok |
| 801E4668 | grOnett_801E43E0 | melee/gr/gronett.c:539 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E46A8 | grOnett_801E43E0 | melee/gr/gronett.c:565 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E47C8 | grOnett_801E43E0 | melee/gr/gronett.c:568 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E47EC | grOnett_801E43E0 | melee/gr/gronett.c:616 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E4AE8 | grOnett_801E43E0 | melee/gr/gronett.c:674 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E4E40 | grOnett_801E43E0 | melee/gr/gronett.c | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E4E74 | grOnett_801E43E0 | melee/gr/gronett.c | fptoi | fctiwz | melee_world/src/tu/gronett.rs:629 | ok |
| 801E5588 | grOnett_801E5538 | melee/gr/gronett.c:889 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:965 | ok |
| 801E558C | grOnett_801E5538 | melee/gr/gronett.c:889 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:965 | ok |
| 801E5618 | grOnett_801E5538 | melee/gr/gronett.c:895 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:965 | ok |
| 801E561C | grOnett_801E5538 | melee/gr/gronett.c:895 | fptoi | fctiwz | melee_world/src/tu/gronett.rs:965 | ok |
| 801E6F04 | grBigBlue_801E6C60 | melee/gr/grbigblue.c:748 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:991 | fixed |
| 801E8FF4 | grBigBlue_801E8D64 | melee/gr/grbigblue.c:1541 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:1584 | fixed |
| 801E97D0 | grBigBlue_801E93D8 | melee/gr/grbigblue.c:1701 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:1723 | fixed |
| 801EA27C | grBigBlue_801EA05C | melee/gr/grbigblue.c:1954 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:1941 | fixed |
| 801EA7E4 | grBigBlue_801EA05C | melee/gr/grbigblue.c:2121 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:2085 | fixed |
| 801EA8E4 | grBigBlue_801EA05C | melee/gr/grbigblue.c:2148 | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:2107 | fixed |
| 801EC9A8 | grBigBlue_801EC6C0 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:2979 | ok |
| 801EC9AC | grBigBlue_801EC6C0 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:2979 | ok |
| 801EE490 | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3583 | fixed |
| 801EE494 | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3642 | fixed |
| 801EE85C | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3642 | fixed |
| 801EE860 | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3642 | fixed |
| 801EED20 | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3642 | fixed |
| 801EED24 | grBigBlue_801EE398 | melee/gr/grbigblue.c | fptoi | fctiwz | melee_world/src/tu/grbigblue.rs:3642 | fixed |
| 801F3470 | grFourside_801F3274 | melee/gr/grfourside.c | fptoi | fctiwz | melee_world/src/tu/grfourside.rs:566 | ok |
| 801F3530 | grFourside_801F3274 | melee/gr/grfourside.c | fptoi | fctiwz | melee_world/src/tu/grfourside.rs:566 | ok |
| 801F4DBC | grGreatBay_801F499C | melee/gr/grgreatbay.c:478 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:569 | fixed |
| 801F4DC0 | grGreatBay_801F499C | melee/gr/grgreatbay.c:479 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:580 | fixed |
| 801F4F04 | grGreatBay_801F499C | melee/gr/grgreatbay.c:493 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:588 | fixed |
| 801F4F08 | grGreatBay_801F499C | melee/gr/grgreatbay.c:494 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:588 | fixed |
| 801F5038 | grGreatBay_801F499C | melee/gr/grgreatbay.c:505 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:588 | fixed |
| 801F503C | grGreatBay_801F499C | melee/gr/grgreatbay.c:506 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:588 | fixed |
| 801F6130 | grGreatBay_801F60C4 | melee/gr/grgreatbay.c:897 | fptoi | fctiwz | melee_world/src/tu/grgreatbay.rs:917 | fixed |
| 801FB110 | grInishie1_801FB0AC | melee/gr/grinishie1.c:489 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:609 | fixed |
| 801FB114 | grInishie1_801FB0AC | melee/gr/grinishie1.c:489 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:615 | fixed |
| 801FB1C4 | grInishie1_801FB0AC | melee/gr/grinishie1.c:496 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:630 | fixed |
| 801FB1C8 | grInishie1_801FB0AC | melee/gr/grinishie1.c:496 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:631 | fixed |
| 801FB2A8 | grInishie1_801FB0AC | melee/gr/grinishie1.c:511 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:635 | fixed |
| 801FB2AC | grInishie1_801FB0AC | melee/gr/grinishie1.c:511 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:635 | fixed |
| 801FB328 | grInishie1_801FB0AC | melee/gr/grinishie1.c:513 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:635 | fixed |
| 801FB32C | grInishie1_801FB0AC | melee/gr/grinishie1.c:513 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:635 | fixed |
| 801FB3A4 | grInishie1_801FB0AC | melee/gr/grinishie1.c:517 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:635 | fixed |
| 801FC844 | grInishie1_801FC664 | melee/gr/grinishie1.c:1072 | fptoi | fctiwz | melee_world/src/tu/grinishie1.rs:1113 | fixed |
| 801FE9C4 | grKraid_801FE818 | melee/gr/grkraid.c:365 | fptoi | fctiwz | melee_world/src/tu/grkraid.rs:355 | ok |
| 801FEA78 | grKraid_801FEA00 | melee/gr/grkraid.c:386 | fptoi | fctiwz | melee_world/src/tu/grkraid.rs:404 | fixed |
| 801FEC28 | grKraid_801FEA00 | melee/gr/grkraid.c:424 | fptoi | fctiwz | melee_world/src/tu/grkraid.rs:442 | fixed |
| 801FEC74 | grKraid_801FEA00 | melee/gr/grkraid.c:433 | fptoi | fctiwz | melee_world/src/tu/grkraid.rs:451 | fixed |
| 801FED00 | grKraid_801FEA00 | melee/gr/grkraid.c:448 | fptoi | fctiwz | melee_world/src/tu/grkraid.rs:464 | fixed |
| 80200298 | grRCruise_80200154 | melee/gr/grrcruise.c:533 | div | divw | melee_world/src/tu/grrcruise.rs:458 | fixed |
| 80200374 | grRCruise_80200154 | melee/gr/grrcruise.c:553 | div | divw | melee_world/src/tu/grrcruise.rs:479 | fixed |
| 80200790 | grRCruise_8020071C | melee/gr/grrcruise.c:665 | fptoi | fctiwz | melee_world/src/tu/grrcruise.rs:595 | fixed |
| 80200CCC | grRCruise_80200C04 | melee/gr/grrcruise.c:788 | div | divw | melee_world/src/tu/grrcruise.rs:684 | fixed |
| 80200CEC | grRCruise_80200C04 | melee/gr/grrcruise.c:791 | fptoi | fctiwz | melee_world/src/tu/grrcruise.rs:686 | fixed |
| 80200FE4 | grRCruise_80200C04 | melee/gr/grrcruise.c:834 | div | divw | melee_world/src/tu/grrcruise.rs:728 | fixed |
| 8020276C | grYorster_8020266C | melee/gr/gryorster.c:297 | fptoi | fctiwz | melee_world/src/tu/gryorster.rs:600 | fixed |
| 802027D8 | grYorster_8020266C | melee/gr/gryorster.c:309 | fptoi | fctiwz | melee_world/src/tu/gryorster.rs:607 | fixed |
| 80202860 | grYorster_8020266C | melee/gr/gryorster.c:325 | fptoi | fctiwz | melee_world/src/tu/gryorster.rs:620 | fixed |
| 802028D4 | grYorster_8020266C | melee/gr/gryorster.c:337 | fptoi | fctiwz | melee_world/src/tu/gryorster.rs:627 | fixed |
| 802032EC | grGarden_80203250 | melee/gr/grgarden.c:305 | fptoi | fctiwz | melee_world/src/tu/grgarden.rs:395 | fixed |
| 8020381C | grVenom_8020362C | melee/gr/grvenom.c | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:827 | ok |
| 80203820 | grVenom_8020362C | melee/gr/grvenom.c | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:827 | ok |
| 80203BE0 | grVenom_80203B18 | melee/gr/grvenom.c | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:984 | ok |
| 80203BE4 | grVenom_80203B18 | melee/gr/grvenom.c | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:984 | ok |
| 80205DB8 | grVenom_80205AD4 | melee/gr/grvenom.c:1344 | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:1720 | fixed |
| 802062D0 | grVenom_80205F30 | melee/gr/grvenom.c:1519 | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:1804 | fixed |
| 80206B38 | grVenom_80206874 | melee/gr/grvenom.c:1761 | fptoi | fctiwz | melee_world/src/tu/grvenom.rs:1993 | fixed |
| 8020902C | grShrineRoute_80208F70 | melee/gr/grshrineroute.c:422 | shift | slw | melee_world/src/tu/grshrineroute.rs:373 | fixed |
| 8020957C | grShrineRoute_80208F70 | melee/gr/grshrineroute.c:586 | shift | slw | melee_world/src/tu/grshrineroute.rs:470 | fixed |
| 8020A714 | grShrineRoute_8020A21C | melee/gr/grshrineroute.c:1060 | fptoi | fctiwz | melee_world/src/tu/grshrineroute.rs:859 | fixed |
| 8020A73C | grShrineRoute_8020A21C | melee/gr/grshrineroute.c:1061 | fptoi | fctiwz | melee_world/src/tu/grshrineroute.rs:860 | fixed |
| 8020A764 | grShrineRoute_8020A21C | melee/gr/grshrineroute.c:1062 | fptoi | fctiwz | melee_world/src/tu/grshrineroute.rs:861 | fixed |
| 8020C8F8 | grBigBlueRoute_8020C85C | melee/gr/grbigblueroute.c | fptoi | fctiwz | melee_world/src/tu/grbigblueroute.rs:759 | ok |
| 8020C93C | grBigBlueRoute_8020C85C | melee/gr/grbigblueroute.c | fptoi | fctiwz | melee_world/src/tu/grbigblueroute.rs:759 | ok |
| 8020C940 | grBigBlueRoute_8020C85C | melee/gr/grbigblueroute.c | fptoi | fctiwz | melee_world/src/tu/grbigblueroute.rs:759 | ok |
| 8020D93C | grBigBlueRoute_8020CD20 | melee/gr/grbigblueroute.c:1011 | fptoi | fctiwz | melee_world/src/tu/grbigblueroute.rs:1062 | fixed |
| 8020F770 | stageGObj1_OnInit | melee/gr/groldkongo.c:206 | fptoi | fctiwz | melee_world/src/tu/grgreens.rs:429 | ok |
| 8020F774 | stageGObj1_OnInit | melee/gr/groldkongo.c:207 | fptoi | fctiwz | melee_world/src/tu/grgreens.rs:429 | ok |
| 8020FA90 | stageGObj1_GObjProc | melee/gr/groldkongo.c:295 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 8020FA94 | stageGObj1_GObjProc | melee/gr/groldkongo.c:296 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 8020FB74 | stageGObj1_GObjProc | melee/gr/groldkongo.c:306 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 8020FB78 | stageGObj1_GObjProc | melee/gr/groldkongo.c:307 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 80210514 | grOldKongo_80210454 | melee/gr/groldkongo.c:513 | fptoi | fctiwz | melee_world/src/tu/groldkongo.rs:624 | ok |
| 80211518 | grOldPupupu_802113E0 | melee/gr/groldpupupu.c:598 | fptoi | fctiwz | melee_world/src/tu/groldpupupu.rs:609 | ok |
| 8021151C | grOldPupupu_802113E0 | melee/gr/groldpupupu.c:599 | fptoi | fctiwz | melee_world/src/tu/groldpupupu.rs:609 | ok |
| 8021160C | grOldPupupu_802113E0 | melee/gr/groldpupupu.c:612 | fptoi | fctiwz | melee_world/src/tu/groldpupupu.rs:609 | ok |
| 80211610 | grOldPupupu_802113E0 | melee/gr/groldpupupu.c:613 | fptoi | fctiwz | melee_world/src/tu/groldpupupu.rs:609 | ok |
| 802118C8 | grOldPupupu_802113E0 | melee/gr/groldpupupu.c:662 | fptoi | fctiwz | melee_world/src/tu/groldpupupu.rs:609 | ok |
| 802121A0 | stageGObj1_GObjProc | melee/gr/grpura.c:471 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 802121E0 | stageGObj1_GObjProc | melee/gr/grpura.c:472 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 80212224 | stageGObj1_GObjProc | melee/gr/grpura.c:473 | fptoi | fctiwz | melee_world/src/tu/grheal.rs:340 | ok |
| 8021458C | grGreens_80213C10 | melee/gr/grgreens.c:635 | div | divw | melee_world/src/tu/grgreens.rs:573 | ok |
| 802177C0 | grFlatzone_802176BC | melee/gr/grflatzone.c:404 | fptoi | fctiwz | melee_world/src/tu/grflatzone.rs:572 | fixed |
| 802179B0 | grFlatzone_802176BC | melee/gr/grflatzone.c:444 | div | divw | melee_world/src/tu/grflatzone.rs:552 | ok |
| 80217F3C | grFlatzone_80217EF0 | melee/gr/grflatzone.c:629 | div | divw | melee_world/src/tu/grflatzone.rs:750 | ok |
| 80218D80 | grPushOn_80218888 | melee/gr/grpushon.c:450 | fptoi | fctiwz | melee_world/src/tu/grpushon.rs:452 | fixed |
| 80218DA8 | grPushOn_80218888 | melee/gr/grpushon.c:451 | fptoi | fctiwz | melee_world/src/tu/grpushon.rs:453 | fixed |
| 80218DD0 | grPushOn_80218888 | melee/gr/grpushon.c:452 | fptoi | fctiwz | melee_world/src/tu/grpushon.rs:454 | fixed |
| 80219960 | grFigureGet_80219898 | melee/gr/grfigureget.c:239 | fptoi | fctiwz | melee_world/src/tu/grfigureget.rs:290 | fixed |
| 8021C590 | grLast_8021C500 | melee/gr/grlast.c:988 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021C5A4 | grLast_8021C500 | melee/gr/grlast.c:989 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021C5B8 | grLast_8021C500 | melee/gr/grlast.c:990 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021C5D4 | grLast_8021C500 | melee/gr/grlast.c:992 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021C5E8 | grLast_8021C500 | melee/gr/grlast.c:993 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021C5FC | grLast_8021C500 | melee/gr/grlast.c:994 | fptoi | fctiwz | melee_world/src/tu/grlast.rs:1031 | ok |
| 8021D818 | grHomeRun_8021D680 | melee/gr/grhomerun.c:374 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:539 | fixed |
| 8021D86C | grHomeRun_8021D680 | melee/gr/grhomerun.c:380 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:544 | fixed |
| 8021E524 | grHomeRun_8021E500 | melee/gr/grhomerun.c:736 | div | divw | melee_world/src/tu/grhomerun.rs:756 | ok |
| 8021E95C | grHomeRun_8021E500 | melee/gr/grhomerun.c:740 | div | divw | melee_world/src/tu/grhomerun.rs:756 | ok |
| 8021E960 | grHomeRun_8021E500 | melee/gr/grhomerun.c:789 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:790 | fixed |
| 8021ED44 | grHomeRun_8021EC58 | melee/gr/grhomerun.c:913 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:907 | fixed |
| 8021EE38 | grHomeRun_8021EDD4 | melee/gr/grhomerun.c:948 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:941 | fixed |
| 8021EE6C | grHomeRun_8021EDD4 | melee/gr/grhomerun.c:957 | fptoi | fctiwz | melee_world/src/tu/grhomerun.rs:950 | fixed |
| 8021F1B8 | stageGObj1_OnInit | melee/gr/grheal.c:232 | div | divw | melee_world/src/tu/grgreens.rs:429 | ok |
| 80227274 | fn_80227188 | melee/db/dbcamera.c:231 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 802272B0 | fn_80227188 | melee/db/dbcamera.c:236 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 802272EC | fn_80227188 | melee/db/dbcamera.c:241 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 80227348 | fn_80227188 | melee/db/dbcamera.c:249 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 80227384 | fn_80227188 | melee/db/dbcamera.c:254 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 802273C0 | fn_80227188 | melee/db/dbcamera.c:259 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 802273EC | fn_80227188 | melee/db/dbcamera.c:265 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 802273F4 | fn_80227188 | melee/db/dbcamera.c:265 | fptoi | fctiwz | melee_runtime/src/tu/dbcamera.rs:314 | ok |
| 8022C0A4 | mn_8022C068 | melee/mn/mnmain.c:1814 | div | divw | melee_modes/src/tu/mnmain.rs:3430 | fixed |
| 8022C0C4 | mn_8022C068 | melee/mn/mnmain.c:1818 | div | divw | melee_modes/src/tu/mnmain.rs:3436 | fixed |
| 8022C0E4 | mn_8022C068 | melee/mn/mnmain.c:1822 | div | divw | melee_modes/src/tu/mnmain.rs:3442 | fixed |
| 8022C270 | fn_8022C128 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3451 | ok-const |
| 8022C290 | fn_8022C128 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3451 | ok-const |
| 8022C2B0 | fn_8022C128 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3451 | ok-const |
| 8022C46C | mn_8022C304 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3471 | ok-const |
| 8022C48C | mn_8022C304 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3471 | ok-const |
| 8022C4AC | mn_8022C304 | melee/mn/mnmain.c | div | divw | melee_modes/src/tu/mnmain.rs:3471 | ok-const |
| 8022E9A8 | mn_8022E978 | melee/mn/mnmain.c:3087 | shift | slw | melee_modes/src/tu/mnmain.rs:4912 | fixed |
| 8022E9D8 | mn_8022E978 | melee/mn/mnmain.c:3089 | shift | slw | melee_modes/src/tu/mnmain.rs:4915 | fixed |
| 8022EB40 | mn_GetDigitAt | melee/mn/mnmain.c:3124 | div | divw | melee_modes/src/tu/mnmain.rs:4960 | fixed |
| 8022EBAC | mn_GetDigitCount | melee/mn/mnmain.c:3135 | div | divw. | melee_modes/src/tu/mnmain.rs:4969 | fixed |
| 80233AF0 | mnItemSw_80233A98 | melee/mn/mnitemsw.c:169 | fptoi | fctiwz | melee_modes/src/tu/mnitemsw.rs:306 | fixed |
| 80233B20 | mnItemSw_80233A98 | melee/mn/mnitemsw.c:174 | fptoi | fctiwz | melee_modes/src/tu/mnitemsw.rs:312 | fixed |
| 80233B44 | mnItemSw_80233A98 | melee/mn/mnitemsw.c:179 | fptoi | fctiwz | melee_modes/src/tu/mnitemsw.rs:316 | fixed |
| 8023EC10 | mnDiagram_GetHitPercentage | melee/mn/mndiagram.c:210 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:361 | fixed |
| 8023EC94 | mnDiagram_GetHitPercentage | melee/mn/mndiagram.c:220 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:377 | fixed |
| 8023ED6C | mnDiagram_GetPlayPercentage | melee/mn/mndiagram.c:240 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:402 | fixed |
| 8023EE00 | mnDiagram_GetPlayPercentage | melee/mn/mndiagram.c:253 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:418 | fixed |
| 8023EEC0 | mnDiagram_GetAveragePlayerCount | melee/mn/mndiagram.c:270 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:440 | fixed |
| 8023EF40 | mnDiagram_GetAveragePlayerCount | melee/mn/mndiagram.c:279 | fptoi | fctiwz | melee_modes/src/tu/mndiagram.rs:455 | fixed |
| 8023F174 | mnDiagram_FormatDecimalNumber | melee/mn/mndiagram.c:387 | div | divwu | melee_modes/src/tu/mndiagram.rs:593 | ok |
| 8023F1DC | mnDiagram_FormatDecimalNumber | melee/mn/mndiagram.c:395 | div | divwu | melee_modes/src/tu/mndiagram.rs:593 | ok |
| 8024A86C | mnSoundTest_8024A790 | melee/mn/mnsoundtest.c:142 | fptoi | fctiwz | melee_modes/src/tu/mnsoundtest.rs:724 | fixed |
| 8024A8F8 | mnSoundTest_8024A790 | melee/mn/mnsoundtest.c:154 | fptoi | fctiwz | melee_modes/src/tu/mnsoundtest.rs:736 | fixed |
| 8024D6EC | mnEvent_8024D5B0 | melee/mn/mnevent.c:311 | fptoi | fctiwz | melee_modes/src/tu/mnevent.rs:417 | fixed |
| 8025A00C | mnStageSel_80259ED8 | melee/mn/mnstagesel.c:728 | fptoi | fctiwz | melee_modes/src/tu/mnstagesel.rs:3837 | fixed |
| 8025C8C0 | mnCharSel_8025C020 | melee/mn/mncharsel.c:701 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:925 | ok |
| 8025C930 | mnCharSel_8025C020 | melee/mn/mncharsel.c:705 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:925 | ok |
| 8025C9A4 | mnCharSel_8025C020 | melee/mn/mncharsel.c | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:925 | ok |
| 8025CA10 | mnCharSel_8025C020 | melee/mn/mncharsel.c | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:925 | ok |
| 80260104 | mnCharSel_CostumeChange | melee/mn/mncharsel.c:2364 | div | divw | melee_modes/src/tu/mncharsel.rs:2262 | ok |
| 80260F2C | mnCharSel_CursorThink | melee/mn/mncharsel.c | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:2453 | ok |
| 80261138 | mnCharSel_CursorThink | melee/mn/mncharsel.c | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:2453 | ok |
| 8026367C | fn_802633B0 | melee/mn/mncharsel.c:4169 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:3602 | ok |
| 80263694 | fn_802633B0 | melee/mn/mncharsel.c:4171 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:3602 | ok |
| 802638F4 | fn_802633B0 | melee/mn/mncharsel.c:4232 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:3602 | ok |
| 802639D0 | fn_802633B0 | melee/mn/mncharsel.c:4259 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:3602 | ok |
| 80263AD4 | fn_802633B0 | melee/mn/mncharsel.c:4275 | fptoi | fctiwz | melee_modes/src/tu/mncharsel.rs:3602 | ok |
| 8026B454 | it_8026B424 | melee/it/it_26B1.c:291 | fptoi | fctiwz | melee_items/src/tu/it_26B1.rs:463 | fixed |
| 8026B784 | it_8026B774 | melee/it/it_26B1.c:485 | shift | slw | melee_fighter/src/tu/plbonuslib.rs:106 | ok |
| 8026C4E4 | it_8026C47C | melee/it/itspawn.c:61 | shift | slw | melee_items/src/tu/itspawn.rs:241 | ok |
| 8026C9E8 | fn_8026C88C | melee/it/itspawn.c | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:410 | ok |
| 8026CA20 | fn_8026C88C | melee/it/itspawn.c | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:410 | ok |
| 8026CAE4 | it_8026CA4C | melee/it/itspawn.c:237 | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:429 | fixed |
| 8026CCFC | it_8026CB9C | melee/it/itspawn.c:301 | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:495 | fixed |
| 8026CEB0 | it_8026CD50 | melee/it/itspawn.c:363 | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:548 | fixed |
| 8026D1F8 | it_8026D018 | melee/it/itspawn.c | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:631 | ok |
| 8026D230 | it_8026D018 | melee/it/itspawn.c | fptoi | fctiwz | melee_items/src/tu/itspawn.rs:631 | ok |
| 8026FED4 | it_8026FE68 | melee/it/itcoll.c:254 | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:486 | fixed |
| 8026FEDC | it_8026FE68 | melee/it/itcoll.c:254 | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:487 | fixed |
| 80270040 | it_8026FE68 | melee/it/itcoll.c:255 | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:511 | fixed |
| 80270048 | it_8026FE68 | melee/it/itcoll.c:291 | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:512 | fixed |
| 8027059C | it_802703E8 | melee/it/itcoll.c:461 | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:652 | fixed |
| 80270B84 | it_802706D0 | melee/it/itcoll.c | fptoi | fctiwz | melee_items/src/tu/itcoll.rs:718 | ok |
| 80272900 | it_802728C8 | melee/it/it_2725.c:173 | fptoi | fctiwz | melee_items/src/tu/it_2725.rs:1455 | fixed |
| 8027290C | it_802728C8 | melee/it/it_2725.c:173 | div | divwu | melee_items/src/tu/it_2725.rs:1456 | fixed |
| 8027435C | it_802742F4 | melee/it/it_2725.c:926 | shift | slw | melee_items/src/tu/it_26B1.rs:253 | ok |
| 8027821C | it_80278108 | melee/it/itmaterial.c:215 | div | divw | melee_items/src/tu/itmaterial.rs:317 | fixed |
| 80278230 | it_80278108 | melee/it/itmaterial.c:217 | div | divw | melee_items/src/tu/itmaterial.rs:319 | fixed |
| 80278294 | it_80278108 | melee/it/itmaterial.c:226 | div | divw | melee_items/src/tu/itmaterial.rs:325 | fixed |
| 802782CC | it_80278108 | melee/it/itmaterial.c:234 | div | divw | melee_items/src/tu/itmaterial.rs:329 | fixed |
| 80278FA0 | it_80278F2C | melee/it/itanimlist.c:57 | fptoi | fctiwz | melee_items/src/tu/itanimlist.rs:296 | fixed |
| 8027BD5C | it_8027BBF4 | melee/it/itzako.c:309 | fptoi | fctiwz | melee_items/src/tu/itzako.rs:575 | fixed |
| 8027C220 | it_8027C0F0 | melee/it/itzako.c:389 | fptoi | fctiwz | melee_items/src/tu/itzako.rs:653 | fixed |
| 8027CC40 | it_8027CBFC | melee/it/itzako.c:624 | fptoi | fctiwz | melee_items/src/tu/itzako.rs:871 | fixed |
| 8027CC58 | it_8027CBFC | melee/it/itzako.c:627 | fptoi | fctiwz | melee_items/src/tu/itzako.rs:872 | fixed |
| 8027CCBC | it_8027CC88 | melee/it/itzako.c:656 | fptoi | fctiwz | melee_items/src/tu/itzako.rs:890 | fixed |
| 8027D790 | itBombhei_Logic6_Spawned | melee/it/kinds/itbombhei.c:194 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:340 | fixed |
| 8027DA4C | it_8027D820 | melee/it/kinds/itbombhei.c:227 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:371 | fixed |
| 8027DE90 | it_8027DE18 | melee/it/kinds/itbombhei.c:281 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:422 | fixed |
| 8027E9BC | it_8027E978 | melee/it/kinds/itbombhei.c:372 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:507 | fixed |
| 8027EC70 | itBombhei_UnkMotion3_Anim | melee/it/kinds/itbombhei.c:391 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:526 | fixed |
| 8027F320 | itBombhei_UnkMotion2_Coll | melee/it/kinds/itbombhei.c | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:574 | ok |
| 8027F934 | it_8027F8E0 | melee/it/kinds/itbombhei.c:510 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:633 | fixed |
| 8027FCFC | fn_8027FCA8 | melee/it/kinds/itbombhei.c:548 | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:669 | fixed |
| 80280128 | fn_8028007C | melee/it/kinds/itbombhei.c | fptoi | fctiwz | melee_items/src/tu/itbombhei.rs:711 | ok |
| 80288B68 | it_3F14_Logic2_DmgReceived | melee/it/kinds/ittaru.c:681 | fptoi | fctiwz | melee_items/src/tu/ittaru.rs:787 | fixed |
| 8028A584 | it_8028A544 | melee/it/kinds/itkusudama.c:431 | fptoi | fctiwz | melee_items/src/tu/itkusudama.rs:622 | fixed |
| 80291D68 | it_80291D38 | melee/it/kinds/itsscope.c:81 | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:316 | fixed |
| 80291D84 | it_80291D38 | melee/it/kinds/itsscope.c:90 | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:316 | fixed |
| 80291D98 | it_80291D38 | melee/it/kinds/itsscope.c:92 | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:316 | fixed |
| 80291E28 | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291E44 | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291E58 | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291EA0 | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291EB8 | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291ECC | it_80291DAC | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:335 | ok |
| 80291F44 | it_80291F14 | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:362 | ok |
| 80291F60 | it_80291F14 | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:362 | ok |
| 80291F74 | it_80291F14 | melee/it/kinds/itsscope.c | fptoi | fctiwz | melee_items/src/tu/itsscope.rs:362 | ok |
| 802A26D4 | it_802A2568 | melee/it/kinds/itlinkhookshot.c | fptoi | fctiwz | melee_items/src/tu/itlinkhookshot.rs:388 | ok |
| 802AADA8 | itNesspkflash_UnkMotion1_Anim | melee/it/kinds/itnesspkflash.c:239 | fptoi | fctiwz | melee_items/src/tu/itnesspkflash.rs:399 | ok |
| 802B23D4 | itPikachuthunder_UnkMotion2_Anim | melee/it/kinds/itpikachuthunder.c | fptoi | fctiwz | melee_items/src/tu/itpikachuthunder.rs:431 | ok |
| 802B2440 | itPikachuthunder_UnkMotion2_Anim | melee/it/kinds/itpikachuthunder.c | fptoi | fctiwz | melee_items/src/tu/itpikachuthunder.rs:431 | ok |
| 802B5740 | it_802B56E4 | melee/it/kinds/itsamuschargeshot.c:105 | fptoi | fctiwz | melee_items/src/tu/itsamuschargeshot.rs:194 | fixed |
| 802B5744 | it_802B56E4 | melee/it/kinds/itsamuschargeshot.c:106 | fptoi | fctiwz | melee_items/src/tu/itsamuschargeshot.rs:195 | fixed |
| 802B7768 | it_802B75FC | melee/it/kinds/itsamusgrapple.c:370 | fptoi | fctiwz | melee_items/src/tu/itsamusgrapple.rs:521 | fixed |
| 802BC0E4 | it_802BC080 | melee/it/kinds/itseakchain.c:633 | fptoi | fctiwz | melee_items/src/tu/itseakchain.rs:729 | fixed |
| 802BC684 | it_802BC080 | melee/it/kinds/itseakchain.c:738 | fptoi | fctiwz | melee_items/src/tu/itseakchain.rs:820 | fixed |
| 802BCD30 | it_802BCB88 | melee/it/kinds/itseakchain.c:899 | div | divw | melee_items/src/tu/itseakchain.rs:940 | ok |
| 802C544C | it_802C53F0 | melee/it/kinds/itmewtwoshadowball.c:274 | fptoi | fctiwz | melee_items/src/tu/itmewtwoshadowball.rs:443 | fixed |
| 802C5450 | it_802C53F0 | melee/it/kinds/itmewtwoshadowball.c:275 | fptoi | fctiwz | melee_items/src/tu/itmewtwoshadowball.rs:444 | fixed |
| 802C87F4 | itGamewatchchef_UnkMotion1_Anim | melee/it/kinds/itgamewatchchef.c:139 | fptoi | fctiwz | melee_items/src/tu/itgamewatchchef.rs:239 | fixed |
| 802C9D98 | it_802C9D40 | melee/it/kinds/itkabigon.c:35 | fptoi | fctiwz | melee_items/src/tu/itkabigon.rs:157 | fixed |
| 802CB00C | it_802CAFD4 | melee/it/kinds/itmatadogas.c:32 | fptoi | fctiwz | melee_items/src/tu/itmatadogas.rs:249 | fixed |
| 802CB2F8 | it_802CB2B0 | melee/it/kinds/itmatadogas.c:95 | fptoi | fctiwz | melee_items/src/tu/itmatadogas.rs:310 | fixed |
| 802CC24C | it_802CC1CC | melee/it/kinds/itlizardon.c:333 | fptoi | fctiwz | melee_items/src/tu/itlizardon.rs:504 | fixed |
| 802CE180 | itHassam_UnkMotion1_Anim | melee/it/kinds/ithassam.c:239 | div | divw | melee_items/src/tu/ithassam.rs:241 | ok |
| 802CE964 | it_802CE8D0 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:330 | ok |
| 802CE9C4 | it_802CE8D0 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:330 | ok |
| 802CEA1C | it_802CE8D0 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:330 | ok |
| 802CEA7C | it_802CE8D0 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:330 | ok |
| 802CEDB0 | it_802CED54 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:450 | ok |
| 802CEDFC | it_802CED54 | melee/it/kinds/itunknown.c | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:450 | ok |
| 802CEFCC | it_2725_Logic38_Spawned | melee/it/kinds/itunknown.c:274 | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:504 | fixed |
| 802CF010 | it_2725_Logic38_Spawned | melee/it/kinds/itunknown.c:276 | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:506 | fixed |
| 802CF050 | it_2725_Logic38_Spawned | melee/it/kinds/itunknown.c:278 | fptoi | fctiwz | melee_items/src/tu/itunknown.rs:508 | fixed |
| 802D20FC | it_802D208C | melee/it/kinds/itlugia.c:460 | fptoi | fctiwz | melee_items/src/tu/itlugia.rs:666 | fixed |
| 802D2160 | it_802D208C | melee/it/kinds/itlugia.c:463 | fptoi | fctiwz | melee_items/src/tu/itlugia.rs:670 | fixed |
| 802D21A8 | it_802D208C | melee/it/kinds/itlugia.c:466 | fptoi | fctiwz | melee_items/src/tu/itlugia.rs:672 | fixed |
| 802D44C0 | it_802D4494 | melee/it/kinds/ithitodeman.c:130 | fptoi | fctiwz | melee_items/src/tu/ithitodeman.rs:484 | fixed |
| 802D7AC0 | itOldkuri_UnkMotion3_Coll | melee/it/kinds/itoldkuri.c:251 | fptoi | fctiwz | melee_items/src/tu/itoldkuri.rs:339 | fixed |
| 802D7BE4 | itOldkuri_UnkMotion4_Coll | melee/it/kinds/itoldkuri.c:286 | fptoi | fctiwz | melee_items/src/tu/itoldkuri.rs:373 | fixed |
| 802D7F88 | it_2725_Logic0_DmgReceived | melee/it/kinds/itoldkuri.c:410 | fptoi | fctiwz | melee_items/src/tu/itoldkuri.rs:496 | fixed |
| 802D8F50 | it_802D8EC8 | melee/it/kinds/itheiho.c:268 | fptoi | fctiwz | melee_items/src/tu/itheiho.rs:437 | fixed |
| 802D9A6C | it_802D9A2C | melee/it/kinds/itlikelike.c:105 | fptoi | fctiwz | melee_items/src/tu/itlikelike.rs:298 | fixed |
| 802D9A80 | it_802D9A2C | melee/it/kinds/itlikelike.c:107 | fptoi | fctiwz | melee_items/src/tu/itlikelike.rs:298 | fixed |
| 802D9D2C | it_2725_Logic5_DmgReceived | melee/it/kinds/itlikelike.c:175 | fptoi | fctiwz | melee_items/src/tu/itlikelike.rs:366 | fixed |
| 802DC7CC | itNokonoko_Logic3_DmgReceived | melee/it/kinds/itnokonoko.c:78 | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:207 | fixed |
| 802DCA20 | it_802DC990 | melee/it/kinds/itnokonoko.c:141 | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:269 | fixed |
| 802DCF80 | it_802DCEC4 | melee/it/kinds/itnokonoko.c | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:398 | ok |
| 802DD148 | itNokonoko_UnkMotion5_Anim | melee/it/kinds/itnokonoko.c:334 | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:453 | fixed |
| 802DD160 | itNokonoko_UnkMotion5_Anim | melee/it/kinds/itnokonoko.c:336 | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:455 | fixed |
| 802DD428 | itNokonoko_UnkMotion3_Anim | melee/it/kinds/itnokonoko.c | fptoi | fctiwz | melee_items/src/tu/itnokonoko.rs:521 | ok |
| 802DF498 | it_802DF230 | melee/it/kinds/itzgshell.c:640 | fptoi | fctiwz | melee_items/src/tu/itzgshell.rs:902 | ok |
| 802DF8E4 | itZrshell_UnkMotion11_Anim | melee/it/kinds/itzgshell.c:708 | fptoi | fctiwz | melee_items/src/tu/itzgshell.rs:965 | ok |
| 802E080C | it_802E0734 | melee/it/kinds/itpatapata.c:122 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:223 | fixed |
| 802E0A68 | itPatapata_UnkMotion3_Phys | melee/it/kinds/itpatapata.c:174 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:270 | fixed |
| 802E0C20 | itPatapata_UnkMotion3_Coll | melee/it/kinds/itpatapata.c:224 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:317 | fixed |
| 802E12A0 | it_802E11E0 | melee/it/kinds/itpatapata.c:436 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:503 | fixed |
| 802E12E8 | it_802E11E0 | melee/it/kinds/itpatapata.c:440 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:508 | fixed |
| 802E15F4 | it_802E15B0 | melee/it/kinds/itpatapata.c:520 | fptoi | fctiwz | melee_items/src/tu/itpatapata.rs:577 | fixed |
| 802E2380 | it_802E2330 | melee/it/kinds/itklap.c:328 | fptoi | fctiwz | melee_items/src/tu/itklap.rs:489 | fixed |
| 802E23B0 | it_802E2330 | melee/it/kinds/itklap.c:333 | fptoi | fctiwz | melee_items/src/tu/itklap.rs:494 | fixed |
| 802E25D4 | it_2725_Logic8_DmgReceived | melee/it/kinds/itoldottosea.c:90 | fptoi | fctiwz | melee_items/src/tu/itoldottosea.rs:172 | fixed |
| 802E38DC | it_802E3884 | melee/it/kinds/itwhitebea.c:325 | fptoi | fctiwz | melee_items/src/tu/itwhitebea.rs:443 | fixed |
| 802E4B58 | it_802E4B00 | melee/it/kinds/itoctarock.c:101 | fptoi | fctiwz | melee_items/src/tu/itoctarock.rs:301 | fixed |
| 802E511C | itOctarock_UnkMotion4_Anim | melee/it/kinds/itoctarock.c:259 | fptoi | fctiwz | melee_items/src/tu/itoctarock.rs:449 | fixed |
| 802E6214 | it_802E61C4 | melee/it/kinds/it_2E5A.c:265 | fptoi | fctiwz | melee_items/src/tu/it_2E5A.rs:519 | ok |
| 802E6254 | it_802E61C4 | melee/it/kinds/it_2E5A.c:268 | fptoi | fctiwz | melee_items/src/tu/it_2E5A.rs:519 | ok |
| 802E8D84 | it_802E8CD8 | melee/it/kinds/itleadead.c:183 | fptoi | fctiwz | melee_items/src/tu/itleadead.rs:299 | fixed |
| 802EB724 | it_802EB6DC | melee/it/kinds/ittincle.c:151 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:293 | fixed |
| 802EB8C4 | it_802EB870 | melee/it/kinds/ittincle.c:203 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:342 | fixed |
| 802EB908 | it_802EB870 | melee/it/kinds/ittincle.c:205 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:344 | fixed |
| 802EBFE0 | it_802EBFAC | melee/it/kinds/ittincle.c:450 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:578 | fixed |
| 802EC08C | it_802EBFAC | melee/it/kinds/ittincle.c:465 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:592 | fixed |
| 802EC0D8 | it_802EBFAC | melee/it/kinds/ittincle.c:475 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:602 | fixed |
| 802EC704 | it_802EC69C | melee/it/kinds/ittincle.c:699 | fptoi | fctiwz | melee_items/src/tu/ittincle.rs:814 | fixed |
| 802ED370 | itKyasarin_UnkMotion4_Coll | melee/it/kinds/itkyasarin.c:286 | fptoi | fctiwz | melee_items/src/tu/itkyasarin.rs:430 | ok |
| 802ED3AC | itKyasarin_UnkMotion4_Coll | melee/it/kinds/itkyasarin.c:288 | fptoi | fctiwz | melee_items/src/tu/itkyasarin.rs:430 | ok |
| 802ED430 | itKyasarin_UnkMotion4_Coll | melee/it/kinds/itkyasarin.c:300 | fptoi | fctiwz | melee_items/src/tu/itkyasarin.rs:430 | ok |
| 802ED46C | itKyasarin_UnkMotion4_Coll | melee/it/kinds/itkyasarin.c:302 | fptoi | fctiwz | melee_items/src/tu/itkyasarin.rs:430 | ok |
| 802ED840 | itKyasarin_UnkMotion7_Anim | melee/it/kinds/itkyasarin.c:379 | fptoi | fctiwz | melee_items/src/tu/itkyasarin.rs:501 | fixed |
| 802EF374 | it_802EF320 | melee/it/kinds/ittools.c:173 | fptoi | fctiwz | melee_items/src/tu/ittools.rs:235 | fixed |
| 802EF410 | itTools_UnkMotion9_Anim | melee/it/kinds/ittools.c:185 | fptoi | fctiwz | melee_items/src/tu/ittools.rs:247 | fixed |
| 802EF4F4 | itTools_UnkMotion9_Coll | melee/it/kinds/ittools.c | fptoi | fctiwz | melee_items/src/tu/ittools.rs:270 | ok |
| 802EF778 | it_2725_Logic22_Clanked | melee/it/kinds/ittools.c:284 | fptoi | fctiwz | melee_items/src/tu/ittools.rs:339 | fixed |
| 802EF820 | it_2725_Logic22_HitShield | melee/it/kinds/ittools.c:299 | fptoi | fctiwz | melee_items/src/tu/ittools.rs:352 | fixed |
| 802EF8C8 | it_2725_Logic22_Absorbed | melee/it/kinds/ittools.c:314 | fptoi | fctiwz | melee_items/src/tu/ittools.rs:365 | fixed |
| 802F04C8 | it_802F0484 | melee/it/kinds/itmasterhandlaser.c:97 | fptoi | fctiwz | melee_items/src/tu/itmasterhandlaser.rs:213 | fixed |
| 802F0550 | itMasterhandlaser_UnkMotion0_Anim | melee/it/kinds/itmasterhandlaser.c | fptoi | fctiwz | melee_items/src/tu/itmasterhandlaser.rs:216 | ok |
| 802F0A18 | it_802F063C | melee/it/kinds/itmasterhandlaser.c:195 | fptoi | fctiwz | melee_items/src/tu/itmasterhandlaser.rs:300 | fixed |
| 802F1974 | itCoin_Logic116_DmgReceived | melee/it/kinds/itcoin.c:214 | fptoi | fctiwz | melee_items/src/tu/itcoin.rs:402 | fixed |
| 802F55B0 | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F55F4 | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F5638 | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F56E4 | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F5728 | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F576C | ifStatus_802F4EDC | melee/if/ifstatus.c | fptoi | fctiwz | melee_world/src/tu/ifstatus.rs:392 | ok |
| 802F8CCC | ifStock_802F89F8 | melee/if/ifstock.c | div | divw | melee_world/src/tu/ifstock.rs:771 | ok-const |
| 802FBE64 | ifMagnify_802FBBDC | melee/if/ifmagnify.c:500 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:462 | fixed |
| 802FBF80 | ifMagnify_802FBBDC | melee/if/ifmagnify.c:538 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC0C8 | ifMagnify_802FBBDC | melee/if/ifmagnify.c:564 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC0CC | ifMagnify_802FBBDC | melee/if/ifmagnify.c:566 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC25C | ifMagnify_802FBBDC | melee/if/ifmagnify.c:568 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC260 | ifMagnify_802FBBDC | melee/if/ifmagnify.c:570 | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC264 | ifMagnify_802FBBDC | melee/if/ifmagnify.c | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FC2E0 | ifMagnify_802FBBDC | melee/if/ifmagnify.c | fptoi | fctiwz | melee_world/src/tu/ifmagnify.rs:498 | fixed |
| 802FD188 | NameTag_Create | melee/if/ifnametag.c | fptoi | fctiwz | melee_world/src/tu/ifnametag.rs:297 | ok |
| 802FD328 | un_802FD28C | melee/if/ifnametag.c | fptoi | fctiwz | melee_modes/src/tu_ext.rs:1146 | ok |
| 803024C8 | DevText_Draw | melee/if/textdraw.c:278 | fptoi | fctiwz | melee_world/src/tu/textdraw.rs:316 | fixed |
| 80302500 | DevText_Draw | melee/if/textdraw.c:280 | fptoi | fctiwz | melee_world/src/tu/textdraw.rs:319 | fixed |
| 80302FAC | un_80302EA4 | melee/if/textlib_1.c:72 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:170 | fixed |
| 80303554 | un_80303444 | melee/if/textlib_1.c:194 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:285 | fixed |
| 80303584 | un_80303444 | melee/if/textlib_1.c:202 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:292 | fixed |
| 803035F4 | un_80303444 | melee/if/textlib_1.c:215 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:304 | fixed |
| 80303820 | un_80303720 | melee/if/textlib_1.c:272 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:356 | fixed |
| 80303850 | un_80303720 | melee/if/textlib_1.c:280 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:363 | fixed |
| 803038B4 | un_80303720 | melee/if/textlib_1.c:293 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:375 | fixed |
| 803042D8 | un_80304210 | melee/if/textlib_1.c:562 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:655 | fixed |
| 803042DC | un_80304210 | melee/if/textlib_1.c:563 | fptoi | fctiwz | melee_world/src/tu/textlib_1.rs:656 | fixed |
| 80304B68 | Toy_80304B0C | melee/ty/toy.c:694 | shift | slw | melee_items/src/tu/toy.rs:694 | fixed |
| 80304DFC | _Toy_80304D30 | melee/ty/toy.c:807 | fptoi | fctiwz | melee_items/src/tu/toy.rs:776 | fixed |
| 80304EC4 | _Toy_80304D30 | melee/ty/toy.c | shift | slw | melee_items/src/tu/toy.rs:764 | ok-const |
| 80304F70 | _Toy_80304D30 | melee/ty/toy.c | shift | slw | melee_items/src/tu/toy.rs:764 | ok-const |
| 8030500C | _Toy_80304D30 | melee/ty/toy.c | shift | slw | melee_items/src/tu/toy.rs:764 | ok-const |
| 803058E8 | Toy_SetUnlockState | melee/ty/toy.c:1279 | fptoi | fctiwz | melee_items/src/tu/toy.rs:1067 | fixed |
| 80305AA8 | Toy_80305918 | melee/ty/toy.c:1376 | shift | slw | melee_items/src/tu/toy.rs:1101 | fixed |
| 80305AF4 | Toy_80305918 | melee/ty/toy.c:1400 | shift | slw | melee_items/src/tu/toy.rs:1108 | fixed |
| 80305B20 | Toy_80305918 | melee/ty/toy.c:1412 | shift | slw | melee_items/src/tu/toy.rs:1115 | fixed |
| 80305B64 | Toy_80305918 | melee/ty/toy.c:1425 | shift | slw | melee_items/src/tu/toy.rs:1118 | fixed |
| 8030842C | Toy_803083D8 | melee/ty/toy.c:2874 | fptoi | fctiwz | melee_items/src/tu/toy.rs:2196 | fixed |
| 80311414 | _Toy_80310B48 | melee/ty/toy.c:6177 | fptoi | fctiwz | melee_items/src/tu/toy.rs:4393 | fixed |
| 80311718 | Toy_80311680 | melee/ty/toy.c:6287 | fptoi | fctiwz | melee_items/src/tu/toy.rs:4470 | fixed |
| 80312404 | Toy_8031234C | melee/ty/toy.c:6655 | shift | slw | melee_items/src/tu/toy.rs:4719 | fixed |
| 803125CC | Toy_803124BC | melee/ty/toy.c:6729 | fptoi | fctiwz | melee_items/src/tu/toy.rs:4765 | fixed |
| 8031272C | Toy_8031263C | melee/ty/toy.c:6751 | fptoi | fctiwz | melee_items/src/tu/toy.rs:4781 | fixed |
| 80313EF4 | _tyList_80313BD8 | melee/ty/tylist.c:738 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:893 | fixed |
| 80313F80 | _tyList_80313BD8 | melee/ty/tylist.c:749 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:904 | fixed |
| 8031412C | _tyList_80313BD8 | melee/ty/tylist.c:798 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:948 | fixed |
| 8031418C | _tyList_80313BD8 | melee/ty/tylist.c:808 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:958 | fixed |
| 803142C0 | _tyList_80313BD8 | melee/ty/tylist.c:823 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:973 | fixed |
| 80314B9C | _tyFigupon_80314B54 | melee/ty/tyfigupon.c:183 | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:254 | fixed |
| 803150D4 | _tyFigupon_80314C5C | melee/ty/tyfigupon.c:256 | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:329 | fixed |
| 80315108 | _tyFigupon_80314C5C | melee/ty/tyfigupon.c:256 | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:329 | fixed |
| 803157AC | _tyFigupon_803155C8 | melee/ty/tyfigupon.c:509 | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:533 | fixed |
| 80315BF4 | _tyFigupon_803155C8 | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:533 | fixed |
| 803163B0 | _tyFigupon_8031638C | melee/ty/tyfigupon.c:795 | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:776 | fixed |
| 80316E7C | _tyFigupon_80316C24 | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:906 | ok |
| 8031708C | _tyFigupon_80316C24 | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:906 | ok |
| 803172B8 | _tyFigupon_80316C24 | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:906 | ok |
| 8031747C | _tyFigupon_80316C24 | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:906 | ok |
| 80318184 | tyFigupon_Scene_OnEnter | melee/ty/tyfigupon.c | fptoi | fctiwz | melee_items/src/tu/tyfigupon.rs:1248 | ok |
| 80318BB8 | _tyDisplay_80318B1C | melee/ty/tydisplay.c:412 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:517 | fixed |
| 80318C0C | _tyDisplay_80318B1C | melee/ty/tydisplay.c:422 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:527 | fixed |
| 80318C74 | _tyDisplay_80318B1C | melee/ty/tydisplay.c:437 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:542 | fixed |
| 80318D68 | _tyDisplay_80318CB4 | melee/ty/tydisplay.c:850 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:628 | fixed |
| 80318EE0 | _tyDisplay_80318CB4 | melee/ty/tydisplay.c:853 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:631 | fixed |
| 80318F04 | _tyDisplay_80318CB4 | melee/ty/tydisplay.c:854 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:632 | fixed |
| 80318FD0 | _tyDisplay_80318CB4 | melee/ty/tydisplay.c:867 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:643 | fixed |
| 803195A8 | _tyDisplay_80319540 | melee/ty/tydisplay.c:955 | div | divw | melee_items/src/tu/tydisplay.rs:707 | fixed |
| 8031A3C0 | _tyDisplay_80319EF0 | melee/ty/tydisplay.c:1256 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:961 | fixed |
| 8031C3A4 | tyDisplay_8031C354 | melee/ty/tydisplay.c:2363 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:1636 | fixed |
| 8031C3F8 | tyDisplay_8031C354 | melee/ty/tydisplay.c:2378 | fptoi | fctiwz | melee_items/src/tu/tydisplay.rs:1650 | fixed |
| 80322900 | __cvt_fp2unsigned | Runtime/runtime.c:36 | fptoi | fctiwz |  | no-port |
| 80322970 | __div2u | Runtime/runtime.c | shift | srw |  | no-port |
| 8032297C | __div2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322984 | __div2u | Runtime/runtime.c | shift | slw |  | no-port |
| 8032298C | __div2u | Runtime/runtime.c | shift | srw |  | no-port |
| 8032299C | __div2u | Runtime/runtime.c | shift | slw |  | no-port |
| 803229A8 | __div2u | Runtime/runtime.c | shift | slw |  | no-port |
| 803229B0 | __div2u | Runtime/runtime.c | shift | srw |  | no-port |
| 803229B8 | __div2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322A88 | __div2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322A94 | __div2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322A9C | __div2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322AA4 | __div2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322AB4 | __div2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322AC0 | __div2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322AC8 | __div2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322AD0 | __div2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322B94 | __mod2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322BA0 | __mod2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322BA8 | __mod2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322BB0 | __mod2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322BC0 | __mod2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322BCC | __mod2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322BD4 | __mod2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322BDC | __mod2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322C98 | __mod2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322CA4 | __mod2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322CAC | __mod2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322CB4 | __mod2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322CC4 | __mod2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322CD0 | __mod2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322CD8 | __mod2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322CE0 | __mod2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D38 | __shl2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D3C | __shl2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322D44 | __shl2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D4C | __shl2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D5C | __shr2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322D60 | __shr2u | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D68 | __shr2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322D70 | __shr2u | Runtime/runtime.c | shift | srw |  | no-port |
| 80322D80 | __shr2i | Runtime/runtime.c | shift | srw |  | no-port |
| 80322D84 | __shr2i | Runtime/runtime.c | shift | slw |  | no-port |
| 80322D8C | __shr2i | Runtime/runtime.c | shift | sraw |  | no-port |
| 80322D98 | __shr2i | Runtime/runtime.c | shift | sraw |  | no-port |
| 80322DE0 | __cvt_sll_flt | Runtime/runtime.c | shift | slw |  | no-port |
| 80322DE4 | __cvt_sll_flt | Runtime/runtime.c | shift | srw |  | no-port |
| 80322DEC | __cvt_sll_flt | Runtime/runtime.c | shift | slw |  | no-port |
| 80322DF4 | __cvt_sll_flt | Runtime/runtime.c | shift | slw |  | no-port |
| 80322EA0 | __cvt_dbl_usll | Runtime/runtime.c | shift | srw |  | no-port |
| 80322EA4 | __cvt_dbl_usll | Runtime/runtime.c | shift | slw |  | no-port |
| 80322EAC | __cvt_dbl_usll | Runtime/runtime.c | shift | srw |  | no-port |
| 80322EB4 | __cvt_dbl_usll | Runtime/runtime.c | shift | srw |  | no-port |
| 80322EF0 | __cvt_dbl_usll | Runtime/runtime.c | shift | slw |  | no-port |
| 80322EF4 | __cvt_dbl_usll | Runtime/runtime.c | shift | srw |  | no-port |
| 80322EFC | __cvt_dbl_usll | Runtime/runtime.c | shift | slw |  | no-port |
| 80322F04 | __cvt_dbl_usll | Runtime/runtime.c | shift | slw |  | no-port |
| 80323378 | __num2dec | MSL/ansi_fp.c:136 | fptoi | fctiwz |  | no-port |
| 80323874 | fwrite | MSL/direct_io.c:115 | div | divwu |  | no-port |
| 80323A5C | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80323A64 | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323A70 | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80323A78 | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323A94 | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80323A98 | __copy_longs_rev_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323B14 | __copy_longs_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323B1C | __copy_longs_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80323B28 | __copy_longs_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323B30 | __copy_longs_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80323B4C | __copy_longs_unaligned | MSL/mem_funcs.c | shift | slw |  | no-port |
| 80323B50 | __copy_longs_unaligned | MSL/mem_funcs.c | shift | srw |  | no-port |
| 80325174 | long2str | MSL/printf.c:323 | div | divwu |  | no-port |
| 80325180 | long2str | MSL/printf.c:324 | div | divwu |  | no-port |
| 80325C68 | __strtoul | MSL/strtoul.c:45 | div | divwu |  | no-port |
| 80325DE4 | __strtoul | MSL/strtoul.c:100 | div | divwu |  | no-port |
| 80326280 | cosf | MSL/trigf.c:76 | fptoi | fctiwz | melee_modes/src/tu/geno_game.rs:59 | ok |
| 80326298 | cosf | MSL/trigf.c:76 | fptoi | fctiwz | melee_modes/src/tu/geno_game.rs:59 | ok |
| 80326414 | sinf | MSL/trigf.c:32 | fptoi | fctiwz | melee_modes/src/tu/geno_game.rs:60 | ok |
| 8032642C | sinf | MSL/trigf.c:32 | fptoi | fctiwz | melee_modes/src/tu/geno_game.rs:60 | ok |
| 8032A7D0 | __TRK_copy_vectors | MetroTRK/dolphin_trk.c:149 | shift | slw |  | no-port |
| 8032B050 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0A8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0B0 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0B8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0C0 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0C8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0D0 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B0D8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B10C | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | slw |  | no-port |
| 8032B19C | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B1B0 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B1C0 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B1D4 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B1E8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B1F8 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B210 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B220 | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032B24C | DBGEXIImm | dolphin/odenotstub/odenotstub.c | shift | srw |  | no-port |
| 8032CEEC | NotifyCompulsorily | dolphin/mcc/mcc.c | div | divwu |  | no-port |
| 8032CF98 | WaitAMinute | dolphin/mcc/mcc.c | div | divwu |  | no-port |
| 8032D4D8 | mccInitializeCheck | dolphin/mcc/mcc.c | div | divwu |  | no-port |
| 8032D72C | MCCInit | dolphin/mcc/mcc.c | div | divwu | melee_runtime/src/tu/hsd_392C.rs:34 | ok-const |
| 8032ECE4 | FIOQuery | dolphin/mcc/fio.c | div | divwu | melee_runtime/src/tu/hsd_3933.rs:38 | ok-const |
| 8032ED60 | FIOQuery | dolphin/mcc/fio.c | div | divwu | melee_runtime/src/tu/hsd_3933.rs:38 | ok-const |
| 8032F26C | fioPacketReceiveResult | dolphin/mcc/fio.c | div | divwu |  | no-port |
| 8032F684 | __THPPrepBitStream | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:541 | fixed |
| 8032F6C0 | __THPPrepBitStream | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:548 | fixed |
| 8032F720 | __THPPrepBitStream | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:548 | fixed |
| 8032F780 | __THPPrepBitStream | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:548 | fixed |
| 8032F7E0 | __THPPrepBitStream | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:548 | fixed |
| 8032FB0C | THPDec_8032F8D4 | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:686 | fixed |
| 8032FB1C | THPDec_8032F8D4 | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:689 | fixed |
| 8032FB30 | THPDec_8032F8D4 | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:693 | fixed |
| 8032FB40 | THPDec_8032F8D4 | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:696 | fixed |
| 80330554 | __THPReadFrameHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1076 | fixed |
| 80330560 | __THPReadFrameHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1077 | fixed |
| 80330588 | __THPReadFrameHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1084 | fixed |
| 803305A8 | __THPReadFrameHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1092 | fixed |
| 803307DC | __THPReadQuantizationTable | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1134 | fixed |
| 80330B08 | __THPReadHuffmanTableSpecification | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1204 | fixed |
| 80330BB4 | __THPReadScaneHeader | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1243 | fixed |
| 80330BD4 | __THPReadScaneHeader | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1250 | fixed |
| 80330C04 | __THPReadScaneHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1260 | fixed |
| 80330C10 | __THPReadScaneHeader | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1264 | fixed |
| 80330C50 | __THPReadScaneHeader | dolphin/thp/THPDec.c | shift | sraw | melee_runtime/src/tu/THPDec.rs:1275 | fixed |
| 803310F4 | THPDec_803310CC | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1427 | fixed |
| 80331114 | THPDec_803310CC | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1433 | fixed |
| 80331144 | THPDec_803310CC | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1444 | fixed |
| 80331168 | THPDec_803310CC | dolphin/thp/THPDec.c | div | divw | melee_runtime/src/tu/THPDec.rs:1451 | fixed |
| 80334A00 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334A74 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334A9C | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334B50 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334B5C | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334B68 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334B78 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334B8C | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334BA8 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334D24 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334D90 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334DC4 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334E5C | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334E64 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334E70 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334E80 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334E8C | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 80334EA4 | __THPHuffDecodeDCTCompY | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1726 | ok-const |
| 8033507C | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803350F0 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335118 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803351CC | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803351D8 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803351E4 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803351F4 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335208 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335224 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 803353C4 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335438 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335460 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335508 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335514 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335520 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335530 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 80335544 | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 8033555C | __THPHuffDecodeDCTCompU | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1736 | ok-const |
| 8033572C | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 803357A0 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 803357C8 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 8033587C | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335888 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335894 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 803358A4 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 803358B8 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 803358D4 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335A74 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335AE8 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335B10 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335BB8 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335BC4 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335BD0 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335BE0 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335BF4 | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | srw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335C0C | __THPHuffDecodeDCTCompV | dolphin/thp/THPDec.c | shift | slw | melee_runtime/src/tu/THPDec.rs:1746 | ok-const |
| 80335F6C | __DBIsExceptionMarked | dolphin/db/db.c | shift | slw |  | no-port |
| 8033C74C | GXSetVtxAttrFmt | dolphin/gx/GXAttr.c | shift | slw | melee_engine/src/gxbridge.rs:169 | ok-const |
| 8033C790 | __GXSetVAT | dolphin/gx/GXAttr.c | shift | slw |  | no-port |
| 8033E814 | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | slw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E818 | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | slw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E83C | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | sraw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E844 | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | sraw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E8A8 | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | slw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E8AC | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | slw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E8C4 | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | sraw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E8CC | GXGetTexBufferSize | dolphin/gx/GXTexture.c | shift | sraw | melee_modes/src/tu/gmcameramode.rs:30 | ok-const |
| 8033E950 | __GetImageTileCount | dolphin/gx/GXTexture.c | shift | slw |  | no-port |
| 8033E960 | __GetImageTileCount | dolphin/gx/GXTexture.c | shift | sraw |  | no-port |
| 8033E964 | __GetImageTileCount | dolphin/gx/GXTexture.c | shift | slw |  | no-port |
| 8033E978 | __GetImageTileCount | dolphin/gx/GXTexture.c | shift | sraw |  | no-port |
| 8033EAB0 | GXInitTexObj | dolphin/gx/GXTexture.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:302 | ok |
| 8033EBD4 | GXInitTexObj | dolphin/gx/GXTexture.c | shift | slw | melee_engine/src/gxbridge.rs:302 | ok-const |
| 8033EBE0 | GXInitTexObj | dolphin/gx/GXTexture.c | shift | slw | melee_engine/src/gxbridge.rs:302 | ok-const |
| 8033EBEC | GXInitTexObj | dolphin/gx/GXTexture.c | shift | sraw | melee_engine/src/gxbridge.rs:302 | ok-const |
| 8033EBF4 | GXInitTexObj | dolphin/gx/GXTexture.c | shift | sraw | melee_engine/src/gxbridge.rs:302 | ok-const |
| 8033ECAC | GXInitTexObjLOD | dolphin/gx/GXTexture.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:316 | ok |
| 8033ED9C | GXInitTexObjLOD | dolphin/gx/GXTexture.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:316 | ok |
| 8033EDE4 | GXInitTexObjLOD | dolphin/gx/GXTexture.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:316 | ok |
| 8033F460 | __GXSetSUTexRegs | dolphin/gx/GXTexture.c | shift | slw |  | no-port |
| 8033F4DC | __GXSetSUTexRegs | dolphin/gx/GXTexture.c | shift | slw |  | no-port |
| 8033F868 | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033F870 | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033F8CC | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033F8D0 | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033F914 | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033F918 | GXSetIndTexMtx | dolphin/gx/GXBump.c | fptoi | fctiwz | melee_engine/src/gxbridge.rs:296 | ok |
| 8033FD4C | __GXUpdateBPMask | dolphin/gx/GXBump.c | shift | slw |  | no-port |
| 80340878 | GXSetFog | dolphin/gx/GXPixel.c | shift | slw | melee_engine/src/gxbridge.rs:247 | ok-const |
| 80345BF4 | EXIImm | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80345C6C | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CC4 | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CCC | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CD4 | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CDC | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CE4 | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CEC | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345CF4 | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345D28 | EXIImm | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 80345EF0 | EXIDma | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346010 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346024 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346034 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346048 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 8034605C | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 8034606C | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346084 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346094 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 803460C0 | EXISync | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 8034658C | EXIAttach | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346658 | EXIDetach | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346740 | EXISelect | dolphin/os/OSExi.c | shift | slw |  | no-port |
| 803469DC | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346A80 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346A94 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346AA4 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346AB8 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346ACC | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346ADC | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346AF4 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346B04 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346B30 | TCIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80346BDC | EXTIntrruptHandler | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 8034706C | EXIGetID | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80347284 | EXIGetID | dolphin/os/OSExi.c | shift | srw |  | no-port |
| 80349694 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 803496A8 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 803496B8 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 803496CC | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 803496E0 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 803496F0 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349708 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349718 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349744 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349770 | CompleteTransfer | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349944 | SIInterruptHandler | dolphin/os/OSSerial.c | shift | sraw |  | no-port |
| 80349A34 | SIInterruptHandler | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349A68 | SIInterruptHandler | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349A9C | SIInterruptHandler | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349AD0 | SIInterruptHandler | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 80349EA0 | __SITransfer | dolphin/os/OSSerial.c | shift | sraw |  | no-port |
| 8034A07C | SIGetStatus | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 8034A290 | SIGetResponseRaw | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 8034A64C | GetTypeCallback | dolphin/os/OSSerial.c | shift | srw |  | no-port |
| 8034A8E4 | SIGetType | dolphin/os/OSSerial.c | shift | sraw |  | no-port |
| 8034AE18 | UnsetRun | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034AF18 | SetEffectivePriority | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034B110 | SelectThread | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034B1DC | SelectThread | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034B718 | OSResumeThread | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034BBB4 | OSWakeupThread | dolphin/os/OSThread.c | shift | slw |  | no-port |
| 8034BCE0 | OSCheckActiveThreads | dolphin/os/OSThread.c | shift | slw | melee_modes/src/tu/gmscene.rs:105 | ok-const |
| 8034CC14 | ClampStick | dolphin/pad/PadClamp.c | div | divw |  | no-port |
| 8034CC18 | ClampStick | dolphin/pad/PadClamp.c | div | divw |  | no-port |
| 8034CC54 | ClampStick | dolphin/pad/PadClamp.c | div | divw |  | no-port |
| 8034CC58 | ClampStick | dolphin/pad/PadClamp.c | div | divw |  | no-port |
| 8034CDB8 | UpdateOrigin | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034CF5C | PADOriginCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034CFA4 | PADOriginCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D014 | PADOriginUpdateCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D0DC | PADProbeCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D110 | PADProbeCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D138 | PADProbeCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D1BC | PADTypeAndStatusCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D1F0 | PADTypeAndStatusCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D268 | PADTypeAndStatusCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D2F4 | PADTypeAndStatusCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D464 | PADTypeAndStatusCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D4D4 | PADReceiveCheckCallback | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034D68C | PADReset | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/controller.rs:45 | ok-const |
| 8034D790 | PADRecalibrate | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/controller.rs:46 | ok-const |
| 8034D9B0 | PADInit | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/controller.rs:47 | ok-const |
| 8034DA54 | PADRead | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/controller.rs:44 | ok-const |
| 8034DAE8 | PADRead | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/controller.rs:44 | ok-const |
| 8034DEE8 | PADControlMotor | dolphin/pad/pad.c | shift | srw | melee_runtime/src/tu/rumble.rs:29 | ok-const |
| 8034E7B8 | OnReset | dolphin/pad/pad.c | shift | srw |  | no-port |
| 8034F1EC | VIInit | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:48 | ok-const |
| 8034F23C | VIInit | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:48 | ok-const |
| 8034F248 | VIInit | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:48 | ok-const |
| 8034F704 | setVerticalRegs | dolphin/vi/vi.c | div | divw |  | no-port |
| 8034FACC | VIConfigure | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:45 | ok-const |
| 8034FB18 | VIConfigure | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:45 | ok-const |
| 8034FB20 | VIConfigure | dolphin/vi/vi.c | div | divw | melee_runtime/src/tu/video.rs:45 | ok-const |
| 8034FCB0 | VIConfigure | dolphin/vi/vi.c | div | divwu | melee_runtime/src/tu/video.rs:45 | ok-const |
| 803501CC | getCurrentFieldEvenOdd | dolphin/vi/vi.c | div | divwu |  | no-port |
| 80350248 | VIGetNextField | dolphin/vi/vi.c | div | divwu | melee_runtime/src/tu/cobj.rs:109 | ok-const |
| 803502F4 | VIGetCurrentLine | dolphin/vi/vi.c | div | divwu |  | no-port |
| 80350F44 | ARInit | dolphin/ar/ar.c | fptoi | fctiwz | melee_runtime/src/tu/lbaudio_ax.rs:46 | ok |
| 80353544 | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035354C | bitrev | dolphin/card/CARDUnlock.c | shift | srw |  | no-port |
| 8035355C | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035356C | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 80353594 | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035359C | bitrev | dolphin/card/CARDUnlock.c | shift | srw |  | no-port |
| 803535AC | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 803535BC | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 803535E4 | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 803535EC | bitrev | dolphin/card/CARDUnlock.c | shift | srw |  | no-port |
| 803535FC | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035360C | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 80353634 | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035363C | bitrev | dolphin/card/CARDUnlock.c | shift | srw |  | no-port |
| 8035364C | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 8035365C | bitrev | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 803537F4 | DummyLen | dolphin/card/CARDUnlock.c | shift | slw |  | no-port |
| 80356220 | DoMount | dolphin/card/CARDMount.c | div | divw |  | no-port |
| 8035789C | CARDCreateAsync | dolphin/card/CARDCreate.c | div | divwu |  | no-port |
| 803579F4 | CARDCreateAsync | dolphin/card/CARDCreate.c | div | divwu |  | no-port |
| 80357A24 | CARDCreateAsync | dolphin/card/CARDCreate.c | div | divwu |  | no-port |
| 80358498 | UpdateIconOffsets | dolphin/card/CARDStat.c | shift | sraw |  | no-port |
| 803584E4 | UpdateIconOffsets | dolphin/card/CARDStat.c | shift | sraw |  | no-port |
| 80358530 | UpdateIconOffsets | dolphin/card/CARDStat.c | shift | sraw |  | no-port |
| 80358580 | UpdateIconOffsets | dolphin/card/CARDStat.c | shift | sraw |  | no-port |
| 8035B6A4 | __AXGetCurrentProfile | dolphin/ax/AXProf.c | div | divwu |  | no-port |
| 8035BAF4 | ReverbHICreate | dolphin/axfx/reverb_hi.c | fptoi | fctiwz |  | no-port |
| 8035BC9C | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BCA4 | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BCAC | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BCB4 | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BD10 | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BD14 | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BD18 | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BD1C | DoCrossTalk | dolphin/axfx/reverb_hi.c | fptoi | fctiw |  | no-port |
| 8035BFFC | HandleReverb | dolphin/axfx/reverb_hi.c | fptoi | fctiwz |  | no-port |
| 8035C18C | HandleReverb | dolphin/axfx/reverb_hi.c | fptoi | fctiwz |  | no-port |
| 8035C85C | ReverbSTDCreate | dolphin/axfx/reverb_std.c | fptoi | fctiwz |  | no-port |
| 8035CB18 | HandleReverb | dolphin/axfx/reverb_std.c | fptoi | fctiwz |  | no-port |
| 8035CC2C | HandleReverb | dolphin/axfx/reverb_std.c | fptoi | fctiwz |  | no-port |
| 8035CF8C | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035CFE4 | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D000 | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D00C | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D010 | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D014 | do_src1 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D14C | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D1D0 | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D1EC | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D1F8 | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D1FC | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D200 | do_src2 | dolphin/axfx/chorus.c | fptoi | fctiwz |  | no-port |
| 8035D51C | AXFXChorusSettings | dolphin/axfx/chorus.c | div | divwu |  | no-port |
| 8035DA0C | AXFXDelayCallback | dolphin/axfx/delay.c | div | divwu | melee_runtime/src/tu/axdriver.rs:54 | ok-const |
| 8035DA28 | AXFXDelayCallback | dolphin/axfx/delay.c | div | divwu | melee_runtime/src/tu/axdriver.rs:54 | ok-const |
| 8035DA44 | AXFXDelayCallback | dolphin/axfx/delay.c | div | divwu | melee_runtime/src/tu/axdriver.rs:54 | ok-const |
| 8035E8C8 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:147 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035E90C | TObjUpdateFunc | sysdolphin/baselib/tobj.c:156 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035E9AC | TObjUpdateFunc | sysdolphin/baselib/tobj.c:193 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035E9D0 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:196 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035E9F4 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:199 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EA18 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:202 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EA3C | TObjUpdateFunc | sysdolphin/baselib/tobj.c:205 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EA60 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:208 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EA84 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:211 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EAA8 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:214 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EACC | TObjUpdateFunc | sysdolphin/baselib/tobj.c:217 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EAF0 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:220 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EB14 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:223 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035EB38 | TObjUpdateFunc | sysdolphin/baselib/tobj.c:226 | fptoi | fctiwz | melee_runtime/src/tu/tobj.rs:421 | ok |
| 8035F504 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F514 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F524 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F534 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F544 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F554 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F564 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 8035F574 | HSD_TObjSetupTextureCoordGen | sysdolphin/baselib/tobj.c | shift | slw | melee_fighter/src/tu/ftmaterial.rs:46 | ok-const |
| 80361A4C | HSD_SetMaterialColor | sysdolphin/baselib/state.c:277 | fptoi | fctiwz | melee_fighter/src/tu/ftmaterial.rs:41 | ok |
| 80362BA8 | ChanUpdateFunc | sysdolphin/baselib/tev.c:492 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362BC8 | ChanUpdateFunc | sysdolphin/baselib/tev.c:495 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362BE8 | ChanUpdateFunc | sysdolphin/baselib/tev.c:498 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362C08 | ChanUpdateFunc | sysdolphin/baselib/tev.c:501 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362C28 | ChanUpdateFunc | sysdolphin/baselib/tev.c:504 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362C48 | ChanUpdateFunc | sysdolphin/baselib/tev.c:507 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362C68 | ChanUpdateFunc | sysdolphin/baselib/tev.c:510 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362C88 | ChanUpdateFunc | sysdolphin/baselib/tev.c:513 | fptoi | fctiwz | melee_runtime/src/tu/tev.rs:121 | ok |
| 80362F28 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:96 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:127 | fixed |
| 80362F4C | MObjUpdateFunc | sysdolphin/baselib/mobj.c:99 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80362F70 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:102 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80362F94 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:105 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80362FB8 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:108 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80362FDC | MObjUpdateFunc | sysdolphin/baselib/mobj.c:111 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80363018 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:117 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 8036303C | MObjUpdateFunc | sysdolphin/baselib/mobj.c:120 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 80363060 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:123 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 8036308C | MObjUpdateFunc | sysdolphin/baselib/mobj.c:127 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 803630B8 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:132 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 803630E4 | MObjUpdateFunc | sysdolphin/baselib/mobj.c:137 | fptoi | fctiwz | melee_runtime/src/tu/mobj.rs:143 | fixed |
| 803655A8 | LObjUpdateFunc | sysdolphin/baselib/lobj.c:184 | fptoi | fctiwz | melee_runtime/src/tu/lobj.rs:342 | ok |
| 803655C8 | LObjUpdateFunc | sysdolphin/baselib/lobj.c:187 | fptoi | fctiwz | melee_runtime/src/tu/lobj.rs:342 | ok |
| 803655E8 | LObjUpdateFunc | sysdolphin/baselib/lobj.c:190 | fptoi | fctiwz | melee_runtime/src/tu/lobj.rs:342 | ok |
| 80365608 | LObjUpdateFunc | sysdolphin/baselib/lobj.c:193 | fptoi | fctiwz | melee_runtime/src/tu/lobj.rs:342 | ok |
| 8036ACA8 | parseFloat | sysdolphin/baselib/fobj.c:140 | shift | slw | melee_runtime/src/tu/fobj.rs:211 | fixed |
| 8036AE20 | parsePackInfo | sysdolphin/baselib/fobj.c:183 | shift | slw | melee_runtime/src/tu/fobj.rs:244 | ok |
| 8036B5B0 | HSD_FObjInterpretAnim | sysdolphin/baselib/fobj.c | shift | slw | melee_runtime/src/tu/fobj.rs:438 | ok-const |
| 8036B790 | HSD_FObjLoadDesc | sysdolphin/baselib/fobj.c:509 | fptoi | fctiwz | melee_runtime/src/tu/aobj.rs:39 | ok |
| 8036B7E4 | HSD_FObjLoadDesc | sysdolphin/baselib/fobj.c | fptoi | fctiwz | melee_runtime/src/tu/aobj.rs:39 | ok |
| 8036C58C | get_shape_vertex_xyz | sysdolphin/baselib/pobj.c:598 | shift | slw | melee_runtime/src/tu/pobj.rs:217 | ok |
| 8036C918 | get_shape_normal_xyz | sysdolphin/baselib/pobj.c:643 | shift | slw | melee_runtime/src/tu/pobj.rs:218 | ok |
| 8036CCC4 | get_shape_nbt_xyz | sysdolphin/baselib/pobj.c:687 | shift | slw | melee_runtime/src/tu/pobj.rs:219 | ok |
| 8036D8F8 | drawShapeAnim | sysdolphin/baselib/pobj.c:871 | fptoi | fctiwz | melee_runtime/src/tu/pobj.rs:222 | fixed |
| 8036D92C | drawShapeAnim | sysdolphin/baselib/pobj.c:871 | fptoi | fctiwz | melee_runtime/src/tu/pobj.rs:222 | fixed |
| 80376AD8 | HSD_PadRenewRawStatus | sysdolphin/baselib/controller.c | div | divw | melee_modes/src/tu/lb_0195.rs:34 | ok-const |
| 80376B94 | HSD_PadRenewRawStatus | sysdolphin/baselib/controller.c | div | divw | melee_modes/src/tu/lb_0195.rs:34 | ok-const |
| 80376C1C | HSD_PadRenewRawStatus | sysdolphin/baselib/controller.c | div | divw | melee_modes/src/tu/lb_0195.rs:34 | ok-const |
| 80376D60 | HSD_PadFlushQueue | sysdolphin/baselib/controller.c | div | divw | melee_modes/src/tu/gmscene.rs:91 | ok-const |
| 80376FF4 | HSD_PadClampCheck3 | sysdolphin/baselib/controller.c:204 | fptoi | fctiwz | melee_runtime/src/tu/controller.rs:265 | ok |
| 80377028 | HSD_PadClampCheck3 | sysdolphin/baselib/controller.c:205 | fptoi | fctiwz | melee_runtime/src/tu/controller.rs:265 | ok |
| 80377174 | HSD_PadClampCheck3 | sysdolphin/baselib/controller.c:210 | fptoi | fctiwz | melee_runtime/src/tu/controller.rs:265 | ok |
| 803771BC | HSD_PadClampCheck3 | sysdolphin/baselib/controller.c:211 | fptoi | fctiwz | melee_runtime/src/tu/controller.rs:265 | ok |
| 8037755C | HSD_PadRenewMasterStatus | sysdolphin/baselib/controller.c | div | divw | melee_modes/src/tu/lb_0195.rs:33 | ok-const |
| 80378AE8 | splGetSplinePoint | sysdolphin/baselib/spline.c:92 | fptoi | fctiwz | melee_runtime/src/tu/spline_world.rs:40 | ok |
| 8037A800 | HSD_GetNbBits | sysdolphin/baselib/util.c:24 | shift | slw |  | no-port |
| 8037A814 | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A828 | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A83C | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A850 | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A864 | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A878 | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A88C | HSD_GetNbBits | sysdolphin/baselib/util.c | shift | slw |  | no-port |
| 8037A9F8 | HSD_ObjAllocAddFree | sysdolphin/baselib/objalloc.c:141 | div | divwu | melee_runtime/src/tu/objalloc.rs:112 | fixed |
| 8037AA08 | HSD_ObjAllocAddFree | sysdolphin/baselib/objalloc.c:143 | div | divwu. | melee_runtime/src/tu/objalloc.rs:113 | fixed |
| 8037DA6C | HSD_FogSet | sysdolphin/baselib/fog.c:40 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:115 | fixed |
| 8037DB40 | HSD_FogSet | sysdolphin/baselib/fog.c:54 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:115 | fixed |
| 8037DBE4 | HSD_FogSet | sysdolphin/baselib/fog.c:74 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:115 | fixed |
| 8037DE28 | HSD_FogAdjInit | sysdolphin/baselib/fog.c:147 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:205 | ok |
| 8037DFB0 | FogUpdateFunc | sysdolphin/baselib/fog.c:201 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:261 | ok |
| 8037DFD0 | FogUpdateFunc | sysdolphin/baselib/fog.c:204 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:261 | ok |
| 8037DFF0 | FogUpdateFunc | sysdolphin/baselib/fog.c:207 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:261 | ok |
| 8037E010 | FogUpdateFunc | sysdolphin/baselib/fog.c:210 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:261 | ok |
| 8037E034 | FogUpdateFunc | sysdolphin/baselib/fog.c:214 | fptoi | fctiwz | melee_runtime/src/tu/fog.rs:261 | ok |
| 80380A28 | HSD_ByteCodeEval | sysdolphin/baselib/bytecode.c:127 | fptoi | fctiwz | melee_runtime/src/tu/bytecode.rs:9 | ok |
| 8038113C | HSD_ByteCodeEval | sysdolphin/baselib/bytecode.c:299 | div | divw | melee_runtime/src/tu/bytecode.rs:9 | ok |
| 80381190 | HSD_ByteCodeEval | sysdolphin/baselib/bytecode.c:307 | div | divw | melee_runtime/src/tu/bytecode.rs:9 | ok |
| 80384F90 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1050 | shift | slw | melee_runtime/src/tu/texp.rs:2023 | fixed |
| 803850B4 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1081 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2089 | fixed |
| 803850B8 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1082 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2094 | fixed |
| 803850BC | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1083 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2099 | fixed |
| 803850F8 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1088 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2108 | fixed |
| 803850FC | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1089 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2113 | fixed |
| 80385100 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1090 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2118 | fixed |
| 80385240 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1117 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2180 | fixed |
| 80385284 | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1121 | fptoi | fctiwz | melee_runtime/src/tu/texp.rs:2192 | fixed |
| 8038539C | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1154 | shift | slw | melee_runtime/src/tu/texp.rs:2238 | fixed |
| 803853EC | HSD_TExpSetReg | sysdolphin/baselib/texp.c:1159 | shift | slw | melee_runtime/src/tu/texp.rs:2245 | fixed |
| 80385950 | order_dag | sysdolphin/baselib/texpdag.c:84 | shift | slw | melee_runtime/src/tu/texpdag.rs:126 | ok |
| 80385A98 | order_dag | sysdolphin/baselib/texpdag.c:85 | shift | slw | melee_runtime/src/tu/texpdag.rs:126 | ok |
| 80385B1C | order_dag | sysdolphin/baselib/texpdag.c:102 | shift | slw | melee_runtime/src/tu/texpdag.rs:126 | ok |
| 803861D0 | make_full_dependancy_mtx | sysdolphin/baselib/texpdag.c:318 | shift | slw | melee_runtime/src/tu/texpdag.rs:322 | ok |
| 803862B4 | HSD_TExpSchedule | sysdolphin/baselib/texpdag.c | shift | slw | melee_runtime/src/tu/texp.rs:60 | ok-const |
| 803895C4 | HSD_Synth_80389334 | sysdolphin/baselib/synth.c:764 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:180 | fixed |
| 8038A064 | HSD_SynthSFXVolumeEnvelope | sysdolphin/baselib/synth.c:1046 | shift | slw | melee_runtime/src/tu/synth.rs:132 | fixed |
| 8038A108 | HSD_SynthSFXVolumeEnvelope | sysdolphin/baselib/synth.c:1057 | shift | slw | melee_runtime/src/tu/synth.rs:132 | fixed |
| 8038A348 | HSD_SynthSFXVolumeEnvelope | sysdolphin/baselib/synth.c:1098 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:142 | fixed |
| 8038A784 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1205 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:164 | fixed |
| 8038A79C | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1206 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:165 | fixed |
| 8038A7B4 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1207 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A7CC | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1208 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A7E4 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1209 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A7FC | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1210 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A838 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1213 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A850 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1214 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A868 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1215 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A89C | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1220 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A8B4 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1221 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A8CC | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1222 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A8FC | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1226 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A914 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1227 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A92C | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1228 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A944 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1229 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A95C | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1230 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038A974 | HSD_SynthSFXUpdateMix | sysdolphin/baselib/synth.c:1231 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:166 | fixed |
| 8038AB24 | HSD_SynthSFXUpdateAllVolume | sysdolphin/baselib/synth.c:1286 | shift | slw | melee_runtime/src/tu/lbaudio_ax.rs:66 | ok |
| 8038AB50 | HSD_SynthSFXUpdateAllVolume | sysdolphin/baselib/synth.c | shift | slw | melee_runtime/src/tu/lbaudio_ax.rs:66 | ok |
| 8038B1B8 | HSD_SynthPStreamFirstHakoDataCallback | sysdolphin/baselib/synth.c:1471 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:201 | fixed |
| 8038B860 | HSD_SynthStreamSetVolume | sysdolphin/baselib/synth.c:1655 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:207 | fixed |
| 8038B894 | HSD_SynthStreamSetVolume | sysdolphin/baselib/synth.c:1656 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:207 | fixed |
| 8038B8D0 | HSD_SynthStreamSetVolume | sysdolphin/baselib/synth.c | shift | slw | melee_runtime/src/tu/lbaudio_ax.rs:65 | ok-const |
| 8038B9EC | HSD_SynthInit | sysdolphin/baselib/synth.c:1677 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:208 | fixed |
| 8038BA20 | HSD_SynthInit | sysdolphin/baselib/synth.c:1678 | fptoi | fctiwz | melee_runtime/src/tu/synth.rs:208 | fixed |
| 8038BFF4 | AXDriverExec | sysdolphin/baselib/axdriver.c:256 | shift | slw | melee_runtime/src/tu/axdriver.rs:285 | fixed |
| 8038D0E8 | HSD_AudioSFXStartParam | sysdolphin/baselib/axdriver.c | shift | slw | melee_runtime/src/tu/axdriver.rs:655 | ok-const |
| 8038E108 | HSD_AudioGetAuxHeapSize | sysdolphin/baselib/axdriver.c:1193 | fptoi | fctiwz | melee_runtime/src/tu/axdriver.rs:1030 | fixed |
| 8038E20C | HSD_AudioGetAuxHeapSize | sysdolphin/baselib/axdriver.c:1208 | fptoi | fctiwz | melee_runtime/src/tu/axdriver.rs:1040 | fixed |
| 8038E734 | HSD_AudioPStreamPauseCh | sysdolphin/baselib/axdriver.c:1373 | shift | slw | melee_runtime/src/tu/axdriver.rs:1187 | fixed |
| 8038E8B8 | HSD_AudioPStreamResumeCh | sysdolphin/baselib/axdriver.c:1420 | shift | slw | melee_runtime/src/tu/axdriver.rs:1225 | fixed |
| 80391B00 | hsd_80391AC8 | sysdolphin/baselib/hsd_3915.c:249 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3915.rs:295 | fixed |
| 80391DE4 | hsd_80391AC8 | sysdolphin/baselib/hsd_3915.c:270 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3915.rs:318 | fixed |
| 80392278 | hsd_803921B8 | sysdolphin/baselib/hsd_3915.c:477 | shift | srw | melee_runtime/src/tu/debugconsole_main.rs:59 | ok |
| 803923E8 | hsd_803922FC | sysdolphin/baselib/hsd_3915.c:544 | shift | srw | melee_runtime/src/tu/debugconsole_main.rs:60 | ok |
| 80392B68 | fn_80392A3C | sysdolphin/baselib/hsd_392A.c:154 | fptoi | fctiwz | melee_runtime/src/tu/hsd_392A.rs:169 | fixed |
| 80393048 | hsd_80392E80 | sysdolphin/baselib/hsd_392C.c:203 | div | divwu | melee_modes/src/tu/gmscene.rs:166 | ok |
| 803933F4 | hsd_80393328 | sysdolphin/baselib/hsd_3933.c:89 | div | divwu | melee_runtime/src/tu/hsd_3933.rs:199 | fixed |
| 80393B68 | hsd_80393A5C | sysdolphin/baselib/hsd_3933.c:354 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3933.rs:379 | fixed |
| 80393C6C | fn_80393C14 | sysdolphin/baselib/hsd_393C.c:46 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:68 | fixed |
| 80393C8C | fn_80393C14 | sysdolphin/baselib/hsd_393C.c:49 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:71 | fixed |
| 80393CD0 | fn_80393C14 | sysdolphin/baselib/hsd_393C.c:58 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:81 | fixed |
| 80393D04 | fn_80393C14 | sysdolphin/baselib/hsd_393C.c:64 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:87 | fixed |
| 80393EA4 | hsd_80393E68 | sysdolphin/baselib/hsd_393C.c:133 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:154 | fixed |
| 80393F6C | hsd_80393EF4 | sysdolphin/baselib/hsd_393C.c:181 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:194 | fixed |
| 803940DC | hsd_80394068 | sysdolphin/baselib/hsd_393C.c:236 | div | divwu | melee_runtime/src/tu/debugconsole_main.rs:62 | ok |
| 803941A0 | hsd_80394128 | sysdolphin/baselib/hsd_393C.c:261 | div | divwu | melee_runtime/src/tu/hsd_393C.rs:266 | ok |
| 80396158 | hsd_80396130 | sysdolphin/baselib/debugconsole_main.c:1649 | div | divwu |  | no-port |
| 8039625C | hsd_80396188 | sysdolphin/baselib/debugconsole_main.c | div | divwu |  | no-port |
| 80396354 | hsd_803962A8 | sysdolphin/baselib/debugconsole_main.c:1738 | div | divwu |  | no-port |
| 80396390 | hsd_803962A8 | sysdolphin/baselib/debugconsole_main.c:1748 | div | divwu |  | no-port |
| 803963CC | hsd_803962A8 | sysdolphin/baselib/debugconsole_main.c:1758 | div | divwu |  | no-port |
| 80396408 | hsd_803962A8 | sysdolphin/baselib/debugconsole_main.c:1768 | div | divwu |  | no-port |
| 80396A64 | hsd_80396A20 | sysdolphin/baselib/debugconsole_main.c:1949 | shift | slw |  | no-port |
| 80396AD0 | hsd_80396A20 | sysdolphin/baselib/debugconsole_main.c:1955 | shift | srw |  | no-port |
| 80396AE0 | hsd_80396A20 | sysdolphin/baselib/debugconsole_main.c:1955 | shift | slw |  | no-port |
| 80396AF4 | hsd_80396A20 | sysdolphin/baselib/debugconsole_main.c:1959 | shift | srw |  | no-port |
| 80396B04 | hsd_80396A20 | sysdolphin/baselib/debugconsole_main.c:1959 | shift | slw |  | no-port |
| 80396FE4 | hsd_80396E40 | sysdolphin/baselib/debugconsole_main.c:2159 | shift | slw |  | no-port |
| 8039766C | hsd_803975D4 | sysdolphin/baselib/debugconsole_main.c:2432 | shift | srw |  | no-port |
| 8039768C | hsd_803975D4 | sysdolphin/baselib/debugconsole_main.c | shift | srw |  | no-port |
| 803976AC | hsd_803975D4 | sysdolphin/baselib/debugconsole_main.c | shift | srw |  | no-port |
| 803976C8 | hsd_803975D4 | sysdolphin/baselib/debugconsole_main.c | shift | srw |  | no-port |
| 80397C88 | fn_80397814 | sysdolphin/baselib/debugconsole_main.c:2741 | div | divw |  | no-port |
| 8039A410 | hsd_8039930C | sysdolphin/baselib/particle.c:1259 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039A43C | hsd_8039930C | sysdolphin/baselib/particle.c:1267 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039A5CC | hsd_8039930C | sysdolphin/baselib/particle.c:1322 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039A9A0 | hsd_8039930C | sysdolphin/baselib/particle.c:1490 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B00C | hsd_8039930C | sysdolphin/baselib/particle.c:1781 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B0E4 | hsd_8039930C | sysdolphin/baselib/particle.c:1815 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B150 | hsd_8039930C | sysdolphin/baselib/particle.c:1827 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B1BC | hsd_8039930C | sysdolphin/baselib/particle.c:1839 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B228 | hsd_8039930C | sysdolphin/baselib/particle.c:1851 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B274 | hsd_8039930C | sysdolphin/baselib/particle.c:1869 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B350 | hsd_8039930C | sysdolphin/baselib/particle.c:1903 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B3C4 | hsd_8039930C | sysdolphin/baselib/particle.c:1915 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B438 | hsd_8039930C | sysdolphin/baselib/particle.c:1927 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B4AC | hsd_8039930C | sysdolphin/baselib/particle.c:1939 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B518 | hsd_8039930C | sysdolphin/baselib/particle.c:1956 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B784 | hsd_8039930C | sysdolphin/baselib/particle.c:2055 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039B8CC | hsd_8039930C | sysdolphin/baselib/particle.c:2117 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BA14 | hsd_8039930C | sysdolphin/baselib/particle.c:2183 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BAAC | hsd_8039930C | sysdolphin/baselib/particle.c:2207 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BB8C | hsd_8039930C | sysdolphin/baselib/particle.c:2242 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BBD4 | hsd_8039930C | sysdolphin/baselib/particle.c:2250 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BC48 | hsd_8039930C | sysdolphin/baselib/particle.c:2262 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BC90 | hsd_8039930C | sysdolphin/baselib/particle.c:2270 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BD04 | hsd_8039930C | sysdolphin/baselib/particle.c:2282 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BD4C | hsd_8039930C | sysdolphin/baselib/particle.c:2290 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BDC0 | hsd_8039930C | sysdolphin/baselib/particle.c:2302 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BE08 | hsd_8039930C | sysdolphin/baselib/particle.c:2310 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BE70 | hsd_8039930C | sysdolphin/baselib/particle.c:2335 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BF08 | hsd_8039930C | sysdolphin/baselib/particle.c:2361 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039BFCC | hsd_8039930C | sysdolphin/baselib/particle.c:2391 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C068 | hsd_8039930C | sysdolphin/baselib/particle.c:2410 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C0B8 | hsd_8039930C | sysdolphin/baselib/particle.c:2420 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C138 | hsd_8039930C | sysdolphin/baselib/particle.c:2436 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C188 | hsd_8039930C | sysdolphin/baselib/particle.c:2446 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C208 | hsd_8039930C | sysdolphin/baselib/particle.c:2462 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C258 | hsd_8039930C | sysdolphin/baselib/particle.c:2472 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C2BC | hsd_8039930C | sysdolphin/baselib/particle.c:2481 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C32C | hsd_8039930C | sysdolphin/baselib/particle.c:2492 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C374 | hsd_8039930C | sysdolphin/baselib/particle.c:2502 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C658 | hsd_8039930C | sysdolphin/baselib/particle.c:2644 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C73C | hsd_8039930C | sysdolphin/baselib/particle.c:2691 | div | divw | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039C8A4 | hsd_8039930C | sysdolphin/baselib/particle.c:2757 | fptoi | fctiwz | melee_runtime/src/tu/particle.rs:1685 | ok |
| 8039E098 | hsd_8039DAD4 | sysdolphin/baselib/generator.c:540 | fptoi | fctiwz | melee_runtime/src/tu/generator.rs:442 | fixed |
| 8039E0E4 | hsd_8039DAD4 | sysdolphin/baselib/generator.c:549 | fptoi | fctiwz | melee_runtime/src/tu/generator.rs:453 | fixed |
| 8039E12C | hsd_8039DAD4 | sysdolphin/baselib/generator.c:557 | fptoi | fctiwz | melee_runtime/src/tu/generator.rs:453 | fixed |
| 8039EEAC | hsd_8039EE24 | sysdolphin/baselib/generator.c:982 | shift | slw | melee_runtime/src/tu/generator.rs:708 | ok |
| 8039FA44 | getColorPrimEnv | sysdolphin/baselib/psdisp.c:153 | div | divw | melee_runtime/src/tu/psdisp.rs:325 | fixed |
| 8039FAE8 | getColorPrimEnv | sysdolphin/baselib/psdisp.c:170 | div | divw | melee_runtime/src/tu/psdisp.rs:352 | fixed |
| 8039FB90 | getColorMatAmb | sysdolphin/baselib/psdisp.c:191 | div | divw | melee_runtime/src/tu/psdisp.rs:386 | fixed |
| 8039FC0C | getColorMatAmb | sysdolphin/baselib/psdisp.c:204 | div | divw | melee_runtime/src/tu/psdisp.rs:407 | fixed |
| 803A025C | psDispParticles | sysdolphin/baselib/psdisp.c:1941 | shift | slw | melee_runtime/src/tu/psdisp.rs:3132 | fixed |
| 803A080C | psDispParticles | sysdolphin/baselib/psdisp.c:2072 | div | divw | melee_runtime/src/tu/psdisp.rs:3395 | fixed |
| 803A156C | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A1680 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A17AC | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A1908 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A1A74 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A1CE4 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A28AC | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A29C8 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A2B84 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A3320 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A3440 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A3604 | psDispParticles | sysdolphin/baselib/psdisp.c | fptoi | fctiwz | melee_runtime/src/tu/psdisp.rs:3078 | ok |
| 803A6D00 | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:270 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A6D04 | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:273 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A6DEC | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:281 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A6E1C | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:282 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A6E34 | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:283 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A6E60 | HSD_SisLib_803A6B98 | sysdolphin/baselib/hsd_3A64.c:284 | fptoi | fctiwz | melee_items/src/tu/tylist.rs:45 | ok |
| 803A749C | HSD_SisLib_803A746C | sysdolphin/baselib/hsd_3A64.c:437 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A64.rs:471 | fixed |
| 803A74A4 | HSD_SisLib_803A746C | sysdolphin/baselib/hsd_3A64.c:439 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A64.rs:473 | fixed |
| 803A7578 | HSD_SisLib_803A7548 | sysdolphin/baselib/hsd_3A64.c:468 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:65 | ok |
| 803A7580 | HSD_SisLib_803A7548 | sysdolphin/baselib/hsd_3A64.c:469 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:65 | ok |
| 803A75A4 | HSD_SisLib_803A7548 | sysdolphin/baselib/hsd_3A64.c:470 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:65 | ok |
| 803A75A8 | HSD_SisLib_803A7548 | sysdolphin/baselib/hsd_3A64.c:471 | fptoi | fctiwz | melee_modes/src/tu/gmregclear.rs:65 | ok |
| 803A780C | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:86 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:166 | fixed |
| 803A7838 | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:88 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:168 | fixed |
| 803A7860 | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:90 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:183 | fixed |
| 803A788C | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:92 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:185 | fixed |
| 803A7B70 | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:149 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:185 | fixed |
| 803A7B9C | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:151 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:185 | fixed |
| 803A7BC4 | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:153 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:185 | fixed |
| 803A7BF0 | HSD_SisLib_803A7684 | sysdolphin/baselib/hsd_3A76.c:155 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3A76.rs:185 | fixed |
| 803A95A4 | hsd_803A949C | sysdolphin/baselib/card.c:186 | div | divwu | melee_runtime/src/tu/card.rs:1155 | fixed |
| 803A986C | hsd_803A949C | sysdolphin/baselib/card.c:255 | div | divwu | melee_runtime/src/tu/card.rs:1155 | fixed |
| 803AA600 | hsd_803A949C | sysdolphin/baselib/card.c:625 | div | divwu | melee_runtime/src/tu/card.rs:1155 | fixed |
| 803AB444 | hsd_803AAA48 | sysdolphin/baselib/card.c:1232 | div | divwu | melee_runtime/src/tu/card.rs:1622 | fixed |
| 803AB864 | hsd_803AAA48 | sysdolphin/baselib/card.c:1336 | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB874 | hsd_803AAA48 | sysdolphin/baselib/card.c:1339 | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB898 | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB8E8 | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB904 | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB920 | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB94C | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803AB9A8 | hsd_803AAA48 | sysdolphin/baselib/card.c | shift | slw | melee_runtime/src/tu/card.rs:1594 | ok |
| 803ABEF0 | hsd_803AAA48 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:1710 | fixed |
| 803ABEFC | hsd_803AAA48 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:1800 | fixed |
| 803AC668 | fn_803AC634 | sysdolphin/baselib/card.c:1747 | div | divwu | melee_runtime/src/tu/card.rs:2828 | ok |
| 803AC694 | fn_803AC634 | sysdolphin/baselib/card.c:1751 | div | divwu | melee_runtime/src/tu/card.rs:2828 | ok |
| 803AC6B0 | fn_803AC634 | sysdolphin/baselib/card.c:1756 | div | divwu | melee_runtime/src/tu/card.rs:2828 | ok |
| 803AC700 | fn_803AC6B8 | sysdolphin/baselib/card.c:1813 | div | divwu | melee_runtime/src/tu/card.rs:2887 | ok |
| 803AC72C | fn_803AC6B8 | sysdolphin/baselib/card.c:1817 | div | divwu | melee_runtime/src/tu/card.rs:2887 | ok |
| 803AC778 | fn_803AC6B8 | sysdolphin/baselib/card.c:1821 | div | divwu | melee_runtime/src/tu/card.rs:2887 | ok |
| 803AC7A4 | fn_803AC6B8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:2887 | ok |
| 803AC7C0 | fn_803AC6B8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:2887 | ok |
| 803AC808 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC834 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC8AC | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC8D8 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC8F4 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC928 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC954 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC970 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC99C | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC9C8 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803AC9E4 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACA14 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACA40 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACA5C | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACA8C | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACAB8 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACAD4 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACB10 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACB3C | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACB58 | fn_803AC7DC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3013 | ok-const |
| 803ACBF8 | fn_803ACBE8 | sysdolphin/baselib/card.c:1952 | div | divwu | melee_runtime/src/tu/card.rs:3077 | ok |
| 803ACC58 | fn_803ACC0C | sysdolphin/baselib/card.c:1982 | div | divwu | melee_runtime/src/tu/card.rs:3087 | ok |
| 803ACCB0 | fn_803ACC0C | sysdolphin/baselib/card.c:2006 | div | divwu | melee_runtime/src/tu/card.rs:3087 | ok |
| 803ACF0C | fn_803ACD58 | sysdolphin/baselib/card.c:2059 | div | divwu | melee_runtime/src/tu/card.rs:3166 | ok |
| 803ACF9C | fn_803ACF30 | sysdolphin/baselib/card.c:2137 | div | divwu | melee_runtime/src/tu/card.rs:3297 | ok |
| 803AD000 | fn_803ACFC0 | sysdolphin/baselib/card.c:2181 | div | divwu | melee_runtime/src/tu/card.rs:3359 | ok |
| 803AD024 | fn_803ACFC0 | sysdolphin/baselib/card.c:2189 | div | divwu | melee_runtime/src/tu/card.rs:3359 | ok |
| 803AD324 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AD350 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AD36C | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AD894 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AD8C0 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AD8DC | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADA18 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADA44 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADA60 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADB44 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADC48 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADC90 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADD24 | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803ADD6C | fn_803AD16C | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:3666 | ok-const |
| 803AE090 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE0BC | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE0D8 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE338 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE394 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE428 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE480 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE5F0 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE684 | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE6DC | fn_803ADF90 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4358 | ok-const |
| 803AE900 | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AE92C | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AE948 | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AECB8 | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AEDD4 | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AEFCC | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AF198 | fn_803AE7F8 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:4576 | ok-const |
| 803AF4F0 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AF51C | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AF538 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AF7F0 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AF908 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AFAE0 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AFD20 | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803AFECC | fn_803AF3F0 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:5531 | ok-const |
| 803B0220 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B024C | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0268 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0600 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0660 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0774 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B093C | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0AA4 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0AE8 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0C88 | fn_803B0120 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:6155 | ok-const |
| 803B0F24 | fn_803B0E9C | sysdolphin/baselib/card.c:4572 | div | divwu | melee_runtime/src/tu/card.rs:6713 | ok |
| 803B107C | fn_803B0E9C | sysdolphin/baselib/card.c:4577 | div | divwu | melee_runtime/src/tu/card.rs:6713 | ok |
| 803B137C | fn_803B1338 | sysdolphin/baselib/card.c:4848 | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B13A8 | fn_803B1338 | sysdolphin/baselib/card.c:4854 | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B13D8 | fn_803B1338 | sysdolphin/baselib/card.c:4876 | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1404 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1420 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1444 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B14B8 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B15F4 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1698 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1818 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1844 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1860 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B18A4 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1948 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1B14 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1BE4 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1C10 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1C40 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1C6C | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1CB4 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1CE0 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1CFC | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1D5C | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B1E00 | fn_803B1338 | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:7154 | ok |
| 803B26A8 | hsd_803B2674 | sysdolphin/baselib/card.c:5381 | div | divwu | melee_runtime/src/tu/card.rs:8277 | ok |
| 803B2758 | fn_803B26CC | sysdolphin/baselib/card.c | div | divwu | melee_runtime/src/tu/card.rs:8288 | ok-const |
| 803B354C | hsd_803B3408 | sysdolphin/baselib/hsd_3B34.c:226 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:67 | fixed |
| 803B35B4 | hsd_803B3408 | sysdolphin/baselib/hsd_3B34.c:233 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:67 | fixed |
| 803B3688 | hsd_803B3408 | sysdolphin/baselib/hsd_3B34.c:251 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:67 | fixed |
| 803B36F4 | hsd_803B3408 | sysdolphin/baselib/hsd_3B34.c | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:67 | fixed |
| 803B38C0 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:348 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:84 | fixed |
| 803B38C8 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:349 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:85 | fixed |
| 803B38D8 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:350 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:86 | fixed |
| 803B38E8 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:352 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:87 | fixed |
| 803B3968 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:360 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:90 | fixed |
| 803B3970 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:362 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:91 | fixed |
| 803B3980 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:364 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:92 | fixed |
| 803B3988 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:366 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3AE4 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:393 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3AEC | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:394 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3AFC | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:395 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3B0C | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:397 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3B8C | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:406 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3B94 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:408 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3BA4 | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:410 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3BAC | fn_803B376C | sysdolphin/baselib/hsd_3B34.c:412 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B34.rs:93 | fixed |
| 803B3D8C | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3DA4 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3DBC | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3DD4 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3DEC | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3E04 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3E1C | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3E34 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3E78 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3E84 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3F68 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B3F74 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4074 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B408C | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B40A4 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B40BC | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B40D4 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B40EC | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4104 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B411C | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4158 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4164 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4238 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4244 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4320 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4338 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4350 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4368 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4380 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4398 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B43B0 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B43C8 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4404 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4410 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B44EC | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B44F8 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B45F4 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4600 | hsd_803B3CD8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:128 | ok-const |
| 803B4B8C | hsd_803B4A2C | sysdolphin/baselib/hsd_3B34.c:646 | div | divw | melee_runtime/src/tu/hsd_3B34.rs:135 | ok |
| 803B4D0C | hsd_803B4A2C | sysdolphin/baselib/hsd_3B34.c:679 | div | divw | melee_runtime/src/tu/hsd_3B34.rs:135 | ok |
| 803B587C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5880 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5890 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5894 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58A4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58A8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58B8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58BC | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58CC | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58D0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58E0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58E4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58F4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B58F8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B590C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5910 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5970 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5974 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5984 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5988 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5998 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B599C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59AC | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59B0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59C0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59C4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59D4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59D8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59E8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B59EC | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A00 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A04 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A4C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A50 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A60 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A64 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A74 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A78 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A88 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A8C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5A9C | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AA0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AB0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AB4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AC4 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AC8 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5ADC | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5AE0 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | div | divw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5B30 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5B38 | hsd_803B51C8 | sysdolphin/baselib/hsd_3B34.c | shift | slw | melee_runtime/src/tu/hsd_3B34.rs:177 | ok-const |
| 803B5D2C | hsd_803B5C4C | sysdolphin/baselib/hsd_3B5C.c:181 | shift | slw | melee_runtime/src/tu/hsd_3B5C.rs:38 | ok |
| 803B5EE0 | hsd_803B5EA0 | sysdolphin/baselib/hsd_3B5C.c:275 | shift | slw | melee_runtime/src/tu/hsd_3B5C.rs:60 | ok |
| 803B5EEC | hsd_803B5EA0 | sysdolphin/baselib/hsd_3B5C.c:276 | shift | slw | melee_runtime/src/tu/hsd_3B5C.rs:60 | ok |
| 803B6164 | hsd_803B5EA0 | sysdolphin/baselib/hsd_3B5C.c:301 | shift | slw | melee_runtime/src/tu/hsd_3B5C.rs:60 | ok |
| 803B6170 | hsd_803B5EA0 | sysdolphin/baselib/hsd_3B5C.c:302 | shift | slw | melee_runtime/src/tu/hsd_3B5C.rs:60 | ok |
| 803B63DC | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:387 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B63E0 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:388 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B63E4 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:390 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B63FC | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:391 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B6410 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:392 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B641C | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:393 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B6444 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:394 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B644C | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:395 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B667C | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:432 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B6680 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:433 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B6684 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:436 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B669C | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:437 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B66B0 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:438 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B66BC | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:439 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B66E4 | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:440 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B66EC | fn_803B61B4 | sysdolphin/baselib/hsd_3B5C.c:441 | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:95 | fixed |
| 803B6A64 | fn_803B6820 | sysdolphin/baselib/hsd_3B5C.c | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:107 | ok |
| 803B6ADC | fn_803B6820 | sysdolphin/baselib/hsd_3B5C.c | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:107 | ok |
| 803B6B58 | fn_803B6820 | sysdolphin/baselib/hsd_3B5C.c | fptoi | fctiwz | melee_runtime/src/tu/hsd_3B5C.rs:107 | ok |
