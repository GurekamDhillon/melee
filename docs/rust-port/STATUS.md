# Rust port: status (read this first; 2026-10-30)

Native Rust port of GD's Melee (branch family `rust-port`, workspace in `rust/`). Everything below is current
truth; history and per-topic detail are in `docs/rust-port/notes/` (index at the bottom).

## Decisions and hard rules
- Native little-endian Rust, no emulation. Game logic is bit-exact against the C build, which is compiled with
  clang's shipping float contraction (`-ffp-contract=on`): fused sites are ported as `fmadds`/`fmsubs` (f64 then f32),
  never `f32::mul_add`. Oracles are built the same way. Detail: notes/fma.md.
- GameCube offset metadata is kept for every record (`GcLayout` / `GcDecode`, `melee_types`), so C snapshots, DAT
  files and the C oracles read through the same tables. `melee_types` is the ONE generator (rust/melee_types/gen);
  never hand-edit `gen.rs`.
- Rust replaces the C build: no C-netplay wire compatibility is kept (the C-vs-Rust-over-the-wire test only proved
  the protocol port, notes/net.md). The port's own "kit" menus (`gmfrontend`) are not ported (notes/menus.md).
- NEVER send OS-level input (SendKeys/SendInput/AppActivate/SetForegroundWindow/pyautogui/computer-use). Input goes
  through `MELEE_PAD_SCRIPT`, the Rust `PadSystem` script source or in-process hooks; runs are headless, C runs
  off-screen (`MELEE_WINDOW_X=30000`, `MELEE_TURBO=1`).
- NEVER commit disc-derived data (snapshots, DATs, renders, audio). Disc path: `GW_ISO_VANILLA` in the workspace
  `.env` (`set -a; . .env; set +a`); never print it. Goldens are hash CSVs only.
- The golden hash (NHV1, notes/golden.md, docs/rust-port/neutral-hash.md) is the definition of "done" for sim code.
- C builds use a private build root `_build/agents/<name>` (pc/docs/PORT_DEV_QUICKREF.md, workspace `tools/port`).
- Cargo: prefix `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`; test only crates you change.

## Crate map (rust/, dependency order)
| crate | holds |
|---|---|
| melee_core | fp (`fmadds`), gc layout/decode traits, StateHash |
| melee_types | every C struct/enum + GC offset tables (generated), neutral hash |
| melee_runtime | MSL math, HSD (JObj/AObj/particles/RNG), lb, db |
| melee_world | mp (collision), cm, ef, gr (stages), if |
| melee_fighter | ft engine, ftCommon, pl |
| melee_chara | per-character ft code |
| melee_items | it (items), ty, vi |
| melee_modes | gm, mn, port game side |
| melee_platform / melee_gx / melee_audio | window+pads, GX on wgpu, AX mixer/synth |
| melee_net / melee_data / melee_script | rollback+UDP, saves/settings/mods/replays, Lua `gd.*` |
| melee (bin) | `vs_match` (the native match), golden/diff/play/view-golden/netplay, disc, UI stand-ins |
| tools/mex-import | m-ex mod packer |

## Commands
Env: `export GW_ISO_VANILLA=...` via the workspace `.env`. Run from the repo root; `cd rust` for cargo.
- (Step 19: the `vs_match` model is gone; every `melee` command runs the engine.)
- Build: `cd rust && cargo build -p melee --release` (exe: `build/cargo/release/melee.exe`). Tests: `cargo test -p <crate>`.
  Fast loop: `bash tools/rust-port/cargo_fast.sh build -p melee --profile fast` (exe `build/cargo/fast/melee.exe`; sccache for deps + incremental; a fighter-file edit 169 -> 48 s) and `PROFILE=fast bash tools/rust-port/gate.sh` (same goldens, GATE OK); release stays the default (notes/fastbuild.md).
- Golden run: `melee golden --iso $GW_ISO_VANILLA --out /tmp/c.csv --scenario cpu_fox_falco_dl [--idle-pads for cpu_*]`
  then `python tools/rust-port/compare_golden.py docs/rust-port/golden/<scn>.csv /tmp/c.csv --allow-short`.
  (CPU scenarios need idle pads; `melee diff` sets that itself.)
- Field diff vs the C build: `melee diff --scenario <scn> --frame N` or `--first` (notes/diff.md). Snapshots come from
  `tools/rust-port/snap/dump.sh <scn> <frames>` (needs a C exe in a private build root; README there).
- Engine debugging: `MELEE_RAND_TRACE=<file>` (each draw with callers), `MELEE_PS_TRACE=1`, `MELEE_GUARD_ALLOC[=bytes]`, `MELEE_LEAK_ALLOC=1`; C draws: `tools/rust-port/trace/rand_resolve.py`, `rand_draws.py`.
- Resume mid-match: `melee golden --from-snapshot FILE.snap --start N` (notes/snapshots.md).
- Viewers: `melee view-golden --scenario pad_falcon_fox_fd` (sim drawn through melee_gx), `melee play` (title to results,
  scripts `tools/rust-port/play/`), `cargo run -p melee_platform --bin gx_viewer -- --stage GrNLa.dat`.
