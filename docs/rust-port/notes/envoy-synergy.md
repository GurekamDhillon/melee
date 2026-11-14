# Envoy synergy lane: proc chains, synergies, evolutions, co-op items (2026-11-13)

Branch `port/envoy-synergy`, built on slice 2 (`envoy-items-design.md` section 11). The data and the chain resolver
are in `envoy_data/src/procs.rs`. The sim side is in `envoy_sim/src/items.rs`, in the "synergy lane" section. The UI is
in `envoy_ui/src/items_ui.rs` (effects, banner, sounds) and `pause.rs` (the Combos page and the item card's
"Combos:" line).

## Proc chains (Risk of Rain 2)

An item hit is any item effect landing on a foe: a burst, a shot or a spark. Each source item has a hit kind
(`hit_kind`) and a **proc coefficient** (`proc_coef`, x1000). Big single hits have 1.0. Sparks and prism bolts have
0.5. Orbs, wake ticks, feathers and ice shards have 0.3. Chain Spark arcs and Kin Tether have 0.2.

| proc item | id | keyword text | base chance | makes | its coefficient |
|---|---|---|---|---|---|
| Ember Seed | 48 | ON BURST HIT: may bloom a fire seed (a burning hit) | 25% | burst | 0.5 |
| Static Coil | 49 | ON SHOT HIT: may arc to another foe | 25% | spark | 0.6 |
| Frost Bud | 50 | ON SPARK HIT: may burst into frost (chill) | 20% | burst | 0.4 |

Chance = base x stack (1 + 0.5 per copy past the first, to x2) x the source's coefficient x (1 + bonus), capped at
100%. A proc's own hit is an item hit of its kind, so chains form. For example, a star hits, Static Coil arcs (spark),
Frost Bud bursts (burst), then Ember Seed blooms (burst), and so on. Each child carries its parent's coefficient
times its own, so a chain fades quickly.

**Caps.** A chain is at most `PROC_DEPTH` = 4 deep and `PROC_MAX` = 12 procs long. A branch whose coefficient falls
under 0.06 stops.

**Determinism.** `procs::resolve` works out the whole chain at once, breadth first, using integers only. Its inputs
are the run seed, the play frame, `S.proc_n` (a per-run nonce in the hashed sim state, bumped per root hit) and the
player. Its RNG is a hash mix, not a stream, so evaluation order cannot shift it. The sim then applies the procs in
the order resolved. Their own hits run with `IN_PROC` set, so they never start a second chain. `boom` now takes
`BOOM_ID` and clears it, so a stale id from a rewound frame can never change a later hit's source.

## Synergies (named pairs; a banner the first time each fires in a run)

| synergy | items | effect |
|---|---|---|
| Thunder Bell | Shine Bell + Chain Spark | the shine's ring arcs to every foe within 2.5x its reach |
| Spectrum | Laser Prism + Halo | the prism splits into five bolts, not three |
| Wildfire | Ember Seed + Fire Lace | Ember Seed procs twice as often |
| Blood Moon | Moon Drops + Crimson Count | moon hits burn |
| Iron Maiden | Bloodlust + Crouch Thorns | taking a hit also grows the thorns |
| Tectonic | Smash Quake + Big Bang | the quake runs twice as long and 1.5x as tall |
| Superconductor | Static Coil + Frost Bud | Static's arcs chill; Frost Bud's bursts arc to the next foe |
| Seeker Stars | Hop Shot + Homing Ink | one more star, and stars home 1.6x harder |

## Evolutions (an item at its cap plus a partner item)

| evolution | recipe | effect |
|---|---|---|
| Storm Crown | Spark Ring x3 + Chain Spark | a nair hit also calls lightning on up to three foes near the victim |
| Eclipse | Moon Drops x4 + Halo | moons hit twice as hard (black moons with gold rims) |
| Inferno Seed | Ember Seed x3 + Fire Lace | seeds launch (a real knockback hit) and proc +50% more |

An evolution is announced with a banner and a burst when it forms, which is when the item that completes it is picked
up.

## Co-op items (2P only; never offered in a solo run)

| item | id | effect |
|---|---|---|
| Kin Tether | 51 | a light line between the partners (when they are under 160 units apart); a foe crossing it is nicked, once every 40 frames per foe |
| Rally Banner | 52 | each KO rallies your partner for 5 s (longer with stacks): +1% on their item hits, +50% proc chance, and a gold pennant over them |
| Kindred Spark | 53 | your item hit on a foe within 60 units of your partner jumps a spark from them (1%), and their proc items roll off it at 0.5 coefficient; once every 20 frames |

## Storage

Ids 48-55 do not fit the view's 48-id block or the record's item words. In the view they live in `V_INV2` (use
`view::inv_word(p, id)` for any id). In the record they take two bits a copy (cap 3, never repeatable) in the spare
high halves of `R_ACTIVE` (player 1) and `R_PIPS` (player 2). The item lint enforces this. Other new view words are
`V_BANNER`, `V_BANNER_AT`, `V_SYN` (held synergies and formed evolutions as bits) and `V_RALLY`. Which synergies have
been announced is run state; it is not in the record, so a resumed run shows a synergy's banner again.

## Visuals and sound

Each new effect has its own `world` burst drawn from the existing world-fx primitives (ring, disc, spikes, line,
beam), and its own Melee hit look (element, sound family, severity) in `hit_look`:

- Ember Seed: five flame petals.
- Static Coil: a crackle star and an arc line.
- Frost Bud: a six-spike frost flower.
- Kin Tether: the line with beads running along it.
- Rally Banner: a pennant.
- Kindred Spark: a counter-spinning star.
- Synergies: a double ring in the synergy's colour.
- Evolutions: an eight-ray gold crown.

The Brawl sound keys are in `sounds`. The banner is a sticker in the lower middle of the screen, never a full-screen
flash.

## Tests

- `envoy_data` unit tests (`procs::tests`, 8 tests):
  - the same seed gives the same chain;
  - different seeds give different chains;
  - depth and length stay within the caps even at 100% chance;
  - the coefficient falls along a chain, and the floor stops it;
  - nothing procs when no proc item is held or the source has no hit kind;
  - the chance scales with the coefficient;
  - a 25% proc rate holds over 20,000 rolls;
  - the tables agree.

  `envoy_sim` is a no_std cdylib that imports the engine, so the resolver it calls lives in `envoy_data`, where it
  can be tested on the host.
- `items_check.sh`:
  - row 20, `synergy`: procs fire, a chain goes two or more deep, no proc is deeper than 4, a synergy banner shows,
    and Storm Crown and Inferno Seed form;
  - row 20b: the proc sequence is identical at both synctest depths;
  - row 21, `coopsyn`: the co-op items fire.

  Every run is a synctest, so rollback safety is checked against 0 mismatches.
