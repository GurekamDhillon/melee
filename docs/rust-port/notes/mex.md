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
  45 / 72 thrown sets. `MELEE_SCENARIO_TSV` must point at the coverage tsv when not run from the worktree.
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
