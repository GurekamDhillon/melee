# Coverage map: which retail functions run in Slippi games, and which the live referee checks (2026-11-28, port/covmap)

Vanilla NTSC 1.02, rust-port e44ef86cf. No replays, disc bytes or player tags are in this note or in git. The raw
per-function table is `_build/agents/covmap/final/functions.csv` (6,775 rows, 1.2 MB, not committed); the tools are
`tools/rust-port/covmap/`.

## 1. Method

**Executed functions (exact call counts).** The existing coverage build (`tools/rust-port/covtest/build_cov.sh`, LLVM
source-based coverage in the six game crates; notes/coverage-testing.md), built with `--features referee` so one exe
serves both passes. In the `llvm-cov export` JSON every function's `count` is its entry count, inlined copies
included. `fncounts.py` names each Rust function by the `fn NAME` on its first line and keeps the names that are
retail functions in `config/GALE01/symbols.txt` (decomp-named TU ports, seams and `ext_fns!` targets all carry the
retail name). Rust-only helpers (decomp static inlines, closures) are dropped: their calls are inside a retail
function. So "executed" = the Rust port of that retail function ran; it assumes the port has the retail call
structure (section 6).

**Families** come from the retail address: `config/GALE01/splits.txt` gives each function's TU (e.g.
`melee/ft/kinds/ftFox/...` -> character: Fox, which also holds Falco's shared code). The "render" family
(dobj/tobj/mobj/pobj/tev/state/displayfunc, GX, ftdrawcommon, itdraw, ...) and "audio" write no sim state, so the
summary also gives a sim-only view without them.

**Checked functions.** The same exe with `MELEE_REFEREE_NATIVE` (notes/referee.md section 10; replay's own Gecko list,
default `MELEE_REFEREE_MAX` 2000 per function per replay). A function is checked when its `S` line has calls; verdict:
mismatch if any compared call mismatched, match if compared > 0 and none did, not checkable if every call was. The
`C` lines (retail branch outcomes the interpreter executed) mark unhooked functions whose retail code ran as an
interpreted callee of a checked call ("checked transitively": their effect is inside a compared result, but they are
not compared on their own).

**Replays.** `pick.py` over `scratch/replayset/regress.tsv`, tier 1 only (Slippi >= 3.0: 1,342 of 1,556): a greedy
cover of every internal character (FighterKind: Sheik/Zelda, Popo/Nana), every stage present, every item kind seen
(Item Update events) and the 3-4 player games, then quotas of 20 per legal stage and 6 per character, `match` replays
first: **156 replays**, covering all 148 content elements present in the set. Count pass: all 156 (151 usable).
Referee pass: the first 52 in pick order (48 reports), stopped there for time (about 4 min a replay at 4 jobs; the
cover replays come first, so the 48 hold most of the content). Runs: `run.sh` (`MODE=count` / `MODE=referee`),
headless, no input, adapter ignored, volume 3, below-normal, 4 jobs, paused while the owner plays.

```
python -I tools/rust-port/covmap/meta.py  scratch/replayset/regress.tsv  _build/agents/covmap/meta.tsv
python -I tools/rust-port/covmap/pick.py  _build/agents/covmap/meta.tsv scratch/replayset/regress.tsv 200 _build/agents/covmap/subset.tsv
COV_TARGET=<private dir> bash tools/rust-port/covtest/build_cov.sh -j 8 --features referee
EXE=<cov exe> bash tools/rust-port/covmap/run.sh subset.tsv RUN;  MODE=referee ... run.sh subset.tsv RUN
python -I tools/rust-port/covmap/agg.py RUN subset.tsv OUT    # functions.csv, summary.txt, hot_unchecked.md, family_top10.md
```

## 2. Summary

| | functions | % | calls | % |
|---|---:|---:|---:|---:|
| retail functions executed (151 replays) | **6,775** | 34.2% of the DOL's 19,827 | **83.93 G** | |
| checked live, match | 28 | 0.41% | 317.1 M | 0.38% |
| checked live, mismatch | 16 | 0.24% | 16.4 M | 0.02% |
| hooked, not checkable (every call) | 8 | 0.12% | 2.3 M | 0.00% |
| capture-mode hook only (`referee_hook!`, offline; not live) | 26 | 0.38% | 3,217.7 M | 3.83% |
| unchecked | 6,697 | 98.85% | 80,376.6 M | 95.77% |
| **coverage (compared at least once)** | **44** | **0.65%** | **333.5 M** | **0.40%** |
| coverage, sim view (render + audio dropped: 6,344 functions, 48.05 G calls) | 43 | 0.68% | | 0.66% |
| unhooked, ran as an interpreted callee of a checked call | 190 | 2.8% | 5.09 G | 6.06% |

