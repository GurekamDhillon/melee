# Envoy drives, milestone V1: the drive slice (2026-11-10)

Built from `envoy-core-v2.md` (port/envoycore2) section 6, V1. Branch `port/v1`, off `port/m1` (whose plumbing it
reuses) with `rust-port` merged in. This replaces milestone 1's crests (`envoy-crests.md`, removed here): the 26
crests, the moment families, Fresh / Benched, the crest offer and `m1_crests.sh` are gone. Only purpose: the owner's
verdict on the feel ("does this feel like the old Envoy?"). Not merged into rust-port until he has played it.

## What is in

| piece | where |
|---|---|
| pool | `rust/mods/envoy_data/pool.toml` -> `build.rs` -> `pool_gen.rs` (`POOL`, `POOL_LEN`, `POOL_DIGEST`): 9 drives (Ember, Nova, Bolt, Rush, Bulwark, Comet, Frost, Shatter, Relay) and 3 keystones (Plague Bearer, Echo Oath, Overclock), the "S" rows of the note's section 4 |
| vocabulary | `envoy_data/src/codes.rs`: 6 families, 6 charge sources, 12 effects, the four words (Burning, Chilled, Marked, Haste), the seven prices, level scale x1.0 / x1.6 / x2.4, the 20-frame fire gap |
| lint | `envoy_data/src/spec.rs`: <= 12 words, <= 2 numbers, no duplicate key / name / rule, a word in the rule is one it gives or reads, a keystone has exactly one price, a drive has a sound and a drive effect, an estimated fire period inside 2-16 s. The full-pool rules (3 per family, 2 givers per read word) switch on with `FULL_POOL` in V2: the slice has 1 giver of Chilled and 1 drive in some families |
| sim | `envoy_sim`: drives `[2][4]` of (id, level), 3 slots open; charges; the fire queue; the 8-entry fire log; statuses; the floor drop; the pick of three with a fighter step; the keystone pick; prices; all in the one `#[repr(C)]` state hashed into `gd_hash_word` every frame |
| world draw | `melee_modes::tu::drive_draw` (new): the `envoy_drives` meshes and atlas (`include_bytes!` from `pc/scripts/examples/envoy_drives/models`), decoded once, lit per vertex, drawn through GX in slot 0's fighter draw pass (`ftdrawcommon::WORLD_DRAW`) with the current camera, depth-tested; stand-in effect shapes (ring, disc, spikes, bolt, beam, line), additive |
| host imports (sim) | `gd_envoy_test_drives` (`MELEE_ENVOY_TEST_DRIVES`), `gd_envoy_shared_drives` (`MELEE_ENVOY_SHARED_DRIVES`); `VIEW_WORDS` 160 -> 256; run record version 3 |
| host imports (UI) | `gd_ui_world_model(model, x, y, z, yaw, scale, glow, alpha)`, `gd_ui_world_fx(kind, x, y, z, a, b, c, rgba)`, `gd_ui_model_tex(model)` (a CPU-rasterized 12-frame turning sprite sheet for 2D cards), `gd_ui_brawl_sfx(name)`. Presentation only: none returns anything a sim could read |
| UI | `envoy_ui`: orbiters, the keystone halo, the orb, foe statuses, the floor drop, each fire's effect, sounds; keystone and drive cards with the turning model; the fighter step; the pair's drives in the camp and the between-fights line |

## The loop

- **Charges** (fighter in play only; the bench holds its charges): hits landed and hits taken from the
  `damage.fighter_take_damage` post hook (the victim's slot says which), landings from `action.Landing.enter` and
  `action.LandingFallSpecial.enter`, time (frames of unfrozen play), tag-in. Shatter only counts hits on a Chilled
  foe. Overclock adds 2 per event / frame.
- **Fire queue**: a full drive fires on the next frame its set has not fired for 20 frames, slot order. Relay's
  volley fires the set's other drives 6 frames apart, outside the gap. Echo Oath repeats each fire 12 frames later at
  half strength.
- **Effects** (damage only: nothing launches in V1; `gd_envoy_blast` is V2): Ember (6% and Burning on the foe its
  charge hit; level 3 splashes), Nova (12% to foes within 38 units), Bolt (8% to the nearest; level 3 chains), Rush
  (Haste 4 s), Bulwark (an orb: the next hit on the fighter is skipped by the take-damage pre hook, so no percent;
  knockback still applies), Comet (6% within 32), Frost (Chilled 4 s), Shatter (10% and the Chill ends; link line),
  Relay. Burning is 2% a second per stack (Plague Bearer stacks to 3); Chilled and Haste multiply walk and dash speed
  (x0.7, x1.3) and are undone on expiry, a KO or a tag.
- **Prices** (on the fighter in play's slot ratios): Fragile defence 1.25, Weak attack 0.8, Bleed 1% per fire.
- **Earning**: the first KO dealt in a fight drops a drive the fighter in play can take (else its partner) on one of
  four floor points (x = -35, -12, 12, 35; y 0); walking over it takes it. If the fight ends with it on the floor it
  is swept up (fights 1, 3 and 4 end on their first KO). Fights 2-5 open with three drive cards (none the pair
  cannot take; one connects to a word the pair has when it can; `+1 stock` third at pool 1), then the fighter. A copy
  of a worn drive levels it (to 3). The run opens with the keystone pick.
- **Shared knob**: `MELEE_ENVOY_SHARED_DRIVES=1` makes set 0 the pair's (cards skip the fighter step).

## Seeing it

Orbiters: up to 4 around the fighter in play at chest height, 3.6 units, glow from charge (pulsing when full). A
fire darts the model to its target in 6 frames, flashes, and it comes back dim in 8. Level 2 and 3 add the
`drive_ring_magic` / `drive_ring_rare` overlay; each keystone adds a `drive_ring_unique` crown over the head. The
drop is the model in the glass shell with a family-colour beam. Cards and the camp show the model turning (sprites
rasterized from the same mesh). Sounds (Brawl wave sounds from the player's disc, keyed so a rollback does not
replay them): fire per family, pickup, level-up, keystone. No text on a fire, a pickup or a level-up.

## View words (envoy_sim `publish`)

See the comment above `publish`: 13-17 the offer and the step, 18 shared, 22-29 drives, 77-84 levels, 85-92 charge
permille, 93-95 keystones, 96-105 positions (f32 bits), 106-115 statuses and the orb, 116-119 the drop, 120-125 the
last gain, 126 the fire log count, 127 can-take bits, 128-191 the fire log.

## Checks

`tools/rust-port/envoy/v1_drives.sh`: the lint tests; a pad run (time drives, Relay, Comet, Echo; the card taken;
tags) and a CPU run (hit drives, Plague Bearer, a drop taken) and the shared knob, each synctest depth 4 over 3600
frames with 0 mismatches.

## Known limits

- Chilled / Haste are attribute multiplies and their inverse (0.1% drift per cycle); the in-place tag prototype
  (`MELEE_TAG_INPLACE=1`) respawns the fighter and can lose or double them.
- The world draw happens in slot 0's fighter pass: while that fighter is not drawn (KO'd) the orbiters and the drop
  are not drawn either.
- Co-op, attribution, launching blasts, elites with drives, keystones after bosses and 4 slots are V2-V3.
