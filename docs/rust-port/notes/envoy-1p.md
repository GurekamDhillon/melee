# Envoy single-player slice (port/envoy1p, 2026-11-09)

A playable 1-player Envoy run as two wasm mods, built on the tag fighter (`tag_swap`) and stage switch (`stage_hop`)
work. Spec: envoy-plan.md (port/envoyplan) with the owner's answers of 2026-11-09 (E8 20 s / offer 0, E10 own % +
shared pool + in place, E12 D-pad down / ~5 s / air, E13 60 / 90 frames, E16 run-triggered switches, E17 wipe and
flash, E18 retail stages, E22 compare rows only). Looks are placeholders (kit slices and text, no art).

## Play

| what | how |
|---|---|
| mods | `bash tools/rust-port/modhost/build_mods.sh`, then `build/mods/envoy_sim.wasm` + `envoy_ui.wasm` in the `MELEE_WASM_MODS` directory. Do not also load `tag_fighter.wasm` / `tag_fighter_ui.wasm` (they would tag a second time) |
| start | `melee play`, VS mode. **P1's pick is the main, P2's slot (HMN or CPU, any character) is the partner.** Any stage, Start. Every VS match while the mod is loaded is the run's next fight |
| fights | 5: 1 CPU lv3 (1 stock), 1 CPU lv5 (2), 2 CPUs lv5-6, 3 CPUs lv6-7, boss (lv9 Ganondorf, x1.25 damage, x0.75 damage taken, x1.25 size, 3 stocks). Stages BF, YS, DL, FoD, FD. Teams on (you vs the CPUs), items off, no timer |
| pool | 4 stocks shared by the pair across the run; a fight is won when every CPU is out; the pool carries over |
| tag | D-pad down, on the ground or in the air when actionable, 5 s cooldown, 60-frame frozen cinematic (D-pad down skips after 6 frames). Each fighter keeps its own % |
| stage shift | from fight 2 on, every 30 s of play: the next legal stage under a 90-frame frozen wipe and flash (the switch at the midpoint) |
| reward | at the start of fights 2-5 the sim freezes: 3 upgrades with before -> after rows; D-pad / stick left-right, A takes; 20 s, then offer 0 |
| end | pool empty: RUN OVER; boss beaten: RUN CLEAR (shown on the results / CSS). The next VS match starts a new run |

Upgrades (stack): Power (damage dealt +0.20), Guard (damage taken -0.15, floor 0.40), Extra stock (+1 pool now),
Heavy (weight x+0.20), Swift (walk / dash speed x+0.12), Spring (jump x+0.12). Ratios are `Player.attack_ratio` /
`defense_ratio` (the handicap path); attributes are the live `Fighter.co_attrs` of both tag fighters, re-applied at
each fight start.

## Pieces

