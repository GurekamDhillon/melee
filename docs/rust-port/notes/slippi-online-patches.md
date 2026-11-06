# Slippi online patches: checksum, RNG seed and gameplay codes (2026-11-06, port/slippi-p1)

Specification in our own words, read from `project-slippi/slippi-ssbm-asm` commit `d0ce23a` (2026-11-02), cloned
read-only into `_build/slippi-src/ssbm-asm` (GPL; not in our repo, nothing pasted). The Dolphin clone's online codeset
(`_build/dolphin-oracle/src/Data/Sys/GameSettings/GALE01r2.ini`, Dolphin `0653df819`) contains the compiled form of the
new seed scheme (the murmur constants `0x85EBCA6B` / `0xC2B2AE35` appear at INI lines 1576-1579), so it is built from
`e03af7f` ("improve rng seed generation", 2026-11-01) or later. **Older Slippi clients used a different seed rule**
(no hash, no pad bits; see unlocked-survey.md: `rotl(f,16) + rng_offset`): the rule below is version-specific.

Terms: *ODB* = the online data buffer the ASM allocates per game; *MSRB* = the match-state reply Dolphin hands the game
(`CMD_GET_MATCH_STATE`); *global frame* = `gm_80479D58.unk_8` (0x80479D60), the scene's unpaused-frame counter (reset
to 0 at scene start, +1 at the end of every unpaused logic iteration).

## 1. Per-frame RNG seed

### 1.1 Start of game (InitOnlinePlay, hook 0x8016E748 in StartMelee, right after `db_Setup`)
1. `rng_offset` = MSRB u32 (Dolphin: the decider's `generator() % 0xFFFF`, from MATCH_SELECTIONS). Stored in the ODB.
2. The seed word (0x804D5F90, `*HSD_RandSeedPtr`) is set to `rng_offset`.
3. A GObj is created: classifier 4, p_link 7 ("on-pause: don't run"), priority 0 -- the call is `GObj_Create(4, 7, 0)`;
   then a proc `SyncRNG` is added on it with s_link priority 0. It runs inside `HSD_GObj_RunProcs` every unpaused frame,
   in s_link 0 before the gobjs created later in that p_link order (fighters are created after it).

### 1.2 SyncRNG (every frame, inside RunProcs)
```
F     = global frame (u32)
x     = rotl32(F, 16) + rng_offset            (wrapping)
x    ^= x >> 16;  x *= 0x85EBCA6B;            (murmur3 fmix32, wrapping u32 multiplies)
x    ^= x >> 13;  x *= 0xC2B2AE35;
x    ^= x >> 16
x    ^= seed_bits[F & 15] << 24
seed  = x                                      (overwrites, does not chain from last frame)
```

### 1.3 Seed bits (TriggerSendInput section 11, hook 0x80376A28 in HSD_PadRenewRawStatus after PAD_Read)
Each pad report is 12 bytes. Bytes 0-7 are the game's PADStatus (u16 buttons BE, s8 stick x, s8 stick y, s8 c x, s8 c y,
u8 L, u8 R). **Byte 8 carries 8 random bits** that travel with the pad (Dolphin draws them: `RXB_NEXT_PAD_RNG_BITS`,
`Common::Random::GenerateValue<u8>()`; the local client stamps them into its own outgoing pad, so byte 8 of every port's
pad for frame F is the bits its owner sent with that frame). Byte 8 is cleared before the game sees the pad. Byte 11 is
only a rollback marker in the predicted-pad history.

For the frame F handed to the game (including resimulated frames):
1. `bits32` = byte 8 of port p's pad at bit position 8*p, p = 0..3 (empty slots contribute 0).
2. `b_held` mask = ports whose buttons have B (0x0200). `prev` = `b_held_hist[(F-1) & 15]`; store `b_held_hist[F & 15]`.
3. A port is **rolling** when B is freshly pressed (held now, not in `prev`) and its external character is
   - Peach (0x0C) with raw stick y <= -44, or
   - Game & Watch (0x03) or Luigi (0x07) with |raw stick x| >= 48.
4. If nobody rolls: `seed_bits = 0`. Otherwise `ports_used` = all four ports, minus the rolling port when exactly one
   port rolls; `seed_bits = XOR of the bytes of the ports in ports_used`.
5. Store `seed_bits_hist[F & 15]`. A predicted remote pad whose byte went into the mix is marked so the frame always rolls
   back once the real pad arrives (the bits of a prediction are not real), even if the inputs matched.

So on almost every frame the seed is a pure function of (F, rng_offset); the pad bytes matter only on a fresh
turnip/hammer/misfire press. A Rust peer must still send real random bytes, and must use the peer's bytes when mixing.

