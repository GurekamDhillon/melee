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
