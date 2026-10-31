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
The sim stops (reports why) on unported code rather than guessing; a stop is not a mismatch. Re-measure with
`melee diff --scenario <scn> --first` (it prints the stop reason).

## Current blockers (golden)
1. Fighter-vs-fighter hit detection corners (shields, clanks, smash hitboxes via script) and air damage states.
2. Shy Guys (`itheiho.c`, item-stage collision `it_8026DA70`) and the random item spawner (`fn_8026C88C`).
3. `melee play` covers 7 fighters / 4 stages; retail `mn*` scenes are stand-ins (notes/play.md).
4. Engine (`--engine tu`): both pad scenarios reach 1801 frames with hash + seed identical to the golden (steps 11-13); the seed needs `tools/rust-port/engine/pre_draws.tsv` dummy draws (1) until the boot flow draws for real. cpu_fox_falco_dl reaches 1197 (blocker A), cpu_peach_samus_ys_items 120 (the golden's end; blocker B); sound is a no-op seam; `melee diff --engine tu` works for fighters, items and dynamic bones, engine snapshots / rollback are step 17 (notes/consolidation.md). The engine now runs the draw callbacks headless (notes/consolidation.md, round 1).
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


## Current blockers (engine, 2026-10-31, round 2)
- A. cpu_fox_falco_dl on `--engine tu`: 1197 frames match (was 950). Fixed this round: (1) `lb_8001044C` read the body colliders as the
  GC `lb_Collider` layout instead of the native `Fighter_x1670_t` (position at +0x1C, found by storing the C's chain inputs in a game
  global and reading the snapshot); (2) `ftCo_800A9CB4` lost its `__frsqrte` seed in the c2rs port (all four `sqrtf_store` sites).
  Next stop: frame 1196 (Fox is hit by Falco, motion 76). The hurt capsules 8 and 10 (the hands: `ftFx_Init_OnKnockbackEnter` part
  animations `ftAnim_ApplyPartAnim(gobj, 3 / 4, 3, 0)`) get bone matrices that differ from the C's (JObj SRT equal), so `ftCo_800A0DA4`
  derives a different `cpu.x560/x564`, the AI range `cpu.x38` differs a frame later and the stick/hash diverge at 1197. Suspect the
  hand joints' mtx dirty/cached state (`HSD_JObjSetMtxDirty` early-out, `lb_8000B4FC` / `ftAnim_8006FA58`, part-anim `flags_b5`) or the
  render pass (see lead below). Not yet checked: JObj flags of the hand joints in the snapshot vs the engine (C 0x49, engine 0x09 on
  all joints at the snapshot point, so the snapshot point differs).
  Then: segfault at frame ~1207 in `efSync_Spawn` from `it_80270E30` (`efLib_Create_Attach_Pos` gets a bad jobj).
- B. cpu_peach_samus_ys_items on `--engine tu`: 559 frames match (the legacy path stops at 120). Done this round: Samus in tu/ (c2rs plus the
  oracle-checked chars bodies re-exported), the sound-id lookups run for real, `Mex_GrBind` identity, the `tu_ext` item/camera hooks wired to the
  engine's TUs (their defaults were the legacy-state ports), stage-item articles word-swapped like the common ones, the Foods attribute rows
  in GC layout with the joint in `itfoods::FOODS_JOINTS`, `ItCollDynamics` rebuilt from the image, `db_ItemSpawnsEnabled` set at boot
  (the random item spawner never ran), and `EfArg` variadic words in `melee_link::va_words` (the `efSync_Spawn` crash at ~1207 on
  cpu_fox_falco_dl and ~308 here). Next stop: frame 559, P1 (Peach) ECB `coll_data.desired_ecb` differs (the same class as A: a bone matrix).
  Then a segfault at ~959. Peach's dress bone sets (`Dynamics P1 bone set 0`) are inactive in the C (bone_id 0x100) but run in the engine: not hashed.
- Both A and B stop on the same thing: a joint's CACHED world matrix (`jobj->mtx`, clean flag, rotation part) differs between the C and the
  engine at the time the hurt capsule / ECB reads it (`lb_8000B1CC`; translation, flags and the joint's own SRT are equal). Measured for
  Fox's hands at frame 1196 by logging every `lb_8000B1CC` call into a C game global (last 256 calls) and the same values from the engine: 252 of
  256 calls identical, the 3 differing ones are the hand joint. So who last wrote that joint's matrix (the render pass of an earlier frame, the
  shadow pass, a hidden-joint skip) is what differs; next step is to log `HSD_JObjSetupMatrixSub` events for that joint on both sides.
- c2rs trap: a declaration whose initialiser comes from a macro (`PUSH_ATTRS`, `COPY_ATTRS`) is emitted with a null initialiser. Check
  generated fighter inits (fixed by hand in `tu/ftsamus.rs`).
- A (lead). The dynamic-bone sim and capsules read world matrices (`jobj->mtx`) that the C render pass leaves fresh; the engine's render
  pass is incomplete: per frame the C build runs `HSD_JObjDispSub` 122x, `HSD_DObjDisp` 374x and `HSD_StartRender` 3x (the `lbShadow` /
  `ifMagnify` offscreen passes are seams that do nothing, `seams_gen.rs`), the engine 24x / 0x / 1x. Make the offscreen passes real
  (at least their matrix-setup effect) and re-check. `MELEE_DIFF_MTX=1 melee diff --engine tu` compares world matrices too.
- Method that worked: store the function's intermediate values in a C game global (`gw_*` array in the TU, rebuild one TU in
  `_build/agents/spine-r1-trace`, `tools/rust-port/snap/dump.sh`) and read it from the snapshot with a small python reader of the
  MELSNAP1 format; the log (`OSReport`) drops lines.
