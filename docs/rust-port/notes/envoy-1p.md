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
