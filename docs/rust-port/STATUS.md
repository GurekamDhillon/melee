# Rust port: status (read this first; checked against rust-port 5138e0ba5, 2026-11-04)

## Orientation
- **What this is:** a native Rust port of GD's Melee (the PC decomp build), little-endian, no emulation. Game logic is
  bit-identical to the C build (same NHV1 per-frame state hash, float contraction included). It boots the retail
  disc and runs the retail menus, matches, 1P modes, rollback netplay, replays, wasm mods and a LAB.
- **Where the code lives:** the Cargo workspace in `rust/` (crate map below), wasm mods in `rust/mods/`, scripts in
  `tools/rust-port/`, notes in `docs/rust-port/notes/`.
- **How to verify:** bring your own retail NTSC 1.02 disc image and point `GW_ISO_VANILLA` at it (workspace `.env`).
  The repo holds no disc data: goldens are hash CSVs only. `bash tools/rust-port/gate.sh` builds `melee`, checks the
  four main goldens from frame 0, the diff baseline and the logic lint, and prints `GATE OK`. Menu flow check:
  `bash tools/rust-port/play/rematch.sh css`.
- **Headline numbers** (source in brackets):
  - main goldens: pad_falcon_fox_fd 1801/1801, pad_marth_sheik_bf 1801/1801, cpu_fox_falco_dl 3601/3601,
    cpu_peach_samus_ys_items 3601/3601, seed column included (`docs/rust-port/engine_progress.tsv`, step 23).
  - coverage goldens: 212 pass, 0 fail, 10 no golden (C run failed or blocked) (notes/coverage.md "port/correct
    round 5", 2026-11-04; `docs/rust-port/golden/coverage/results.tsv`).
  - external Slippi replays: 38 of 125 frame-exact against the console's post-frame data (notes/slpcheck.md round 2,
    2026-11-02; not re-measured since).
  - speed: about 9x cheaper per sim frame than melee-unlocked's Static Recomp headless playback on one replay
    (notes/bench-unlocked.md, 2026-11-02, one machine).
- **Known gaps:** 87 of 125 external replays still diverge (Stadium / Fountain platforms, 1-ULP contraction sites, hit
  timing; notes/slpcheck.md); launcher parity gaps (online lobby protocols, `--mod-gci` / TE packs, `.slp` playback
  from the launcher, his DS4 / Switch binding tables; notes/launcher-seam.md section 4); consolidation step 23
  unfinished and step 24 (lint enforcing, empty baseline) not started; audio `--pause` / `--time-secs` proofs open;
  user-only checks (visuals, audio by ear, real controllers, online across machines).

## Decisions and hard rules
- Native little-endian Rust, no emulation. Game logic is bit-exact against the C build, which is compiled with
  clang's shipping float contraction (`-ffp-contract=on`): fused sites are ported as `fmadds`/`fmsubs` (f64 then f32),
  never `f32::mul_add`. Oracles are built the same way. Detail: notes/fma.md.
- GameCube offset metadata is kept for every record (`GcLayout` / `GcDecode`, `melee_types`), so C snapshots, DAT
  files and the C oracles read through the same tables. `melee_types` is the ONE generator (rust/melee_types/gen);
  never hand-edit `gen.rs`.
- Rust replaces the C build: no C-netplay wire compatibility is kept (notes/net.md). The port's own "kit" menus
  (`gmfrontend`) are not ported; retail menus only (notes/menus.md).
- Nothing outside the sim may change a hash: the settings overlay, cosmetics, UI / LAB mods and the browser work at
  the render or input boundary only (module docs of `settings_overlay.rs`, `cosmetics.rs`, `browser.rs`).
- NEVER send OS-level input. Input goes through `MELEE_PAD_SCRIPT`, the Rust `PadSystem` script source or in-process
  hooks; test runs use `MELEE_PAD_IGNORE_ADAPTER=1`, `MELEE_WINDOW_UNFOCUSED=1`, `MELEE_WINDOW_TITLE=<who>:<scenario>`.
