# slpcheck: a Slippi replay as a direct oracle (2026-11-02, branch port/slporacle)

No replay data is in this note or in git. The 125 files live untracked in the workspace `scratch/replays-ext/`
(see notes/replays-ext.md). Every number below is an aggregate.

A `.slp` carries the console's own Post-Frame Update (0x38) for every fighter and frame. `melee slpcheck` runs the
engine on the replay's pre-frame inputs and compares our fighter state with that record after every frame. No C run is
involved, so any divergence is either a port bug, a missing Slippi code, or a stage/hardware difference listed below.

## Usage
```
melee slpcheck --slp FILE [--frames N] [--full] [--no-resync] [--gecko] [--csv SUMMARY.csv] [--iso DISC.iso]
tools/rust-port/slp/run_all.sh REPLAY_DIR OUT_DIR [slpcheck args]     # one process per file, JOBS=4, all.csv
```
- Default stops at the first divergent frame and prints
  `SLPCHECK <file> diverged first divergence: frame F port P field X engine=.. replay=..`. `--full` keeps running to the
  end of the replay (or the engine's match end) and fills the per-field "first divergent frame" columns (`fd_*`).
  `SLPCHECK_DUMP=1` prints engine and replay values for every frame (stderr, large).
- `--gecko` prints the replay's Gecko list (type, address, lines, hash). `--no-resync` stops forcing each frame's
  Frame Start seed.
- Summary CSV columns: file, status (match | diverged | panic@F | refused | boot-error), slippi, stage, roster
  (`p<port>:<external character id>`), ucf_dash_shield (dashback/shield-drop words per port), frozen_ps, pal, gecko_codes,
  gecko_hash, replay_frames, frames_compared (fighter-frames), last_frame, first_frame/port/field, engine_value,
  replay_value, `fd_<field>` per field, panic, notes, seed_first (experimental).
- A panic inside the engine aborts the process (the engine's `extern "C"` frames do not unwind), so `run_all.sh` records such a
  file as `crash-rc<N>` with the log kept next to it.

## What is compared
Taken at the point Slippi 3.x recorded post-frame (end of the frame, `Fighter_UnkCallCameraCallback_8006D9EC`, the C's
`ftReplay_TraceFighter`; 1.x/2.x files use `Fighter_procMap`, picked from the file's version). Floats must be bit-equal.

| field (Post-Frame offset, added in) | engine source |
|---|---|
| x, y position (0xA, 0xE; 0.1.0) | `cur_pos` |
| facing (0x12; 0.1.0) | `facing_dir` |
| percent (0x16; 0.1.0) | `dmg.x1830_percent` |
| action state id (0x8; 0.1.0) | `motion_id` |
| action state frame counter (0x22; 0.2.0) | `cur_anim_frame` |
| shield size (0x1A; 0.1.0) | `shield_health` |
| stocks (0x21; 0.1.0) | `Player_GetStocks` |
| last hitting attack id (0x1E; 0.1.0) | `x208C` (the combo logic's last attack id; sticky until the next hit) |
| combo count (0x1F; 0.1.0) | `x2090` |
| hitlag frames remaining (0x49; 3.8.0) | `dmg.x195c_hitlag_frames` |
| ground/air (0x2F; 2.0.0) | `ground_or_air` != ground |

Version rule: a field is compared only when the file's payload size for 0x38 covers it (the spec appends fields). The
reader is `melee_data::slp` (written from the public SPEC.md, parses the raw element, Event Payloads, Game Start, Post-Frame,
Gecko List via Message Splitter). Followers (Nana) are not compared; ports that the Game Start block marks empty are not run.

Findings made while building it (all wrong at first, all in the repo now):
- `x2088` is the attack INSTANCE counter and `x2068_attackID` the CURRENT move id (1 = none); neither is Slippi's "last hitting attack id".
- the VS mode's own `Replay_Active` hook was never set, so Slippi's Neutral Spawn (gmvs.c, gated on `Replay_Active()`) did not run: fighters
  started at (0, 8) on Battlefield and with port-indexed spawn slots. Set before the match boots (`replay_modes_active`).
- baselib rumble (`PADControlMotor`) was unhooked: the first hit panicked (non-unwinding). Now a no-op host output.
- `Fighter_SmashChargeShakeTable`: the loader set the tu static but not `hooks::FIGHTER_SMASH_CHARGE_SHAKE_TABLE`, which `ft_0df0` reads:
  every smash charge panicked ("not loaded"). That was the known Marth-on-BF panic (22 of 125 files hit it).

## Game Start conversion (golden --slp and slpcheck)
`slpcheck::convert` turns the Game Start block into the engine's `Scenario`: external stage id (any, no FD/BF gate), ports keep the file's
player index (the engine's fighter slot and hash column), character, costume, team and team on/off, handicap, per-player stocks and the
rule stock count, game mode (time/stock), timer seconds (0 = no timer), item spawn behaviour (off or frequency), the RNG seed (as
MELEE_TEST_SEED, as before). A CPU slot runs as a human on its recorded inputs. Recorded but NOT applied (listed in the CSV `notes`):
offense/defense/model scale, start/spawn damage, player bitfield flags (stamina, metal, invisible, low gravity, warp-in start), damage
ratio, bomb rain, PAL, the item-spawn mask. UCF dashback / shield-drop words are only recorded (`ucf_summary`); the sim already applies
UCF 0.84 for a port whose word is 1 through the existing `Replay_UcfVersion` hooks (melee_data::player), 0.73 for pre-3.0 files.
Slippi_Codes (tournament = 1), Slippi_Version and the Frozen Pokemon Stadium flag are now routed from the replay to the sim.

## Results over scratch/replays-ext (125 files, Slippi 3.17.0, 1.29 M replay frames)
Build f7d0d025d + slporacle (HEAD at the time of the run), default mode (stop at the first divergence), 4 jobs.

| | |
|---|---|
| replays run | 125 of 125 (0 refused, 0 crashed, 0 boot errors) |
| fully matching | 0 |
| crashes | 0 now. Before the shake-table fix 22 crashed (all `Fighter_SmashChargeShakeTable not loaded`). The frame-8792 panic of 20476_06200225 cannot be reached: that file diverges at frame -8 and the engine's match ends at tick 2707 |
| stage mix | ext31 33, ext8 25, ext32 21, ext3 18, ext28 18, ext2 10 |

First divergence frame (0 = timer start; min -33, p25 601, median 1157, p75 1987, max 8677):

| frame | files |
|---|---|
| < 0 | 3 |
| 0-299 | 5 |
| 300-599 | 23 |
| 600-899 | 18 |
| 900-1199 | 15 |
| 1200-1799 | 25 |
| 1800-3599 | 26 |
| 3600+ | 10 |

First divergent field (the first in the order x, y, facing, percent, action, state frame, shield, stocks, last attack, combo, hitlag, airborne):

| field | files | what it is |
|---|---|---|
| action | 49 | 39: DamageFlyRoll (91) vs DamageFlyHi/N/Lw (87-89), both directions, on the frame of a hit, positions still equal; 6: action 4 vs 6 (death states); 4 singletons |
| x | 28 | 16 of 18 Dream Land (ext28) files: a constant 0.2 per frame extra push on a fighter in damage (engine-minus-replay = +-0.2, growing 0.2/frame); 3.96 jumps on several stages in a shield state (a shield push the console applies and the engine does not, or a frame apart); rest small |
| hitlag | 18 | hit connects on a different frame (engine 0, replay 3-7 or the reverse) |
| y | 18 | 10 of 18 Pokemon Stadium (ext3) and 6 of 10 Fountain (ext2): platform height differences of 2.5-7 units |
| percent | 6 | a hit the engine did not get at frame -8 (Sheik needle on Fox), staling-sized differences (1-4) and 28 |
| last_attack | 6 | a hit landed in the replay that the engine has not (or has with another attack id) |

By stage (files, median first divergence frame): ext2 10 / 507; ext3 18 / 945; ext8 25 / 1500; ext28 18 / 608; ext31 33 / 1983; ext32 21 / 1549.

## Which divergences are expected
All 125 files are Slippi 3.17.0 console/tournament recordings, UCF dashback = 1 and shield drop = 1 for every player (UCF 0.84 by the
version rule), Frozen Pokemon Stadium flag = 1, PAL = 0. Gecko lists carry 88, 89, 95 or 113 codes (153 distinct codes; 70 identical in
all four list shapes; per file the hash differs because some payloads embed per-game data). Code addresses mapped through
config/GALE01/symbols.txt; the gameplay-touching injection sites present in every list:
- UCF: `ftCo_Turn_IASA+0x4C` (dashback), `Fighter_Spaghetti_8006AD10+0x3D0` / `+0x750` (input buffer / 1.0 cardinals), `ftCo_Damage_OnEveryHitlag+0x5C` (SDI),
  `ftCo_DamageFall_IASA+0xCC` (tumble), `ftCo_80093240+0x54` (shield SDI), `ftCo_8009A080+0x38` (shield drop extended), `ftCo_800998A4` (shield drop, 113-code lists).
- Neutral Spawn: `fn_8016E730+0x1C` (266 lines) and the VS spawn code, `Fighter_Create+0x54` (Init Player Data), `ftCo_LandingAir_EnterWithLag+0x9C`,
  `Fighter_procMap+0xA8`, `ftCo_800AC5A0+0x18`.
- Stage codes: `grStadium_801D13E0/2344/4548/..` (Frozen Pokemon Stadium), `Ground_GetStageGObj+0x7C`, sudden death setup (`gm_SetupSuddenDeath`).
- Recording and menu codes with no sim effect: `Fighter_UnkCallCameraCallback_8006D9EC+0x48`, `gm_Scene_Vs_OnFrame+0x84`, `mnCharSel_*`, `mnStageSel_*`, `gmMainLib_*`.
- The 113-code lists (event 20689) add Camera, NameTag, DevText, CObjLoad and `ftLib_80086A8C` writes, not understood further.

What the sim already emulates (src: gmvs, ftCo_Turn/Damage/DamageFall/Guard/Escape/Pass, Fighter, grpstadium): UCF 0.84 and 0.73, Neutral Spawn,
Init Player Data zeroing, Frozen Stadium, Freeze Glitch Fix (Nana). Expected (not bugs) therefore only where a code is NOT emulated:
- UCF toggles other than 1 (0 off or 2 Dween) are not emulated (none in the corpus, all 125 are 1/1).
- Sudden death setup, Camera/Offscreen codes, `Ground_GetStageGObj+0x7C` and any stage-specific code are not emulated; Pokemon Stadium
  `y` divergences (10 of 18 ext3 files) and Fountain platform `y` (6 of 10 ext2 files) are the first suspects for Frozen-PS / platform
  code or stage state that the console starts differently, not individually verified.
- Dream Land 64 (ext28): the constant 0.2/frame extra push is Whispy Blow wind. The console's wind timer comes from stage-load RNG; the
  engine's stage load draws from our seed. Not a fighter bug. (Slippi 3.18 adds the Whispy Blow Direction event, absent in 3.17 files.)
- The 39 DamageFlyRoll vs DamageFlyHi/N/Lw mismatches: the Roll ("knockback spiral") choice is RNG dependent (the spec's Frame Start event
  exists to fix exactly this desync). The engine forces each frame's Frame Start seed, but the console's seeds in these files advance by exactly
  0x10000 per frame (online-style synthetic seeds), and the Pre-Frame seeds are not an LCG descendant of them within any small step count,
  so the number of draws before the damage code cannot be compared this way. Unresolved, listed as a blocker.
- Hit-connection timing (hitlag, percent, last_attack, 3.96 shield pushes): not explained by a Slippi code; unresolved, listed as a blocker.

## Blockers recorded (no progress within the timebox)
1. DamageFlyRoll vs DamageFlyHi/N/Lw choice (39 of 125 first divergences). Needs the number of RNG draws between Frame Start and the damage code in the
   console; Post-Frame data cannot give it. Candidates: Slippi's synthetic per-frame seed scheme vs `HSD_Rand` draws in effects/camera.
2. Hit-connection timing (hitlag/percent/last_attack/shield push, ~30 files): a hit connects a frame earlier or later than the console's. Start from
   a `--full` run on a 1-hit file and compare hitbox positions; character mix is wide (Fox, Marth, Sheik, Falco, Falcon, Peach, Puff, Ganondorf).
3. Pokemon Stadium and Fountain `y` (16 files) and the Dream Land wind (16 files): stage state at start.
4. The experimental `SLPCHECK_SEEDS=1` probe (RNG at each fighter's pre-frame point vs the recorded Pre-Frame seed) mismatches at the first frame after
   -123 in every file; the recorded seeds are not a few LCG steps from the Frame Start seed. Kept off by default.

## Known harness limits
- Followers (Ice Climbers) are not compared; a replay with Popo/Nana would still run (none in the corpus).
- The engine's match end (stock loss, time) stops the run even if the replay continues; the summary then shows `last_frame` < the replay's.
- Stops at the first divergence by design; `--full` shows later fields but later values are consequences.


## Round 2 (2026-11-02, port/replays): 0 -> 38 of 125 frame-exact

Same 125 files, `fast` profile, `tools/rust-port/slp/run_all.sh` at 2 jobs (about 5 min). Frame-exact = status `match` over the whole replay.

| step | fully matching | median first divergence (diverged files) |
|---|---|---|
| port/slporacle (start) | 0 | 1157 |
| lead 1: RNG reseeded from each fighter's Pre-Frame seed (69fcaf59c) | 32 | 1262 |
| + match seeded from the Game Start seed, no per-frame Frame Start reseed, BGM-pick draw (396345cfc) | 38 | 2322 |

What fixed what (all in `melee/src/slpcheck.rs`; the engine only gained a seed column in `MELEE_RAND_TRACE`):
- `Replay_ApplyMatch` (the seam answers 0) set the RNG to 0 at the match start, so the stage-init, fighter-create and stage-proc draws came from the
  wrong stream. slpcheck now answers the Game Start Random Seed. Dream Land's first-divergence `x` (Whispy wind) went from 18 files to 7.
- The Frame Start seeds of these files are synthetic (they advance by 0x10000 a frame). The console's RNG is a continuous LCG: consecutive
  Pre-Frame seeds are `k` LCG steps apart with `k` = the draws made in between. Forcing the Frame Start seed every frame therefore destroyed the
  stage's draws between the last fighter of frame N and the first fighter of N+1 (the stage procs run there). Not forced any more
  (`SLPCHECK_FS_RESYNC=1` restores it). Disproves the note above: the DamageFlyRoll choice (S1) was this, not a missing code.
- Game seed -> first Pre-Frame seed is 6 to 12 LCG steps; the engine matched that for Fountain, Yoshi's, Dream Land and was exactly one draw short on
  Pokemon Stadium, Battlefield and Final Destination (a stage with several BGM tracks draws once more); slpcheck adds the dummy draw (`MELEE_ENGINE_PRE_DRAWS=1`) for stages 3, 31, 32.
- Probe: `SLPCHECK_SEEDLOG=1` prints, per fighter pre-frame point, the LCG steps since the previous point in the console (`prev_to_rec`) and in the engine
  (`prev_to_eng`). After the fixes they are equal at every point up to each file's first divergence on all 87 diverging files (so no remaining
  divergence is an RNG draw-count problem). `SLPCHECK_DUMP=1` now also prints the recorded pre-frame stick (`PRE` lines).
- The Gecko lists of these tournament files do NOT contain WhispyBlowDirFix (8008653C); `Slippi_Codes == 2` (online) gates it in the sim, which is right.
  Present and affecting gameplay in every list: Init Stage Data (801C154C, zero 516 bytes), Init Player Data, Nana determinism and the Pokemon Stadium
  preload codes (801D14C8, 801D45EC, 801D460C, 801D4610, 801D4724, 801D4F14: transformation chosen at load with `HSD_Randi(4)`, `SkipNormalDecision`), the PS camera-independent monitor (801D24FC).
  Sources: slippi-ssbm-asm `Common/Preload Stadium Transformations`, `Online/Core/WhispyBlowDirFix`, `Common/Initialize Stage Data`, `Online/Core/Hacks/Stadium` (GPL-3.0, read only; nothing copied yet).

Remaining first divergences (87 files):

| class | files | notes |
|---|---|---|
| Pokemon Stadium `y` | 16 (+2 `x`) | the engine lands a falling fighter on a platform line at y=25.0001 (side platforms, x in 26..54 either side) that the console fighter falls through, at 25 to 3300 frames in. Spawn frame -33 in two files (p4 x=40). Stage state, not input (both ports hold down). Next: dump the engine's PS platform collision/animation phase against the Preload codes above; does Frozen PS keep the Normal form's platforms in a different state? |
| Fountain `y` | 8 (+2 `x`) | platform heights: the random walk in `grIzumi_801CC358` (draws match, values differ); check `rand_range` / `HSD_Randf` use and the first-pose heights |
| 1-ULP `x`/`y` (e.g. -30.344479 vs -30.344484) | about 25, all stages | float contraction sites where MWCC's fused ops differ from clang's (notes/fma.md); needs the per-function census melee-unlocked made (`MU_NO_CONTRACT` list). Candidate method: flip the site in the function that computes the field and re-run |
| `percent`/`hitlag`/`shield`/`action` | about 20 | hits a frame apart; usually after a 1-ULP drift, not yet separated |
| Dream Land `x` | 7 | 0.05 to 4 wide: not the wind any more |
| `x` jumps of 3.96 / 3.81 | 7 | shield push / knockback one frame apart |

Frame-exact on the benchmark file `20476_06200225` (Final Destination, 10071 frames, 20142 fighter-frames): see bench-unlocked.md.

## Round 3 (2026-11-05, port/slp-stages): 53 -> 61 of 125 frame-exact

Baseline re-measured at rust-port b3c5e1ead (fast profile, 2 jobs): **53** match, not the 38 above. The round-2 Pokemon Stadium `y`
(25.0001 side-platform landings) and Fountain `y` classes no longer appear as first divergences at that tip (0 files each; the
remaining ext3/ext2 first divergences are 1-ULP x/shield or the classes below). Not re-investigated: fixed by intervening commits.

Fix (d419eb03f): UCF 0.84 shield SDI (`ftCo_Ucf084ShieldSdi`, C ftCo_Guard.c TARGET_PC block and Rust ftCo_Guard.rs). The replay's
code at 0x80093294 tests last frame's stick x as `lfs f12,0x628(r4); fcmpu f12,f0; bge` - signed, no fabs - while the port used
|x|. A stick slammed left and held gets a second shield SDI on the next hitlag frame in the console (the 3.96 = 6 * 0.66 `x` jumps,
3.1-3.9 on slopes). Console-vs-port correction for UCF replays only (Replay_UcfVersion 84); no vanilla golden reads it.
Fixed every large-`x` first divergence: Dream Land `x` 5 -> 0 big ones, the 3.96/3.81 class 7 -> 0.

Remaining first divergences (64 files):

| class | files | notes |
|---|---|---|
| 1-ULP `x`/`y`/`shield`/`percent` | about 40 | FMA/contraction class, another agent's |
| ~~`percent` engine = replay + 1.0~~ (fixed, round 4) | 12, all 20689_* (113-code lists) | the offscreen 1%/interval damage (Fighter_8006A360 via ifMagnify) fires in the engine, not in the console. Those lists carry a widescreen camera code: 04 writes at Camera_80030BBC+0xC0/+0xCC (`li r0,100` / `li r0,540`: the on-screen x test uses 100..540 instead of the scissor), ftLib_80086A8C+0x98 nop, 803BB05C = 0x3EB00000 and more. Emulating it is a camera feature (C frozen: not done here) |
| `action` 38 vs 358, 41/40, 29/90; hitlag 5/8; percent 10 | 5 | not looked at |
| shield 49.38 vs 42.64 (ext8) | 1 | not looked at |

Goldens: classic_step04 and allstar_step04 re-recorded from a C build of this branch (the shield SDI fix fires in 1P
runs too, UCF being on outside replays). Reason: UCF 0.84 shield SDI previous-frame check is signed, as in the console code.

## Round 4 (2026-11-05, port/slp-stages): 61 -> 72 of 125 frame-exact

Slippi's widescreen camera code, Rust only, armed only when the replay's own Gecko list carries it
(`melee_data::slp::widescreen_code`, read from the code bytes; `melee_core::codes::set_replay_widescreen`, set by slpcheck and
`golden --slp` before the match boots). Its logic-visible parts:
- `C2 8036A4A8` (CObjLoad, replacing the perspective `lfs f1,0x34(r31)`): aspect = (aspect * 320) / 219 (`fmuls`, `fdivs`;
  the two floats follow the code's `blrl`). Ported in `melee_runtime::tu::cobj::CObjLoad` (the authored aspect the logic camera
  and `Camera_LogicToScreen` use).
- `04 80030C7C 38000064`, `04 80030C88 3800021C`: Camera_80030BBC's left/right scissor loads become 100 / 540. Ported in
  `Camera_LogicToScreen` (the off-camera flag `x221F_b0` that ifMagnify and the offscreen 1%/interval damage read).
- `04 80086B24 60000000` (ftLib_80086A8C+0x98, the `beq` after Camera_80030CFC): only the draw decision; the flag write is
  before it, so no logic effect. 803BB05C (lbbgflash's camera desc) and the ifMagnify/NameTag/DevText/menu codes: render only.
The 12 `+1.0 percent` files: 11 now match, 1 diverges later on a 1-ULP value. No C change (C frozen; replay-only path).

Remaining non-ULP first divergences (7): action 38/358 x2 (Dream Land), 41/40, 29/90, hitlag 5/8, percent +10, shield 49.4/42.6.
