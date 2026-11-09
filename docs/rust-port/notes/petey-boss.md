# Petey boss slice (port/petey)

Status: code written, **not built or tested yet** (builds were on hold for the Envoy crash fix). Plan:
brawl-boss-petey.md; decisions P1-P9 of 2026-11-09. Owner-run asset extraction: petey-extract.md.

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

## Stubbed / not done

- Not built or run yet: the root-joint offset maths, the cpu_kind-0 carrier standing still and the KO path need a
  first in-game check, then gate / replays / lab-check / geno-tests / a Petey synctest.
- Real model / animations from the owner's extracted files: not loaded (placeholder capsules only).
- HP / cage HP / timings are guesses until the owner reads the real ones (petey-extract.md step 6).
- Projectiles and items do not damage Petey (only fighter hitboxes); no hitlag or sfx on hitting him.
- Player count for HP scaling is fixed at 1 (Envoy is single-player today).
- The P2 HUD still shows the carrier's (always 0%) damage box.
- Original arena: FD stands in.
