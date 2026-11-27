# Static audits of the retail main.dol: undefined register reads, fused multiply-add sites

Date 2026-11-27, branch `port/dol-audit`. Static only: nothing was built or run. Inputs: the retail NTSC 1.02
`main.dol` (local, never committed), `config/GALE01/symbols.txt`, `config/GALE01/splits.txt`, the decomp C under
`src/` and `libs/dolphin/`, and the Rust port under `rust/melee_*/src/`. Scripts: `tools/rust-port/dol_audit/`
(`dol.py` shared, `regread.py` audit 1, `fusion.py` audit 2); each runs as `python -I tools/rust-port/dol_audit/<x>.py`
with the default paths and prints counts, names and addresses only (`--tsv OUT` writes the same rows to a file).
Runtime about 1 min (regread) and 40 s (fusion).

Areas (from the split's TU path): fighter = `melee/ft/`, `melee/pl/`; items = `melee/it/`; stage/coll/cam/fx =
`melee/gr/`, `melee/mp/`, `melee/cm/`, `melee/ef/`; lb = `melee/lb/`; HSD = `sysdolphin/`; other = the rest
(menus, modes, SDK, MSL, TRK...).

## Audit 1: registers read before any path wrote them

### Method

- Every `type:function` symbol (19 827) is decoded over `[addr, addr+size)` into basic blocks. Branches inside the
  range are edges; `b` out of the range is a tail call; `bctr` is resolved from MWCC's jump-table idiom
  (`lis/addi` table base, `lwzx`, `mtctr`, bounded by the preceding `cmplwi`); a `bctr` that does not match is
  treated as an exit and listed.
- Forward dataflow over r0-r31, f0-f31, the 32 CR bits, CTR and XER[CA]. Each register carries the set of
  "undefined origins" that can reach it on some path (empty set = defined on every path; merge = union).
  Origins:
  - callee-saved caller value: r14-r31, f14-f31, cr2-cr4 at entry. A store of such a value to the frame
    (`stw/stmw/stfd/psq_st rN, x(r1)`) is a save, not a read.
  - argument register beyond the prototype: r3-r10 / f1-f8 not covered by the decomp C prototype (ints and
    pointers r3.. in order, 64-bit aligned to an odd pair, a struct return takes r3, floats f1..; varargs: all
    defined, cr1 too). Prototypes are parsed from the C definitions (8 functions have none: treated as varargs).
  - scratch register unset at entry: r0, r11, r12, f0, f9-f13, cr0/1/5-7, CTR, CA.
  - caller-saved garbage after a call: after `bl`, r0, r3-r12, f0-f13, cr0/1/5-7, CTR, CA are undefined, except
    the callee's return register per its prototype (r3 for int/pointer/struct, r3:r4 for 64-bit, f1 for float;
    an unknown or asm callee defines r3, r4, f1).
  - result register of a void callee: r3/f1 after a call to a function whose prototype returns void (or r3 after
    a float function).
  - stack slot read before any store: frame-relative loads (`x(r1)`, sp delta tracked through `stwu`/`addi r1`)
    of bytes no path stored. Taking a local's address (`addi rX, r1, k`) counts as a store of everything above it.
- Taint propagates through computation (an instruction with an undefined input produces undefined outputs).
  A tainted value reaching a store (not to the frame), a load address, a conditional branch, a call argument
  (callee's prototype), an indirect jump or the return value (caller's prototype) is a **sink**. "dead" = the
  value reaches no sink (benign (a)). Reads at a `bl` of the callee's parameter registers count as reads, which
  is how the `ftCo_800AC5A0` r5 case shows up.
- Missing returns: a non-void function whose `blr` sees r3/f1 undefined ("missing return value"), and one whose C
  body has no `return <expr>` at all while some path reaches `blr` without writing r3/f1 ("no return statement;
  r3/f1 = entry value"). For both, every DOL call site is scanned forward for a read of r3/f1 before it is written:
  if none reads it, the hit is "benign(result unused)".
- Rust port lookup: the same-named function (or its snake-case body behind a hook shim) in the file named after
  the TU, else the largest non-forwarder body. Its comments are searched for uninitialised/zero-init/console
  wording; `port:` says what the comment says ("0 (per comment)", "console value (per comment)") or "no comment
  (not checked)". The Rust port initialises locals (c2rust style `let mut x = 0`), so an unchecked site most likely
  uses 0 there: that is an inference, not checked per site.

### Validation against the cases found by hand

| case | found | how |
|---|---|---|
| `ftCo_800ADE48` caller's r31 (switch_cmd) | yes | r31 callee-saved, read@800AE270, sink branch |
| `ftCo_800AC5A0` unset r5 / r30 to `ftCo_800B46B8` | yes | r5 arg beyond prototype read@800AC74C, r30 callee-saved read@800AC754, both sink call-arg |
| `it_802E7654`, `itLinkArrow_802A81C4` no return | yes | missing return value; no DOL caller reads r3 (itLinkArrow_802A81C4 has no `bl` caller) |
| `it_802A4BFC`, `mpLib_80056C54`, `lbColl_800077A0` sqrtf spill slots outside a local | no (by design) | on the console the spill is a store then a load of the same slot: defined in the binary. These are C-level UB (the C build/clang drop the store), not undefined reads; `mpLib_80056C54` does show a separate hit (f27 callee-saved, sink branch) |

### Limits (what the dataflow cannot see)

- Path-insensitive: correlated conditions are not followed. Most hits are "variable set in every `case` of a
  switch without a default" or "set inside a loop that always runs" (e.g. `ftMs_SpecialS_80137940`: `msid` is set
  for the two motions the function is only ever called in). Such a hit is real in the binary but needs a path
  feasibility check; "OBSERVABLE?" means only "reaches a sink", not "happens in a match".
- Memory other than the own frame is not tracked (struct fields read before init, heap, globals). Stack slots
  passed by address to a callee are assumed written. Stack-slot hits are often partial struct copies (a Vec3
  with an unset field copied with lwz/stw).
