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
