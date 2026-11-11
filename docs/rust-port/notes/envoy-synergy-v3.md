# Envoy synergy architecture, V3: one hit, one combo, drives that feed each other (design, 2026-11-10)

Design only. Nothing here was built or run. Branch `port/synergy3`, worktree `C:/melee-wt/synergy3`, off `pub/port/v2`
(`288627e37`). By Fable. It revises `envoy-core-v2.md` (port/envoycore2) sections 2.1, 2.3-2.5, 3.3, 4 and 6, and
builds on `envoy-drives.md` (the V2 build note, this branch). Everything else in those notes stands.

Tags: **[R]** read from the repo (cited), **[W]** read on the web this session (section 14), **[K]** the author's own
knowledge of the game, not fetched this session, **[I]** inferred, **[J]** judgement, **[U]** unverified,
**[D]** an owner decision already taken (memory `envoy-lab-answers-2026-11-09.md`), **[O]** the owner's own words.

## 0. One page

**Why.** The owner played V2 (20 drives, 6 keystones) and said: "the games still feel pretty shallow and un-unique"
[O]. The agreed diagnosis: every drive is a standalone "every N hits do X". Nothing is shared between drives, a fire
almost never causes another fire, and the numbers only add. Of V2's 20 drives, 14 neither read a word nor act on
another drive [R `pool.toml`: three read a word; Echo, Relay and Surge act on the set]; the four words are nearly the
only edges.

**What the three reference games share** (section 1): one small thing that every item touches (the hit, the tear, the
scored card), numbers that multiply instead of adding, effects that count as triggers for other effects, and a
position or a set that makes the same items behave differently.

**The design in twelve lines.**

- **The hit is the shared thing.** Every hit the player lands is one record in the sim. Drives either shape it
  (element, extra hit, shield damage, hitlag, size) or react to it.
- **The combo is the score.** A combo has two numbers on the HUD: **Base** (every hit adds) and **Mult** (drives add
  to it or multiply it). Their product is **Power**.
- **Mult makes drives hit harder during the combo.** When the combo ends it **cashes out**: a burst sized by Power,
  coins, and charge for the active drive.
- **Fires are hits.** A drive's fire counts as a weaker hit for the drives **to its right**, by its **coefficient**.
  That is a proc chain; position and the coefficient are the brakes.
- **Slot order matters.** Mult is applied left to right, chains run left to right, and two drives act on a neighbour.
- **Technique is a resource.** L-cancel, wavedash, tech, ledge grab, shield drop, power shield and edgeguard are
  conditions and charge sources. All but L-cancel and the meteor are detectable today by polling action states.
- **Three of a family on one fighter is a set**: a visible transformation with its own rule.
- **One active drive per player on D-pad right**, charged by cashed-out Power. D-pad down stays the tag, D-pad up
  stays the taunt.
- **Scaling drives** grow for the whole run. **Keystones are the devil deals**: eight, each bends the formula.
- **Coins** come from Power and style, and are spent at the rest stage.
- **Three rarity tiers** and **pools per encounter kind**. Multipliers live in the upper tiers.
- **V3 is a slice**: the combo, chains, order, four techniques, 16 drives, 3 keystones, 1 active, a plain shop
  screen, in the 5-fight Short run. Its question is "do two runs feel different?".

Kept as decided [D]: drives per fighter; floor drops plus a pick of three; a duplicate levels a drive (to 3); no bag;
keystones at the start and after the first two bosses; fires are loud; only timed drives launch; elites and bosses
carry drives; co-op drops per player; content-only unlocks; NG+; the Brawl disc required; the drive models.

## 1. How the reference games get depth

Mechanisms in my own words. No item text is reproduced.

### 1.1 Risk of Rain 2

| mechanism | how it works | what Envoy takes |
|---|---|---|
| on-hit items | most items react to "a hit landed", so every item shares one trigger | the hit record (2) |
| proc coefficient | each attack and each item effect has a multiplier, usually 0 to 1, on how likely or how strong on-hit effects are from it. Fast weak attacks get low values, heavy ones high. An effect at 0 never triggers anything [W1] | every hit and fire has a coefficient (4) |
| proc chains | an item's effect is itself a hit, so it can trigger other items at its own coefficient: a full-strength missile can trigger chain lightning at the full rate, while the lightning (0.2) triggers the missile at a fifth [W2]. A chain cannot trigger the same item twice [K] | fires feed drives, with a brake (4) |
| stacking curves | linear (each copy adds the same), hyperbolic (each copy adds less, never reaching 100%; used for block and control chances so they cannot become certain), exponential (each copy multiplies) [W3] | levels are linear; a cap wherever a number could reach "always" (3.4) |
| lunar items | a strong effect with a built-in drawback, bought with a separate currency; some stack exponentially on both sides [W3] | keystones (7.2) |
| design intent | Hopoo: each item starts from a specific purpose, and balance across tiers is never finished [W4] | every drive names the job it does in a build (8) |

### 1.2 The Binding of Isaac

| mechanism | how it works | what Envoy takes |
|---|---|---|
| the tear | nearly every offensive item changes the player's one attack: its damage, rate, shape, path or what it does on contact. Changes combine instead of replacing each other [K]. McMillen: about 70% of items stack, and every item affects the character in some way [W5] | shapers change the player's own hit and all apply at once (2.2) |
| visible growth | items were designed so the character visibly changes over a run [W5] | sets transform the fighter (6.2) |
| transformations | three items from a themed group give a bonus and a new look; several can be held at once [W6] | sets (6.2) |
| active items | one held at a time; it charges by clearing rooms (1 to 12) or by time, then is spent by a button [W7] | one active per player, charged by Power (6.3) |
| devil deals | after a boss, a room may offer strong items paid in permanent health; the chance depends on how the floor was played [W8] | keystone prices; the stock pact (7.3) |
| item pools | each room type draws from its own pool; an item seen is removed or made less likely; an empty pool falls back to the main one [W9] | pools per encounter kind (7.5) |
| item quality | items carry a quality from 0 to 4 [W7] | three rarity tiers (7.4) |

### 1.3 Balatro

| mechanism | how it works | what Envoy takes |
|---|---|---|
| chips x mult | a hand scores chips times mult. Every source is flat chips, flat mult or a multiplier of mult [W10] | Base x Mult = Power (3) |
| +Mult before xMult | effects resolve left to right, so adding before multiplying scores more: in the wiki's example the same two jokers score 640 one way and 480 the other [W11] | slot order (5) |
| activation sequence | a fixed order: each played card scores, then per-card joker effects left to right, then retriggers, then held cards, then independent jokers [W11] | the scoring step is written down as a fixed order (3.3) |
| retriggers | an effect that makes a card score again, repeating everything attached to it [W10][W11] | Echo, Shocked, Echo Oath (5, 8) |
| scaling jokers | start weak and grow from events over the run [W10] | scaling drives (7.1) |
| copy jokers | reproduce a neighbour's ability, so position decides what they are [W10] | Mirror, Echo (5) |
| editions | a modifier on any joker: flat chips, flat mult, a multiplier, or "takes no slot" [W10] | editions, after the slice (7.4) |
| slot limit | five jokers, so every pick is also a cut [W10] | 5 slots per fighter (5.3) |
| economy | pay per round, pay for unused hands, interest on money held up to a cap; spent in a shop between rounds [W12] | coins and the shop (7.3) |
| origin | jokers were added as passive items to give runs unpredictability, with Luck be a Landlord's scoring as the largest influence [W13] | the combo as a score, not only as damage |

