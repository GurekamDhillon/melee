# Golden corpus and rounds (history)

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 69-85 -->
## Golden corpus (phase 1)
- `MELEE_NEUTRAL_HASHLOG=<csv>`: byte-order-neutral gameplay hash (`RB_NeutralHash`, fighter.c),
  RB_GameHash's field set by logical value, minus PC-only add-ons. Spec + GC offsets:
  `docs/rust-port/neutral-hash.md`. The Rust port must reproduce this exactly.
- Native code must not pass a pointer into game code to receive results (game stores are
  big-endian); results come back one u32 at a time (`RB_NeutralHashWord`).
- Record: `tools/rust-port/golden/record.sh` (runs each scenario twice, requires identical CSVs);
  goldens in `docs/rust-port/golden/*.csv`; compare with `tools/rust-port/compare_golden.py`.
- Build root: `_build/agents/rustgold` (agent_new.sh's root half, run against this worktree).
- Traps found recording it (2026-10-21): (1) MSVC `std::chrono::current_zone()` throws for the
  Windows zone "British Columbia Standard Time" (ICU lacks it) and every exe died at boot frame 2,
  the shared baseline too; `extern/aurora/lib/dolphin/os/OSTime.cpp` now falls back to the CRT.
  (2) `gd.input(port, spec, 1)` from `on_frame_pre` is never read in turbo (the hold is consumed
  before the next PADRead); use frames=2. `pc/tests/gdlp_whole/pad.lua` has the same bug (its
  fighters stand still). (3) A fresh lane's Aurora dep copy can fail on read-only `.git` packs
  (`dawn-src.pending`): `attrib -R /S`, delete, robocopy the dep in by hand.


<!-- RUST_PORT.md lines 432-512 -->
## Sim round 2: pad_falcon_fox_fd to frame 103 (2026-10-21)
- `melee golden` now matches the golden on **every column (seed and hash included) for frames 0-102**
  (103 of 1801; round 1 ended at 64). It stops at frame 103, the first frame that needs
  character code: Falcon's B + side (Raptor Boost, msid 357, with TransN root motion).
- What closed round 1's two gaps: the fighter skeleton animated from its figatree
  (`melee/src/skeleton.rs`) gives the six ECB joints, and `melee_world/src/mp/coll.rs` ports the
  mpcoll.c air and grounded steps over the group 2 mplib queries. Frame 64's 1-ulp miss was the
  4-substep split of the Entry->Fall ECB jump; landing happens at frame 77 as in the C.
- Then, in order of the frames they unlocked: Landing (77), the command-script interpreter and the
  landing dust (the seed at 77-80), Fox's hand pose (79, ftAnim_ApplyPartAnim), pad input and the
  GO unlock (84: the IfAll Ready HUD's animation ends at frame 85, its end callback
  `fn_8016B7F8 -> ftLib_800868A4` runs in golden frame 84's s_link 0), Turn (84), animation
  blending + Wait + Walk (95-102).
- Ground truth: the C build itself, instrumented with temporary logs (per-substep collision, ECB
  joints and every joint's local SRT, motion changes, the pad status and fighter input, IASA
  entry) written by `MELEE_ECB_LOG=<file>`, plus `MELEE_RAND_TRACE` + `MELEE_RAND_TRACE_STACK`
  resolved against melee-pc.map. The instrumentation is reverted in the tree; it is kept as
  `tools/rust-port/golden/c_trace.patch` (`git apply` it on a scratch build root to use it again;
  it also contains round 1's uncommitted RAND_TRACE_STACK hack in gw_replay.c).
- Facts found: the pad script's sample for golden frame g is the one set at g-1 (n = g - 1);
  the PC pad shim passes raw sticks through (100 -> 80/80 = 1.0); fighter input is zeroed by
  `x221D_b4` until GO, which also keeps x67F/x680 at 255; footsteps on FD's floor material draw no
  RNG (no footstep sound in the C trace); `x594` must be stored in GameCube byte order for the
  generated bitfield accessors (x596_bits.x7 = Turn's 2 drives a part copy at the Wait blend).
- Next: Raptor Boost (ftcaptainspecials.c, the TransN path ftAnim_8006E054 / x6A4), Fox's jab
  (hitbox commands ftAction_8007121C, ftColl hit detection), then the run off the stage (frame
  142+), death and rebirth. The other three goldens need their stages, fighters, CPU and items.

## Sim round 1: pad_falcon_fox_fd through the entry sequence (2026-10-21)
- `melee golden` matches the golden on **every column, seed and hash included, for frames 0-63**;
  frames 64-66 differ in s0/hash only (Falcon's y by 1 ulp), 67-74 are identical again, and the run
  stops before frame 75. Checked with `compare_golden.py ... --allow-short` against the committed CSV.
- Ground truth came from the C build itself: a temporary `OSReport` of every hashed fighter field in
  `nh_Fighter` (under MELEE_LOG_MOTION, reverted, never committed) and `MELEE_RAND_TRACE` (+ the
  uncommitted `MELEE_RAND_TRACE_STACK` hack in gw_replay.c) resolved against melee-pc.map. Every draw
  in golden frames 5-63 comes from the particle procs (`hsd_8039EE24`/`8039DAD4`/`8039930C`/`8039F05C`).
- What was needed past frame 5: EntryStart (timer x6BC=30, y = x4 + x20*t, x20 = (f32)(1.497345 *
  trophy_scale)); EntryStart's figatree length from `ftData->xC[238]` + PlXxAJ.dat (10 for both);
  EntryEnd (x6C0, no animation: frame_count keeps 10 on the change frame, then -1); the entry beam
  effect 0x24 (EfCoData desc: lifetime 60, two tracks spawning bank-0 particles 445/449 at frame 0 and
  448 at frame 10) created at s_link 9 and animated at s_link 15 ahead of the particle procs; Fall with
  the ported ftcommon fall/drift and a looping AObj (Fall row: 8 frames, x594 bit 1 = loop).
- Frame 64's 1-ulp miss: `mpColl_80043754` recomputes `cur_pos = last_pos + vel/steps` (steps times)
  with steps = |largest ECB or position delta| / 6 + 1. On the Entry->Fall frame the ECB jumps from the
  entry box to the bone-driven one, so the C takes 2 (or 4) substeps; the native run takes the
  position as is. Matching it needs the ECB, i.e. the fighter skeleton animated from its figatree.
- Next: ECB (ftColl ECB from bones: JObj tree + figatree DObj/AObj animation), mpColl air/ground
  collision over the ported mplib lines (landing at golden frame 77), Landing/Wait, then input.

## Native golden run: pad_falcon_fox_fd (2026-10-21)
- `melee golden --iso <disc> --out C.csv` (`rust/melee/src/vs_match.rs`, `stage.rs`, `disc.rs`) is
  the first real `Sim`: it reads `GrNLa.dat` and `PlCo.dat` from the user's disc (local only, not
  written anywhere), starts the VS match and steps frames until the next frame needs unported code.
  `compare_golden.py docs/rust-port/golden/pad_falcon_fox_fd.csv C.csv --allow-short`.
- Ground truth for frame 0 came from the C build itself (a scratch Lua pad script that logged
  `gd.player()` each frame, hashes unchanged): both fighters are in Entry (322), airborne, at
  (-60, 10, 0) and (60, 10, 0), facing +1/-1, shield 60, cmd frame_count -1, jumps used 1,
  x67F = x680 = 255, stocks 3. A brute force over the NHV1 word confirmed exactly that tuple.
- **Result: 5 frames (0-4) written. First diverging frame: 0. First diverging field: `seed`
  (golden 9D38619F, native 00003039 = MELEE_TEST_SEED 12345 undrawn), hence `hash`.** `s0` and `s2`
  (every fighter field) are equal on all 5 frames; with `--seed-from <golden>` (diagnostic: copies
  the golden seed column) `compare_golden.py` reports the 5 frames identical, so the frame combine
  and the fighter words are right and the seed is the only difference there.
- The seed: the C build draws 54 times (12345 -> 9D38619F) before frame 0, then 19, 25, 19, 7, 22,
  42, 56, ... per frame (MELEE_RAND_TRACE + melee-pc.map). Every draw in logic frames 29-34 (golden
  0-5) comes from the HSD particle generator for the entry effects: `hsd_8039930C` (particle.c),
  `hsd_8039DAD4`, `hsd_8039EE24`, `hsd_8039F05C` (generator.c). Matching `seed` needs
  generator.c/particle.c ported (group 1) plus the ef/ entry-effect spawns that drive them. (So
  particle state, excluded from the hash, still reaches it through the RNG.)
- Where the run stops: frame 5, Falcon's Entry timer (Player_GetUnk4C = 5; Fox's is 10) reaches 0
  and `ftCo_Entry_Anim` calls `ftCo_800C6408` (EntryStart: Fighter_ChangeMotionState, the entry
  animation; the C then moves y to 0x1.438386p+3 by frame 6). Not ported: Fighter_ChangeMotionState,
  the fighter animation/figatree path (group 1 DObj/AObj walkers are stubs) and the per-frame
  fighter update (fighter.c Fighter_UnkProcess*); Fox diverges the same way at frame 10.
- Approximations in what runs: the spawn point's world translation is composed from the joint
  descriptors (exact only with no rotation/scale above it; FD's are, `SpawnPoint::exact`), not
  through the JObj system; the Entry timer lives beside the Fighter, not in `fp->mv`; fighter
  defaults are only the fields the hash reads, not all of Fighter_UnkInitReset; stocks 3 is the
  scene's, not read from the VS rules struct. `MELEE_GOLDEN_DUMP=1` prints the start state.
- Other scenarios are not wired (only falcon/fox and FD).


<!-- RUST_PORT.md lines 962-1238 -->
## Golden round 1: pad_falcon_fox_fd matches in full (2026-10-24)
- `melee golden --scenario pad_falcon_fox_fd` is identical to `docs/rust-port/golden/pad_falcon_fox_fd.csv` on all
  1801 frames (0..1800; `compare_golden.py ... --allow-short` exit 0). The sim then stops on the first unported
  state the script reaches past the golden (`ftCo_RunBrake_Enter`, frame 1802). Start of the round: 216 frames.
- Ported in `rust/melee/src/vs_match.rs` and `vs_match/{rebirth,fox,attackair,dashrun}.rs`: blast-zone KO and DeadDown;
  respawn (Rebirth / RebirthWait, the platform path of `Fighter_UnkProcessDeath`, `fn_8016758C` spot countdown, the
  spawn probe); shield (GuardOn / Guard / GuardOff, power shield, upkeep); jumps (KneeBend, Jump, JumpAerial,
  FallAerial); Fox neutral-B in the air (gun and laser items, muzzle flash); aerial attacks (stick / C-stick
  selection, Anim / IASA / Phys / Coll, landing through `landing_enter_basic`); Dash and Run; the player nudge
  (`ftCommon_8007E0E4`, radii `*ftData->x50` into `fp->x2C4`).
- Traps found (each cost a bisect):
  - 1-ulp position differences are almost always a collision substep in the C (ECB Clear / locked ECB, or an
    item spawn moving > 6 units: `it_80276100` -> `mpColl_800471F8` adds `vel / steps` per substep). Compare the
    value rounded to the coarser exponent grid before suspecting velocity.
  - `ftCo_800CC988` runs after `ftCo_Fall_Anim_Inner` every frame the fall blend is nonzero; poses feed the ECB
    (and the Rebirth spawn probe's locked ECB bottom), so a wrong pose shows up only at a landing or a respawn.
  - `Fighter_ChangeMotionState` clears `x221D_b5` (the "no nudge" flag Rebirth sets).
  - Debugging the C: temporary `OSReport` in a game TU, `tools/port/build.sh` (about 1 min incremental), `crun.sh`
    with `dump.lua` (Lua `n` = golden frame + 1 but `gd.player` x reads mid-frame; trust the hash columns).
    OSReport lines of per-frame call sites can be dropped by the log rate limit: print only a rare condition
    (an item kind) instead. Revert the TU afterwards (`git checkout src`).
- Not ported (the next stops): RunBrake / TurnRun / smash turn, Z grab and every hit (`hit_guard` stops the run),
  Fox ground neutral-B, Falcon's other specials, platforms and ledges, shield break, `x2135 != -1` respawn stages.
- Next: the other three goldens are not wired (`VsMatch::new` supports `GrNLa.dat` only; `golden()` has no Marth,
  Sheik, Battlefield `grbattle.c`, or CPU input). Order of attack: Battlefield stage init and its materials ->
  Marth / Sheik file and effect-bank loaders -> the states `pad_fixed.lua` reaches first.

## Golden round 2: pad_marth_sheik_bf wired, frames 0..268 match (2026-10-24)
- `melee golden --scenario pad_marth_sheik_bf` matches the golden on every column for frames 0..268
  (`compare_golden.py ... --allow-short` exit 0); pad_falcon_fox_fd still matches 0..1800. The sim stops on
  "player 2: neutral B in the air (kind 7) not ported" (Sheik's Needle Storm; needs the It_Kind_Seak_Needle*
  items). cpu_fox_falco_dl and cpu_peach_samus_ys_items are NOT wired (CPU input, Falco, Peach, Samus,
  Dream Land, Yoshi's Story, items).
- Wired in `vs_match.rs`: `GrNBa.dat` (StKind 0x1F), Marth (`PlMs*`, EfMsData bank 16) and Sheik (`PlSk*`,
  EfZdData bank 17) loaders, and the Sheik/Zelda partner fighter (`Player_80031AD0` second `Fighter_Create`,
  `ftCo_800BFD04` Sleep; hash slot `slot*2+1`; `Player::follower`, skipped by every per-frame loop and by hits).
- Traps found (each cost a bisect; C trace = `MELEE_RAND_TRACE` + the stack hack of `c_trace.patch`):
  - Stage points are scaled by `grGroundParam->y` (Battlefield 0.8): the spawn (0,10) became (0,8). FD is 1.0.
  - The spawn facing is `fn_8016DEEC` (face the nearest other fighter; within 5 units, away from its facing,
    +1 when unset), not `direction(x)`; at equal x the first player faces +1, the second -1.
  - `grBattle_OnInit` draws one `HSD_Randi(1200)` (BG timer) after the effect banks, before the fighters.
  - Battlefield's stage creates 4 generators at init (bank 30 ids 0x7532..0x7535, the particle tracks of map
    gobjs 0 and 1 anim 0). They draw no RNG, but they are in the generator list: the insertion cursor
    (`hsd_804D78F8`) puts later gens after them. Without them the first fighter effect's generators were
    visited in a different order and its RNG draws moved (a 9 vs 17 draws row, found with the C stack trace).
  - `ftData->x1C` (hand-pose table) has only as many entries as the fighter has slots; later words are other data.
  - Command-script ops 0x2E (colour overlay) and 0x31 (afterimage) are render-only: skip one word.
- Ported with it: Marth's Counter (ground), the forward/up tilts (`ftCo_AttackS3`, `AttackHi3`; `AttackLw3` and
  the smashes stop the run), Marth's GuardOn (model part flag), efAsync 0x3FE (generator 0x107).
- Next: Sheik's neutral B (SpecialN/SpecialAirN, needle items), then `AttackLw3`, smashes, the air Counter,
  every hit (`hit_guard`), then the two CPU goldens.

## Golden round 3: pad_marth_sheik_bf matches in full (2026-10-24)
- `melee golden --scenario pad_marth_sheik_bf --frames 1801` is identical to the golden on all 1801 frames;
  pad_falcon_fox_fd still matches 0..1800 (stops at 1802 on `ftCo_RunBrake_Enter`). The two CPU goldens
  (cpu_fox_falco_dl, cpu_peach_samus_ys_items) are still not wired (CPU input, Falco, Peach, Samus, Dream Land,
  Yoshi's Story, items). Note `golden` without `--frames` now runs on forever once nothing is unported.
- Ported: Sheik Needle Storm (`vs_match/sheik.rs`: held bundle, thrown needle incl. ground stick / bounce, destroy
  effect gen 0x3F with its 3 randf; article data from `ft_data->x48_items[0]`), LandingAir with the L-cancel,
  Ottotto (teeter), floor-end probes `mpColl_8004A45C/A678_Floor`, air wall push-out `mpColl_80046224/800454A4`
  (fma_sites.py: 5 fused sites each), RebirthWait landing, Walk enter from Wait/Landing, per-slot respawn on
  Battlefield (`stage_info.unk8C.b4`), dust gen 0x407, shared grounded IASA chain for Landing.
- Traps found (each cost a bisect):
  - `mpColl_804D64AC` is bumped every frame by each map gobj's s_link-1 proc: the first substep of every
    fighter collision call takes the `Remap` variants, which do NOT extend floor ends by 1 unit
    (`mpLib_8004ED5C`). Without the per-frame increment a fall past a floor corner landed on the corner.
  - Stage particle animations (grAnime_801C8138) end with `HSD_JObjAnimAll`: the track is one frame ahead
    from frame 0, and a nonzero entry of `map_head.unk8[gobj].x28` sets AOBJ_LOOP (BF gobj 1 loops at 400:
    the 4 stage gens respawn at golden frame 399, not 400).
  - `Fall_Enter` from the ground must call `ftCommon_8007D5D4` (ground to air) itself.
  - Z press in the air = A pressed + the synthetic LR bit: `checkItemThrowInput` (a new aerial) comes BEFORE
    `ftCo_800D705C` in DO_IASA. The analog trigger alone is read by nothing in the air IASAs.
  - Debugging: a temporary `OSReport` in a game TU did not print under the rand-trace patch; the
    `MELEE_RAND_TRACE_STACK` caller addresses resolved with the map file were enough. Never `git checkout`
    a file with uncommitted work (lost and redone once), and `git checkout pc` after reverting c_trace.patch.
- Next: the unported states past the goldens (RunBrake / smash turn, hits via `hit_guard`, item hit detection,
  Sheik's other specials), then the CPU goldens.

## Golden round 4: cpu_fox_falco_dl wired, frames 0..63 match (2026-10-24)
- pad_falcon_fox_fd (0..1800) and pad_marth_sheik_bf (0..1800) still match in full. `melee golden --scenario
  cpu_fox_falco_dl` (Fox/Falco CPU9 on Green Greens, `GrGr.dat`, ext StKind 0x11 = "dreamland") is newly wired and
  matches golden frames 0..63; it then diverges at frame 64 (Falco, s2) and stops at frame 91 on
  "ftCo_800B2790: behaviour 3 not ported". cpu_peach_samus_ys_items is not wired.
- Wired: Green Greens init (`greens_init`: 18 BOMB draws, block timer, wind timer, the 18 block items at the grid
  positions from the 30 block joints times `grGroundParam->y` (0.75), item kind 160 msid 0), Falco (Fox's effect
  bank, `PlFc*`), CPU slots (`fox/cpu9`: slot type Cpu, cpu kind 4, level 9, `fp.cpu` hash words), and the CPU
  controller `vs_match/cpu.rs` (ftCo_800B3900 at s_link 2: sensing 800B33B0, 800B2AFC/24B8, 800B2790 behaviours
  1, the island/target helpers, 800ADE48, the ftcpuattack.c attack choosers 4AB0/8A9C/9CBC/732C/77E8). The pad
  input of a CPU is `fp.cpu` (`input::fighter_input(.., cpu)`). Every unported branch returns `Err("... not ported")`.
- Traps found:
  - The CPU draws RNG from the first match frame: golden frame 0 = C logic frame 36, six draws (B33B0 x3 per
    fighter pair + 0CB0 + 4BEC); `ftCo_CpuInitNoTarget(fp, true)` in B24B8 (the `false` one is B1EF0's).
  - `stage_info` for Green Greens: ground param y 0.75 scales the map joint positions (block items).
  - Hurt capsule `bone_idx` indexes `fp->parts[]` (the DFS part number) directly, NOT through `part_to_joint`.
  - `ftCo_800A0DA4` (hurtbox extents x55C..x568) runs every frame at the end of `Fighter_ProcessHit_8006D1EC`.
- Debugging that worked: temporary `OSReport` in `nh_Fighter` (all CPU hash words per call), rebuild with
  tools/port/build.sh, `MELEE_GOLDEN_CPU=1` / `MELEE_GOLDEN_HRT=1` in the Rust side print the same lines
  (removed again from the C; `git checkout src` afterwards).
- First divergence (frame 64, Falco): `cpu.x38` = `x56C + target.x564`, and `target.x564` comes from the hurt
  capsules, of which the Fox TAIL (hurt capsule 12, part 18) is a dynamics bone: parts 18..20 differ from frame 1
  on. `ftCo_8009DD94` -> `lb_8001044C` (lb_00F9.c cloth/tail physics, ported on raw HSD_JObj in
  `melee_runtime::tu::lb_00F9`, not on the sim's `Skeleton` tree) must run for the sim's skeleton. Also open at
  frame 64: `cpu.xEC` is 2 here, 1 in the C (one avoid-list push too many in B8A9C/B77E8, likely the same cause).
- Next: dynamics bones for the sim skeleton, then behaviour 3 (800B9020), the rest of the CPU decision code
  (A9904, A96B8, ABBA8, ...), Fox/Falco ground neutral-B and shine, hits, Green Greens' per-frame stage procs
  (blocks, wind, apples), then cpu_peach_samus_ys_items.

## Status after the stub-fill merges and golden rounds 1-4 (2026-10-24)

All six `port/fill-melee_*` branches and `port/shard-665-672` are merged into `rust-port` (nothing pushed). `cargo build --workspace` passes (in `rust/`).

Fixed this session: nine failing oracle targets (shadow native headers, `fv64` draw order, null stale pointers, `LOWMEM` pools, `PTR_MASK`, `tu_sobjlib` `origin_x` /2.0); stub audit found no recursion; 7 duplicate ports now forward to the TU ports; `missing.md` written.

Stub fill (counts are distinct names / sites from `docs/rust-port/missing.md`; the doc was not regenerated, per-crate figures come from each fill report):

| crate | before (sites: NO_PORT / SIGNATURE / CYCLE) | now |
|---|---|---|
| melee_runtime | 534 / 466 / 177 | SIGNATURE ~216 left, NO_PORT 534, CYCLE 177 (250 SIGNATURE sites forwarded; unverified vs C) |
| melee_world | 171 / 460 / 182 names | 273 of 813 names left (87 / 69 / 117), plus 41 names the doc missed; ~115 new ports verified vs C |
| melee_fighter | 276 / 302 / 188 names | ~277 of 766 names left (~71 / ~55 / ~151) |
| melee_chara | 164 / 149 / 6 sites | 0 |
| melee_items, melee_modes | 311 / 4321 / 471 and 912 / 2428 / 0 sites | many wired (adapters, native db ports, PS/MTX bridge); no recount recorded, regenerate `missing.md` to get numbers |

Not in `missing.md`: ports that still panic from `.expect("TODO ...")` hooks (e.g. `tev.rs` `OBJ_ALLOC_INIT`). `tu/globals.rs` exists in both melee_runtime and melee_world (identical); runtime and world may duplicate raw GObj/JObj/CObj work. There is one `controller` static (melee_world re-exports melee_runtime's).

Golden frames matching (`melee golden`, `compare_golden.py --allow-short`):

| scenario | matching frames | stop |
|---|---|---|
| pad_falcon_fox_fd | 0..1800 (all 1801) | frame 1802, `ftCo_RunBrake_Enter` not ported |
| pad_marth_sheik_bf | 0..1800 (all 1801) | none in range |
| cpu_fox_falco_dl | 0..63 | diverges at 64 (Falco); sim stops at 91, `ftCo_800B2790` behaviour 3 |
| cpu_peach_samus_ys_items | not wired | needs CPU input, Peach, Samus, Yoshi's Story, items |

Blockers: (1) Fox tail dynamics bones (`ftCo_8009DD94` -> `lb_8001044C`) not run on the sim `Skeleton`, which breaks hurtbox extents and `cpu.x38`/`xEC`; (2) remaining stubs need a global GX/GObj/JObj host layer (SIGNATURE), SDK/hardware and platform-glue ports (NO_PORT), and moving DevText/Camera/Mushroom/ftDrawCommon down a crate (CYCLE).

Next plan: regenerate `missing.md`; dynamics bones for the sim skeleton; CPU behaviour 3 (800B9020) and the rest of the CPU decision code; RunBrake / smash turn / hits (`hit_guard`) for the fd golden's tail; Green Greens stage procs; wire cpu_peach_samus_ys_items; host layer for the SIGNATURE stubs.

## Golden round 5: cpu_fox_falco_dl frames 0..91 match (2026-10-25)
- pad_falcon_fox_fd and pad_marth_sheik_bf still match in full (0..1800). `cpu_fox_falco_dl` now matches golden frames
  0..91 (was 0..63); the sim stops at frame 92: "ftCo_800BB220 (an item with a hitbox) not ported" (Fox sees Falco's
  laser). cpu_peach_samus_ys_items is still not wired (needs Peach, Samus, Yoshi's Story, items; fighter file names
  for peach/samus are unverified).
- Fixed, in order of first divergence:
  - Frame 64 (hurtbox extents): the fighter dynamic bones (Fox tail) now run on the sim `Skeleton` (`melee/src/dynamics.rs`,
    a transcription of `lb_8001044C` over the tree; `Fighter_8006D9AC` at s_link 0x10). The chains' `unk_2C` world
    positions are taken in `ftCo_8009CF84` at the origin in the bind pose (Dynamics::load right after the skeleton
    is built, before the spawn translate) -- loading them at the spawn pose was the bug. Per-frame debug:
    `MELEE_GOLDEN_DN`, `MELEE_GOLDEN_PJ`, `MELEE_NO_DYN`.
  - Frame 64 (CPU `xEC`): Green Greens' block map joints. `grGreens_80215358` moves joint `num` (lowest unused) to the
    cell via `mpLib_80055E9C`; the unused 12 joints are hidden by `Ground_UpdateMapColl`. `greens_sync_joints` +
    `MapTree` (skeleton.rs) + `GroundColl::mp_lib_80055e9c` (uniform-scale path only).
  - Frames 84-97: the stage procs start at GO (`Ground_801C0FB8` runs the deferred `fn_80214658`): `vs_match/greens.rs`
    ports the Whispy wind state machine (`grGreens_80213C10`: states 0 and the start of 1; 2 = apples, wind force
    `lb_80011A50` and the fighters' wind reaction are Err) and the block gobj (`grGreens_8021479C`: timer, stitching of
    the resting blocks with `mpLib_800581DC`; spawning/breaking blocks is Err). Map animations: `grAnime_801C8138` is
    `greens_set_anim` (tracks of `dptcl_tracks` + the timing AObj `stage::anim_timing`); `AOBJ_REWINDED` is
    `grAnime_801C84A4`. These run at s_link 4 ahead of the fighters (p_link 5 < 8); the timing AObj steps at s_link 1.
  - Frame 98: `Fighter_8006A360` sets `pos_delta = cur_pos - prev_pos` (CPU attack choosers read it).
  - CPU behaviours 2 (`ftCo_800B658C`) and 3 (`ftCo_800B9020`); `ftFx_SpecialN_Enter` on the ground
    (`ftCommon_8007D7FC` through `Player.gc` / `common_c`, set each frame) and Falco on the ftFx code paths (`is_fx_kind`).
- Debugging recipe that worked, WITHOUT any OS input or focus: temporary `OSReport` in the C (rebuild one TU with
  `GW_BUILD_ROOT=_build/agents/golden2 tools/port/build.sh --tu <file>`; 20 s after the first full build, which took
  20 min), run the C with `SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0 MELEE_WINDOW_X=30000 MELEE_UNATTENDED=1 MELEE_TURBO=1
  MELEE_PAD_SCRIPT=tools/rust-port/golden/pad_idle.lua` (see record.sh for the scene strings), `MELEE_RAND_TRACE=<file>`
  gives every draw with the caller address (resolve against `_build/.../melee-pc.map`: the CSV is
  `logic frame,caller,seed after,stream`). C logic frame = golden frame + 29..36 of pre-golden frames; the
  hash column `H` of a printed per-fighter line is the key that aligns C and Rust lines.
  The C log is rate-limited (OSReport): print rare conditions only. Revert the C afterwards (`git apply -R`).
- Next: the laser as a threat (`ftCo_800BB220`: item hitbox state `x5D4_hitboxes`, `lbColl_8000ACFC`, `lbColl_80006094`),
  then hits (`hit_guard`), Whispy's wind (state 1 onward, `lb_80011A50`, fighters' wind) and apples, block spawning,
  then wire Peach / Samus / Yoshi's Story / items.

## Golden round 6: cpu_fox_falco_dl 0..117, cpu_peach_samus_ys_items wired, 0..94 (2026-10-25)
- pad_falcon_fox_fd and pad_marth_sheik_bf still match in full (0..1800, checked at the end of the round).
  `cpu_fox_falco_dl` matches 0..117 (was 0..91); it stops at frame 118 ("a hit with knockback ... damage states not
  ported": Falco's laser on Fox, kb 23.6). `cpu_peach_samus_ys_items` is wired (`melee golden --scenario ...`, Peach and
  Samus vs CPU 9 on Yoshi's Story `GrSt.dat`, items=4) and matches 0..94; it stops at frame 95 ("ftCo_SpecialS_CheckInput
  (side-B, kind 13)": Samus' missile).
- Done, in order of first divergence:
  - cpu_fox_falco_dl 92: the laser as a CPU threat (`ftCo_800BB220`, `vs_match/cpu.rs`): the lasers carry their hit capsules
    (`fox::LaserHits`, steps 1 -> 2 -> 3 as `it_8027137C`). Fox's article script steps the boxes once at creation (state 2,
    joint at the creation position with Z scale 1); Falco's leaves them Enabled (state 1, not live) -- read off the C
    (`RPL`/`RPE`/`RPU` prints), not parsed from PlFx/PlFc. Shots made at s_link 1 are visible to the CPUs of the same frame
    (`fox::pending_lasers`). CPU behaviour 7 (`ftCo_800BA9A0`), `ftCo_800BB768`.
  - cpu_fox_falco_dl 111: items hitting fighters (`vs_match/hit.rs`, s_link 13): `ftColl_8007925C` for a laser against the
    hurt capsules (`lbColl_8000805C`, capsules refreshed from the skeleton), `ftColl_80077C60` (damage log, stale moves via
    `melee_fighter::stale`, combo counter), `ftColl_8007A06C` (knockback by `ftcoll_80079ab0`, the hit spark: `Randi(8)`
    picks effect 9/10, then `Randi(x3F4)`), the percent commit of `Fighter_ProcessHit` at s_link 14 for a hit without
    knockback, the shot's destruction. The lasers' item procs now run ahead of s_link 13 (`lasers_step`). The HUD percent
    shake (`ifStatus_802F5B48/4EDC/4B84`): 8 `HSD_Randf` per frame for 9 frames after a percent increase (`hit::HudDamage`).
    `efLib_SpawnParticleEffect` ids 0x2/0x6/0xA3.. (generator scale = effect root scale).
  - cpu_peach_samus_ys_items: Peach / Samus files and effect banks (PlPe/PlSs, EfPeData index 15, EfSsData index 2); Yoshi's
    Story (`vs_match/story.rs`: Randall's puff `grStory_801E366C`, the Shy Guys' timer; the shy-guy spawn at frame 120 is an
    Err); the random item spawner's setup draw (`it_8026D018`, `item_spawner_init`: ItCo.usd range * stage param, the
    countdown fires at golden frame ~210: Err); dynamics bone stride is 0x18 (`BoneDynamicsDesc`), not 0x1C; the avoid list
    for Peach/Koopa/Yoshi (`ftCo_800B77E8`), `ftCo_800A2718` on Story, soft landing to Wait, the footstep and script sound
    commands (`ft_PlaySFX`: `HSD_Randi(200)` when `ft_80087D0C` leaves the id in 332..370).
- Traps found: the C draws the first CPU frame's `B33B0` pair at logic frame 29 for this scenario (golden frame 0), so
  the init draw count must be right (the item spawner's setup consumes one draw); `golden frame = C logic frame - 29` here
  (the CPU golden of round 5 used the same offset, "36" in that note was a typo of the frame label). The Rust run must be
  capped with `--frames` (a scenario that never stops would write forever).
- Debugging recipe (no OS input, no focus): temporary `OSReport` prints in the C (`tools/port/build.sh --tu ...` with
  `GW_ENV_RESOLVED=1 GW_BUILD_ROOT=_build/agents/golden2`; `.env` otherwise overrides GW_BUILD_ROOT and would relink the
  shared `_build`), run with `SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0 MELEE_WINDOW_X=30000 MELEE_UNATTENDED=1 MELEE_TURBO=1` and
  `--max-seconds 40`; `MELEE_RAND_TRACE` for the draws (caller addresses against `melee-pc.map`; statics are missing from
  the map, e.g. the HUD shake shows as `un_802FE6A8`). Revert the C afterwards.
- Next: the damage states for a hit with knockback (`ftCo_8008DCE0` -> `ftCo_Damage*`, `ftCo_80090594`, hitlag:
  `ftCommon_CalcHitlag`, `Fighter_UnkRecursiveFunc_8006D044`, SDI / `ftCo_Damage_OnEveryHitlag`), Samus' missile
  (`ftSs_SpecialS_*`, `It_Kind_Samus_Missile`), the Shy Guys (`it_802D8618`, `itHeiho`), the random item spawn
  (`it_8026C75C`, `Stage_80224FDC`), fighter-vs-fighter hits (`hit_guard` still stops them).

## Golden round 7: cpu_fox_falco_dl 0..139, cpu_peach_samus_ys_items 0..119 (2026-10-25)
- pad_falcon_fox_fd and pad_marth_sheik_bf still match in full (0..1800, checked at the end). `cpu_fox_falco_dl` matches
  0..139 (stops at 140: Falco's forward smash, `ftCo_AttackS4`, then fighter-vs-fighter hits / `hit_guard`);
  `cpu_peach_samus_ys_items` matches 0..119 (stops at 120: the Shy Guys' spawn, `it_802D8618` / `It_Kind_Heiho`).
- No OS input anywhere: the runs are `melee golden` (headless Rust) and, for the oracle, the C exe started off-screen
  (`SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0`, `MELEE_WINDOW_X=30000`, `MELEE_TURBO`, pad script `pad_idle.lua`, 40 s cap).
- Done, in order of first divergence:
  - cpu_fox_falco_dl 118 (a hit with knockback): `vs_match/damage.rs`. Every `Player` owns a stand-in `HSD_GObj`
    (`user_data` -> its `Fighter`, `fp.gobj` set); `VsMatch::with_sim` publishes the running match in a thread-local so the
    ported `ftCo_Damage.c` (`ftCo_8008EC90` -> `ftCo_8008DCE0`, the Damage/DamageFly state callbacks) can reach the
    sim through its `ext::hook::*` tables (`Fighter_ChangeMotionState` -> `change_motion_state_flags`, `ftAnim_8006EBA4` ->
    `anim_step`, `ft_8008A2BC`, `ftCo_Fall_Enter`, ground coll/phys leaves ...); a hook that cannot proceed records
    `VsMatch::dmg_err`. `process_hit_damage` (hit.rs) is now the whole of `Fighter_ProcessHit_8006D1EC`'s damage part:
    percent commit, knockback branch, `ftCo_80090594`, hitlag (`calc_hitlag`, `allow_sdi`, `x221A_b3`, pre_hitlag_cb),
    the field resets. Tables needed: PdPm.dat `plLoadCommonData` (`pl_804D6470`), PlCo pData[21] (`gCrowdConfig`),
    pData[9] (`Fighter_804D6530`, the hit model shift rows). Hurt capsules now get `height`/`is_grabbable`/`bone_idx`
    (`ftColl_8007B320`) -- the hit capsule's height picks the damage motion (High=2 -> DamageHi1).
  - 118-119: `lbColl_800083C4` caches a hurt capsule's end positions once per frame (`skip_update_pos`, cleared by
    `ftColl_8007AF28` at the end of `Fighter_procUpdate`), and the laser hit test computes them lazily per capsule up to
    the first hit; `ftCo_800A0DA4` (CPU extents) therefore sees the PRE-hit pose of the capsules after the hit -- the
    C's one-frame lag, now reproduced (`hurt_position` in hit.rs, `hurt_extents`).
  - Hitlag in the frame loop: s_link 0 `Fighter_8006A1BC` (`hitlag_countdown`), anim/phys gated on `x2219_b5`,
    `hitlag_cb`/`ftColl_80076528`/`mpGetSpeed` after phys, the knockback decay + `x1948` interpolation in
    `proc_update_integrate`, the invincibility countdowns `x1990/x1994` (`ftColl_8007B7A4` is called from
    `ftCo_Damage_OnExitHitlag`), `x18C8`, `x18ac_time_since_hit` (init -1), the model shift counters.
    `change_motion_state_flags` now does the rest of `Fighter_ChangeMotionState`'s clears (dmg.kb_applied = 0, callbacks
    None, x100, x2100, x209A, x221C_b6 ...). CPU `ftCo_800ADE48` just-hit path: `switch_cmd` is uninitialised in the C on
    that path; assuming 0 matches (set `MELEE_CPU_SWITCH` to try 1).
  - cpu_peach_samus_ys_items 95: Samus' side special (`vs_match/samus.rs`: ground states 349/350, `ftSs_SpecialS_8012A074`
    accessory), `ftCo_800B630C` for DK/Kirby/Peach, then the missile article (`It_Kind_Samus_Missile`, 95): spawn at joint 56
    + x34 (initial collision step `spawn_step`), smoke generator (`hsd_8039EFAC(0, 2, 0x7DE, item joint)`) and muzzle
    generator (`efLib_CreateGenerator(0x7D7)`), flight (anim: lifetime/var0/var1; phys: homing `it_802B64FC`
    with `lbVector_Rotate`; pos += vel), explosion (`it_802B701C`: zero velocity, life 80, generator 0x19), removal;
    `ftCo_800BB220` takes the missile's single hit capsule (state 2 on the creation frame, then 3; scale 0x3FF9FBE7).
- Debugging recipe: the C patch used for the field-level comparisons is a temporary `OSReport` in `nh_Fighter` / the item
  loop of `RB_NeutralHash` (`fighter.c`), printing the hashed words, the CPU words and the item/hit-capsule fields
  (`HB`, `ITM`, `NHC`, `HC` lines); build with `GW_ENV_RESOLVED=1 GW_BUILD_ROOT=_build/agents/golden2 tools/port/build.sh
  --tu src/melee/ft/fighter.c`, run with `MELEE_NEUTRAL_HASHLOG=<csv>` (nh_Fighter only runs with the hash log on),
  revert with `git checkout src/melee/ft/fighter.c`. `golden frame = C log frame - 31` for these two scenarios.
- Next: Falco's forward smash (`ftCo_AttackS4`, hitboxes via the script) and fighter-vs-fighter hit detection
  (`hit_guard`: `ftColl_800779xx`, shields, clanks), air damage states (`ft_80084DB0/EEC`, `ft_80081DD4`, DamageFall,
  `ftCo_80090780`, camera quake), the Shy Guys (`it_802D8618`, `itheiho.c`: item-stage collision `it_8026DA70`, states
  0..4), the random item spawn (`fn_8026C88C`, golden frame ~210), the missile's own hits and its smash variant.

## Status: golden frames matching per scenario, and blockers (2026-10-25, after round 7)
- Frames matching (count from frame 0, first divergence = count):

  | scenario | frames matching | first divergence / blocker |
  |---|---|---|
  | pad_falcon_fox_fd | 1801 of 1801 (full) | none |
  | pad_marth_sheik_bf | 1801 of 1801 (full) | none |
  | cpu_fox_falco_dl | 140 (0..139) | 140: Falco forward smash `ftCo_AttackS4`, then fighter-vs-fighter hits stopped by `hit_guard` |
  | cpu_peach_samus_ys_items | 120 (0..119) | 120: Shy Guys spawn (`it_802D8618`, `It_Kind_Heiho`); later the random item spawn (~frame 210) |

- History: cpu_fox_falco_dl 92, 118, 140, 142, 142, 142, 140; cpu_peach_samus_ys_items 0, 95, 120, 120, 120, 120, 120.
  The 142 -> 140 drift in the last rounds is not explained by a known logic change; re-check next round.
- Blockers, in order: (1) fighter-vs-fighter hit detection (`hit_guard`: `ftColl_800779xx`, shields, clanks) and smash
  hitboxes through the script; (2) air damage states (`ft_80084DB0/EEC`, `ft_80081DD4`, DamageFall, `ftCo_80090780`, camera
  quake); (3) Shy Guys (`itheiho.c`, item-stage collision `it_8026DA70`); (4) random item spawn (`fn_8026C88C`); (5) the
  missile's own hits and smash variant.
- Test-input rule (user request): no OS-level input (SendKeys, SetForegroundWindow, AppActivate, computer-use, etc.) from
  any test or agent run; tests were stealing the user's focus. All input goes through `MELEE_PAD_SCRIPT`,
  `MELEE_PAD_IGNORE_ADAPTER=1`, the Rust `PadSystem` script source, or in-process hooks. Runs are headless (`melee golden`)
  or, for the C oracle, off-screen, unfocused (`SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0`, `MELEE_WINDOW_X=30000`, `MELEE_TURBO`)
  and self-terminating (frame/time caps). Features that cannot be driven this way get an in-engine scripted-input hook.