- Callee effects are only the ABI: a callee that preserves a scratch register or returns in r4 without a 64-bit
  prototype is not modelled; prototypes come from the decomp (a decomp prototype that differs from what the
  caller's TU declared shows up as "argument register beyond the prototype" or a call-arg hit).
- 28 `bctr` in 20 functions are not resolved (listed below; code reached only through them is not analysed).
- Hand-written asm functions (`asm` in the C) are analysed but flagged; nearly all asm hits are in other (SDK, TRK,
  exception handlers saving whole contexts), where reads of "undefined" registers are intended.
- Interrupt/exception entry points read everything by design (TRK, OS exception vectors).

### Totals

```
functions analysed 19827, without a C prototype 8, bctr not resolved to a table 28 (in 20 functions)

hits (one per function x origin): 835  observable-candidate: 789
  argument register beyond the prototype                         dead            8
  argument register beyond the prototype                         sink           92
  callee-saved caller value                                      dead            8
  callee-saved caller value                                      sink          478
  caller-saved garbage after a call                              sink           46
  missing return value                                           sink           40
  missing return value (no return statement; r3/f1 = entry value) sink            3
  result register of a void callee                               sink            2
  scratch register unset at entry                                sink           44
  stack slot read before any store                               dead           30
  stack slot read before any store                               sink           84

by area (all / observable-candidate / in asm functions):
  fighter                  89     86      0
  items                    48     44      0
  stage/coll/cam/fx       127    127      0
  lb                       38     13      0
  HSD                      67     66      0
  other                   466    453    304
```

In the online-relevant areas (fighter, items, stage/coll/cam/fx, lb, HSD): 369 hits; 33 benign (value dead),
11 benign (no caller reads the missing result), 325 reach a sink. By origin, of those 325: 223 callee-saved
caller value, 56 stack slot, 21 argument register beyond the prototype, 16 missing return value, 8 scratch at
entry, 1 caller-saved after a call. Other: 466 hits, 296 of them in asm functions.

### Hand-checked observable hits (fighter / items first)

| function | reg | what the console does | Rust port |
|---|---|---|---|
| `ftCo_800ADE48` (CPU) | r31 | reads the caller's r31 for `switch_cmd` on one path | reproduces it: `ftCo_800ADE48_r31(fp, caller_r31_nonzero)` (ftCo_0A01.rs:5574) |
| `ftCo_800AC5A0` (CPU behaviour 18) | r5, r30 | passes unset r5 / r30 to two `ftCo_800B46B8` calls (near_zero path) | `stick_x`/`stick_y` start at 0, so 0 (what Slippi's `NanaDeterminism.asm` forces, not vanilla); the comment at ftCo_0A01.rs:5195 says so |
| `ftCo_8008E9D0` (damage enter) | r31 | inlines `ftCo_8008DA4C`, whose `result` is unset when `dmg.x1838_percentTemp == 0`; the branch then tests the caller's r31. Both DOL callers (`Fighter_procCollResolve`, `ftCo_8008EC90`) set r31 from r3 at entry (a pointer, so likely nonzero: `ftCo_800C0408` colanim cleanup runs). UNCERTAIN: r31 at the call was taken from a linear backward scan | `ftCo_8008DA4C` returns `false` in the else branch (ftCo_Damage.rs:268): does not call `ftCo_800C0408`. No comment. Likely divergence on 0-percent hits (colanim state) |
| `ftCo_LandingAir_EnterWithLag` | f1 | `lag` unset only when msid stays None, then it is not used: correlated path, infeasible | comment at the site; benign |
| `ftAnim_8006F3DC` | f1 | no return: returns the caller's f1 | reproduces it (`AOBJ_STALE_F1`, per comment) |
| `ftMs_SpecialS_80137940` and 5 siblings | r30 | `msid` unset for motions other than the two cases; called only from those motions (not checked exhaustively) | no comment; Rust local starts at 0 (inference) |
| `ftGw_SpecialS_GetRandomInt` | r30 | `result` unset if no roll bucket matches; `HSD_Randi(total)` < last cumulative bucket, so infeasible unless both excluded judges cover every weight (not checked) | no comment |
| `ft_80084A80` | r25, r31 | switch on `arg0` 0..2 without default: callers pass 0..2 (not checked) | no comment |
| `ftColl_8007A06C` (hit resolution) | r23, f22, r18, r22, f24 | `best_idx`/`kb` and friends set only inside the switch for entry kinds 1-3 and only when a candidate wins | no comment; feasibility needs a damage log entry of another kind |
| `it_802E7654`, `itLinkArrow_802A81C4`, `itLinkArrow_802A850C`, `it_8027BA54` | r3 | missing return | benign: no DOL caller reads r3 |

Port status of every hit is in the `port:` column of the list below. Comments found: 9 say the port uses 0, 3
say it reproduces the console value, 1 points elsewhere; 310 observable candidates in these areas have no comment
at the port site (most likely 0 there, not checked).

### Every hit in the online-relevant areas

Columns: function address, function, register (or `stack`), origin class, first read address, sinks reached,
verdict (`OBSERVABLE?` = reaches a sink; feasibility not proven), port status.

```
[fighter] 89
  80035EE8 Player_SetFlagsAEBit1              r3     missing return value                     read@80035F68 sinks=return               benign(result unused)  port: no comment (not checked)
  80035EE8 Player_SetFlagsAEBit1              r3     missing return value (no return statemen read@80035F68 sinks=return               benign(result unused)  port: no comment (not checked)
  80037B2C pl_80037B2C                        r3     missing return value                     read@80037BBC sinks=return               OBSERVABLE?            port: no comment (not checked)
  8006F3DC ftAnim_8006F3DC                    f1     missing return value                     read@8006F480 sinks=return               OBSERVABLE?            port: console value (per comment)
  8006F994 ftAnim_8006F994                    r3     missing return value                     read@8006FA54 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80070CC4 ftAnim_80070CC4                    r31    callee-saved caller value                read@80070E58 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80071FC8 ftAction_80071FC8                  r27    callee-saved caller value                read@80072214 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80072320 ftAction_80072320                  f31    callee-saved caller value                read@8007240C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80075240 ftParts_80075240                   r3     missing return value                     read@80075300 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80078C70 ftColl_80078C70                    r25    callee-saved caller value                read@80078F58 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8007925C ftColl_8007925C                    r25    callee-saved caller value                read@80079664 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8007925C ftColl_8007925C                    stack  stack slot read before any store         read@80079860 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8007A06C ftColl_8007A06C                    f22    callee-saved caller value                read@8007A6A8 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  8007A06C ftColl_8007A06C                    r23    callee-saved caller value                read@8007A71C sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  8007A06C ftColl_8007A06C                    f24    callee-saved caller value                read@8007A928 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8007A06C ftColl_8007A06C                    r18    callee-saved caller value                read@8007A960 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8007A06C ftColl_8007A06C                    r22    callee-saved caller value                read@8007A974 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80084A80 ft_80084A80                        r25    callee-saved caller value                read@80084B24 sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  80084A80 ft_80084A80                        r31    callee-saved caller value                read@80084BC4 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  8008A7A8 ftCo_8008A7A8                      r26    callee-saved caller value                read@8008A914 sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  8008D5FC ftCo_LandingAir_EnterWithLag       f1     argument register beyond the prototype   read@8008D6A4 sinks=call-arg             OBSERVABLE?            port: see comment
  8008E9D0 ftCo_8008E9D0                      r31    callee-saved caller value                read@8008EB24 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8009DD94 ftCo_8009DD94                      f31    callee-saved caller value                read@8009DFC8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800A4A40 ftCo_800A4A40                      f31    callee-saved caller value                read@800A4BAC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A4BEC ftCo_800A4BEC                      f31    callee-saved caller value                read@800A4DF4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A4E8C ftCo_800A4E8C                      f31    callee-saved caller value                read@800A5094 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A50D4 ftCo_800A50D4                      f31    callee-saved caller value                read@800A5254 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A53DC ftCo_800A53DC                      r27    callee-saved caller value                read@800A5628 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A53DC ftCo_800A53DC                      f31    callee-saved caller value                read@800A5644 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A5CE0 ftCo_800A5CE0                      f31    callee-saved caller value                read@800A5F0C sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A5F4C ftCo_800A5F4C                      f28    callee-saved caller value                read@800A618C sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A61D8 ftCo_800A61D8                      f28    callee-saved caller value                read@800A6440 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A648C ftCo_800A648C                      f28    callee-saved caller value                read@800A66B4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800A8940 ftCo_800A8940                      r31    callee-saved caller value                read@800A8C1C sinks=call-arg,load-addr   OBSERVABLE?            port: no comment (not checked)
  800AC5A0 ftCo_800AC5A0                      r5     argument register beyond the prototype   read@800AC74C sinks=call-arg             OBSERVABLE?            port: console value (per comment)
  800AC5A0 ftCo_800AC5A0                      r30    callee-saved caller value                read@800AC754 sinks=call-arg             OBSERVABLE?            port: console value (per comment)
  800ADE48 ftCo_800ADE48                      r31    callee-saved caller value                read@800AE270 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800B4AB0 ftCo_800B4AB0                      r3     missing return value                     read@800B52A8 sinks=return               OBSERVABLE?            port: no comment (not checked)
  800B52AC ftCo_800B52AC                      r3     missing return value                     read@800B5AAC sinks=return               OBSERVABLE?            port: no comment (not checked)
  800B5AB0 ftCo_800B5AB0                      r3     missing return value                     read@800B6204 sinks=return               OBSERVABLE?            port: no comment (not checked)
  800B6208 ftCo_800B6208                      r3     missing return value                     read@800B6308 sinks=return               OBSERVABLE?            port: no comment (not checked)
  800BEC08 ftDemo_GetMotionFileString         r3     missing return value                     read@800BEC70 sinks=return               OBSERVABLE?            port: no Rust function
  800C2600 ftCo_800C2600                      r25    callee-saved caller value                read@800C2940 sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  800C2600 ftCo_800C2600                      r30    callee-saved caller value                read@800C2964 sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  800C2FD8 ftCo_800C2FD8                      r5     argument register beyond the prototype   read@800C30D4 sinks=call-arg,store,load-addr OBSERVABLE?            port: no comment (not checked)
  800C2FD8 ftCo_800C2FD8                      r29    callee-saved caller value                read@800C3108 sinks=store,load-addr      OBSERVABLE?            port: no comment (not checked)
  800CCEC4 fn_800CCEC4                        r3     missing return value                     read@800CCF54 sinks=return               benign(result unused)  port: no comment (not checked)
  800CCF58 ftCo_Attack_800CCF58               r31    callee-saved caller value                read@800CD03C sinks=call-arg,branch,load-addr OBSERVABLE?            port: no comment (not checked)
  800CCF58 ftCo_Attack_800CCF58               r27    callee-saved caller value                read@800CD09C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC28 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC30 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC3C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC58 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC5C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCC68 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCD38 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCD3C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DC920 ftCo_800DC920                      stack  stack slot read before any store         read@800DCD48 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800DFDDC ftWalkCommon_800DFDDC              f31    callee-saved caller value                read@800DFEA4 sinks=call-arg             OBSERVABLE?            port: 0 (per comment)
  800E0CE0 ftMr_Init_GetMotionFileString      r0     scratch register unset at entry          read@800E0D0C sinks=return,load-addr     OBSERVABLE?            port: no comment (not checked)
  800E3EAC ftCa_SpecialHi_800E3EAC            r29    callee-saved caller value                read@800E3F7C sinks=call-arg,load-addr   OBSERVABLE?            port: 0 (per comment)
  800EEBF8 ftKb_Init_GetMotionFileString      r0     scratch register unset at entry          read@800EEC24 sinks=return,load-addr     OBSERVABLE?            port: no comment (not checked)
  800FA254 ftKb_PkSpecialAirN_Anim            f31    callee-saved caller value                read@800FA3C8 sinks=call-arg,branch      OBSERVABLE?            port: 0 (per comment)
  800FB500 ftKb_SpecialNLk800FB500            r29    callee-saved caller value                read@800FB58C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FB5F4 ftKb_SpecialNLk800FB5F4            r29    callee-saved caller value                read@800FB66C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FB6DC ftKb_SpecialNLk800FB6DC            f31    callee-saved caller value                read@800FB788 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FB880 ftKb_SpecialNLk800FB880            f31    callee-saved caller value                read@800FB968 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FBA00 ftKb_SpecialNLk800FBA00            f31    callee-saved caller value                read@800FBAD8 sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  800FBBC4 ftKb_LkSpecialNStart_Anim          f1     argument register beyond the prototype   read@800FBC48 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FBF34 ftKb_LkSpecialAirNStart_Anim       f31    callee-saved caller value                read@800FC14C sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  800FC274 ftKb_LkSpecialAirNEnd_Anim         f31    callee-saved caller value                read@800FC324 sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  800FC374 ftKb_LkSpecialNStart_IASA          f2     argument register beyond the prototype   read@800FC3DC sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  800FC540 ftKb_LkSpecialAirNStart_IASA       f2     argument register beyond the prototype   read@800FC5A8 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  800FE100 ftKb_SpecialNFx_800FE100           r28    callee-saved caller value                read@800FE1CC sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FE240 ftKb_SpecialNFx_800FE240           r28    callee-saved caller value                read@800FE2EC sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  800FE928 ftKb_FxSpecialAirNEnd_Anim         f31    callee-saved caller value                read@800FEA00 sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  80117F24 ftNs_SpecialAirHi_CollisionModVel  f1     argument register beyond the prototype   read@801180F0 sinks=call-arg             OBSERVABLE?            port: 0 (per comment)
  80137940 ftMs_SpecialS_80137940             r30    callee-saved caller value                read@801379A4 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  801379D0 ftMs_SpecialS_801379D0             r30    callee-saved caller value                read@80137A3C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80137CBC ftMs_SpecialS_80137CBC             r30    callee-saved caller value                read@80137D34 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80137D60 ftMs_SpecialS_80137D60             r30    callee-saved caller value                read@80137DE0 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80137FF8 ftMs_SpecialS_80137FF8             r30    callee-saved caller value                read@80138070 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8013809C ftMs_SpecialS_8013809C             r30    callee-saved caller value                read@8013811C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80142640 ftLg_Init_GetMotionFileString      r0     scratch register unset at entry          read@8014266C sinks=return,load-addr     OBSERVABLE?            port: no comment (not checked)
  8014C6B4 ftGw_SpecialS_GetRandomInt         r30    callee-saved caller value                read@8014C778 sinks=return,store         OBSERVABLE?            port: no comment (not checked)
  8014F9A4 ftGk_Init_GetMotionFileString      r0     scratch register unset at entry          read@8014F9BC sinks=return,load-addr     OBSERVABLE?            port: no comment (not checked)
  80031AD0 Player_80031AD0                    stack  stack slot read before any store         read@80031C3C dead                       benign(dead)           port: no comment (not checked)
  8007A06C ftColl_8007A06C                    f25    callee-saved caller value                read@8007A924 dead                       benign(dead)           port: no comment (not checked)
  800DFEC8 ftWalkCommon_800DFEC8              f31    callee-saved caller value                read@800E0018 dead                       benign(dead)           port: no comment (not checked)

[items] 48
  8026C75C it_8026C75C                        r30    callee-saved caller value                read@8026C850 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802706D0 it_802706D0                        r28    callee-saved caller value                read@80270910 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80270E30 it_80270E30                        r26    callee-saved caller value                read@802710E8 sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  8027770C it_8027770C                        stack  stack slot read before any store         read@802777A0 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8027770C it_8027770C                        stack  stack slot read before any store         read@802777AC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8027770C it_8027770C                        stack  stack slot read before any store         read@802777B4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8027BA54 it_8027BA54                        r3     missing return value                     read@8027BB18 sinks=return               benign(result unused)  port: no comment (not checked)
  80289BE8 it_80289BE8                        r29    callee-saved caller value                read@80289F18 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8028EB88 it_8028EB88                        r5     argument register beyond the prototype   read@8028EBB8 sinks=branch,load-addr     OBSERVABLE?            port: 0 (per comment)
  80291DAC it_80291DAC                        r6     argument register beyond the prototype   read@80291ED8 sinks=branch               OBSERVABLE?            port: 0 (per comment)
  802A2568 it_802A2568                        r20    callee-saved caller value                read@802A2AEC sinks=store                OBSERVABLE?            port: no comment (not checked)
  802A2568 it_802A2568                        r21    callee-saved caller value                read@802A2AF0 sinks=return               OBSERVABLE?            port: no comment (not checked)
  802A2568 it_802A2568                        r19    callee-saved caller value                read@802A2AF4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802A81C4 itLinkArrow_802A81C4               r3     missing return value                     read@802A832C sinks=return               benign(result unused)  port: no comment (not checked)
  802A850C itLinkArrow_802A850C               r30    callee-saved caller value                read@802A8920 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802A850C itLinkArrow_802A850C               r29    callee-saved caller value                read@802A8940 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802A850C itLinkArrow_802A850C               r3     missing return value                     read@802A8980 sinks=return               benign(result unused)  port: no comment (not checked)
  802AAB70 itNesspkflash_UnkMotion0_Anim      r28    callee-saved caller value                read@802AAC2C sinks=branch               OBSERVABLE?            port: no comment (not checked)
  802AAFDC itNesspkflash_UnkMotion0_Phys      r4     argument register beyond the prototype   read@802AB050 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  802AF32C it_802AF32C                        r4     argument register beyond the prototype   read@802AF3AC sinks=call-arg,branch,load-addr OBSERVABLE?            port: no comment (not checked)
  802AF434 itLinkBow_Logic100_PickedUp        r4     argument register beyond the prototype   read@802AF4B0 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  802B18B0 it_802B18B0                        r30    callee-saved caller value                read@802B1948 sinks=branch               OBSERVABLE?            port: 0 (per comment)
  802B75FC it_802B75FC                        r18    callee-saved caller value                read@802B7B64 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802B75FC it_802B75FC                        r19    callee-saved caller value                read@802B7B68 sinks=return               OBSERVABLE?            port: no comment (not checked)
  802B75FC it_802B75FC                        r17    callee-saved caller value                read@802B7B6C sinks=store                OBSERVABLE?            port: no comment (not checked)
  802BAF2C it_802BAF2C                        r20    callee-saved caller value                read@802BB1EC sinks=store                OBSERVABLE?            port: no comment (not checked)
  802BAF2C it_802BAF2C                        r21    callee-saved caller value                read@802BB1F0 sinks=return               OBSERVABLE?            port: no comment (not checked)
  802BAF2C it_802BAF2C                        r19    callee-saved caller value                read@802BB1F4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802BC080 it_802BC080                        r25    callee-saved caller value                read@802BC924 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802BE65C it_802BE65C                        r18    callee-saved caller value                read@802BE6E8 sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  802BE65C it_802BE65C                        r20    callee-saved caller value                read@802BE934 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802BE65C it_802BE65C                        r21    callee-saved caller value                read@802BE938 sinks=return               OBSERVABLE?            port: no comment (not checked)
  802BE65C it_802BE65C                        r19    callee-saved caller value                read@802BE93C sinks=store                OBSERVABLE?            port: no comment (not checked)
  802C248C it_802C248C                        r18    callee-saved caller value                read@802C2510 sinks=call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  802C248C it_802C248C                        r20    callee-saved caller value                read@802C2730 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802C248C it_802C248C                        r21    callee-saved caller value                read@802C2734 sinks=return               OBSERVABLE?            port: no comment (not checked)
  802C248C it_802C248C                        r19    callee-saved caller value                read@802C2738 sinks=store                OBSERVABLE?            port: no comment (not checked)
  802C4D10 it_802C4D10                        r3     missing return value                     read@802C4F4C sinks=return               OBSERVABLE?            port: no comment (not checked)
  802CF544 itEntei_UnkMotion0_Anim            r29    callee-saved caller value                read@802CF608 sinks=call-arg             OBSERVABLE?            port: 0 (per comment)
  802CF978 itRaikou_UnkMotion0_Anim           r29    callee-saved caller value                read@802CFA3C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  802CFDAC itSuikun_UnkMotion0_Anim           r29    callee-saved caller value                read@802CFE70 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  802D1F64 it_802D1F64                        r4     argument register beyond the prototype   read@802D1F84 sinks=return,call-arg,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  802E609C it_802E609C                        r30    callee-saved caller value                read@802E6134 sinks=return               OBSERVABLE?            port: 0 (per comment)
  802E7654 it_802E7654                        r3     missing return value                     read@802E79C4 sinks=return               benign(result unused)  port: no comment (not checked)
  802B1DF8 it_802B1DF8                        stack  stack slot read before any store         read@802B1EA4 dead                       benign(dead)           port: no comment (not checked)
  802E6AEC it_802E6AEC                        stack  stack slot read before any store         read@802E6BB8 dead                       benign(dead)           port: no comment (not checked)
  802EE200 it_802EE200                        stack  stack slot read before any store         read@802EE2A0 dead                       benign(dead)           port: no comment (not checked)
  802F0AE0 it_802F0AE0                        stack  stack slot read before any store         read@802F0B60 dead                       benign(dead)           port: no comment (not checked)

[stage/coll/cam/fx] 127
  8002958C Camera_8002958C                    f31    callee-saved caller value                read@80029A58 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002958C Camera_8002958C                    f30    callee-saved caller value                read@80029A68 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002958C Camera_8002958C                    f29    callee-saved caller value                read@80029A70 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002958C Camera_8002958C                    f28    callee-saved caller value                read@80029A74 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002DFE4 Camera_8002DFE4                    f31    callee-saved caller value                read@8002E0F8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002E158 Camera_8002E158                    f4     argument register beyond the prototype   read@8002E220 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002E234 Camera_8002E234                    stack  stack slot read before any store         read@8002E3FC sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8002E234 Camera_8002E234                    stack  stack slot read before any store         read@8002E408 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8002E234 Camera_8002E234                    stack  stack slot read before any store         read@8002E418 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8002F4D4 Camera_SetUpPauseCamera            f31    callee-saved caller value                read@8002F6A4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80030758 Camera_GetBackgroundColor          stack  stack slot read before any store         read@8003077C sinks=return               OBSERVABLE?            port: no comment (not checked)
  80042D24 mpColl_LoadECB                     f31    callee-saved caller value                read@80042D84 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80042D24 mpColl_LoadECB                     f30    callee-saved caller value                read@80042D88 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80043754 mpColl_80043754                    r25    callee-saved caller value                read@800439DC sinks=return               OBSERVABLE?            port: no comment (not checked)
  80044E10 mpColl_80044E10_RightWall          f29    callee-saved caller value                read@800453AC sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80044E10 mpColl_80044E10_RightWall          f28    callee-saved caller value                read@800453B4 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80045B74 mpColl_80045B74_LeftWall           f29    callee-saved caller value                read@80046128 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80045B74 mpColl_80045B74_LeftWall           f28    callee-saved caller value                read@80046130 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80046F78 mpColl_80046F78                    r3     missing return value                     read@800471F4 sinks=return               benign(result unused)  port: no comment (not checked)
  800471F8 mpColl_800471F8                    f30    callee-saved caller value                read@80047260 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800471F8 mpColl_800471F8                    f31    callee-saved caller value                read@80047264 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800473CC mpColl_800473CC                    f30    callee-saved caller value                read@80047434 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800473CC mpColl_800473CC                    f31    callee-saved caller value                read@80047438 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800474E0 mpColl_800474E0                    f30    callee-saved caller value                read@80047548 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800474E0 mpColl_800474E0                    f31    callee-saved caller value                read@8004754C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800476B4 mpColl_800476B4                    f30    callee-saved caller value                read@80047728 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800476B4 mpColl_800476B4                    f31    callee-saved caller value                read@8004772C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800477E0 mpColl_800477E0                    f30    callee-saved caller value                read@80047848 sinks=store                OBSERVABLE?            port: no comment (not checked)
  800477E0 mpColl_800477E0                    f31    callee-saved caller value                read@8004784C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800478F4 mpColl_800478F4                    f30    callee-saved caller value                read@8004795C sinks=store                OBSERVABLE?            port: no comment (not checked)
  800478F4 mpColl_800478F4                    f31    callee-saved caller value                read@80047960 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047AC8 mpColl_80047AC8                    f30    callee-saved caller value                read@80047B3C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047AC8 mpColl_80047AC8                    f31    callee-saved caller value                read@80047B40 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047BF4 mpColl_80047BF4                    f30    callee-saved caller value                read@80047C68 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047BF4 mpColl_80047BF4                    f31    callee-saved caller value                read@80047C6C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047E14 mpColl_80047E14                    f30    callee-saved caller value                read@80047E88 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047E14 mpColl_80047E14                    f31    callee-saved caller value                read@80047E8C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047F40 mpColl_80047F40                    f30    callee-saved caller value                read@80047FB4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80047F40 mpColl_80047F40                    f31    callee-saved caller value                read@80047FB8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048160 mpColl_80048160                    f30    callee-saved caller value                read@800481C8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048160 mpColl_80048160                    f31    callee-saved caller value                read@800481CC sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048274 mpColl_80048274                    f30    callee-saved caller value                read@800482DC sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048274 mpColl_80048274                    f31    callee-saved caller value                read@800482E0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048464 mpColl_80048464                    f30    callee-saved caller value                read@800484CC sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048464 mpColl_80048464                    f31    callee-saved caller value                read@800484D0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048654 mpColl_80048654                    f30    callee-saved caller value                read@800486BC sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048654 mpColl_80048654                    f31    callee-saved caller value                read@800486C0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80048AB0 mpColl_80048AB0_RightWall          f29    callee-saved caller value                read@800490D0 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80048AB0 mpColl_80048AB0_RightWall          f28    callee-saved caller value                read@800490D8 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80049778 mpColl_80049778_LeftWall           f29    callee-saved caller value                read@80049DB0 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80049778 mpColl_80049778_LeftWall           f28    callee-saved caller value                read@80049DB8 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8004A45C mpColl_8004A45C_Floor              r31    callee-saved caller value                read@8004A62C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004A678 mpColl_8004A678_Floor              r30    callee-saved caller value                read@8004A8B0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004ACE4 mpColl_8004ACE4                    f31    callee-saved caller value                read@8004AE38 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8004ACE4 mpColl_8004ACE4                    f30    callee-saved caller value                read@8004AE40 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8004ACE4 mpColl_8004ACE4                    f29    callee-saved caller value                read@8004B028 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8004ACE4 mpColl_8004ACE4                    f28    callee-saved caller value                read@8004B030 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8004B108 mpColl_8004B108                    f30    callee-saved caller value                read@8004B170 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B108 mpColl_8004B108                    f31    callee-saved caller value                read@8004B174 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B2DC mpColl_8004B2DC                    f30    callee-saved caller value                read@8004B344 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B2DC mpColl_8004B2DC                    f31    callee-saved caller value                read@8004B348 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B4B0 mpColl_8004B4B0                    f30    callee-saved caller value                read@8004B518 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B4B0 mpColl_8004B4B0                    f31    callee-saved caller value                read@8004B51C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B5C4 mpColl_8004B5C4                    f30    callee-saved caller value                read@8004B62C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004B5C4 mpColl_8004B5C4                    f31    callee-saved caller value                read@8004B630 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004C328 mpColl_8004C328_Ceiling            r31    callee-saved caller value                read@8004C4E8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004C750 mpColl_8004C750                    f30    callee-saved caller value                read@8004C7B8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004C750 mpColl_8004C750                    f31    callee-saved caller value                read@8004C7BC sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004D288 mpLibLoad                          r26    callee-saved caller value                read@8004D44C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8004ED5C mpLib_8004ED5C                     f4     argument register beyond the prototype   read@8004EFC4 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  80051BA8 mpLib_80051BA8_Floor               f31    callee-saved caller value                read@80051DC4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80051BA8 mpLib_80051BA8_Floor               f30    callee-saved caller value                read@80051E68 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  80051BA8 mpLib_80051BA8_Floor               f29    callee-saved caller value                read@80051E88 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@80052458 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@8005245C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@80052468 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    r30    callee-saved caller value                read@80052478 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    r29    callee-saved caller value                read@80052484 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@80052490 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@80052494 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80051EC8 mpCheckMultiple                    stack  stack slot read before any store         read@800524A0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80055C5C mpJointUpdateDynamics              r27    callee-saved caller value                read@80055D90 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  80056C54 mpLib_80056C54                     f27    callee-saved caller value                read@80056ED0 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8005BC50 efLib_Update                       stack  stack slot read before any store         read@8005BDBC sinks=store                OBSERVABLE?            port: no comment (not checked)
  8005BC50 efLib_Update                       stack  stack slot read before any store         read@8005BDE4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8005DBC0 efLib_Cb_ParticleRender            r3     missing return value (no return statemen read@8005DCCC sinks=return               benign(result unused)  port: no comment (not checked)
  8005FDDC efSync_Spawn                       r25    callee-saved caller value                read@80060BD0 sinks=store,load-addr      OBSERVABLE?            port: no comment (not checked)
  80063930 efAsync_Dispatch                   r26    callee-saved caller value                read@800657A8 sinks=store,load-addr      OBSERVABLE?            port: no comment (not checked)
  801C20E0 Ground_801C20E0                    r31    callee-saved caller value                read@801C21B4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801C20E0 Ground_801C20E0                    r30    callee-saved caller value                read@801C21BC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801C20E0 Ground_801C20E0                    r29    callee-saved caller value                read@801C21C4 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801C62B4 grDatFiles_801C62B4                r3     missing return value                     read@801C6320 sinks=return               OBSERVABLE?            port: no comment (not checked)
  801C6478 grDatFiles_801C6478                r30    callee-saved caller value                read@801C6504 sinks=return,branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  801C9A70 grLib_801C9A70                     r31    callee-saved caller value                read@801C9AE0 sinks=store,load-addr      OBSERVABLE?            port: no comment (not checked)
  801D3BBC grStadium_801D3BBC                 r4     caller-saved garbage after a call        read@801D3E14 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  801D4548 grStadium_801D4548                 r29    callee-saved caller value                read@801D4734 sinks=call-arg,load-addr   OBSERVABLE?            port: no comment (not checked)
  801D577C grKongo_801D577C                   f30    callee-saved caller value                read@801D5838 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801D7BBC grKongo_801D7BBC                   r29    callee-saved caller value                read@801D7E38 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  801D8314 grKongo_801D8314                   f31    callee-saved caller value                read@801D842C sinks=return               OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   r31    callee-saved caller value                read@801DBCF0 sinks=call-arg,load-addr   OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   f28    callee-saved caller value                read@801DBE7C sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   f30    callee-saved caller value                read@801DBE7C sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   f29    callee-saved caller value                read@801DBE88 sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   f31    callee-saved caller value                read@801DBE88 sinks=call-arg,branch      OBSERVABLE?            port: no comment (not checked)
  801DBB60 grZebes_801DBB60                   r3     missing return value                     read@801DC25C sinks=return               benign(result unused)  port: no comment (not checked)
  801E03C8 grCorneria_801E03C8                r3     missing return value                     read@801E0674 sinks=return               OBSERVABLE?            port: no comment (not checked)
  801E8560 fn_801E8560                        r29    callee-saved caller value                read@801E85C8 sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  801EB004 grBigBlue_801EB004                 r26    callee-saved caller value                read@801EB204 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801EB004 grBigBlue_801EB004                 r25    callee-saved caller value                read@801EB21C sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801F2AB0 grMuteCity_801F2AB0                r3     missing return value                     read@801F2B54 sinks=return               OBSERVABLE?            port: no comment (not checked)
  801F62F8 grGreatBay_801F62F8                r31    callee-saved caller value                read@801F63B8 sinks=return,branch        OBSERVABLE?            port: no comment (not checked)
  801F686C grIceMt_801F686C                   r30    callee-saved caller value                read@801F6A7C sinks=call-arg,branch,load-addr OBSERVABLE?            port: no comment (not checked)
  801F686C grIceMt_801F686C                   r29    callee-saved caller value                read@801F6BF8 sinks=call-arg,branch,load-addr OBSERVABLE?            port: no comment (not checked)
  801F686C grIceMt_801F686C                   r28    callee-saved caller value                read@801F6DDC sinks=call-arg,load-addr   OBSERVABLE?            port: no comment (not checked)
  801F7A2C stageGObj10_GObjProc               r30    callee-saved caller value                read@801F7CEC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  801F96E0 grIceMt_801F96E0                   f1     missing return value                     read@801F98A4 sinks=return               benign(result unused)  port: no comment (not checked)
  8020C530 grBigBlueRoute_8020C530            r3     missing return value                     read@8020C858 sinks=return               OBSERVABLE?            port: no comment (not checked)
  8020F888 stageGObj1_GObjProc                f30    callee-saved caller value                read@8020F944 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80210650 grOldKongo_80210650                f31    callee-saved caller value                read@80210768 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80215358 grGreens_80215358                  r21    callee-saved caller value                read@80215638 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80215B84 fn_80215B84                        r29    callee-saved caller value                read@80215D24 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80219230 grPushOn_80219230                  r3     missing return value                     read@802192A0 sinks=return               OBSERVABLE?            port: no comment (not checked)
  8021E500 grHomeRun_8021E500                 r28    callee-saved caller value                read@8021E5F4 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80224E64 Stage_80224E64                     stack  stack slot read before any store         read@80224F7C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80224E64 Stage_80224E64                     stack  stack slot read before any store         read@80224F80 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80224E64 Stage_80224E64                     stack  stack slot read before any store         read@80224F8C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80225074 Stage_80225074                     r31    callee-saved caller value                read@8022514C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)

[lb] 38
  8000AFBC lbTime_GetTimeInSeconds            r31    callee-saved caller value                read@8000AFFC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8000D8F4 lbVector_RotateAboutUnitAxis       f3     argument register beyond the prototype   read@8000DAC8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8000D8F4 lbVector_RotateAboutUnitAxis       f2     argument register beyond the prototype   read@8000DAD4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8000DB00 lbVector_Rotate                    f0     scratch register unset at entry          read@8000DC5C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8000DB00 lbVector_Rotate                    f2     argument register beyond the prototype   read@8000DC60 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8000DB00 lbVector_Rotate                    f3     argument register beyond the prototype   read@8000DC64 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8000FD48 lb_8000FD48                        r31    callee-saved caller value                read@8000FE00 sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  80011AC4 lb_80011AC4                        r29    callee-saved caller value                read@80011B50 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80011E24 lb_80011E24                        r27    callee-saved caller value                read@80011EF8 sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  800138EC lb_800138EC                        r3     missing return value                     read@80013B10 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80014FC8 lbMemory_80014FC8                  r31    callee-saved caller value                read@800150AC sinks=store                OBSERVABLE?            port: no comment (not checked)
  8001E560 lbAnim_InitFrames                  r29    callee-saved caller value                read@8001E5F4 sinks=return               OBSERVABLE?            port: no comment (not checked)
  800251EC fn_800251EC                        stack  stack slot read before any store         read@800252D0 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  800122F0 lb_800122F0                        stack  stack slot read before any store         read@800124D8 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012A3C dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012AC8 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012B58 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012BE8 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012C78 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012D08 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012D94 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012E20 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012EAC dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012F38 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80012FC8 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013058 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@800130E4 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013170 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@800131FC dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013288 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013318 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@800133A4 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013430 dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@800134BC dead                       benign(dead)           port: no comment (not checked)
  80012994 lb_80012994                        stack  stack slot read before any store         read@80013544 dead                       benign(dead)           port: no comment (not checked)
  80013614 fn_80013614                        stack  stack slot read before any store         read@800137F0 dead                       benign(dead)           port: no comment (not checked)
  8001A184 taskMount                          r26    callee-saved caller value                read@8001A374 dead                       benign(dead)           port: no comment (not checked)
  8001A8A4 taskFormat                         r28    callee-saved caller value                read@8001A99C dead                       benign(dead)           port: no comment (not checked)

[HSD] 67
  8035F7D0 MakeColorGenTExp                   r14    callee-saved caller value                read@8035FC0C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8035F7D0 MakeColorGenTExp                   r18    callee-saved caller value                read@8035FC1C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8035F7D0 MakeColorGenTExp                   r19    callee-saved caller value                read@8035FC2C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8035F7D0 MakeColorGenTExp                   r21    callee-saved caller value                read@8035FC3C sinks=store                OBSERVABLE?            port: no comment (not checked)
  8035F7D0 MakeColorGenTExp                   r22    callee-saved caller value                read@8035FC4C sinks=store                OBSERVABLE?            port: no comment (not checked)
  80366A78 HSD_LightID2Index                  r31    callee-saved caller value                read@80366B4C sinks=return               OBSERVABLE?            port: no Rust function
  803676F8 HSD_CObjEraseScreen                f28    callee-saved caller value                read@8036781C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  803676F8 HSD_CObjEraseScreen                f27    callee-saved caller value                read@80367824 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  803676F8 HSD_CObjEraseScreen                f30    callee-saved caller value                read@8036782C sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  803676F8 HSD_CObjEraseScreen                f29    callee-saved caller value                read@80367830 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80367B68 makeProjectionMtx                  r31    callee-saved caller value                read@80367C10 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80367C28 setupNormalCamera                  r31    callee-saved caller value                read@80367E74 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  80367EB0 setupTopHalfCamera                 r27    callee-saved caller value                read@803680CC sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  803680F8 setupBottomHalfCamera              r31    callee-saved caller value                read@80368378 sinks=call-arg             OBSERVABLE?            port: no comment (not checked)
  8036D7E4 drawShapeAnim                      r24    callee-saved caller value                read@8036DA64 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8037231C resolveIKJoint1                    f27    callee-saved caller value                read@80372958 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  8037231C resolveIKJoint1                    f28    callee-saved caller value                read@80372958 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80378F38 splArcLengthGetParameter           f29    callee-saved caller value                read@80379264 sinks=return               OBSERVABLE?            port: no comment (not checked)
  803805DC HSD_ByteCodeEval                   r30    callee-saved caller value                read@80380660 sinks=call-arg,branch,load-addr OBSERVABLE?            port: no comment (not checked)
  803805DC HSD_ByteCodeEval                   r28    callee-saved caller value                read@80380670 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80381FA8 hsdAllocMemPiece                   r30    callee-saved caller value                read@80382118 sinks=store,load-addr      OBSERVABLE?            port: no comment (not checked)
  80383488 HSD_TExpColorInSub                 r27    callee-saved caller value                read@80383618 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  8038EB50 HSD_DevComStdCallback              r31    callee-saved caller value                read@8038EBAC sinks=store                OBSERVABLE?            port: no comment (not checked)
  8038EBCC HSD_DevComARAMCallback             r29    callee-saved caller value                read@8038ECB0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8038F098 HSD_DevComDVDStdCallback           r31    callee-saved caller value                read@8038F0F0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  8038F110 HSD_DevComDVDARAMEndCallback       r30    callee-saved caller value                read@8038F174 sinks=store                OBSERVABLE?            port: no comment (not checked)
  80397E0C baselib_mfspr                      ca     scratch register unset at entry          read@80398098 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80397E0C baselib_mfspr                      ctr    scratch register unset at entry          read@803980A8 sinks=return               OBSERVABLE?            port: no comment (not checked)
  80398614 psInitDataBankLocate               r0     scratch register unset at entry          read@8039875C sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  80398614 psInitDataBankLocate               r7     argument register beyond the prototype   read@8039875C sinks=branch               OBSERVABLE?            port: no comment (not checked)
  80398614 psInitDataBankLocate               r8     argument register beyond the prototype   read@8039876C sinks=branch,store,load-addr OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    r19    callee-saved caller value                read@803A0300 sinks=branch               OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A1648 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A1650 sinks=call-arg,branch,store OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A165C sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A1664 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A16A8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A16B4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A16B8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A16BC sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A32D8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A32FC sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3304 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3348 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3360 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3364 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3368 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3398 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A33B0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A33B4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A33B8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A33F8 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3410 sinks=call-arg,branch,store OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A341C sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A3424 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A35CC sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A35D0 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A35D4 sinks=store                OBSERVABLE?            port: no comment (not checked)
  803A8134 HSD_SisLib_803A8134                r29    callee-saved caller value                read@803A83F0 sinks=load-addr            OBSERVABLE?            port: no comment (not checked)
  803A84BC HSD_SisLib_803A84BC                f17    callee-saved caller value                read@803A912C sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  803A84BC HSD_SisLib_803A84BC                f19    callee-saved caller value                read@803A9134 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  803A84BC HSD_SisLib_803A84BC                f15    callee-saved caller value                read@803A913C sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  803A84BC HSD_SisLib_803A84BC                f16    callee-saved caller value                read@803A9144 sinks=branch,store         OBSERVABLE?            port: no comment (not checked)
  803AD16C fn_803AD16C                        stack  stack slot read before any store         read@803AD6EC sinks=branch               OBSERVABLE?            port: no comment (not checked)
  803B6BE4 hsd_803B6BE4                       r29    callee-saved caller value                read@803B71F0 sinks=return,call-arg,branch OBSERVABLE?            port: no comment (not checked)
  803B6BE4 hsd_803B6BE4                       r28    callee-saved caller value                read@803B71F4 sinks=return,call-arg,branch OBSERVABLE?            port: no comment (not checked)
  803A0088 psDispParticles                    stack  stack slot read before any store         read@803A32F0 dead                       benign(dead)           port: no comment (not checked)
```

```
unresolved bctr (treated as an exit):
  80019CB0 executeNextTask                    80019D8C
  80039618 fn_80039618                        80039994
  8003C340 fn_8003C340                        8003C5C8
  800B3E04 ftCo_800B3E04                      800B3ED8
  801F04B8 grMuteCity_801F04B8                801F0574
  803606F0 HSD_TObjAssignResources            803607FC
  8036D4D0 interpretShapeAnimDisplayList      8036D57C 8036D57C
  8037BD84 resolveLimits                      8037BE54
  8038687C SimplifyThis                       80386AD0
  8038C6C0 AXDriverInterp                     8038C708 8038C760 8038C708
  8039930C hsd_8039930C                       8039977C
  8039DAD4 hsd_8039DAD4                       8039E1C8 8039E1C8
  803A8134 HSD_SisLib_803A8134                803A8200 803A8200
  803A84BC HSD_SisLib_803A84BC                803A8A54 803A8A54
  803AAA48 hsd_803AAA48                       803AAB40
  803B302C decryptByte                        803B3070
```

## Audit 2: fused multiply-add sites, retail vs port

### Method

- DOL: per function, count `fmadds/fmsubs/fnmadds/fnmsubs` and the double `fmadd/fmsub/fnmadd/fnmsub` (shown
  together as `dbl`, since the port has one double helper for all four sign forms). Paired-single fused ops are
  counted apart (none in the online-relevant areas).
- Rust: helper call sites in the port's function (`fmadds`, `fmsubs`/`fmsubs_neg`, `fnmsubs`, `fnmadds`;
  `.fmadd(`/`fmadd(`/`.mul_add(` as `dbl`). "fold" adds the Rust function's direct callees, up to 4 levels, when
  the DOL function has no `bl` to a symbol of that name: Rust-only helpers (no DOL symbol: `X__orig`, crate
  helpers) always, DOL-named callees when `#[inline]` or at most 12 lines. `mod_hook!`/`ct!` macro text is not a
  call. Calls resolve by module path (`crate::pure::my_sqrtf` -> pure.rs), then the caller's file, then its crate.
- A function counts as agreeing when the per-kind counts match directly or after folding, or when the totals
  match after adding lines a comment marks as fused that compute the form by hand in f64 (`manual-f64`, e.g.
  `ftwalkcommon_800dfec8`).
- Score (higher = more likely real): |DOL total - folded total|, doubled when the direct and folded counts are
  off in the same direction, +3 when no folding was involved, +2 when one side is zero; 0.5 when the totals match
  and only the sign forms differ (Rust `fmadds(-a, c, b)` for an `fnmsubs`: same value except the sign of an
  exact zero).

### Limits

- Per-function counts are a signal, not a proof: MWCC inlines static helpers and macros (`SQ`), and the port
  inlines or splits differently. Folding a big Rust-only helper can over-count; a Rust function that does the work
  of several DOL functions over-counts; a port that writes the fused form by hand without the helper and without a
  comment under-counts.
- The DOL counts include MWCC's inlined `sqrtf` (three `fnmsub` Newton steps in double), so `dbl 3` per inlined
  sqrtf is expected; the port's `sqrtf` helpers differ per crate.
- Same-name static functions in different TUs (`stageGObj2_GObjProc` x4...) are matched by TU file name; when the
  Rust file is named differently the largest same-named body is used.

### Totals

```
functions in scope 16049, with a Rust function of the same name 15809
DOL functions with scalar/double fused instructions: 720 (of which no Rust function of that name: 7)

DOL fused totals (scope): {'fmadd': 119, 'fmadds': 1970, 'fmsub': 13, 'fmsubs': 163, 'fnmadds': 2, 'fnmsub': 841, 'fnmsubs': 184}
  fighter              {'fmadd': 28, 'fmadds': 497, 'fmsubs': 18, 'fnmsub': 256, 'fnmsubs': 42}
  items                {'fmadd': 19, 'fmadds': 427, 'fmsubs': 23, 'fnmsub': 169, 'fnmsubs': 25}
  stage/coll/cam/fx    {'fmadd': 35, 'fmadds': 348, 'fmsub': 9, 'fmsubs': 33, 'fnmadds': 1, 'fnmsub': 88, 'fnmsubs': 49}
  lb                   {'fmadd': 8, 'fmadds': 266, 'fmsubs': 36, 'fnmadds': 1, 'fnmsub': 100, 'fnmsubs': 21}
  HSD                  {'fmadd': 29, 'fmadds': 432, 'fmsub': 4, 'fmsubs': 53, 'fnmsub': 228, 'fnmsubs': 47}
Rust helper call sites in those functions: {'fmadd': 113, 'fmadds': 1053, 'fmsubs': 37, 'fnmadds': 4, 'fnmsubs': 31, 'manual': 5};  with inlined callees folded: {'fmadd': 1108, 'fmadds': 2842, 'fmsubs': 60, 'fnmadds': 4, 'fnmsubs': 37, 'manual': 7}
functions with a Rust counterpart: counts agree 15380, agree after folding 15462, agree only by counting comment-marked hand-written f64 forms 3, disagree 230
```

### Hand-checked disagreements

| function | DOL | Rust | finding |
|---|---|---|---|
| `grCorneria_801E08CC`, `grCorneria_801E0A74` | fmadds 3, fnmsubs 3 each | 0 | real: the C `3200.0f * Ground_801C0498() / 2 - x` (and `+ x`) is compiled by MWCC as `fmuls` then `fnmsubs`/`fmadds` with the halving folded into the fused op (the `/2` became a multiply by a constant); the port computes `/ 2.0` then subtracts. Affects the camera-in-range tests on Corneria map object 8 (return value of the function) only near the bounds |
| `ftWalkCommon_800DFEC8` | fnmsubs 1 | manual f64 form with a `fused` comment | agrees (counted by the manual rule) |
| `HSD_MtxInverseConcat`, `HSD_MtxInverse`, `HSD_MtxInverseTranspose` | 68 / 20 / 14 fused | 9 / 7 / 5 after folding into `hsd_mtx` | the port's inverse is a different formulation (cofactor helpers); not checked bit for bit. HSD_MtxInverseConcat is used by JObj/collision matrix code |
| `ftYs_JumpAerial_Enter`, `ft_800CB6EC`, `ftYs_JumpAerial_Anim_Cb` | fnmsubs 1 each | 0 | UNRESOLVED: no `a*b-c` is visible in the C body of `ftYs_JumpAerial_Enter`; the fused op may come from an inlined helper |
| `ftCo_800A229C` | fmadds 3, fnmsubs 3, dbl 1 | fmadds 14, dbl 1 | the port has 15 fused sites, the DOL 7 (CPU code); not checked site by site |

### Ranked disagreements (all, online-relevant areas)

Columns: score, address, function, area, DOL counts, Rust direct counts, Rust counts with folded callees.

```
  118.0 80379598 HSD_MtxInverseConcat                 HSD      DOL[fmadds 50, fmsubs 8, fnmsubs 10] Rust[0] fold[fmadds 9]  folded: determinant3x4,inv_t,inverse_concat
   79.0 800CF6E8 ftCo_800CF6E8                        fighter  DOL[fmadds 37] Rust[0] fold[0]
   73.0 800096B4 lbColl_800096B4                      lb       DOL[dbl 3] Rust[0] fold[fmadds 73, dbl 3]  folded: capsule_mtxs,concat,determinant3x4,draw_list,inv_t,inverse_in_place,nr_step,sqrt
   61.0 80008FC8 lbColl_80008FC8                      lb       DOL[dbl 3] Rust[0] fold[fmadds 61, dbl 3]  folded: capsule_mtxs,concat,determinant3x4,draw_list,inv_t,inverse_in_place,nr_step,sqrt
   59.0 8036D7E4 drawShapeAnim                        HSD      DOL[fmadds 30] Rust[fmadds 2] fold[fmadds 2]
   46.0 80378A94 splGetSplinePoint                    HSD      DOL[fmadds 45, fmsubs 1, fnmsubs 1] Rust[0] fold[fmadds 24]  folded: bezier_point,bspline_point,cardinal_point,get_spline_point,weigh
   37.0 8037231C resolveIKJoint1                      HSD      DOL[fmsubs 1, dbl 15] Rust[0] fold[0]
   35.0 80055E9C mpLib_80055E9C                       stage/co DOL[fmadds 22] Rust[fmadds 6] fold[fmadds 6]
   35.0 8037EF28 QuatSlerp                            HSD      DOL[fmadds 15] Rust[0] fold[0]
   34.0 8000E9F0 lbShadow_8000E9F0                    lb       DOL[fmadds 31, fmsubs 3, fnmsubs 2] Rust[0] fold[fmadds 19]  folded: splGetBSplineTangent,splGetBezierTangent,splGetCardinalTangent
   26.0 80379310 HSD_MtxInverse                       HSD      DOL[fmadds 2, fmsubs 8, fnmsubs 10] Rust[0] fold[fmadds 7]  folded: determinant3x4,inv_t,inverse
   25.0 8037B7B0 resolveCnsOrientation                HSD      DOL[dbl 6] Rust[0] fold[fmadds 25, dbl 6]  folded: concat,determinant3x4,inv_t,inverse_concat,mtx_col_mag,nr_step,sqrtf,update
   23.0 80372B08 resolveIKJoint2                      HSD      DOL[dbl 9] Rust[0] fold[0]
   22.0 8037A54C HSD_MtxScaledAdd                     HSD      DOL[fmadds 12] Rust[0] fold[fmadds 1]  folded: scaled_add
   21.0 80022E68 atanf                                lb       DOL[fmadds 6, fnmsubs 2] Rust[0] fold[0]
   20.0 801DBB60 grZebes_801DBB60                     stage/co DOL[fmadds 16, dbl 18] Rust[fmadds 16, dbl 3] fold[fmadds 16, dbl 8]  folded: sqrtf
   20.0 8021FE80 stageGObj2_GObjProc                  stage/co DOL[0] Rust[fmadds 2] fold[fmadds 9]  folded: Camera_Bounds,Camera_LogicBounds,project_ground_x
   20.0 80220164 stageGObj2_GObjProc                  stage/co DOL[0] Rust[fmadds 2] fold[fmadds 9]  folded: Camera_Bounds,Camera_LogicBounds,project_ground_x
   20.0 80220448 stageGObj2_GObjProc                  stage/co DOL[0] Rust[fmadds 2] fold[fmadds 9]  folded: Camera_Bounds,Camera_LogicBounds,project_ground_x
   19.0 8000DE38 lbVector_8000DE38                    lb       DOL[fmadds 9, fmsubs 3] Rust[fmadds 4] fold[fmadds 4]
   19.0 800A229C ftCo_800A229C                        fighter  DOL[fmadds 3, fnmsubs 3, dbl 1] Rust[fmadds 14, dbl 1] fold[fmadds 14, dbl 1]
   18.0 80379A20 HSD_MtxInverseTranspose              HSD      DOL[fmadds 2, fmsubs 5, fnmsubs 7] Rust[0] fold[fmadds 5]  folded: determinant3x4,inverse_transpose
   17.0 801E08CC grCorneria_801E08CC                  stage/co DOL[fmadds 3, fnmsubs 3] Rust[0] fold[0]
   17.0 801E0A74 grCorneria_801E0A74                  stage/co DOL[fmadds 3, fnmsubs 3] Rust[0] fold[0]
   17.0 80368E70 HSD_CObjGetUpVector                  HSD      DOL[dbl 6] Rust[0] fold[0]
   17.0 803699C0 HSD_CObjSetRoll                      HSD      DOL[dbl 6] Rust[0] fold[0]
   16.0 80021410 lbBgFlash_80021410                   lb       DOL[fmadds 18, fnmadds 1, fnmsubs 1, dbl 12] Rust[fmadds 19, fnmsubs 1] fold[fmadds 19, fnmsubs 1, dbl 4]  folded: sqrtf_store
   16.0 801DC744 grZebes_801DC744                     stage/co DOL[fmadds 2, fnmsubs 1, dbl 8] Rust[fnmsubs 1, dbl 16] fold[fmadds 2, fnmsubs 1, dbl 16]  folded: grZebes_SpawnBubbleLine
   15.0 803B376C fn_803B376C                          HSD      DOL[dbl 12] Rust[dbl 6] fold[dbl 6]
   15.0 803B61B4 fn_803B61B4                          HSD      DOL[dbl 12] Rust[dbl 6] fold[dbl 6]
   14.0 801C4FAC Ground_801C4FAC                      stage/co DOL[fmadds 1, dbl 9] Rust[0] fold[dbl 3]  folded: sqrtf
   13.0 801E1060 grCorneria_801E1060                  stage/co DOL[fmadds 4] Rust[0] fold[0]
   13.0 801EA05C grBigBlue_801EA05C                   stage/co DOL[fmadds 4] Rust[0] fold[0]
   13.0 803805DC HSD_ByteCodeEval                     HSD      DOL[fnmsubs 1, dbl 3] Rust[0] fold[0]
   12.0 800A53DC ftCo_800A53DC                        fighter  DOL[fmadds 4] Rust[0] fold[fmadds 4, dbl 12]  folded: ftCo_800A1AB4,nr_step,sqrtf
   12.0 800BB220 ftCo_800BB220                        fighter  DOL[fmadds 12] Rust[fmadds 3] fold[fmadds 6]  folded: predict_hitbox
   12.0 801DB3CC grZebes_801DB3CC                     stage/co DOL[fmadds 7, dbl 9] Rust[fmadds 7] fold[fmadds 7, dbl 3]  folded: sqrtf
   12.0 8027781C it_8027781C                          items    DOL[fmadds 4, dbl 8] Rust[0] fold[fmadds 4, dbl 2]  folded: product_xy,return_sqrt_value,sqrtf_accurate
   11.0 801E03C8 grCorneria_801E03C8                  stage/co DOL[fmadds 3] Rust[0] fold[0]
   11.0 8021483C grGreens_8021483C                    stage/co DOL[fmadds 8] Rust[fmadds 4] fold[fmadds 4]
   11.0 80373078 HSD_JObjSetupMatrixSub               HSD      DOL[dbl 3] Rust[0] fold[0]
   10.0 8007A06C ftColl_8007A06C                      fighter  DOL[fmadds 12, fmsubs 2] Rust[fmadds 2] fold[fmadds 21, dbl 3]  folded: Geno_Autolink,ftcoll_80079ab0,inner_fixed,inner_percent,knockback,sqrtf
   10.0 800C2600 ftCo_800C2600                        fighter  DOL[fmadds 23, dbl 3] Rust[0] fold[fmadds 30, dbl 6]  folded: afterimage_strip,angle,dot3,lbvector_cos,lbvector_sin,nr_step,quintic,rotate_abo
   10.0 801152D0 ftNs_AttackHi4_YoyoSetHitPosUnk      fighter  DOL[fmadds 3] Rust[0] fold[fmadds 13]  folded: blend_hit_pos,hand_pos,lbvector_cos,lbvector_sin,quintic,rotate
    9.0 80084CE4 ft_80084CE4                          fighter  DOL[0] Rust[fmadds 2] fold[fmadds 2]
    9.0 801DFC98 grCorneria_801DFC98                  stage/co DOL[fmadds 2] Rust[0] fold[0]
    9.0 801DFF20 grCorneria_801DFF20                  stage/co DOL[fmadds 2] Rust[0] fold[0]
    9.0 801E01A8 grCorneria_801E01A8                  stage/co DOL[fmadds 2] Rust[0] fold[0]
    9.0 801E0678 grCorneria_801E0678                  stage/co DOL[fmadds 1, fnmadds 1] Rust[0] fold[0]
    9.0 801EACE8 grBigBlue_801EACE8                   stage/co DOL[fmadds 1, fnmsubs 1] Rust[0] fold[0]
    9.0 802CED54 it_802CED54                          items    DOL[fnmsubs 2] Rust[0] fold[0]
    9.0 803A4A68 HSD_SObjLib_803A4A68                 HSD      DOL[fmadds 2] Rust[0] fold[0]
    8.0 80118384 ftNs_SpecialHi_Enter                 fighter  DOL[fmadds 1, fmsubs 1] Rust[fmadds 1] fold[fmadds 4, dbl 6]  folded: angle,dot3,len,nr_step,set_tilt,sqrtf,tilt_rot_x
    8.0 8020A21C grShrineRoute_8020A21C               stage/co DOL[dbl 6] Rust[0] fold[dbl 2]  folded: sqrtf
    8.0 80218888 grPushOn_80218888                    stage/co DOL[dbl 6] Rust[0] fold[dbl 2]  folded: sqrtf
    7.0 800077A0 lbColl_800077A0                      lb       DOL[fmadds 12, fmsubs 1, fnmsubs 1, dbl 6] Rust[fmadds 8, fmsubs 2] fold[fmadds 15, fmsubs 2, dbl 10]  folded: angle_xy,dot2,len_xy_accurate,mult_vec,nr_step,sq3,sqrtf_accurate,sqrtf_store
    7.0 80022DBC asinf                                lb       DOL[fnmsubs 1] Rust[0] fold[0]
    7.0 800B21C8 ftCo_800B21C8                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,ftCo_800ADE48_impl,ftCo_800ADE48_r31,ftCo_CpuUpdateTargetDistance,
    7.0 800CB6EC ft_800CB6EC                          fighter  DOL[fnmsubs 1] Rust[0] fold[0]
    7.0 800CBE98 ftYs_JumpAerial_Enter                fighter  DOL[fnmsubs 1] Rust[0] fold[0]
    7.0 800CC3C4 ftYs_JumpAerial_Anim_Cb              fighter  DOL[fnmsubs 1] Rust[0] fold[0]
    7.0 800D0CBC ftCo_800D0CBC                        fighter  DOL[fmadds 1] Rust[0] fold[0]
    7.0 800D0EC8 ftCo_800D0EC8                        fighter  DOL[fmadds 1] Rust[0] fold[0]
    7.0 80115C9C ftNs_AttackHi4_Anim                  fighter  DOL[fmadds 1] Rust[0] fold[0]
    7.0 801D577C grKongo_801D577C                     stage/co DOL[fmadds 1] Rust[fmadds 3] fold[fmadds 3]
    7.0 801E43E0 grOnett_801E43E0                     stage/co DOL[fmadds 2] Rust[fmadds 1] fold[fmadds 9]  folded: Camera_Bounds,Camera_LogicBounds,grOnett_WaitCar,project_ground_x
    7.0 802100FC stageGObj2_GObjProc                  stage/co DOL[fmadds 1, fmsubs 1] Rust[fmadds 2] fold[fmadds 9]  folded: Camera_Bounds,Camera_LogicBounds,project_ground_x
    7.0 80210D10 grOldPupupu_80210D10                 stage/co DOL[fmadds 1, fmsubs 1, fnmsubs 2] Rust[fmadds 4] fold[fmadds 11]  folded: Camera_Bounds,Camera_LogicBounds,project_ground_x
    7.0 802EE374 it_802EE374                          items    DOL[fmsubs 1] Rust[0] fold[0]
    7.0 8035F0EC TObjSetupMtx                         HSD      DOL[fmadds 5] Rust[fmadds 3] fold[fmadds 3]
    7.0 803B3408 hsd_803B3408                         HSD      DOL[fmadds 5, fmsubs 2, fnmsubs 1] Rust[fmadds 6] fold[fmadds 6]
    6.0 800A4A40 ftCo_800A4A40                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,nr_step,sqrtf
    6.0 800A4BEC ftCo_800A4BEC                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,nr_step,sqrtf
    6.0 800A50D4 ftCo_800A50D4                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,nr_step,sqrtf
    6.0 800A5CE0 ftCo_800A5CE0                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,nr_step,sqrtf
    6.0 800AF290 ftCo_800AF290                        fighter  DOL[fmadds 3, dbl 3] Rust[0] fold[fmadds 3, dbl 9]  folded: ftCo_800A1AB4,ftCo_CpuApproachFoodItemTarget,ftCo_CpuUpdateTargetDistance,nr_ste
    6.0 801F1A34 grMuteCity_801F1A34                  stage/co DOL[fmadds 4, fmsubs 1, dbl 6] Rust[fmadds 3, dbl 3] fold[fmadds 4, dbl 4]  folded: grMc_RandomWreckOffset,sqrtf
    6.0 8027737C it_8027737C                          items    DOL[dbl 4] Rust[0] fold[dbl 1]  folded: sqrtf_accurate,sqrtf_accurate_store
    6.0 802775F0 it_802775F0                          items    DOL[dbl 4] Rust[0] fold[dbl 1]  folded: sqrtf_accurate
    6.0 8029B8EC itMariofireball_UnkMotion0_Coll      items    DOL[dbl 4] Rust[0] fold[dbl 1] manual-f64 1  folded: calc_dist_2d_accurate,sqrtf_accurate
    6.0 802B51A0 itSamusbomb_UnkMotion2_Phys          items    DOL[dbl 4] Rust[0] fold[dbl 1]  folded: my_sqrtf
    6.0 802C0368 itLuigifireball_UnkMotion0_Coll      items    DOL[dbl 4] Rust[0] fold[dbl 1] manual-f64 1  folded: calc_dist_2d_accurate,sqrtf_accurate
    6.0 8037B400 set_dirup_matrix                     HSD      DOL[dbl 6] Rust[0] fold[fmadds 6, dbl 6]  folded: nr_step,sqrtf,update,vec_dot
    5.0 800122F0 lb_800122F0                          lb       DOL[fnmsubs 2] Rust[fmadds 3] fold[fmadds 3]
    5.0 80022D1C acosf                                lb       DOL[fnmsubs 4] Rust[fnmsubs 1] fold[fmadds 6, fnmsubs 3]  folded: atanf_fp
    5.0 800C0D0C ftCo_800C0D0C                        fighter  DOL[fmadds 2] Rust[fmadds 3] fold[fmadds 3]
    5.0 800C3D6C ftCo_AirCatch_Anim                   fighter  DOL[dbl 3] Rust[dbl 2] fold[dbl 2]
    5.0 800CF594 ftCo_CalcYScaledKnockback            fighter  DOL[fmadds 2] Rust[fmadds 1] fold[fmadds 1]
    5.0 800D8EC8 fn_800D8EC8                          fighter  DOL[dbl 2] Rust[dbl 1] fold[dbl 1]
    5.0 801ED694 grBigBlue_801ED694                   stage/co DOL[fmadds 8, fmsubs 3, fnmsubs 3] Rust[fmadds 9, fmsubs 3, fnmsubs 3] fold[fmadds 9, fmsubs 3, fnmsubs 3]
    5.0 801EE398 grBigBlue_801EE398                   stage/co DOL[fmadds 3, fnmsubs 1] Rust[fmadds 2, fmsubs 1] fold[fmadds 2, fmsubs 1]
    5.0 80367EB0 setupTopHalfCamera                   HSD      DOL[fnmsubs 3] Rust[fmadds 2] fold[fmadds 2]
    5.0 803680F8 setupBottomHalfCamera                HSD      DOL[fmadds 3] Rust[fmadds 2] fold[fmadds 2]
    5.0 803A8134 HSD_SisLib_803A8134                  HSD      DOL[fmadds 2, fnmsubs 2] Rust[fmadds 2, fmsubs 1] fold[fmadds 2, fmsubs 1]
    4.0 80020AEC fn_80020AEC                          lb       DOL[dbl 3] Rust[0] fold[dbl 1]  folded: newton_rsqrt
    4.0 80020E38 lbBgFlash_80020E38                   lb       DOL[fmadds 2, dbl 3] Rust[fmadds 2] fold[fmadds 2, dbl 1]  folded: newton_rsqrt
    4.0 80103FA8 ftKb_PrSpecialNTurn_Phys             fighter  DOL[fmadds 4] Rust[0] fold[fmadds 2]  folded: turn_gr_vel
    4.0 801408B8 ftPr_SpecialNTurn_Phys               fighter  DOL[fmadds 4] Rust[0] fold[fmadds 2]  folded: turn_gr_vel
    4.0 801D99E0 grZebes_801D99E0                     stage/co DOL[fmadds 1, dbl 4] Rust[fmadds 1, dbl 1] fold[fmadds 1, dbl 2]  folded: sqrtf
    4.0 801E8560 fn_801E8560                          stage/co DOL[fmadds 3, dbl 3] Rust[fmadds 3] fold[fmadds 3, dbl 1]  folded: sqrtf,sqrtf_
    4.0 80200578 grRCruise_80200578                   stage/co DOL[fmadds 3, dbl 3] Rust[fmadds 3] fold[fmadds 3, dbl 1]  folded: sqrtf
    4.0 80207C88 grKinokoRoute_80207C88               stage/co DOL[dbl 3] Rust[0] fold[dbl 1]  folded: sqrtf
    4.0 802A5FE0 it_802A5FE0                          items    DOL[fmadds 14, dbl 12] Rust[0] fold[fmadds 12, dbl 12]  folded: it_802A3C98,it_802A3C98_inl,len3,nr_step,place,sqrtf
    4.0 802D208C it_802D208C                          items    DOL[fmadds 3, fnmsubs 1, dbl 3] Rust[fmadds 2] fold[fmadds 2, dbl 3]  folded: it_802D208C_inline,my_sqrtf,nr_step,sqrtf
    4.0 80391F28 hsd_80391F28                         HSD      DOL[fmadds 1, dbl 3] Rust[0] fold[fmadds 1, dbl 1]  folded: hsd_80391F28_len
    4.0 8039930C hsd_8039930C                         HSD      DOL[fmadds 17, fmsubs 4, dbl 9] Rust[0] fold[fmadds 19, dbl 9]  folded: interpret,nr_step,run_bytecode,sqrtf
    3.0 8002B3D4 Camera_8002B3D4                      stage/co DOL[fmadds 2] Rust[0] fold[fmadds 2, dbl 3]  folded: nr_step,sqrtf,update_transform,update_zoom_distance
    3.0 8002CDDC Camera_8002CDDC                      stage/co DOL[fmadds 3] Rust[fmadds 1] fold[fmadds 3, dbl 3]  folded: nr_step,sqrtf,update_transform,update_zoom_distance
    3.0 8002D318 Camera_8002D318                      stage/co DOL[fmadds 13, fnmsubs 1] Rust[fmadds 12] fold[fmadds 14, dbl 3]  folded: nr_step,sqrtf,update_transform,update_zoom_distance
    3.0 8002D85C Camera_8002D85C                      stage/co DOL[fmadds 12] Rust[fmadds 10] fold[fmadds 12, dbl 3]  folded: nr_step,sqrtf,update_transform,update_zoom_distance
    3.0 8002DDC4 Camera_8002DDC4                      stage/co DOL[fmadds 14] Rust[fmadds 11] fold[fmadds 14, dbl 3]  folded: lbVector_Len,nr_step,set_bounds_z,smooth_fixed_camera_interest,sqrtf
    3.0 800942A0 ftpickupitem_800942A0                fighter  DOL[fmadds 2] Rust[fmadds 1] fold[fmadds 5]  folded: Geno_ItemPressDist,geno_item_lift,in_box
    3.0 800A20A0 ftCo_800A20A0                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 1, dbl 3]  folded: ftCo_800A1AB4,nr_step,sqrtf
    3.0 800AC7D4 ftCo_800AC7D4                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 1, dbl 3]  folded: ftCo_800A1AB4,nr_step,sqrtf
    3.0 800ACB44 ftCo_800ACB44                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 1, dbl 3]  folded: ftCo_800A1AB4,nr_step,sqrtf
    3.0 800AD7FC ftCo_800AD7FC                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 1, dbl 3]  folded: ftCo_800A1AB4,nr_step,sqrtf
    3.0 800AF78C ftCo_800AF78C                        fighter  DOL[fmadds 2, dbl 3] Rust[0] fold[fmadds 2, dbl 6]  folded: ftCo_800A1AB4,ftCo_CpuApproachFoodItemTarget,ftCo_CpuUpdateTargetDistance,nr_ste
    3.0 800B1478 ftCo_800B1478                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 1, dbl 3]  folded: ftCo_800A1AB4,ftCo_CpuUpdateTargetDistance,nr_step,sqrtf
    3.0 800B4AB0 ftCo_800B4AB0                        fighter  DOL[fmadds 8, dbl 17] Rust[0] fold[fmadds 11, dbl 17]  folded: nr_step,pred_rel_y,pred_self_y,reset_ranges,select_vs_fighter,sqrtf
    3.0 80120E68 ftPp_SpecialS_80120E68               fighter  DOL[fnmsubs 1, dbl 3] Rust[fmadds 1] fold[fmadds 1, dbl 6]  folded: len,my_sqrtf,normalize,nr_step,sqrtf
    3.0 802D472C it_802D472C                          items    DOL[dbl 3] Rust[0] fold[dbl 6]  folded: len,normalize,nr_step,sqrtf
    2.0 8007DFD0 ftCommon_8007DFD0                    fighter  DOL[fmadds 2] Rust[0] fold[fmadds 4]  folded: ftcommon_8007dd7c,nudge_dx
    2.0 800ABBA8 ftCo_800ABBA8                        fighter  DOL[fmadds 2, dbl 11] Rust[fmadds 4, dbl 2] fold[fmadds 4, dbl 11]  folded: nr_step,sqrtf
    2.0 80141254 ftPr_SpecialNRelease_Coll            fighter  DOL[fmadds 2] Rust[0] fold[fmadds 1]  folded: ftKb_PrWallBounceEffect,wall_check
    2.0 801419E0 ftPr_SpecialAirNChargeRelease_Coll   fighter  DOL[fmadds 2] Rust[0] fold[fmadds 1]  folded: ftKb_PrWallBounceEffect,wall_check
    2.0 8021D680 grHomeRun_8021D680                   stage/co DOL[fmadds 4, fnmsubs 3] Rust[fmadds 3, fnmadds 3] fold[fmadds 6, fnmadds 3]  folded: set_text_pos
    2.0 8021DF50 grHomeRun_8021DF50                   stage/co DOL[fmadds 1] Rust[0] fold[fmadds 3]  folded: set_text_pos,text_stage_proc
    2.0 8021E0D4 grHomeRun_8021E0D4                   stage/co DOL[fmadds 1] Rust[0] fold[fmadds 3]  folded: set_text_pos,text_stage_proc
    2.0 8021E258 grHomeRun_8021E258                   stage/co DOL[fmadds 1] Rust[0] fold[fmadds 3]  folded: set_text_pos,text_stage_proc
    2.0 8021E3DC grHomeRun_8021E3DC                   stage/co DOL[fmadds 1] Rust[0] fold[fmadds 3]  folded: set_text_pos,text_stage_proc
    2.0 8027C8D0 it_8027C8D0                          items    DOL[fmadds 1, fmsubs 1, dbl 3] Rust[fmadds 2] fold[fmadds 4, dbl 3]  folded: itzako_sqrtf,product_xyz,return_sqrt_value3
    2.0 80376E90 HSD_PadClampCheck3                   HSD      DOL[dbl 6] Rust[fmadds 2] fold[fmadds 2, dbl 6]  folded: nr_step,sqrtf
    2.0 80378F38 splArcLengthGetParameter             HSD      DOL[fmadds 14] Rust[0] fold[fmadds 10, dbl 6]  folded: arc_length_get_parameter,arc_length_polynomial,nr_step,simpsons_middle,sqrtf
    2.0 8039DAD4 hsd_8039DAD4                         HSD      DOL[fmadds 17, fnmsubs 1, dbl 18] Rust[fmadds 16, fnmsubs 1] fold[fmadds 16, fnmsubs 1, dbl 18]  folded: nr_step,sqrtf
    2.0 803A0088 psDispParticles                      HSD      DOL[fmadds 143, fmsubs 9, fnmsubs 2, dbl 24] Rust[fmadds 16] fold[fmadds 156, dbl 24]  folded: nr_step,psDispSub,psDispSubAPPSRTPoint,psDispSubAppSRT,psDispSubMakePolygon,sqrt
    2.0 803A84BC HSD_SisLib_803A84BC                  HSD      DOL[fmadds 12, fnmsubs 6, dbl 1] Rust[0] fold[fmadds 13, fmsubs 4, dbl 1]  folded: sis_render_text
    1.0 8008DCE0 ftCo_8008DCE0                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 2]  folded: ScriptGame_ShockExtra,ScriptGame_ShockTake,calcAngle,sq_len2
    1.0 80090594 ftCo_80090594                        fighter  DOL[fmadds 1] Rust[0] fold[fmadds 2]  folded: calc_hitlag,calc_shift
    1.0 800C318C ftCo_800C318C                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 3]  folded: inlineA0
    1.0 800C4550 ftCo_800C4550                        fighter  DOL[fmadds 2] Rust[0] fold[fmadds 3]  folded: commonCall
    1.0 800D9930 fn_800D9930                          fighter  DOL[dbl 3] Rust[0] fold[dbl 4]  folded: grapple,jitter
    1.0 80113838 ftSk_SpecialHi_80113838              fighter  DOL[fmadds 1, dbl 3] Rust[fmadds 1] fold[fmadds 2, dbl 3]  folded: angle_xy,dot2,my_sqrtf
    1.0 80145990 ftMt_SpecialHi_Enter                 fighter  DOL[fmadds 1, dbl 3] Rust[fmadds 1] fold[fmadds 2, dbl 3]  folded: angle_xy,dot2,nr_step,sqrtf
    1.0 80391664 DrawASCII                            HSD      DOL[fmadds 4, dbl 5] Rust[fmadds 4] fold[fmadds 4, dbl 6]  folded: fma_lit
    0.5 8000D148 lb_8000D148                          lb       DOL[fmadds 2, fmsubs 1, dbl 3] Rust[fmadds 3] fold[fmadds 3, dbl 3]  folded: nr_step,sqrtf
    0.5 8000D8F4 lbVector_RotateAboutUnitAxis         lb       DOL[fmadds 9, fmsubs 5, dbl 3] Rust[fmadds 10] fold[fmadds 14, dbl 3]  folded: lbvector_cos,lbvector_sin,nr_step,quintic,sqrtf
    0.5 8000DB00 lbVector_Rotate                      lb       DOL[fmadds 5, fmsubs 5] Rust[fmadds 6] fold[fmadds 10]  folded: lbvector_cos,lbvector_sin,quintic
    0.5 8000E530 lbVector_CreateEulerMatrix           lb       DOL[fmadds 8, fmsubs 8] Rust[fmadds 4] fold[fmadds 16]  folded: lbvector_cos,lbvector_sin,quintic
    0.5 8000F38C lbShadow_8000F38C                    lb       DOL[fnmsubs 3, dbl 3] Rust[fmsubs 3] fold[fmsubs 3, dbl 3]  folded: nr_step,sqrtf
    0.5 8001044C lb_8001044C                          lb       DOL[fmadds 7, fmsubs 1, fnmsubs 1, dbl 9] Rust[fmadds 7, fmsubs 2] fold[fmadds 7, fmsubs 2, dbl 9]  folded: nr_step,sqrtf
    0.5 80021CE8 lbRefract_80021CE8                   lb       DOL[fmadds 3, fnmsubs 1, dbl 3] Rust[fmadds 1] fold[fmadds 4, dbl 3]  folded: lbRefract_WriteTexCoord,my_fmodf,nr_step,sqrtf
    0.5 80022DF8 lb_sqrtf                             lb       DOL[fnmsubs 3] Rust[0] fold[fmadds 3]  folded: nr_step_f
    0.5 80029CF8 Camera_80029CF8                      stage/co DOL[fmadds 4, fmsubs 2, fnmsubs 2] Rust[fmadds 5, fnmsubs 2] fold[fmadds 6, fnmsubs 2]  folded: get_y_bias
    0.5 800307D0 Camera_800307D0                      stage/co DOL[fmadds 2, fmsubs 2, fnmsubs 3] Rust[0] fold[fmadds 7]  folded: Camera_Bounds,project_ground_x
    0.5 8004293C mpColl_LoadECB_Fixed                 stage/co DOL[fmadds 2, fmsubs 2] Rust[fmadds 4] fold[fmadds 4]
    0.5 80043C6C mpColl_80043C6C                      stage/co DOL[fmadds 1, fnmsubs 3] Rust[fmadds 3, fnmsubs 1] fold[fmadds 3, fnmsubs 1]
    0.5 80076528 ftColl_80076528                      fighter  DOL[fnmsubs 2] Rust[0] fold[fmadds 2]  folded: combo_count_push
    0.5 80085030 ft_80085030                          fighter  DOL[fmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 80085154 ft_80085154                          fighter  DOL[fmadds 1, fmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 80091620 ftCo_DamageIce_Collide               fighter  DOL[fnmsubs 1, dbl 3] Rust[fmadds 1] fold[fmadds 1, dbl 3]  folded: ftCo_Speed,nr_step,sqrtf
    0.5 8009EE30 ftCo_8009EE30                        fighter  DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 800B52AC ftCo_800B52AC                        fighter  DOL[fmadds 10, fmsubs 1, dbl 17] Rust[0] fold[fmadds 11, dbl 17]  folded: nr_step,pred_rel_y,pred_self_y,reset_ranges,select_vs_fighter,sqrtf
    0.5 800BC3D0 ftCo_800BC3D0                        fighter  DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 800C18A8 ftCo_800C18A8                        fighter  DOL[fmadds 1, fnmsubs 1, dbl 3] Rust[fmadds 2] fold[fmadds 2, dbl 3]  folded: nr_step,sqrtf
    0.5 800C1E64 ftCo_800C1E64                        fighter  DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 800C6408 ftCo_800C6408                        fighter  DOL[fnmsubs 1] Rust[0] fold[fmsubs 1]  folded: accessory_pos
    0.5 800C69F4 fn_800C69F4                          fighter  DOL[fnmsubs 1] Rust[0] fold[fmsubs 1]  folded: accessory_follow,accessory_pos
    0.5 800C6F34 fn_800C6F34                          fighter  DOL[fnmsubs 1] Rust[0] fold[fmsubs 1]  folded: accessory_follow,accessory_pos
    0.5 800C7DC4 fn_800C7DC4                          fighter  DOL[fnmsubs 1] Rust[fmsubs 1] fold[fmsubs 1]
    0.5 800D4FF4 ftCo_800D4FF4                        fighter  DOL[fmadds 2, fnmsubs 1] Rust[fmadds 2, fmsubs 1] fold[fmadds 2, fmsubs 1]
    0.5 800D54A4 fn_800D54A4                          fighter  DOL[fnmsubs 1] Rust[fmsubs 1] fold[fmsubs 1]
    0.5 800E7758 ftFx_SpecialAirHi_Phys               fighter  DOL[fnmsubs 2] Rust[fmadds 2] fold[fmadds 2]
    0.5 800E8F20 ftFx_SpecialLw_Turn                  fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: turn_body
    0.5 800E8FDC ftFx_SpecialLwTurn_Anim              fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: ftFox_SpecialLw_Turn_Inline,turn_anim,turn_body
    0.5 800E90EC ftFx_SpecialAirLwTurn_Anim           fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: ftFox_SpecialLw_Turn_Inline,turn_anim,turn_body
    0.5 80114F70 ftNs_AttackHi4_YoyoApplyDamage       fighter  DOL[fmadds 1, fmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 80115680 ftNs_AttackHi4_YoyoSetChargeDamage   fighter  DOL[fmadds 1, fmsubs 1] Rust[0] fold[fmadds 2]  folded: charge_damage_scale
    0.5 80118570 ftNs_SpecialAirHi_Enter              fighter  DOL[fmadds 1, fmsubs 1] Rust[0] fold[fmadds 2]  folded: NessFloatMath_PKThunder2,set_tilt,tilt_rot_x
    0.5 80118B04 ftNs_SpecialAirHiHold_Anim           fighter  DOL[fmadds 1, fmsubs 1] Rust[0] fold[fmadds 2]  folded: NessFloatMath_PKThunder2,ftNs_SpecialAirHi_Enter,hold_anim,set_tilt,tilt_rot_x
    0.5 8011901C ftNs_SpecialHi_Phys                  fighter  DOL[fmsubs 1, fnmsubs 1] Rust[fmadds 1] fold[fmadds 2]  folded: set_tilt,tilt_rot_x
    0.5 80119254 ftNs_SpecialAirHi_Phys               fighter  DOL[fmsubs 1, dbl 3] Rust[0] fold[fmadds 1, dbl 3]  folded: ftNess_atan2,len_xy,nr_step,set_tilt,sqrtf,tilt_rot_x
    0.5 8011A700 ftNs_SpecialLwTurn_Anim              fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: turn_anim,turn_rot_y,turn_step
    0.5 8011A810 ftNs_SpecialAirLwTurn_Anim           fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: turn_anim,turn_rot_y,turn_step
    0.5 80123390 ftPp_SpecialHi_0_Phys                fighter  DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 80123F98 ftPp_SpecialS_0_Coll                 fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: coll_follow
    0.5 801241A0 ftPp_SpecialS_1_Coll                 fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: coll_follow
    0.5 80126144 ftPk_SpecialHiStart1_Anim            fighter  DOL[fmsubs 2] Rust[0] fold[fmadds 2]  folded: start1_anim
    0.5 801262B4 ftPk_SpecialAirHiStart1_Anim         fighter  DOL[fmsubs 2] Rust[0] fold[fmadds 2]  folded: start1_anim
    0.5 801310E8 ftYs_SpecialAirSLoop_0_Phys          fighter  DOL[fmadds 3, fnmsubs 2] Rust[fmadds 5] fold[fmadds 5]
    0.5 8014DEF0 ftGw_SpecialHi_ItemRescueSetup       fighter  DOL[fnmsubs 1] Rust[0] fold[fmadds 1]  folded: rescue_spawn_y
    0.5 801DC408 grZebes_801DC408                     stage/co DOL[fmadds 2, fnmsubs 1, dbl 1] Rust[fmadds 3, dbl 1] fold[fmadds 3, dbl 1]
    0.5 801DD9A8 grCorneria_801DD9A8                  stage/co DOL[fmsubs 2] Rust[fmadds 2] fold[fmadds 2]
    0.5 801E2228 grCorneria_801E2228                  stage/co DOL[fmadds 2, fmsubs 3] Rust[fmadds 5] fold[fmadds 5]
    0.5 801E2EEC grCorneria_801E2EEC                  stage/co DOL[fmadds 1, fnmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 801E2FCC grCorneria_801E2FCC                  stage/co DOL[fmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 801E5214 grOnett_801E5214                     stage/co DOL[fmadds 1, fnmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 801EBAF8 grBigBlue_801EBAF8                   stage/co DOL[fnmsubs 3, dbl 2] Rust[fmsubs 3, dbl 2] fold[fmsubs 3, dbl 2]
    0.5 801F7A2C stageGObj10_GObjProc                 stage/co DOL[fmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 801F8B10 stageGObj8_GObjProc                  stage/co DOL[fmadds 1, fnmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 801F993C grIceMt_801F993C                     stage/co DOL[fmadds 1, fnmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 801FFADC grRCruise_801FFADC                   stage/co DOL[fnmsubs 2] Rust[0] fold[fmadds 2]  folded: grRCruise_801FFADC_inline
    0.5 8020C85C grBigBlueRoute_8020C85C              stage/co DOL[fmsubs 1] Rust[0] fold[fmadds 1]  folded: grBigBlueRoute_SpawnRoute
    0.5 8020CD20 grBigBlueRoute_8020CD20              stage/co DOL[fmadds 1, fmsubs 1] Rust[fmadds 2] fold[fmadds 2]
    0.5 8020F088 grOldYoshi_8020F088                  stage/co DOL[fmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 802184CC grPushOn_802184CC                    stage/co DOL[fmsubs 4] Rust[fmadds 4] fold[fmadds 4]
    0.5 8021ADD0 grLast_8021ADD0                      stage/co DOL[fmadds 2, fmsubs 2] Rust[fmadds 4] fold[fmadds 4]
    0.5 8021EA30 grHomeRun_8021EA30                   stage/co DOL[fnmsubs 1] Rust[fnmadds 1] fold[fnmadds 1]
    0.5 8027BB1C it_8027BB1C                          items    DOL[fmadds 3, fmsubs 3] Rust[fmadds 6] fold[fmadds 6]
    0.5 8027BBF4 it_8027BBF4                          items    DOL[fmadds 1, fmsubs 2] Rust[fmadds 3] fold[fmadds 3]
    0.5 8027C0F0 it_8027C0F0                          items    DOL[fmadds 1, fmsubs 2] Rust[fmadds 3] fold[fmadds 3]
    0.5 802AA75C itNesspkfirepillar_Logic24_DmgReceived items    DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 802B900C it_802B900C                          items    DOL[fmadds 10, fnmsubs 1] Rust[fmadds 9] fold[fmadds 11]  folded: samus_grapple_calc_grav
    0.5 802B9328 it_802B9328                          items    DOL[fmadds 10, fnmsubs 1] Rust[fmadds 9] fold[fmadds 11]  folded: it_802B9328_grav
    0.5 802B99A0 it_802B99A0                          items    DOL[fmadds 11, fnmsubs 2] Rust[fmadds 9] fold[fmadds 13]  folded: samus_grapple_calc_grav
    0.5 802B9CE8 it_802B9CE8                          items    DOL[fmadds 11, fnmsubs 2] Rust[fmadds 9] fold[fmadds 13]  folded: samus_grapple_calc_grav
    0.5 802B9FD4 it_802B9FD4                          items    DOL[fmadds 7, fnmsubs 1] Rust[fmadds 6] fold[fmadds 8]  folded: samus_grapple_calc_grav
    0.5 802BBED0 it_802BBED0                          items    DOL[fmadds 6, fnmsubs 1] Rust[fmadds 7] fold[fmadds 7]
    0.5 802BC080 it_802BC080                          items    DOL[fmadds 8, fmsubs 2] Rust[fmadds 10] fold[fmadds 10]
    0.5 802CA3F4 it_802CA3F4                          items    DOL[fmsubs 2] Rust[fmadds 2] fold[fmadds 2]
    0.5 802D1830 it_802D1830                          items    DOL[fnmsubs 1, dbl 4] Rust[fmadds 1] fold[fmadds 1, dbl 4]  folded: my_sqrtf_accurate,nr_step,sqrtf_accurate
    0.5 802D4B14 itHitodeman_UnkMotion1_Phys          items    DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 802D8A54 itHeiho_UnkMotion1_Phys              items    DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 802D9384 itHeiho_UnkMotion4_Phys              items    DOL[fnmsubs 1] Rust[fmadds 1] fold[fmadds 1]
    0.5 802EA804 it_802EA804                          items    DOL[fnmsubs 4] Rust[0] fold[fmadds 4]  folded: neg_dot
    0.5 8037A120 HSD_MkRotationMtx                    HSD      DOL[fmadds 2, fmsubs 2] Rust[0] fold[fmadds 4]  folded: d2,mk_rotation_mtx
    0.5 8037A250 HSD_MtxSRT                           HSD      DOL[fmadds 2, fmsubs 2] Rust[0] fold[fmadds 4]  folded: d2,srt
    0.5 8037EE0C EulerToQuat                          HSD      DOL[fmadds 2, fmsubs 2] Rust[0] fold[fmadds 4]  folded: euler_to_quat
    0.5 80398F8C hsd_80398F8C                         HSD      DOL[fmadds 7, fmsubs 1, dbl 3] Rust[fmadds 8] fold[fmadds 8, dbl 3]  folded: nr_step,sqrtf
    0.5 8039F89C calcTornadoLastPos                   HSD      DOL[fmadds 5, fmsubs 1] Rust[fmadds 6] fold[fmadds 6]
    0.5 803B6820 fn_803B6820                          HSD      DOL[fmadds 1, fnmsubs 2, dbl 2] Rust[0] fold[fmadds 3, dbl 2]  folded: jpeg_store_rgb565
```
