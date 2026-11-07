# Coverage testing: how much of the game code the console has checked (2026-11-06, branch port/covtest)

No disc data, replay or disc path is in this note or in git. The corpus lives in `_build/covcorpus/` (untracked).

"Tested everything" becomes a number: **branch outcomes of the ported game code that ran in an engine run checked
against the console (Dolphin), out of all of them**, plus, for the float/int helpers, **call sites where an alternative
semantics would have given a different result, on a frame the console verified**. Phase A (this note) builds the
measurement and three seed sources; phases B and C (end) use it to steer.

## 1. Branch counting (the instrumented build)

| | |
|---|---|
| build | `bash tools/rust-port/covtest/build_cov.sh` -> `_build/cargo-agents/covtest/cov/melee.exe`; `DISC=1` -> `covdisc/melee.exe` (+ section 3) |
| how | cargo profile `cov` (= `fast`) and `RUSTC_WORKSPACE_WRAPPER=rustc_cov.exe` (built from `tools/rust-port/covtest/rustc_cov.rs`), which adds `-C instrument-coverage -Z coverage-options=branch` (LLVM source-based coverage with branch counters; `RUSTC_BOOTSTRAP=1` for those rustc processes only) to melee_fighter, melee_items, melee_world, melee_runtime, melee_modes, melee_chara and the `melee` binary (the binary must be instrumented so the profiler runtime links; its files are dropped from the report). Renderer, net, host, platform, tooling crates are not instrumented |
| normal builds | unchanged: no other profile uses the wrapper; release/fast/perf/dev compile exactly as before (GATE OK and rematch css on this branch) |
| speed | a 3,724-frame CPU match through `slpcheck`: 1.8 s uninstrumented, 6.4 s instrumented, 8.0 s with the discrimination bits (boot included; ~600-700 frames/s) |
| run | `LLVM_PROFILE_FILE=<dir>/x.%p.profraw melee.exe slpcheck ...`; raw profiles are ~10 MB each, so `run_batch.py` merges them into one `prof/<lane>.profdata` per lane after every match |
| report | `bash tools/rust-port/covtest/coverage.sh CORPUS [EXE]` (`PROF_GLOB`, `COV_OUT` pick a subset): llvm-profdata/llvm-cov from rustup's `llvm-tools` (they must match rustc's LLVM: the LLVM 23 install in Program Files rejects rustc 1.96's raw profile v10), `llvm-cov export`, then `report.py` |

`report.py` counts LLVM **branch outcomes** (every condition twice: taken true, taken false) and regions in
`rust/<crate>/src/tu/*.rs` whose stem is a decomp C file (`src/melee/**`, `src/sysdolphin/**`): the ported TUs. The
port's own TUs (script_game, geno_*, the tests, mex: 28 files) are left out. Generic instantiations of one function are
merged by source location; a function is named by the `fn NAME` on its first line, which is the decomp name.
Outputs: `summary.txt` (total, per crate, the 50 largest never-entered functions), `functions.tsv`, `files.tsv`.

## 2. Seed matches

### CPU seed matches on the console

| step | tool |
|---|---|
| configs | `gen_matches.py OUT.jsonl`: `pairs` all 676 ordered pairs of the 26 selectable CharacterKinds (Zelda and Sheik both; Ice Climbers one slot) on Battlefield, CPU 9; `stages` six matchups on the other 28 stages (legal + every casual VS stage except Akaneia/Icetop, which are not on the disc's VS list); `levels` CPU 1/3/5/7 on FD; `items` the legal six with items on (medium, all switches); `doubles` 2v2 teams on the legal six. 4 stocks, a 1-minute stock-match timer: 908 configs |
| record | `record.py CONFIG OUT.slp LANE`: the patched Slippi Dolphin of notes/dolphin-oracle.md, unchanged, no `-i`, a private user dir per lane with a generated `GameSettings/GALE01.ini`: Slippi's own `General Codes` and `Slippi Recording` (copied from the build's Netplay game settings at run time, as Slippi online runs; the Sys playback codes disabled) plus two covtest codes from `gecko_boot.py`. Hidden, Null video, no audio, Idle priority, no input. The script ends its own Dolphin (by handle) once the .slp is closed: in recording mode nothing else stops the console |
| boot codes | `041BFA20 38600002`: bootOnLeave's `lbz r3,4(r31)` -> `li r3,2`, the boot goes to GM_VS. `C21A55EC`: an assembled block at gmVsMelee_Mode_OnLoad's first instruction doing what the port's `SceneLaunch_SeedVs` does (VsModeData players and rules, GameRules/GamePrefs that gm_80167BC8 rebuilds the rules from, gm_80473814.stage_id, preloadCache.scene, then `lbDvd_SetupVsPreloadCache()` and gm_SetGameModeStateId(2)); offsets from melee_types' GC layout, addresses from config/GALE01/symbols.txt, instructions checked against main.dol (`dol.py`) |
| check | `check.sh MATCH.slp OUT`: the .slp played twice in Dolphin concurrently (run_dolphin.ps1; `DOLPHIN_USER_DIR` gives each run its own user dir: several runs on one hang at start-up), `raw2csv.py`, then the engine twice: `slpcheck --cpu-slots` (the CPUs stay CPU slots, as the console's playback keeps them, fed their recorded inputs) and `slpcheck --cpu-ai` (the same start, the engine's own CPU AI makes the inputs). Both compared with compare_golden.py `--ignore-seed --allow-short`; slpcheck's own Post-Frame comparison is kept too |
| batch | `run_batch.py CONFIGS CORPUS [--lanes N]`: N lanes (default cores/5 = 4 here; a lane peaks at two Dolphins and one engine); resumable; `index.jsonl` one line per match (config, what it covers, both results) |
| results | `summarize.py CORPUS` -> `results.txt` |

