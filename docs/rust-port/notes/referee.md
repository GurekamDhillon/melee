# DOL referee: Rust functions checked against retail main.dol (2026-11-27, branch port/referee-mvp)

Vanilla Melee NTSC 1.02 only (no m-ex, mods, Akaneia, ACE). No disc bytes, instruction words, dumps or replays are in
this note or in git: main.dol is read at run time from the user's own checkout (`REFEREE_DOL`, never printed), and the
captures (engine values from replay runs) stay in the workspace `_build/referee-mvp/`.

The correctness reference of the Rust port is the retail `main.dol` (STATUS.md, "The C build is legacy"). The referee runs
the retail function itself in the Gekko-exact PowerPC interpreter of `tools/mex-import` on the inputs the engine's Rust
port saw on a real game state, and compares the return value and every byte the function wrote, bit for bit. This is
coverage-testing.md's phase C, done in-process with no Dolphin per call.

## 1. Interpreter trust check (`tools/mex-import/tests/gekko_trust.rs`, `dol_census.rs`)

**Scope.** `dol_census` decodes every symbols.txt function of the DOL (19,827 functions, 968,511 words) and counts each
decoded form. Outside the interpreter: 88 paired-single forms (opcode 4, plus psq_l/psq_st 56/60/61: 1,013 words, all in
the SDK PSMTX/PSVEC routines, THP video, `WriteMTXPS*` and one `fn_80166A8C`), OS/TRK SPRs and `mffs`/`mtfsf`
(`__OSSaveFPUContext`, `setjmp`), cache/sync ops. No game FP instruction sets Rc. That fixes the set the check covers.

**Method.** An independent model of Dolphin's interpreter written in the test from Dolphin's semantics
(Interpreter_Integer/FloatingPoint/LoadStore/Branch.cpp, Interpreter_FPUtils.h, Common/FloatUtils.cpp, read in
`_build/dolphin-oracle/src`): its own decoder, the NI_* NaN rules (frA, then frB, then frC; default NaN positive),
Force25Bit with the denormal case, ForceSingle (NI = 0), ConvertToSingle/ConvertToDouble for stfs/lfs, the fres/frsqrte
tables, fctiw/fctiwz with Dolphin's -0 bit, FPCC compares, XER masking, the bc/bclr/bcctr rules. Its fused multiply-add is
an exact software fma on integer mantissas (not the host `mul_add`, which the interpreter uses), so the host fma is checked
too. Each of 162 instruction forms (every form the census lists, every Rc variant) gets fuzzed operands with the corner
classes weighted in: +-0, double and single-range denormals, quiet/signalling NaNs with payloads, infinities, the single
overflow edge, Force25 rounding-bit ties, fctiw range edges (+-2^31, +-0.5, ...), shift amounts 0..129, INT_MIN/-1, x/0,
r0 as an operand, aliasing registers, random CR/CTR/LR. Assumed FPSCR: RN = 0, NI = 0, no exception enables.

**Disagreements found** (20,000 sets per form, before the fixes), all fixed in Gekko mode only (`Machine::new_gekko`,
used by the referee and the m-ex guest; the plain mode stays gw_ppc.c op for op for tests/ppc_oracle.rs):

