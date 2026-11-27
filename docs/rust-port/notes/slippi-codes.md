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
