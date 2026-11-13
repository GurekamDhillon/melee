# Petey boss slice (port/petey)

Superseded for behaviour by petey-behavior.md (2026-11-09, petey4): the rules, timings, HP and hitboxes now follow
the clean-room spec; the stand-in attacks, HP and stun below are gone.

Status: built and checked 2026-11-09 (gate OK; replays 392/428, 125/125, 917/917 as before; lab-check ALL OK;
geno-tests 64/64; a Petey synctest depth 4, 0 neutral mismatches). Second pass the same day: CHR0 animation played
per frame from the sim state, item / projectile hits, attacker hitlag + hit sound, the player-count HP scale; the same
checks again (Petey synctest depth 4 over 8143 passes, 0 neutral mismatches). Petey is an enemy only: the Envoy boss entity,
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
| partner in a cage | dropped in candidate8 (owner: switching characters must work against Petey): `petey::partner_caged()` is always false, so tags work in the boss fight; stage shifts are still refused during the fight |
| HUD | `gd_ui_petey` (read-only, 12 words) -> envoy_ui: HP bar, two cage bars, phase 2 / stun / result |
| draw | `ftdrawcommon::BOSS_DRAW`, called in the carrier's render pass: body capsule, head, cage bars, the partner |

Rules: HP separate from %, super armour (never flinches); cages have their own HP; breaking either cage frees the
partner and wins, draining HP wins. HP = base x (1 + 0.6 (players - 1)), players = the slots on the player's side
(1-2; counted in `envoy::match_begin`, so 1 until Envoy has a second human).

Hits on Petey: fighter hitboxes (latched per hitbox lifetime) and every item's live hitboxes (projectiles, thrown
items; latched per item gobj + hitbox in `LATCH_IT`, items of the carrier skipped) damage the body / head and the
cages. A fighter's landed hit sets `dmg.x1914` (the engine's "dealt damage" word, as when hitting an item), so the
next `Fighter_procCollResolve` gives it Melee's own hitlag and deal-damage callback; the hit's sound is Melee's
`lbColl_80005BB0` for that hitbox (its sfx kind / severity).

Animation (with the local model): `petey::anim_choice` picks the CHR0 and frame from the boss statics and the match
frame only (no wall clock; a replay or a rollback redraws the same picture): idle `Wait1` (`Wait2` after odd attack
instances and in phase 2), `Angry` once when phase 2 begins (`p2_frame`) and for the bite, `BasketAttackL` / `R` for
the slam / sweep over the whole wind-up + active + recovery timeline, `JumpStart` / `JumpLoop` / `JumpAttack` /
`JumpAttackLanding` for the belly flop, `DamageLeft` / `Right` for the stun, `Down` (held) when beaten; the spin is a
fast `Wait2` with the yaw turning. The arm models play the CHR0 of the same suffix and are shifted so their centre
sits on the sim's cage (what the hit test uses). Skinning is per frame from the bone matrices
(`petey_model::Rig::skin`). Phase 2 at 50% HP: wind-ups and recoveries
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
- The real model animates: idle, slam / sweep basket attacks, the belly-flop jump set and the bite pose seen in shots;
  the arms follow their own CHR0 and sit on the sim's cages.

## Stubbed / not done

- HP / cage HP / timings are still our stand-ins (300 / 70): see petey-extract.md "Numbers" for how far the param
  accessor was traced; no field could be confirmed as HP or cage HP.
- Items hitting Petey are not destroyed or reflected by him (they pass through after their one hit).
- Envoy has one human today, so the 2-player HP scale is wired but not exercised.
- Original arena: FD stands in.