### Move soak (our own recordings)

`gen_soak.py`: per character a txt pad script for port 1 (jab, tilts, smashes, a charged smash, dash attack, the four
ground specials, grab + pummels, the four throws, the five aerials, a fast-fall, air specials, shield, rolls, spot and
air dodge, crouch, double jump), each at three spacings, facing toward and away from port 2 (20,000 frames).
Port 2 is a human with no controller. `run_batch.py soak.jsonl CORPUS --prof-prefix soak_` records each with
`melee play --match <char>,falcon,32 --stocks 99 --headless` (vanilla rules, no Gecko codes, no mods) and
`MELEE_SLP_RECORD`, then `check.sh` verifies it in the Dolphin oracle like a CPU match.

## 3. Discrimination bits

Feature `disc` (melee_core `disc.rs`, enabled only by `DISC=1 build_cov.sh`; in every other build the attributes and
calls are compiled out). The helpers (`fmadds fmsubs fmsubs_neg fnmsubs fnmadds`, `Ppc::pdiv prem pshl pshr`, `fctiwz
fctiwz_f32`) are `#[track_caller]` there, and each call site gets a cell keyed by its location: operand classes (float:
±0, ±denormal, ±normal, ±inf, NaN per operand; int: negative, zero, MIN, shift >= width, divisor 0) and one bit per
alternative whose result differed: fused (`mul_add`), unfused (f32 multiply then f32 add), f64 intermediate kept,
zero sign (fnm: `fmadds(-a,c,b)`), flooring division/modulo, wrapping (masked shift amount; i64->i32 cast instead of
saturation). Each bit keeps the first frame it was set on; `slpcheck` writes the table to `MELEE_DISC_OUT`.
`disc_recheck.py CORPUS` re-runs the engine side of each checked match with that build (the kept dolphin.csv) and
`disc_report.py CORPUS [--sets S --base B]` counts a bit as **verified** when it was first set before that run's
first NHV1 difference from the console. Totals are the helper call sites in the decomp-named TUs, counted in source.

## 4. First numbers

Corpus `_build/covcorpus/` (rust-port 845cc9946 + this branch): 908 CPU seed matches (676 pairs on Battlefield, 168 stage
sweep, 16 levels, 24 items, 24 doubles) and 26 move soaks; 934 .slp (1.4 GB), checks 3.3 GB, profiles 19 MB. About 5 h
wall on 4 lanes (the slower stages and two builds ran alongside). Every engine run counts below, also those that
later diverged: "exercised in a run checked against the console", not "verified frame-exact".

### Branch coverage of the game code (branch outcomes; `coverage.sh`)

| | CPU matches | move soak | both |
|---|---|---|---|
| **five crates** (fighter, items, world, runtime, modes) | 30,568 / 80,140 (38.1%) | 15,958 (19.9%) | **30,723 / 80,140 (38.3%)** |
| + melee_chara | 35,275 / 88,938 (39.7%) | 18,572 (20.9%) | 35,717 / 88,938 (40.2%) |

Per crate (both sources): melee_fighter 11,006/16,898 (65.1%), melee_items 5,400/13,682 (39.5%), melee_world
9,463/18,054 (52.4%), melee_runtime 3,929/11,486 (34.2%), melee_modes 925/20,020 (4.6%), melee_chara 4,994/8,798
(56.8%). Regions: 42.5% (five crates); functions entered 11,841/29,164 (40.6%). The soak adds 155 branch outcomes
in the five crates (442 with melee_chara, mostly character moves the CPUs do not use) over the CPU matches.