### 1.4 Other seed writers online
- FD background (hook 0x8021AAE4): the seed is saved before `FinalDestination_BGTransformThink` and restored after, so
  FD's background never consumes RNG.
- After a game, the online scene asks Dolphin for a new seed (`CMD_GET_NEW_SEED`) for the menus (random character).
- No online patch calls `HSD_Randi` itself.

## 2. Desync checksum (StartEngineLoop, hook 0x801A4DE4 at the top of the engine loop in gm_801A4D34)

### 2.1 When
At the start of each engine-loop iteration, after any rollback load, with `F` = global frame: the checksum is taken of
the state **entering** frame F (i.e. after the iteration whose global frame was F-1) and stored under frame F in a ring of
21 entries (`ROLLBACK_MAX_FRAME_COUNT` 7 x 3); a resimulated frame overwrites its entry. Each entry also stores
the timer seconds (0x8046B6C8) and per-port percent (PlayerBlock_LoadDamage) and stocks for ranked desync recovery.
The pad sent for frame F+delay carries `(finalized_frame, checksum[finalized_frame])` (TXB, then Dolphin's
SLIPPI_PAD packet `checksum_frame`, `checksum`).

### 2.2 Formula
```
cs = 0 (u32), fsum = 0.0 (f32)
for slot in 0..4:                                 player slots at 0x80453080 + slot*0xE90
    for gobj in [slot+0xB0 (main), slot+0xB4 (sub: Nana / other Zelda-Sheik form)]:
        if gobj == null: continue
        fp = gobj->user_data
        cs ^= u32(fp+0x10)        motion (action state) id
        cs ^= bits(fp+0xB0)       cur_pos.x as raw f32 bits
        cs ^= bits(fp+0xB4)       cur_pos.y bits
        cs ^= bits(fp+0x1830)     percent bits
        cs ^= u32(fp+0x8)         spawn number (x8_spawnNum)
        fsum = f32(fsum + pos.x); fsum = f32(fsum + pos.y); fsum = f32(fsum + percent)   (single adds, in order)
    cs ^= u8(slot+0x8E)           stocks, even for an empty slot
hi  = (cs >> 16) ^ (cs & 0xFFFF)
lo  = low 16 bits of fctiwz(fsum)  (truncate toward zero to s32; NaN / out of range saturate as PowerPC does)
checksum = (hi << 16) | lo
```

### 2.3 Comparison (done by the ASM, not Dolphin)
For each remote player's latest (frame, checksum) pair, if `frame <= stable finalized frame` and `frame > 84`
(`UNFREEZE_INPUTS_FRAME`), find the local entry for that frame:
- **hard desync**: `|s16(lo_local) - s16(lo_remote)| > 1` -> "DESYNC DETECTED", the game ends (ranked then restarts from the
  synced state via SLIPPI_SYNCED_STATE).
- **soft**: `hi` halves differ -> a "Desync Risk" HUD label only.
The desync check is skipped on frame 0 and once a desync was shown.

## 3. Gameplay-affecting online patches

Groups (netplay.json): *General* = `Required: General Codes`, *Rec* = `Required: Slippi Recording`, *Online* =
`Required: Slippi Online` (all of Online/Core recursively, Common/FastForward, NanaDeterminism, PSCameraIndependentMonitor,
PreventWobbling non-recursive). "online-only" = the code checks scene `SCENE_ONLINE_IN_GAME` (0x0208).

