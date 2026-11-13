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
| `price` | devil / curse items: `stock`, `maxhp`, `curse:<id>`; shop: coins from quality |

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
| devil deal | red door | after the boss, sometimes: 1-2 items, each costs a stock or max HP (a lower KO ceiling, see below) | `devil` | chance |
| angel room | white door | replaces the devil door if you took no devil deal this run: one free strong item | `angel` | chance |
| secret room | none (hidden) | revealed by a technique on a marked wall-ledge, e.g. waveshine into the wall, or 3 ledge grabs in 2 s | `secret` | 0-1 |
| curse room | purple | an item plus a curse (a run-long drawback item) | `curse` | 0-1 |
| challenge (optional) | crossed swords | a harder fight; win for an `item`-pool item | `item` | 0-1 |

Devil chance: 35% after floor 1's boss, +15% if you lost no stock on the floor, -20% per devil deal taken; an angel
roll replaces it when you have taken none. Max HP in a stock fight means a **KO ceiling**: you are KO'd at a
percent that starts at 999% and a devil deal drops it by 40% (to 120% at worst). Honest, visible, Melee-native.

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
  (two pedestals); shop items are bought by whoever walks in, from the shared purse; a devil deal's stock comes out of
  the shared pool as stocks already do, the KO ceiling only on the buyer. A few items have a co-op line ("also on
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

### Devil pool (`devil`, costs a stock or a KO-ceiling cut)

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

## Open questions for the owner

1. KO ceiling as "max HP" for devil deals, or stocks only?
2. Secret room trigger: waveshine into a marked wall, or something per stage?
3. Does Heat stay as a look only, or go entirely?