The 50 largest never-entered functions are `cov_all/summary.txt`; the top of the list is all outside VS play:
`fn_802545C4` (mnsnap, 396 branch outcomes), `mnCharSel_CursorThink` (358), `fn_801701C0` (gm_16f1, 218), `fn_80039618`
(plbonus, 216), `fn_801AB200` (gmstaffroll, 168), `_Toy_80309404` (164), `hsd_803A949C` (card, 160), `mnCharSel_802640A0`,
the memory-card TU (`fn_803AE7F8`, `hsd_803AAA48`, `fn_803AD16C`, ...), `mpLib_80059554` (128), the tournament
mode (gmtou*), name entry, rules menu, the event, staff roll and opening scenes, `grBigBlueRoute_8020CD20`,
`grShrineRoute_80208F70`, Master/Crazy Hand. So: menus, card and 1P modes are the next seeds; mpLib_80059554 is
the one large in-match gap.

### Dolphin-exact seed matches (NHV1 rows, seed ignored; `summarize.py`)

| | recorded inputs (`--cpu-slots`) | own CPU AI (`--cpu-ai`) |
|---|---|---|
| CPU matches, all NHV1 words | **0 / 908** | **0 / 908** |
| CPU matches, slpcheck Post-Frame (the 11 recorded fields) | 807 / 908 | 0 / 908 |
| move soak, all NHV1 words | **21 / 26** | (no CPU) |
| move soak, Post-Frame | 26 / 26 | |

Dolphin's two playbacks disagreed in 2 of 934 runs.

First-difference classes (`classify.sh` names the fields for a sample):
- **CPU AI scratch floats before GO** (792 of the 908 first differ on NHV1 frames 1-122, most at 64-92, words s0/s2; 67 more after GO): `cpu.x38`, `cpu.x55C/x560/x564`
  differ by a few ULPs (e.g. 7.832886 vs 7.8329225) while every fighter field still agrees; sometimes a hurt-capsule z or
  an ECB x one ULP off at the same frame. The recorded-inputs mode then stays on the console's Post-Frame for most
  matches (807), so it is the AI's internal state, hashed in the CPU block, that is off. With its own AI the engine's
  moves follow from that state: every AI run leaves the console's x/y/action within the first seconds.
- **frame 0** (49, mostly the stage sweep: Onett etc.): `cpu.x30`, `x20A4.bf_0` and the RNG seed: stage-dependent CPU
  setup before the first frame.
- **soak**: Zelda's spawn landing (frame 5, `coll_data.cur_pos.y`, the known spawn class of dolphin-oracle.md),
  Ice Climbers Nana's CPU block (s1, frame 284), and item fields (`itemsum`: Link/Young Link frame 3132 a projectile's
  `pos.x`/collision, Samus frame 11564 an item `msid`).

### Discrimination bits (`disc_report.py`, 1,868 engine runs)

Sites discriminated and verified (first set before the run's first difference from the console) / helper call sites:

| helper | alternative | verified / sites | seen in any run | soak adds |
|---|---|---|---|---|
| fmadds/fmsubs | fused (one rounding) | 0 / 1,768 | 11 | 0 |
| | unfused (f32 mul + f32 add) | 360 / 1,768 | 578 | +69 |
| | f64 intermediate kept | 565 / 1,768 | 848 | +136 |
| fnmsubs/fnmadds | zero sign (fmadds(-a,c,b)) | 2 / 46 | 3 | +1 |
| | fused | 2 / 46 | 3 | +1 |
| | unfused | 6 / 46 | 14 | +3 |
| pdiv / prem | flooring | 0 / 48, 0 / 45 | 0 | 0 |
| pshl / pshr | wrapping amount | 0 / 67, 0 / 5 | 0 | 0 |
| fctiwz | wrapping cast | 0 / 591 | 0 | 0 |

No run produced a negative-and-inexact division, a shift amount >= 32 or an out-of-range float-to-int conversion, so
the integer semantics are untested by play; those are phase C's operand-class targets. "Fused" is verified nowhere:
the double rounding of the f64 model differs from a single rounding only in 11 sites, all after their run diverged.

## 5. Next phases

- **B. Coverage-guided fuzzing from save points.** Snapshot the engine (and Dolphin savestates) at points of the
  verified corpus, mutate inputs from there, keep inputs that add branch outcomes or discrimination bits
  (functions.tsv / disc.tsv are the feedback), and verify each kept input in the oracle. The never-entered list in
  section 4 is the target list (menus and 1P modes need their own seeds: the CSS, SSS, Classic, Adventure, events).
- **C. Per-function differential tests against PowerPC.** For functions the matches cannot reach (or reach without
  discriminating), run the decomp function on the console (Dolphin, an HLE call into the DOL with chosen operands from
  the operand-class table) and the Rust port on the same inputs, comparing outputs and memory. The discrimination
  bits say which call sites still need an operand class (a NaN, a -0, a denormal) to tell the semantics apart.