### 1.4 Two others

- **Hades.** Each god's boons carry a theme, a boon offer is three choices [W14], and a duo boon appears only when
  the player already holds a qualifying boon from each of two gods [K]. Envoy takes: two **duo drives** that the
  offer shows only when the fighter holds a drive from each of two families (7.5).
- **Slay the Spire.** Relics are run-long passives from elites, bosses and shops; the boss tier trades a strong
  effect for a drawback [K]. Balance was driven by pick-rate and win-rate metrics from play [W15]. Envoy takes: the
  elite drops what it carried (7.5), and `sim.py` measures build spread before a playtest (12).

## 2. The shared primitive: the hit

### 2.1 The hit record

Every time damage from a player's side lands on a foe, the sim builds one record. Today's take-damage post hook
already runs at that moment and knows the victim, the damage and the attacker [R `envoy_sim` `on_damage`,
`envoy::attacker`].

| field | values | where it comes from |
|---|---|---|
| `owner` | the player's set in play | `gd_envoy_attacker` [R] |
| `victim` | match slot | the hook's fighter [R] |
| `src` | the fighter's move, or a drive slot (a fire) | the attacker's action state [R, polled today for aerials]; fires are raised by the sim itself |
| `weight` | **Light** (under 4%), **Solid** (4% to under 12%), **Heavy** (12% and over) | the hook's damage [R] |
| `coef` | x1000. A move: 1000. A fire: the drive's coefficient times its parent's | pool data (4.1) |
| `depth` | 0 for a move; 1-4 along a chain | the sim |
| `tags` | AERIAL, SMASH, SPECIAL, THROW, OFFSTAGE, METEOR, plus the technique marks of 2.3 | action states, the stage table, the event word (2.3) |

Weight is the answer to multi-hit moves. In V2 each hit of a drill charges a drive as much as a knee. In V3 a Light
hit is worth half a hit, a Solid one, a Heavy one and a half [J], so fast weak attacks and slow strong ones charge at
a similar pace. This is what the proc coefficient does for fast weapons in Risk of Rain 2 [W1].

### 2.2 Shapers: drives that change the hit

A shaper is passive. It does not fire. It changes a property of the player's own hits, and several shapers all apply
to the same hit.

| property | what the player sees | engine seam | limit | status |
|---|---|---|---|---|
| **word on hit** | the hit sets Burning, Chilled, Marked or Shocked | the sim's status windows | per word | exists [R] |
| **bonus damage** | extra percent after the hit; the hit's own knockback is unchanged | the take-damage post hook | +100% of the hit | exists [R] |
| **extra hit** | a second, smaller hit at the same place a few frames later | percent plus a mark in the fire log; a blast when it should flinch [R `envoy::blast`] | one per move hit | exists |
| **element** | the engine's own fire, electric or ice hit effect and sound on the player's move | the hit's `element` is copied to the victim where a hit connects (`ftcoll.rs:510` fighter hits, `:840` item hits) [R]; a per-slot override read there | cosmetic plus the word. The ice element can freeze the victim [K], so it is allowed only on a cash-out finisher | new, S, [U] |
| **shield damage** | shields shrink faster; a break is an event | the capsule's shield-damage field (`x34`) where a hit meets a shield [I] | +100% | new, S, [U] |
| **hitlag** | a heavier freeze on impact | `dmg.x195c_hitlag_frames` after `ftCommon_CalcHitlag` (`fighter.rs:2912`) [R]; attacker and victim get the same extra frames | +6 frames | new, S, [U] |
| **size** | a slightly larger hitbox | `HitCapsule.scale` when the move creates it [R struct] | +25%, own moves only | new, M, [U]. It changes spacing, so it is not in the slice |
| **knockback** | the hit launches further | `dmg.kb_applied`, the field `gd_envoy_absorb` already zeroes [R] | only through Grand Finale (8.3); +50% | new, S, [I] |

