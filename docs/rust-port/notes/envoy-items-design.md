# Envoy items: replacing drives with hand-made items (design, 2026-11-13)

Design only. Nothing here is built yet. It replaces the drive / keystone / Heat-chain reward layer described in
`envoy-drives.md`, `envoy-v3-rules.md` and `envoy-v3-picker.md`. The fights, foes, loops, camp, Petey, the effects
layer (`envoy-fx.md`) and co-op stay.

## 1. Why the drives go

- A drive is a cell in a grid: one charge source (hit, taken, landing, jump, time, tag-in, % dealt ...) times one
  effect (blast, burn, chill, heal ...). The pool fills every cell, so every drive reads the same: "a meter fills,
  something fires". The owner: "homogenous and grey", "too programmatic".
- The effects are mostly numbers (6%, x1.3, Marked +30%) and orbiters. Nothing changes how the fighter looks or how a
  move feels in the hand, the way Isaac's tears or Megabonk's weapons do.
- Too many rewards, too often: a pick every fight, drops on every KO, a shop every session. Slots (3-5), levels and
  row-replacing make management fiddly ("shabby").

The new rule: **every item is one hand-made idea, tied to one thing you do in Melee, with an effect you can see.**

## 2. The item format

Items live in `envoy_data/items.toml` (replacing `pool.toml`), compiled by `build.rs` into a static table as the pool
is today, with a digest hashed into the run record.

| field | meaning |
|---|---|
| `id` | stable small integer; never reused (saves and replays store it) |
| `name` | 1-2 words |
| `pools` | one or more of `item`, `shop`, `boss`, `devil`, `angel`, `secret`, `curse` |
| `quality` | 0-4, as Isaac: drives pool weights and shop price |
| `trigger` | one entry from section 3 (or `passive` for modifiers and a few always-on items) |
| `effect` | one hand-written effect function in `envoy_sim` (`fn item_<name>`); no generic effect table |
| `visual` | the signature: a trail, projectile, orbital, spark, fighter tint or sound; a named entry in the fx layer |
| `tags` | what it produces, read by modifiers: `projectile`, `burst`, `trail`, `orbital`, `spark`, `status`, `launch` |
| `mods` | for modifiers only: which tags it changes and how |
| `stack` | what a second copy does (see below) |
| `price` | devil / curse items: `slot:1`, `slot:2`, `curse:<id>`; shop: coins from quality |

**Stacking.** No slots, no levels. Picking an item you have adds a copy. Each item states its stack rule in one line,
from a small set: `more` (one more projectile / orbital per copy), `bigger` (radius or length +50% per copy, capped),
`stronger` (damage x(1 + 0.5 per copy), capped at x3), `faster` (cooldown / 2^copies, floor 6 frames). The cap is per
item and written next to it. Unlimited items, but each item's own growth is bounded so knockback stays sane (the
balance pass's blast cap, `FOE_BLAST_CAP`, still clamps what lands).

**Modifiers.** A modifier has no trigger. It changes every item effect carrying a matching tag: "your projectiles home",
"your bursts leave fire", "your sparks chain". Two copies of a modifier stack by its own rule. This is where the
combinations come from (Isaac's Brimstone + Spoon Bender), not from a grid.

**Determinism (rollback netplay).** Everything the sim does with items is inside the one `#[repr(C)]` state hashed
into `gd_hash_word` each frame, as the drives are today:

- Inventory is a fixed array per player: `[u16 id; 64]` + `[u8 copies]` (64 distinct ids is far above a run's 15-30;
  no heap, no `Vec`). Item projectiles and orbitals live in a fixed pool (`[Proj; 48]` per run) with integer or
  fixed-point positions; the sim owns them, the UI only draws them from the published view.
- Every roll (pools, room contents, shop stock) comes from the run seed + floor + room index with the existing
  seeded RNG; nothing reads time, frame rate or host state.
- Triggers are read from engine event bits set at the exact engine site during the sim frame (section 3), never from
  the renderer or the UI. Visuals are presentation only (the `envoy-fx.md` rule): the sim never reads them.
- Hitboxes go through `gd_envoy_blast` (engine hit path, rolled back with the fighter) or the percent hook.
- One effect per item per frame, evaluated in id order, player 1 before player 2, so co-op order is fixed.

## 3. Triggers

The owner's list plus a few that fall out of the same hooks. "Have" means the engine already tells the sim today.

| trigger | engine event | today | needs |
|---|---|---|---|
| on hit (any) | victim's take-damage hook + `gd_envoy_attacker` | have | - |
| on whiff | an attack state's hitbox window ends with no hit registered | partial: state is polled, no hit-flag | new bit: `EVENT_WHIFF` set when an attack action ends with `x1A58`-style hit-registered flag clear |
| on aerial hit / nair hit | attacker in `AttackAir*` (the move id is known: AttackAirN, F, B, Hi, Lw) | have (aerial), move id: small read | a `gd_envoy_last_move(slot)` read returning the motion state at hit time |
| on smash hit | attacker in `AttackS4/Hi4/Lw4` | have (Grand Finale uses it) | - |
| on laser hit | Falco/Fox blaster item hits | no: projectiles are items, not the fighter | hook in the item hit path tagging the owner, `EVENT_LASER` |
| on shine / waveshine | special-low state entered; waveshine = shine then jump-cancel then LandingFallSpecial within N frames | shine state readable; wavedash already detected | add shine to `tech_frame` (state sequence, no engine change) |
| on wavedash | LandingFallSpecial within 3 frames of EscapeAir | have (`tech_frame`) | - |
| on L-cancel | `ENVOY_LCANCEL` hook | have | - |
| on short hop | jump squat released before the full-hop frame (`x1968`, jump kind) | jumps are polled, not hop height | new bit `EVENT_SHORTHOP` at the JumpF/JumpB entry where the short-hop branch is chosen |
| on roll / spotdodge / airdodge | `EscapeF/B`, `Escape`, `EscapeAir` entered | polled states: have | - |
| on ledge grab | `CliffCatch` entered | polled: have | - |
| on tech | `Passive*` states | have | - |
| on crouch cancel / ASDI down | `SquatWait` while hit | no | `EVENT_CC` at the hitlag site |
| on landing / jump | polled | have | - |
| on taking a hit | take-damage hook | have | - |
| on KO | KO path | have | - |
| on tag-in | tag fighter | have | - |
| on offstage hit | ledge table (5 stages) | have, limited | stage-extent read for other stages |
| on parry-like perfect shield | `GuardReflect`/shield hit in the first 2 frames | no | `EVENT_POWERSHIELD` at the shield hit site |
| on grab / throw | `Catch`, `Throw*` | polled: have | - |

New engine work is five small event bits in the existing `EVENTS[slot]` word (whiff, short hop, laser, crouch
cancel, powershield) plus the last-move read. Each is set at one engine site, only while a run is armed, the way the
L-cancel bit is. Everything else is state polling in `tech_frame`, which already exists.

## 4. Pools and reward rooms

Most rooms are fights with **no item**. Items come only from these rooms.

| room | mark on map | how you get the item | pool | per floor |
|---|---|---|---|---|
| item room | gold star | walk in, one item on a pedestal, take it (co-op: one each) | `item` | 1, always |
| shop | coin | 3 items for coins, plus a stock / a reroll | `shop` | 1 on floors 2+ |
| boss | skull | the boss drops one from a choice of 2 | `boss` | 1 (floor end) |
| devil deal | red door | after the boss, sometimes: 1-2 items, each costs 1-2 stock slots (see below) | `devil` | chance |
| angel room | white door | replaces the devil door if you took no devil deal this run: one free strong item | `angel` | chance |
| secret room | none (hidden) | revealed by a technique on a marked wall-ledge, e.g. waveshine into the wall, or 3 ledge grabs in 2 s | `secret` | 0-1 |
| campfire | orange flame | no item: refills stocks to the stock limit | none | 0-1 |
| curse room | purple | an item plus a curse (a run-long drawback item) | `curse` | 0-1 |
| challenge (optional) | crossed swords | a harder fight; win for an `item`-pool item | `item` | 0-1 |

Devil chance: 35% after floor 1's boss, +15% if you lost no stock on the floor, -20% per devil deal taken; an angel
roll replaces it when you have taken none.

**Stocks work like Isaac's hearts (owner's call, 2026-11-13).** Two numbers per run:
- **Stock limit** (heart containers): the most stocks you can hold. Starts at 4.
- **Stocks** (red hearts): what you have now. Dying costs one; 0 stocks ends the run. Stocks carry between fights.