- Netplay/rollback: `melee synctest`, `melee netplay` (notes/net.md). Record goldens from C: `tools/rust-port/golden/record.sh`.

## Golden status (measured 2026-10-30, frames matching from 0)
| scenario | frames matching | first stop / blocker |
|---|---|---|
| pad_falcon_fox_fd | 1801 of 1801 (full) | none |
| pad_marth_sheik_bf | 1801 of 1801 (full) | none |
| cpu_fox_falco_dl | 323 (0..322), no hash divergence (legacy) | legacy only; engine 3601 |
| cpu_peach_samus_ys_items | 120 (0..119), no hash divergence | stops at 120: Shy Guys spawn `it_802D8618` (`itheiho.c`) |
(Engine, default since step 18: all four goldens are complete on `--engine tu`, seed column included: 1801 / 1801 / 3601 / 3601; the rows above are the pre-engine legacy numbers.)
The sim stops (reports why) on unported code rather than guessing; a stop is not a mismatch. Re-measure with
`melee diff --scenario <scn> --first` (it prints the stop reason).

## Consolidation status (2026-11-01)
- Steps 1-22 DONE; 23 (TU-to-TU dedupe) in progress; 24 (lint enforcing, empty baseline) not started.
- Engine (`--engine tu`, the only engine since step 19) frame counts: pad_falcon_fox_fd 1801, pad_marth_sheik_bf 1801, cpu_fox_falco_dl 3601, cpu_peach_samus_ys_items 3601 (all full goldens, seed column included).
- Baselines: logic lint `tools/rust-port/lint_baseline.txt` 398 entries (R1 29, R2 286, R3 79, R4 4; `lint_allow.txt` 13); diff baseline 144 entries.
- `port/dedupe` merged into rust-port (c333d57c9); one lint remains (`lint_logic.py`). Gate: `tools/rust-port/gate.sh` GATE OK.
- Remaining: finish 23 (pointer-based render model for retail menu stand-ins, 109 type-mismatched forwarders in `dedupe/fwd_skip.txt`, melee_audio sim vs lbaudio, melee_modes gm* typed copies), then 24; open audio proofs (`--pause`, `--time-secs`); blocker D (Crazy Hand / Jigglypuff retail callbacks) and E (`todo!` stubs).

## Coverage goldens (C oracle, recorded 2026-10-30)
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

### Rust engine vs coverage goldens (2026-11-02, after covfix round 5; detail and per-scenario table in notes/coverage.md)
`melee golden --engine tu` runs the VS coverage scenarios with idle pads (`run_rust.sh`; `MELEE_EXE=` runs a copy of the exe). 12 -> 28 (round 1) -> 34 (rounds 2, 3) -> 51 (round 4) -> **53 of the 79 wired** (round 5). Round 4: the goldens are the C PC build, so its widescreen projection scale (1920x1080 window, `Engine::boot_host`), its `Callback1` ABI mismatch (`HSD_AObjSetRate` reads the JObj pointer as the rate) and its UB `sqrtf` spill (`it_802A4BFC_sqrtf_offset` = `return x`) are reproduced; Unown / Zebes DAT pointers; `HSD_JObjResetRST`. The C call trace is buildable again in minutes (`tools/rust-port/trace/build_trace_reuse.sh`).

| result | scenarios |
|---|---|
| pass | 51 |
| fail | 28 |
| not wired (1p modes / Lua pads) | 79 |
| no golden (C failed or blocked) | 10 |

## Current blockers (coverage, covfix 2026-11-02, round 5)
- Round 5 fixed: Kongo Jungle (Klap Klap hurtbone list rebuilt at full count, `dat_custom` ARTICLE_HURTBONES), Castle (native `castle8.plat[0].state` sat on the `castle11.xD0` slot; now the s16 at gp+CC; `ptr_u32` handle table for pointers above 4 GiB; Mewtwo shadow ball charge is `s32`), Venom crash (counters through the `starfox` view; 1389 -> 2777, now a hash divergence).
- Yoshi's Story (448, Kirby hat load): the Yoshi hat joint tree walks 50 joints but Kirby's `parts` has 46 `flags_b1` bones (parts_num 59), so `ftKb_SpecialN_insert_joint_refs` runs off `fp->parts` and the envelope lookup fails. The tree structure itself decodes consistently; find why C has 50 flagged bones (or the hat has 46 joints). Items Stadium (3529): `ftCo_800D3158` death with a garbage `fp->player_id` (192). Rainbow Cruise (177), Corneria (955, 512 owned-gobj cap), Fountain (3275, seed only).
- Item hash divergences (`itemsum`): Mario 455 (Yoshi stars), Mushroom Kingdom II 803, Ice Mt 1472, Great Bay 1623, Temple 2477, Mewtwo 2116, cpulv9_marth_sheik 1798. Fighter hashes at 442 to 3502 (Sheik / Zelda, Kirby, Yoshi, Roy, Bowser, Ness, Zelda, Captain). Method: C call trace + temporary DBG prints (notes/coverage.md round 4); suspect more PC-build ABI/UB differences in the goldens before suspecting the Rust ports.

