# Upstream doldecomp sync (task #13)

Branch `port/upstream-sync` (worktree `worktrees/upstream-sync`), cut from `rust-port` (d373bba89), then
`git merge origin/master` (doldecomp/melee, fetched 2026-11-01, tip 598987cf3). Merge base 8ad0271e0 (2026-09-19);
116 upstream commits. `rust-port` is untouched and this branch is not merged into it.

**Result: merged, C reference builds and links, all four C goldens reproduce byte-identically (1801 / 1801 / 3601 / 3601 frames). No golden changed, so nothing was kept-and-listed.**
Not merged into `rust-port`; Rust was not touched. Open risks are listed under "Not verified".

## What the merge touched
- 1063 upstream files changed (+31k/-35k lines in src/ and libs/); our side changed 349 files (+41k/-1k, mostly new pc-port files).
- 92 files conflicted (317 hunks). Resolution kept every pc-port (`TARGET_PC`) behaviour and took upstream's structure.
- Upstream moved or split units: `src/placeholder.h` and `m2c_macros.h` moved to `libs/doldecomp/include/`; the `*.static.h`
  headers were folded into their `.c` (lbheap, lbaudio_ax, mndatadel, mnstagesel, synth); `soundtest.c` split into `if_2FFC.c` +
  `if_3004.c`; the tail of `gm_17C0.c` moved to `gm_17DB.c`; new TUs `ft_0763.c`, `crowdsfx_1.c`, `hsd_3910.c`. Our edits to the
  folded/split code were re-applied by hand (see "PC re-applications").
- The workspace pipeline (`_build/masstest/pipe_win.sh`, `files.txt`, link rsp) only knows the old TU list and include roots, so this
  branch carries three forwarding headers `pc/placeholder.h`, `pc/dat_macros.h`, `pc/m2c_macros.h` (the pipeline passes `-Ipc`).
  The workspace `files.txt` and `melee_link_objects.rsp` need these six entries before the branch builds in the shared lanes:
  `src/melee/ft/ft_0763.c src/melee/gm/gm_17DB.c src/melee/if/if_2FFC.c src/melee/if/if_3004.c src/melee/sfx/crowdsfx_1.c src/sysdolphin/baselib/hsd_3910.c`.

## Traps found (read before the next sync)
1. `TARGET_PC` is also upstream's macro now. Upstream's own `TARGET_PC` blocks (dat-cli native archive readers in `archive.h`,
   `archive.c`, `lbarchive.c`, `lbdvd.c`) turn on in our pipeline (`-DTARGET_PC`) and reference `HSD_ArchiveGetPublicTyped` /
   `HSD_ArchiveReleaseNative`, which only tools/dat-cli's native library provides. They are re-guarded with `DAT_NATIVE_ARCHIVE`
   (never defined here), so upstream's `HSD_ArchiveGetPublicAs` stays the plain cast.
2. `ft_80459B88_t` (Kirby copy-ability table) changed from `hats[Ft_Kind_Max]` at +4 to `copies[Ft_Kind_Max + 1]` at +0, i.e.
   `hats[k] == copies[k + 1]`. Upstream re-spelled every index by FighterKind (`hats[Ft_Kind_Koopa]` -> `copies[Ft_Kind_Link]`).
   Our code keeps working through a `TARGET_PC` anonymous union in `ft/types.h` that exposes both `copies[]` and `hats[]`. A Rust
   mirror that indexes the old way must shift by one entry.