| patch (group) | address(es) | when | behaviour |
|---|---|---|---|
| UCF 0.84 (General) | dashback 0x800C9A44; shield drop 0x800998A4, extended 0x8009A0B8; pad buffer + 1.0 cardinals 0x8006B460; SDI 0x8008E54C; shield SDI 0x80093294; tumble 0x800908F4; DBOOC squat-reverse 0x800D65EC | all scenes, no toggles | standard UCF 0.84 (the asm files are raw words only; follow the published UCF 0.84 behaviour, which `ucf` implements) |
| NeutralSpawn (General) | 0x8016E510 | every multi-player scene except training / target test | singles: spawn index = number of present players in lower slots, from a per-stage point table (FD (-60,10)(60,10)(-20,10)(20,10); BF (-38.8,35.2)(38.8,35.2)(0,8)(0,62.4); YS (-42,26.6)(42,28)(0,46.9)(0,4.9); DL (-46.6,37.2)(47.4,37.3)(0,7)(0,58.5); FoD (-41.25,21)(41.25,27)(0,5.25)(0,48); PS (-40,32)(40,32)(70,7)(-70,7)); teams 2v2 own table; facing left when x > 0; other stages vanilla spawn id |
| FreezeGlitchFix (General) | 0x801239A8 (one nop) | always | IC freeze glitch removed |
| Init Stage Data / Reset Castle Wind / Reset Kongo Log Bridge (Rec) | 0x801C154C, 0x801CD37C, 0x801D69B0 | always | zero 516 bytes of stage data on alloc; reset castle wind step to 0x3C8EFA35 and mode 1 (0x804D45E0/E4); zero the 15 Kongo log velocities |
| Init Player Data (Rec) | 0x80068EEC | always | zero the fighter data block after allocation |
| NanaDeterminism (Online) | 0x800AC5B8 | always | Nana's throw-DI stick is (0,0) instead of stale registers |
| PreventWobbling (Online) | 0x800DB880, 0x800DBBD4 (init), 0x8008F090 (check) | all scenes (unconditional variant) | counter at fp+0x2384, last move id fp+0x2386; on the 4th distinct Nana/owned-item hit of a held victim in singles, the IC leader's grab breaks (`no-wobble`) |
| PSCameraIndependentMonitor (Online) | 0x801D24FC | always | the Stadium monitor "player damaged" test becomes x in [-120,120] and y in [-20,80] instead of camera bounds |
| FastForward DynamicsFix (Online) | 0x8009E090 | always | after dynamics, rebuild dirty JObj matrices even without rendering (a port that always computes them is unaffected) |
| FreezeDeadUpFallPhysics (Online) | 0x800D4C1C, 0x800D4D68, 0x80080E80 | always | star-KO fall integrated in fighter-data fields (+0x2348 vy, +0x234C vz, +0x2350 pos) with gravity/terminal velocity, model position = camera-transformed, physics independent of the camera |
| BrawlOffscreenDamage (Online) | 0x8006A880 | not home-run | offscreen = outside the stage camera limits (not the live camera); dead or state 4/6 never |
| DesyncProofBGTransformations (Online) | 0x8021AAE4 | always | FD background think cannot consume RNG (seed saved/restored) |
| Stadium hacks (Online) | IngameCheckIfFrozen 0x801D457C, StadiumFileLoad 0x800165AC, GrPsxIsValid 0x801D4760, CustomZeroBuffer 0x801C65C8 | Stadium | frozen/unfrozen from MSRB `alt_stage_mode`; transformation files preloaded (deterministic load), header zeroed for rollback |
| WhispyBlowDirFix (Online) | 0x8008653C | always | fighters in a dead state (<= 0xB) are ignored by Whispy's direction choice |
| InitOnlinePlay (Online) | 0x8016E748 | online-only | ODB setup; match block (0x138 bytes) copied from MSRB; Stadium frozen flag; seed = rng_offset; SyncRNG proc; clears the first word of each port's pad entry at 0x804C20BC + 0x44*i (no held A at start); ranked tiebreak: 180 s timer and 1 stock; teams costumes; delay clamped 1..15 |
| InitPause / PauseCounter / PreventPauseStranding / Teams PreventDeadStranding (Online) | 0x8016E904, 0x8016D26C, 0x8016CD08, 0x8016BE28 | online-only | the sim never pauses online (a client-side pause UI); L+R+A+Start ends the game as LRAS: match struct 0x8046B6A0 +1 = port, +8 = 7, +0x24D5 = 0x1E |
| LGLExceededGameEnd (Online) | 0x802F70C4 | online, not teams/party | ledge-grab limit 45 message at time-out (winner decided elsewhere) |
| Remember Who LRA Started, ReportNoContestLRAS, CustomizeMessageLRAS | 0x8016EA30, 0x802F7094, 0x8016E9E8 | | results-screen data and sounds only |
| Sound/Music, PreventFileAlarms, HandleRumble, crowd chants, pad alarm, no-disk/video asserts | various | online | no seed or HSD_Randi use: not simulation |
| rollback engine: TriggerSendInput, StartEngineLoop, LoopEngineForRollback 0x801A4DB4, ForceEngineOnRollback 0x801A5014, SkipNewInputFetchOnRollback 0x80376A20, ForceInputRefetchOnAdvance 0x80019614 | | online-only | savestate/resim driver; a native port replaces it, keeping section 1.3 and section 2 |

Not in the netplay set: Preload Stadium Transformations (the only extra `HSD_Randi` caller), PortPriority.

## 4. Rust implementation (this branch)
`melee_core::slippi_online` (pure functions + per-game state, off by default) and the engine hooks behind it; see the
module docs and `docs/rust-port/notes/slippi-online.md` section 9 for what was verified.