- NEVER commit disc-derived data (snapshots, DATs, renders, audio). Never print the disc path.
- The golden hash (NHV1, notes/golden.md, docs/rust-port/neutral-hash.md) is the definition of "done" for sim code.
- C builds use a private build root `_build/agents/<name>` (pc/docs/PORT_DEV_QUICKREF.md, workspace `tools/port`).
- Cargo: prefix `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`; test only crates you change.

## Crate map (rust/)
| crate | holds |
|---|---|
| melee_core | fp (`fmadds`), gc layout/decode traits, StateHash, opt-in gameplay codes (`codes`) |
| melee_types | every C struct/enum + GC offset tables (generated from `gen/`), neutral hash |
| melee_runtime | MSL math, HSD (JObj/AObj/particles/RNG), lb, db |
| melee_world | mp (collision), cm, ef, gr (stages), if |
| melee_fighter | ft engine, ftCommon, pl |
| melee_chara | per-character ft code |
| melee_items | it (items), ty, vi |
| melee_modes | gm, mn, script_game, Geno, m-ex tests |
| melee_link | `install()`: every `ext_fns!` hook set to its canonical TU function (generated `gen_table.rs`) |
| melee_engine | engine host: OS heaps, DVD seam, archive images; no game logic |
| melee_platform | window + 60 Hz loop (winit), pads (gilrs, GC adapter, keyboard), controller profiles / box mode + SOCD, launcher bindings |
| melee_gx | GX on wgpu, TEV -> WGSL, texture decode, UI batch renderer, SDF UI fonts (`ui_font.rs`), texture overrides (`tex_override.rs`) |
| melee_audio | AX voice pool, DSP-ADPCM mixer, AXFX, output on cpal |
| melee_net / melee_relay | rollback + UDP transport / self-hostable rendezvous and relay server |
| melee_data | saves, settings.cfg, mods, replays, crash logs |
| melee_script | Lua 5.4 `gd.*` host, Geno registry, LAB mod loader |
| melee_modhost | wasm mod host (mod-c-v1, hooks-v1) and the gd-ui-v1 / gd-lab-v1 presentation host |
| melee (bin) | `melee play` (retail flow), golden / diff / slpcheck / synctest / netplay, launcher compat, settings overlay, cosmetics, browser host |
| rust/mods/* | wasm mods: `lab` (LAB HUD and menus), `browser` (in-game browser), demos |
| tools/mex-import | m-ex mod packer (outside `rust/`) |

## Commands
Env: `GW_ISO_VANILLA` via the workspace `.env` (`set -a; . .env; set +a`). Run from the repo root; `cd rust` for cargo.
- Build: `cd rust && cargo build -p melee --release` (exe: `build/cargo/release/melee.exe`). Tests: `cargo test -p <crate>`.
  Fast loop: `bash tools/rust-port/cargo_fast.sh build -p melee --profile fast` and `PROFILE=fast bash tools/rust-port/gate.sh` (notes/fastbuild.md).
- Golden run: `melee golden --iso $GW_ISO_VANILLA --out <csv> --scenario cpu_fox_falco_dl [--idle-pads for cpu_*]`
  then `python tools/rust-port/compare_golden.py docs/rust-port/golden/<scn>.csv <csv> --allow-short`.
- Coverage: `tools/rust-port/golden/coverage/run_rust.sh -j 2 --all` (notes/coverage.md).
- Field diff vs the C build: `melee diff --scenario <scn> --frame N` or `--first` (notes/diff.md). Snapshots come from
  `tools/rust-port/snap/dump.sh <scn> <frames>` (needs a C exe in a private build root).
- Replay oracle: `melee slpcheck --slp F` (notes/slpcheck.md).
- Engine debugging: `MELEE_RAND_TRACE=<file>`, `MELEE_PS_TRACE=1`, `MELEE_GUARD_ALLOC[=bytes]`, `MELEE_LEAK_ALLOC=1`; C draws: `tools/rust-port/trace/rand_resolve.py`, `rand_draws.py`.
- Resume mid-match: `melee golden --from-snapshot FILE.snap --start N` (notes/snapshots.md).
- Play: `melee play` (title to results, scripts in `tools/rust-port/play/`; settings overlay: F1, or hold D-pad Down
  and press Start; browser: Z on the main menu; notes/play.md), `melee view-golden --scenario <scn>`.
- Netplay / rollback: `melee synctest`, `melee netplay` (notes/net.md, notes/spectate-replay.md). Record goldens from C: `tools/rust-port/golden/record.sh`.
- Cosmetics: `melee cosmetics import|list|select|portrait|check` (module doc of `rust/melee/src/cosmetics.rs`).

## Consolidation status (re-checked 2026-11-04)
- Steps 1-22 done; 23 (TU-to-TU dedupe) in progress; 24 (lint enforcing, empty baseline) not started.
- Logic lint baseline `tools/rust-port/lint_baseline.txt`: 399 entries (R1 29, R2 287, R3 79, R4 4).
- Gate: `tools/rust-port/gate.sh` (four goldens from `engine_progress.tsv`, diff baseline, lint).

## Recent features (`git log 2a7bdd9c0..5138e0ba5`)
- In-game settings overlay (`rust/melee/src/settings_overlay.rs`): pages VIDEO, AUDIO, OVERLAYS, GAME, CUSTOMIZE;
  writes settings.cfg; rows marked LIVE apply at once (volume, vsync, frame cap, interpolation, overlays, theme,
  skins offline), the rest at next launch (MSAA among them).
- Display (`rust/melee/src/display.rs`): fps cap above 60, render-only frame interpolation / extrapolation,
  renderer-side widescreen; the sim stays at 60 Hz.
- Cosmetics: costume skins, mounted only when the skeleton equals the vanilla costume's (hash-neutral); importers for
  costume .dat / .usd, mod ZIP, Nucleus vault ZIP, portrait / stock PNG (`cosmetics.rs`, `cosmetics_import.rs`,
  melee_gx `tex_override.rs`).
- In-game browser (`rust/mods/browser`, host side `rust/melee/src/browser.rs`): Modes, Mods, Replays, Skins, Online.
- LAB as a wasm mod (`rust/mods/lab`): HUD presets, freeze / step / scrubber, save / load slots, START-hold menu (M3),
  radial quick menu and command bar (M9), online lab commands (notes/lab-ui-spec.md).
- UI fonts: Inter and JetBrains Mono as SDF glyphs (melee_gx `ui_font.rs`).
- Fixes: `ftCommonData_ExtendCpuTables` copied PlCo's f32 table as pointers (results-screen fault, dd40077fc);
  netplay headless audio capture grown in the arena corrupted the game after a rollback (the "50 ms crash",
  63287ac06); `fn_801695BC` walked past an empty queue's terminator (adventure_step02 crash, 5138e0ba5).

# History (dated sections, kept for detail; the orientation above supersedes their numbers)

## Golden status (legacy model, measured 2026-10-30; superseded: the engine runs all four goldens in full)
| scenario | frames matching | first stop / blocker |
|---|---|---|
| pad_falcon_fox_fd | 1801 of 1801 (full) | none |
| pad_marth_sheik_bf | 1801 of 1801 (full) | none |
| cpu_fox_falco_dl | 323 (0..322), no hash divergence (legacy) | legacy only; engine 3601 |
| cpu_peach_samus_ys_items | 120 (0..119), no hash divergence | legacy only; engine 3601 |

## Coverage goldens (C oracle, recorded 2026-10-30; the list grew with the geno / mex merges: results.tsv has 222 rows)
168 scenarios in `docs/rust-port/golden/coverage/scenarios.json`; 158 recorded (each run twice, identical CSVs), 2 failed in the
C build, 8 blocked (no `MELEE_SCENE` keyword). Hash CSVs only, `docs/rust-port/golden/coverage/<name>.csv`. Record with
`tools/rust-port/golden/coverage/record_coverage.sh` (`--batch N`/`--group G`/names; see notes/coverage.md).
- Failed: `stage_26_icetop` (assert `bgm!=BGM_Undefined`, `gr/ground.c:2128`, frame 3), `stage_21_akaneia` (ACCESS_VIOLATION
  at frame 3; mod stage absent from the vanilla disc). Neither is a definition problem.
- Blocked: `homerun_contest`, `multiman_{10,100,3min,15min,endless,cruel}`, `event_match` (retail menus only).
- Report: `python tools/rust-port/golden/coverage_report.py [--md] [--group G]` lists every scenario (golden rows, Rust frames or
  `not wired`). `--run --melee build/cargo/release/melee.exe` runs the wired ones (`melee golden`, can take long) and fills the
  "rust frames matched" column. A scenario is wired when `fn golden` in `rust/melee/src/main.rs` accepts its name.

| group | total | recorded | failed | blocked | golden rows | rust frames matched |
|---|---|---|---|---|---|---|
| chars (26 char_*, cpu/pad one-offs) | 31 | 31 | 0 | 0 | 106831 | pending consolidation |
| stages | 31 | 29 | 2 | 0 | 104429 | pending consolidation |
| items | 10 | 10 | 0 | 0 | 34210 | pending consolidation |
| cpulevels | 17 | 17 | 0 | 0 | 61218 | pending consolidation |
| pads | 9 | 9 | 0 | 0 | 16209 | pending consolidation |
| modes: classic | 11 | 11 | 0 | 0 | 38489 | pending consolidation |
| modes: adventure | 12 | 12 | 0 | 0 | 41561 | pending consolidation |
| modes: allstar | 13 | 13 | 0 | 0 | 44884 | pending consolidation |
| modes: targettest | 26 | 26 | 0 | 0 | 44247 | pending consolidation |
| blocked modes | 8 | 0 | 0 | 8 | 0 | n/a |

### Rust engine vs coverage goldens (2026-11-02, after covfix round 5; superseded by 212 pass / 0 fail on 2026-11-04; detail and per-scenario table in notes/coverage.md)
`melee golden --engine tu` runs the VS coverage scenarios with idle pads (`run_rust.sh`; `MELEE_EXE=` runs a copy of the exe). 12 -> 28 (round 1) -> 34 (rounds 2, 3) -> 51 (round 4) -> **53 of the 79 wired** (round 5). Round 4: the goldens are the C PC build, so its widescreen projection scale (1920x1080 window, `Engine::boot_host`), its `Callback1` ABI mismatch (`HSD_AObjSetRate` reads the JObj pointer as the rate) and its UB `sqrtf` spill (`it_802A4BFC_sqrtf_offset` = `return x`) are reproduced; Unown / Zebes DAT pointers; `HSD_JObjResetRST`. The C call trace is buildable again in minutes (`tools/rust-port/trace/build_trace_reuse.sh`).

| result | scenarios |
|---|---|
| pass | 51 |
| fail | 28 |
| not wired (1p modes / Lua pads) | 79 |
| no golden (C failed or blocked) | 10 |

## Current blockers (coverage, covfix 2026-11-02, round 5)
- Round 5 fixed: Kongo Jungle (Klap Klap hurtbone list rebuilt at full count, `dat_custom` ARTICLE_HURTBONES), Castle (native `castle8.plat[0].state` sat on the `castle11.xD0` slot; now the s16 at gp+CC; `ptr_u32` handle table for pointers above 4 GiB; Mewtwo shadow ball charge is `s32`), Venom crash (counters through the `starfox` view; 1389 -> 2777, now a hash divergence).
- Round 6 (cov2): Rainbow Cruise, char_ness and Fountain pass (native union arm aliases in Kirby SpecialLw / ThrownKirby / CaptureKirby / grIzumi; Kirby hat archives now decode through `dat_custom::kirbyhat`, notes/coverage.md round 6). Left from that cluster: Temple 2477 (Sheik chain item state), Great Bay 1623 (Tingle balloon state), Yoshi's Story 590 and items_hi_yoshistory 2805 (Jigglypuff retail callbacks, blocker D: `ftData_SpecialS[15]` is not set on the engine). Items Stadium (3529): `ftCo_800D3158` death with a garbage `fp->player_id` (192). Corneria (955, 512 owned-gobj cap).
- Item hash divergences (`itemsum`): Mario 455 (Yoshi stars), Mushroom Kingdom II 803, Ice Mt 1472, Great Bay 1623, Temple 2477, Mewtwo 2116, cpulv9_marth_sheik 1798. Fighter hashes at 442 to 3502 (Sheik / Zelda, Kirby, Yoshi, Roy, Bowser, Ness, Zelda, Captain). Method: C call trace + temporary DBG prints (notes/coverage.md round 4); suspect more PC-build ABI/UB differences in the goldens before suspecting the Rust ports.

## Current blockers (slpcheck, 2026-11-02, port/replays)
`melee slpcheck --slp F` compares the engine with the console's own post-frame data (notes/slpcheck.md). 125 external replays: **38 frame-exact**
(was 0), median first divergence of the other 87 = frame 2322 (was 1157). Fixed this round: the match was seeded with 0 (`Replay_ApplyMatch` seam), the
synthetic Frame Start seeds were forced every frame (the console RNG is continuous; Pre-Frame seeds chain by LCG steps), the BGM-pick draw on stages 3/31/32,
per-fighter reseed from the Pre-Frame seed. RNG draw counts now equal the console's up to every file's first divergence. Remaining classes:
- S1/S3 (Pokemon Stadium `y`, 16 files): falling fighters land on a y=25.0001 platform line the console fighter falls through. Fountain platform `y` (8 files).
- 1-ULP `x`/`y` (about 25 files): float contraction sites (MWCC vs clang) in ported functions.
- Hit timing (`percent`/`hitlag`/`shield`, about 20): not separated from the ULP drift yet.
Leads 2 (raw sticks + Gecko-list UCF) and 3 (Whispy / Stadium codes): UCF 0.84 and the raw-stick pad queue were already in; the tournament Gecko lists carry no
WhispyBlowDirFix, and the Stadium preload codes are the S1/S3 suspects (notes/slpcheck.md round 2). Lead 4 (spawn 1-ULP baseline) not needed: the 38 exact files match from frame -123.

## Current blockers (coverage cluster A, port/cov1, 2026-11-02)
- char_mario fixed (Yoshi attr offsets). Open: char_zelda 2446 and stage_19 2662 (a mover's `cur_pos.x` 3.0 off at frame 2443 before `mpColl_80043754`),
  cpulv5_puff_link 673 (CPU AI output differs from identical state), stage_20 803 / char_kirby 755 / sheik_cpu_vs_zelda_cpu 442 (JObj MTX_DIRTY / `lbCopyJObjSRT` count),
  cpulv9_marth_sheik 1798, stage_25_icemt 1472, char_bowser 791 (not analysed past triage). Details in notes/coverage.md "Cluster A".
## Current blockers (coverage, cov3 round 6, 2026-11-02; details in notes/coverage.md "cov3 round")
- Cluster C result: 12 scenarios now pass (items_hi_stadium, char_mewtwo, cpulv9_marth_sheik, stage_14_temple, cpu_ffa4_levels, char_yoshi, char_younglink, char_captain, stage_22_venom, plus bowser / mario / ness / roy / zelda / fountain / sheik_cpu / mushroomkingdom as side effects). Dominant cause: **union members that alias at one GC offset do not alias natively** (pointer widening), so a write through one name and a read through another sees stale data; plus attribute structs that start with a pointer (`ftYs_DatAttrs`, `itKyasarinAttributes`) read one word late from the word-swapped block. Scanner idea: `rust/melee_types` GcLayout offsets vs native offsets (see notes/coverage.md).
- stage_07_corneria (955): the far Arwing's joint 4 / 1 translations differ from C from its first frame (`Ground_801C3FA4(arwing, 4)` translate.z -31.9 in C, 0 in Rust; joint 1 animates along z=4752 in C, 634 in Rust), so the Arwing laser decision at 955 differs. Check the Arwing stage gobj's joint tree / animation set (`grCorneria_801DD534(1)` + `grAnime_801C8098`) and `arwing_types`-selected anim.
- stage_13_greatbay (1623): Tincle (kind 221) motion 2 -> 3 in C at the frame where `it_8026DAA8` (item collision) reports a hit; Rust sees none. Item position matches until then.
- stage_20_mushroomkingdom2 (1254, then a crash at 1455 `read of 0xFFFFFFFFFFFFFFFF`): after the Kyasarin attribute fix an RNG draw count differs (Kyasarin `itKyasarin_Randi` users) and later a bad pointer.
- stage_25_icemt (1953): the block scroll is exact now (`table[0]` fix) but C wraps the blocks (`grIceMt_801F9ACC`, `cur < -y0` at -277.1) where Rust never reaches that `cur` (the scroll stalls at -131.8: speed index picks differ, see `fn_801F8E58` / `grIceMt_GetRandomIndex`).
- Not in cluster C (other agents): char_kirby (755), cpulv5_puff_link (673), items_hi_yoshistory (2805), stage_08_yoshistory (448), stage_11_rainbowcruise (177).

## Current blockers (slpcheck, 2026-11-02, port/slporacle; superseded by round 2: 38 of 125 exact)
`melee slpcheck --slp F` compares the engine with the console's own post-frame data (notes/slpcheck.md). 125 of 125 external replays run, none matches yet (median first divergence frame 1157). Blockers, none solved inside the timebox:
- S1. DamageFlyRoll (91) vs DamageFlyHi/N/Lw choice at a hit (39 files): RNG draw count between Frame Start and the damage code.
- S2. Hit-connection timing (hitlag / percent / last attack / shield push, ~30 files).
- S3. Pokemon Stadium / Fountain platform `y` (16 files) and Dream Land wind (16 files): stage state at start.
Fixed on the way: VS-mode `Replay_Active` (Slippi Neutral Spawn), rumble motor hook, smash-charge shake table in the engine.

**Leads for S1-S3: try these FIRST** (from melee-unlocked's frame-exact replay path, notes/unlocked-survey.md rows "Per-frame replay inputs and RNG", "UCF and the other gameplay codes", "Denylist and code list", "Replay compare gate"):
1. Re-seed the RNG from the replay's pre-frame seed EVERY frame (likely S1: the tumble choice is a random draw).
2. Decide UCF from the gameplay injections in the replay's own Gecko list, not from the per-player toggle words. Feed the RAW stick bytes from the pre-frame update into the pad queue (HSD_PadLibData queue) so UCF sees raw input, not just the processed values (likely S2).
3. Slippi gameplay codes the replay enables: WhispyFix (Dream Land wind, S3), the 4 Pokemon Stadium codes, InitStage/InitPlayerData, NeutralSpawn, FreezeGlitch, BrawlOffscreenDamage, DeadUpFall, FD BG seed, Nana determinism, PreventWobbling. Gate each on the replay's Gecko list, using Slippi's InjectionLists "[affects-gameplay]" tags.
4. Allow melee-unlocked's known baseline: one-ULP `y` differences at frames -118..-108 (spawn animation). Require exactness from frame 0.

Licensing (user decision, 2026-11-02): implement the codes from Slippi's own public sources (slippi-ssbm-asm, GPL-3.0; InjectionLists), NOT from melee-unlocked's mu_gecko.c, mu_replay.c or mu_online.c. Taking GPL-3 code makes the combined work GPL-3, which the user accepts. Credit slippi-ssbm-asm in each file.

## Current blockers (golden, 2026-10-30 to 11-01; items 2 and 4 superseded: all four goldens pass on the engine)
1. Fighter-vs-fighter hit detection corners (shields, clanks, smash hitboxes via script) and air damage states.
2. Shy Guys (`itheiho.c`, item-stage collision `it_8026DA70`) and the random item spawner (`fn_8026C88C`).
3. (2026-10-30, outdated) `melee play` covered 7 fighters / 4 stages with stand-in `mn*` scenes; it now runs the retail menus (notes/play.md, notes/menus.md).
4. Engine (default since step 18; `--engine legacy` = the model): both pad scenarios reach 1801 frames with hash + seed identical to the golden (steps 11-13); the seed needs `tools/rust-port/engine/pre_draws.tsv` dummy draws (1) until the boot flow draws for real. cpu_fox_falco_dl 3601 (the whole golden), cpu_peach_samus_ys_items 1652 (blocker C); sound is a no-op seam; `melee diff --engine tu` works for fighters, items and dynamic bones, engine snapshots / rollback are step 17 (notes/consolidation.md). The engine now runs the draw callbacks headless (notes/consolidation.md, round 1).
5. User-only checks: visuals, audio by ear, real controller, online vs another machine (notes/platform.md).

## Writing notes (merge-friendly: other branches append concurrently)
- Put detail in the matching `docs/rust-port/notes/<topic>.md` (append a dated section; do not touch STATUS.md unless a
  decision, command, status-table row or blocker changed).
- Add ONE line to the changelog in `RUST_PORT.md` (`- 2026-10-30 branch: what, see notes/<topic>.md`). Never paste long text there.

## Topic notes
- notes/toolchain.md: oracle, phases, c2rust/translation, workspace layout, inventory (old).
- notes/golden.md: golden corpus, record/compare, every golden round and its traps.
- notes/diff.md: `melee diff` usage, baseline, proofs.   notes/snapshots.md: state dump/resume.
- notes/fma.md: float contraction.   notes/types.md: melee_types generator.   notes/groups.md: fighter-group notes.
- notes/oracles-mtu.md: whole-TU ports with state oracles (items, world, modes), efn.   notes/traces.md: pre-melee_* traces.
- notes/data.md: DAT reader, mex-import, saves/mods/replays.   notes/integration.md: integration nights, recounts, merges.
- notes/render.md, notes/audio.md, notes/net.md, notes/script.md, notes/play.md, notes/platform.md, notes/menus.md.
- notes/consolidation.md: plan to run matches on the real loop + tu/ dedupe.   notes/slpcheck.md: `melee slpcheck`, a .slp replay as a direct oracle (results over 125 tournament replays).
- notes/settings, cosmetics, browser: module docs plus notes/play.md.   notes/lab-ui-spec.md, notes/wasm-ui-lab-api.md: LAB / UI mods.   notes/launcher-seam.md, notes/bench-unlocked.md: Melee Unlocked launcher and benchmark.   notes/geno.md, notes/mex.md, notes/modhost.md, notes/display.md, notes/controllers.md, notes/spectate-replay.md, notes/gameplay-codes.md.
- notes/traps.md: index of traps.   Other: docs/rust-port/inventory.md, neutral-hash.md, group-*.md, tools/rust-port/snap/README.md.


## Current blockers (engine, 2026-11-01, round 4)
Round 4 done: steps 17 (engine snapshot / rollback: `melee synctest`), 18 (cutover, GX bridge draws the real frame), 19 (the `vs_match` model deleted, engine audio equal to the C capture).
Round 5: steps 20-22 done as a mechanical fold (notes/consolidation.md Round 5; tools/rust-port/dedupe/): the C-named bodies of chars/rN, the non-tu melee_fighter / world / items / modes / runtime modules live in their canonical tu files, logic lint baseline 9225 -> 4439 (R1 265 left).
Next: step 23 (TU-to-TU local copies, `gen_link.py --convert`), the typed `_fp` / `*_ref` variants and `adapt.rs` / `stubs.rs`, 24 (lint enforcing). Known stale: `tests/chars_dispatch.rs` names (`_fp`) vs `tests/data/chars.txt`, so `chars_ref` and `oracle_ftyoshispecials` fail at HEAD and now.
Open audio: `--pause` and `--time-secs` captures (notes/consolidation.md Round 4).
Engine (`--engine tu`) frame counts: pad_falcon_fox_fd 1801, pad_marth_sheik_bf 1801, cpu_fox_falco_dl 3601, cpu_peach_samus_ys_items 3601 (all full goldens). Lint baseline 403 (round 10).
Blockers A and B of round 2 (stale hand matrices) are fixed: the stage draw callbacks were skipped (`Wb_RenderOption` answered 0), `HSD_MkRotationMtx`
was unlinked, and several GC-layout reads went through native structs; details in notes/consolidation.md "Round 3".
- C (SOLVED in round 8: cpu_peach_samus_ys_items now matches all 3601 rows, seed included; the text below is the history). Was 1562, then 1652. The old blocker C was three real bugs: `Fighter_804D6550` (PlCo root 1, the item-throw float table) was a pointer table and its hook returned the
  variable's address (`melee_link::manual` now returns the table, `dat_custom::plco` decodes native words); item state-table predicates were `bool`-returning fns transmuted to
  `int`-returning ones (`kinds_inl::pred_b!` now makes an int-returning shim: a Rust `bool` leaves garbage in the upper bits of eax, which made items vanish at random); the full
  `Item` record is now compared by `melee diff --engine tu` (`ItemRecord`). Next stop: frame 1652, P2 Samus. seqdiff (inlined C names ignored: /tmp-style list in notes) is identical up to
  `ftco_attack1_checkinput`; the C then presses A (`cpu.buttons` 0x100, `lstick_x` 0.567, motion 92) and the engine does not. State at 1651 is equal (diff: only audio-voice garbage),
  the seed after 1652 is equal, but `cpu.buffer` (the CPU command script, `ftcmdscript` writer) holds different command bytes after the frame (C `86 01 87 01 7f 7f 02 7f 7f 00 09 86`,
  engine `.. 0a 02 8e 01 09 86 01 0a 02 7f 7f 86`) and `cpu.x55C..x568` floats differ: the AI built a different action with the same draws. Suspects: the PlCo attack-entry tables
  for Samus (`dat_custom::attack_lists`, 9-word entries) or a float step of the command builder. Not yet instrumented per value.
- D. (SOLVED, port/blockerd 2026-11-02) Retail tables with unresolved callbacks: Jigglypuff Rollout/Pound, Crazy Hand and the Crazy Hand captures are ported
  and wired (`tools/rust-port/link/wire_unresolved.py`; `melee_link/retail_unresolved.txt` is empty). items_hi_yoshistory and cpulv5_puff_link now pass. See notes/consolidation.md "Blocker D".
- E. (2026-11-01; 14ae6cb5c later wired many `todo!()` hook defaults, not re-audited here) Direct `todo!` stubs left in melee_fighter tu: `ftCo_Damage` (lbBgFlash_80021C48, efAsync_Spawn, Camera_RequestQuake), `JumpAerial` (HSD_JObjSetMtxDirtySub), `ft_0c8c`
  (lbBgFlash_80021C48, gm_80167470, ftMh / ftCh), `ft_0cdd` done, `ft_0cdf` done. They panic when reached (outside the render pass).
- F. Peach's dress dynamics (`Dynamics P1 bone set 0`) run on the engine but are inactive in the C (bone id 0x100): not hashed, still a diff in `melee diff`.
- Method (works, keep using it): `melee diff --engine tu --frame N` (captures at Script_FramePost like the C snapshot; `MELEE_DIFF_MTX=1` adds world matrices;
  JObj2 lines = the blend-source tree), `tools/rust-port/trace/countdiff.py` / `seqdiff.py --anchor Fighter_8006A1BC --auto-ignore N`, the C trace build with a
  throw-away `dbg_jobj_out` logger, and `MELEE_CT_LIVE_FROM` to find the function a segfault is in. Do not hunt single fields first.
- c2rs trap: a declaration whose initialiser comes from a macro (`PUSH_ATTRS`, `COPY_ATTRS`) is emitted with a null initialiser. Check generated fighter inits.
