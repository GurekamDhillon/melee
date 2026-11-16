# m-ex fighter slots in the Rust engine (branch port/mex)

An m-ex disc (ACE, Akaneia) carries its content tables in `MxDt.dat`; fighter mods (`ports/README.md`) replace a row
of it. The C build reads it in `pc/platform/gw_mex_ftfunction_runtime.c` (+ `gw_mex_grfunction.c`, `gw_mex_graudio.c`).

| part | where |
|---|---|
| mexData readers (slots, names, Pl/AJ files, costumes, demo strings, effect banks, sound banks, music, menu params, Kirby copy rows, stage tables) | `rust/melee_engine/src/mex_host.rs` (MxDt.dat relocated into a low block, read big-endian as the C does) |
| `fighter_function` / `kirby_function` callbacks | a vanilla guest address resolves to the native entry of the retail row (internal 0..26, the six bosses) that carries it in the same table (`ftData_MexRetailSlot`); one no retail row carries is left empty (the clone base's), as the C leaves an unbridged one |
| m-ex motion-row rewrite (`ftData_8008572C`) | the row walk runs on the raw image in the decoder (`dat_custom::mex_motion_rows`); the decoder is told the loading kind (`FTDATA_LOADING_KIND`: PlBm.dat names its data `ftDataKirby`) |
| m-ex symbols | `Stc_icns` (IfAll stock atlas) and `effBehaviorTable` (an effect bank's behaviours) decoders in `dat_custom.rs` |
| mod-only files | `Gcm` gives a path only the mods have an entry past the FST (PlBm.dat, EfBmData.dat) |
| Geno on an m-ex slot | the boot registry reads words files and the dense slots through `mex_host::GenoEnv` (geno.json `PlBm.dat` -> kind 57) |
| PC palette POBJs (`engine.pobj_palette: 1`) | `melee_gx`: GX_AURORA_LOAD_PALETTE / END_PALETTE in the FIFO, `GxState::palette`, `pn_palette` in the pipeline config, the slots in `abuf` at `imm._pad` (Aurora's path). Sim unaffected; checked by `melee view-golden` shots |
| launch | `MELEE_SCENE="mode=vs;at=match;p1=<token>/cpu9;p2=fox;stage=fd"`: `melee play` (no menus) and the coverage scenes. Tokens: the disc's names (`brawlmetaknight`, `ultimatekirby`, `ultimatesora` - the installed name, see ports/README.md), `mexint:N`, `mex:N`, `mexext:N`, resolved once the disc mounts |

Not ported: the PPC interpreter / guest `ftFunction` code (none of the installed fighters has a dispatched override:
Halberd's one, slot 45 GetTrailData, is not dispatched by the C either; ACE's own fighters with guest code - its Meta
Knight PlMk.dat, Wolf - would need it), custom m-ex stages (GrFunctionInit / row callbacks), custom items, the CSS/SSS
m-ex tables (the scene launch skips them).

## Coverage (group `mex`, iso ACE)

Mods are local (`bash tools/rust-port/golden/coverage/mex_mods.sh` copies the installs into `_build/mex-mods/<set>`;
nothing generated is committed). Goldens: `record_coverage.sh --bin _build/agents/sfxparity --group mex` (each twice,
identical). `run_rust.sh` reads the scenario's `iso` (`COV_ISO` -> `GW_ISO_<NAME>`).

| scenario | result |
|---|---|
| `mex_ace_cpu` (ACE disc alone, Kirby vs Fox) | PASS 3601/3601 |
| `mex_halberd_cpu`, `mex_halberd_bhv` (Brawl Meta Knight, Geno specials) | PASS 3601, 2401 |
| `mex_ukirby_cpu` (Ultimate Kirby) | PASS 3601 |
| `mex_sora_cpu` (Sora, Kirby host) | PASS 3601 |
| `mex_sorapal_cpu`, `mex_sorapal_bhv` (Sora, palette build, Marth host, Geno articles) | PASS 3601, 2401 |
| `mex_goku_cpu` (goku-xv2, vanilla disc, a base "none" Geno define) | PASS 3601 (port/genonone) |

Open: FD on the ACE disc draws magenta edges in `view-golden` (a texture the renderer does not decode; render only).

## ACE v2.0.1 (Nov 16, branch port/ace201)

Disc: the release's `patch.xdelta` applied with xdelta3 to a clean 1.02 disc (`GW_ISO_ACE201`; `GW_ISO_ACE` stays the
older build). `melee mexid` now also lists the stage table (`stage=ext:N` -> grkind, file, `mex` for m-ex stages).

- New fighters: Liu Kang (`liukang`, PlKg, ftFunction with 14 overrides) and Sub-Zero (`subzero`) load, move and fight
  CPU 9 vs Fox; the VS CSS shows the ACE grid with both. Wireframes / Giga Bowser are the vanilla kinds (`ck:27`, `ck:28`,
  `ck:29`) and play. Liu Kang's and Chun-Li's onLoad end in `bctrl ftSk_Init_OnLoad`; a direct call to a function the
  generator bridges only as a gobj callback (`GOBJ_FN_NAMES`) now goes through `mex_bridge::gobj_fn` instead of
  faulting as unresolved (the whole guest onLoad was dropped).
- Kirby's copy of internal kind 50 uses effect bank 50: the gate is now the particle system's 65 banks
  (`efAsync_DatEntries`), not the C's 50.
- m-ex stage rows (round 2): the engine's `Mex_GrRowCallbacks` / `Mex_GrOn*` hooks were seams_gen stubs, so an added
  stage got no StageCallbacks[] and no handlers (Suzaku drew nothing; Ganon's Tower, Mad Monster Mansion and N64
  stopped in `Ground_801C43C4`'s HSD_ASSERT 3652 with the default lights). `mex_host` now matches each row word against
  the retail StageData rows in the DOL (`stage_datas`, 0x803DFEDC) and hands back the native row's table / handler of
  that clone base. ACE 2.0.1's four new stages carry no grFunction blob (Suzaku = Battlefield's handlers, the other three
  Dream Land 64's), so they load, draw and play; the same path serves the other added stages (96..148). A stage WITH a
  grFunction blob still runs only its row's retail handlers (logged). `MELEE_MEXID_ROWS=1 melee mexid` prints the rows.
- codes.gct writes: `melee_core::codes::disc_writes` holds the disc's type-04 writes (393 on ACE 2.0.1). The two new in
  2.0.1 are honoured: 0x800CD05C nop (every fighter takes the Swing42 item-smash motions, `get_anim_id_kind`) and
  0x80146C58 nop (Mewtwo's Confusion reflect no longer sets `x2218_b4`). The other 391 (also on the older ACE disc) are
  not applied generally.
- Bridged by hand: `ftParts_80074B0C`, `ftCommon_CalcSelfAccel_DeaccelAir` (Shadow now runs clean). Meta Knight's onLoad
  then faults on a `memcpy` of 0x2AA0 bytes out of a native MotionState table (guest bytes of a native record array):
  still the clone base's onLoad.
- With port/slp-export2 merged (the disc's codes.gct as the replay's Gecko list) slpcheck takes ACE ids past 32
  (fighters to 65, stages to 365). Our ACE 2.0.1 replays self-check (Liu Kang, stages 357..360) and the oracle Dolphin
  plays them: the four stages match visually; Liu Kang diverges from slippi frame -12 (Dolphin holds a second item, his
  custom article kind 352, which the engine's m-ex custom item path does not spawn).
- Kirby inhaling Knuckles (port kind 55) gets PlKbCpKx and effect bank 50 (pad script, idle Knuckles).
- Wireframes on L-press: not reachable. The CSS L badge draws, but the alt pick is guest code: codes.gct C2 0x802669DC
  loads `AltSplitCSS.dat` / `altcssFunction`, called each CSS frame from C2 0x80263334. Running it needs the CSS side of
  the guest interpreter. The Wireframes / Giga Bowser play when launched as `ck:27..29`.

### Round 4 (Nov 16): a vanilla replay on ACE 2.0.1, thrown victims

- A vanilla-fighter replay (Marth CPU 9 vs Fox CPU 9, FD, 900 frames, `MELEE_SLP_RECORD` + run1.sh) oracled on ACE 2.0.1
  shows the same early camera difference as Liu Kang (`cam x2B4`/`x2B8` two frames behind; their ratio `x2B0`, the
  only value read, is equal) but not the LbShadow / `x2227_b7` / interest ones: those are Liu Kang's. It diverged at
  slippi 234, Fox's first ThrownF frame: the bone inputs were one joint off.
- Cause: codes.gct 04 0x8006FD08 = `lwz r29,0x14C(r2)` over `lbz r29,0x597(r3)` in `ftAnim_8006FCE4` (the
  thrown/remap path). r2+0x14C is m-ex's copy of the metadata's external fighter count (r2+0x148 internal, +0x150 CSS
  icons ... = mexData metadata words 1..), and the `clrlwi 26` after it keeps 6 bits: on ACE 2.0.1 (66) every thrown
  victim decodes with kind 2's (Captain Falcon's) parts tables. Dolphin probe at 0x8006FD1C: r29 = 2, r2+0x14C = 0x42.
  The engine now does the same when the disc has that write (`mex_remap_author_kind`, counts in
  `melee_core::codes::MEX_FT_*_COUNT`). Akaneia (41 -> 41, past the retail kinds) keeps the vanilla author kind:
  unverified, and the likely cause of the Akaneia Tails thrown-pose file. m-ex also gates the remap on fp+0x23EC
  (C2 0x8006FE20, 0x8006F96C) and gives `ftAnim_8006F628` the unmasked internal count (C2 0x8006F66C): not ported.
- After it the replay matches to slippi 394 (5 of 900 frames differ): Fox's CPU lstick (0 vs -112, 32) with every
  dumped CPU field and the seed equal. Open.
- Liu Kang frame 112 (`cpu.x30` 22 vs 29, `x570`): not a draw-count difference (seed column equal, same callers in
  the same order: guest onFrame `HSD_Randi(100)`, `ftCo_800B33B0`, `ftCo_800A4BEC`). On the console the CPU's two draws
  get the values the engine gives the guest call and the CPU's first draw: the guest's per-frame draw sits one
  particle-generator draw (`hsd_8039EE24`) earlier relative to the frame boundary. `MELEE_RAND_TRACE_DEPTH=N` (default
  6) lengthens the trace's caller chain; run the target exe (its PDB) for names.

- Stage sound rows: the "lbAudioAx: N stage audio rows from mexData" OSReport can never print in the Rust engine:
  `lbAudioAx_8002838C` (which calls `lbAudioAx_MexStageAudio`) is not run; the sound is `melee_audio`'s driver, which
  had only the retail 0x6F rows (an m-ex stage got the out-of-range [55, 1, 1]). mex_host now hands the driver the
  mexData rows (`melee_audio::lbaudio::MEX_STAGE_AUDIO`); a run on stage ext:357 logs
  `lbaudio: stage sound rows from mexData (grkind 140: ssm 55 reverb 56 56)`. A row naming a bank past the driver's 55
  (an m-ex SSM) still loads no stage bank.

### Round 5 (Nov 16): the rest of the 0x8006FD08 family

- The whole family is ported (`ftanim.rs`, gated on the 04 at 0x8006FD08; same codes on ACE 2.0.1 and Akaneia). The
  flag m-ex keeps at fp+0x23EC is set by C2 0x80069CE0 in `Fighter_ChangeMotionState`, just before
  `ftData_80085CD8` inside the not-SkipAnim branch: fp+0x23EC = (arg3 != NULL), i.e. "this motion is a thrown pose".
  `ftAnim_8006FE08` / `ftAnim_8006F954` take the remap path on that flag (not on kind != author kind, so a same-kind
  throw remaps too); `ftAnim_8006F628` takes the internal count (unmasked) as its author kind when it is set.
  The engine kept the flag in a side table keyed by the Fighter address (not in a savestate); Round 6 moved it.
- The author kinds index PlCo's own tables (the disc's internal kinds), now kept as `hooks::FT_PARTS_TABLE_RAW` /
  `FIGHTER_804D6540_RAW`. Index "count" is a real table: on Akaneia (41/41) the Dolphin probe at ftPartsRemap
  (Tails 230702, slippi 336) reads `ftPartsTable[41]`, distinct from every fighter's. Akaneia's Tails file 230702
  (open since the step-split rounds, slippi 476) now matches Dolphin over all 10167 frames. ACE 2.0.1 Marth/Fox is
  unchanged (kind 2, first difference still slippi 394).

- Marth/Fox slippi 394 (Fox's CPU lstick 0,0 vs -112,32) is retail undefined behaviour, not m-ex. Fox is in CPU
  behaviour 18 (`ftCo_800AC5A0`, shield DI from `x8c_kb_vel`). With the knockback nearly zero the console code jumps
  to the two `ftCo_800B46B8(SetLstickX/Y)` pushes without setting their arguments: X gets whatever r5 the dispatcher
  (`ftCo_800B2790`, after `ftCo_800ADC28`) left, Y gets the caller's r30 (= fp). On this frame r5 = 0x804D5F90 (the
  HSD_Rand seed address, from the last rand call) -> 0x90 = -112, and r30 = 0x80E8C220 (Fox's console Fighter) ->
  0x20 = 32. Shown by the command buffer (`PROBE_AT=800B3E28 PROBE_MEM=r29+0x44C:0x30`: `80 90 81 20 09 8e 01 7f`)
  and the push probe (r5 = 804d5f90 / 80e8c220 at lr 800ac750 / 800ac760). The engine pushes the C's zero-initialised
  locals. Not ported: exact needs the console's Fighter address (heap layout) and `ftCo_800ADC28`'s exit r5 on every
  path. `MELEE_CPU_TRACE=1` lists every CPU command push (engine frame = slippi + 123) to line up with such probes.
  The Dolphin push hooks log the JIT block's registers, so their argument registers are only right where the logged
  lr is the call's own return address.

### Round 6 (Nov 16): rollback safety, retail uninitialised stick

- The thrown flag (m-ex fp+0x23EC, one word past the retail Fighter) now lives where m-ex keeps it: past the fighter.
  On the m-ex discs `Fighter_800679B0` sizes the fighter pool `size_of::<Fighter>() + 8` and the flag is the word after
  the native struct, so it is arena memory and rides savestates and rollback. Vanilla discs keep the old pool size.
  The word is reached through the gobj's `user_data` (the allocation pointer) with volatile access: the first try,
  through a pointer cast from `&Fighter`, had its store dropped by the optimiser (outside the reference's bytes), so
  the remap never ran and ACE 2.0.1 Kirby vs Falcon panicked "atree data error" at the first throw.
- Proof (synctest, 3000 frames, depths 4 and 7, with a temporary set/read trace): ACE 2.0.1 `mex_ace_cpu` 0
  mismatches with 25 / 40 thrown-flag reads true; Akaneia (ck:17 vs ck:2 on FD, a scratch scenario
  row; `stage_21_akaneia` crashes at boot in `Ground_EnableMatchCamera` in synctest, unrelated) 0 mismatches with
  45 / 72 thrown sets. Re-run on the committed build (no trace, df83104a3): all four 0 mismatches. `MELEE_SCENARIO_TSV` must point at the coverage tsv when not run from the worktree.
- Retail uninitialised stick in CPU behaviour 18 (`ftCo_800AC5A0`), traced in the 1.02 DOL. On the nearly-zero
  knockback path (`bne 0x800AC744` at 0x800AC6D0) the function makes no call before the two pushes, so
  `SetLstickX` gets r5 as it was on entry and `SetLstickY` gets r30 as it was on entry (r30 is callee-saved and only
  loaded on the non-zero paths). The dispatcher `ftCo_800B2790` sets r30 = fp (0x800B27A4), so **Y = low byte of the
  console Fighter address**. Its r5 comes from `ftCo_800ADC28`: every path that pushes ends in
  `ftCo_800B46B8(fp, cmd, 1)`, which keeps r5 = its argument (`ftCo_800B463C` / `ftCo_800B462C` do not touch r5
  unless they assert), so **X = 1** after a push; the early exits (0x800ADC44 cpu+0x18 == 10, and the kind/motion
  checks for kinds 3 and 0xD) leave r5 untouched, and `ftCo_800B462C` (0x800B27EC) does not write it, so X is r5 as
  `ftCo_800B3900` left it after `ftCo_800B2AFC`, i.e. whatever the last r5-writing instruction in that whole call
  tree was (Marth/Fox slippi 394: `HSD_Randi`'s seed address 0x804D5F90 -> 0x90).
- Not ported. Exact needs (a) the console heap address of each Fighter (the port allocates natively; `gc_addr32` is
  identity over the engine's low heaps, there is no console-heap model) and (b) a register-level model of r5 through
  `ftCo_800B2AFC`'s call tree. A partial rule (X = 1 after an ADC28 push, else unknown) would still be wrong on Y.
  The engine and the C build both push the decomp's locals (the Rust engine pushes 0, which is what the goldens
  carry), so goldens are unaffected; this is retail code, so vanilla CPU replays that reach this path with
  near-zero knockback also diverge. No vanilla replay hitting it was checked this round.
- Akaneia corpus rerun (slpcheck, `run_all.sh`, the 51 files of Hero's 2026-11-06 set that `replay_codes.py --disc`
  puts on Akaneia, port/ace201 at the thrown-flag commit): **32 of 51 match** (Round 7 of dolphin-oracle.md: 17).
  The 19 divergers (file, roster, first slippi frame, port, field): 0411T223211 0+28 7017 percent; 0411T223434 0+27
  5124 action; 0411T223810 0+31 1031 x; 0502T230601 31+26 1098 action; 0502T232612 27+0 1264 action; 0704T231714
  7+28 3085 percent; 0704T232232 15+31 790 x; 0710T231752 0+28 2949 percent; 0710T233221 7+31 2122 action (p2);
  0710T233518 7+31 1882 x; 0710T233854 7+28 3907 action; 0718T225033 28+20 916 last_attack; 0718T225603 31+0 81
  action; 0801T234358 9+27 132 percent; 0814T223533 20+28 4551 action; 0814T232949 0+27 2065 action; 0815T231817
  28+1 1061 combo; 0815T232950 31+15 1207 x; 0822T002252 0+28 1596 percent. Not oracled in Dolphin this round.
- Not done this round: Liu Kang frame-112 rand order (item 4) and the Meta Knight multi-table `build_move_table`
  (item 5).

### Round 7 (Nov 16)

- `stage_21_akaneia` boot crash is not an engine bug. `stage=ext:21` is grkind 26, an unused retail slot:
  `stage_datas[26]` is null on every disc (Akaneia's `melee mexid` row: `stage 21 grkind 26 -`). Retail
  `Ground_801C0754` (0x801C0754) has no null check: it loads `stage_datas[kind]->data1` (lwz 8(r30) with r30 = 0) and
  calls `grDatFiles_801C6038` with garbage; the port returns early, so `stage_info.param` stays null and
  `Ground_EnableMatchCamera` reads `param->x4C`. The coverage row was a vanilla stage-sweep row (the C build fails the
  same way, coverage.md). Renamed to `stage_288_akaneia` on `stage=ext:288` (GrOPc, an Akaneia m-ex stage).
  That row now stops at `grOldPupupu_80211110` (`__assert: line 1366: archive` in `grAnime_801C8138`); 289 (GrOHy)
  does the same and 290 (GrOSf) faults: Akaneia's added stages carry grFunction blobs, which are still not run (only
  the clone base's retail handlers run, see round 2). That is the next stage item, not a boot-order bug.
- Game_20260718T225603 (Dedede vs Falcon, first diverger at slippi 81): Dedede's inhale Anim (MoveLogic row 7, console
  0x80D86B3C) compares `ftCo_800BD19C(victim, &pos)` with attr[0x1C]^2. The bridge passed `&Vec3` untyped (the
  generator looked up `Vec3`, the GC record is `Vec`), so the native callee read big-endian floats and returned +inf;
  the swallow never happened. Fixed (REC_ALIAS in gen_mex_bridge.py; ftCo_800BD19C, ftCo_800BDA74): 81 -> 127.
  At 127 Dedede's walk-with-victim (console 0x80D89724) calls `ftWalkCommon_800DFCA4` through a function-pointer word
  in the blob (bctrl), which was not bridged; added (Dolphin: CMS 0x163 from 0x800DFD7C, the engine now does the
  same call). slpcheck still reports slippi 127 (x -42.0 vs -42.0889): not yet oracled after this fix.
- Still untyped pointer args in the bridge that may carry GC-endian records: HSD_MtxSRT, PSMTXConcat (13 in all).
  A full gen_mex_bridge.py regen on this branch also changes unrelated arms (atan2f, ft_8008A1FC, gm_8016B168 drop
  out), so both fixes were applied to the arms by hand.
- `MELEE_CPU_TRACE=1` now also prints `cpub18 frame=F pN near_zero=0|1` for each behaviour-18 stick push pair
  (near_zero=1 is the uninitialised-register path). The replays-ext count was not run.
- Not done: the 51-file rerun, the other 18 divergers, Liu Kang rand order, Meta Knight multi-table.

### Round 8 (Nov 16)

- Untyped pointer args, audited (13 arms passed `c.ptr(i, None)`). Strings (`HSD_ArchiveGetPublicAddress` name,
  `__assert`), `HSD_StateInitTev` (a native render object) and the Item_GObj firsts of `it_802F23AC` / `it_802F28C8`
  (registered native gobjs) are safe. Fixed: `Call::words(i, n)` marshals n plain words (native object field as is,
  guest memory word by word through `ld32`/`st32`, written back in `finish`) for `HSD_MtxSRT` m, `PSMTXConcat` a/b/ab
  (12 words each), `ftColl_80076640` / `ftColl_80078710` f32 outs and `mpCheckAllRemap` line/flags outs;
  `Item_8026AC74`, `it_8026BB68`, `it_8027429C` now pass their Vec3 args as `Some("Vec")`. gen_mex_bridge.py has the
  same rule (`WORD_ARGS`). Akaneia 51 after it: **33 match** (32 before; 0801T234358 now matches, five divergers
  moved later: 223810 1031->1088, 230601 1098->1135, 232232 790->913, 233518 1882->2194, 232950 1207->3246).
- **Regen bug, still open:** a full `gen_mex_bridge.py` run drops unrelated arms (atan2f, ft_8008A1FC,
  gm_8016B168 seen in Round 7), so every bridge change this round was made to `mex_bridge.rs` by hand, with the
  generator edited to match. Do not regenerate until that is fixed.
- 225603 slippi 127 (Dedede walk-with-victim): two misses. (1) `GOBJ_FN` rows carry the port's snake-case names
  (`ftwalkcommon_800e0060`), while `vanilla_name` gives `ftWalkCommon_800E0060`; the binary search missed, the call
  was dropped and Dedede never moved in SpecialNEatWalk (Rust gr_vel 0, console -0.088875 = accel with accel_mul 0.6
  from ftData+0x48). `gobj_fn` / `gobj2_fn` now fall back to a case-blind match (generator too). (2)
  `ftWalkCommon_800DFEC8(gobj, cb)` (the walk-type resync, cb = guest `SpecialN_ASWalk`, called from the IASA through
  a data word) was not bridged; hand_bridge now runs the native function with a trampoline that runs the guest
  callback with the frame in f1 (`run_at_f1`). 225603: 127 -> 4537 (action 382 vs 383, not looked at).
  `MELEE_MEX_CALLTRACE=1` showed both (a call listed, no native effect).
- Rerun after both: still **33 of 51** (the gobj_fn fix only moved 225603). Divergers (file, roster, first slippi,
  port, field): 223211 0+28 7017 percent; 223434 0+27 5124 action; 223810 0+31 1088 x; 230601 31+26 1135 x;
  232612 27+0 1264 action; 231714 7+28 3085 percent; 232232 15+31 913 x; 231752 0+28 2949 percent; 233221 7+31 2122
  action; 233518 7+31 2194 action; 233854 7+28 3907 action; 225033 28+20 916 last_attack; 225603 31+0 4537 action;
  223533 20+28 4551 action; 232949 0+27 2065 action; 231817 28+1 1061 combo; 232950 31+15 3246 action;
  002252 0+28 1596 percent. Largest group: ext fighter 28 (7 files, 4 of them `percent`).
- 0822T002252 oracled (`--at 1580`, scratch r8_o252b): the first difference is slippi 1580, an m-ex custom article
  (item kind 262, fighter 28's) on its spawn frame. Its spawn position is one ulp off (pos.x 15.900756 vs 15.900755)
  and its itemColl was not updated on the spawn frame: Rust cur_pos = the console's last_pos, `x378_itemColl.x38`
  one less. So the console runs one item collision pass at spawn that the port skips (or runs before the position
  is set). Not fixed; that is the next item for the group of 28.
- Behaviour 18 (`MELEE_CPU_TRACE=1`, scratch/replays-ext, 125 vanilla replays, all 125 match): 4 replays reach
  behaviour 18, **2 reach near_zero=1** (3 events in all); both still match, so the uninitialised-register stick did
  not change the outcome there.
- Not done: Liu Kang rand order (probe_lk_rand), Meta Knight multi-table, the other groups. Akaneia's added-stage
  grFunction blobs (ext:288-290) stay a separate item.

## Guest-code bridge notes (ACE crash fixes, Nov 13)

- Floor-line filters (`ft_800831CC`'s first callback, `LineCb`): `Call::line` resolves guest code to a line trampoline
  (gobj in r3, line id in r4) and a retail address to the port's filter (`ftCo_80096CC8`, `fn_800DD6E4`). 0 stays NULL,
  which the C treats as "every line lands" (`mpColl_80044628_Floor`); `adapt::line_tramp` does the same instead of
  panicking. Read as a gobj callback before, the retail `ftCo_80096CC8` (Diddy's SpecialAirHiDamage_Coll, PlTd) and a
  NULL filter (Wario, Meta Knight, Knuckles' side-B air, Chun-Li, PlNm) both ended the game ("adapt: null line callback").
- `ft_800831CC` hands the guest the bool the console leaves in r3 (Knuckles' side-B air Coll tests it).
- An ftFunction with codeSize 0 (older MexTK; 11 ACE slots: PlWr, PlDa, PlLc2, PlRc, PlSm, PlSd, PlFy, PlMk, PlSc, PlLb, PlMM) gets the same
  recovered bound items already had; the empty blob panicked in `build_move_table` as the fighter loaded.
- A fighter GObj passed where a callee takes `Fighter*` resolves to its fighter (Knuckles' side-B air Phys passes the
  gobj to `ftCommon_CalcSelfAccel_Deaccel`; the native callee wrote self_vel past the native GObj onto fp->kind). A
  guest run that still leaves its fighter's kind outside the tables gets the kind back (`guard_kind`, logged once).
- Sweep: `MELEE_SCENE="mode=vs;at=match;p1=mexext:N/cpu9;p2=fox/cpu9;stage=fd"` for every N; `MELEE_MEX_CALLS=1` lists a
  blob's vanilla calls, `MELEE_MEX_DIS=1` disassembles it.
- `fp->dat_attrs_backup` is mapped as `Backing::Words` (guest byte `o` = native byte `o ^ 3`): an onLoad memcpys its
  big-endian attributes there, and the clone base's native specials read them as f32. As plain bytes, PlDl's
  Luigi-base SpecialSStart read byte-swapped floats and its ECB went NaN (`mpCollInterpolateECB` assert); the assert's
  report now also names the fighter kind, motion, anim and frame.

### Round 9 (Nov 16)

- Ext fighter 28 group fixed (Charizard, PlLz, internal kind 35). Its article (item kind 262, itFunction[0]) has one
  state row whose coll callback is a retail address, 0x802AD1E4 `itKoopaFlame_UnkMotion0_Coll`. `fn_native` resolves
  retail callbacks through `GOBJ_FN`, which only holds `void(gobj)` functions, so the row's `collided` was NULL: the
  item skipped its coll proc every frame (the spawn-time `it_8026D9A0` the guest calls itself was the only pass, so
  Rust coll = console's previous pass). `item_i32_fn` (mex_guest.rs, by hand, not generated) now maps the
  bool-returning item callbacks; a named retail callback that still resolves to nothing logs
  `mexguest: vanilla callback ... has no native port` once. None logged on the 51 after the fix.
- Akaneia 51 after it: **41 match** (33 before; all eight ext-28 files). Divergers (file, roster, first slippi, port,
  field): 223434 0+27 5124 action; 223810 0+31 1088 x; 230601 31+26 1135 x; 232612 27+0 1264 action; 232232 15+31
  913 x; 233221 7+31 2122 action; 233518 7+31 2194 action; 225603 31+0 4537 action; 232949 0+27 2065 action;
  232950 31+15 3246 action. Largest group now ext 31 (Dedede, PlDe, internal 38): 7 files.
- 232232 oracled (scratch r9_o232, `--at 913`): Dedede inhales ext 15 (s0). Guest calls
  `Fighter_ChangeMotionState(victim, 0x122, 0x100)`, `ftCommon_8007E2F4(vfp, 0x1FF)`, `ftColl_8007B62C(victim, 2)`,
  `ftCommon_InitGrab`, `ftAnim_8006EBA4`, `ftCommon_8007D5D4`, `ftCommon_SetAccessory` (no guest stores to the
  victim's CollData, MELEE_MEX_TRACE=1). On that frame the console's victim ECB collapses (bottom/left/right 0, top
  4.95, prev_ecb 6.42) and prev_pos is one full 3.87 step back; Rust's ECB is ill-formed (top 0.94, left.y 12.55) and
  prev_pos half a step back (the coll substep count follows ECB size). Not fixed: the next look is which native call
  on that frame gives the console a zero ECB (victim jobj scale after `Fighter_UpdateModelScale`, or the ECB pass
  with the victim's bones hidden).
- 225603 oracled (scratch r9_o603, `--at 4537`): Dedede enters 382 at anim frame 14 (`ChangeMotionState(.., 0x17E,
  0x4000, f1=14)` from the landing callback), both sides reach frame 16 with the same position; the console's anim
  callback (`ftAnim_IsFramesRemaining` false) moves it to 383 at 4537, Rust says frames remain. So the bone AObj end
  test (`lb_8000B074` over the parts) differs for Dedede's anim 328 started mid-way; not looked at further. Baseline
  there also has an item (it0) hitbox live in Rust and cleared in the console from before 4534.
- Not done: Liu Kang rand order, Meta Knight multi-table.

### Round 10 (Nov 16)

- 232232 (Dedede inhale, victim ECB at 913): the native call is `ftCo_800BDA74(victim, &normal)` from Dedede's
  guest accessory callback (slot 2000; MELEE_MEX_CALLTRACE, scratch r10_ct232). It runs `ft_800824A0` on
  `fp->mv.co.thrownkirby.coll_box` (fp+0x2368), which Dedede never writes: the box is whatever the victim's motion
  vars hold, so the console's collapsed ECB (top 4.95, the rest 0) is that leftover box after `mpColl_8004730C`.
  Found and fixed one cause: natively `thrownkirby.thrower_gobj` is 8 bytes, so every later field sat one word past
  its GC offset, while guest stores into `Fighter_MotionVars` land at their GC word (WORD_UNIONS). The pointer now
  sits after the C's 0x40 bytes (gen/native_unions.py, same pattern as itemthrow/Master Hand). Rust's box moved by a
  word (desired_ecb top 0.94 -> 0.042) but still is not the console's: the leftover words at fp+0x236C..0x2374 are
  the guest's earlier `capturekirby.scale` stores (MELEE_MEX_TRACE=1), nonzero on our side; the console's box
  reads 0 there. Next: dump the victim's mv words 0x2340..0x2380 on both sides at 912/913 (Dolphin memory read)
  to see which writer differs. 913 unchanged.
- 225603 (motion 382 end check at 4537): `ftAnim_IsFramesRemaining` and `lb_8000B074` match retail (disassembled:
  part flag byte 0x40 set, 0x80/0x04 clear; jobj2 when x8A4 != 0; AObj present and not NO_ANIM). The data differs:
  with blend 6, every part's jobj2 AObj (the anim skeleton) is at frame 15 of end 16 at the 4537 check, one frame
  behind the root (cur_anim_frame 16), so ours says frames remain. `Fighter_ChangeMotionState`'s anim_start path
  (8006EBE8 with start - rate, then 8006E9B4 twice) reads the same as the C. Not fixed; the next look is which
  pass advances the skeleton AObjs on the entry frame on the console (an extra HSD_JObjAnimAll(x8AC) or a
  FIRST_PLAY consumed earlier). The it0 hitbox item was not looked at.
- Akaneia 51 after the layout fix: still **41 match**, the same ten divergers at the same frames.
- Not done: Liu Kang rand order, Meta Knight multi-table.

### Round 11 (Nov 16)

- 232232 (Dedede inhale, victim ECB at 913) found with Dolphin probes (`PROBE_AT` on the native calls around it,
  `PROBE_MEM=r2+<fixed fp offset>`; the guest's own code dumped from console memory and disassembled). Motion vars
  were not the cause: union layouts and the console's 0x2340..0x2380 words matched ours up to 912, and nothing
  clears motion vars on the state change. On 913 the console's guest (`AS_EnterStarSpitState`) stores the victim's
  `thrownkirby.coll_box` (fp+0x2368..0x237C = attrs * fp->x34_scale.y) right after `ftCommon_SetAccessory`. Ours
  never got there: SetAccessory's joint is a pointer into the guest copy of PlDe.dat (`ft_data->x48_items[0]->x10[0]`),
  which had no native object, so the bridge faulted (MELEE_MEX_REPORT=1 shows these: `mexguest: kind K slot S: ...`)
  and the rest of the callback was dropped. `dat_joint` now decodes a joint tree from the fighter's dat file once per
  guest address.
- 225603 (motion 382 anim end at 4537) needed no change of its own: it matched once the Dedede guest stopped
  faulting. The one-frame skeleton lag of Round 10 was a symptom.
- Every remaining divergence of the round was a guest fault. Fixed: hand bridges (no generator row) for
  `ft_80088510`, `ftCo_800CB110`, `ftKb_SpecialN_800F597C` (bool) and `mpUpdateFloorSkip`, all reached through
  pointers in the blob's data; `*mut Fighter_GObj` / `*mut Item_GObj` fields were untyped pointers in mex-import's
  layout (build.rs did not resolve aliases behind a pointer), so loading fp->x1A5C faulted unless that gobj was
  already registered; an auto-registered array reached 0x2000 bytes, and Diddy's `Fighter_TaroStateChange` reads row
  275 of fp->x1C (0x2260 in): now 0x4000; the common MotionState table's callbacks are closures with no reverse name
  (`map_common_motion_table` pairs each row with the DOL's `ftData_MotionStateList`, 0x803C2800); the fighter's own
  table (fp->x20) holds the shared MOVE_TRAMPS, so `register_move_table` makes those slots read back as the blob's
  guest addresses. Also: an m-ex behaviour-6 effect queued through efAsync kind 0 has no facing argument
  (`efLib_CreateGenerator_AppSRT_SetFacingDir`/`SetFacingDirScale` now skip the rotation instead of reading NULL).
- Akaneia 51: **49 match** (41 before). Left: 223434 (0+27, slippi 5154, p2 y) and 232949 (0+27, 2099, p2 y), both
  Diddy, no guest fault logged. Vanilla 125: 125 match.
- Not done: the two Diddy files, Liu Kang rand order, Meta Knight multi-table.

### Round 12 (Nov 16)

- 223434 (5154) and 232949 (2099), Diddy vs Falcon, no guest fault: the first wide difference was the victim's
  `x221C_u16_y` (0 vs 3) on the frame Diddy's guest re-grabbed it into CaptureCaptain (232949 at 2075). The guest asks for
  anim row 331 of Diddy's 331-row table (mexData's count). The console reads the authored row that follows in PlDd.dat:
  its script runs `ft_8008A1B8(3)`, and `ftData_80085CD8` loads no clip (msid >= x58C), so the victim keeps row 330's.
  Our decode stopped at the count: NULL script, and the `x10` byte pair read past its allocation. `dat_custom` now decodes up
  to 16 rows past the count of an m-ex fighter's `xC` / `x10` when the raw image holds them. Found with
  `MELEE_LOG_FTCMD=1` and a Dolphin probe on `ftData_80085CD8` / `ftData_80085FD4` / `lbArq_80014BD0` (no clip load on
  that frame). Note: `MELEE_DOLPHIN_NHV1_PROBE_CHAIN` caps LEN at 0x1000.
- Guest-fault sweep on ACE 2.0.1: `synctest --frames 1800 --depth 4`, `MELEE_MEX_REPORT=1`,
  `p1=mex:N/cpu9;p2=fox/cpu9;stage=fd` (scratch tsv; `mex:N` = kind N+6, so the m-ex fighters are mex:27..58).
  No synctest mismatch anywhere. Fixed: Lucas (mex:30) with hand bridges `it_80276308`, `ft_80088770`, `ft_80088328`.
  Open (kind: fault):
  - crashes (rc 139): Wario 40, Lucas2 42, PlTd 62, and mex:53 (Luigi 58) rc 127 (each logs one fault first:
    `ftParts_80074A74` on 58, `lbDvd_8001819C` on 40, `it_8026B3F8` on 42, `OSReport` on 62).
  - unresolved, no generator row: `Item_8026AE84` (51, 52), `PSVECNormalize` (48), `ftColl_8007B1B8` (41),
    `it_8026B3F8` (41, 42), `lbArchive_LoadArchive` (44, 46, 48), `ftParts_80074A74` (54, 58), `OSReport` (60, 62),
    `Player_GetPtrForSlot` (49), `GObj_InitUserData` (43), `lbDvd_8001819C` (40).
  - memory: reads of 0x2C (a NULL record + 0x2C) on 41, 44, 46, 48, 49; 0x90004998 on 44 and 64; 0x900DD9DC on 45
    and 47 (Meta Knight's onLoad, the multi-table memcpy); 0x20 in `MexCPU_ProcCustom` on 39.
  - an earlier Lucas run (before `ft_80088328` was bridged) crashed in `it_802799E4` reading 0x37E7C: an item command
    script goto to an unrelocated word, after `Item_80268E5C(state 0)` from the guest. Not reproduced after the bridge.
- Verification: Akaneia 51/51 match, vanilla 125/125, synctest 3000 frames depth 4 and 7 on `mex_ace_cpu` and the
  Akaneia row (ck:17 vs ck:2, FD): 0 mismatches.
- Not done: Liu Kang rand order (probe_lk_rand), Meta Knight multi-table, the open sweep faults above.
