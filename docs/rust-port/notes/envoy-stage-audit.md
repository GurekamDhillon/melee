# Envoy extra-stage audit (port/stageaudit, 2026-11-09)

Question 6 of `full-game-plan.md` (on pub/port/fullplan): can Kongo Jungle 64, Pokemon Stadium (transformations frozen),
Yoshi's Island 64 and the All-Star rest area join the five audited shift stages (BF, YS, DL, FoD, FD) in an Envoy run?
Base: pub/port/envoydoor. Screenshots: `E:\Projects\Melee Workspace\scratch\envoy-demo\screenshots\stageaudit\` (not in the repo).

## Verdicts

| stage | grkind | verdict | one line |
|---|---|---|---|
| Yoshi's Island 64 | 29 (`GrOy.dat`, SSS ext 29) | **in** | passes every check; only the allow-list entry is missing (needs the hop fix below, which is committed) |
| Kongo Jungle 64 | 30 (`GrOk.dat`, ext 30) | **in with fixes** | crashed on every shift into it until the hop parse fix (committed here); then passes. The barrel cannon is a mild hazard: policy call |
| Pokemon Stadium, frozen | 16 (`GrPs.dat`, ext 3) | **in with fixes** | works, but Envoy must force Frozen PS on, the jumbotron is black after a re-entry, and it leaks ~2x a normal stage per shift |
| All-Star rest area | 66 (`Gr_Kind_Heal`) | **out** (as a VS shift target) | no external stage id, reads All-Star mode state, and the match ends ~150 frames after the shift ("Game!", RUN OVER with 4 stocks left) |

"In" means: add the kind to `st_retail_supported` (`rust/melee_modes/src/tu/script_game.rs` and the C twin
`pc/gameworld/script_stage_slot_retail.h`) and to the sim's `STAGES` table. I did not add any: the audit was done with
`MELEE_STAGE_HOP_EXTRA`, so nothing ships by accident.

## What was run

Headless goldens (`melee golden --engine tu`, the same path as `m0_stage.sh`) with the Envoy sim mod, plus titled `melee play`
windows for the pictures. Every run: `MELEE_VOLUME=3`, a `MELEE_WINDOW_TITLE`, no OS input (plays use `MELEE_PAD_SCRIPT` =
one idle line; `MELEE_SCENE` for the match; `MELEE_TURBO=1`), own cargo target dir (deleted afterwards), cargo `-j 4`.

* Scene: Fox (P1, made a level-9 CPU by `MELEE_ENVOY_TEST_P1CPU=1`) + Marth (partner) against Falco lv9 (the Envoy plan then
  rewrites it to fight 4: 3 CPUs, teams, stock pool, `MELEE_ENVOY_START_FIGHT=4`), start stage BF or the candidate.
* New test knobs (this branch, both inert unless set):
  * `MELEE_ENVOY_ARM=1`: the run counts as armed without the camp (a scene / `--match` start has no menus). `envoy::armed()`.
  * `MELEE_STAGE_HOP_EXTRA=30,29,16`: those ground kinds also pass `stage_hop::supported` (start stage, shift source and target).
* An audit copy of `envoy_sim` with `STAGES = [BF, X, BF, X, BF]` and a shift every 600 frames (not 1800), so one run
  alternates X -> BF -> X ... (`tools/rust-port/envoy/stage_audit.sh GRKIND` rebuilds it and does the checks below).
* Check "wipe": each shift must freeze the per-frame hash for 90 frames (the sim's stage hold: switch at 45, release at 90).
  Every shift of every stage showed the two frozen halves (44 + 45 frames) in the hash CSV, e.g. 1800-1843 + 1844-1889.
  The reward screen (frames 1-1201, 20 s) is the first frozen run.
* Check "deterministic": two golden runs, byte-identical hash CSVs.
* Check "memory": `heap0 free` printed by every switch, and the host working set sampled every 3 s.
* Camera / CPUs: 4-CPU free-for-all (4 stocks, lv9) on the stage alone, 3600 frames, shots at 300 / 900 / 1800 / 3000 / 3550,
  next to the same on BF.

## Results

| | Yoshi's Island 64 | Kongo Jungle 64 | Pokemon Stadium (frozen) | rest area |
|---|---|---|---|---|
| loads and plays with CPUs | yes (3600 frames alone, 6364-frame Envoy fight won by the CPU team) | yes (3600 alone, 6146-frame fight won) | yes (3600 alone, 7200 Envoy frames) | no (see below) |
| camera frames | yes | yes; tightest of the three (stage camera zoom), left edge of the background shows black when it pans far | yes | n/a |
| shift into and out of, 90-frame wipe | 7 shifts, all 90 frames | 6 shifts, all 90 frames (**after the fix**) | 8 shifts, all 90 frames | 1 shift in, loads and draws |
| CPUs self-destruct or stick | no: 150-210 % and 3-4 stocks each at frame 3550, same as BF | no: stocks 4 -> 2-3 over 3550 frames | no | n/a |
| crash | none | **assert before the fix**, none after | none (also through a real transformation, see below) | none |
| `heap0 free` drop per shift | 0.34 MB | 0.34 MB | 0.94 MB | n/a |
| working set (host) | 127 MB flat | 127 MB flat | 126 -> 160 MB | 126 MB |
| deterministic | yes (6364 rows x2) | yes (6146 rows x2) | yes (7201 rows x2, frozen and unfrozen) | yes (2163 rows x2) |

Baseline for the same harness: BF <-> YS drops 0.45 MB per shift; FD, FoD, DL pass the same script (deterministic, no
panic). `heap0 free` starts near 21 MB, so at 0.4 MB per shift the known switch leak (plan 6.2: "fix the stage-switch heap
leak", milestone 0) runs the budget dry well before a 50-70 shift run. That is not specific to these stages, but PS costs
2.5x more, so it must be rare in the shift pool until the leak is fixed.

### Kongo Jungle 64: the crash and the fix (committed)

Shifting into Kongo Jungle 64 died in `grOldKongo_80210650` (`HSD_ASSERT(786, 0) groldkongo.c`): the barrel cannon's
direction roll sums `yakumono_param.rrate_barrel_*`, and after a shift the whole record was zero. Cause:
`melee_engine::dat_custom` decodes the archive's `yakumono_param` root at `HSD_ArchiveParse` time and picks the field
layout from `stage_info.grkind`; `stage_hop::switch_to` parsed the destination archive while `stage_info.grkind` still named
the outgoing stage, so the record was decoded with the wrong stage's layout (BF's: two pointers, `0x8` bytes). Fix: set
`stage_info.grkind` to the destination around the parse only. It also changes the hash of a hop into any stage whose
layout differs from the outgoing stage's (YS out of BF, now correct): determinism is unaffected, but any golden that
hashes such a hop (e.g. `stage_switch.sh`'s CSV against an older run) differs from before. This probably is not why FD shows
pink edge lines (the demo README), but a hop into FD / BF had the same wrong decode (their records are pointer tables), so
look at that again after this lands.

### Pokemon Stadium

* Transformations: `MELEE_SLIPPI_FROZEN_PS=1` (the hook in `melee_link/src/manual.rs`) freezes them and works unchanged here.
  Unfrozen, a transformation starts about 3700 frames after entering (golden `pslong`: hashes diverge at frame 4943, no
  crash, no leak jump). Envoy must turn Frozen PS on itself for every armed run (one line in that hook: `|| envoy::armed()`),
  and since each visit is short with a 90-frame wipe in and out, a visit under 60 s never transforms anyway.
* The jumbotron: first visit shows the "Current Stage / Combatants" text, then the live match feed; after a re-entry by a
  shift the screen stays black (`shift-ps-age2555`, `age2650`). Cosmetic, not investigated.
* Leak: entering costs 14 owned objects and a 1.47 MB archive; about 2 MB per BF -> PS -> BF cycle (BF <-> YS is 0.9).

### The rest area (Gr_Kind_Heal, 66)

* No external stage id: `Stage_StageSlotSelect(66)` is -1, so `match_begin` leaves the scenario's stage (BF) and a VS match
  cannot start on it. A shift into it is possible through `stage_hop` (`MELEE_STAGE_HOP_EXTRA=66`, 840 650-byte archive).
* After the shift it loads and draws correctly (park, trophies, healing cylinder: `rest-heal-age1900-after-shift.png`), but
  "Game!" appears about 150 frames later (`rest-heal-age2000-game-end.png`) with Fox at 4 stocks, and the sim logs RUN OVER.
  I did not chase the cause. `grheal.rs` reads the All-Star state (`gm_80473A18`: roster, coins), which a VS match does not have.
* It needs its own scene (a camp-like stage built from the All-Star rest scene), not a shift. Out for 1.0's shift pool;
  worth a separate design item for the rest encounter.

## Not covered

* Real windows at 60 Hz with the wipe overlay: the turbo shots show the stage under the wipe but not the mod-drawn flash;
  the wipe was verified by the 90 frozen frames in the hash trace.
* Items, the Envoy HUD and tag cinematics during a shift (items were off, tags not triggered by the CPU player).
* Netplay (the plan is offline-only in 1.0).
* Kongo's barrel cannon and YI64's moving cloud as encounter hazards (policy: "hazard stages never in 1.0").

## Files

* `rust/melee_modes/src/tu/stage_hop.rs`: the parse fix, `MELEE_STAGE_HOP_EXTRA`.
* `rust/melee_modes/src/tu/envoy.rs`, `rust/melee_modhost/src/lib.rs`: `MELEE_ENVOY_ARM`.
* `tools/rust-port/envoy/stage_audit.sh`: the repeatable check (`GW_ISO_VANILLA=... stage_audit.sh 30`).
