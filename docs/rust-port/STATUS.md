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
| cpu_fox_falco_dl | 323 (0..322), no hash divergence | stops at 323: P2 shield roll `ftCo_800992A8` not ported |
| cpu_peach_samus_ys_items | 120 (0..119), no hash divergence | stops at 120: Shy Guys spawn `it_802D8618` (`itheiho.c`) |
(Engine, default since step 18: all four goldens are complete on `--engine tu`, seed column included: 1801 / 1801 / 3601 / 3601; the rows above are the pre-engine legacy numbers.)
The sim stops (reports why) on unported code rather than guessing; a stop is not a mismatch. Re-measure with
`melee diff --scenario <scn> --first` (it prints the stop reason).

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
- notes/consolidation.md: plan to run matches on the real loop + tu/ dedupe.
- notes/traps.md: index of traps.   Other: docs/rust-port/inventory.md, neutral-hash.md, group-*.md, tools/rust-port/snap/README.md.


## Current blockers (engine, 2026-11-01, round 4)
Round 4 done: steps 17 (engine snapshot / rollback: `melee synctest`), 18 (cutover, GX bridge draws the real frame), 19 (the `vs_match` model deleted, engine audio equal to the C capture).
Round 5: steps 20-22 done as a mechanical fold (notes/consolidation.md Round 5; tools/rust-port/dedupe/): the C-named bodies of chars/rN, the non-tu melee_fighter / world / items / modes / runtime modules live in their canonical tu files, logic lint baseline 9225 -> 4439 (R1 265 left).
Next: step 23 (TU-to-TU local copies, `gen_link.py --convert`), the typed `_fp` / `*_ref` variants and `adapt.rs` / `stubs.rs`, 24 (lint enforcing). Known stale: `tests/chars_dispatch.rs` names (`_fp`) vs `tests/data/chars.txt`, so `chars_ref` and `oracle_ftyoshispecials` fail at HEAD and now.
Open audio: `--pause` and `--time-secs` captures (notes/consolidation.md Round 4).
Engine (`--engine tu`) frame counts: pad_falcon_fox_fd 1801, pad_marth_sheik_bf 1801, cpu_fox_falco_dl 3601 (full golden), cpu_peach_samus_ys_items 1652.
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