A devil deal costs **stock slots**: the limit drops by 1 (2 for the strongest items), and stocks above the new limit
are lost. You can't pay with your last slot. Ways back up:
- **Campfire** (new map room, orange flame, 0-1 per floor): refills stocks to the limit for free. Never raises it.
- Shop: buy one stock (refill, up to the limit); rarely a stock slot for a high price.
- Items and events that add a slot: e.g. a "Spare Heart" item (+1 limit, +1 stock), an angel room's prize,
  a boss drop on later floors, a no-hit floor bonus. Hard cap 8.
The HUD shows stocks as full icons and empty slots as outlines, so a deal's cost is visible. The KO-ceiling idea
is dropped. Co-op: each player has their own limit and stocks; a devil deal costs only the buyer.

**Pacing (one loop = 3 floors, the endless NG+ loop kept):**

| | items expected | from |
|---|---|---|
| floor 1 | 2 | item room, boss |
| floor 2 | 3-4 | item room, shop, boss, maybe devil/angel |
| floor 3 | 3-4 | as floor 2, maybe secret / curse |
| loop 1 total | 8-10 | |
| loops 2-3 | +7-10 each | the same rooms, deeper pools (quality 3-4 weighted up) |

A good run ends around 15-30 items. Coins come from fights (the V3 cash-out stays as coins only) and are spent only
in shops. An item taken leaves its pool for the run (no duplicates from pools; copies come from a few items marked
`repeatable` and from the boss's choice).

## 5. Floors and roads

A floor is a small branching map shown on a map screen (the road screen grows into it):

```
 start - fight - fight -+- fight - [item]  -+- fight - boss
                        +- [shop]  - fight -+
                        +- fight - [secret?]
```

- 6-9 rooms a floor: 4-6 fights (duel, pack, elite), 1 item room, 0-1 shop, 0-1 curse / challenge, the boss.
- At each fork you pick one branch; special rooms are marked, so the choice is "the shop or the elite and the item
  room". This **is** the road: the current Open / Steep / Shifting roads become fight modifiers on branches (a Steep
  branch: foes carry more; a Shifting branch: stage switches on KO) rather than a whole-session pick.
- The map is generated from the run seed and floor number, so it is identical for both co-op players and on replay.
- The road-change-on-loop bug is being fixed separately; this design makes the choice per fork, every floor, every
  loop, so the bug's shape (road fixed after loop 1) cannot come back.
- Floors keep the stage rotation (BF / YS / DL / FoD / FD); a floor has a home stage, fights may switch.

## 6. Inventory, UI, co-op

- **In fight:** no item HUD. The items show on the fighter (trails, orbitals, tints). A new item flashes its icon
  and name over the fighter for 2 s with a one-line "what it does" ("Nair hits throw a spark ring").
- **Pause screen:** an icon strip per player, in pickup order, a copy count badge (x2) on stacked items. Hover/select
  shows name, trigger, effect, stack rule. Curses and devil items in their own colour. No management: nothing to
  reorder, sell or slot. That removes the "shabby" drive management outright.
- **Pedestal rooms:** the item floats above a pedestal with its icon; walk into it to take it (the current drop
  pickup code). Shop items show the price under them; walking in with enough coins buys.