Of the 74 `referee_native!` hooks in the source, 52 are executed retail functions here; 20 are Bob-omb states (no
items in these replays), plus HSD_JObjMakeMatrix and ftColl_8007925C (the latter does report referee calls, so its
hook sits in a Rust fn whose name the count pass does not match; section 6). Not-checkable calls, by reason (all hooks, 369,733 calls): .bss reads 41.8% (mostly
mpIsland/Ground statics and gm's scene block), the current CObj (`HSD_CObjGetCurrent`, camera) 24.5%, faulted reads
16.8% (spline records: `splArcLengthPoint`), `Fighter+0x2d4` (dat_attrs, a void*) 7.9%, `HSD_AObj` allocator .bss
stores 2.8%, `selected_stage` 2.2%, `HSD_Mtx` allocator .bss 2.1%, AXDriver 1.0%.

### Calls per family

| family | executed | calls | checked (compared) | checked share of calls |
|---|---:|---:|---:|---:|
| HSD scene graph + animation | 155 | 33,883,113,817 | 1 | 0.8% |
| render (no sim state) | 403 | 32,783,650,345 | 1 | 0.1% |
| math (MSL, lbvector, PSMTX) | 46 | 6,101,874,796 | 0 | 0.0% |
| audio | 28 | 3,094,652,118 | 0 | 0.0% |
| HSD misc | 68 | 1,845,367,685 | 0 | 0.0% |
| collision/mp | 261 | 1,470,712,243 | 3 | 1.3% |
| fighter core | 341 | 1,266,112,392 | 3 | 0.3% |
| stages | 732 | 769,010,200 | 3 | 0.0% |
| lb misc | 173 | 673,905,319 | 0 | 0.0% |
| camera | 54 | 396,048,613 | 0 | 0.0% |
| HUD (if) | 101 | 279,939,230 | 0 | 0.0% |
| common fighter states | 811 | 265,382,722 | 25 | 0.8% |
| hit/damage | 96 | 253,971,885 | 5 | 3.7% |
| player (pl) | 183 | 246,686,379 | 0 | 0.0% |
| modes (gm/mn) | 148 | 162,095,259 | 0 | 0.0% |
| effects/particles | 87 | 159,595,760 | 0 | 0.0% |
| gobj/proc | 27 | 149,544,596 | 0 | 0.0% |
| RNG | 4 | 42,320,008 | 0 | 0.0% |
| items: core | 201 | 30,942,243 | 0 | 0.0% |
| SDK misc | 6 | 25,524,736 | 0 | 0.0% |
| unknown TU | 1 | 13,343,752 | 0 | 0.0% |
| items: per-kind (it/kinds) | 881 | 5,631,763 | 3 | 0.0% |
| character: Kirby | 144 | 3,943,614 | 0 | 0.0% |
| character: Purin | 60 | 2,473,829 | 0 | 0.0% |
| other | 7 | 1,543,902 | 0 | 0.0% |
| character: Peach | 96 | 1,176,073 | 0 | 0.0% |
| character: Fox | 156 | 617,058 | 0 | 0.0% |
| character: Koopa | 76 | 157,935 | 0 | 0.0% |
| character: Captain | 73 | 106,486 | 0 | 0.0% |
| character: Seak | 126 | 91,513 | 0 | 0.0% |
| character: Mario | 54 | 64,235 | 0 | 0.0% |
| character: Luigi | 81 | 62,072 | 0 | 0.0% |
| character: Link | 90 | 61,874 | 0 | 0.0% |
| character: Mewtwo | 83 | 52,907 | 0 | 0.0% |
| character: Donkey | 58 | 49,296 | 0 | 0.0% |
| character: Pikachu | 116 | 47,406 | 0 | 0.0% |
| character: GameWatch | 138 | 45,404 | 0 | 0.0% |
| character: Mars | 85 | 45,108 | 0 | 0.0% |
| character: Ness | 125 | 39,644 | 0 | 0.0% |
| character: Yoshi | 105 | 38,657 | 0 | 0.0% |
| character: Samus | 81 | 36,518 | 0 | 0.0% |
| character: Popo | 71 | 35,759 | 0 | 0.0% |
| character: MasterHand | 1 | 11,566 | 0 | 0.0% |
| character: Zelda | 75 | 9,745 | 0 | 0.0% |
| character: DrMario | 12 | 3,400 | 0 | 0.0% |
| character: Falco | 4 | 1,856 | 0 | 0.0% |
| character: CLink | 16 | 1,704 | 0 | 0.0% |
| character: Nana | 24 | 1,175 | 0 | 0.0% |
| character: Pichu | 4 | 484 | 0 | 0.0% |
| character: Ganon | 3 | 462 | 0 | 0.0% |
| character: Emblem | 4 | 358 | 0 | 0.0% |

### Hooked functions: verdicts (48 referee reports)

| function | verdict | referee calls | compared | mismatch | not checkable | top not-checkable reason |
|---|---|---:|---:|---:|---:|---|
| Fighter_procAnim | match | 91552 | 5998 | 0 | 85554 | read .bss X (...bss.0+0x0) at gecko-area+0x255c |
| HSD_JObjSetupMatrixSub | match | 96000 | 88528 | 0 | 7472 | read .bss X (HSD_Mtx_X+0x0) at HSD_ObjAlloc+0x14 |
| fn_800DB790 | match | 83 | 83 | 0 | 0 |  |
| fn_800DBAE4 | match | 412 | 412 | 0 | 0 |  |
| ftCo_80093240 | match | 6554 | 6554 | 0 | 0 |  |
| ftCo_80099894 | match | 36 | 34 | 0 | 2 | read faulted X (FighterPartsTable+0x4) at ftParts_GetBoneInd |
| ftCo_8009A080 | match | 2961 | 2961 | 0 | 0 |  |
| ftCo_Damage_OnEveryHitlag | match | 76237 | 76237 | 0 | 0 |  |
| ftCo_Dash_Anim | match | 39219 | 39219 | 0 | 0 |  |
| ftCo_Dash_Phys | match | 39559 | 39559 | 0 | 0 |  |
| ftCo_DeadUpFall_Anim | match | 516 | 512 | 0 | 4 | store to Bss X (...bss.0+0xd) at ifStatus_X+0x20 |
| ftCo_DeadUpFall_Phys | match | 512 | 512 | 0 | 0 |  |
| ftCo_Fall_Phys | match | 34930 | 34027 | 0 | 903 | read .bss X (lbl_X+0x84) at fn_X+0x10 |
| ftCo_GuardOn_IASA | match | 11693 | 11666 | 0 | 27 | store to Text X (gecko-area+0xb08) at gecko-area+0xb24 |
| ftCo_Guard_IASA | match | 19532 | 19510 | 0 | 22 | store to Text X (gecko-area+0xb08) at gecko-area+0xb24 |
| ftCo_Jump_CheckInput | match | 26019 | 26019 | 0 | 0 |  |
| ftCo_Run_Phys | match | 8565 | 8565 | 0 | 0 |  |
| ftCo_SquatRv_CheckInput | match | 5373 | 5266 | 0 | 107 | store to Text X (gecko-area+0x5b0) at gecko-area+0x5cc |
| ftCo_Turn_Anim | match | 11917 | 11917 | 0 | 0 |  |
| ftCo_Turn_Phys | match | 11984 | 11984 | 0 | 0 |  |
| ftCo_Wait_Phys | match | 42900 | 42900 | 0 | 0 |  |
| ftCo_Walk_Phys | match | 18966 | 18966 | 0 | 0 |  |
| ftDrawCommon_80080E18 | match | 95240 | 4733 | 0 | 90507 | read .bss X (current+0x0) at HSD_CObjGetCurrent+0x0 |
| itBombhei_UnkMotion10_Anim | match | 92 | 86 | 0 | 6 | read faulted X (X) at itBombhei_UnkMotion10_Anim+0x48 |
| itBombhei_UnkMotion10_Phys | match | 92 | 92 | 0 | 0 |  |
| itBombhei_UnkMotion8_Anim | match | 49 | 46 | 0 | 3 | read faulted X (X) at itBombhei_UnkMotion8_Anim+0x68 |
| lbColl_80007ECC | match | 27841 | 27841 | 0 | 0 |  |
| mpBoundingCheck | match | 95012 | 95012 | 0 | 0 |  |
| ftCo_8008EC90 | mismatch | 4403 | 638 | 213 | 3765 | read .bss X (AXDriver_X+0x0) at HSD_AudioSFXKeyOffTrack+0x1c |
| ftCo_DamageFall_IASA | mismatch | 15219 | 15059 | 7 | 160 | read faulted X (Fighter+0x2d4) at ftMs_SpecialAirS_Enter+0x1 |
| ftCo_Dash_IASA | mismatch | 39093 | 39076 | 2 | 17 | read faulted X (Fighter+0x2d4) at ftFx_SpecialAirHiStart_Ent |
| ftCo_Fall_IASA | mismatch | 35914 | 35675 | 6 | 239 | read faulted X (Fighter+0x2d4) at ftFx_SpecialAirLw_Enter+0x |
| ftCo_Landing_IASA | mismatch | 46414 | 46404 | 12 | 10 | read faulted X (Fighter+0x2d4) at ftMt_SpecialN_Enter+0x3c |
| ftCo_Run_IASA | mismatch | 8444 | 8440 | 2 | 4 | read faulted X (Fighter+0x2d4) at ftKp_SpecialAirHi_Enter+0x |
| ftCo_Squat_IASA | mismatch | 5650 | 5638 | 7 | 12 | read faulted X (Fighter+0x2d4) at ftLg_SpecialLw_Enter+0x30 |
| ftCo_Turn_IASA | mismatch | 11416 | 11411 | 1 | 5 | read faulted X (Fighter+0x2d4) at ftFx_SpecialAirHiStart_Ent |
| ftCo_Wait_IASA | mismatch | 52005 | 51930 | 20 | 75 | read faulted X (Fighter+0x2d4) at ftMt_SpecialN_Enter+0x3c |
| ftCo_Walk_IASA | mismatch | 18832 | 18782 | 2 | 50 | read .data X (ftMapping_list+0x2c) at Player_X+0x78 |
| grBigBlue_801EBAF8 | mismatch | 2000 | 1997 | 1994 | 3 | read .bss X (...bss.0+0x8) at mpIsland_X+0x2c |
| grBigBlue_801EC58C | mismatch | 2000 | 2000 | 1999 | 0 |  |
| grStadium_801D2344 | mismatch | 6000 | 6000 | 6 | 0 |  |
| lbColl_8000805C | mismatch | 90640 | 90608 | 36 | 32 | read .bss X (HSD_Mtx_X+0x0) at HSD_ObjAlloc+0x14 |
| mpCheckMultiple | mismatch | 16277 | 16277 | 62 | 0 |  |
| mpLib_80055E9C | mismatch | 80631 | 4813 | 83 | 75818 | read .bss X (...bss.0+0x8) at mpIsland_X+0x2c |
| ftCo_800AC5A0 | not checkable | 440 | 0 | 0 | 440 | store faulted X (Fighter+0x1fdc) at ftCo_X+0x6c |
| ftNn_Init_80123954 | not checkable | 22 | 0 | 0 | 22 | read faulted X (Fighter+0x2d4) at ftNn_Init_X+0x38 |
| grBigBlue_801EA05C | not checkable | 2000 | 0 | 0 | 2000 | read faulted X (X) at grDatFiles_X+0x40 |
| grLast_8021AAB0 | not checkable | 8000 | 0 | 0 | 8000 | read .data X (selected_stage+0x0) at Stage_X+0x0 |
| grStadium_801D4548 | not checkable | 6000 | 0 | 0 | 6000 | read .bss X (...bss.0+0x0) at gm_GetCurrentGameMode+0x8 |
| itBombhei_UnkMotion10_Coll | not checkable | 91 | 0 | 0 | 91 | store to Bss X (mpColl_X+0x0) at mpCollPrev+0x2c |
| splArcLengthPoint | not checkable | 57901 | 0 | 0 | 57901 | read faulted X (X) at splArcLengthGetParameter+0xa0 |
| splGetSplinePoint | not checkable | 2000 | 0 | 0 | 2000 | read faulted X (X) at splGetSplinePoint+0x2b8 |

### Mismatches: what they are

Every mismatch but one group comes from replays the engine plays frame-exact against the console (slpcheck "match"),
so a wrong engine result would have had to stay invisible in Post-Frame. Classes:

1. **ChangeMotionState into a character special state** (ftCo_Wait / Walk / Dash / Run / Turn / Squat / Fall /
   Landing / DamageFall IASA, 59 calls): retail's boundary call names a special-state id (0x15C..0x18F) where Rust
   goes to a common state, or the reverse, and `Fighter+0x2200..0x234c` differ. The same replays' not-checkable
   reason is `Fighter+0x2d4` (dat_attrs) in `ft*_Special*_Enter`. Same class as referee.md 10.1 item 3 (Samus): the
   per-kind FighterVars / dat_attrs mapping. **Likely referee artifact.**
2. **ftCo_8008EC90** (213 of 638 compared, in 40 of the 48 replays, all of them console-exact):
   `Fighter+0x408` / `+0x410` (Rust writes 1 / a float, retail leaves 0); its usual not-checkable reason is the
   AXDriver .bss (an SFX call in its tree). A systematic difference in a field Post-Frame does not carry, or a sync
   gap; not looked at. **First to explain** (cheap: one function, many reproductions).
3. **lbColl_8000805C** (36 of 90,608, one hero replay that matches the console, frames 244-245): hurt capsule
   world positions (`HurtCapsule+0x28..0x30`) differ in the low mantissa bits. Either a real FP difference that does
   not reach Post-Frame, or bone matrices that differ from the retail run's (same lead as referee.md 10.2 (a)).
   **Worth a look.**
4. **grStadium_801D2344** (2-3 calls in every Pokemon Stadium replay, same frames, e.g. 724): Rust writes
   `Ground+0xf0` / `+0x100`, retail does not. Not looked at (Stadium transformation timing; the replays match).
5. **Big Blue, v320 ...12d593** (the "jump" case of referee.md 10.2 (b); slpcheck diverged at frame 2332):
   grBigBlue_801EBAF8 (1,994 of 1,997: Rust leaves `HSD_JObj+0x3c` and `Ground+0xcc` zero where retail writes
   values from frame 1), grBigBlue_801EC58C (1,999 of 2,000: retail returns -FLT_MAX, Rust -24.99), mpLib_80055E9C
   (83: CollLine flag bits) and mpCheckMultiple (62, a return value from frame 285), all in this one replay. The
   first two read Ground .sbss that the sync table does not carry yet, so this may be sync; it is also the first
   referee result on the jump case. **Lead for the Big Blue hard desync, unproven.**

## 3. Hot unchecked: the 200 executed-but-unchecked functions with the most calls (sim view)

Render and audio are left out (they write no sim state; with them, 98 render functions and 1 audio function would take places in the 200).
`trigger` is the content common to every replay that ran the function ("all" = every replay; "mixed" = no common
element). Character action callbacks are far down by calls (they run only in their states), so they get their own
table after this one.

**HSD scene graph + animation** (63 of the 200, 33,554,200,464 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_FObjSetState | 10,732,090,990 | 151 | all |
| HSD_FObjGetState | 7,195,105,741 | 151 | all |
| parseFloat | 3,428,171,403 | 151 | all |
| HSD_AObjInterpretAnim | 1,660,344,705 | 151 | all |
| HSD_MtxInverseTranspose | 926,691,366 | 151 | all |
| HSD_MtxScaledAdd | 902,769,670 | 151 | all |
| HSD_FObjInterpretAnim | 814,415,044 | 151 | all |
| FObjUpdateAnim | 814,293,856 | 151 | all |
| HSD_JObjDispAll | 683,989,897 | 151 | all |
| JObjUpdateFunc | 647,883,508 | 151 | all |
| HSD_JObjCheckDepend | 513,084,723 | 151 | all |
| HSD_JObjAnim | 513,084,723 | 151 | all |
| HSD_JObjClearFlags | 364,877,383 | 151 | all |
| HSD_JObjSetMtxDirtySub | 362,818,591 | 151 | all |
| HSD_FObjInterpretAnimAll | 345,176,615 | 151 | all |
| HSD_JObjUnref | 303,049,947 | 151 | all |
| HSD_JObjSetCurrent | 298,540,292 | 151 | all |
| JObjAnimAll | 290,515,516 | 151 | all |
| HSD_JObjClearFlagsAll | 264,357,253 | 151 | all |
| HSD_ObjAlloc | 235,322,380 | 151 | all |
| HSD_ObjFree | 235,048,055 | 151 | all |
| parsePackInfo | 231,538,917 | 151 | all |
| HSD_JObjSetFlags | 194,138,410 | 151 | all |
| HSD_FObjGetAllocData | 119,617,332 | 151 | all |
| HSD_AObjReqAnim | 117,674,342 | 151 | all |
| HSD_FObjRemoveAll | 87,564,079 | 151 | all |
| HSD_FObjAlloc | 59,853,219 | 151 | all |
| HSD_FObjRemove | 59,763,962 | 151 | all |
| HSD_FObjFree | 59,763,962 | 151 | all |
| HSD_JObjSetFlagsAll | 59,000,396 | 151 | all |
| HSD_AObjSetRate | 58,752,598 | 151 | all |
| HSD_JObjGetFlags | 58,645,408 | 151 | all |
| HSD_AObjGetAllocData | 55,639,495 | 151 | all |
| HSD_AObjRemove | 54,812,877 | 151 | all |
| EulerToQuat | 47,641,192 | 151 | all |
| HSD_JObjReqAnimByFlags | 43,871,337 | 151 | all |
| HSD_JObjReqAnimAllByFlags | 43,708,170 | 151 | all |
| HSD_JObjRemoveAnimByFlags | 41,098,407 | 151 | all |
| HSD_JObjRemoveAnimAllByFlags | 41,096,692 | 151 | all |
| JObjForeachAnim | 37,973,887 | 151 | all |
| DObjForeachAnim | 37,390,949 | 151 | all |
| RObjForeachAnim | 37,386,926 | 151 | all |
| GetMemoryEntry | 35,126,397 | 151 | all |
| HSD_AObjInitEndCallBack | 30,249,928 | 151 | all |
| HSD_AObjInvokeCallBacks | 30,249,928 | 151 | all |
| HSD_AObjSetFlags | 29,998,712 | 151 | all |
| TObjForeachAnim | 28,861,075 | 151 | all |
| HSD_AObjSetRewindFrame | 27,839,428 | 151 | all |
| HSD_AObjSetEndFrame | 27,839,428 | 151 | all |
| HSD_AObjSetFObj | 27,839,378 | 151 | all |
| HSD_AObjAlloc | 27,839,378 | 151 | all |
| HSD_AObjFree | 27,800,117 | 151 | all |
| HSD_JObjAnimAll | 26,903,344 | 151 | all |
| HSD_FObjLoadDesc | 25,020,393 | 151 | all |
| callbackForeachFunc | 20,574,794 | 151 | all |
| hsdAllocMemPiece | 17,570,876 | 151 | all |
| HSD_JObjReqAnimAll | 16,256,613 | 151 | all |
| hsdFreeMemPiece | 16,226,516 | 151 | all |
| HSD_FObjStopAnim | 13,015,170 | 151 | all |
| HSD_MkRotationMtx | 12,590,997 | 151 | all |
| HSD_AObjLoadDesc | 11,561,645 | 151 | all |
| QuatSlerp | 11,175,639 | 151 | all |
| FObjLaunchKeyData | 11,096,493 | 151 | all |

**math (MSL, lbvector, PSMTX)** (24 of the 200, 6,064,294,735 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| PSMTXConcat | 2,318,386,333 | 151 | all |
| sinf | 1,348,843,617 | 151 | all |
| cosf | 1,301,936,504 | 151 | all |
| PSMTXMultVec | 181,514,337 | 151 | all |
| lbVector_Normalize | 159,254,298 | 151 | all |
| PSVECCrossProduct | 97,456,498 | 151 | all |
| acosf | 97,206,228 | 151 | all |
| lbVector_Angle | 87,353,934 | 147 | mixed |
| lbVector_RotateAboutUnitAxis | 76,408,462 | 144 | mixed |
| PSMTXRotAxisRad | 65,891,108 | 151 | all |
| lbVector_CreateEulerMatrix | 57,583,858 | 115 | mixed |
| lbVector_Rotate | 48,090,992 | 151 | all |
| PSVECNormalize | 36,167,397 | 151 | all |
| PSMTXCopy | 33,447,275 | 151 | all |
| lbVector_Add | 21,562,407 | 151 | all |
| MTXPerspective | 21,504,020 | 151 | all |
| PSVECMag | 19,285,265 | 151 | all |
| C_MTXLookAt | 19,198,982 | 151 | all |
| PSMTXQuat | 16,544,925 | 150 | mixed |
| PSMTXScale | 14,312,448 | 151 | all |
| PSMTXTrans | 12,051,695 | 115 | mixed |
| lbVector_Diff | 11,046,930 | 151 | all |
| PSMTXMultVecSR | 9,708,620 | 151 | all |
| PSVECDotProduct | 9,538,602 | 151 | all |

**HSD misc** (11 of the 200, 1,766,026,100 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_PerfCountEnvelopeBlending | 1,074,770,629 | 151 | all |
| HSD_SListGetAllocData | 163,063,485 | 151 | all |
| HSD_GObj_JObjCallback | 130,270,163 | 151 | all |
| HSD_SListAlloc | 81,627,622 | 151 | all |
| HSD_SListRemove | 81,435,863 | 151 | all |
| HSD_SListAllocAndPrepend | 81,390,277 | 151 | all |
| HSD_SListPrependList | 81,390,277 | 151 | all |
| HSD_GObj_LObjCallback | 35,970,048 | 151 | all |
| HSD_PadClampCheck1 | 12,684,588 | 151 | all |
| hsd_80391A04 | 12,146,004 | 151 | all |
| HSD_SisLib_803A84BC | 11,277,144 | 151 | all |

**collision/mp** (18 of the 200, 1,287,009,888 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| mpLineIntersection | 453,991,203 | 148 | mixed |
| mpRemap2d | 273,717,370 | 151 | all |
| mpCheckedBounding | 130,559,983 | 151 | all |
| mpLineIntersectionV | 82,042,153 | 74 | mixed |
| mpLineGetNext | 48,699,847 | 151 | all |
| mpLineGetPrev | 48,622,682 | 151 | all |
| mpLib_8004ED5C | 48,145,724 | 151 | all |
| mpLineIntersectionH | 44,033,965 | 148 | mixed |
| mpCheckLeftWallRemap | 23,287,379 | 151 | all |
| mpCheckRightWallRemap | 23,287,379 | 151 | all |
| mpCheckRightWall | 17,748,558 | 151 | all |
| mpCheckLeftWall | 17,747,599 | 151 | all |
| mpCheckFloor | 16,829,776 | 151 | all |
| mpUncheckBounding | 13,263,725 | 151 | all |
| mpLib_800511A4_RightWall | 12,618,204 | 151 | all |
| mpLib_800515A0_LeftWall | 12,618,204 | 151 | all |
| mpLib_80054ED8 | 10,368,824 | 151 | all |
| mpBoundingCheck2 | 9,427,313 | 151 | all |

**fighter core** (5 of the 200, 1,134,293,491 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_JObjSetMtxDirty | 691,920,654 | 151 | all |
| ftParts_PObjSetupMtx | 268,970,096 | 151 | all |
| ftParts_80074D7C | 79,090,981 | 151 | all |
| ftParts_800750C8 | 75,449,408 | 151 | all |
| ftParts_80074B6C | 18,862,352 | 151 | all |

**stages** (16 of the 200, 657,340,315 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| Ground_801C2D24 | 237,817,540 | 151 | all |
| Stage_GetCamBoundsBottomOffset | 88,713,240 | 151 | all |
| Stage_GetCamBoundsLeftOffset | 49,265,601 | 151 | all |
| Stage_GetCamBoundsRightOffset | 49,199,924 | 151 | all |
| Stage_GetCamBoundsTopOffset | 47,963,289 | 151 | all |
| Ground_801C4368 | 33,458,393 | 150 | mixed |
| Ground_801C2C8C | 25,533,459 | 151 | all |
| grIzumi_801CD280 | 21,975,367 | 22 | stage:Fountain of Dreams |
| grStory_801E36D8 | 19,420,950 | 26 | item:Heiho;stage:Yoshi's Story |
| Ground_801C0498 | 17,626,694 | 151 | all |
| Stage_GetBlastZoneBottomOffset | 16,386,105 | 151 | all |
| Ground_801C4640 | 10,885,396 | 151 | all |
| Stage_GetBlastZoneLeftOffset | 10,107,048 | 151 | all |
| Stage_GetBlastZoneRightOffset | 10,055,059 | 151 | all |
| Stage_GetBlastZoneTopOffset | 9,781,271 | 151 | all |
| Stage_UnkSetVec3TCam_Offset | 9,150,979 | 151 | all |

**lb misc** (16 of the 200, 583,240,521 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| lb_8000B1CC | 216,802,492 | 151 | all |
| lbGetFreeColorRegister | 48,042,796 | 151 | all |
| lbGetFreeColorRegImpl | 46,637,968 | 151 | all |
| lb_8000B4FC | 23,073,347 | 151 | all |
| lbLang_IsSavedLanguageUS | 22,545,595 | 151 | all |
| lbFileGetFullName | 22,376,336 | 151 | all |
| lbDvd_8001819C | 22,372,939 | 151 | all |
| lbDvd_GetPreloadedArchive | 22,371,088 | 151 | all |
| lbDvd_800187F4 | 22,370,797 | 151 | all |
| lbDvd_800189EC | 22,370,797 | 151 | all |
| lb_80014258 | 22,249,114 | 151 | all |
| lbAnim_8001E6D8 | 21,757,300 | 151 | all |
| lb_80019A30 | 17,985,024 | 151 | all |
| lb_8000C490 | 17,824,703 | 151 | all |
| lbCopyJObjSRT | 17,569,666 | 151 | all |
| lbAnim_InitFrames | 16,890,559 | 151 | all |

**camera** (8 of the 200, 327,044,943 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| Camera_8003108C | 107,257,552 | 151 | all |
| Camera_80030B24 | 59,509,383 | 98 | mixed |
| Camera_80031060 | 39,630,351 | 151 | all |
| Camera_80029124 | 32,601,635 | 150 | mixed |
| Camera_80030A78 | 31,424,324 | 151 | all |
| Camera_80030AC4 | 28,608,739 | 151 | all |
| Camera_8002928C | 18,626,315 | 150 | mixed |
| Camera_8002AF68 | 9,386,644 | 151 | all |

**hit/damage** (3 of the 200, 237,497,061 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_JObjSetupMatrix | 189,795,833 | 151 | all |
| lbColl_800083C4 | 38,579,157 | 151 | all |
| lbColl_80006E58 | 9,122,071 | 145 | mixed |

**common fighter states** (5 of the 200, 201,260,927 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| ftCo_8009F75C | 142,387,362 | 151 | all |
| ftCo_IsCpuControlled | 26,137,836 | 151 | all |
| ftCo_8009F5AC | 11,604,040 | 151 | all |
| ftCo_8009F7F8 | 11,604,040 | 151 | all |
| ftCo_8009F54C | 9,527,649 | 151 | all |

**player (pl)** (9 of the 200, 200,070,601 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| Player_80036394 | 44,052,685 | 151 | all |
| pl_8003906C | 31,709,190 | 151 | all |
| Player_8003248C | 26,138,068 | 151 | all |
| Player_GetStaleMoveTableIndexPtr2 | 26,134,587 | 151 | all |
| Player_GetCostumeId | 22,462,518 | 151 | all |
| Player_8003221C | 14,766,085 | 151 | all |
| Player_GetFalls | 13,878,243 | 151 | all |
| Player_GetKOsByPlayerIndex | 11,309,011 | 151 | all |
| pl_800386D8 | 9,620,214 | 151 | all |

**HUD (if)** (8 of the 200, 192,021,654 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| ifStatus_GetHUDInfo | 76,532,652 | 151 | all |
| ifMagnify_802FB8C0 | 26,977,536 | 151 | all |
| NameTag_RenderCallback | 26,977,536 | 151 | all |
| fn_802FDA4C | 17,985,024 | 151 | all |
| ifAll_IsHUDHidden | 14,838,024 | 151 | all |
| ifStatus_802F5DE0 | 9,570,294 | 151 | all |
| ifStatus_802F5E50 | 9,570,294 | 151 | all |
| fn_802F94E0 | 9,570,294 | 151 | all |

**gobj/proc** (2 of the 200, 141,242,791 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_GObj_80390EB8 | 116,499,862 | 151 | all |
| HSD_GObj_80390ED0 | 24,742,929 | 151 | all |

**effects/particles** (4 of the 200, 118,382,468 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| hsd_8039930C | 77,193,316 | 151 | all |
| hsd_8039D214 | 17,949,521 | 151 | all |
| hsd_8039D3AC | 13,655,921 | 151 | all |
| efLib_render_callback | 9,583,710 | 151 | all |

**modes (gm/mn)** (5 of the 200, 101,641,394 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| gm_801A3714 | 35,970,048 | 151 | all |
| gm_80168BF8 | 22,354,696 | 151 | all |
| gm_80168B34 | 21,572,183 | 145 | mixed |
| gm_801A3820 | 11,990,016 | 151 | all |
| gm_8016B1C4 | 9,754,451 | 151 | all |

**RNG** (1 of the 200, 41,996,191 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| HSD_Randf | 41,996,191 | 151 | all |

**SDK misc** (1 of the 200, 22,376,336 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| strcpy | 22,376,336 | 151 | all |

**unknown TU** (1 of the 200, 13,343,752 calls)

| function | calls | replays | trigger |
|---|---:|---:|---|
| memset | 13,343,752 | 151 | all |

### Character action callbacks, common states and items (executed, unchecked; by family)

| family | unchecked executed | calls | hottest (calls) |
|---|---:|---:|---|
| common fighter states | 785 | 263,135,584 | ftCo_8009F75C (142,387,362), ftCo_IsCpuControlled (26,137,836), ftCo_8009F5AC (11,604,040) |
| items: core | 201 | 30,942,243 | it_8026BC68 (2,322,314), it_802799E4 (1,164,480), it_80279BE0 (1,159,754) |
| items: per-kind (it/kinds) | 877 | 5,631,439 | it_802A24A0 (681,380), it_802D98C4 (539,939), it_802D96B0 (537,402) |
| character: Kirby | 144 | 3,943,614 | ftKb_SpecialN_800F1D24 (3,171,411), ftKb_UnkIntBoolFunc0 (625,335), ftKb_UnkMtxFunc0 (80,315) |
| character: Purin | 60 | 2,473,829 | ftPr_Init_UnkIntBoolFunc0 (2,196,780), ftPr_Init_UnkMtxFunc0 (264,810), ftPr_SpecialAirS_Coll (1,355) |
| character: Peach | 96 | 1,176,073 | ftPe_8011BA54 (438,623), ftPe_8011BAD8 (411,338), ftPe_Float_CheckContinueInput (184,377) |
| character: Fox | 156 | 617,058 | ftFx_AppealS_CheckInput (263,796), ftFx_SpecialN_GetBlasterAction (46,976), ftFx_Throw_Anim (31,523) |
| character: Koopa | 76 | 157,935 | ftKp_Init_UnkMotionStates3 (60,046), ftKp_SpecialLw_80134D78 (60,046), ftKp_SpecialN_80135780 (11,740) |
| character: Captain | 73 | 106,486 | ftCa_SpecialHi_Phys (9,750), ftCa_SpecialHi_Coll (9,487), ftCa_SpecialAirHi_Phys (9,413) |
| character: Seak | 126 | 91,513 | ftSk_Init_UnkMotionStates4 (19,337), ftSk_SpecialNLoop_IASA (4,911), ftSk_SpecialNLoop_Anim (4,827) |
| character: Mario | 54 | 64,235 | ftMr_SpecialN_ItemFireSpawn (8,255), ftMr_SpecialAirN_Phys (5,812), ftMr_SpecialAirN_Coll (5,812) |
| character: Luigi | 81 | 62,072 | ftLg_SpecialN_FireSpawn (3,088), ftLg_SpecialLw_Anim (2,180), ftLg_SpecialLw_Coll (2,177) |
| character: Link | 90 | 61,874 | spawnBomb (6,121), ftLk_SpecialAirLw_Phys (3,736), ftLk_SpecialAirLw_Coll (3,735) |
| character: Mewtwo | 83 | 52,907 | ftMt_SpecialN_Shoot (7,480), ftMt_Init_UnkMotionStates4 (5,797), ftMt_SpecialN_CheckShadowBallRemove (4,700) |
| character: Donkey | 58 | 49,296 | ftDk_Init_UnkMotionStates4 (8,958), ftDk_SpecialAirHi_Coll (4,282), ftDk_SpecialAirHi_Phys (4,134) |
| character: Pikachu | 116 | 47,406 | ftPk_SpecialAirN_Phys (3,033), ftPk_SpecialAirN_Coll (3,033), ftPk_SpecialAirN_Anim (2,970) |
| character: GameWatch | 138 | 45,404 | ftGw_Init_UnkMotionStates4 (4,448), ftGw_Init_8014A538 (2,554), ftGw_AttackAirN_Coll (2,382) |
| character: Mars | 85 | 45,108 | ftMs_SpecialAirS1_Coll (4,261), ftMs_SpecialAirS1_Phys (4,212), ftMs_SpecialAirS1_Anim (4,198) |
| character: Ness | 125 | 39,644 | ftNs_SpecialHi_ItemPKThunder_CheckNessCollide (3,027), ftNs_SpecialAirHiHold_Phys (2,426), ftNs_SpecialAirHiHold_Coll (2,425) |
| character: Yoshi | 105 | 38,657 | ftYs_Init_8012B804 (7,940), ftYs_Init_8012B8A4 (3,949), fn_8012E110 (1,989) |
| character: Samus | 81 | 36,518 | ftSs_Init_UnkMotionStates4 (3,674), ftSs_SpecialAirLwBomb_Phys (3,576), ftSs_SpecialAirLwBomb_Coll (3,576) |
| character: Popo | 71 | 35,759 | fn_80122D2C (3,485), ftPp_SpecialLw_Anim (3,403), ftPp_SpecialLw_Coll (3,403) |
| character: MasterHand | 1 | 11,566 | doAnim (11,566) |
| character: Zelda | 75 | 9,745 | ftZd_SpecialAirHiStart_0_Phys (802), ftZd_SpecialAirHiStart_0_Coll (802), ftZd_SpecialAirHiStart_0_Anim (796) |
| character: DrMario | 12 | 3,400 | ftDr_AppealS_Anim (680), ftDr_AppealS_Coll (680), ftDr_AppealS_Phys (680) |
| character: Falco | 4 | 1,856 | ftFc_Init_OnKnockbackEnter (1,271), ftFc_Init_OnKnockbackExit (399), ftFc_Init_OnDeath (137) |
| character: CLink | 16 | 1,704 | ftCl_Init_8014919C (710), ftCl_Init_OnKnockbackEnter (270), ftCl_Init_OnKnockbackExit (136) |
| character: Nana | 23 | 1,107 | ftNn_Init_80123B10 (207), ftPp_SpecialS_1_Coll (190), ftPp_SpecialS_1_Phys (187) |
| character: Pichu | 4 | 484 | ftPc_Init_OnKnockbackEnter (286), ftPc_Init_OnKnockbackExit (166), ftPc_Init_OnDeath (25) |
| character: Ganon | 3 | 462 | ftGn_Init_OnKnockbackEnter (295), ftGn_Init_OnKnockbackExit (141), ftGn_Init_OnDeath (26) |
| character: Emblem | 4 | 358 | ftFe_Init_OnKnockbackEnter (232), ftFe_Init_OnKnockbackExit (102), ftFe_Init_OnDeath (19) |

## 4. Recommendation: what to bulk-hook first, and how

Ranked by sim calls gained per unit of work, with the sync work each needs (the not-checkable reasons above say most
new hooks would otherwise come back "not checkable").

1. **HSD animation core (FObj / AObj / JObj anim): ~33.6 G calls, about 70% of all sim calls, about 40 functions.**
   `HSD_FObjSetState/GetState`, `parseFloat`, `HSD_AObjInterpretAnim`, `HSD_FObjInterpretAnim`, `FObjUpdateAnim`,
   `JObjUpdateFunc`, `HSD_JObjAnim`, `JObjAnimAll`. This is the "inputs to the bone matrices" candidate of
   referee.md 10.2 (a) (Bob-omb one frame early). How: `referee_native!` on the few entry points (`HSD_JObjAnimAll`,
   `HSD_AObjInterpretAnim`, `HSD_FObjInterpretAnimAll`) with `R(jobj, "HSD_JObj")` / `R(aobj, "HSD_AObj")`; the
   callees run inside the retail tree, so one hook covers the chain transitively. Needs: AObj/FObj/AnimJoint records
   registered in `remap::GuestSpace` (as HSD_JObj already is), and the `HSD_ObjAlloc` free lists (`store to Bss
   HSD_AObj_X`, `HSD_Mtx_X`) either synced or run as native callees (`MELEE_REFEREE_NATIVE_CALLEES=HSD_ObjAlloc,
   HSD_ObjFree`, the way HSD_Randi is).
2. **Math leaves: ~6.1 G calls, 46 functions** (PSMTXConcat, sinf/cosf, sqrtf, PSVEC*, lbVector_*, HSD_Mtx*).
   Pure, no records: the capture-mode `referee_hook!` set already matches (notes/referee.md 4) but is offline. How:
   promote them to `referee_native!` with `P(mtx, 48)` / `F(..)` arguments, generated from one list (name,
   signature) now that paired singles run (referee.md 1.1). Cheapest family; no sync work.
3. **Fighter action callbacks, all characters at once: 1,538 executed functions (24% of the executed sim
   functions), 13.5 M calls.** Every row of every MotionState table (the common 341 rows and each kind's own) is
   filled in one generated file, `rust/melee_link/src/retail_ms.rs` (`tools/rust-port/link/gen_retail_ms.py`):
   each `anim_cb/input_cb/phys_cb/coll_cb` is a `cabi::c1(|a0| tu::<mod>::<Name>(..))` closure with the retail name
   at hand. How: in a referee build the generator emits `|a0| { referee_native!(Name, [R(g, "HSD_GObj")], v,
   Name(g)); Name(g) }` (or a `referee_ms!` wrapper macro that expands to the plain call without the feature); the
   generator checks every name against symbols.txt. That gives the enumeration from the state tables with no per-TU
   edits. **Do the per-kind mapping first:** `Fighter+0x2d4` (dat_attrs, the kind's attribute record) and the
   per-kind FighterVars / MotionVars unions (the union-member hook of referee.md 10.3). Without it, the special-state
   paths are exactly what gave the class-1 mismatches above, and character callbacks would be mostly
   not checkable or false mismatches.
4. **Item state and logic tables: 340 executed functions (175 ItemStateTable rows, 165 logic-table callbacks),
   6.7 M calls.** Same pattern: `st_row(..)` in the item TUs (callbacks take an item gobj, return bool/void) and
   `retail_it.rs` (`gen_retail_it.py`). Needs the per-kind ItemVars union and ECBSource (referee.md 10.3). Little
   item content in the replay set (section 5), so this pays off only with item replays.
5. **Stages + collision/mp: ~2.2 G calls, ~990 functions** (`Ground_801C2D24` 238 M, `Stage_GetCamBounds*`,
   mpLib/mpColl line queries). How: hook the stage's per-frame callbacks from the gr TUs' StageCallbacks tables and
   the mpColl entry points (`mpColl_*` take a CollData record). Needs Ground .sbss and the mpIsland .bss in the sync
   table (the largest not-checkable reason, 41.8%). This is also where the Big Blue lead (section 2, class 5) is.
6. **Camera: 54 functions, 0.40 G calls,** and the second-largest not-checkable reason (`HSD_CObjGetCurrent`,
   24.5%, which blocks `Fighter_procAnim` and `ftDrawCommon_80080E18`). Sync the current CObj and the camera
   statics (0x80452C68) first; that alone turns about 90,000 not-checkable calls of existing hooks into compared ones.

Not worth hooking: the render family (no sim state) and audio (no sim state; but its .bss shows up as a not-checkable
reason inside fighter functions, so run `HSD_AudioSFX*` as no-op native callees).

## 5. Missing content (tier-1 regression set; the full 8,697-file tier-1 corpus has the same gaps)

- **Stages never present (11):** Kongo Jungle, Brinstar, Jungle Japes, Green Greens, Fourside, Mushroom Kingdom II,
  Venom, Icicle Mountain, Flat Zone, Yoshi's Island (N64), Kongo Jungle (N64). Thin (1-3 replays): Peach's Castle,
  Brinstar Depths, Mushroom Kingdom, Onett, Corneria, Mute City, Great Bay, Yoshi's Island, Poke Floats, Big Blue.
  All six legal stages are well covered (FoD 157, PS 162, YS 336, DL 156, BF 308, FD 191).
- **Characters:** all 26 present (FighterKind incl. Sheik, Zelda, Popo, Nana); fewest: Bowser 14, Roy 19, Pichu 25,
  Zelda 27 (internal), GnW 27.
- **Items:** no tier-1 replay has items switched on (item spawn "off" in all 1,342; also in all 8,697 of the full
  corpus). Seen: fighter projectiles, Peach's pulls (Bob-omb, Mr. Saturn, Beam Sword in the full corpus) and stage
  items (Shy Guys, Pokemon Stadium and other stage entities). Never seen: every other common item (capsule only via
  Peach, crates, barrels, eggs, party ball, barrel cannon, hearts, tomato, starman, bat, parasol, shells, ray gun,
  freezie, food, motion-sensor bomb, flipper, super scope, star rod, lip's stick, fan, fire flower, mushrooms, hammer,
  warp star, screw attack, bunny hood, metal box, cloaking device, poke ball), all Poke Ball Pokemon, Kirby's copied
  projectiles (26 kinds), Game & Watch's Panic, Master/Crazy Hand items, and the old-stage enemies (Klap, Likelike,
  ...). So `items: per-kind` coverage (881 functions) is almost all fighter projectiles.

**Lunar fetch: not run.** `tools/rust-port/slp/lunar_fetch.py` can target only characters (p1/p2CharacterId
searches); it has no stage or item filter, and no character is missing. Lunar's corpus is netplay/tournament
games (items off, legal stages), so it is the wrong source for the gaps anyway. What would fill them: our own
recordings (coverage-testing.md section 2: `gen_matches.py stages` / `items` configs recorded in the Dolphin oracle,
which already covers all 28 VS stages and items-on), run through this map's count pass. Up to 40 such seeds: one
per missing stage (11), one items-on match per legal stage (6), one Poke Ball / Pokemon-heavy items match on
Pokemon Stadium, and Kirby copying each of the 18 characters whose copy makes an item = 36.

## 6. Unproven / limits

- **Executed = the Rust port ran.** The map assumes each retail function's Rust port is called where retail calls
  it. Functions the port restructured (merged, inlined into a caller with no own fn, or split) are attributed to
  whatever Rust fn carries the retail name; retail functions with no same-named Rust fn are invisible. 6,775
  executed is a lower bound on retail functions reached.
- **Counts are the engine's slpcheck runs** (`--full`), boot and scene setup included (some modes/gm functions run
  once per replay); 5 of 156 count runs crashed in the instrumented build (all 5 match in the regression set with the
  normal exe: a cov-build or referee-feature crash, not looked at) and 4 of 52 referee runs gave no report.
- **Referee pass on 48 replays** (the first in pick order), `MELEE_REFEREE_MAX` 2000 per function per replay, so
  per-function referee call counts are capped and the verdict is over a sample.
- **Mismatches are not triaged** beyond section 2. Class 1 is very likely an artifact (known mapping gap); 2, 3, 4
  and 5 are open; 5 may be the Big Blue desync or a sync gap.
- "Checked transitively" (190 functions) means the retail code ran inside a compared call; the function's own Rust
  port is then not compared directly, and a not-checkable run stops before the compare.
- The render/audio split is by TU; a few TUs mix sim and draw code (e.g. parts of jobj.c), so the sim view is
  approximate.
- No engine code changed in this branch (tools and this note only), so the real-net check and gate were not needed;
  the cov build is the existing opt-in profile.
