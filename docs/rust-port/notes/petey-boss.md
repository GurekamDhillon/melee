# Petey boss slice (port/petey)

Status: built and checked 2026-11-09 (gate OK; replays 392/428, 125/125, 917/917 as before; lab-check ALL OK;
geno-tests 64/64; a Petey synctest depth 4, 0 neutral mismatches). Petey is an enemy only: the Envoy boss entity,
never a playable character (no CSS slot, no Vs / LAB selection, no fighter define). Plan:
brawl-boss-petey.md; decisions P1-P9 of 2026-11-09. Where his data was found and how it loads: petey-extract.md.

## Play

`MELEE_ENVOY_BOSS=petey` plus the Envoy mods (envoy-1p.md). Fight 5 (`MELEE_ENVOY_START_FIGHT=5` to jump there)
becomes Petey on Final Destination; without the variable the level-9 Ganondorf stand-in stays.

## How it works

| piece | where |
|---|---|
| boss state, attacks, hit tests, placeholder draw | `rust/melee_modes/src/tu/petey.rs` (statics: snapshot / rollback covered; own LCG) |
| switch-on | `envoy::match_begin`: plan word 26 (`P_BOSS`, set by envoy_sim on fight 5) and the env var; arena FD |
| carrier | the fight's one CPU (slot 1): `cpu_kind` 0, invisible, intangible; Petey's attacks are its capsule slots (engine hit path); on a win it takes the bottom-blast KO so Envoy's win check runs |
| per frame | `petey::tick` after `tag_swap::tick` (modhost post chain); skips frames where the match frame does not move |
| partner in a cage | `tag_swap::partner_kind` is -1 while caged (no tags); stage shifts refused during the fight |
| HUD | `gd_ui_petey` (read-only, 12 words) -> envoy_ui: HP bar, two cage bars, phase 2 / stun / result |
| draw | `ftdrawcommon::BOSS_DRAW`, called in the carrier's render pass: body capsule, head, cage bars, the partner |

Rules: HP separate from %, super armour (never flinches); cages have their own HP; breaking either cage frees the
partner and wins, draining HP wins. HP = base x (1 + 0.6 (players - 1)). Phase 2 at 50% HP: wind-ups and recoveries
x0.65, shorter idles, 50% attack chains, faster spin.

Attacks (stand-in numbers [?]): cage slam, cage sweep, belly flop + ground shockwave (grounded-only), spin (re-hits
every 12 frames), bite lunge; a head hit of 12+ damage stuns for 120 frames (once per 10 s).

## Verified in game

- The carrier stands still (x 60 over a whole fight, cpu_kind 0), invisible, no HUD slot and no CP pointer.
- Attacks hit where the cages are (hitbox offsets through the root joint); a broken cage frees the partner, the
  carrier takes the KO and Envoy's win check ends the run (RUN CLEAR).
- No stage shift fires (the engine refuses it; the countdown is hidden). The tick keys on the match frame count
  (`gm_GetFrameCount`; `gm_8016AEEC` is the timer seconds and froze Petey).
- With the owner's local Brawl files present the real model draws (petey-extract.md); without them the capsules.

## Stubbed / not done

- Animation: the model is posed by the first frame of `Wait1` only; attacks move it rigidly (arms by the cage offset).
- HP / cage HP / timings are our stand-ins (the params blob is located but its fields are not mapped).
- Projectiles and items do not damage Petey (only fighter hitboxes); no hitlag or sfx on hitting him.
- Player count for HP scaling is fixed at 1 (Envoy is single-player today).
- Original arena: FD stands in.