- **Co-op: per-player items.** Each player has their own strip; item rooms and the boss give one item per player
  (two pedestals); shop items are bought by whoever walks in, from the shared purse; each player has their own stock limit and stocks, and a devil deal costs only
  the buyer's stock slots. A few items have a co-op line ("also on
  your partner's hits"). The existing `MELEE_ENVOY_SHARED_DRIVES` knob becomes a shared-inventory knob.

## 7. The first 40 items

Effects are integer and capped; numbers are starting points for the balance harness. "Q" = quality.

### Item room pool (`item`)

| # | name | trigger | what you see and feel | stack |
|---|---|---|---|---|
| 1 | Spark Ring | nair hit | the nair hit pops a ring of 6 small sparks that fly out and nick anyone near | more |
| 2 | Glide Wake | wavedash | the wavedash leaves a burning streak on the floor for 1.5 s; foes standing in it flinch | bigger |
| 3 | Shine Bell | shine | each shine rings a visible shock ring that pushes foes out 1.5x the shine's reach | bigger |
| 4 | Hop Shot | short hop | at the top of a short hop you spit a small arcing star toward the nearest foe | more |
| 5 | Ledge Spikes | ledge grab | grabbing ledge stabs a column of ice spikes up from the lip, spiking anyone above | stronger |
| 6 | Whiff Echo | whiff | a whiffed attack leaves a ghost of your fighter that replays the same move 20 frames later | more |
| 7 | Roll Mines | roll | rolling drops a blinking mine where you started; it pops after 1 s or on touch | more |
| 8 | Laser Prism | laser hit | a laser that hits splits into 3 thinner lasers fanning past the target | more |
| 9 | Comet Tail | full hop | your fighter trails a comet tail in the air; foes touching the tail take 2% and stagger | bigger |
| 10 | Tech Bloom | tech | teching blooms a flower of light that knocks foes away from you | bigger |
| 11 | Grab Static | grab | while you hold someone, arcs crawl over them; each pummel jumps a spark to the next foe | stronger |
| 12 | Smash Quake | smash hit | a smash that lands cracks the floor; a shockwave runs along the stage both ways | bigger |
| 13 | Dodge Afterimage | spotdodge | a cyan afterimage stays where you dodged and takes the next hit meant for you | more |

### Shop pool (`shop`)

| # | name | trigger | what you see and feel | stack |
|---|---|---|---|---|
| 14 | Bounce Boots | landing | landings leave a ripple; land on a foe's head and you bounce with a ring | bigger |
| 15 | Moon Drops | passive | 2 small moons orbit you and nick foes they pass through | more |
| 16 | L-Flash | L-cancel | a clean L-cancel flashes white and fires a short horizontal spark both ways | stronger |
| 17 | Edge Lantern | offstage hit | a lantern hangs where the foe was hit offstage and pulls them down slightly while lit | stronger |
| 18 | Coin Magnet | KO | KO'd foes burst into coins that fly to you (+coins; a gold shower you can see) | stronger |
| 19 | Air Dash Trail | airdodge | airdodges leave a hard-light rail behind; foes touching it are tripped | bigger |
| 20 | Throw Sling | throw | the thrown foe drags a chain; anyone it hits mid-flight is pulled along | stronger |
| 21 | Crouch Thorns | crouch cancel | crouch-cancelling a hit grows thorns from your back that hit the attacker back | stronger |

### Boss pool (`boss`)

| # | name | trigger | what you see and feel | stack |
|---|---|---|---|---|
| 22 | Petey's Cage | passive | a rotating half-cage orbits you and blocks one projectile every 4 s (it flashes when it does) | more |
| 23 | Thunder Nair | nair hit | nair hits call a lightning bolt down on the target (a real launching hit) | stronger |
| 24 | Black Hole Shine | waveshine | a waveshine leaves a tiny black hole for 1 s that pulls foes in | bigger |
| 25 | Meteor Heel | spike (down-angle hit) | spiked foes trail fire and explode on the blast line or the stage | stronger |
| 26 | Perfect Guard | powershield | a powershield reflects a gold beam back at the attacker | stronger |

### Devil pool (`devil`, costs 1-2 stock slots)

| # | name | trigger | what you see and feel | stack |
|---|---|---|---|---|
| 27 | Brimstone Jab | jab hit | jabs fire a blood-red beam across the stage | bigger |
| 28 | Demon Wings | passive | black wings; one extra mid-air jump that fires a downward feather fan | more |
| 29 | Crimson Count | KO | every KO stacks a red sigil around you; each sigil adds a hit of fire to all your effects | more |
| 30 | Bloodlust | taking a hit | when hit, red tendrils lash out at the attacker | stronger |

### Angel pool (`angel`)

| # | name | trigger | what you see and feel | stack |
|---|---|---|---|---|
| 31 | Halo | passive | a halo; your projectiles turn holy white and pierce | bigger |
| 32 | Guardian Feather | passive | a feather orbits you and once per stock eats a KO-strength hit (it shatters loudly) | more |

### Curse pool (`curse`: comes with a drawback)

| # | name | trigger | what you see and feel | drawback |
|---|---|---|---|---|
| 33 | Haunted Shell | whiff | every whiff throws a homing ghost skull | foes also get a skull when *they* whiff |
| 34 | Glass Fist | any hit | hits crack the air with big glass shards | you take +25% knockback |

### Modifiers (no trigger; change other items by tag)