One new import carries all the new rows: `gd_envoy_shape(slot, field, value)`. It writes a per-slot record in
`melee_modes::tu::envoy`, whose statics are covered by the snapshot [R, the module's header]. The sim writes it at
frame end from its own hashed state; the engine reads it at the seams above on the next frame. No float leaves the
sim: values are x1000 integers as V2's are [R `codes.rs`].

Rules that keep Melee readable:

- A shaper never changes the knockback of a hit that can combo. Only a keystone does (8.3).
- One element is shown per fighter: the leftmost element shaper's (order, 5). Every shaper's word still applies.
- Shapers act on the owner's moves and projectiles only, never on a teammate's.

### 2.3 Technique: what the port can see

The sim polls each fighter's action state every frame and already derives landings, jumps and aerial hits from it
[R `envoy_sim` lib.rs:117-123, 1529]. Technique is the same method: a state entered, and the state before it.

| technique | detection | state ids [R `melee_types/src/gen.rs`] | engine work |
|---|---|---|---|
| **L-cancel** | an aerial landing entered while the trigger timer is inside the window: the engine's own test is `x67F < xE4` in `ftCo_LandingAir_EnterWithLag` [R `ftCo_LandingAir.rs:65`] | LandingAirN-Lw 70-74 | **one event bit at that line, S.** The sim cannot read the timer: `gd_fighter_read` returns a fixed 20-word record without it [R `melee_modhost` lib.rs:536] |
| **wavedash / waveland** | LandingFallSpecial entered from EscapeAir. A wavedash if the jump squat was within 10 frames, else a waveland [J] | 43 from 236; KneeBend 24 | none (a hook row also exists: `action.LandingFallSpecial.enter` [R `hooks-v1.tsv`]) |
| **tech** | a Passive state entered | 199-204 (ground, forward, back, wall, wall jump, ceiling) | none |
| **ledge grab / ledge option** | CliffCatch entered; then a climb, attack, roll or jump | 252; 254-263 | none |
| **ledgedash** | CliffWait, then LandingFallSpecial within 30 frames [J] | 253, 43 | none |
| **shield drop** | Pass entered from a Guard state | 244 from 178-181 | none [I: the previous-state test] |
| **power shield** | GuardReflect entered | 182 | none [I] |
| **out of shield** | an attack, grab or jump within 4 frames of leaving Guard [J] | 178-181 | none |
| **dash dance** | Dash, Turn, Dash within 20 frames [J] | hook rows `action.Dash.enter`, `action.Turn.enter` exist [R] | none |
| **move variety** | the attacker's action state when the hit lands: a different attack state is a different move. The engine also keeps an attack id [R `ft_0763.rs:26`], finer but not exposed | the attack states | none |
| **taunt** | Appeal entered (D-pad up, `ftCo_AppealS.rs:76` [R]) | Appeal | none |
| **offstage hit (edgeguard)** | the victim's position is past a ledge or under the stage when the hit lands | positions [R `gd_fighter_get_pos_*`] | a table of ledge x and floor y per stage in the run's stage list (5 stages today [R lib.rs:115]) |
| **edgeguard KO** | a foe KO'd within 120 frames of an offstage hit without landing between [J] | the sim's KO detection [R] | none |
| **meteor** | a hit whose launch angle points down on an airborne foe | - | needs the hit's angle: one event word, S, [U] |
| **foe held** (for the combo, 3.1) | the foe is in a damage, down, tech, grabbed or thrown state | Damage 75-91, DamageFall 38, Down 183-198, Passive 199-204, Catch/Capture/Throw 212-243, ShieldBreak 205 on [I] | none |

Where a row needs engine work, the pattern is one word: `gd_envoy_events(slot) -> u32`, bits set by the engine at
the exact site during the frame, cleared at frame start, kept in the snapshot. Hook callbacks and polls run inside the
sim's frame, so a resimulation raises the same events [R core v2 2.7].

**The slice needs one small engine piece for technique**: the L-cancel bit. Wavedash, tech, ledge grab and the
offstage hit are polled.

Technique marks last a short time on the fighter: "L-cancelled" covers hits in the next 20 frames, "from the ledge"
2 seconds [J]. A drive reads a mark, so its rule can say "after an L-cancel" in a few words.

## 3. Layered maths: the combo

### 3.1 What a combo is

- A combo belongs to a **player**, not to a fighter or a foe. A tag does not end it (the tag is a frozen cinematic
  [D E13]); a pack's foes share it.
- It **starts** on a move hit. A drive's fire cannot start one.
- A foe is **held** while its state is in the "foe held" row of 2.3, or it is in hitlag.
- The combo has a timer. While any foe it has hit is held, the timer sits at **45 frames**. Otherwise it runs down.
  A move hit refills it. A hit on a shield adds nothing but sets the timer to at least 20 frames. Against a boss
  with no hitstun (Petey, the Hands) the refill is 90 frames [J].
- So a true combo, a tech chase and a tight string are all one combo; a reset to neutral ends it.
- It **ends** four ways:

| end | when | cash-out |
|---|---|---|
| **finished** | a foe in the combo is KO'd | x1.5 |
| **escaped** | the timer reaches 0 | x1 |
| **dropped** | the player takes a hit | x0.5 |
| **lost** | the player loses a stock | nothing |

### 3.2 The two numbers

| number | starts at | what raises it |
|---|---|---|
| **Base** | 0 | each move hit: Light 5, Solid 10, Heavy 15. Each fire: +2. `+Base` drives |
| **Mult** | x1.0 | `+Mult` drives add; `xMult` drives multiply. Stored x1000; a product rounds down |
| **Power** | - | Base x Mult, shown when the combo ends |

`+Mult` is common and small (+0.2 to +0.5 for an event). `xMult` is Rare or Mythic (x1.2 to x2). A multiplier that
triggers on a repeating event is what makes growth exponential in combo length: Crescendo multiplies by 1.5 on every
fifth hit, so a 15-hit combo carries x3.4 from that one drive before anything else.

### 3.3 The scoring step (fixed order, on every move hit)

1. The engine applies the hit as retail does, with the shapers of 2.2.
2. Base rises by the hit's weight, then by `+Base` drives, left to right.
3. Mult drives whose condition this hit meets apply, **left to right**.
4. Each on-hit trigger gains charge: weight x coefficient.
5. Full drives fire, left to right, 6 frames apart, as a chain (4).
6. The timer refills.

### 3.4 What Mult and Power do, and the caps

| output | rule | cap | why |
|---|---|---|---|
| **the player's own hits** | unchanged: the same percent and knockback as retail | - | combos must stay learnable; a move that launches differently every hit cannot be followed up |
| **a drive's fire during the combo** | its number x level scale x Mult | Mult counts up to **x4** for this; one fire deals at most 30% (a launching timed fire 40%) | the fifth Ember of a combo is visibly bigger than the first |
| **the cash-out burst** | Power / 10 percent to the last foe hit, percent only, no launch | 40% | the pay-off the two numbers were building to |
| **overflow** | Power the caps did not use | none | it becomes coins (7.3), so a huge combo is never wasted |
| **the active's charge** | Power fills its pips (6.3) | - | long combos buy the big button |
| **knockback** | never, except Grand Finale (8.3) | +50% | one keystone owns this, so it is a choice, not a default |

The caps are on single events, not on Mult. Mult can read x13 on the HUD; the foe still takes at most 40% from the
burst, and the rest is paid in coins. A fighter dies at about 150%, so damage has a natural ceiling anyway; bosses
with real HP (Petey) and NG+ foes are where the uncapped part of a build shows.

Because Mult now supplies growth, level-1 numbers drop by about 40% from V2 (Ember 6% to 4%) [J]. A build with no
synergy is a little weaker than in V2; a built one is several times stronger. The target for `sim.py` is in 12.

### 3.5 Determinism

All of it is integer state in the sim's one hashed struct [R core v2 2.7]: Base, Mult, the timer, the held set, the
chain queue, technique marks, coins, the active's pips. No chance rolls: a coefficient scales **charge**, not
probability, so there is no dice roll to desync and the player can plan a chain. The manifest's `state_bytes` is 4096
today [R lib.rs:2149] and will need raising.

## 4. Proc chains

### 4.1 The rule

- Every trigger drive has a **coefficient** in the pool data: 1.0 for a launching timed blast, 0.5-0.6 for a damage
  burst, 0.3-0.4 for a status fire, 0.3 for each small hit of a multi-hit fire, 0.2 for a damage-over-time tick, 0
  for heals and buffs.
- A fire is a hit with weight = its coefficient x its parent's coefficient. It adds charge **only to trigger drives
  to its right** (5).
- A drive that fills fires 6 frames after its parent. A chain is at most **4 links**. One chain at a time per
  fighter; the next may start 20 frames after the last link (V2's fire gap [R `codes.rs`], now between chains).
- A chained fire never launches. Only timed drives launch, as decided [D]; a launching timed blast keeps
  coefficient 1.0, so it is the best chain starter.

Three brakes, then: the coefficient shrinks each link, the chain only runs rightward so it cannot loop, and it stops
at four. Worst case is 4 fires every 38 frames, about V2's limit of 6 a second [R core v2 3.3].

### 4.2 A worked chain

Slots, left to right: Ember (5 hits, 0.5) | Bolt (4 hits, 0.6) | Hex (6 hits, 0.4). Bolt holds 3.6 of 4, Hex 5.3 of 6.

| frame | event | effect |
|---|---|---|
| 0 | the fifth Solid hit lands | Ember is full and fires: its burst, Burning. Link 1, weight 0.5 |
| 0 | Ember's fire reaches Bolt and Hex | Bolt 3.6 + 0.5 = 4.1: full. Hex 5.3 + 0.5 = 5.8: not yet |
| 6 | Bolt fires | lightning. Link 2, weight 0.5 x 0.6 = 0.3, passed to Hex: 6.1, full |
| 12 | Hex fires | Marked. Link 3. Nothing to its right: the chain ends |

Had Bolt been left of Ember, Ember's fire would not have reached it, and Hex would have stayed short. The same three
drives in another order are a different build.

## 5. Slot order

1. **Mult applies left to right** (3.3). `+0.5` then `x1.5` from x2.0 gives x3.75; the other order gives x3.5.
2. **Chains run left to right** (4.1). Starters go left, finishers right.
3. **Neighbour drives** act on a position: **Echo** makes the drive to its right fire twice; **Mirror** copies the
   drive to its left.
4. **The leftmost element shaper** sets the element shown (2.2).
5. With **Twin Soul** the benched fighter's row continues the active one: slots 6-10 of one line.

### 5.1 Reordering

Free, at any fight boundary and at the rest stage, on the reward screen. Never during a fight. Order is drawn in the
orbit: the first slot leads, and a chain's link flash runs along the orbit in order.

### 5.2 What the pick tells the player

A card shows what the drive would do **in the slot it would take**, with the neighbours it would act on lit. The
"compare rows only" rule [D E22] stays: no strength number.

### 5.3 Slots

V2 has 3 slots, then 4 after the first boss [R]. V3 proposes 3, 4 after the first boss and 5 after the second
(question 3). Five matches the limit that makes every pick a cut in Balatro [W10]. The orbit budget was 4 [R core v2
3.3]; shapers are drawn smaller and closer to the body than triggers, so the loud objects stay near four.

## 6. Sets, and the active drive

### 6.1 Families

The six model bodies and colours stay [D]. Their meanings sharpen so that each has a job in the formula:

| family | V2 meaning | V3 job |
|---|---|---|
| RED | damage | fire, bursts, chain starters |
| GREEN | speed | Haste and ground technique; `+Mult` |
| BLUE | defence | shield, tech and defence turned into attack |
| YELLOW | air | aerials, edgeguards, multi-hit feeders |
| PURPLE | status | words, and pay-offs that read words |
| WHITE | wild | order, retriggers, multipliers, scaling, coins |

A fifth word joins the four: **Shocked** (the next fire that hits the foe fires twice, then it ends). It is the
retrigger as a status.

### 6.2 Sets (three different drives of one family on one fighter)

| set | look | rule |
|---|---|---|
| RED: **Inferno** | the fighter's hits trail fire | Burning stacks to 6, and its ticks count as hits at 0.2 |
| GREEN: **Tempest** | afterimages at all times | Haste never runs out during a combo; each technique adds +0.1 Mult |
| BLUE: **Bastion** | a faint shell | a tech, a power shield or a shield drop fires your leftmost drive |
| YELLOW: **Skybound** | wing glints at the shoulders | aerial hits add double Base; landing does not start the combo timer for 30 frames |
| PURPLE: **Blight** | a dark haze on afflicted foes | each word on a foe is +0.5 Mult while a combo is on it |
| WHITE: **Conductor's Line** | the orbit becomes one bright ring | chains run one link longer and the first link keeps full strength |

A set is per fighter, as drives are [D]. Levels do not count toward it; three different drives do. A fighter with
five slots holds one set and two others, so the pair can hold two sets. A set is shown by one persistent look, the
only persistent look besides statuses and orbiters (core v2 3.3's rule stands otherwise).

### 6.3 The active drive

- **One per player**, not per fighter: it stays through a tag. It does not take a drive slot.
- **Button: D-pad right.** D-pad down is the tag [R `envoy_sim` lib.rs:1779, D E12]; D-pad up is the retail taunt
  [R `ftCo_AppealS.rs:76`]; D-pad left and right are read only in the reward freeze [R lib.rs:833], where the fight
  is stopped. So right is free in a fight, and left stays free for later.
- **Charge:** pips. One pip is 150 Power cashed out [J]. An active needs 2-4 pips. Pips are kept between fights.
- **Use:** on the ground or in the air when actionable, as the tag is. It is the player's own timing, so by the
  standing rule an active may launch [D].
- Got at the rest stage, from the first boss, or as a rare card. Taking a new one replaces the old.

| active | pips | effect |
|---|---|---|
| **Overdrive** | 3 | for 5 seconds every hit counts as Heavy |
| **Volley** | 3 | every drive fires now, in order, as one chain with no link limit |
| **Detonate** | 2 | every word on every foe bursts for 6% a stack and ends; each burst is a hit at 0.5 |
| **Recall** | 2 | warp to the centre of the stage, intangible for 1 second |
| **Double Down** | 4 | the current combo's Mult doubles; if you are hit before it ends, it is lost |

## 7. Scaling, deals, economy, tiers and pools

### 7.1 Scaling drives

A scaling drive keeps a counter for the whole run, in the run record [R record v4], shown on its orbiter as pips.

| drive | grows when | gives |
|---|---|---|
| **Cinder Heart** | a foe is KO'd while Burning | +1 Base on every hit, per KO, to +10 |
| **Snowball** | a cash-out of 200 Power or more | +0.05 starting Mult, per cash-out, to +2.0 |
| **Magpie** | (not a scaler; it feeds them) | cash-outs pay half again in coins |

Scalers are weak when found and strong when found early, so the same card is a different decision in act 1 and act 3.

### 7.2 Keystones as devil deals

Timing as decided: one at the run's start, one after each of the first two bosses [D]. Each is one rule and one
price. V3's change is what the rules touch: every keystone bends the formula of section 3 or the chain of section 4,
so a keystone picks the kind of run. The eight are in 8.3.

### 7.3 Coins, the shop and the pact

| coins from | amount [J] |
|---|---|
| cash-out | Power / 25, plus overflow / 10 |
| a fight cleared | 20; +10 with no stock lost |
| an edgeguard KO | +10 |
| a taunt inside 2 seconds of a KO | +5 (the retail taunt; you stand still for it) |
| interest at each fight's start | +1 per 10 held, to +5 |

Coins belong to each player in co-op, as drops do [D].

The **rest** encounter [D Q5] is the shop. Until the rest stage exists (core v2's milestone V8) it is a screen at
the same point in the run.

| on sale | price [J] |
|---|---|
| three drives, from the shop pool | Common 40, Rare 80, Mythic 150 |
| a level for a drive you wear | 60 to level 2, 120 to level 3 |
| an active | 80 |
| a reroll of the three | 10, then +10 each time |
| replacing a worn drive with a bought one | free; the old one is gone (there is no bag [D]) |
| reordering | free |

**The pact** (question 7): once per act the rest stage offers one Mythic drive for **one stock from the pool**. In
this game the shared stock pool [D] is the health a devil deal would take.

### 7.4 Rarity tiers and editions

Core v2 left rarity out because nothing the owner had said asked for it. Tiers now have a job: they ration
multipliers.

| tier | share of the pool | holds | mark |
|---|---|---|---|
| **Common** | about half | triggers, shapers, `+Mult`, technique charge | the plain glass shell |
| **Rare** | about a third | `xMult` on a condition, retriggers, scalers, set-makers | a coloured beam on the floor drop (the old game had a beam per rarity [R core v2 1.1]) |
| **Mythic** | the rest | repeating `xMult`, copy, the duo drives | a beam and a ring |

Levels (a duplicate, to 3) stay as they are and are separate from tier [D].

**Editions** (after the slice): a drop can carry one modifier, whatever the drive: Tempered (+5 Base when it fires),
Radiant (+0.3 Mult on its first fire of a combo), Prismatic (x1.25 Mult on its first fire of a combo), Hollow (takes
no slot). They multiply the pool's variety without new rules to learn.

### 7.5 Pools by encounter kind

| source | pool | rule |
|---|---|---|
| duel drop | Common 80%, Rare 20% | any family |
| pack drop | Common, weighted to PURPLE and RED | many foes: spreading and chain drives |
| shift room drop | Common and Rare, weighted to GREEN and YELLOW | movement and technique |
| elite | Rare | one of its two drops is **a drive the elite carried** [D elites carry drives]: you take what was used on you |
| boss | Mythic, a pick of three; then the keystone pick [D] | the first boss also offers the first active |
| pick of three (every boundary) | one card from the encounter's pool, one that connects to what the fighter wears, one free | "connects" now means: a word it gives or reads, its family (toward a set), or a technique the player has used this run |
| shop | all tiers at their prices | the only place to buy a level |
| pact | Mythic | one stock |
| duo drives | Mythic | offered only when the fighter wears a drive from each of the two families |

A drive declined in a pick is half as likely for the rest of the run (Isaac lowers or removes seen items [W9]). A
level-3 drive is never offered again [D]. Unlocks stay content-only [D]: Common is open from the start, Rares and
Mythics unlock by clears and NG+ levels.

## 8. The pool: 40 drives, 5 actives, 8 keystones

Kinds: **T** trigger (charges and fires; its coefficient in brackets), **S** shaper, **+M** adds Mult, **xM**
multiplies Mult, **G** grows over the run, **E** economy, **N** neighbour. Tiers: C, R, M. Numbers are level 1 and
first guesses. "V2" marks a drive kept from V2, changed where the rule says so.

### 8.1 Drives

| # | family | name | tier | kind | rule | job in a build |
|---|---|---|---|---|---|---|
| 1 | RED | **Ember** V2 | C | T (0.5) | Every 5 hits: a fire burst, 4% and Burning. | chain starter |
| 2 | RED | **Kindle** | C | S | Your smash attacks are fire: they set Burning. | a word with no charge needed |
| 3 | RED | **Bolt** V2 | C | T (0.6) | Every 4 hits: lightning strikes the nearest foe, 5%. | was on a clock; now on hits so it sits in chains |
| 4 | RED | **Nova** V2 | R | T (1.0) | Every 12 seconds: a blast around you, 8%. | timed, launches; the full-strength starter |
| 5 | RED | **Wildfire** V2 | R | T (0.5) | A combo ends on a Burning foe: 10% to every foe. | pay-off for Burning; pack clear |
| 6 | RED | **Cinder Heart** | R | G | Each Burning KO: +1 Base on every hit, this run. | scaler |
| 7 | GREEN | **Rush** V2 | C | T (0) | Every 6 hits: Haste for 4 seconds. | gives Haste |
| 8 | GREEN | **Slipstream** V2 | C | T (0) | Tag in: Haste for 6 seconds. | gives Haste |
| 9 | GREEN | **Tailwind** V2 | C | +M | In Haste, each hit adds +0.2 Mult. | was "+25% damage" |
| 10 | GREEN | **Slick** | C | S | Each wavedash gives every drive half a hit of charge. | technique as charge |
| 11 | GREEN | **Clean Landing** | C | +M | Each L-cancel adds +0.3 Mult. | technique as Mult |
| 12 | GREEN | **Perfect Form** | R | xM | The third L-cancel of a combo: x1.5 Mult. | goes right of Clean Landing |
| 13 | BLUE | **Bulwark** V2 | C | T (0) | Every 15 seconds: an orb absorbs the next hit. | protects a combo from being dropped |
| 14 | BLUE | **Thorns** V2 | C | T (0.4) | When hit: the attacker takes 4% and is Marked. | a fire that starts a chain while you are losing |
| 15 | BLUE | **Leech** V2 | C | S | A cash-out heals 1% for each 30 Power, to 10%. | was "every 5th hit" |
| 16 | BLUE | **Riposte** | C | S | After a tech or power shield, your next hit counts Heavy. | defence into Base |
| 17 | BLUE | **Breaker** | R | S | Your hits deal double shield damage. A shield break: x2 Mult. | shield pressure build |
| 18 | BLUE | **Anchor** | R | xM | A hit out of a shield drop: x1.3 Mult. | technique xMult |
| 19 | YELLOW | **Comet** V2 | C | T (1.0) | Every 3rd landing: a shockwave, 4%. | timed, pops up; full strength |
| 20 | YELLOW | **Updraft** V2 | C | T (0) | Every 3 aerial hits: an air jump back. | longer air combos |
| 21 | YELLOW | **Feather Storm** V2 | C | T (0.3 x3) | Every 6th jump: three feathers, 2% each. | three small hits: the best feeder |
| 22 | YELLOW | **Skyhook** | R | S | Aerial hits count one weight heavier. | Base and charge for air play |
| 23 | YELLOW | **Gatekeeper** | R | xM | The first hit on an offstage foe: x1.5 Mult. | edgeguard |
| 24 | YELLOW | **Meteor** | M | xM | A meteor smash: x2 Mult. | the hardest condition, the biggest single step |
| 25 | PURPLE | **Frost** V2 | C | T (0.3) | Every 5 hits: Chill the foe for 4 seconds. | gives Chilled |
| 26 | PURPLE | **Shatter** V2 | C | T (0.6) | Hit a Chilled foe: ice shards, 6%, ending the Chill. | reads Chilled |
| 27 | PURPLE | **Hex** V2 | C | T (0.4) | Every 6 hits: Mark the foe for 5 seconds. | gives Marked |
| 28 | PURPLE | **Static** | C | S | Your specials are electric: they Shock the foe. | gives Shocked, the retrigger word |
| 29 | PURPLE | **Affliction** | R | +M | Each hit adds +0.2 Mult for every word on the foe. | reads every word |
| 30 | PURPLE | **Contagion** | R | S | A word you apply also lands on foes nearby. | packs; with Blight |
| 31 | WHITE | **Relay** V2 | R | T (1.0) | Tag in: your drives fire in order, as one chain. | the tag as a chain starter |
| 32 | WHITE | **Echo** V2 | R | N | The drive to the right fires twice. | was "every 3rd fire repeats" |
| 33 | WHITE | **Conductor** | R | S | Chained fires keep full strength for one more link. | lifts every coefficient |
| 34 | WHITE | **Metronome** | C | +M | Each different move in a combo adds +0.3 Mult. | variety; no technique needed |
| 35 | WHITE | **Crescendo** | M | xM | Every 5th hit of a combo: x1.5 Mult. | the pure exponential |
| 36 | WHITE | **Snowball** | R | G | A cash-out of 200 Power: +0.05 starting Mult, this run. | scaler |
| 37 | WHITE | **Magpie** | C | E | Cash-outs pay half again in coins. | economy |
| 38 | WHITE | **Mirror** | M | N | Copies the drive to its left. | a second Crescendo, or a second Ember |
| 39 | RED + PURPLE | **Steam** | M duo | xM | A foe both Burning and Chilled: x2 Mult; both end. | needs two givers |
| 40 | YELLOW + RED | **Thunderhead** | M duo | T (0.6) | Each aerial hit on an offstage foe calls a lightning bolt. | edgeguard into chains |

Not carried from V2: **Spire** and **Surge**. Spire's job (a finisher on a foe above 100%) is the cash-out burst now;
Surge keyed on a stage shift, which only one encounter kind has. Either can return as a later drive.

Checks the lint keeps or gains (11): every word read has two givers (Burning: Ember, Kindle, Plague Bearer; Chilled:
Frost, Winter Oath; Marked: Thorns, Hex; Haste: Rush, Slipstream). Shocked has one giver in this list (Static); Bolt
at level 3 becomes the second [J]. Every `xM` drive is Rare or Mythic. Every technique drive has a drive of the same
family with no technique condition.

### 8.2 Actives

The five of 6.3: Overdrive, Volley, Detonate, Recall, Double Down.

### 8.3 Keystones (8)

| # | name | rule | price | what it does to the formula |
|---|---|---|---|---|
| 1 | **Plague Bearer** V2 | Every hit sets Burning, to 3 stacks. Burning ticks count as hits. | Fragile | Burning ticks (0.2 each) feed every drive: a build that charges itself |
| 2 | **Echo Oath** V2 | Every drive fire repeats once at half strength. | Weak | a retrigger on everything; your own hits matter less |
| 3 | **Overclock** V2 | Drives charge twice as fast. | Bleed | more chains, each fire costing 1% |
| 4 | **Twin Soul** V2 | The benched fighter's drives join your row and fire too. | Lock | a ten-slot line: two sets, long chains, few tags |
| 5 | **Winter Oath** V2 | All your hits Chill for 2 seconds. | Slow | a word on every hit; Shatter, Steam and Affliction are always live |
| 6 | **Glass Core** V2 | Mult starts at x2. You take 60% more. | own | was "deal and take 60% more"; now it is half of every Power you will ever cash out, at the risk of dropping it |
| 7 | **Grand Finale** | A smash attack cashes the combo out into its launch. | Weak | the only knockback scaling: +1% knockback per 20 Power, to +50%. Build Power, then end it yourself |
| 8 | **All In** | Your combo never times out. A hit you take cashes it out on you. | own | Power climbs for a whole stock; one mistake takes up to 40% |

## 9. Five builds, and how each snowballs

Numbers use the rules of section 3 and level-1 values. They are worked by hand, not simulated [J].

### 9.1 Firestorm (proc chain)

- **Pieces:** Plague Bearer; Ember | Bolt | Wildfire | Cinder Heart, then Conductor in the fifth slot. Three RED
  drives make **Inferno**.
- **How it runs:** every hit sets Burning. Three stacks tick as three 0.2 hits a second, so Ember and Bolt fill
  while the player is only chasing. Ember's fire feeds Bolt, Bolt's fire lands on a Burning foe, and the combo's end
  on a Burning foe fires Wildfire at every foe, which sets up the next one.
- **Snowball:** with Inferno the stacks go to six (1.2 hits a second from ticks alone). Each Burning KO adds +1 Base
  to every hit through Cinder Heart, so by act 3 a hit is worth twice the Base. Conductor then keeps the second link
  at full strength.
- **Weak to:** single foes with long invulnerability; the Fragile price.

### 9.2 Tech Monk (technique Mult)

- **Pieces:** Grand Finale; Clean Landing | Metronome | Perfect Form | Crescendo. Two GREEN, two WHITE: no set, all
  Mult.
- **How it runs:** an eight-hit combo of Solid hits with four L-cancels and four different moves. Base 80. Clean
  Landing gives +1.2 and Metronome +1.2 over the combo. Crescendo multiplies by 1.5 on the fifth hit and Perfect
  Form by 1.5 on the third L-cancel. In the order the events arrive that is about x7: Power about 560, against 80
  for the same combo with no drives. A smash then cashes out with +28% knockback, so it kills clearly earlier.
- **Snowball:** the overflow (the burst caps at 400 Power) pays coins every combo, which buy Mirror next to
  Crescendo: x1.5 twice on every fifth hit. Longer combos then reach Grand Finale's +50% cap at 1000 Power.
- **Weak to:** a player who drops L-cancels. Metronome alone still gives the build a floor.

### 9.3 Frozen Clockwork (words and retriggers)

- **Pieces:** Winter Oath; Echo | Shatter | Affliction | Static, then Frost and Hex for **Blight**.
- **How it runs:** every hit Chills, so Shatter is live on every other hit. Echo, to its left, makes it fire twice. A
  special Shocks the foe, so the next Shatter fires twice again: four sets of shards from one hit. Affliction adds
  +0.2 Mult per word on the foe per hit, and the foe usually carries two or three.
- **Snowball:** Steam (the duo, if a RED giver is added) multiplies by 2 whenever Burning and Chilled meet. Blight
  adds +0.5 Mult per word. Contagion spreads all of it across a pack.
- **Weak to:** bosses, where Chilled and Marked act at half strength [R `codes.rs`]; the Slow price.

### 9.4 Sky Pirate (air and edgeguard)

- **Pieces:** All In or Overclock; Feather Storm | Skyhook | Gatekeeper | Meteor | Updraft, all YELLOW:
  **Skybound**. Active: Recall.
- **How it runs:** jumps charge Feather Storm, whose three feathers are three small hits for whatever sits to its
  right. Aerials count a weight heavier and, with Skybound, add double Base. The first hit offstage multiplies by
  1.5, a meteor by 2. An edgeguard KO finishes the combo at x1.5 and pays +10 coins.
- **Snowball:** edgeguard KOs are the best coin source in the game, so this build reaches the shop's Mythics first.
  Thunderhead (the duo) turns every offstage aerial into lightning. Recall is the way back from deep.
- **Weak to:** stages with walls or no ledges, and the boss fights with no ledge play.

### 9.5 Twin Engine (the tag, and co-op)

- **Pieces:** Twin Soul; fighter A: Relay | Ember | Bolt | Mirror; fighter B: Slipstream | Tailwind | Snowball |
  Magpie. Active: Volley.
- **How it runs:** with Twin Soul both rows are one line of eight to ten. A tag fires Relay: fighter A's whole row
  as one chain at full strength, which runs on into fighter B's row. Slipstream gives Haste on the same tag, and
  Tailwind turns the Haste into Mult for the combo the chain just started.
- **Snowball:** Snowball raises starting Mult every big cash-out; Magpie funds levels. Volley is a second Relay on a
  button. In co-op one player builds the chain row and the other the Mult row, and they share a foe's combo state
  only through the words on it.
- **Weak to:** the Lock price (a 10-second tag cooldown), which makes each tag a commitment.

A sixth shape exists without being named as a build: the banker (Magpie, Snowball, interest), which is weak for an
act and then buys whatever it wants.

## 10. What the player sees and hears

| event | sight | sound |
|---|---|---|
| a combo is running | beside the player's percent: Base in blue, an x, Mult in red, and a thin timer bar under them | - |
| a hit | Base ticks up | - |
| `+Mult` | the red number bumps; the drive that gave it blinks | a tick, one step higher in pitch each time within a combo |
| `xMult` | the red number flashes white and grows for 10 frames; a ring off the giving drive | a distinct chime |
| a technique registers | a small spark at the fighter's feet in the reading drive's colour | a soft click |
| a chain | the link flash (ruled "awesome" [O]) runs drive to drive along the orbit, one link every 6 frames | each link a step higher |
| a retrigger | the drive darts out twice | the same sound twice |
| cash-out | the two numbers slide together into Power; the burst has three sizes (under 100, under 400, over) | a rising hit; a coin sound for overflow |
| dropped or lost | the numbers break and fall | a low thud |
| the active is ready | its icon by the percent glows; the D-pad right glyph shows | one chime |
| a set completes | the fighter's look changes for good | a fanfare once |
| a foe's drives chain | the same link flash, in the foe's colour | the same, lower |

The two numbers are HUD, not a toast, so the "no text on a fire, a pickup or a level-up" ruling [O] stands. They are
the one place the fight shows digits. The technique spark is also a training aid: a player sees which L-cancels
counted.

Budget, on top of core v2 3.3: one chain at a time per fighter, four links, six frames apart; one set look per
fighter; the cash-out burst is the only effect larger than a fire, at most 30 frames.

## 11. From V2: what is reused and what changes

| V2 piece [R `envoy-drives.md`] | V3 |
|---|---|
| `pool.toml` -> `build.rs` -> `pool_gen.rs`, `POOL_DIGEST` | **Reuse.** New fields: `tier`, `kind`, `coef`, `mult_add`, `mult_mul`, `mark` (a technique), `set`, `duo` |
| the lint | **Reuse and extend.** New: every trigger has a coefficient; every `xM` is Rare or Mythic; every technique drive has a same-family drive without one; a chain of the whole pool in any order ends inside 4 links; the fire-period band now runs with chains on |
| the charge sources (hit, taken, landing, jump, time, tag_in, aerial_hit, hit_100, ko, shift, fire, passive) | **Reuse**, plus `tech`, `lcancel`, `wavedash`, `ledge`, `shield_drop`, `pshield`, `offstage_hit`, `cashout`. `fire` becomes the chain rule for every drive |
| charges as integer counts | **Change:** milli-hits, so weights and coefficients are fractions |
| the fire queue, one fire per fighter per 20 frames | **Change:** the chain queue (4.1); the 20 frames now sit between chains |
| the fire log (12 entries) in the view | **Reuse**, with link depth and parent, so the UI can draw chains |
| levels x1.0 / x1.6 / x2.4, no bag, duplicates level | **Reuse** [D] |
| slots 3 then 4, per fighter | **Change:** ordered; a fifth after the second boss (question 3) |
| the four words and their windows | **Reuse**, plus Shocked |
| `gd_envoy_blast` and the "only timed drives launch" test | **Reuse** [D]. Chained fires use the percent path |
| `gd_envoy_attacker`, `gd_envoy_absorb`, `gd_envoy_speed`, `gd_envoy_jumps` | **Reuse** |
| action-state polling for landings and jumps | **Reuse and extend** to the technique table (2.3) |
| keystones: six, three per run, the seven prices | **Reuse** the timing and prices [D]; six rules restated, two new |
| foes carry drives | **Reuse** [D]. A foe has a combo too, without a cash-out burst in the slice |
| per-player drops, picks, pool 5 / 4, co-op | **Reuse** [D]; coins and the active are per player |
| the pick of three's "connects to a word" rule | **Change:** connects to a word, a family toward a set, or a technique used this run |
| the run record v4 | **Change:** v5 adds order, scaler counters, coins, the active and its pips |
| `sim.py` (seeds x policies) | **Reuse and extend:** a combo model per policy (length, technique rates), and the build-spread measure (12) |
| the UI's orbiters, halo, link flash, Brawl sounds | **Reuse**; new: the two numbers, the timer bar, the chain line, set looks |
| the 20 drives | 18 kept (most restated), 2 set aside (Spire, Surge), 22 new |
| "the full build is 0.6 s between fires" tuning | **Change:** the dial is now chains per second and the spread, not fires per second |

New host work, all small: `gd_envoy_events` (the L-cancel bit first, the meteor's angle later), `gd_envoy_shape`
(element, shield damage, hitlag; later size and knockback), a stage ledge table in the sim, raising `state_bytes`.

## 12. The V3 milestone: a slice to judge depth

Core v2's milestones V3-V11 each move down one; this slice takes the number V3. Off `port/v2`.

**In the slice.**

| piece | scope |
|---|---|
| the combo | 3.1-3.4 in full: Base, Mult, the timer, the four ends, the cash-out burst, the caps |
| chains | 4.1 in full: coefficients, rightward only, 4 links, 6 frames |
| order | ordered slots; reorder on the reward screen; Echo as the one neighbour drive |
| technique | L-cancel (the one event bit), wavedash, tech, ledge grab, offstage hit (polled; a ledge table for the 5 Short-run stages) |
| drives (16) | Ember, Bolt, Nova, Wildfire; Rush, Tailwind, Clean Landing, Perfect Form; Riposte; Feather Storm, Gatekeeper; Frost, Shatter; Echo, Metronome, Crescendo |
| keystones (3) | Plague Bearer, Glass Core, Grand Finale |
| the active (1) | Volley, on D-pad right, given at the run's start |
| a set (1) | Inferno, so the three-of-a-family moment can be felt |
| coins | earned as in 7.3; spent on one plain screen after fight 3: three drives, a level, a reroll |
| tiers | the three tiers as data and as a beam colour; no editions |
| HUD | the two numbers, the timer bar, the chain line, the technique spark, stand-in sounds |
| shapers | words and bonus damage only (no engine seam): element, shield damage, hitlag and size wait |
| checks | the lint; `sim.py` with the combo model; synctest depth 4 and 7 over 3600 frames, 0 mismatches, across a chain, a cash-out, a technique mark, the active and a shop buy |

**Out of the slice:** co-op (V2's works and is re-tested after), the rest stage, the pact, editions, duo drives,
`gd_envoy_shape`, four of the five actives, five of the six sets, foes' cash-outs.

**The measure before the playtest.** `sim.py` gains a build-spread number: the median Power per combo of the best
five-drive build over that of a random five-drive build, for each policy. V2's pool would score near 1, since its
drives do not interact [I]. The slice's band is **3 to 8** [J]: under 3 the builds are still shallow, over 8 the pool
is solved by one build. A second number: the share of the 16 drives that appear in at least one of the top ten
builds, at least three quarters.

**The playtest.** Two Short runs. The reward screen is nudged toward Firestorm in one and Tech Monk in the other.
The question for the owner: "did those two runs play differently, and did you want a third?".

**Size and models.** M (one to two weeks). Sonnet: pool data, lint, the sim's combo and chain state, `sim.py`,
synctests. Opus: the HUD numbers and their feel, the chain line, the shop screen.

**Risks.**

| risk | answer |
|---|---|
| the HUD numbers pull the eye off the fight | they sit by the percent the player already reads; the sound carries Mult without looking |
| the combo rule feels unfair (a combo that "should" have counted ends) | the 45-frame grace is one constant; the timer bar shows it; tune in the playtest |
| technique drives lock out players who do not L-cancel | every technique drive has a same-family drive without one (lint); a pick shows at most one technique card unless the player has used that technique this run |
| chains bring the October noise back | four links, one chain at a time, rising pitch so a chain reads as one event |
| the L-cancel bit slips | Clean Landing and Perfect Form take wavedash conditions for the slice; the bit is a day's work |
| CPU-proxy tuning misleads (CPUs do not combo like people) | the spread is a property of the pool, not of the pace; the owner's play sets the dials |

## 13. Questions for the owner (one at a time; each with my recommendation)

1. **Is the combo (Base x Mult, then a cash-out) the centre of V3, with two numbers on the HUD during a fight?**
   *Recommendation: yes.* It is the one shared score every drive can feed, and it is the part of Balatro that
   Melee already has a natural shape for. The numbers are the only digits shown in a fight.
2. **Does slot order matter: Mult applied left to right, and a fire only feeding drives to its right?**
   *Recommendation: yes, with free reordering between fights.* It makes the same drives into different builds and it
   stops loops without a hidden rule. The alternative is no order and a "no drive twice per chain" rule.
3. **How many slots per fighter?** *Recommendation: 3, then 4 after the first boss, then 5 after the second.* Five
   leaves room for a set of three plus two others. The alternative is to stay at 4.
4. **The active drive on D-pad right?** D-pad down is the tag and D-pad up is the taunt, so right and left are the
   free ones. *Recommendation: D-pad right, one active per player, kept through a tag.*
5. **Should Mult ever change the knockback of your own moves?** *Recommendation: no, except through the Grand Finale
   keystone.* Your moves keep retail percent and knockback so combos stay learnable; drives and the cash-out carry
   the growth.
6. **How much should technique matter?** *Recommendation: technique drives are a bonus path, never a requirement:
   every family has drives with no technique condition, and a pick shows at most one technique card until you have
   used that technique in the run.* The alternative is to make technique the main source of Mult.
7. **Coins and a shop at the rest point, and the pact (one Mythic drive for one stock from the pool, once per act)?**
   *Recommendation: yes to both.* Coins give big combos a use beyond damage; the pact is the devil deal in the
   currency this game already has.
8. **Rarity tiers now, editions later?** *Recommendation: three tiers in the slice, as a beam colour on the drop;
   editions after the slice has been judged.* Tiers ration the multipliers; editions are variety that can wait.

## 14. Sources

### Web (read 2026-11-10)

- [W1] Risk of Rain 2 Wiki, "Proc Coefficient": <https://riskofrain2.wiki.gg/wiki/Proc_Coefficient>
- [W2] Rogue Ranker, "Risk of Rain 2 Ukulele" (the chain example, as a search excerpt):
  <https://rogueranker.com/risk-of-rain-2-ukulele/>
- [W3] Risk of Rain 2 Wiki, "Item Stacking": <https://riskofrain2.wiki.gg/wiki/Item_Stacking>
- [W4] Game Developer, "How moving from 2D to 3D shaped the design of Risk of Rain 2" (search excerpt):
  <https://www.gamedeveloper.com/disciplines/how-moving-from-2d-to-3d-shaped-the-design-of-i-risk-of-rain-2-i->
- [W5] Game Developer, "Postmortem: McMillen and Himsl's The Binding of Isaac":
  <https://www.gamedeveloper.com/business/postmortem-mcmillen-and-himsl-s-i-the-binding-of-isaac-i->
- [W6] Binding of Isaac: Rebirth Wiki, "Transformations": <https://bindingofisaacrebirth.wiki.gg/wiki/Transformations>
- [W7] Binding of Isaac: Rebirth Wiki, "Items": <https://bindingofisaacrebirth.wiki.gg/wiki/Items>
- [W8] Binding of Isaac: Rebirth Wiki, "Devil Room": <https://bindingofisaacrebirth.wiki.gg/wiki/Devil_Room>
- [W9] Binding of Isaac: Rebirth Wiki, "Item Pool": <https://bindingofisaacrebirth.wiki.gg/wiki/Item_Pool>
- [W10] Balatro Wiki, "Jokers": <https://balatrowiki.org/w/Jokers>
- [W11] Balatro Wiki, "Guide: Activation Sequence": <https://balatrowiki.org/w/Guide:_Activation_Sequence>
- [W12] Balatro Wiki, "Money": <https://balatrowiki.org/w/Money>
- [W13] Shacknews, "Balatro was largely inspired by slot machine roguelike Luck Be a Landlord", and Rogueliker's
  interview (search excerpts): <https://shacknews.com/article/139116/balatro-inspiration-luck-be-a-landlord-reddit-ama>,
  <https://rogueliker.com/balatro-interview/>
- [W14] Wikipedia, "Hades (video game)": <https://en.wikipedia.org/wiki/Hades_(video_game)>
- [W15] Wikipedia, "Slay the Spire", and GDC 2019, Anthony Giovannetti, "Slay the Spire: Metrics Driven Design and
  Balance" (announcement page): <https://en.wikipedia.org/wiki/Slay_the_Spire>,
  <https://gdconf.com/news/learn-slay-spires-successful-metrics-driven-approach-game-balancing-gdc-2019>

Not reached this session, so the claims that rest on them are tagged [K]: a Risk of Rain 2 page on the proc chain
rule that stops an item triggering twice in one chain; the Hades duo boon pages; the Slay the Spire relic pages.
No Hopoo or LocalThunk design talk was found that covers proc chains or joker ordering directly; the mechanics above
come from the wikis.

### Repo (this branch unless named)

- `docs/rust-port/notes/envoy-drives.md`; `envoy-core-v2.md` on `pub/port/envoycore2`.
- `rust/mods/envoy_data/pool.toml`, `src/codes.rs`; `rust/mods/envoy_sim/src/lib.rs` (imports 39-81, action-state
  ranges 117-123, the damage hooks 1378-1425, the tag 1779-1801, the hook table 2137-2149).
- `rust/melee_modes/src/tu/envoy.rs` (`absorb`, `attacker`, `blast`, the snapshot note in the header);
  `rust/melee_modhost/src/lib.rs:536` (`gd_fighter_read`'s fixed record).
- `rust/melee_fighter/src/tu/ftCo_LandingAir.rs:65` (the L-cancel test), `ftCo_Landing.rs:140`
  (LandingFallSpecial), `ftCo_AppealS.rs:76` (the taunt on D-pad up), `ftcoll.rs` (510, 840: the element at a
  connect), `fighter.rs:2912` (hitlag), `ft_0763.rs:26` (the attack id).
- `rust/melee_types/src/gen.rs` (motion-state ids, `HitCapsule`); `docs/specs/hooks-v1.tsv` (the action hook rows).
- Memory: `envoy-lab-answers-2026-11-09.md`.