| form | interpreter did | Dolphin |
|---|---|---|
| addic / addic. | Rc taken from bit 0 of the immediate (gw_ppc.c's bug, mirrored by the decoder): `addic` with an odd immediate wrote CR0, `addic.` with an even one did not | addic. is opcode 13 |
| subfic, addic | rA = r0 read as 0 | the register's value |
| slw, srw | amount masked to 5 bits | bit 5 shifts everything out (as notes/semantics.md already has for melee_core) |
| divw | x/0 -> 0, INT_MIN/-1 -> INT_MIN | -1 for a negative dividend, else 0 |
| fadd/fsub/fmul/fdiv(s), fmadd/fmsub(s) | x86's NaN choice and its negative default NaN; fmsub negated frB's NaN | NaN of frA, frB, frC in that order, quieted; default NaN 7FF8... |
| fnmadd/fnmsub(s) | negated a NaN result | a NaN is not negated |
| fmuls/fmadds... frC | Force25 without the denormal branch (melee_core::fp::force25) | a denormal frC is normalised before rounding |
| fres | result rounded to single (truncating a NaN payload) | stored as computed |
| fsel | NaN frA selected frC | selects frB |
| fabs, fnabs | arithmetic compare (fabs(-0) = -0, NaN sign kept) | sign-bit operations |
| fctiw, fctiwz | a zero result of a negative operand had bit 32 clear | set (Dolphin's hardware-tested rule) |
| fcmpu, fcmpo | NaN gave CR field = XER.SO | FU (1); SO not involved |
| lfs | host float->double (quiets a signalling NaN) | exact bit conversion |
| stfs | host double->float, rounding | truncating ConvertToSingle; doubles in the single denormal range shifted |
| bclr, bcctr | target used as is | target & ~3 |
| mtxer | all 32 bits kept | SO, OV, CA, byte count and compare byte only |
| dcbz | effective address used as is | aligned down to 32 bytes |

Kept on purpose (listed as known in the test): **addme / subfme carry**. Dolphin's interpreter computes it as
`Helper_Carry(a, carry - 1)`, which gives CA = 0 when CA was 1; the architecture and Dolphin's JIT (host adc/sbb, what
played the online replays) give CA = 1. The interpreter keeps the architected carry. Neither form is used by the game's
gameplay code (census: no addme/subfme in the DOL at all).

The model had two bugs of its own on the way (subfc's carry, frsqrte's exponent halving), found by the disagreements and
fixed in the model against Dolphin's source.

**Result after the fixes:** 162 forms x 2,000,000 operand sets = 324,000,000 sets, **0 disagreements** outside the kept
addme/subfme carry (REFEREE_FUZZ=2000000, see the run line in section 6). Default `cargo test` runs 20,000 per form (~11 s).

**m-ex guest.** `mex_guest.rs` runs the same Gekko machine, unchanged. Its results change only in the corner cases of the
table (NaN propagation, a shift by 32..63, x/0, r0 in addic/subfic, addic Rc, stfs of an over-precise double, denormal
frC): every one of them was wrong before (it did not match Dolphin). Not re-run on m-ex mods here (vanilla scope).

`tests/ppc_oracle.rs` (plain mode vs gw_ppc.c) fails 4 of 6,000 cases both before and after this branch (checked by
building the unmodified interp.rs): pre-existing, not touched.

### 1.1 Paired singles (2026-11-27, port/slipcodes)

Gekko mode now runs opcode 4 and psq_l/psq_lu/psq_st/psq_stu (56, 57, 60, 61) and psq_lx/stx/lux/stux, as Dolphin's
Interpreter_Paired.cpp / Interpreter_LoadStorePaired.cpp have them: `Cpu::ps1` (each FPR's second slot, raw double bits)
and `Cpu::gqr` (GQR0..7). Every single-precision scalar op (lfs, the `s` arithmetic forms, fres, frsp) now also writes
ps1 (Dolphin's `Fill`), the double ones only ps0 (`SetPS0`); ps0 results are unchanged. ps_* arithmetic: NI_* NaN rules
and Force25Bit frC as the scalar forms, both results rounded to single; ps_res stored as computed, ps_rsqrte rounded to
single; ps_sel / merges / moves bit-exact. psq: GQR type 0 float (ConvertToDouble on load, **ConvertToSingleFTZ** on
store: a value below the single normal range stores as a signed zero, unlike stfs), 4/5/6/7 u8/u16/s8/s16 with the
dequantize/quantize scale tables, clamp then truncate; W = 1 loads ps1 = 1.0. Rc = 1 paired forms and the invalid GQR
types 1..3 decode / run as faults (no FPSCR is kept; the game has neither). The plain (gw_ppc.c) machine still faults on
all of them, and `Op::executable()` stays false for them (m-ex translation unchanged); the referee counts with
`gekko_executable()`.

Trust test (`tests/gekko_trust.rs`, extended): the Dolphin model got its own ps1 / GQR state and an opcode-4 executor
written from the Dolphin source (not from the interpreter), ps1 Fill/SetPS0 for the scalar forms, and 37 new forms (17
A-form, 12 X-form, 8 psq) with random ps1 values (single-valued half the time, related pairs), random valid GQRs (GQR0 =
0 half the time). Run: default 20,000 sets per form, **199 forms (162 + 37), 3,980,000 sets, 0 disagreements** outside
the kept addme/subfme carry (the scalar forms, now also compared on ps1, included). Not run at 2,000,000 per form in this
round. Census after the change: 1,767 Ps + 1 PsCmp + 1,013 Psq words decode as paired ops (PSMTX*/PSVEC*, THP, WriteMTXPS*, fn_80166A8C); dcbz_l (one word,
`__LCEnable`) stays Bad. The referee starts every run with the game's GQRs (`boot_gqr`: GQR2..5 = 0x00040004..0x00070007,
as `main`'s inlined OSInitFastCast sets them; GQR0 = 0).

## 2. The referee harness

| piece | where |
|---|---|
| DOL + symbols | `tools/mex-import/src/dol.rs` (header parse, `symbols.txt`, `Dol::find`: `$REFEREE_DOL`, else `orig/` of this checkout or the workspace's main `melee/`) |
| retail runner | `tools/mex-import/src/referee.rs` (`Referee::check`) |
| CLI | `mex-import referee --capture F[,F..] [--fns a,b] [--out DIR] [--cov-in C --cov-out C]`, `mex-import referee-plan --fns a,b` |
| capture | `rust/melee_core/src/referee.rs` (`referee_hook!`, feature `referee`; `melee --features referee`) |

**Guest memory.** Every DOL section at its retail address, read-only, kinds named from symbols.txt (text, constants,
.data/.sdata, .bss/.sbss as zeros). r2 = `_SDA2_BASE_` 0x804DF9E0, r13 = `_SDA_BASE_` 0x804DB6A0, so .sdata2 constants load
as the game loads them. At setup the DOL's static initialisers (`_ctors`, as `__init_cpp` runs them at boot) run once
with .data/.bss writable: MSL's `__sinit_trigf_c` fills `__four_over_pi_m1`, which sinf/cosf read (without it the first run
had 209 false sinf/cosf mismatches). A stack (0x81700000) and one region per group of the call's pointer arguments
(native addresses grouped by overlap, so aliasing such as `lbVector_Diff(a, b, a)` stays aliasing).

**A run.** Integer and pointer arguments go to r3.. in order, floats to f1.., LR = 0 (returning there ends the run). Code
is fetched from the text only; every callee inside the text is interpreted too (the retail tree, not the Rust one), and the
report names them; a call out of the text faults. A store outside the stack and the argument regions ends the run as "not a
leaf of the checked class" (not compared); a read of .data/.sdata/.bss/.sbss is listed per function ("mutable globals":
image values after the static initialisers, not the engine's). Budget 2,000,000 instructions.

**Compare.** Return value (r3, f1 as a single, or the returned pointer mapped back to the argument it names) and every
32-bit word of every argument group, bit-exact; a mismatch names the argument, the offset, both values and the frame.
Negative control: flipping one bit of a captured output or return value is reported as a mismatch (checked).

**Capture.** `referee_hook!(name, [P(ptr, bytes), I(int), F(float)...], v|i|f|p, call)` at the top of a Rust function
records the inputs (the bytes of every pointed-to f32 record before the call), calls the function itself (the nested entry
is not captured, nor are hooked callees of a captured call), then the bytes after and the return. Env:
`MELEE_REFEREE_OUT=<file>`, `MELEE_REFEREE_FNS=a,b`, `MELEE_REFEREE_MAX` (calls kept per function, default 400),
`MELEE_REFEREE_EVERY`. Without the feature the macro expands to nothing (normal builds unchanged). Captured replays still
match the console frame for frame (slpcheck "match" on all five below), so capturing does not disturb the run.

Hooked (30): lbVector_Normalize, NormalizeXY, Add, Add_xy, Sub, Diff, Angle, AngleXY, RotateAboutUnitAxis, Rotate,
Mirror, CosAngle, Lerp, 8000DE38, EulerAnglesFromONB, ApplyEulerRotation, sqrtf_accurate, CreateEulerMatrix, 8000E838;
atan2f, acosf, asinf, atanf, sinf, cosf; lbColl_80005C44, 80005EBC, 80005FC0, 80006094, 800067F8 (the capsule
distance / closest-point math of hit detection). Chosen with `referee-plan`: inputs are registers plus f32 records, and
the retail callee tree is pure and has no paired singles. Not hookable yet in this class: lbVector_CrossprodNormalized and
EulerAnglesFromPartialONB (retail calls PSVECCrossProduct, paired singles), lbColl_800077A0 / 80006E58 (PSMTXMultVec,
PSVECNormalize, PSMTXCopy).

## 3. Retail branch coverage

Every conditional branch (BO not "always") the interpreter executes is counted by outcome (taken / not taken). Per
checked function the report gives its own body's outcomes hit / total and its interpreted callees' outcomes;
`uncovered.tsv` lists each missing (address, outcome); `--cov-out` writes the cumulative counts and `--cov-in` reads them
back, so a run reports the outcomes it adds. "Done" for a function = every reachable outcome hit with 0 mismatches.

## 4. Numbers (5 online replays of the hero set, fast build)

Replays: Game_20251031T213013, 20260411T224941, 20260613T000702, 20260801T234358, 20260912T233947 (all `slpcheck`
"match" with capture on). Up to 2,000 calls kept per function per replay. 18 of the 30 hooked functions were reached.

| function | calls compared | mismatches | own outcomes | callee outcomes (interpreted) |
|---|---|---|---|---|
| lbVector_Normalize | 10,000 | 0 | 2/4 | - |
| lbVector_Add | 10,000 | 0 | 0/0 | - |
| lbVector_Sub | 536 | 0 | 0/0 | - |
| lbVector_Diff | 10,000 | 0 | 0/0 | - |
| lbVector_Angle | 5,343 | 0 | 6/10 | 26/28 (acosf, atanf) |
| lbVector_AngleXY | 1,555 | 0 | 5/10 | 26/28 (acosf, atanf) |
| lbVector_Rotate | 10,000 | 0 | 9/18 | - |
| lbVector_RotateAboutUnitAxis | 10,000 | 0 | 8/14 | - |
| lbVector_CreateEulerMatrix | 2,065 | 0 | 14/24 | - |
| lbVector_EulerAnglesFromONB | 268 | 0 | 2/6 | 33/40 (atan2f, asinf, lb_sqrtf, atanf) |
| lbColl_80005EBC | 5,538 | 0 | 4/4 | - |
| lbColl_80006094 | 156 | 0 | 40/102 | - |
| atan2f | 10,000 | 0 | 9/12 | 23/24 (atanf) |
| acosf | 10,000 | 0 | 3/4 | 23/24 (atanf) |
| asinf | 1,972 | 0 | 0/0 | 24/28 (lb_sqrtf, atanf) |
| atanf | 10,000 | 0 | 23/24 | - |
| sinf | 10,000 | 0 | 6/6 | (fabsf) |
| cosf | 10,000 | 0 | 6/6 | (fabsf) |
| **total** | **117,433** | **0** | 137/244 (+ "done": lbColl_80005EBC, sinf, cosf) | |

Not reached by these replays: lbVector_NormalizeXY, Add_xy, Mirror, CosAngle, Lerp, 8000DE38, ApplyEulerRotation,
sqrtf_accurate, 8000E838, lbColl_80005C44, 80005FC0, 800067F8 (some are reached only nested inside a captured call).
108 outcomes uncovered (62 of them in lbColl_80006094); "done" functions: lbColl_80005EBC, sinf, cosf (all outcomes, 0
mismatches), and the branch-free lbVector_Add/Sub/Diff. Referee time for the 117k calls: under 1 s.

**Mismatches: none.** The engine's Rust port of these 18 functions gave the retail result bit for bit on every compared
call.

## 5. Picking replays that reach code (layers 2 and 3; layer 1 not run in this round)

1. Server-side search (Lunar Melee filters: matchup, stage, characters) chooses what to download:
   `tools/rust-port/slp/lunar_fetch.py` (polite: searches under the rate limit, one bulk job per batch, back-off, no player
   names, git-ignored dirs). Not run in this round.
2. `tools/rust-port/referee/slpscan.py EXPR FILES|DIRS [--list]`: a local scan of the Post-Frame records (action, state
   frame, last attack landed, last hit by, percent change, position, ...) with a Python predicate per fighter and frame
   (`a`, the other port `o`, the previous frame `pa`), e.g. Zelda f-tilt connecting on a running Link:
   `"a.char==0x13 and a.last_attack==9 and pa and pa.last_attack!=9 and o and o.action==0x15"`. ~0.12 s a file
   (428 files in 50 s). Prints matching files and frames; `--list` feeds layer 3.
3. `tools/rust-port/referee/pick.py --melee <capture build> --mex <mex-import> --fns ... --work DIR REPLAYS`: per replay, the
   capture run (`slpcheck`, hidden, no input, adapter ignored, volume 3, below-normal) then the referee with
   `--cov-in/--cov-out`; a replay is kept only if it adds retail branch outcomes of the functions under check. The five
   replays above went through it: +88, +4, +14, +23, +8 outcomes, all kept.

The exact "what the target code saw" hook is the capture itself (the function's inputs at the call); the uncovered list of
section 3 is the target list.

## 6. How to run

```
# step 1 (light):
REFEREE_FUZZ=2000000 bash tools/rust-port/cargo_fast.sh test --profile fast -p mex_import --test gekko_trust -- --nocapture --test-threads 1
bash tools/rust-port/cargo_fast.sh test --profile fast -p mex_import --test dol_census -- --nocapture
# capture build (heavy):
bash tools/rust-port/cargo_fast.sh build --profile fast -p melee -p mex_import --features melee/referee
# one replay:
MELEE_REFEREE_OUT=_build/referee-mvp/cap.txt melee slpcheck --slp R.slp --full
mex-import referee --capture _build/referee-mvp/cap.txt --out _build/referee-mvp/report
# or the picker over many: python tools/rust-port/referee/pick.py --melee ... --mex ... --fns ... --work DIR R1.slp R2.slp ...
```
`REFEREE_TRACE=1` prints every retail instruction of a referee run (debugging the referee).

## 7. Limits

- Records: f32 records behind pointers only (Vec, Mtx, f32 out-params); native records with pointers (Fighter, Item,
  HSD_GObj, HitCapsule) need the in-process mode (next steps). So the audit's suspects (pub/port/dol-audit) are not
  reachable yet: ftCo_8008E9D0 / ftCo_8008DA4C, ftColl_8007A06C, ftCo_800A229C (Fighter / gobj records, many callees),
  grCorneria_801E08CC / 801E0A74, mpLib_80055E9C (stage gobjs, globals, HSD callees), HSD_MtxInverse / InverseConcat /
  InverseTranspose (Mtx-only arguments, but their retail tree calls PSMTXCopy / PSMTXIdentity: paired singles).
  Verdict for all: **unchecked** in this round.
- Paired singles are not in the interpreter: PSMTX*/PSVEC* callers are excluded.
- Mutable globals read by a checked function come from the boot image (after the static initialisers), not from the
  engine's state at the call; listed per function (here only MSL constants filled by `__sinit_trigf_c`, and
  `MSL_TrigF_80400774`, an initialised .data float).
- Rust writes outside the captured records are not seen; retail writes outside them end the run (counted, not compared).
- FPSCR assumed RN = 0, NI = 0.
- Slippi netplay codes (owner's step after this one): the 29 [affects-gameplay] injections of `list_netplay.json`
  (ssbm-asm d0ce23a) hook fighter, stage and player-init functions (UCF x8, NeutralSpawn, FreezeGlitchFix, Init Stage /
  Player Data, Castle wind, Kongo log, BrawlOffscreenDamage, FreezeDeadUpFallPhysics x3, FD DesyncProofBGTransformations,
  Stadium x4, WhispyBlowDirFix, NanaDeterminism, PSCameraIndependentMonitor, PreventWobbling x3). None is inside a function
  this harness can check yet, so applying them to the guest image is not built in this round: **all 29 unchecked**.

## 8. Next steps

1. Paired singles in Gekko mode (ps_* + psq_l/psq_st with GQR 0, Dolphin semantics, the same trust test): unlocks
   PSMTX/PSVEC, HSD_Mtx*, lbColl_800077A0 / 80006E58, lbVector_CrossprodNormalized.
2. (Built 2026-11-27, section 10.) In-process mode for native records: snapshot the records a call touches, run retail on the live objects through
   `remap::GuestSpace` (handles for pointers), restore, run Rust, compare field by field through the GC tables; plus a
   sync table of engine globals (p_ftCommonData etc., as mex_guest's `sync_globals`). That reaches ftCommon physics,
   ftColl, mpLib and the dol-audit suspects.
3. Slippi codes: apply the netplay list (04 writes, C2 hooks: a branch from the hooked address to the inserted block,
   which branches back; per Dolphin's GeckoCode.cpp) to the guest image, then gate against `slippi_np::online_codes`.
4. Use `uncovered.tsv` + slpscan + a small Lunar bulk job to find replays for the 108 missing outcomes.

## 9. Slippi codes on the guest image (2026-11-27, port/slipcodes; next-steps item 3, first half)

`tools/mex-import/src/gecko.rs`: a Dolphin game INI's enabled codes (`parse_ini`: `[Gecko_Enabled]` names, `[Gecko]`
bodies) applied to the DOL image (`apply`) as the Gecko handler does at boot: 00/02/04 writes, 06 strings, C2 inserts
(block copied to a new text section at 0x81600000, hook word = `b block`, block's last word = `b hook+4`; a hook
inserted twice keeps the later block). C0 / 08 / po-relative / conditional types are listed, not applied. Code bytes
are read at run time from the local ssbm-asm checkout; nothing is committed. `Referee::new(dol, syms)` takes the
patched image as it takes the plain one.

`tests/gecko_apply.rs` (local; skips without main.dol or ssbm-asm): Slippi's `Output/Netplay/GALE01r2.ini` at d0ce23a,
6 enabled groups (Required General / Recording / Online, Recommended Normal Lag Reduction / Apply Delay / Lagless FoD):
86 writes, 198 inserts, 0 skipped types, 5 writes outside the DOL sections (0x80452F54..64, in .bss: not applied, the
referee's .bss starts zero), 1 hook inserted twice (0x8016EA30, gm_Scene_Vs_OnExit). Every insert's hook branch and
return branch check out; all 29 [affects-gameplay] hooks are patched. 275 words inside the blocks do not decode (the
`blrl` data tables and strings Slippi's asm keeps inline).

Not done: running the patched functions against the engine. Every [affects-gameplay] hook sits in a function that takes
Fighter / Ground / gobj records or reads engine globals (ftCo_Turn_IASA, Fighter_procInput, grStadium_801D4548, ...),
which needs the native-records mode (section 8 item 2), not built in this round. Per-code verdicts: slippi-codes.md.


## 10. Native-records mode (2026-11-27, port/livereferee; next-steps item 2)

`rust/melee_engine/src/live_referee.rs`, feature `referee` (`melee --features referee`), on with
`MELEE_REFEREE_NATIVE=<report>`. A Rust function with `melee_core::referee_native!(name, [R(gobj, "HSD_GObj"), I(..),
F(..), P(mtx, 48)], v|i|f, call)` is checked inside the running match (slpcheck). Before the Rust call, the retail
function of the same name runs in the Gekko interpreter on the engine's live records. Then the Rust function runs and the
two are compared. In normal builds the macros expand to nothing.

**Guest image.** main.dol plus a Gecko list, applied the way Dolphin's handler applies it (`gecko::apply`). The default
list is the replay's own; `MELEE_REFEREE_GECKO=ini:<path>` takes a Dolphin INI (Slippi's netplay list), and `none`
gives vanilla. The boot initialisers run once on the image with .data/.bss writable: the static ctors (MSL's trig
tables) and `JObjInfoInit` (hsdJObj's methods). The bytes they write can then be read as constants.

**Records** are reached through `remap::GuestSpace`. The arguments get handles. Every pointer the retail code follows
registers the native object it names:
- gobj user_data by class: Fighter, Item, Ground;
- hsd_obj: the root JObj;
- native callbacks: by their retail address;
- the native hsdJObj: the retail one.

**Sync table.** Engine globals at their retail addresses, rebuilt before each check:
- Native records in place: player_slots (`_StaticPlayer`[6]), stage_info, StartMeleeRules (0x8046DB68).
- Native words, journaled: the RNG seed (0x804D5F90, the engine's current seed word) and the mpLib / mpColl scalar
  statics.
- Raw bytes in place: the 64 flags of Ground_804D6950.
- Read-only guest words: HSD_RandSeedPtr, and the record pointers p_ftCommonData, Fighter_804D64FC, it_804D6D28,
  mpLib_804D64B4, groundCollVtx/Line/Joint, jointListStart/End and Big Blue's yakumono_param.
- Read-only guest tables: ftPartsTable, HSD_GObjPLinkHead.
- Read-only, built from replay data:
  - the HSD pad queue (HSD_PadLibData plus the PADStatus ring at 0x8046B108), made from the replay's raw sticks; the
    UCF 0.7x/0.8 codes read it;
  - UCF 0.84's pad buffer: the engine's ftUcf_Pad, placed in the block of C2 8006B460.

**Compare.** `GuestSpace.journal` records every host byte that a store changes. After the retail run, the journal gives
what retail wrote, and it is then undone in reverse order. The Rust function then runs, and these are compared:
- the return value;
- every byte retail wrote;
- every byte Rust changed in an object retail reached, where retail did not write that byte;
- at a boundary callee (default `Fighter_ChangeMotionState`; set with `MELEE_REFEREE_BOUNDARY=a,b`), the call itself
  and its arguments. Retail's run ends at the callee's entry; Rust's state is taken at its own `referee_boundary!`.

**Not checkable.** Counted per function with the address and symbol; never counted as a pass:
- a read of .bss;
- a read of a .data object that is neither compiler-constant (@NNN, strings, words holding code addresses,
  `CONST_DATA`) nor written by the boot initialisers;
- a store to the image or to a read-only copy;
- an unmapped address, or a field with no native counterpart;
- an unsupported instruction, or running out of budget.

OS interrupt masking is stubbed, since it touches no game state. `MELEE_REFEREE_NATIVE_CALLEES=HSD_Randi,HSD_Randf` runs
those two as the engine's functions, with their seed writes journaled.

**Tools.**
- `MELEE_REFEREE_FNS`; `_MAX` (checks per function, default 2000); `_FROM` / `_TO` (frame window; referee frames are
  slippi + 123).
- `_LOG` prints one line per check.
- `_TRACE=<frame>` prints every retail instruction on that frame.
- `_NEGCTL` flips one retail byte (negative control). On the Fox replay it gave 143/168 mismatches: every call in which
  retail wrote a byte.

**Report lines.**
- `S`: one line per function: calls, compared, match, mismatch, notcheck, and own retail branch outcomes hit/total.
- `B`: boundary.
- `N`: not-checkable reasons.
- `M`: mismatches.
- `D`: .data read as constant.
- `G`: Gecko-area loads.
- `C`: per-branch counts.

**Hooked functions** (about 60):
- 25 ftCommon state functions: the Wait / Walk / Dash / Run / Squat / Turn / Fall / Landing / Guard / GuardOn IASAs,
  their Phys / Anim, Damage_OnEveryHitlag, DamageFall_IASA, 8008EC90, 80093240, 80099894, 8009A080, SquatRv_CheckInput,
  Jump_CheckInput;
- the Bob-omb's 24 state callbacks;
- lbColl_8000805C and lbColl_80007ECC (hurtbox bone positions plus the capsule test);
- HSD_JObjSetupMatrixSub and HSD_JObjMakeMatrix (the jobj fast_matrix path);
- grBigBlue_801EA05C and grBigBlue_801EBAF8;
- the functions Slippi's gameplay codes hook (slippi-codes.md section 7).

### 10.1 Numbers (fast build, `slpcheck --full`, headless; literal report output)

| run | functions | calls compared | mismatches | not checkable |
|---|---|---|---|---|
| lunar2 Peach ditto ...f1243f34 (its own 0.84 list), all hooks | 34 | 13,736 | 0 | 406 |
| six hero online replays, Slippi netplay INI (d0ce23a) | 39 | about 103,000 | 6 (5 DeadUpFall, 1 Walk) | about 87,000 (mostly camera .bss, stage, audio) |
| lunar2 Fox ditto ...f1239a6e (its own 0.8 list), Turn_IASA only, before / after fix 1 | 1 | 168 / 276 | 1 / 0 | 0 |

Every mismatch:

1. **Fox Turn_IASA**, frame 3918 (slippi 3795, the replay's hard desync). Retail, with the replay's UCF 0.8 dashback
   insert, sets fp+0x2340 = 1 and facing -1. The engine's 0.73 rule did neither. The engine was wrong. **Fixed in
   de86fec54.**
2. **ftCo_DeadUpFall_Phys** with the netplay list (FreezeDeadUpFallPhysics UpdateFallVelocity), hero
   Game_20260321T214020, frames 9994-9996. fp+0x2348 was 1 ULP over (0x3F4CCCCD against retail's 0x3F4CCCCC): the code
   stores a double with stfs, which truncates. **Fixed in f6a7f2166**; 94/94 calls match after the fix.
3. **ftCo_Walk_IASA**, hero Game_20251114T234228, frame 10526 (Samus). Retail calls ChangeMotionState(0x35) where Rust
   calls (0xD4). The trace shows retail's fn_800D952C reading fp->u.ss.x223C as non-null. This is a **referee
   artifact**, not an engine bug: Fighter_FighterVars is mapped as native 32-bit words at GC offsets, but Samus's
   variant has an 8-byte pointer at that place natively. The replay matches the console.

Earlier mismatches removed by sync fixes, not engine bugs:
- the Fall_IASA airdodge velocity, which came from sinf/cosf reading zeroed trig tables before the ctors ran;
- 0.84 Turn_IASA, before the pad buffer was synced;
- a wrong player_slots record name.

### 10.2 Hard-desync suspects

**(a) Bob-omb one frame early** (lunar2 ...f1243f34, slippi 3049-3053).

In the window, retail agrees with the engine on:
- all 42 lbColl_8000805C calls (item hitbox against fighter hurtbox: lb_8000B1CC bone positions from the hurtbox
  joints' matrices, then the capsule test);
- every compared HSD_JObjSetupMatrixSub call (1,354 in the window; 1,717 of the first 2,000 of the replay);
- the bomb's Anim / Phys callbacks.

So on the engine's state, neither the hurtbox math, nor the jobj fast_matrix path, nor the hit test disagrees with
retail.

Not checkable here:
- the thrown state's Coll (itBombhei_UnkMotion10_Coll). It stores into ECBSource's union, which is laid out natively as
  its first member, and that member holds pointers;
- 270 of the window's SetupMatrixSub calls: joints whose class method has no retail address (not hsdJObj).

Remaining candidates:
- the inputs to the bone matrices (the animation interpolation that feeds the joints' TRS);
- the thrown state's collision;
- the unchecked joints.

No fix.

**(b) Big Blue.** Only the drift case (3.19.1 ...72a57258) was run, and grBigBlue_801EA05C / 801EBAF8 are not
checkable yet:
- 801EA05C reaches Ground_804D6954 and other Ground .sbss statics through Ground_UpdateMapColl;
- 801EBAF8 hits a null at +0x44, even after yakumono_param was mapped.

The jump cases (...daec8, ...12d593) were not run. No verdict.

**(c) Yoshi's Story respawn slot** (hero Game_20251114T233428): not run. fn_8016758C / fn_80167638 work on gm's
`controller` block (0x8046B6A0); a sync entry for VsSceneState is needed first.

### 10.4 Regression set before and after the fixes (1556 replays, `hard_desync_sweep.sh`, JOBS 3-4)

`exe-base` is rust-port before de86fec54; `exe-after` has both fixes. Columns: `slpcheck` status and
`hard_desync.py`'s `first_hard` verdict. Only a frame number in `first_hard` is a real hard desync (Slippi would have
ended the game). `old` (below Slippi 3.0, or no end-of-frame state), `gap` (frames missing) and `error` (unreadable
raw block) are scorer markers, not desyncs.

| set | status | first_hard | before | after |
|---|---|---|---|---|
| ext | match | none | 125 | 125 |
| gang | match / diverged | old | 60 / 132 | 60 / 132 |
| gang | match / diverged | error | 14 / 1 | 14 / 1 |
| hero | match | none | 420 | 420 |
| hero | match / diverged | error | 6 / 1 | 6 / 1 |
| hero | diverged | real hard | 1 | 1 |
| lunar | match / diverged | none | 119 / 2 | 119 / 2 |
| lunar | diverged | gap | 1 | 1 |
| lunar2 | match / diverged | none | 457 / 83 | 467 / 74 |
| lunar2 | diverged | gap | 1 | 1 |
| lunar2 | diverged | real hard | 3 | 2 |
| v320 | match | none | 128 | 128 |
| v320 | diverged | real hard | 2 | 2 |

Totals: match 1329 -> 1339, diverged 227 -> 217. Real hard desyncs 6 -> 5. The other 216 rows with a non-empty
`first_hard` before (192 `old`, 22 `error`, 2 `gap`) are scorer markers, so "rows with first_hard set" is not a
hard-desync count; count only numeric `first_hard`.

Moved: 10 replays, all lunar2, all carrying the UCF 0.8 `DB.asm` insert (26 such replays in the set), all
diverged -> match, including Fox ...f1239a6e (real hard at 3800 -> matches its 8796 frames). Nothing else moved
in any column. Of the 26 UCF 0.8 replays, 23 now match; 3 still diverge (lunar mewtwo/...1245f0c at 8965, lunar2
6ab9717e...ba55 at 6045, ...baeb at 4040), cause not looked at. Fix 2 (DeadUpFall stfs) moved no replay in this set; its proof is the
referee's 94/94 calls (10.1).

Tiers (regress-set.md, "Replay tiers"): the 192 `old` rows (all gang, pre-3.0) are tier 2, pinned; they are not
open bugs. Tier-1 hard rows after port/hardfix (e44ef86cf): 4, of which Yoshi's Story hero Game_20251114T233428 is a
recording artifact (retail Dolphin reproduces the engine) and 3 are Big Blue (open).

### 10.3 Limits and next steps

- **Per-kind unions** (Fighter_FighterVars / MotionVars / ItemVars, ECBSource_x4): the word mapping is right only for
  members without pointers. Next step: a union-member hook in remap that picks the variant record by the owner's kind or
  tag.
- **Unsynced globals** cause most not-checkable checks:
  - the camera (0x80452C68, 0x80452F24);
  - Fighter_804D653C;
  - the audio driver;
  - ftData_SpecialS (.data read through the kind table);
  - Fighter.dat_attrs (a void*: needs the kind's attribute record);
  - the gm scene block;
  - Ground's .sbss.
- Rust-only writes are seen only in objects that retail reached.
- Writes after a boundary are not compared.