| # | name | pools | changes | what you see and feel |
|---|---|---|---|---|
| 35 | Homing Ink | item, shop | `projectile` | every projectile curves toward the nearest foe and leaves an ink trail |
| 36 | Fire Lace | item, devil | `burst`, `trail`, `projectile` | everything you make leaves embers; hits Burn |
| 37 | Ice Glass | item, shop | `projectile`, `spark` | projectiles turn to ice and shatter into 3 shards on hit |
| 38 | Chain Spark | boss, secret | `spark`, `projectile` | each hit arcs once to the next foe (a visible lightning line) |
| 39 | Twin Shot | shop, angel | `projectile`, `orbital` | doubles projectiles and orbitals, each at 60% |
| 40 | Gravity Well | secret, curse | `burst`, `launch` | bursts pull in before they push out (visible inward swirl) |
| 41 | Big Bang | devil | `burst`, `launch` | bursts x2 radius, screen shake, but -20% damage |
| 42 | Bouncy | secret | `projectile` | projectiles bounce off the stage and walls up to 3 times |

(42: the 8 modifiers are on top of 34 trigger items. Secret rooms draw from the modifiers marked `secret` plus #24.)

## 8. The first slice: 12 items

The slice proves: granular triggers, visible effects, stacking, and modifiers, with the fewest new engine hooks.

| item | why in the slice | new hook? |
|---|---|---|
| 1 Spark Ring | nair hit, the owner's example | last-move read |
| 2 Glide Wake | wavedash, already detected | no |
| 3 Shine Bell | shine, Fox/Falco | no (state poll) |
| 4 Hop Shot | short hop, a projectile for modifiers | `EVENT_SHORTHOP` |
| 5 Ledge Spikes | ledge grab | no |
| 6 Whiff Echo | whiff, the most "Isaac" idea | `EVENT_WHIFF` |
| 7 Roll Mines | roll | no |
| 12 Smash Quake | smash hit, a big readable burst | no |
| 15 Moon Drops | passive orbitals (reuses orbiter drawing) | no |
| 18 Coin Magnet | ties to shop / coins | no |
| 35 Homing Ink | modifier on projectiles | no |
| 36 Fire Lace | modifier on everything, reuses Burning | no |

Rooms in the slice: fights, the item room, the shop, the boss drop. Devil / angel / secret / curse come next.
Laser hit (#8) waits for the item-hit hook.

**From the current drives:**

| keep (as code or idea) | cut |
|---|---|
| Burning, Chilled statuses (as modifier outputs) | the charge-meter model, all 12 charge sources as a vocabulary |
| `gd_envoy_blast` launching hitbox (Nova, Comet, Spire shapes become Smash Quake, Ledge Spikes, Thunder Nair) | slots, levels, row replacing, reorder |
| tech detection (`tech_frame`, L-cancel hook, wavedash) | keystones (good ideas move to devil / angel items: Glass Core -> Glass Fist curse, Echo Oath -> Whiff Echo) |
| coins and the cash-out paying them; shop prices +25% a loop | Heat as a reward multiplier and chains (Heat can stay as the post-fx mood only, or go) |
| drops on the floor and their pickup (now pedestals) | per-fight picks and banked picks; KO drops |
| foe drives on elites/bosses (renamed foe items, same code) | Volley, Inferno, Grand Finale (Grand Finale -> Smash Quake idea) |
| loop scaling, waves, Petey, stage switches | the 20-frame fire queue |

## 9. What existing code is reused

| need | reuse |
|---|---|
| deterministic state, hashing, record | `envoy_sim` `St` (`#[repr(C)]`, `gd_hash_word`), run record (bump to v9, items replace drives) |
| item table | `envoy_data` `build.rs` + `pool_gen.rs` pattern and its lint (`spec.rs`): the lint now checks every item has a trigger, a visual, a stack rule and a cap |
| triggers | `EVENTS[slot]` + `gd_envoy_events`, `ENVOY_LCANCEL` pattern, `tech_frame`, `gd_envoy_attacker`, `petey::HITS` |
| hitboxes | `gd_envoy_blast` + `envoy::tick` (the owner never hits teammates); `gd_envoy_percent` for chip damage |
| statuses | Burning / Chilled / Marked on every match slot, `gd_envoy_speed` |
| visuals | `envoy-fx.md` world layer (materials, trails, dissolves, bloom), `drive_draw::SHOW` becomes the item / projectile list; the orbiter drawing; Brawl sounds |
| pedestal pickup | the drop code (floor points, owner colour, sweep-up) |
| shop | `shop_frame` and `envoy_ui/shop.rs` (stock becomes items, no level-ups) |
| map screen | the road screen (`V_ROAD_*`, P1 cursor / P2 flag) grown into a floor map |
| room fights | `room_waves`, `make_plan`, `foe_roll`, minion spawn (`gd_envoy_enemy_spawn`), Petey, stage switches |
| co-op | per-player blocks in the view, both-players-pick freeze (now: both walk to pedestals) |
| balance | `tools/rust-port/envoy/sim.py` (policies x seeds), kblog, the foe blast cap |
| pause UI | `envoy_ui` kit (`kit.rs`) and the picker's card rendering for the icon strip tooltips |

New: the item effect functions, the projectile pool, five engine event bits and the last-move read, the floor map
generator, special-room scenes (pedestal, devil door), the pause strip, item icons.

## 10. Fights 2: gauntlets, live stage transitions, crowd knockback (2026-11-13, built)

Owner, after playing the items slice: "Maybe more 1v1s, they can be back to back. Different characters are good. We
gotta find a better way to do stage switches. They should be less instant and have more of a effect live while we
are fighting transitioning across the stages maybe over a few seconds. If we are doing a multienemy fight (1v3 or
more ...), the 3+ enemies should have higher knockback scaling so they get knocked out and KOd easier."

### 10.1 The room mix and gauntlets

One table, `ROOM_MIX` in `envoy_sim` (per fight of a loop, per loop 1 / 2 / 3+: foes on the field, waves):

| room | loop 1 | loop 2 | loop 3+ | what it is |
|---|---|---|---|---|
| 1 | 1 foe x 2 | 1 x 3 | 1 x 3 | the opening gauntlet |
| 2 | 3 foes x 1 | 3 x 1 | 3 x 2 | the crowd (1v3, crowd knockback) |
| 3 | 1 foe x 3 | 1 x 3 | 1 x 4 | the elite gauntlet: plain duels, the elite in the last |
| boss | 1 | 1 | 1 | Petey / the stand-in |

So a loop is 6-8 duels and one crowd fight before the boss (it was 1 duel, 2 two-foe waves, 2 elite-room waves).
Co-op adds one foe to a duel or pack room, as before (a gauntlet of 2v2 duels; the crowd is 2v2).

- **A gauntlet** is a 1-foe room with more than one wave. The KO'd foe is parked (its stocks set back) and, when its
  KO countdown ends, comes back as the next character on the respawn platform: no results screen, no menu, the
  next duel is on about two seconds after the KO (the retail rebirth). Each duel is a different character.
- **Faces.** Wave 1 and wave 2 characters ride the plan as before (P_CPU_CK, P_WAVE_CK); waves 3-5 of foe rows 0-1
  ride a new queue (plan words 44-49, `P_GAUNT_CK`). The engine preloads every queued character with the match's
  files (`Player_80031CB0`) and pops the next one into `ENVOY_WAVE_CK` after each respawn, so no character file is
  read mid-match. The sim's `pick_face` never repeats a face within a room and prefers characters not among the
  run's last ten foes (`S.seen`, hashed state); it falls back to any unused face once the cast runs out.
- **The elite** arrives in the elite room's last wave (plan word 50, `P_ELITE_WAVE`); the duels before it are plain
  foes with their own stocks and numbers.

### 10.2 Live stage transitions

The freeze-and-cut (a 90-frame freeze, a wipe covering the screen, the switch under it) is replaced in Envoy by a
**rift sweep** with the fight running:

1. **Request** (frame 0): the same triggers as before (a new wave, an elite break, the Shifting road), the same caps.
   No freeze, no pause: tags, strikes and attacks carry on.
2. **The sweep** (frames 0-96): a bright sheared band in the destination stage's colour crosses the screen from the
   left. The side it has passed is veiled in the destination's colour (<= 20% alpha); a small caption reads
   "SHIFTING TO <stage>". The player sees the new stage coming and where it has "reached".
3. **The hand-off** (frame 96, the band's core over the middle): the engine swaps stage visuals and collision in one
   step between logic frames (`stage_hop::switch_to_ex(.., live)`). The core flares wide for ~8 frames with one
   soft flash and a ring (one flash, under the 3-a-second rule), which covers the one-frame swap.
4. **The settle** (frames 96-180): the band runs on off the right edge; the side it has not yet reached carries the
   old stage's colour veil, thinning out, so the old stage reads as being wiped away. Caption: "NOW ON <stage>".

Total 3 s (LIVE_LEN 180, LIVE_AT 96 in the sim). Why this one: a single live stage is rendered at a time (the engine
has no second stage scene to split the screen with), so a true two-stage split would need a render-target capture
in melee_gx; the rift gives the same read (one stage giving way to the other across the screen, over seconds) with
one honest swap frame hidden under the brightest moment, and it never takes control away.

**Fighters at the hand-off (Fights 3, owner: "try harder to maintain player positions ... rather than resetting them";
"have fallbacks if a player's position is ... between, or lower than the main platform").** `stage_hop::live_place`:
nobody is sent to a spawn point. Before the teardown every fighter's place and the old main floor (its y under x = 0
and how far it runs each way, probed with `mpCheckFloor`) are recorded; after the install:
- on or above the main floor and within its span: proportional x across the span, the same height over the floor;
  action, facing and momentum kept; a grounded fighter is set on the new floor under it (same action);
- beside the stage (past a ledge) or under the main floor's height: re-anchored to the nearest ledge of the new stage
  with the same offset (dx outward, dy below), so a recovery stays the same recovery;
- hanging on a ledge (the cliff states): the same offset from the same side's ledge, facing in, still, falling into
  the grab box (it re-grabs or gets up at once);
- a mapped point inside the new geometry (a floor top above it with no ceiling between: the body, not the pit under
  it): a step of at most 12 up onto a floor when that is the shorter way (never for an under-stage or ledge case:
  never up through the stage), else slid outward horizontally until free (at most 80);
- the point is kept inside the new blast box; everyone is intangible for 20 frames; no forced float, no reset.
The live check (items_check row 8) fails on any spawn-point reset, a push past the mapped point over 12 (80 for a
slide out of the body), or a run without a ledge-hang and an under-stage case (the test knob
`MELEE_ENVOY_LIVE_PROBE=1` takes P1 as hanging on the right ledge and P2 as under the stage at every other hand-off).
The rift platform under a fighter over a pit was not needed: proportional mapping never moves an onstage fighter
over a pit, and an offstage one keeps the recovery it had.

**Rollback and zero lag.** The transition is game state: `tag_swap::LIVE` (a plain static the snapshot covers),
advanced in `tag_swap::tick` between logic frames; the hand-off frame is a pure function of the record, so a
rollback across any frame of it resimulates the same frames. The destination is known when the fight is planned
(plan word 51, `P_STAGE_NEXT`), and the plan hint sends its stage file to the asset worker then, and again at the
request (`STAGE_HINT`), so the hand-off reads a prepared file and never touches the disc. The presentation only
reads `gd_ui_stage_live`.

The freeze path stays for the stage_switch mod and behind the test knob `freezeshift`.

**Stale free fixed.** `grZakoGenerator_StageSlotRelease` freed `lbl_8049F030.x4` on every hop, but that static
outlives a match (retail never clears it): after a match on a stage with a zako generator, a later hop freed a
pointer into the dead scene's heap. The release now frees only the table the live stage made (`ZAKO_LIVE`, set by
`grZakoGenerator_801CAE04`, cleared by every stage init `Ground_801BFFB0`); retail state is untouched.

**Measured (2026-11-13 20:31, quiet machine, hidden window, MELEE_PROFILE=1, four-five live hand-offs (hand-off ticks 2.4-4.4 ms on 85a02a1b2), after merging
port/diverge1's pipeline warm list and running `pipe_seed.sh`).** Hand-off ticks 1.9-7.0 ms with 0 pipelines created
on them; no hand-off frame over 16.7 ms (the only over-budget frames were the first frames of the match and the CSS
after it). Before the warm list (fights2): 59-65 ms per hand-off, 32-43 ms of it GPU pipeline creation.

### 10.3 Crowd knockback

In a fight with 3 or more foes (`CROWD_FROM`), every foe hit is scaled by `CROWD_KB` = x1.35 knockback, +0.10 for
each foe past three (`CROWD_KB_STEP`), at most x1.5 (`CROWD_KB_CAP`, also the engine's clamp in `gd_envoy_kb_scale`).
Applied in the sim's take-damage post hook to foes only (never a player, never the boss), Envoy only. Logged once a
fight (`envoy: crowd fight, foes*10000 + knockback x1000`) and per hit (`envoy: crowd knockback, slot*10000 + x1000`);
the HUD's room line says "Crowd". With four ports and a human, 1v3 is the most today; the Subspace enemies are not
fighters (they have their own HP), so they do not count toward the crowd.

### 10.4 Checks

`tools/rust-port/envoy/items_check.sh` rows 7-9: `gauntlet` (fight 1: a second duel with a new face, no repeat),
`live` (the torture knob's switches as live transitions: hand-off, carry, no KO caused, no freeze), `crowd` (fight 2:
x1350 on foe hits, none on a player); each a synctest at depth 4 and 7 with 0 mismatches.

## 11. Slice 2: rooms, 20 more items, new triggers, co-op stocks, the act map (2026-11-13/14, built)

Code: `envoy_sim/src/rooms.rs` (stocks, doors, the secret wall, the act map), `orbs.rs` (orbitals), `items.rs` (the
effects), `envoy_data/src/items.rs` (the table: 32 items), `envoy_data/src/view.rs` ("slice 2" words). Checks:
`items_check.sh` rows 10-19 (below).

### 11.1 Stocks, per player (co-op too)

Each player has their own stock limit (start 4, hard cap 8) and stocks (`St::lims`, `St::pools`; the run record keeps
both as P1 | P2 << 8, so a slice-1 record still loads: both players take its one limit). Co-op was a shared pool of 5
(a stock lost by one was lost by both); now each player loses their own, the fight goes on while either has a stock,
and a co-op player who ran out comes back with one stock in the next fight (the partner carried it). A devil deal,
the shop's stock and stock slot and Spare Heart act on that one player. The campfire refills each to their own limit.

### 11.2 Rooms

- **Devil deal** (red door, after the boss, sometimes). Chance: 35%, +15% with no player stock lost on the floor,
  -20% per deal taken in the run (`rooms::door_pct`; the map shows it on the door node). It offers 1 or 2 devil-pool
  items per player (seeded). Each costs its item's `slots` (1 or 2) of the buyer's stock limit: the limit drops, stocks
  above it are lost, and a pedestal that would take the last slot is locked (drawn crossed). Walking in buys. The room
  is optional: D-pad down leaves it (or it ends when every remaining deal is locked); unbought deals never fly in.
- **Angel room** (white door). It replaces the devil door when no deal has been taken this run, half the time a door
  opens (`ANGEL_PCT`). One free item from the angel pool (strong: Halo, Guardian Feather, Spare Heart, Twin Shot).
- **Secret room.** One per act: a cracked wall stands in one fight node (seeded node; marked on the map). Any attack
  of a player's that reaches it (within 18 of it, 26 high) cracks it; 3 hits break it. After that fight the secret room
  opens in the held match: one secret-pool item, or a choice of two (seeded).
- **Campfire**: a room node (orange flame). Refills each player's stocks to their own limit; never raises it.
- **Shop**: a room node. The stock refill as before (the buyer only) and, in 30% of shops, a **stock slot** (+1 limit,
  +1 stock) for 160 coins (+40 a loop): the SHOP_SLOT row.
- **Spare Heart** (item 43, angel and secret pools): on pickup +1 stock limit and +1 stock, to 8.

### 11.3 No timers on any choice

Owner: "We shouldn't be rushed on making our build." No countdown anywhere: pedestals (item room, boss drop, devil,
angel, secret), the room choice on the map, the shop. The held match waits (`HOLD_FOREVER`) until every player has
taken their item or left; the shop until each player picks Leave. All foes are already KO'd while a room is open, so
the players are safe while they walk to a pedestal. A pedestal whose owner is out of stocks flies to them at once.
The CPU-driven checks use the test knob `autotake` (the old wait, then the cursor's choice); it is never on in play.

### 11.4 The act map (Slay the Spire style; replaces road1's three roads and the first floor map)

Owner: "the map/path choice ... more like the map on slay the spire ... preplanning your path across the map, a few
map tiles in advance."

- One map per loop (an act), made only from the run seed and the loop (`rooms::node_kind`, `rooms::links`): the
  same on both co-op screens, both netplay peers and on replay.
- Rows from the bottom, three lanes each: fight rows 0, 2, 4 (the act's fights 1-3), room rows 1, 3, 5, then the
  boss (row 6) and the door after it (row 7). Each node leads to the same lane above and, seeded, to one neighbour;
  two edges never cross.
- Generation rules (StS's, sized to this act): the first fights are duels or a crowd, never an elite; the first room
  row is item rooms only (no shop or campfire that early); elites in the middle and late fight rows; the middle room
  row is one shop, one campfire, one item room (so never two shops on a path); the row before the boss is all
  campfires. The fight node's kind picks the fight (duel gauntlet / crowd / elite gauntlet: `rooms::template`).
- Choosing: the next fight node on the road screen (camp, between fights, and before loop 1's first fight): P1's
  left / right walks the reachable nodes (road1's pass-through of the stick and D-pad is kept), P2 plants a wish flag
  (no effect). The planned path ahead (the cursor's lane carried up) is marked, so the route is planned rows ahead.
  The room above a won fight is chosen in the held match the same way (A takes it): the item room's pedestals, the
  campfire, or the shop at the stop.
- The pause Run page, the camp and the in-match room choice draw the same map from the view (V_MAP*).
- Road1's Steep / Shifting roads are retired with the map (their foe-drive and stage-shift twists may come back as
  node modifiers).

### 11.5 New triggers

| trigger | how it is read |
|---|---|
| whiff (fixed) | an attack state ends with no hit **and no shield contact**: the engine's new shield-hit bit (`EVENT_SHIELDHIT`, set in `ftColl_80076CBC` on the attacker) marks the attack as connected; the log tells a shielded attack apart |
| crouch cancel | `EVENT_CC`, set where `ftCo_Damage_CalcKnockback` cuts a crouching fighter's knockback |
| powershield | the fighter enters GuardReflect (only a powershield does) |
| laser hit | the victim's damage source is a Fox / Falco laser item (`gd_envoy_hit_item`: kinds 54, 55) |
| jab hit, taking a hit | the attacker's Attack11-13 state at the hit; the take-damage records |
| while airdodge / nair / dash / shield | the stance orbs' states (11.7) |

### 11.6 The Spark Ring bar (a rule for every item)

Owner: "the spark with the effect on nair is so sick, and visually distinct, and it has a sound effect ... The moon
balls ... dont play a sound or do knockback so its hard to tell if they are working." So every item hit must have
(a) a visually distinct effect, (b) its own sound from the Melee disc (so it works without Brawl), and (c) readable
impact: a hit spark on the foe, and a flinch or knockback where it fits.

How: every item hit now lands as an engine hit (`gd_envoy_blast`) whose capsule carries the item's own hit element
(the engine's spark: normal, fire, electric, slash) and Melee hit-sound family and severity (`gd_envoy_blast_look`,
table `items::hit_look`). Light hits (sparks, moons, arcs, shards, feathers) use a small blast with a light flinch
(`TAP_KBG` 18, `TAP_BKB` 22) instead of a silent percent nick (the nick stays only as the fallback when no hit capsule
is free). Audit of the first 12: Spark Ring's sparks were silent nicks with a Brawl-only sound (fixed: electric
spark, Melee sound, flinch); Moon Drops were silent nicks (fixed, see 11.7); Glide Wake, Shine Bell, Hop Shot, Ledge
Spikes, Whiff Echo, Roll Mines and Smash Quake were real hits but all with the same punch sound and spark (each has
its own now); Coin Magnet hits nothing (its coin shower and pickup sound stay); Homing Ink and Fire Lace are modifiers.

### 11.7 Orbitals and stance orbs

Owner-approved rework of every orbital: each orb is a real small hit (light knockback, hitstun and hitlag, its spark
and sound). A foe's attack that comes within 11 of an orb, or a foe projectile that touches it, shatters it (the
projectile is removed: the orb blocked it); it respawns after 4 s (`ORB_RESPAWN` 240) or at once when its owner lands
a hit. Stacks add orbs and widen the orbit a little. One family hits a given foe at most every 30 frames.

Stance orbs exist only while the fighter is in one movement state and take their shape from it:

| # | item | while | shape |
|---|---|---|---|
| 44 | Dodge Orb | airdodge (and the wavedash slide after it, 14 frames) | leads in the dodge's direction, side by side per copy |
| 45 | Nair Ring | nair | 3 + copies orbs ringing the fighter, spinning fast |
| 46 | Dash Orb | dash / run | leads in front, one more ahead per copy |
| 47 | Shield Orbs | shield (incl. powershield) | 2 + copies orbs circling the shield; they shatter on blocks |

### 11.8 The new items (20)

| # | item | pool | does | stack |
|---|---|---|---|---|
| 8 | Laser Prism | item (Fox/Falco pairs) | a laser that hits splits into three thin bolts fanning past the target | more |
| 21 | Crouch Thorns | shop | a crouch-cancelled hit grows thorns from the back that launch everyone close | stronger |
| 26 | Perfect Guard | boss | a powershield fires a gold beam toward the nearest foe | stronger |
| 27 | Brimstone Jab | devil (2 slots) | a jab that hits fires a blood-red beam across the stage | bigger |
| 28 | Demon Wings | devil (1) | one more air jump per copy; using it rains a fan of feathers | more |
| 29 | Crimson Count | devil (1) | each KO adds a red sigil for the fight (3 a copy); every item hit burns more per sigil | more |
| 30 | Bloodlust | devil (1) | when hit, red tendrils lash the attacker 6 frames later | stronger |
| 31 | Halo | angel | modifier: shots turn holy white and pierce (Hop Shot's star flies on) | bigger |
| 32 | Guardian Feather | angel | once a stock per copy, a hit that would take you past 100% (8%+) is eaten whole | more |
| 37 | Ice Glass | item, shop | modifier: shots shatter into three ice shards on hit and chill | stronger |
| 38 | Chain Spark | boss, secret | modifier: every item hit arcs to the next foe (a lightning line; one more arc a copy) | more |
| 39 | Twin Shot | shop, angel | modifier: every shot comes with a twin (60%); moons doubled | more |
| 40 | Gravity Well | secret | modifier: bursts pull foes in for 16 frames, then burst | bigger |
| 41 | Big Bang | devil (1) | modifier: bursts twice as big with a screen quake, damage -20% | bigger |
| 42 | Bouncy | secret | modifier: shots bounce off the floor and its edges, 3 times a copy | more |
| 43 | Spare Heart | angel, secret | +1 stock limit, +1 stock (to 8) | more |
| 44-47 | stance orbs | item / shop / secret | see 11.7 | more |

Fire Lace joins the devil pool (1 slot). Items stack without limit (each item's own growth is capped).

### 11.9 Item language (keywords)

Owner: "a really distinct item language so certain words or phrases are reserved and bolded or coloured ... like
pokemon or magic the gathering". Item text is data: tokens in `envoy_data` with a fixed keyword vocabulary in
categories (timing ON / WHEN / WHILE / AFTER; conditions IF; moves NAIR ... JAB, SMASH, SPECIAL, GRAB, THROW, SHINE,
LASER; tech WAVEDASH, SHORT HOP, L-CANCEL, TECH, LEDGE GRAB, ROLL, AIRDODGE, DASH, CROUCH CANCEL, POWERSHIELD;
outcomes HIT, WHIFF, SHIELD, KO, TAKE HIT; effects PROJECTILE, BURST, TRAIL, AURA, ORB/ORBIT, BEAM; statuses BURN,
CHILL, MARK; stacking STACK, PER STACK), each category its own style and colour, one card template for every item,
and a glossary page. Each trigger id maps 1:1 to its keyword words in `envoy_data`, so the mechanics and the words
cannot drift. (Built in the UI lane, port/envoy-slice2-ui.)

### 11.10 Checks (items_check.sh, synctest at depth 4 and 7, 0 mismatches)

10 shop (map shop, item + stock slot), 11 devil (deal paid in slots), 12 angel, 13 secret (wall hit, broken, room),
14 whiff (true whiff vs shielded), 15 triggers (crouch cancel, powershield, laser, jab, Bloodlust), 16 coopstock (the
deal costs only the buyer), 17 map (fight node and room taken), 18 campfire (refill to each limit), 19 orbs (hits,
shatter; stance-orb hits and blocks reported). ready_checks.sh `fresh`: two New runs roll different salts and foes.

### 11.11 Pickups take a button; items show only when used (owner, Nov 13)

- **Pickups as in Melee.** Walking past an item never takes it. In range (16 across, 28 up) the UI shows an "A"
  prompt over the pedestal (`V_PED_NEAR`, per player); A on the ground, Z / L / R (the airdodge-grab) in the air takes
  it. Every pedestal (item room, boss drop, devil, angel, secret) and so Spare Heart and every drop follows it. The
  read is the sim's pad edge, so it is deterministic and rolled back with the rest. Test knob `autotake` (CPU checks)
  still takes on touch.
- **Items reveal themselves during their use** ("We gotta have some ways to show off our bling"). In a fight an
  item is invisible until it procs; then its hit look and sound play (11.6). Moon Drops are out only during the
  owner's attack (their hit window) and 40 frames after any proc; the stance orbs only in their state. The worn looks
  (sigils, Guardian Feather, Halo, Demon Wings) show for 45 frames after a proc. `V_PROC_AGE` lets the UI fade them.
- **Showing off a build**: the camp party cards and the pause build page (every item, icon and card); the build
  flourish (`V_FLOURISH`, 60 frames) at each fight's start and when a room opens after a win: the held items flash
  round the fighter; a subtle per-player aura tint from the rarest item held (`V_AURA`: devil/angel/secret, then
  boss, then quality); the run-end summary lists the build. All are view words drawn by the UI: no sim cost.

### 11.12 Body-group glow (spec; follow-up, not in slice 2)

The C-build roguelite (pc/scripts/examples/roguelite/visuals.lua, bindings.lua) tinted fighter draw objects by role:
an offline "part lab" measured each costume's DObjs over several poses and bound them to three body roles (assault:
arms and hands, traversal: legs and feet, guard: torso and head), and `gd.dobj_tint(port, dobj, colour)` set a solid
colour per role in three phases (equipped, charging, ready). Envoy's version, in spirit:

- Each item family owns a body group: hits from the hands (Spark Ring, Brimstone Jab, Chain Spark, Laser Prism) the
  arms and fists; movement (Glide Wake, Hop Shot, Dash Orb, Dodge Orb, Demon Wings) the legs; defence (Shine Bell,
  Perfect Guard, Shield Orbs, Guardian Feather, Halo) the torso; devil items a faint red at the head.
- In play it is a subtle tint (10-15% toward the item colour, the player's colour kept); while the item procs, its
  group flares for 12 frames (an additive glow).
- Groups come from the joint tree, not from guessed mesh names: each DObj takes the body group of its joint's
  nearest named ancestor in the fighter's bone table (hands/arms, legs/feet, spine/chest, head), cached per costume
  at load, so it costs nothing a frame.
- The Rust renderer has no per-DObj tint yet (`gd.dobj_tint` was a C-build script host call). Needed: a per-player
  per-group colour multiply + add in the fighter material path, fed from the view (`V_AURA` / `V_PROC_AGE` plus a
  group mask), deterministic (presentation only, never read by the sim). Estimated well under 0.1 ms a frame.

### 11.13 Effects, check notes (Nov 14)

- The vfx lane (port/envoy-vfx, notes/envoy-vfx.md) is merged: orbs (each family its own shader look; shattered as
  faint ice), orb shatter (impact + ice) and block (hex dome + pop), and the devil / angel doors are vfx quads; the
  other slice 2 shapes go through lib.rs's shim onto the same quads. Big Bang's shake is a screen-space shockwave.
- Scripted flows (camp_run, ready_checks) carry the test knob `autotake` now: rooms wait for the player, and the pad
  scripts do not walk to pedestals or pick rooms.
- items_check row 15: crouch cancel, powershield and jab hits depend on what the CPUs do and are reported, not
  required (a pad script forcing each is a follow-up); Bloodlust and the laser hit are required/seen.
- The pause Words glossary is off the L / R cycle (up on the tab row opens it), so the four main pages keep their order.

## Open questions for the owner

1. Decided (owner, 2026-11-13): devil deals cost stock slots; Isaac-style stock limit + campfire refills.
2. Secret room trigger: waveshine into a marked wall, or something per stage?
3. Does Heat stay as a look only, or go entirely?
