# Slippi gameplay codes on the Rust engine (2026-11-27, branch port/slipcodes)

Vanilla Melee NTSC 1.02 + Slippi's own codes only (no m-ex, no mods). Source of truth: `project-slippi/slippi-ssbm-asm`
at `d0ce23a`, read-only in `_build/slippi-src/ssbm-asm` (GPL; nothing pasted here, behaviour described in our words):
`Output/InjectionLists/*.json` (address, tags), `Output/Netplay/GALE01r2.ini` (what Slippi Dolphin loads online), the
`.asm` sources. Hooked addresses mapped to retail functions with `config/GALE01/symbols.txt`; the retail code around
each hook was read from the user's own `main.dol` at run time (never committed). No disc bytes, code words or replays here.

## 1. Inventory

### 1.1 The netplay list (`list_netplay.json`: 310 entries; 204 asm, 72 raw 04, 34 raw C2; 29 tagged [affects-gameplay])

Groups: *Gen* = "Required: General Codes", *Rec* = "Required: Slippi Recording" (also in every console core list),
*Onl* = "Required: Slippi Online". "Gate" is what turns the engine's version on.

| # | code (asm) | grp | hook -> retail function | what it does | engine | how | gate |
|---|---|---|---|---|---|---|---|
| 1 | NeutralSpawn | Gen | 8016E510 fn_8016E2BC+0x254 | singles/teams spawn from a per-stage point table | `melee_modes/src/tu/gmvs.rs` (`neutral_spawn_table`) | hand port | codes `NEUTRAL_SPAWN` |
| 2 | UCF 0.84 DBOOC SquatRv Fix | Gen | 800D65EC ftCo_SquatRv_CheckInput+0x14 | squat-reverse on a 0.84 axis | `melee_fighter/src/tu/ftCo_SquatRv.rs` (`ucf_axis`) | hand port | UCF version 84 (codes `UCF` online; replay: C2 800D65EC listed) |
| 3 | UCF 0.84 Dashback | Gen | 800C9A44 ftCo_Turn_IASA+0x4C | dashback out of Turn on a fast flick | `ftCo_Turn.rs` (`ftCo_Turn_Ucf084Dashback`, 0.73 variant `ftCo_Turn_Ucf073Dashback`) | hand port | UCF |
| 4 | UCF 0.84 Pad Buffer + 1.0 Cardinals | Gen | 8006B460 Fighter_procInput+0x750 | per-port raw stick history; 1.0 on a cardinal | `melee_fighter/src/fighter.rs` (`ftucf_pad_*`), `melee_data/src/player.rs` (`ucf_cardinals`) | hand port | UCF; replays without this C2 run as 0.73 (`Replay::ucf_073_list`) |
| 5 | UCF 0.84 SDI | Gen | 8008E54C ftCo_Damage_OnEveryHitlag+0x5C | SDI uses the pad history | `ftCo_Damage.rs` (`ftCo_Ucf084Sdi`) | hand port | UCF |
| 6 | UCF 0.84 Shield Drop Extended | Gen | 8009A0B8 ftCo_8009A080+0x38 | extended shield-drop window | `ftCo_Pass` / shield-drop sites via `Replay_UcfVersion` | hand port | UCF |
| 7 | UCF 0.84 Shield Drop | Gen | 800998A4 ftCo_80099894+0x10 | shield drop on a down notch | same | hand port | UCF |
| 8 | UCF 0.84 Shield SDI | Gen | 80093294 ftCo_80093240+0x54 | shield SDI uses the pad history | `ftCo_Guard.rs` (`ftCo_Ucf084ShieldSdi`) | hand port | UCF |
| 9 | UCF 0.84 Tumble | Gen | 800908F4 ftCo_DamageFall_IASA+0xCC | tumble wiggle-out on a fast flick | `ftCo_DamageFall.rs` (`ftCo_Ucf084Tumble`) | hand port | UCF |
| 10 | FreezeGlitchFix | Gen | 801239A8 ftNn_Init_80123954+0x54 | IC freeze glitch (one nop) | `melee_chara/src/tu/ftnanaspecials.rs` | hand port | codes `FREEZE_FIX`, or Slippi codeset 2 / codeset 1 + version >= 2.0 |
| 11 | Init Stage Data | Rec | 801C154C Ground_GetStageGObj+0x7C | zero 516 bytes (the Ground) after alloc | `melee_world/src/tu/ground.rs` (`alloc_user_data_ground`) | hand port | unconditional (fresh `HSD_MemAlloc` block; equal to the vanilla result when the heap is fresh) |
| 12 | Reset Castle Wind | Rec | 801CD37C grCastle_801CD37C+0 | wind step 0x3C8EFA35, wind mode 1 (boot values) | `melee_world/src/tu/grcastle.rs` (`grCastle_801CD37C`) | **ported here** | `codes::STAGE_CASTLE_WIND` |
| 13 | Reset Kongo Log Bridge | Rec | 801D69B0 grKongo_801D69B0+0 | zero the 15 log velocities | `melee_world/src/tu/grkongo.rs` (`grKongo_801D69B0`) | **ported here** | `codes::STAGE_KONGO_LOG` |
| 14 | Init Player Data | Rec | 80068EEC Fighter_Create+0x54 | zero the Fighter after alloc | `melee_fighter/src/tu/fighter.rs` (Fighter_Create, `write_bytes`) | hand port | unconditional |
| 15 | BrawlOffscreenDamage | Onl | 8006A880 Fighter_procAnim+0x520 | offscreen = outside the stage camera bounds; never dead / state 4, 6 / home-run | `fighter.rs` (`ftSlippi_IsOffscreen`) | hand port | `Slippi_Codes() == 2` |
| 16 | FreezeDeadUpFallPhysics InitHitVelocity | Onl | 800D4C1C ftCo_DeadUpFall_Anim+0x214 | star-KO fall velocity in mv, self_vel zero | `ft_0d31.rs` | hand port | `slippi_codes() == 2` |
| 17 | FreezeDeadUpFallPhysics UpdateFallVelocity | Onl | 800D4D68 ftCo_DeadUpFall_Phys+0x80 | gravity / terminal velocity on mv | `ft_0d31.rs` | hand port | same |
| 18 | FreezeDeadUpFallPhysics UpdateModelPos | Onl | 80080E80 ftDrawCommon_80080E18+0x68 | camera-space point to the jobj only | `ftdrawcommon.rs` | hand port | same |
| 19 | DesyncProofBGTransformations | Onl | 8021AAE4 grLast_8021AAB0+0x34 | FD backdrop think between a seed save and restore | `melee_world/src/tu/grlast.rs` (`grLast_8021AAB0`) | hand port (already present; re-read against the asm, matches) | `Slippi_Codes() == 2` (grlast hook) |
| 20 | Stadium CustomZeroBuffer | Onl | 801C65C8 grAnime_801C65B0+0x18 | archive-slot release without lbArchive_80016EFC; skip an all-zero slot | `melee_world/src/tu/granime.rs` | **ported here** | `codes::STAGE_PS_FILES` |
| 21 | Stadium GrPsxIsValid | Onl | 801D4760 grStadium_801D4548+0x218 | case 1: "archive present" instead of "read finished" | `grpstadium.rs` (`grStadium_SlippiPsxIsValid`) | **ported here** | `STAGE_PS_FILES` |
| 22 | Stadium IngameCheckIfFrozen | Onl | 801D457C grStadium_801D4548+0x34 | frozen toggle (MSRB alt_stage_mode) in place of the training test | `grpstadium.rs` (`ps_freeze_at_entry`, `Slippi_FrozenStadium`) | hand port | online: `alt_stage_mode != 0`; replay: C2 801D457C |
| 23 | Stadium StadiumFileLoad | Onl | 800165AC lbFile_80016580+0x2C (only retail caller: 801D4754) | read the transformation file and set up its archive inside the call | `grpstadium.rs` case 0, `grdatfiles.rs` (`grDatFiles_SlippiStadiumArchive`) | **ported here** | `STAGE_PS_FILES` |
| 24 | WhispyBlowDirFix | Onl | 8008653C ftLib_GetOpponentsDir+0x94 | dead fighters (state <= 0xB) ignored by Whispy | `melee_fighter/src/ftlib.rs` | hand port | `Slippi_Codes() == 2` |
| 25 | NanaDeterminism | Onl | 800AC5B8 ftCo_800AC5A0+0x18 | r5 = r30 = 0 at entry: the near-zero-knockback shield-DI push is (0, 0) | `ftCo_0A01.rs` (`ftCo_800AC5A0`) | hand port, implicit and unconditional (the stick locals start at 0; retail's uninitialised r5/r30 values, mex.md Round 6, are not reproduced offline) | none |
| 26 | PSCameraIndependentMonitor | Onl | 801D24FC grStadium_801D2344+0x1B8 | monitor test on x [-120,120], y [-20,80] | `grpstadium.rs` | hand port | `ps_camera_independent_monitor` (online, replay C2) |
| 27 | PreventWobbling Init Air | Onl | 800DB880 fn_800DB790+0xF0 | wobble counter reset | `ftCo_Capture*.rs`, `ftCo_Damage.rs` | hand port | codes `NO_WOBBLE` |
| 28 | PreventWobbling Init Ground | Onl | 800DBBD4 fn_800DBAE4+0xF0 | same | same | hand port | same |
| 29 | PreventWobbling Wobble Check | Onl | 8008F090 ftCo_8008EC90+0x400 | 4th distinct partner hit breaks the grab | `ftCo_Damage.rs` (`inlineB2_wobble`) | hand port | same |

**Totals (netplay [affects-gameplay], 29):** hand port before this branch 24 (one of them, NanaDeterminism, implicit);
missing 5 (Castle wind, Kongo log, and three of the four Stadium hooks; FD DesyncProofBGTransformations was already
there); ported on this branch 5. Now 29 / 29 have an engine version.

Untagged codes of the list that still touch simulation state, and how the engine stands on them:
InitOnlinePlay / SyncRNG / the desync checksum (slippi-online-patches.md, `melee_core::slippi_online`, verified on 11,293
oracle frames); the pause / LRAS codes (InitPause, PauseCounter, PreventPauseStranding: the online sim never pauses,
`pause_allowed`); LGLExceededGameEnd (codes `LGL`, limit 45); FastForward DynamicsFix (no effect on a port that always
builds its matrices); the rollback driver (StartEngineLoop, LoopEngineForRollback, ... replaced by `melee_net`). The raw 04
/ C2 entries of "Required: General/Slippi Online" are menus, the scene loop and EXI/DVD plumbing; the two
"Recommended" sets (Normal Lag Reduction: pad polling; Lagless FoD: particles / reflections) and the "Optional" ones
(widescreen, screen shake, HUD, indicators, freeze FD background: render only, the FD one is the 04 that skips
grLast_8021B2E8, i.e. the `FROZEN_STAGES` FD site) carry no tag.

### 1.2 Gameplay codes of the other lists that replays carry

| code | list | hooks | engine | how / gate |
|---|---|---|---|---|
| UCF 0.84 (8 inserts, as above) | `list_console_UCF_084.json`, console tournament / stealth sets | as above | as above | replay: dashback/shield-drop words = 1 and C2 8006B460 present |
| UCF 0.8 (DB, SD, Tumble) | `list_console_UCF.json` | 800C9A44, 800998A4, 800908F4 | `ftCo_Turn_Ucf073Dashback`, 0.73 shield drop, tumble | replay lists without 8006B460 run as 0.73 (`ucf_073_list`); with C2 800908F4 the 0.84 pad-queue tumble rule |
| UCF 0.73 / 0.74 | pre-3.0 files (no list) | same three | 0.73 rules | `Replay_UcfVersion` 73 for pre-3.0 files |
| Frozen Pokemon Stadium | `list_console_stages_stadium.json` | 04 801D45FC (FreezePokemon), C2 801D457C (toggle) | `grpstadium.rs` (`Slippi_FrozenStadium`, `ps_freeze_at_entry`) | Game Start frozen byte; C2 801D457C |
| Frozen Stages (all) | `list_console_stages_all.json` | 803E67E0, 8021AAE4, 801D1548, 801E3348 | codes `FROZEN_STAGES` (grLast, grStory, grOldPupupu, grStadium) | opt-in codes word (gameplay-codes.md); a replay does not arm it from its list yet |
| Preload Stadium Transformations | console core (Rec) | 801D14C8, 801D45EC, 801D460C, 801D4610, 801D4724, 801D4F14 | `grpstadium.rs` (`PS_PRELOAD_*`) | replay list (`ps_preload_transformations`) |
| Ledge Grab Limit | `list_console_gameplay_lgl.json` | C2 801A5E90, 80165C48, 04 8016EBD8, 801B04xx | codes `LGL` (ranking in fn_80165AC0) | the sudden-death 04 edits are not ported (gameplay-codes.md) |
| Disable Wobbling | `list_console_gameplay_wobbling.json` | 800DB880, 800DBBD4, 8008F090 | codes `NO_WOBBLE` | replay: C2 800DB880 listed (slpcheck) |
| PSCameraIndependentMonitor, NanaDeterminism | console core (Rec) | as above | as above | as above |

## 2. The codes ported on this branch

The online mode turns them on in `slippi_np::online_codes` (all three bits of the new `melee_core::codes::SLIPPI_STAGE`
word), and a replay arms each one from its own Gecko list (`melee_data::slp::slippi_stage_word`: the C2 at the code's
address; the Stadium bit needs all three of its C2s), in `slpcheck` and `golden --slp` (`slp_open`). `restore_offline_codes`
clears the word. Offline (no Slippi) every site runs the game's own code, which is right: vanilla keeps these statics.
`SLPCHECK_SLIPPI_STAGE=off` runs a replay without them.

- **FD DesyncProofBGTransformations** (8021AAE4). Already in `grLast_8021AAB0`: the seed word is read before the
  backdrop think and written back after it when `Slippi_Codes() == 2`. The asm does exactly that (r31 holds the seed
  across `FinalDestination_BGTransformThink`, then r31 is reloaded from the gobj). No change.
- **Reset Castle Wind** (801CD37C, Peach's Castle: `grCastle_801CD37C` is its stage init). At the start of the init:
  `grCs_804D45E0` = f32 bits 0x3C8EFA35 and `grCs_804D45E4` = 1. The engine's statics start with those values, so a
  replay in a fresh process cannot move; it matters for the second game on the stage in one process (an online set).
- **Reset Kongo Log Bridge** (801D69B0, `grKongo_801D69B0`, Kongo Jungle's log bridge init): before the init's loop,
  `grKg_803E188C[i]` + 0x10 (the velocity, `unk10`) = 0.0 for i in 0..15. Same fresh-process remark.
- **Stadium file hooks** (StadiumFileLoad 800165AC, GrPsxIsValid 801D4760, CustomZeroBuffer 801C65C8). Retail loads a
  transformation with an async DVD read (`lbFile_80016580`, callback `fn_801D4220` clears the gp's `xC4_b1`), and
  case 1 of the transformation machine waits for that (`grStadium_801D42B8`: bit clear -> set up the archive, xD0).
  Online, the read and the archive set-up happen inside the load call, the callback never runs (`xC4_b1` stays set),
  and case 1 asks whether the map id's archive slot has an archive with a non-zero size. In the engine the DVD layer is
  synchronous (`HSD_DevComRequest`, consolidation.md), so both paths reach xDC = 2 on the same frame: the port changes
  how, not when. The archive set-up (`grDatFiles_SlippiStadiumArchive`) is grDatFiles_801C6478 without its
  RENDER_SHADOW pass, because Slippi's code hands `grDatFiles_801C6228` the address of the slot's head pointer instead
  of the pointer (on the console that pass reads past the 4-slot table and so does not flag the transformation's
  materials; render only). CustomZeroBuffer in `grAnime_801C65B0`: an all-zero slot is left alone, `unk8 == 1` frees
  unk0 to heap 0, any other slot is only zeroed (retail calls `lbArchive_80016EFC` when `unk8 == 0`). Retail's case 6
  passes the file buffer xCC there (the engine's TARGET_PC passes xD0; unchanged here, memory only).

Gameplay effect expected from the port: none on any existing replay (fresh-process statics, synchronous DVD). It is
there so the engine runs Slippi's code, and for a second game on Castle / Kongo in one process.

### 2.1 Checks run (fast build of this branch, `slpcheck --full`, headless, 3 at a time)
41 replays chosen from the sweep2 rows: all 7 online 3.x Pokemon Stadium files that diverge, 16 online PS files that
match, the only online Peach's Castle file (lunar), 6 online FD files (one diverging), 6 console PS files. Every one gave
the same status, first frame, port, field and engine value as the sweep2 row (exe 15faeb741): **41 same, 0 changed**.
No online 3.x Kongo Jungle replay exists in the sets (the three Kongo files are pre-3.0), so the Kongo reset is
checked by reading only. Loopback: not run for these codes (they cannot move a fresh-process match; see above).

## 3. Slippi 3.20.0 Dream Land (2026-11-27): the first frame's seed

**Symptom.** 130 Slippi 3.20.0 online replays (`_build/replays/lunar-2026-11-27/v320`): 19 of 20 Dream Land files
diverged (fighter x, frames 564-1061) and went hard; every other stage matched (Big Blue: section 4).

**Cause.** ssbm-asm `e03af7f` ("improve rng seed generation", 2026-11-01, first released in 3.20.0) changed SyncRNG
(Online/Core/InitOnlinePlay.asm): the per-frame seed is now murmur3 fmix32(rotl(F, 16) + rng_offset), XOR the seed
bits << 24. Before it the seed was rotl(F, 16) + rng_offset, so at global frame 0 (replay frame -123) it equalled
rng_offset, which is the Game Start block's seed. The replay player forced the Game Start seed on the first frame
(`Player::resync_seed`, FIRST_FRAME) and the Frame Start seed on every later one. From 3.20.0 the two differ on the
first frame (example file: Game Start 0x0000FD66, Frame Start -123 0xA07D18B8 = fmix32(0xFD66)). The seed is forced
again from frame -122 on, so fighters never saw it. Dream Land's Whispy does: grOldPupupu_802113E0 draws the first
wait of its wind cycle (state 0, `randrange(yp.xC, yp.x8)`) on that first frame and counts it down. With the wrong
draw, the wind started at a different frame (example: console first wind event at frame 938, recorded by
SendDreamlandInfo; the engine's push showed at 695, 0.2 in x). Fountain, Yoshi's Story, Stadium and the other stages
make no lasting draw on frame -123, so they matched.

**Fix** (`melee_data/src/player.rs`, `resync_seed`): an online replay that has a Frame Start seed for its first frame
uses it there. For 3.0-3.19 online files that value equals the Game Start seed (rotl(0,16) + offset), so nothing changes
for them. This is replay playback only; live online play was already right (`slippi_online` computes the seed for
frame 0 the same way as for every other frame).

**Checks** (fast build, `slpcheck --full`, headless):
- the 130 v320 files: Dream Land 20 / 20 match (19 were diverged), the other 110 stay as before (108 match on 12 other
  stages, Big Blue 2 still diverged). Re-scored with `hard_desync.py` (`--slippi-checksum-out`): 128 no hard desync, the 2 Big Blue
  files hard (frames 2231, 2332).
- gate: 125 replays (60 hero, 40 lunar2, 25 Dream Land 3.17-3.19 from both) give the sweep2 row unchanged (status,
  first frame / port / field / value, frames compared): **125 same, 0 changed**. Goldens and synctest: not run (the
  change is in replay playback of online files; offline scenarios and the synctest never read a replay seed).

### 3.1 The live online path and our own online replays (2026-11-27)

- Live SyncRNG (`slippi_online::seed_for_frame`, the proc `gmvs::slippi_sync_rng` at InitOnlinePlay's hook) already
  is e03af7f's `fmix32(rotl(g, 16) + rng_offset) ^ (seed_bits << 24)` on every frame, global frame 0 included. Proof
  from a loopback Dream Land match (rng_offset 0x6e40, 1390 frames, 0 desyncs): the engine's RNG state at the start of
  frames -122.. is exactly fmix32(rotl(g,16)+0x6e40) for g = 0, 1, 2 ... plus that frame's draws, i.e. frame -123 ran
  on the hashed g = 0 seed, as a 3.20.0 console does (console 0xFD66 file: -123 = 0xA07D18B8 = fmix32(0xFD66)).
- Bug found in the exporter, not the sim: `slp_export` wrote each frame's Frame Start seed as the tick-start RNG
  state (the previous frame's last draw) and the Game Start major scene as 2 (offline), so an online .slp the engine
  wrote (live `--slp`, `slippi-replay --slp`) did not play back as online and diverged (frame 886 in the loopback
  file). Fixed: in Slippi online mode Frame Start carries the SyncRNG seed of that frame and the major scene is 8.
  After the fix the re-exported loopback file has 1391/1391 seeds equal to the formula and `slpcheck --full` plays
  it back as online: match, 2782 fighter-frames. Checksums of the run are unchanged (export only).
- Whispy, direct: the trace replayed through `slippi-replay` (the live sim path) with the rng_offset and roster of
  console file 6b050266...9a4 (0x4B75, Fox vs Falcon): first wind side 2 at stage frame 1100, the console's (event
  0x40 at frame 976 = stage frame 1100; engine playback of the console file: the same). With 0xFD66 (Falco dittos)
  the live run gave side 2 at 1029 against the console's side 1 at 1062, and zero inputs move 0x4B75 too: the first
  wait is drawn after inputs reach the RNG, so only a run on the console's own inputs is comparable; the online-mode
  playback above is that comparison for our own match. `MELEE_WHISPY_LOG=1` prints each wind start.
- lunar2's 106 3.20.0 files (`_build/replays/lunar-2026-11-20`, incl. the 6 Dream Land hard desyncs of the
  pre-fix sweep: 6afd2e79..bb2/bb6, 6affeb47..bbc/bc8/bcc/bde): `slpcheck --full`, 106/106 match (DL 6, stages
  2/3/8/31/32 the rest); hard_desync.py: 0 hard.

## 4. Big Blue (ext stage 24): the 3 failures classified (2026-11-27)

Engine `cur_pos` (slpcheck `--wide-out`) against the replay's post-frame x/y, per frame, port 1 (the only one off):

| file | Slippi | first frame | class | what the frames show |
|---|---|---|---|---|
| lunar2 6ab6c163...72a57258 | 3.19.1 | 459 y | **drift** | grounded Wait (motion 14) on floor line 216, whose normal turns every frame (a tilting / moving surface). x exact; y off by 138 ULP at 459, then 400, 788, 1440, 2498 ... ULP, smooth and growing, no action change. The surface's own geometry (the animated joint that line 216 follows) drifts, not the fighter. Candidate: float contraction / matrix order in the line's animated joint (the JObj fast-matrix path and the HSD_MtxInverse fusion mismatches in dol-audit.md, owner lead 1 is the same class). |
| v320 6b07dda9...daec8 | 3.20.0 | 2229 x | **jump** | action 191 to Fall (29) on frame 2229 in both, leaving floor line 263. The console's fighter is carried on: x -0.063, then -1.12, -1.46 per frame; the engine's keeps only its own -0.06 / -0.08. A moving surface's velocity handed to the fighter as it leaves (or the line moving under it on the release frame) is missing. Not RNG. |
| v320 6b09281d...12d593 | 3.20.0 | 2332 y | **jump** | the fighter falls (29) at 0.68 / -1.6 per frame in both; on 2332 the engine's lands on floor line 156 (motion 42, y snapped up 5.78) while the console's keeps falling. A surface is where the console has none at that frame: a car (spawn timing / position, the RNG candidate) or a moving line's position. |

So no shared cause: one drift on a tilting surface, two jumps on moving surfaces (velocity carry; a line present
where the console has none). Online files force the RNG seed each frame, so a car-spawn RNG difference would need a
draw-order difference inside a frame, not a seed one. Not fixed in this round.

## 5. Owner leads (recorded; not worked this round except 4)

1. Hurtboxes follow animated bones: check hurtbox bone world matrices on frames 3051-3053 of the lunar2 Peach vs
   Peach Bob-omb case (Mewtwo's tail is the same class). Suspects: the JObj fast_matrix path
   (`melee_runtime/src/tu/jobj.rs`) and the HSD_MtxInverse fusion mismatches in dol-audit.md. Big Blue file 1 above
   may be the same class.
2. UCF is public code: apply its real implementation (the replay's own Gecko bytes, `tools/mex-import/src/gecko.rs`
   applies a list to the guest image), never a hand rule.
3. Variants in the corpora (Hax$'s 1.03, newer UCF): listed per replay by the Gecko list hash in slpcheck's CSV
   (`gecko_codes`, `gecko_hash`); step 6 (each replay's own list in slpcheck) is where they would be run.
4. NeutralSpawn: its hooks are at fn_8016E2BC (match start) only; it writes the player blocks' initial coordinates and
   facing (PlayerBlock_StoreInitialCoords, UpdateFacingDirection). It does not touch the respawn slot table or
   fn_80167638, so it does not explain the respawn-slot case or one-ULP respawn heights.

## 6. Step 5 verdicts: the netplay list on the guest image

`tests/gecko_apply.rs` applies Slippi's `Output/Netplay/GALE01r2.ini` (d0ce23a, 6 enabled groups) to main.dol: 86
writes, 198 inserts, 0 unsupported types, 5 .bss writes skipped (0x80452F54..64), 1 hook overridden by a later code
(0x8016EA30). Per [affects-gameplay] code (section 1.1's 29): **all 29 hooks patched and structurally checked** (hook
branch, return branch). **None executed against the engine**: each hooked function takes Fighter / Ground / GObj
records or engine globals, which needs the native-records mode (step 4), not built. Verdict for every one of the 29:
"applied to the image, not run". The engine-side verdicts stay those of section 1.1 (24 hand ports before this branch,
5 added here) and the replay checks of sections 2.1 and 3.

## 7. The 29 codes run against the engine (2026-11-27, port/livereferee; referee.md section 10)

**How these were run.** Slippi's `Output/Netplay/GALE01r2.ini` (d0ce23a) was applied to the guest image
(`MELEE_REFEREE_GECKO=ini:...`). The referee then checked each hooked function with that patched code, in Slippi online
mode, on the states of six hero online replays: PS (3), FD (32), DL (28), FoD (2), BF (31), and an
IC game on FD. All six play as "match". The counts below are compared calls; "matches" means 0 mismatches.

| # | code | hooked function | verdict |
|---|---|---|---|
| 1 | NeutralSpawn | fn_8016E2BC (match start) | not reachable: match setup (creates players, allocates) |
| 2 | UCF 0.84 DBOOC SquatRv | ftCo_SquatRv_CheckInput | matches (482; 5 not checkable: the code stores into its own block) |
| 3 | UCF 0.84 Dashback | ftCo_Turn_IASA | matches (678 + 175 on lunar2 ...f1243f34; 203 not checkable: ftData_SpecialS .data read in SpecialS_CheckInput) |
| 4 | UCF 0.84 Pad Buffer + 1.0 Cardinals | Fighter_procInput | not run (not hooked: the function is large). Its buffer is handed to codes 2/3/5/6/8/9 from the engine's ftUcf_Pad, so their matches include it |
| 5 | UCF 0.84 SDI | ftCo_Damage_OnEveryHitlag | matches (11,417) |
| 6 | UCF 0.84 Shield Drop Extended | ftCo_8009A080 | matches (255) |
| 7 | UCF 0.84 Shield Drop | ftCo_80099894 | matches (4) |
| 8 | UCF 0.84 Shield SDI | ftCo_80093240 | matches (407) |
| 9 | UCF 0.84 Tumble | ftCo_DamageFall_IASA | matches (2,139; 9 not checkable: Fighter.dat_attrs) |
| 10 | FreezeGlitchFix | ftNn_Init_80123954 | not checkable (7 calls: Fighter.dat_attrs, a void*) |
| 11 | Init Stage Data | Ground_GetStageGObj | not reachable: allocation at stage setup |
| 12 | Reset Castle Wind | grCastle_801CD37C (stage init) | not reachable: stage setup; no 3.x Castle game in the run |
| 13 | Reset Kongo Log Bridge | grKongo_801D69B0 (stage init) | not reachable: stage setup; no 3.x Kongo replay exists |
| 14 | Init Player Data | Fighter_Create | not reachable: allocation |
| 15 | BrawlOffscreenDamage | Fighter_procAnim | matches on 1,333; 20,903 not checkable (camera .bss 0x80452F24) |
| 16 | FreezeDeadUpFallPhysics InitHitVelocity | ftCo_DeadUpFall_Anim | matches (93; 1 not checkable) |
| 17 | FreezeDeadUpFallPhysics UpdateFallVelocity | ftCo_DeadUpFall_Phys | **differed**: fp+0x2348 1 ULP over on 3 frames of hero Game_20260321T214020 (5 of 94 calls before the fix). stfs truncates the double; the port rounded. **Fixed** (f6a7f2166): 94/94 |
| 18 | FreezeDeadUpFallPhysics UpdateModelPos | ftDrawCommon_80080E18 | matches on 1,333; 22,667 not checkable (camera .bss 0x80452C68) |
| 19 | FD DesyncProofBGTransformations | grLast_8021AAB0 | not checkable (`selected_stage` .data read through Stage_80225194) |
| 20 | Stadium CustomZeroBuffer | grAnime_801C65B0 | not run (archive release: file layer) |
| 21 | Stadium GrPsxIsValid | grStadium_801D4548 | not checkable (gm_GetCurrentGameMode .bss 0x80479D30) |
| 22 | Stadium IngameCheckIfFrozen | grStadium_801D4548 | same as 21 |
| 23 | Stadium StadiumFileLoad | lbFile_80016580 | not run (DVD / file layer) |
| 24 | WhispyBlowDirFix | ftLib_GetOpponentsDir | not run (no hook: the Rust function sits under another name in ftlib.rs) |
| 25 | NanaDeterminism | ftCo_800AC5A0 | not checkable (126 calls: a store into Fighter+0x1FDC in ftCo_800B46B8, a field without a native counterpart) |
| 26 | PSCameraIndependentMonitor | grStadium_801D2344 | matches (2,000 on hero PS Game_20251031T214231) |
| 27 | PreventWobbling Init Air | fn_800DB790 | not reached (0 calls in these replays) |
| 28 | PreventWobbling Init Ground | fn_800DBAE4 | matches (19) |
| 29 | PreventWobbling Wobble Check | ftCo_8008EC90 | matches on 6; 358 not checkable (audio driver .bss through HSD_AudioSFXKeyOffTrack) |

**Totals.** Of the 29 codes:
- 13 match, counting partial coverage: 2, 3, 5, 6, 7, 8, 9, 15, 16, 18, 26, 28, 29;
- 1 differed and is now fixed: 17;
- 6 not checkable yet: 10, 19, 21, 22, 25, and most of 15, 18 and 29;
- 9 not run or not reachable: 1, 4, 11, 12, 13, 14, 20, 23, 24 (plus 27, which got no calls).

### 7.1 The UCF versions replays carry

Survey: `geckoscan` (scratch only) over the regress set, hashing each replay's C2 800C9A44 block. Hashes are listed, no
bytes.

| variant | where | count |
|---|---|---|
| 0.84 (block 40c7d5d7c5, with 8006B460) | 3.19 / 3.20 | 974 |
| 0.84-era (c2939c27ba, with 8006B460) | 3.17 | 341 |
| **UCF 0.8 `DB.asm`** (ba5f4a85dc, no 8006B460) | 3.17 lunar / lunar2 | 27 |
| other (513936be9f) | 3.17 | 1 |
| no list | pre-3.0 | 192 |

0.73 / 0.74 (pre-3.0 files, no Gecko list) cannot be run from the replay's own bytes. Hax$'s 1.03 is not in this corpus
as a separate list.

**lunar2 Fox ditto ...f1239a6e (3.17.0, Fountain)**, run with its own list: at slippi 3795, retail plus its UCF 0.8
insert grants the smash turn, and the engine's 0.73 rule did not. The difference is not in the threshold:
- 0.8 tests the animation frame (fp+0x894) == 2.0, where the 0.73 rule used the command-script frame's upper half;
- 0.8 uses |stick x|, where the 0.73 rule used stick x * facing_after;
- 0.8 reads the raw stick of fp+0x618, the controller port.

**Fixed** (de86fec54): `slp::ucf08_dashback_code` recognises the insert's form from its own instructions. After the
fix the replay matches the console to its end, and 276/276 Turn_IASA checks match.

The 0.84 lists (lunar2 Peach ditto, own list): all 175 Turn_IASA calls match, and the other UCF hooks match too
(section 10.1 of referee.md).

### 7.2 Verdict table with versions, evidence and score (2026-11-28, port/codesreview)

Written from sections 1.1, 2, 6, 7 above, `referee.md` section 10, and the git history of `_build/slippi-src/ssbm-asm`
(1,355 commits, HEAD `d0ce23a`). No run was made for this section.

**Versions.** The replay spec number is `CURRENT_VERSION` in ssbm-asm `Recording/Recording.s`. Its bumps:
3.18 = bb01e0e (2025-01-27), 3.19.0 = aac7d76 (2025-08-11), 3.19.1 = 30ca204 (2026-04-29), 3.20.0 = 4497d56
(2026-10-20); 3.17 is the spec from 2024-06-30 (c852afa). The Ishiiruka release that shipped each code variant was
read from each tag's `Data/Sys/GameSettings/GALE01r2.ini` (blocks hashed per hooked address, tags 3.4.1 to 3.7.0):

| spec | Ishiiruka releases | what differs in the tagged gameplay codes |
|---|---|---|
| 3.17 | 3.4.1 (2024-07-09), 3.4.2, 3.4.3 | 22 codes common to all versions + FreezeStadium + 6 Preload Stadium Transformations hooks (legacy set) |
| 3.18 | 3.4.4 (2025-01-27), 3.4.5, 3.4.6 | same set as 3.17 (bytes identical) |
| 3.19.0 | 3.5.0 (2025-08-12), 3.5.1 (08-13), 3.5.2 (2026-01-14) | legacy 7 gone; Stadium GrPsxIsValid, IngameCheckIfFrozen, StadiumFileLoad from 3.5.0; CustomZeroBuffer and the FD DesyncProofBGTransformations C2 from 3.5.1 (3.5.0 still has the FD 04 nop); Nana v2 from 3.5.2 |
| 3.19.1 | 3.6.0 (2026-04-29) to 3.6.4 | same as 3.5.2 |
| 3.20.0 | 3.7.0 (2026-11-17) | + Reset Castle Wind, Reset Kongo Log Bridge (d0ce23a, 2026-11-02) |

"Variants" below: **S** = compiled bytes identical in every Ishiiruka tag from 3.4.1 to 3.7.0 (every version). The
replay block hash for UCF differs between 3.17 (c2939c27ba) and 3.19/3.20 (40c7d5d7c5), see 7.1, yet the compiled
netplay C2 at 800C9A44 is the same in every tag, so that hash difference is not a change of the netplay code. Why the
corpora hash differently is not explained (open item O12).

Verdict words: **checked-match** (referee, calls compared with 0 mismatches, on the 3.20 codeset d0ce23a against the
states of the six hero online replays), **checked-mismatch** (found, fixed, rechecked), **not checkable**, **not run**.

| # | code | what it does | port | referee verdict | evidence | variants |
|---|---|---|---|---|---|---|
| 1 | NeutralSpawn | spawn points from a per-stage table | hand port (`NEUTRAL_SPAWN`) | not run (match setup) | referee cannot reach fn_8016E2BC; indirectly, start positions agree on every matching replay | S |
| 2 | UCF 0.84 DBOOC SquatRv | squat-reverse on 0.84 axis | hand port | checked-match | 482 calls, 0 mismatches; 5 not checkable | S |
| 3 | UCF 0.84 Dashback | dashback out of Turn on fast flick | hand port (+ 0.8 `DB.asm` recogniser) | checked-match (a 0.8-form mismatch fixed in de86fec54) | 678 + 175 (lunar2 ...f1243f34) calls, 0 mismatches; 203 not checkable. Fox ...f1239a6e (UCF 0.8): 1 mismatch, 276/276 after the fix | S |
| 4 | UCF 0.84 Pad Buffer + 1.0 Cardinals | raw stick history, 1.0 on cardinals | hand port | not run (Fighter_procInput not hooked) | its buffer feeds 2, 3, 5, 6, 8, 9, whose matches include it (indirect only) | S |
| 5 | UCF 0.84 SDI | SDI from pad history | hand port | checked-match | 11,417 calls, 0 mismatches | S |
| 6 | UCF 0.84 Shield Drop Extended | longer shield-drop window | hand port | checked-match | 255 calls | S |
| 7 | UCF 0.84 Shield Drop | shield drop on down notch | hand port | checked-match (thin) | 4 calls | S |
| 8 | UCF 0.84 Shield SDI | shield SDI from pad history | hand port | checked-match | 407 calls | S |
| 9 | UCF 0.84 Tumble | tumble wiggle-out | hand port | checked-match | 2,139 calls; 9 not checkable (Fighter.dat_attrs) | S |
| 10 | FreezeGlitchFix | IC freeze glitch (one nop) | hand port (`FREEZE_FIX`) | not checkable | 7 calls, all touch Fighter.dat_attrs (void*) | S |
| 11 | Init Stage Data | zero 516 bytes of the Ground after alloc | hand port | not run (allocation at stage setup) | none | S |
| 12 | Reset Castle Wind | reset two stage statics | ported 2026-11-27 | not run (stage setup) | no 3.x Castle game in the run | 3.20 only (absent 3.17 to 3.19) |
| 13 | Reset Kongo Log Bridge | zero 15 log velocities | ported 2026-11-27 | not run (stage setup) | no 3.x Kongo replay exists | 3.20 only |
| 14 | Init Player Data | zero the Fighter after alloc | hand port | not run (allocation) | none | S |
| 15 | BrawlOffscreenDamage | offscreen = outside camera bounds | hand port | checked-match (thin) | 1,333 calls, 0 mismatches; 20,903 not checkable (camera .bss) | S |
| 16 | FreezeDeadUpFall InitHitVelocity | star-KO fall velocity | hand port | checked-match | 93 calls; 1 not checkable | S |
| 17 | FreezeDeadUpFall UpdateFallVelocity | gravity on mv | hand port | checked-mismatch, fixed f6a7f2166, rechecked: checked-match | 5 of 94 differed (stfs truncates the double, port rounded); 94/94 after | S |
| 18 | FreezeDeadUpFall UpdateModelPos | camera-space point to the jobj | hand port | checked-match (thin) | 1,333 calls; 22,667 not checkable (camera .bss) | S |
| 19 | DesyncProofBGTransformations | FD backdrop think between seed save / restore | hand port (`Slippi_Codes()==2`) | not checkable | `selected_stage` .data read via Stage_80225194 | 3 variants: 3.17, 3.18 and Ishiiruka 3.5.0: plain 04 nop of the call (backdrop frozen, no RNG draw); 3.5.1 and later: the save / restore wrapper |
| 20 | Stadium CustomZeroBuffer | archive-slot release, skip zero slot | ported 2026-11-27 | not run (file layer) | replay checks only (section 2.1: 41 same, 0 changed) | 3.5.1 and later; absent before |
| 21 | Stadium GrPsxIsValid | case 1: archive present | ported 2026-11-27 | not checkable | gm_GetCurrentGameMode .bss | 3.5.0 and later |
| 22 | Stadium IngameCheckIfFrozen | frozen toggle | hand port | not checkable (same as 21) | same | 3.5.0 and later; 3.17/3.18 have FreezeStadium instead (always frozen online) |
| 23 | Stadium StadiumFileLoad | read + set up the archive inside the call | ported 2026-11-27 | not run (DVD layer) | replay checks only | 3.5.0 and later |
| 24 | WhispyBlowDirFix | dead fighters ignored by Whispy | hand port | not run (no hook: Rust fn under another name) | none | S |
| 25 | NanaDeterminism | r5 / r30 = 0 at entry | implicit, unconditional (the v2 behaviour) | not checkable | 126 calls, store into Fighter+0x1FDC | 2 variants: v1 (only r30 = 0) in 3.4.1 to 3.5.1; v2 (r5 and r30 = 0, 95fb888 2025-09-11) from 3.5.2 |
| 26 | PSCameraIndependentMonitor | monitor test on fixed x / y box | hand port | checked-match | 2,000 calls (hero PS Game_20251031T214231) | S |
| 27 | PreventWobbling Init Air | wobble counter reset | hand port (`NO_WOBBLE`) | not run (0 calls reached) | none | S |
| 28 | PreventWobbling Init Ground | wobble counter reset | hand port | checked-match (thin) | 19 calls | S |
| 29 | PreventWobbling Wobble Check | 4th distinct partner hit breaks the grab | hand port | checked-match (thin) | 6 calls; 358 not checkable (audio .bss) | S |

**Legacy gameplay set of Slippi 3.17 / 3.18** (7 more `[affects-gameplay]` hooks in Ishiiruka 3.4.1 to 3.4.6, gone
from 3.5.0; not among the 29 of HEAD, and no referee run used them, since the run used d0ce23a's list):

| code | address | what it does | port | verdict | evidence |
|---|---|---|---|---|---|
| FreezeStadium (`Online/Core/PreventFileAlarms/FreezeStadium.asm`) | 801D4578 | online PS never transforms (skips the decision) | `frozen-ps` from the replay's Game Start byte; nothing keys on this C2 | not run | replay checks only |
| Preload Stadium Transformations (Init isLoaded Bool, Load Transformation, GetPreloadedTransition, SkipNormalDecision1, SkipNormalDecision2, Reset isLoaded) | 801D14C8, 801D45EC, 801D460C, 801D4610, 801D4724, 801D4F14 | preload the transformation files at boot | `grpstadium.rs` `PS_PRELOAD_*`, armed from the replay's list | not run | replay checks only |

**Variant gaps in the port.** The engine runs one variant per code: the 3.20 one. Two cases where an older version's
code is different and the port's equality with it is argued, not checked:
- FD (19): 3.17, 3.18 and 3.5.0 skip the backdrop think (no RNG use); the port runs it and restores the seed. Equal if
  the think changes nothing gameplay-relevant besides the seed (it animates the backdrop). Not run on a 3.17 / 3.18 FD
  replay in this review.
- Nana (25): v1 leaves r5 as the retail code left it. The port's zero is v2. They differ only in an uninitialised
  register, which is the bug the code fixes.

**Not in the 29 but gameplay and untagged:** SyncRNG / InitOnlinePlay (e03af7f: 3.20.0 changes the seed to
fmix32(rotl(F,16) + offset); 3.19.x uses rotl(F,16) + offset). The port has both: the live path 3.20 only, the replay
path both (section 3). Checked by replay (130 v320 files, 128 no hard desync), not by the referee.

**Score (tier 1 = Slippi 3.x: 3.17, 3.18, 3.19.x, 3.20).**
- A code counts only with a checked-match referee verdict and one variant across every version that carries it.
- Counting 2, 3, 5, 6, 7, 8, 9, 15, 16, 17, 18, 26, 28, 29: **14 of 29 = 48%**. All 14 are variant S (same bytes in
  every Ishiiruka tag), so the check on the 3.20 list covers all versions.
- Thin cover, counted but flagged: 7 (4 calls), 15 (1,333 of 22,236 calls), 18 (1,333 of 24,000), 28 (19), 29 (6 of
  364). Without those five: **9 of 29 = 31%**.
- Over the 36 distinct codes any supported version applies (29 + 7 legacy): 14 of 36 = 39%.
- The codes with a version-dependent variant (19, 20 to 23, 25, plus 12 and 13 by absence): 0 of 8 checked on any
  variant.

### 7.3 Open items (every code without a checked-match verdict)

| # | codes | status now | what it would take to check |
|---|---|---|---|
| O1 | 1 NeutralSpawn | not run | a referee hook on fn_8016E2BC with the player blocks and spawn tables in the sync table; or a Dolphin-oracle run comparing frame -123 positions with `--codes neutral-spawn` on and off (replays agree indirectly already) |
| O2 | 4 Pad Buffer + Cardinals | not run | hook Fighter_procInput (large): native-records for the Fighter pad block plus the 0x8046B108 PAD ring; or compare `ftUcf_Pad` with the retail block's output on replay stick streams in a unit harness |
| O3 | 10 FreezeGlitchFix, 25 Nana (and part of 3, 9) | not checkable (`dat_attrs` void*, a field with no native counterpart) | map Fighter.dat_attrs to the kind's attribute record in remap; give Fighter+0x1FDC a shadow word (referee.md 10.3 per-kind unions) |
| O4 | 19 FD DesyncProof (3 variants) | not checkable | sync `selected_stage` (.data through Stage_80225194) so the grLast hook runs; then run the nop variant (Ishiiruka 3.4.6 ini via `MELEE_REFEREE_GECKO=ini:`) and the 3.5.1+ wrapper on an FD replay of each version |
| O5 | 21, 22 Stadium | not checkable | sync gm_GetCurrentGameMode (.bss 0x80479D30) and the gm scene block |
| O6 | 11, 14 Init Stage / Player Data; 12, 13 Castle / Kongo | not run, not reachable | allocation and stage setup: a loopback run of two games in one process (the codes matter for the second game), comparing the zeroed Ground / Fighter bytes with retail's after the call; needs a Kongo and a Castle 3.20 replay (none in the sets) |
| O7 | 20, 23 Stadium file hooks | not run | the DVD / lbFile layer is synchronous in the port: compare archive slot state after the call in a harness, or accept replay evidence |
| O8 | 24 WhispyBlowDirFix | not run | register `ftLib_GetOpponentsDir` under its Rust name in `referee_native!` (the hook is missing, not the function) |
| O9 | 27 PreventWobbling Init Air | not reached | pick replays with an air chain-grab; count calls of 800DB880 in the slpcheck report |
| O10 | thin: 7, 15, 18, 28, 29 | thin | sync the camera .bss (0x80452C68, 0x80452F24) and the audio driver; then 15 and 18 gain about 20,000 calls each |
| O11 | legacy 7 (FreezeStadium + 6 Preload) | not run | run the referee with Ishiiruka 3.4.6's ini on a 3.17 / 3.18 PS replay (same Stadium syncs as O5); also confirm that a 3.17 / 3.18 replay's frozen PS reads as 1 (the engine reads Game Start byte 0x1A2 and ignores C2 801D4578) |
| O12 | UCF replay block hash 3.17 vs 3.19 | open | decode one 3.17 C2 800C9A44 block (c2939c27ba) and compare with the 3.4.6 ini bytes |
| O13 | Nana v1, FD nop variants | open | find 3.17 to 3.5.1 replays that hit the Nana situation (Nana DI of a near-zero-knockback throw); FD replays of 3.17 / 3.18 for O4 |