3. `CommandInfo`'s anonymous union is now named `x8` (`cmd->x8.u`); `TARGET_PC` keeps both spellings with an outer anonymous union.
4. `LBHEAP_HEAP_COUNT` (m-ex's 7th heap) now lives in `lb/lbheap.h` next to upstream's `LbHeapKind`; `LbHeapKind_Count` (6) is still
   the "no predecessor" sentinel in the descriptor table.
5. `GObj_RenderFunc` takes `intptr_t` as its second argument; every pc render callback had to follow (clang rejects `int`).
6. Process trap that cost time: an early resolver pass applied per-hunk token renames to our side of a hunk and replaced unrelated
   tokens in the pc-port additions (`extern` -> `int`, `float` -> `return`, `i` -> `NULL`, words in comments). It was caught by
   comparing every line of every conflicted file against {ours, base, theirs} (a "novel line" scan) and repaired from ours. Keep that
   scan in the loop for any future 3-way merge against rename-heavy upstream hunks.

## PC re-applications (upstream refactors that swallowed our edits)
- lbheap.c: taken from upstream; our 7-heap table, `Scene` name, trace, report guard, `lbHeap_IdOfHandle`, `lbHeap_Capacity/Free`
  and `lbheap_stage_slots.inc` re-added.
- mnstagesel.c: static.h contents folded in with our `SSS_ICON_MAX`/`NUM_STAGES` m-ex expansion; `mnStageSel_804D6C98` -> `sss_models`.
- lbaudio_ax.c, synth.c, mndatadel.c: our static.h hunks applied as patches.
- soundtest.c / if_2FFC.c / if_3004.c: only the dev-menu "Unlock All" entries and their functions were re-applied; our `un_803FA258`
  overlay workarounds are obsolete (upstream typed the data) and were dropped.
- gm_17DB.c: the four pc hunks that belong to `gm_8017DB88` / `fn_8017DE54`.
- ftcoll.c: upstream refactored the clank and knockback helpers (`updateClankDamage`, `recordItemClank`; `ftColl_80079C70` now calls
  `ftColl_80079AB0`); our echo-capsule checks and `ScriptGame_HitRule*` hooks were moved into the new helpers.
- ftCo_09F7.c: m-ex custom effect ids and the Geno "no joint for body part" guard re-inserted into the goto-free version.
- toy.c / mndiagram.c / mnvibration.c: upstream typed these, so our raw-offset pc hacks (`TOY_*`, `MNVIB_LAYOUT`, the `assets`
  overlay hand-off) were dropped. Behaviour of the stage-select, trophy and records screens on the pc build is therefore unverified.

## Build (C reference)
- Private lane: `_build/agents/upstream-sync` (made with the `agent_new.sh` steps minus the worktree add; GW_MELEE = this worktree).
- Compile check: all 993 TUs of the workspace list plus the 6 new TUs pass `clang -fsyntax-only` with the pipeline flags
  (`ppc32-none-eabi -std=c99 -DTARGET_PC ... -Isrc -Ipc`), then the real gwtool pipeline built all of them; full `build.sh` link and
  bridge fixpoint passed (`melee-pc.exe`, build id `d373bba89+local`).
- Because `files.txt` / `melee_link_objects.rsp` are shared workspace files they were not edited: the 6 new TUs were compiled with
  `build_objects.py game --tu ...` and appended to a private `upsync_link_objects.rsp`, selected through `GW_LINK_OBJECTS`.
- Fast error loop used (23 s for all TUs, 12 jobs): `xargs -P12 clang -fsyntax-only` over `files.txt` with the flags from
  `_build/masstest/pipe_win.sh`; then `posfix` (error-position-driven member renames). The gwtool compile of ~1000 TUs takes a few minutes.
- Only game TUs and the link were exercised. The pc native tests (`run.sh --test tests`) were not run; see "Not verified".

## C goldens
Command: `tools/rust-port/golden/record.sh` logic with the output redirected to `_build/agents/upstream-sync/rustgold` (one pass per
scenario instead of two, `docs/rust-port/golden/*.csv` not rewritten), `MELEE_TURBO=1`, window parked off-screen at 30000,30000,
`MELEE_WINDOW_UNFOCUSED=1`, pads driven by `MELEE_PAD_SCRIPT` only, then `tools/rust-port/compare_golden.py old new`.

| scenario | frames | result |
|---|---|---|
| pad_falcon_fox_fd | 1801 | identical to docs/rust-port/golden |
| pad_marth_sheik_bf | 1801 | identical |
| cpu_fox_falco_dl | 3601 | identical |
| cpu_peach_samus_ys_items | 3601 | identical |

Changed goldens and the upstream commit that caused each: none. Suspects that did not move any of the four (they would show up
in other scenarios): c39895846 (integer damage for hitlag) and none else is behavioural among the 116 commits by subject; the rest are
renames, typing, goto removal, named-constant substitution and dat tooling (which are meant to be behaviour-neutral).
Determinism was not re-proved (single pass); the goldens being identical to the committed ones is the evidence.

### Not verified
- Single pass per golden (record.sh does two); identical-to-committed makes that moot for these four.
- The pc test suite (`run.sh --test tests`) and any m-ex/ACE disc paths: not run. Places where upstream typed the code and our raw
  pc hacks were dropped (toy.c trophy list, mndiagram.c, mnvibration.c, if_2FFC/if_3004 soundtest menu, mn stage select
  `sss_models`) compile but are untested at runtime on an m-ex disc.
- The auto-resolved hunks (about 200 of 317) were reviewed by compile errors and a novel-line scan, not read one by one.

## Upstream commits that rename symbols or retype fields that the Rust mirrors use
Rust mirrors C names directly (`melee_types` is generated from the C headers; per-function ports keep the C symbol).
Word hits are counted over `rust/` (`.rs/.py/.txt/.tsv/.toml/.json`), so they include the generated tables.
Mirror-relevant commits (touching `ft/types.h`, `mp/types.h`, `it/types.h`, `gm/types.h`, `lb/types.h`, `gr/types.h`, `ftlib.h`,
`fighter.h`, baselib particle/generator/gobj/jobj, mpisland.h, mplib.h):

- a62c7a329 2026-10-23 rename symbols, remove pad stack in hsd_8039D9C8 (#3680)
- b624b45c6 2026-10-23 use more hsdDelete calls, change hsd_object in aobj to HSD_JObj* (#3682)
- 00f46b175 2026-10-13 grAnime: preserve callback pointers and search results (#3674)
- 88be350af 2026-10-13 Use typed storage and operands in fighter commands (#3671)
- ff9923ac5 2026-10-11 particle: correct bank count and generator callback types (#3660)
- 79b13a324 2026-10-08 dat: type stage and fighter external attributes (#3652)
- e865ef5df 2026-10-08 dat: infer byte-command CPU scripts (#3651)
- 355e23c02 2026-10-07 dat: extend fighter and stage item inference (#3650)
- 73c982624 2026-10-06 dat: correct archive pointer types and list bounds (#3643)
- 2c8a4d519 2026-10-05 dat: count fighter effect bones and color animations (#3642)
- 29b5bd518 2026-10-05 dat: count nested fighter visibility lists (#3636)
- c43d64709 2026-10-05 Fix the sizes of `kills` and `char_data` in `gm/types.h` (#3638)
- 160f05aa0 2026-10-05 Fix todo syntax (#3641)
- 71929abaf 2026-10-05 dat: type orphan scenes and scripts, count more tables (#3634)
- d52f44943 2026-10-05 dat: implement C deserialization codegen, improve coverage (#3632)
- 6b836c777 2026-10-05 dat: type remaining fighter item-slot layouts (#3633)
- c18349268 2026-10-05 dat: select joint views for known fighter item slots (#3631)
- c5564c665 2026-10-05 Type fixes and item enum and reorg work (#3629)
- 12556da73 2026-10-05 dat: report and raise byte coverage, type unused data by address in dat_symbols.txt (#3630)
- a1184c996 2026-10-04 dat: sample each case once, show every piece of each archive, one header per unit (#3628)
- aae5d1558 2026-10-04 dat-cli: preserve loader bindings and type fighter item slots (#3619)
- c05757540 2026-10-03 it: declare Article attribute unions and typed callers (#3600)
- e05c62ea9 2026-10-03 Describe stage joint tables for DAT coverage (#3622)
- 837a9fc82 2026-10-03 Clean up melee/mp (#3621)
- 9d75d14dc 2026-09-30 dat: readable symbol names, local symbols for unnamed data, terminated lists (#3596)
- 772e531d1 2026-09-30 dat: per-archive rebuilds, complete flags, hoist dat types to TU headers (#3594)
- d532c30f7 2026-09-30 dat: fix symbol ordering, format generated source in-place (#3593)
- 5e8cad226 2026-09-30 Expand dat coverage and verify types with objdiff samples (#3588)
- e645a716b 2026-09-29 Item enum and reorg work (#3586)
- 02c58abe3 2026-09-29 Implement initial dat file annotations and tooling (#3587)
- 3f7e7ea64 2026-09-29 Name all anonymous types and ban their usage (#3583)
- 4847219d9 2026-09-25 Name 79 ftLib functions (#3559)
- 8e6529547 2026-09-24 Add misc pointer type clean ups for portability  (#3556)
- c29c67335 2026-09-24 Clean up 74 of 117 remaining compiler pragmas (#3555)
- af680b1e1 2026-09-23 Use pointer sized int for render callback param (#3554)
- 1f0499ba7 2026-09-22 Fighter cleanup and naming (#3550)
- 430c286ba 2026-09-21 Fix native aliasing state views (#3535)
- b4ec422b8 2026-09-21 Clean up some core memory / allocator code (#3547)

Key renames and retypes, by commit:
- 1f0499ba7 Fighter cleanup and naming (#3550): `Fighter.player_id` -> `player_idx`, `x618_player_id` -> `pad_port`, `x619_costume_id` ->
  `costume_id`, `x221F_b3` -> `is_sleeping`, `x2224_b2` -> `stamina_dead`; the per-frame `Fighter_*` procs were renamed (table below).
- 4847219d9 Name 79 ftLib functions (#3559): every `ftLib_8008xxxx` (table below; `ftLib_GetPos` alone has 3480 word hits).
- 3f7e7ea64 Name all anonymous types (#3583): `CommandInfo.x8`, `MatchPlayerData.x3_u.x0.x3_b*`, `Item.xDAA.xDAA_flag`,
  `Fighter.x594.x0.x594_b*`, `Fighter.x594.x0_1.x597_bits`, `Fighter.x221B.x221B_b*`, `FighterBone.x8.x0.flags_b*`,
  `UnkFlagStruct.x0.b*`, `StageCallbacks.x10`, `MotionState.x8.x0.move_id`.
- 837a9fc82 Clean up melee/mp (#3621): `mp_UnkStruct0` -> `mpIsland` (fields `next/vtx0/vtx1/pos0/pos1/flags/line0/line1/joint_id`),
  `mpIsland_80458E88_t` fields -> `floors/ceilings/non_dynamic_*_tail/dynamic_*/b1_*/free_list`, `CollLine.x0` -> `inner`,
  `CollVtx.x0/x10` -> `base_pos/prev_pos`, `CollJoint.x20` -> `jobj`; mpisland.h prototypes retyped to `mpIsland*`.
- 5e8cad226 (dat coverage): `ft_80459B88_t.hats` -> `copies` (+4 -> +0, see trap 2).
- 88be350af typed fighter commands (#3671) and 00f46b175 grAnime (#3674): `HitCapsule.owner` -> `x134.owner`,
  `DmgLogEntry.hit0/hit1/unk_anim0/hurt1` -> `xC.*` / `x10.*`, `Ground_801C34AC_entry` / slot markers retyped.
- af680b1e1 render callback param (#3554), 8e6529547 pointer cleanups (#3556): `GObj_RenderFunc` second arg `int` -> `intptr_t`; several
  `u32` pointers became `uintptr_t` or typed (`a_arenaHi`, `HSD_SList*` heads, `Handle.next/lo/hi/blocks`).
- b4ec422b8 allocator cleanup (#3547): `LbHeapKind`, `Handle` field names.
- a62c7a329 (#3680) and ff9923ac5 (#3660): particle renames `hsd_803991D8` -> `applyForceJObj`, `hsd_8039CF4C` -> `psSetPointJObj`,
  `hsd_804D78E2` -> `numActiveParticles` (non-static for us), generator callback types.
- 772e531d1 / 5e8cad226 / 79b13a324 (dat): `MnSelectStageModels` (`icon_large/icon_random/icon_special/stage_name/...`),
  `SoundTestLoadData`, `UnkStageDat_x8_t` (`joint/anims/matanims/shapeanims`), `ftData.x14/xC/x1C` DAT_EXTENT counts.
- c39895846 Use integer damage values for hitlag calculations (#3679): behavioural, not a rename; first suspect if a golden moves.

### Function renames (header diff keyed by the `/* ADDR */` comment)
| header | addr | old | new | Rust word hits | top Rust crates |
|---|---|---|---|---|---|
| fighter.h | 068354 | `Fighter_UnkProcessDeath_80068354` | `Fighter_Spawn` | 24 | melee_fighter:11,melee_modes:7,melee_link:6 |
| fighter.h | 06A1BC | `Fighter_8006A1BC` | `Fighter_procHitlag` | 7 | melee_fighter:6,melee_core:1 |
| fighter.h | 06A360 | `Fighter_8006A360` | `Fighter_procAnim` | 7 | melee_fighter:7 |
| fighter.h | 06ABA0 | `Fighter_8006ABA0` | `Fighter_procCpu` | 6 | melee_fighter:6 |
| fighter.h | 06AD10 | `Fighter_Spaghetti_8006AD10` | `Fighter_procInput` | 7 | melee_fighter:6,melee:1 |
| fighter.h | 06C5F4 | `Fighter_8006C5F4` | `Fighter_procIK` | 6 | melee_fighter:6 |
| fighter.h | 06C624 | `Fighter_CallAcessoryCallbacks_8006C624` | `Fighter_procAccessory` | 6 | melee_fighter:6 |
| fighter.h | 06C80C | `Fighter_8006C80C` | `Fighter_procCollPos` | 6 | melee_fighter:6 |
| fighter.h | 06CA5C | `Fighter_UnkProcessGrab_8006CA5C` | `Fighter_procGrabColl` | 6 | melee_fighter:6 |
| fighter.h | 06CB94 | `Fighter_8006CB94` | `Fighter_procAttackColl` | 6 | melee_fighter:6 |
| fighter.h | 06D1EC | `Fighter_ProcessHit_8006D1EC` | `Fighter_procCollResolve` | 17 | melee_modes:8,melee_fighter:7,melee_link:2 |
| fighter.h | 06D9AC | `Fighter_8006D9AC` | `Fighter_procDynamics` | 6 | melee_fighter:6 |
| fighter.h | 06D9EC | `Fighter_UnkCallCameraCallback_8006D9EC` | `Fighter_procCamera` | 6 | melee_fighter:6 |
| fighter.h | 06DA4C | `Fighter_8006DA4C` | `Fighter_procPlayer` | 6 | melee_fighter:6 |
| ftlib.h | 0860C4 | `ftLib_800860C4` | `ftLib_CountFighters` | 13 | melee_link:5,melee_fighter:4,melee_items:4 |
| ftlib.h | 086198 | `ftLib_80086198` | `ftLib_FindLowestPercentOpponent` | 11 | melee_link:4,melee_items:3,melee_world:3 |
| ftlib.h | 08627C | `ftLib_8008627C` | `ftLib_FindNearestOpponent` | 68 | melee_items:34,melee_link:28,melee_chara:2 |
| ftlib.h | 086368 | `ftLib_80086368` | `ftLib_FindNearestOpponentInDir` | 11 | melee_items:5,melee_link:4,melee_fighter:2 |
| ftlib.h | 0864A8 | `ftLib_800864A8` | `ftLib_GetOpponentsDir` | 25 | melee_link:12,melee_world:7,melee_items:5 |
| ftlib.h | 0865C0 | `ftLib_800865C0` | `ftLib_GetFacingDir` | 117 | melee_items:59,melee_link:50,melee_fighter:5 |
| ftlib.h | 0865CC | `ftLib_800865CC` | `ftLib_GetGroundAir` | 63 | melee_items:28,melee_link:25,melee_fighter:7 |
| ftlib.h | 0865D8 | `ftLib_800865D8` | `ftLib_GetLStick` | 39 | melee_items:23,melee_link:15,melee_fighter:1 |
| ftlib.h | 0865F0 | `ftLib_800865F0` | `ftLib_GetHipJObj` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 086630 | `ftLib_80086630` | `ftLib_GetPartJObj` | 123 | melee_items:77,melee_link:45,melee_fighter:1 |
| ftlib.h | 086644 | `ftLib_80086644` | `ftLib_GetPos` | 3480 | melee_world:3265,melee_link:124,melee_items:78 |
| ftlib.h | 086664 | `ftLib_80086664` | `ftLib_SetPos` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 086684 | `ftLib_80086684` | `ftLib_GetPrevPos` | 56 | melee_world:43,melee_link:12,melee_fighter:1 |
| ftlib.h | 0866DC | `ftLib_800866DC` | `ftLib_GetCameraBonePos` | 79 | melee_link:42,melee_items:28,melee_fighter:5 |
| ftlib.h | 086724 | `ftLib_80086724` | `ftLib_ReleaseItem` | 9 | melee_items:5,melee_link:4 |
| ftlib.h | 086764 | `ftLib_80086764` | `ftLib_ClearHeldSpecialItem` | 9 | melee_items:5,melee_link:4 |
| ftlib.h | 0867A0 | `ftLib_800867A0` | `ftLib_IsHoldingItem` | 18 | melee_items:12,melee_link:5,melee_fighter:1 |
| ftlib.h | 0867CC | `ftLib_800867CC` | `ftLib_GetHeldSpecialItem` | 17 | melee_items:11,melee_link:5,melee_fighter:1 |
| ftlib.h | 0867D8 | `ftLib_800867D8` | `ftLib_IsInputDisabled` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 0867E8 | `ftLib_800867E8` | `ftLib_DisableInput` | 15 | melee_link:8,melee_fighter:7 |
| ftlib.h | 086824 | `ftLib_80086824` | `ftLib_DisableAllInput` | 6 | melee_modes:4,melee_link:2 |
| ftlib.h | 08688C | `ftLib_8008688C` | `ftLib_EnableInput` | 9 | melee_fighter:5,melee_link:4 |
| ftlib.h | 0868A4 | `ftLib_800868A4` | `ftLib_EnableAllInput` | 16 | melee_modes:9,melee_link:6,melee:1 |
| ftlib.h | 0868D4 | `ftLib_800868D4` | `ftLib_IsItemVisible` | 10 | melee_link:6,melee_items:4 |
| ftlib.h | 086960 | `ftLib_80086960` | `ftLib_IsFighter` | 248 | melee_items:103,melee_link:78,melee_world:24 |
| ftlib.h | 086984 | `ftLib_80086984` | `ftLib_GetCollData` | 35 | melee_link:15,melee_world:14,melee_items:5 |
| ftlib.h | 086990 | `ftLib_80086990` | `ftLib_GetECBCenter` | 10 | melee_link:6,melee_items:4 |
| ftlib.h | 0869D4 | `ftLib_800869D4` | `ftLib_GetModelScale` | 72 | melee_items:36,melee_link:35,melee_fighter:1 |
| ftlib.h | 086A0C | `ftLib_80086A0C` | `ftLib_GetScale` | 47 | melee_items:31,melee_link:15,melee_fighter:1 |
| ftlib.h | 086A18 | `ftLib_80086A18` | `ftLib_IsShielding` | 7 | melee_link:4,melee_items:3 |
| ftlib.h | 086A58 | `ftLib_80086A58` | `ftLib_GetScreenPos` | 0 |  |
| ftlib.h | 086A8C | `ftLib_80086A8C` | `ftLib_UpdateScreenVisibility` | 6 | melee_fighter:4,melee_link:2 |
| ftlib.h | 086B64 | `ftLib_80086B64` | `ftLib_IsOffscreen` | 15 | melee_world:9,melee_link:5,melee_fighter:1 |
| ftlib.h | 086B74 | `ftLib_80086B74` | `ftLib_GetCameraSubject` | 45 | melee_world:34,melee_link:10,melee_fighter:1 |
| ftlib.h | 086B90 | `ftLib_80086B90` | `ftLib_GetCameraSubjectBonePos` | 36 | melee_world:16,melee_link:15,melee_fighter:5 |
| ftlib.h | 086BB4 | `ftLib_80086BB4` | `ftLib_IsCameraSubjectInBounds` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 086BE0 | `ftLib_80086BE0` | `ftLib_GetPlayerIndex` | 258 | melee_world:169,melee_link:40,melee_items:31 |
| ftlib.h | 086BEC | `ftLib_80086BEC` | `ftLib_GetPosDelta` | 37 | melee_world:17,melee_items:12,melee_link:7 |
| ftlib.h | 086C18 | `ftLib_80086C18` | `ftLib_StartRumble` | 225 | melee_world:220,melee_link:5 |
| ftlib.h | 086C9C | `ftLib_80086C9C` | `ftLib_StartRumbleAll` | 10 | melee_world:8,melee_link:2 |
| ftlib.h | 086D40 | `ftLib_80086D40` | `ftLib_StartItemRumble` | 30 | melee_link:16,melee_items:14 |
| ftlib.h | 086DC4 | `ftLib_80086DC4` | `ftLib_StartItemRumbleAll` | 9 | melee_items:5,melee_link:4 |
| ftlib.h | 086E68 | `ftLib_80086E68` | `ftLib_StopItemRumble` | 9 | melee_items:5,melee_link:4 |
| ftlib.h | 086EB4 | `ftLib_80086EB4` | `ftLib_GetTeam` | 45 | melee_items:29,melee_link:15,melee_fighter:1 |
| ftlib.h | 086EC0 | `ftLib_80086EC0` | `ftLib_IsInHitstun` | 1215 | melee_world:1209,melee_link:5,melee_fighter:1 |
| ftlib.h | 086ED0 | `ftLib_80086ED0` | `ftLib_CanBeMagnified` | 12 | melee_link:6,melee_world:6 |
| ftlib.h | 086F80 | `ftLib_80086F80` | `ftLib_GetNameTagHeight` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 086FA8 | `ftLib_80086FA8` | `ftLib_IsSwordSwing` | 8 | melee_items:4,melee_link:4 |
| ftlib.h | 086FD4 | `ftLib_80086FD4` | `ftLib_IsSamePlayer` | 23 | melee_link:12,melee_items:8,melee_world:2 |
| ftlib.h | 08701C | `ftLib_8008701C` | `ftLib_IsSleeping` | 38 | melee_link:15,melee_world:15,melee_fighter:8 |
| ftlib.h | 08702C | `ftLib_8008702C` | `ftLib_IncFighterRefCount` | 10 | melee_link:6,melee_items:4 |
| ftlib.h | 087050 | `ftLib_80087050` | `ftLib_DecFighterRefCount` | 10 | melee_link:6,melee_items:4 |
| ftlib.h | 087074 | `ftLib_80087074` | `ftLib_GetShakeOffset` | 17 | melee_link:8,melee_items:6,melee_fighter:3 |
| ftlib.h | 0870BC | `ftLib_800870BC` | `ftLib_GetSubColor` | 11 | melee_items:6,melee_link:4,melee_fighter:1 |
| ftlib.h | 0870F0 | `ftLib_800870F0` | `ftLib_SetPercent` | 9 | melee_fighter:5,melee_link:4 |
| ftlib.h | 087120 | `ftLib_80087120` | `ftLib_GetPercent` | 20 | melee_fighter:8,melee_link:8,melee_chara:4 |
| ftlib.h | 0871A8 | `ftLib_800871A8` | `ftLib_ApplyMetalBox` | 17 | melee_link:8,melee_fighter:6,melee_items:3 |
| ftlib.h | 0872B0 | `ftLib_800872B0` | `ftLib_GetShadow` | 28 | melee_link:10,melee_world:9,melee_runtime:8 |
| ftlib.h | 0872BC | `ftLib_800872BC` | `ftLib_CastsShadow` | 12 | melee_link:6,melee_runtime:6 |
| ftlib.h | 087300 | `ftLib_80087300` | `ftLib_GetLastAttackerSlot` | 19 | melee_fighter:11,melee_link:8 |
| ftlib.h | 08730C | `ftLib_8008730C` | `ftLib_IsLastAttackerSubFighter` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 08731C | `ftLib_8008731C` | `ftLib_IsSleeping_8008731C` | 41 | melee_world:24,melee_link:10,melee_fighter:4 |
| ftlib.h | 08732C | `ftLib_8008732C` | `ftLib_IsDead` | 648 | melee_world:607,melee_link:25,melee_fighter:16 |
| ftlib.h | 087354 | `ftLib_80087354` | `ftLib_IsDeadUp` | 9 | melee_fighter:5,melee_link:4 |
| ftlib.h | 08737C | `ftLib_8008737C` | `ftLib_IsDeadUpStar` | 9 | melee_fighter:5,melee_link:4 |
| ftlib.h | 0873A4 | `ftLib_800873A4` | `ftLib_IsDeadUpFall` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 0873CC | `ftLib_800873CC` | `ftLib_IsRebirth` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 0873F4 | `ftLib_800873F4` | `ftLib_IsEntry` | 17 | melee_world:14,melee_link:2,melee_fighter:1 |
| ftlib.h | 08741C | `ftLib_8008741C` | `ftLib_FindBySpawnNum` | 11 | melee_fighter:7,melee_link:4 |
| ftlib.h | 087454 | `ftLib_80087454` | `ftLib_GetCrowdKnockback` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 087460 | `ftLib_80087460` | `ftLib_GetSpawnNum` | 7 | melee_link:4,melee_fighter:3 |
| ftlib.h | 0874BC | `ftLib_800874BC` | `ftLib_IsSubFighter` | 52 | melee_link:25,melee_fighter:10,melee_world:9 |
| ftlib.h | 087508 | `ftLib_80087508` | `ftLib_LoadFighterCostume` | 19 | melee_modes:10,melee_link:6,melee_fighter:3 |
| ftlib.h | 087574 | `ftLib_80087574` | `ftLib_LoadFighterAllCostumes` | 7 | melee_modes:4,melee_link:2,melee_fighter:1 |
| ftlib.h | 087610 | `ftLib_80087610` | `ftLib_LoadKirbyHats` | 14 | melee_link:6,melee_fighter:4,melee_modes:4 |
| ftlib.h | 0876B4 | `ftLib_800876B4` | `ftLib_IsFramesRemaining` | 8 | melee_modes:5,melee_link:2,melee_fighter:1 |
| ftlib.h | 0876D4 | `ftLib_800876D4` | `ftLib_IsChargingSmash` | 8 | melee_link:4,melee_items:3,melee_world:1 |
| ftlib.h | 0876F4 | `ftLib_800876F4` | `ftLib_GetLastHitSourceType` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 087700 | `ftLib_80087700` | `ftLib_GetLastHitSourceKind` | 8 | melee_fighter:4,melee_link:4 |
| ftlib.h | 08770C | `ftLib_8008770C` | `ftLib_GetGameWatchColor` | 10 | melee_items:6,melee_link:4 |
| ftlib.h | 087744 | `ftLib_80087744` | `ftLib_GetGameWatchOutlineColor` | 10 | melee_items:6,melee_link:4 |
| ftlib.h | 08777C | `ftLib_8008777C` | `ftLib_GetGroundSlopeAngle` | 14 | melee_fighter:10,melee_link:4 |
| ftlib.h | 0877D4 | `ftLib_800877D4` | `ftLib_IsBeingNudged` | 9 | melee_fighter:5,melee_link:4 |
| grgreens.h | 213B1C | `fn_80213B1C` | `grGreens_80213B1C` | 9 | melee_world:9 |
| generator.h | 39D1E4 | `hsd_8039D1E4` | `psSetUserFunc` | 16 | melee_world:7,melee_link:5,melee_runtime:4 |
| generator.h | 39D1EC | `hsd_8039D1EC` | `psGetNewIDNum` | 15 | melee_runtime:10,melee_link:5 |
| particle.h | 3991D8 | `hsd_803991D8` | `applyForceJObj` | 5 | melee_runtime:5 |
| particle.h | 39CF4C | `hsd_8039CF4C` | `psSetPointJObj` | 5 | melee_runtime:5 |
| particle.h | 39D048 | `hsd_8039D048` | `psClearPointJObj` | 15 | melee_modes:7,melee_runtime:6,melee_link:2 |

### Field renames found by the offset-comment scan (partial: only structs that carry offset comments)
| header | struct | offset | old | new | Rust word hits |
|---|---|---|---|---|---|
| types.h | ftData | 14 | `x14` | `DAT_EXTENT` | 17007 |
| types.h | ftData | c | `xC` | `DAT_EXTENT` | 13480 |
| types.h | ftData | 1c | `x1C` | `DAT_EXTENT` | 4140 |
| types.h | UnkStageDat_x8_t | 0 | `unk0` | `joint` | 808 |
| types.h | UnkStageDat_x8_t | 4 | `unk4` | `anims` | 419 |
| types.h | UnkStageDat_x8_t | 8 | `unk8` | `matanims` | 302 |
| types.h | UnkStageDat_x8_t | c | `unkC` | `shapeanims` | 172 |
| types.h | UnkStageDat_x8_t | 14 | `x14` | `camera_anim` | 17007 |
| types.h | UnkStageDat_x8_t | 28 | `x28` | `DAT_EXTENT` | 3590 |
| itCharItems.h | itToolsAttributes | 10 | `motions` | `DAT_EXTENT` | 36 |
| itCommonItems.h | itSScopeAttributes | 0 | `pad_x0` | `x0` | 38 |
| types.h | mpIsland_80458E88_t | 0 | `next` | `floors` | 12781 |
| types.h | mpIsland_80458E88_t | 4 | `x4` | `ceilings` | 19267 |
| types.h | mpIsland_80458E88_t | 8 | `x8` | `non_dynamic_floors_tail` | 13918 |
| types.h | mpIsland_80458E88_t | c | `xC` | `non_dynamic_ceilings_tail` | 13480 |
| types.h | mpIsland_80458E88_t | 10 | `x10` | `dynamic_floors` | 55637 |
| types.h | mpIsland_80458E88_t | 14 | `x14` | `dynamic_ceilings` | 17007 |
| types.h | mpIsland_80458E88_t | 18 | `x18` | `b1_floors` | 10558 |
| types.h | mpIsland_80458E88_t | 1c | `x1C` | `b1_ceilings` | 4140 |
| types.h | mpIsland_80458E88_t | 20 | `x20` | `free_list` | 11254 |
| types.h | CollLine | 0 | `x0` | `inner` | 21090 |
| types.h | CollVtx | 0x00 | `x0` | `base_pos` | 21090 |
| types.h | CollVtx | 0x10 | `x10` | `prev_pos` | 55637 |
| types.h | CollJoint | 0x20 | `x20` | `jobj` | 11254 |

## Unmatched functions
Upstream commits no function-level list. What exists:
- `configure.py`: all 1117 `Object(...)` entries are `Matching` (plus 4 `Debug`), none `NonMatching` or `Equivalent`. Progress
  numbers come from objdiff reports built by CI (`.github/workflows/build.yml`), which need the retail DOL and are not in the repo.
- `libs/dolphin/nonmatchings/`: `GXRedirectWriteGatherPipe.s`, `GXRestoreWriteGatherPipe.s`, `__GXVerifyTEV.s`. These are the only
  committed unmatched functions (asm only).
- 271 `MUST_MATCH` sites in 143 files and 34 compiler-pragma sites mark code that differs between the matching build and the
  portable build (`git grep -n MUST_MATCH -- src`).
- Rust relevance: none of the three asm-only dolphin functions is mirrored; the `MUST_MATCH` branches are not compiled by the pc pipeline.

## Follow-ups for the Rust side (not done here, rust-port unchanged)
1. Regenerate `melee_types` (`rust/melee_types/gen`) from this branch's headers and diff `gen.rs`; expect the Fighter, mpIsland,
   DmgLogEntry, CommandInfo, MatchPlayerData and ft_80459B88 changes above.
2. Rename the 103 C symbols in Rust (a sed over `rust/` with the table above is mechanical); `ftLib_GetPos` (old `ftLib_80086644`)
   and `ftLib_IsInHitstun` (old `ftLib_80086EC0`) dominate.
3. Pick up `c39895846` (integer hitlag damage) in the Rust hitlag code if a golden moves.

## Shared `_build` lanes after the sync (port/gdsync, 2026-11-05)
- An earlier gdsync run had appended the 6 new TUs to the workspace's shared `_build/masstest/files.txt` and
  `_build/melee_link_objects.rsp` and patched `tools/port/link_response.py` (skip missing objects), all uncommitted in the
  workspace repo. Those three edits are **reverted** (the files equal the pre-edit backups / the workspace HEAD), so every other
  lane is as before.
- A synced tree builds its C reference with `tools/rust-port/c_lane_newtus.sh` instead: it compiles the TUs of
  `docs/rust-port/game_tus.txt` that the shared list lacks into the lane (`build_objects.py game --tu`) and writes
  `<lane>/melee_link_objects_newtus.rsp`; then `GW_LINK_OBJECTS=<that rsp> tools/port/build.sh`. Verified on the lane
  `_build/agents/upstream-sync` with `GW_MELEE=worktrees/gdsync`: build OK, bridge audit clean.
- The tree also needs the generated (disc-derived, git-ignored) `build/GALE01/include` (copy it from `<workspace>/melee`), else
  `hsd_3915.c` fails on `debug_font.inc`.
- When the workspace's shared lists are updated for good (after rust-port carries the sync), the script prints 0 extra TUs and
  is a no-op.

## tu_gm_1798 oracle re-point (user-approved)
`rust/melee_modes/oracle_tu/cfgs.py` patches the oracle's copy of gm_1798.c so fn_8017AA78's six `*(U32Pair*) u16[4] = pair`
stores write the GameCube halves order (the native LE oracle swapped them; the game build does not). The C game source is unchanged.
Regenerated from a 66d48cc1b tree export (in-tree regeneration fails: `rust/melee_types/gen/all_types.c` still includes the folded
`*.static.h` headers, follow-up 1 above): byte-identical to the committed `tests/data/tu_gm_1798.txt`, 200 rows differ from the old
oracle, all fn_8017AA78.