| piece | where |
|---|---|
| match rewrite | `melee_modes::tu::envoy::match_begin`, top of `fn_8016E730` (VS only, never on replay playback): players, teams, stocks, ratios, CPU line-up, stage, `TAG_PARTNER_CK[0]`. Inert while no mod has published a plan (`PLAN[0] == 0`) |
| sim imports | `gd_envoy_plan/publish/seq/roster/apply`, `gd_tag_begin_ex(slot, len, flags)` (bit 0: air), `gd_stage_transition(grkind, len)`, `gd_hold_begin(len)` / `gd_hold_release()`, `gd_transition_kind()` |
| UI import | `gd_ui_envoy(out, n)`: the published view, read-only |
| freezes | `tag_swap`: kind 2 (stage: switch at len/2, release at len, both inside `tick`), kind 3 (hold: released by the mod). Air tags enter Fall instead of Wait |
| load hook | modhost calls an optional `gd_mod_loaded()` export once after a mod loads (the run's first plan exists before the first match) |
| mods | `rust/mods/envoy_sim` (run state, tags, switches, rewards, outcome, `gd_hash_word`), `rust/mods/envoy_ui` (HUD, reward cards, tag cinematic with the record's length, stage wipe, between-fight line) |

## Stubbed / not done

- Second Wind (3 s rewind per fight): not in; a sim-side rewind needs `gd_match_rewind` (plan 3.9), not cheap from a mod.
- Master Hand boss: not in (a buffed level-9 Ganondorf stands in).
- No partner pick UI: the P2 CSS slot is the pick; Zelda / Sheik / Ice Climbers fall back (main: Fox; partner: Marth, or Fox
  when the main is Marth).
- Run state lives in the sim mod's memory for the session only (no save); quitting a fight retries it.
- Offline only: the plan is not in the netplay handshake.

## Test knobs and checks

| knob | effect |
|---|---|
| `MELEE_ENVOY_START_FIGHT=N` | each new run starts at fight N (1-5): reward / stage shift / boss without winning the earlier fights |
| `MELEE_ENVOY_TEST_P1CPU=1` | the player's slot is a level-9 CPU (the headless win path) |

Verified 2026-11-09 (pad scripts only): two goldens with tags + reward + a stage shift byte-identical (2401 rows);
synctest depth 4 over 2400 frames 0 mismatches (9588 checks); `MELEE_SNAP_CHECK=1` passes; `melee play --match`
headless: fight won -> results -> CSS -> SSS -> fight 5 with the pool carried -> RUN OVER -> results -> a new run.
Found on the way: a VS rewrite must also set `match_kind` (the `--match` preset is a time match) and keep handicap 9;
CPUs need `cpu_kind` 4. Stage switches leak a little heap 0 each time (~0.7 MB per switch in a long fight; stage_hop).

Demo crash 2026-11-09 (`link cast: size mismatch *mut i8 -> &[usize] (site 79202)`), two faults:
- The `OSReport(fmt, arg)` link glue of sislib / sobjlib / synth cast the varargs argument into `os_error::OSReport`'s
  `&[usize]` and panicked on any report. gen_link.py `osreport_adapter` now builds the slice (melee_link/tests/osreport_glue.rs).
  gen_link.py's full output has drifted from the committed gen_table.rs, so only the three OSReport lines were spliced in.
- The report was sislib's "Memory Empty": a 4-player results screen (1 vs 3 CPUs, fights 3-4) holds ~126 texts, which
  overflows the retail 0xC000 text pool with the native 8-byte-pointer structs. `HSD_SisLib_803A6048` now scales the
  pool by `HSD_SISLIB_POOL_SCALE` (2 on 64-bit; the C-oracle test sets 1). Reproduced with `MELEE_ENVOY_START_FIGHT=4`
  + `MELEE_ENVOY_TEST_P1CPU=1` on a pad-script loop (crashed at the 5th results screen); not Envoy-specific, any
  4-player VS results screen is near that limit.

## The door (port/envoydoor, 2026-11-09; envoy-redesign.md A1)

| piece | where |
|---|---|
| 1-P menu row | **native cursor, mod-drawn row.** The 1-P submenu is part of the mnMain scene (not a scene of its own), so `gd_ui_scene_replace` cannot replace it, and a real retail row would need a new jobj model from disc data. Instead `mn_8022D7F4` (1-P think) gets a virtual row past the last unlocked entry (`envoy::door_1p`, inert unless `DOOR_ON`); envoy_ui draws it as a kit panel under the list. The retail cursor stays lit on Training while the Envoy row is hovered (placeholder) |
| camp | envoy_ui scene `envoy.camp` (kit nodes): Continue / New run / Party / Abandon (asks twice) / Back on the menu; after a fight Continue / Party / Abandon / Leave, or RUN CLEAR / OVER with Back |
| imports | sim: `gd_envoy_armed()`, `gd_envoy_record(out, cap)`. UI: `gd_ui_envoy_door(scene)`, `gd_ui_envoy_state()`, `gd_ui_envoy_saved(out, n)`, `gd_ui_envoy_arm(op)`, `gd_ui_scene_replace_when(major, minor, scene, cond)` (cond 1: armed and a fight since arming, so plain VS never sees a custom frame) |
| arming | `envoy::ARMED` (menu-time static). `match_begin` and the sim's `make_plan` both need it; cleared by Abandon, Leave, the end of a run (the camp's Back) and entering the main menu. Camp launches VS through `envoy::LAUNCH_VS` (mnMain on-frame, the menu's own two writes) |
| record | 16 words in VIEW[40..56] (magic, version, seed, runs, fight, pool, stats x5, main, partner, picks, -, FNV digest); `melee/src/envoy_store.rs` writes `<exe>/melee_data/envoy/run.bin` (or `MELEE_ENVOY_DIR`) on change while live, deletes it on clear / over / New run / Abandon; offline only |
| browser | MODES `envoy` (group Solo) is available while envoy_ui registered the door; LAUNCH opens the camp (`DOOR_OPEN`) and the browser closes |

Verified (pad scripts, MELEE_BOOT_MENU=1, MELEE_PAD_CHANNELS=01): 1-P > Envoy > camp > New run > CSS > fight 1 won
(`MELEE_ENVOY_TEST_P1CPU=1`) > results > camp "Next: fight 2/5 pool 4"; next session the camp shows the saved run;
VS with both mods loaded and a saved run on disk: retail 2-minute time match, no "envoy: match rewritten" line, no HUD.
