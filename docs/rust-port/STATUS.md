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
- Build: `cd rust && cargo build -p melee --release` (exe: `build/cargo/release/melee.exe`). Tests: `cargo test -p <crate>`.
- Golden run: `melee golden --iso $GW_ISO_VANILLA --out /tmp/c.csv --scenario cpu_fox_falco_dl [--idle-pads for cpu_*]`
  then `python tools/rust-port/compare_golden.py docs/rust-port/golden/<scn>.csv /tmp/c.csv --allow-short`.
  (CPU scenarios need idle pads; `melee diff` sets that itself.)
- Field diff vs the C build: `melee diff --scenario <scn> --frame N` or `--first` (notes/diff.md). Snapshots come from
  `tools/rust-port/snap/dump.sh <scn> <frames>` (needs a C exe in a private build root; README there).
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
The sim stops (reports why) on unported code rather than guessing; a stop is not a mismatch. Re-measure with
`melee diff --scenario <scn> --first` (it prints the stop reason).

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

## Current blockers (golden)
1. Fighter-vs-fighter hit detection corners (shields, clanks, smash hitboxes via script) and air damage states.
2. Shy Guys (`itheiho.c`, item-stage collision `it_8026DA70`) and the random item spawner (`fn_8026C88C`).
3. `melee play` covers 7 fighters / 4 stages; retail `mn*` scenes are stand-ins (notes/play.md).
4. User-only checks: visuals, audio by ear, real controller, online vs another machine (notes/platform.md).

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