## Current blockers (coverage, cov3 round 6, 2026-11-02; details in notes/coverage.md "cov3 round")
- Cluster C result: 12 scenarios now pass (items_hi_stadium, char_mewtwo, cpulv9_marth_sheik, stage_14_temple, cpu_ffa4_levels, char_yoshi, char_younglink, char_captain, stage_22_venom, plus bowser / mario / ness / roy / zelda / fountain / sheik_cpu / mushroomkingdom as side effects). Dominant cause: **union members that alias at one GC offset do not alias natively** (pointer widening), so a write through one name and a read through another sees stale data; plus attribute structs that start with a pointer (`ftYs_DatAttrs`, `itKyasarinAttributes`) read one word late from the word-swapped block. Scanner idea: `rust/melee_types` GcLayout offsets vs native offsets (see notes/coverage.md).
- stage_07_corneria (955): the far Arwing's joint 4 / 1 translations differ from C from its first frame (`Ground_801C3FA4(arwing, 4)` translate.z -31.9 in C, 0 in Rust; joint 1 animates along z=4752 in C, 634 in Rust), so the Arwing laser decision at 955 differs. Check the Arwing stage gobj's joint tree / animation set (`grCorneria_801DD534(1)` + `grAnime_801C8098`) and `arwing_types`-selected anim.
- stage_13_greatbay (1623): Tincle (kind 221) motion 2 -> 3 in C at the frame where `it_8026DAA8` (item collision) reports a hit; Rust sees none. Item position matches until then.
- stage_20_mushroomkingdom2 (1254, then a crash at 1455 `read of 0xFFFFFFFFFFFFFFFF`): after the Kyasarin attribute fix an RNG draw count differs (Kyasarin `itKyasarin_Randi` users) and later a bad pointer.
- stage_25_icemt (1953): the block scroll is exact now (`table[0]` fix) but C wraps the blocks (`grIceMt_801F9ACC`, `cur < -y0` at -277.1) where Rust never reaches that `cur` (the scroll stalls at -131.8: speed index picks differ, see `fn_801F8E58` / `grIceMt_GetRandomIndex`).
- Not in cluster C (other agents): char_kirby (755), cpulv5_puff_link (673), items_hi_yoshistory (2805), stage_08_yoshistory (448), stage_11_rainbowcruise (177).

## Current blockers (slpcheck, 2026-11-02, port/slporacle)
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

## Current blockers (golden)
1. Fighter-vs-fighter hit detection corners (shields, clanks, smash hitboxes via script) and air damage states.
2. Shy Guys (`itheiho.c`, item-stage collision `it_8026DA70`) and the random item spawner (`fn_8026C88C`).
3. `melee play` covers 7 fighters / 4 stages; retail `mn*` scenes are stand-ins (notes/play.md).
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
- D. Retail tables with unresolved callbacks (no callback at all on the engine): Crazy Hand `ftCh_*` (40), Jigglypuff `ftPr_*` (80), Crazy Hand captures (12)
  (`melee_link/retail_unresolved.txt`). Not in a golden; any scenario with these fighters will freeze them.
- E. Direct `todo!` stubs left in melee_fighter tu: `ftCo_Damage` (lbBgFlash_80021C48, efAsync_Spawn, Camera_RequestQuake), `JumpAerial` (HSD_JObjSetMtxDirtySub), `ft_0c8c`
  (lbBgFlash_80021C48, gm_80167470, ftMh / ftCh), `ft_0cdd` done, `ft_0cdf` done. They panic when reached (outside the render pass).
- F. Peach's dress dynamics (`Dynamics P1 bone set 0`) run on the engine but are inactive in the C (bone id 0x100): not hashed, still a diff in `melee diff`.
- Method (works, keep using it): `melee diff --engine tu --frame N` (captures at Script_FramePost like the C snapshot; `MELEE_DIFF_MTX=1` adds world matrices;
  JObj2 lines = the blend-source tree), `tools/rust-port/trace/countdiff.py` / `seqdiff.py --anchor Fighter_8006A1BC --auto-ignore N`, the C trace build with a
  throw-away `dbg_jobj_out` logger, and `MELEE_CT_LIVE_FROM` to find the function a segfault is in. Do not hunt single fields first.
- c2rs trap: a declaration whose initialiser comes from a macro (`PUSH_ATTRS`, `COPY_ATTRS`) is emitted with a null initialiser. Check generated fighter inits.
