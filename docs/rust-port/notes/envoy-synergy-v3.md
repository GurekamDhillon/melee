# Envoy synergy architecture, V3: Heat, and drives that feed each other (design, revised 2026-11-10)

Design only. Nothing here was built or run. Branch `port/synergy3`, worktree `C:/melee-wt/synergy3`, off `pub/port/v2`
(`288627e37`). By Fable. It revises `envoy-core-v2.md` (port/envoycore2) sections 2.1, 2.3-2.5, 3.3, 4 and 6, and
builds on `envoy-drives.md` (the V2 build note, this branch). Everything else in those notes stands.

> **Changed on 2026-11-10 (owner decision).**
>
> **Replaced:** the combo as the centre of the design: Base x Mult built up over a combo, then a cash-out when the
> combo ended. With it go the hit weights (Light / Solid / Heavy), the combo timer and its 45-frame grace, the four
> combo endings, Power, and every drive, keystone, set and active that counted hits or read the combo.
>
> **Why, in the owner's terms:**
>
> 1. A "true combo" is too strict. Most characters land one or two hits true, and the loosened window did not change
>    that enough. A combo-centred system rarely builds, and it favours a few combo-heavy characters.
> 2. Multi-hit moves (Fox's drill) proc many times for a free, huge count. Counting a Light hit as half a hit only
>    softened that.
>
> **Now:** **Heat**. One meter per player. It fills by **damage dealt (percent)**, never by the number of hits: a
> 7-hit drill totalling 14% is worth what one 14% hit is worth. It carries through neutral, drops by a chunk when
> you are hit, and cashes out on a KO. Drives add to it or multiply it. **All drive charge is by damage, not by hit
> count**, everywhere.
>
> **Kept:** proc chains, slot order, sets, technique, the active drive on D-pad right, coins and the shop, rarity
> tiers, scaling drives, keystones as devil deals. Each was re-derived on damage; the numbers are new.

Tags: **[R]** read from the repo (cited), **[W]** read on the web (section 14), **[K]** the author's own knowledge
of the game, not fetched, **[I]** inferred, **[J]** judgement, **[U]** unverified in code, **[D]** an owner decision
already taken (memory `envoy-lab-answers-2026-11-09.md`, or the Heat decision above), **[O]** the owner's own words.
Every number in this note is a first guess [J] unless tagged otherwise. Worked examples are **hand-worked**, not
simulated, and say so.

## 0. One page

**Why V3.** The owner played V2 (20 drives, 6 keystones) and said: "the games still feel pretty shallow and
un-unique" [O]. The agreed diagnosis: every drive is a standalone "every N hits do X". Nothing is shared between
drives, a fire almost never causes another fire, and the numbers only add. Of V2's 20 drives, 14 neither read a word
nor act on another drive [R `pool.toml`].

**What the reference games share** (section 1): one small thing that every item touches, numbers that multiply
instead of adding, effects that count as triggers for other effects, and a position or a set that makes the same
items behave differently.

**The design in twelve lines.**

- **Damage is the shared thing.** Every piece of damage a player's side deals to a foe is one record: who, to whom,
  how much, from what. Nothing in the design counts hits.
- **Heat is the shared number.** One meter per player. 1% dealt is 1 Heat. 1% taken costs 2 Heat. No decay.
- **Heat has four states**: Cold, Warm (25), Hot (60), Blazing (110), full at 150. The state scales every drive
  fire: x1, x1.25, x1.5, x2.
- **A KO cashes out.** It pays coins, charge for the active drive and a flare at the other foes, all sized by Heat.
  Then you keep half.
- **Drives charge by damage.** "Every 40% dealt", not "every 5 hits". A drill and a single hit of the same total
  charge the same.
- **Fires are damage.** A drive's fire charges the drives **to its right** by its damage times its **coefficient**,
  and feeds Heat the same way. That is a proc chain; position, the coefficient and a four-link limit are the brakes.
- **Slot order matters.** Heat steps apply left to right, chains run left to right, and two drives act on a
  neighbour.
- **Technique is a resource.** L-cancel, wavedash, tech, ledge grab, shield drop, power shield and edgeguard give
  Heat or speed up charge, but only through damage that follows them or a hit they answered, so none can be farmed
  in a corner.
- **Three of a family on one fighter is a set**: a visible transformation with its own rule.
- **One active drive per player on D-pad right**, charged by cash-outs.
- **Scaling drives** grow for the whole run. **Keystones are the devil deals**: eight, each bends a Heat rule.
- **V3 is a slice**: Heat, damage charge, chains, order, four techniques, 16 drives, 3 keystones, 1 active, a plain
  shop screen, in the 5-fight Short run. Its question is "do two runs feel different?", and now also "does every
  character build Heat?".

Kept as decided [D]: drives per fighter; floor drops plus a pick of three; a duplicate levels a drive (to 3); no bag;
keystones at the start and after the first two bosses; fires are loud; only timed drives launch; elites and bosses
carry drives; co-op drops per player; content-only unlocks; NG+; the Brawl disc required; the drive models.

## 1. How the reference games get depth

Mechanisms in my own words. No item text is reproduced. The research is unchanged from the first version of this
note; the right-hand column is what Envoy takes **after** the Heat decision.

### 1.1 Risk of Rain 2

| mechanism | how it works | what Envoy takes |
|---|---|---|
| on-hit items | most items react to "a hit landed", so every item shares one trigger | the damage record (2) |
| proc coefficient | each attack and each item effect has a multiplier, usually 0 to 1, on how likely or how strong on-hit effects are from it. Fast weak attacks get low values, heavy ones high. An effect at 0 never triggers anything [W1] | Envoy goes one step further: charge is the damage itself, so a fast weak attack is already worth less. Only fires carry a coefficient (4) |
| proc chains | an item's effect is itself a hit, so it can trigger other items at its own coefficient [W2]. A chain cannot trigger the same item twice [K] | fires feed drives, with brakes (4) |
| stacking curves | linear, hyperbolic (never reaching 100%), exponential [W3] | levels are linear; a cap wherever a number could reach "always" (3.6) |
| lunar items | a strong effect with a built-in drawback [W3] | keystones (7.2) |
| design intent | Hopoo: each item starts from a specific purpose [W4] | every drive names the job it does in a build (8) |

### 1.2 The Binding of Isaac

| mechanism | how it works | what Envoy takes |
|---|---|---|
| the tear | nearly every offensive item changes the player's one attack, and changes combine [K]. McMillen: about 70% of items stack [W5] | shapers change the player's own damage and all apply at once (2.2) |
| visible growth | the character visibly changes over a run [W5] | the Heat state and sets change the fighter's look (6.2, 10) |
| transformations | three items from a themed group give a bonus and a new look [W6] | sets (6.2) |
| active items | one held at a time; it charges, then is spent by a button [W7] | one active per player, charged by cash-outs (6.3) |
| devil deals | strong items paid in permanent health [W8] | keystone prices; the stock pact (7.3) |
| item pools | each room type draws from its own pool [W9] | pools per encounter kind (7.5) |
| item quality | a quality from 0 to 4 [W7] | three rarity tiers (7.4) |

### 1.3 Balatro

| mechanism | how it works | what Envoy takes |
|---|---|---|
| chips x mult | a hand scores chips times mult [W10] | **not** a per-combo score any more (rejected 2026-11-10). What stays: one shared number, with drives that add to it (+Heat), multiply its growth (xGain) or multiply it (xHeat) |
| +Mult before xMult | effects resolve left to right, so adding before multiplying scores more [W11] | slot order: +Heat left of xHeat (5) |
| activation sequence | a fixed, written order [W11] | the Heat step is written down as a fixed order (3.3) |
| retriggers | an effect that makes a card score again [W10][W11] | Echo, Shocked, Echo Oath (5, 8) |
| scaling jokers | start weak and grow over the run [W10] | scaling drives (7.1) |
| copy jokers | reproduce a neighbour's ability [W10] | Mirror, Echo (5) |
| editions | a modifier on any joker [W10] | editions, after the slice (7.4) |
| slot limit | five jokers, so every pick is also a cut [W10] | 5 slots per fighter (5.3) |
| economy | money per round, interest, a shop between rounds [W12] | coins and the shop (7.3) |

### 1.4 Two others

- **Hades.** A duo boon appears only when the player already holds a qualifying boon from each of two gods [K].
  Envoy takes two **duo drives** (7.5).
- **Slay the Spire.** Balance was driven by pick-rate and win-rate metrics from play [W15]. Envoy takes: the elite
  drops what it carried (7.5), and `sim.py` measures build spread and character spread before a playtest (12).

## 2. The shared primitive: damage dealt

### 2.1 The damage record

Every time damage from a player's side lands on a foe, the sim builds one record. Today's take-damage post hook
already runs at that moment and knows the victim, the damage and the attacker [R `envoy_sim` `on_damage`,
`envoy::attacker`].

| field | values | where it comes from |
|---|---|---|
| `owner` | the player's set in play | `gd_envoy_attacker` [R]; it already resolves an item or projectile to its owner [R `envoy.rs:427-441`] |
| `victim` | match slot | the hook's fighter [R] |
| `src` | the fighter's move, or a drive slot (a fire), or a word (Burning, Marked) | the attacker's action state [R, polled today for aerials]; fires and word damage are raised by the sim itself |
| `dmg` | the percent dealt, as an integer: 1% = 1000 | the hook's `f32` damage [R], rounded once at the hook (3.7) |
| `coef` | x1000. A move: 1000. A fire: the drive's coefficient times its parent's. Word damage: 200 | pool data (4.1) |
| `depth` | 0 for a move; 1-4 along a chain | the sim |
| `tags` | AERIAL, SMASH, SPECIAL, THROW, OFFSTAGE, METEOR, plus the technique marks of 2.3 | action states, the stage table, the event word (2.3) |

There is no weight class and no hit counter. A record's worth is `dmg x coef`. That one fact is the answer to both of
the owner's objections: a character with two-hit strings and a character with ten-hit strings are worth the percent
they deal, and the seven hits of a drill add up to the same record total as one hit of the same percent (3.7 shows
it is exact in the sim's integers).

### 2.2 Shapers: drives that change the damage

A shaper is passive. It does not fire. It changes a property of the player's own hits, and several shapers all apply
to the same hit.

| property | what the player sees | engine seam | limit | status |
|---|---|---|---|---|
| **word on damage** | the damage sets Burning, Chilled, Marked or Shocked | the sim's status windows | per word | exists [R] |
| **bonus damage** | extra percent, **proportional to the damage dealt**; the hit's own knockback is unchanged | `gd_envoy_percent` [R] | +100% of the damage | exists [R] |
| **element** | the engine's own fire, electric or ice hit effect and sound on the player's move | the hit's `element` is copied to the victim where a hit connects (`ftcoll.rs:510`, `:840`) [R]; a per-slot override read there | cosmetic plus the word. The ice element can freeze [K], so it is allowed only on a KO | new, S, [U] |
| **shield damage** | shields shrink faster; a break is an event | the capsule's shield-damage field (`x34`) where a hit meets a shield [I] | +100% | new, S, [U] |
| **hitlag** | a heavier freeze on impact | `dmg.x195c_hitlag_frames` after `ftCommon_CalcHitlag` (`fighter.rs:2912`) [R] | +6 frames | new, S, [U]. It is per hit by nature, so a multi-hit move freezes longer in total. It changes no number, but it stays out of the slice |
| **size** | a slightly larger hitbox | `HitCapsule.scale` [R struct] | +25%, own moves only | new, M, [U]. Not in the slice |
| **knockback** | the hit launches further | `dmg.kb_applied`, the field `gd_envoy_absorb` already zeroes [R `envoy.rs:422`] | only through Grand Finale (8.3); +50% | new, S, [I] |

V3's first draft had an "extra hit" shaper (one more hit per move hit). It is gone: it was a per-hit rule. Bonus
damage does its job proportionally.

One new import carries the new rows: `gd_envoy_shape(slot, field, value)`. It writes a per-slot record in
`melee_modes::tu::envoy`, whose statics are covered by the snapshot [R, the module's header]. The sim writes it at
frame end from its own hashed state; the engine reads it on the next frame. Values are x1000 integers [R `codes.rs`].

Rules that keep Melee readable:

- A shaper never changes the knockback of the player's own moves. Only one keystone does (8.3).
- One element is shown per fighter: the leftmost element shaper's (order, 5). Every shaper's word still applies.
- Shapers act on the owner's moves and projectiles only, never on a teammate's.

**Words are count-free too.** Three rules cover every status:

1. A word that only **refreshes** (Chilled, Marked, Haste, one stack of Burning) may be set by any damage. Hitting
   seven times refreshes it seven times, which is the same as once.
2. A word that **stacks** (Burning past one stack) stacks by damage: one stack per 8% dealt to that foe.
3. A word that is **used up** (Shocked) is given by a trigger drive charged by damage, never by "each hit".

### 2.3 Technique: what the port can see

The sim polls each fighter's action state every frame and already derives landings, jumps and aerial hits from it
[R `envoy_sim` lib.rs:117-123, 1529]. Technique is the same method: a state entered, and the state before it.

| technique | detection | state ids [R `melee_types/src/gen.rs`] | engine work |
|---|---|---|---|
| **L-cancel** | an aerial landing entered while the trigger timer is inside the window: the engine's own test is `x67F < xE4` in `ftCo_LandingAir_EnterWithLag` [R `ftCo_LandingAir.rs:65`] | LandingAirN-Lw 70-74 | **one event bit at that line, S.** The sim cannot read the timer: `gd_fighter_read` returns a fixed 20-word record without it [R `melee_modhost` lib.rs:536] |
| **wavedash / waveland** | LandingFallSpecial entered from EscapeAir. A wavedash if the jump squat was within 10 frames, else a waveland [J] | 43 from 236; KneeBend 24 | none (a hook row also exists [R `hooks-v1.tsv`]) |
| **tech** | a Passive state entered | 199-204 | none |
| **ledge grab / ledge option** | CliffCatch entered; then a climb, attack, roll or jump | 252; 254-263 | none |
| **ledgedash** | CliffWait, then LandingFallSpecial within 30 frames [J] | 253, 43 | none |
| **shield drop** | Pass entered from a Guard state | 244 from 178-181 | none [I] |
| **power shield** | GuardReflect entered | 182 | none [I] |
| **out of shield** | an attack, grab or jump within 4 frames of leaving Guard [J] | 178-181 | none |
| **move variety** | the attacker's action state when damage lands: a different attack state is a different move | the attack states | none |
| **taunt** | Appeal entered (D-pad up, `ftCo_AppealS.rs:76` [R]) | Appeal | none |
| **offstage damage (edgeguard)** | the victim's position is past a ledge or under the stage when damage lands | positions [R `gd_fighter_get_pos_*`] | a table of ledge x and floor y per stage in the run's stage list (5 stages today [R lib.rs:115]) |
| **edgeguard KO** | a foe KO'd within 120 frames of offstage damage without landing between [J] | the sim's KO detection [R] | none |
| **meteor** | damage whose launch angle points down on an airborne foe | - | needs the hit's angle: one event word, S, [U] |

The "foe held" row of the first draft is gone. Nothing needs to know whether a foe is in hitstun any more, which
removes the most fragile piece of the old design.

Where a row needs engine work, the pattern is one word: `gd_envoy_events(slot) -> u32`, bits set by the engine at
the exact site during the frame, cleared at frame start, kept in the snapshot. Hook callbacks and polls run inside
the sim's frame, so a resimulation raises the same events [R core v2 2.7].

**A technique never pays by itself.** It leaves a **mark**, a short time window on the fighter ("L-cancelled": 20
frames; "wavedashed": 60 frames; "from the ledge": 2 seconds) [J]. A drive pays when **damage is dealt inside the
window**, or when the technique could only happen in a fight (a tech, a power shield). So a player cannot wavedash in
a corner to charge drives, and the sim needs no counter of techniques.

## 3. Heat

### 3.1 The rules

One Heat meter per **player** [D]. Not per fighter, not per foe.

| rule | value | notes |
|---|---|---|
| **gain** | 1 Heat per 1% dealt to a foe by your moves | times the record's coefficient for fires (4) and word damage (0.2); times your gain multiplier (3.3) |
| **overkill** | the part of the damage that lands above 150% on a foe counts half | the foe's percent before the damage is read [R `gd_fighter_get_percent`]; the damage is split at 150, so it is still exact for multi-hits. Bosses with HP have no such line |
| **decay** | none | argued in 3.2 |
| **loss when hit** | 2 Heat per 1% you take | any source except yourself and a teammate (9.6); never below 0 |
| **you lose a stock** | Heat halves, no pay | "any KO halves it; yours pays" |
| **a KO (cash-out)** | pays on the Heat you hold (3.5), then you keep half | every player who dealt 10% or more to that foe on that stock cashes out |
| **a tag** | nothing changes | Heat belongs to the player [D] |
| **a fight starts** | Heat is 0 | each fight has its own arc; scaling drives can raise the start (7.1) |
| **cap** | 150 | drives and keystones can raise it; never above 250 |
| **over the cap** | discarded | a full meter says "go and take the KO"; one economy drive turns it into coins (8.1) |
| **frozen** | during the tag cinematic and the reward freeze | the sim does not step Heat while the fight is stopped |

### 3.2 Why these and not the alternatives

**No decay.** The owner's rule is that Heat carries through neutral [D]. I looked for a reason to add a slow decay
anyway and did not find one that survives:

- A decay is a combo timer under another name. It pays fast characters, who touch the foe more often, which is the
  first objection again.
- Nothing is gained by stalling. Heat does nothing for a player who is not dealing damage, and Heat over the cap is
  thrown away.
- With no decay the meter moves only when something visible happens: you deal damage, you take damage, somebody is
  KO'd. Every change has a cause the player saw.

What stops "sit on full Heat forever" is not time but three events: a KO halves it (which is also the pay-off), a
boss "break" halves it (9.6), and being hit drains it at twice the rate it fills. If the playtest shows a player
parked at Blazing while timed drives do the work, the fix is a cooling rule (after 10 seconds with no damage dealt,
lose 5 Heat a second). It is not in the slice.

**Loss scaled by damage taken, 2 for 1.** The three candidates:

| loss rule | what goes wrong |
|---|---|
| a flat chunk per hit (say 25) | it counts hits. A foe's drill or rapid jab would strip the meter; a 1% laser would cost as much as a smash. This is the second objection, turned on the player |
| a fraction per hit (say a quarter) | the same: seven small hits leave 13% of the meter, one big hit leaves 75% |
| **2 Heat per 1% taken** | nothing counts hits. A jab (3%) costs 6. A typical hit (10%) costs 20, about an eighth of a full meter: a visible chunk. The hardest single hits in the game (20-30% [K]) cost 40-60, at most 40% of a full meter |

So one touch never loses everything, and a full meter survives about 75% taken with no reply. Losing an exchange
badly (a 60% punish) costs 120, which is nearly everything, and should.

Two for one, not one for one, because at one for one Heat is simply "damage dealt minus damage taken" and a hit
taken feels weightless. At two for one, trading evenly loses Heat, so the meter rewards **winning exchanges**
[O], which every character can do.

**A KO keeps half.** If a KO took everything, packs of weak foes would never leave Cold: each small KO would empty
the meter. If a KO took nothing, a good player would sit at the cap from the second minute of a fight. Half gives a
sawtooth in both cases (3.4).

**Fights start at 0.** Carrying Heat between fights would let one good fight decide the next three, and would make
the first seconds of a fight identical for every build. Starting cold makes "how fast does this build get hot" a
property of the build.

### 3.3 The Heat step (fixed order, on every piece of damage a player deals to a foe)

1. The engine applies the hit as retail does, with the shapers of 2.2.
2. The sim reads the damage. Self-damage, friendly damage and hazard damage stop here (9.6).
3. **Eligible damage** = damage x the record's coefficient, with the part above 150% on the foe halved.
4. **Gain multiplier**: starts at x1. Each `xGain` drive whose condition this damage meets multiplies it, **left to
   right**. Capped at x4.
5. Heat rises by eligible damage x the gain multiplier.
6. **Event drives**, left to right: a `+Heat` drive whose event this is adds its amount; an `xHeat` drive multiplies
   the Heat held. Heat is clamped to the cap after each.
7. **Charge**: each trigger drive whose source this damage matches gains damage x coefficient. If the record is a
   fire, only drives to its right gain.
8. Full drives fire, left to right, 6 frames apart, as a chain (4).
9. The damage is added to that player's tally on that foe (for the cash-out, 3.5).

When a player **takes** damage from a foe or the stage: Heat falls by 2 x the damage; drives charged by damage
taken gain it; "without taking damage" streaks reset.

Three kinds of Heat drive, then:

| kind | what it does | where it lives |
|---|---|---|
| **+Heat** | a flat amount on an event (an L-cancelled aerial that dealt damage: +4) | Common |
| **xGain** | multiplies how fast damage becomes Heat while a condition holds (in Haste: x1.5) | Rare and up |
| **xHeat** | multiplies the Heat you hold, once, on a hard event (a meteor smash: x1.5) | Mythic |

### 3.4 What Heat does while you hold it

| state | Heat | drive fires deal | what else |
|---|---|---|---|
| **Cold** | 0-24 | x1.0 | - |
| **Warm** | 25-59 | x1.25 | - |
| **Hot** | 60-109 | x1.5 | drives and sets can read "Hot or hotter" |
| **Blazing** | 110 and up | x2.0 | the top state; drives can read it |

- The **state scales every drive fire**. That is where the build-up shows: the same Ember is a puff at Cold and a
  blast at Blazing. It also makes chains reach further when you are hot (4.2).
- The player's **own moves are unchanged**: retail percent and knockback. A move that launched differently
  depending on a meter could not be followed up. One keystone breaks this on purpose (8.3).
- States are thresholds, not a curve, so there are four things to read, and a drive rule can say "while Hot" in
  two words.

Hand-worked pacing with **no drives at all** [J]. The "reference stock": the foe is KO'd at 120% total; the player
deals it in four lots of 30% and takes 10% between lots.

| moment | Heat |
|---|---|
| deal 30 / take 10 / deal 30 / take 10 / deal 30 / take 10 / deal 30 | 30, 10, 40, 20, 50, 30, **60** |
| first KO | pays on 60, keeps 30 |
| second KO, same pattern | reaches **90**, keeps 45 |
| third KO | reaches **105**, keeps 52 |

So ordinary good play (four for one) reaches Hot at the first KO and stays just under Blazing. An even trade
(deal 30, take 15 each time) is back at 0 after every exchange. Blazing early in a fight has to come from the
build. That is the room drives have.

### 3.5 The cash-out

When a foe loses a stock, every player whose tally on that foe (this stock) is 10% or more cashes out. Who landed
the last hit does not matter, so a foe that falls to its own mistake after you damaged it still pays, and co-op
partners never fight over a KO.

| pays | amount | cap |
|---|---|---|
| **coins** | Heat / 10 | none (the meter is capped) |
| **the active's charge** | 1 pip per 100 Heat cashed, the remainder carried | - |
| **the flare** | Heat / 10 percent to every **other** foe in play, no launch | 15% each |
| **drives that read a cash-out** | Wildfire, Leech, Snowball, Magpie (8.1) | each its own |

Then the player keeps half. The pay is multiplied by what the foe was: a pack minion x0.5, a fighter x1, an elite
x1.5, a boss break x1, the boss's last break x2 [J].

There is a small reason to delay a KO: to cash out at more Heat. It is bounded three ways: the cap (at most 15
coins from a fighter), the overkill rule (a foe above 150% gives half), and the 2-for-1 loss if the plan goes wrong.
`sim.py` reports it (12).

### 3.6 Caps

| thing | cap | why |
|---|---|---|
| Heat | 150; 250 with every cap-raiser | the meter has an end the player can see |
| the gain multiplier | x4 | a full meter in about 40% dealt at most |
| one fire | 30% (a launching timed fire 40%) | as before |
| the state's scale on fires | x2 | Blazing |
| the flare | 15% per foe | it is a bonus, not a second KO |
| a chain | 4 links, 6 frames apart, 20 frames between chains | 4.1 |
| knockback | never, except Grand Finale: +50% | one keystone owns this |

A fighter dies at about 150%, so damage has a natural ceiling anyway; bosses with real HP and NG+ foes are where
the uncapped part of a build shows.

Because the Heat state now supplies up to x2, level-1 fire numbers drop by about 40% from V2 (Ember 6% to 4%), and
Burning drops from 2% to 1% a second per stack [R `codes.rs` for the V2 values; the new ones J].

### 3.7 Determinism

All of it is integer state in the sim's one hashed struct [R core v2 2.7]. No floats, no chance rolls.

- **Damage** enters once, at the hook: the engine's `f32` percent [R `on_damage(h, d: f32)`] is rounded to the
  nearest 0.001% and stored as an `i32` (1% = 1000). V2 already converts at the same place [R `x100`]. The engine's
  float is the same on both peers, or rollback would already be broken [I].
- **Heat** is an `i32` in millionths of a point (150 Heat = 150,000,000). Gain is `damage x multiplier`, both
  x1000, **with no division**, done in `i64` and clamped to the cap. Because nothing is divided or rounded per
  record, seven records of 2% and one of 14% add exactly the same Heat. The only grain is the rounding at the
  hook: at most 0.0005% per hit.
- **Charge** is the same unit (millionths of a percent), in `i64`. When a drive fires, its need is subtracted and
  **the remainder stays**. Without that, one big hit would waste what a multi-hit move keeps.
- The **gain multiplier** is a product of x1000 values, rounded down after each step, left to right, then capped.
- **Coins and pips** carry their remainders the same way, so no cash-out loses a fraction.
- The overkill split, the tally per foe and player (4 x 2 `i32`), the Burning damage counter per foe, technique
  marks, streak counters, the chain queue, the active's pips and coins are all plain integers in the same struct.
- A coefficient scales **charge**, not probability, so there is no dice roll to desync and the player can plan a
  chain.

The manifest's `state_bytes` is 4096 today [R lib.rs:2149] and will need raising.

**Fire timing is the one thing that is not split-proof.** A 14% hit and a 7-hit drill fill a drive at the same total,
but the drill may fill it a few frames later, so the fire can come out a few frames later and, rarely, in a
different Heat state. The amounts of Heat and charge are identical; the frame of a fire is not. I think that is
right: it is how the move really landed.

## 4. Proc chains

### 4.1 The rule

- Every trigger drive has a **need** in percent ("every 40% dealt") and a **coefficient** in the pool data: 1.0 for
  a launching timed blast, 0.5-0.6 for a damage burst, 0.3-0.4 for a status fire (which now always carries a little
  damage, so it has something to pass on), 0 for heals and buffs.
- A fire deals its number x the level scale x the Heat state's scale. That damage is a record with `coef` = the
  drive's coefficient x its parent's. It feeds **Heat**, and it charges **only trigger drives to its right** (5),
  both by damage x `coef`.
- A drive that fills fires 6 frames after its parent, once per chain. A chain is at most **4 links**. One chain at a
  time per fighter; the next may start 20 frames after the last link (V2's fire gap [R `codes.rs`]).
- A chained fire never launches. Only timed drives launch [D]; a launching timed blast keeps coefficient 1.0, so it
  is the best chain starter.
- **Word damage** (Burning ticks, the extra a Marked foe takes) is a record at 0.2. It feeds Heat. It does **not**
  charge drives, unless a keystone says so (8.3), and then never a drive that gives that word.

### 4.2 A worked chain (hand-worked)

Slots, left to right: Ember (every 40%, 4%, 0.5) | Bolt (every 30%, 5%, 0.6) | Hex (every 50%, 2% and Marked, 0.4).
Ember holds 33 of 40, Bolt 19.5 of 30, Hex 36 of 50. The player is **Hot** (fires x1.5). A 9% move lands.

| frame | event | effect |
|---|---|---|
| 0 | 9% lands | Heat +9. Ember 42: full, 2 left over. Bolt 28.5. Hex 45 |
| 0 | Ember fires: 4% x 1.5 = 6%, Burning | record 6% at 0.5: Heat +3.0; Bolt +3.0 = 31.5, full; Hex +3.0 = 48 |
| 6 | Bolt fires: 5% x 1.5 = 7.5% | record at 0.5 x 0.6 = 0.3: Heat +2.25; Hex +2.25 = 50.25, full |
| 12 | Hex fires: 2% x 1.5 = 3%, Marked | record at 0.12: Heat +0.36. Nothing to its right: the chain ends |

One 9% move became 25.5% and 14.6 Heat. The same move at **Cold**: Ember's 4% passes 2.0 (Bolt 30.5, full), Bolt's
5% passes 1.5, and Hex stops at 48.5. Two links, not three. Heat makes chains longer, not only bigger.

Had Bolt been left of Ember, Ember's fire would not have reached it. The same three drives in another order are a
different build.

### 4.3 No loop is possible: the check

| path a loop could take | why it ends |
|---|---|
| inside one chain | charge only moves rightward and each drive fires once per chain, so a chain ends in at most 4 links (5 with Conductor's Line) however many slots there are |
| fire -> charge -> fire, across chains | a fire returns less charge than it cost. Lint: for every damage-charged trigger, coefficient x fire damage / need is at most 0.1 at level 1 Cold and at most 0.5 at level 3 Blazing. Ember: 4 x 0.5 / 40 = 0.05; at its worst 19.2 x 0.5 / 40 = 0.24. The leftmost trigger is charged only by moves, time and technique |
| fire -> Heat -> bigger fire | bounded three times: the state's scale stops at x2, Heat stops at the cap, one fire stops at 30% |
| word damage -> charge -> the word again | word damage charges nothing by default. Under Plague Bearer it charges at 0.2 but never a drive that gives Burning, and every status ends on its own timer |
| your fire -> the foe's "when damaged" drive -> its fire -> your "when damaged" drive | fire damage taken charges at the record's coefficient, so each bounce is at most 4 x 0.6 / 20 = 0.12 of the last. It dies in one or two bounces |
| Echo and Mirror | a neighbour drive never acts on another neighbour drive (lint, and inert at run time) |
| Relay, Volley | each drive fires once |
| time | the hard limit stays: 4 fires every 38 frames per fighter, about V2's 6 a second [R core v2 3.3] |

Drives charged by a clock, a tag or a cash-out (Nova, Relay, Wildfire) are outside the "returns less than it cost"
lint, because damage does not charge them; a Blazing level-3 Nova can fill a drive to its right in one fire. That is
its job, once every 12 seconds. Every damage-charged trigger in 8.1 passes the lint by hand: the tightest are Bolt,
Shatter, Feather Storm, Comet and Thunderhead, all at 0.1 at level 1 and 0.48 at their worst.

## 5. Slot order

1. **Heat steps apply left to right** (3.3). From 60 Heat, `+4` then `x1.5` gives 96; the other order gives 94.
   `+Heat` goes left of `xHeat`.
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

V2 has 3 slots, then 4 after the first boss [R]. V3 proposes 3, 4 after the first boss and 5 after the second. Five
matches the limit that makes every pick a cut in Balatro [W10]. The orbit budget was 4 [R core v2 3.3]; shapers are
drawn smaller and closer to the body than triggers, so the loud objects stay near four.

## 6. Sets, and the active drive

### 6.1 Families

The six model bodies and colours stay [D]. Each has a job with Heat:

| family | V2 meaning | V3 job |
|---|---|---|
| RED | damage | fire, bursts, chain starters; spends the Heat state |
| GREEN | speed | Haste and ground technique; makes Heat |
| BLUE | defence | keeps Heat: less loss, defence turned into Heat |
| YELLOW | air | aerial damage, edgeguards; makes Heat offstage |
| PURPLE | status | words, and pay-offs that read words |
| WHITE | wild | order, retriggers, multipliers, scaling, coins |

A fifth word joins the four: **Shocked** (the next fire that damages the foe fires twice, then it ends). It is the
retrigger as a status.

### 6.2 Sets (three different drives of one family on one fighter)

| set | look | rule |
|---|---|---|
| RED: **Inferno** | the fighter's hits trail fire | Burning stacks to 6, and its ticks feed Heat at 0.5 instead of 0.2 |
| GREEN: **Tempest** | afterimages at all times | while you are Hot or hotter you are always in Haste |
| BLUE: **Bastion** | a faint shell | a tech, a power shield or a shield drop fires your leftmost drive |
| YELLOW: **Skybound** | wing glints at the shoulders | aerial damage counts double for charge |
| PURPLE: **Blight** | a dark haze on afflicted foes | a cash-out pays half again for each word on the KO'd foe, to x2.5 |
| WHITE: **Conductor's Line** | the orbit becomes one bright ring | chains run one link longer and the first link keeps full strength |

A set is per fighter, as drives are [D]. Levels do not count toward it; three different drives do. A fighter with
five slots holds one set and two others, so the pair can hold two sets. A set is shown by one persistent look.

### 6.3 The active drive

- **One per player**, not per fighter: it stays through a tag. It does not take a drive slot.
- **Button: D-pad right.** D-pad down is the tag [R `envoy_sim` lib.rs:1779, D E12]; D-pad up is the retail taunt
  [R `ftCo_AppealS.rs:76`]; D-pad left and right are read only in the reward freeze [R lib.rs:833]. So right is free
  in a fight, and left stays free for later.
- **Charge:** pips. One pip is 100 Heat cashed out (3.5). An active needs 2-4 pips. Pips are kept between fights.
- **Use:** on the ground or in the air when actionable, as the tag is. It is the player's own timing, so by the
  standing rule an active may launch [D].
- Got at the rest stage, from the first boss, or as a rare card. Taking a new one replaces the old.

| active | pips | effect |
|---|---|---|
| **Overdrive** | 3 | for 5 seconds Heat fills twice as fast and damage you take costs none |
| **Volley** | 3 | every drive fires now, in order, as one chain with no link limit |
| **Detonate** | 2 | every word on every foe bursts for 6% a stack and ends; each burst is a record at 0.5 |
| **Recall** | 2 | warp to the centre of the stage, intangible for 1 second |
| **Double Down** | 4 | Heat jumps to the cap. If you take damage in the next 5 seconds, you lose all of it |

## 7. Scaling, deals, economy, tiers and pools

### 7.1 Scaling drives

A scaling drive keeps a counter for the whole run, in the run record [R record v4], shown on its orbiter as pips.

| drive | grows when | gives |
|---|---|---|
| **Cinder Heart** | a foe is KO'd while Burning | Heat fills 2% faster, per KO, to +40% |
| **Snowball** | you cash out while Blazing | +2 Heat at the start of every fight, per cash-out, to +40 |
| **Magpie** | (not a scaler; it feeds them) | cash-outs pay half again in coins; Heat over the cap becomes coins |

Scalers are weak when found and strong when found early, so the same card is a different decision in act 1 and
act 3.

### 7.2 Keystones as devil deals

Timing as decided: one at the run's start, one after each of the first two bosses [D]. Each is one rule and one
price. Every keystone now bends one rule of section 3 or 4, so a keystone picks the kind of run. The eight are in
8.3.

### 7.3 Coins, the shop and the pact

| coins from | amount |
|---|---|
| a cash-out | Heat / 10, times what the foe was (3.5) |
| a fight cleared | 20; +10 with no stock lost |
| an edgeguard KO | +10 |
| a taunt inside 2 seconds of a KO | +5 (the retail taunt; you stand still for it) |
| interest at each fight's start | +1 per 10 held, to +5 |

Hand-worked: a fight with three fighter KOs at 60, 90 and 105 Heat pays 25 coins from cash-outs and 20-30 for the
clear. A Common costs 40.

Coins belong to each player in co-op, as drops do [D].

The **rest** encounter [D Q5] is the shop. Until the rest stage exists (core v2's milestone V8) it is a screen at
the same point in the run.

| on sale | price |
|---|---|
| three drives, from the shop pool | Common 40, Rare 80, Mythic 150 |
| a level for a drive you wear | 60 to level 2, 120 to level 3 |
| an active | 80 |
| a reroll of the three | 10, then +10 each time |
| replacing a worn drive with a bought one | free; the old one is gone (there is no bag [D]) |
| reordering | free |

**The pact**: once per act the rest stage offers one Mythic drive for **one stock from the pool**. In this game the
shared stock pool [D] is the health a devil deal would take.

### 7.4 Rarity tiers and editions

Tiers ration the multipliers.

| tier | share of the pool | holds | mark |
|---|---|---|---|
| **Common** | about half | triggers, shapers, `+Heat`, technique | the plain glass shell |
| **Rare** | about a third | `xGain` on a condition, retriggers, scalers, set-makers | a coloured beam on the floor drop (the old game had a beam per rarity [R core v2 1.1]) |
| **Mythic** | the rest | `xHeat`, copy, the duo drives | a beam and a ring |

Levels (a duplicate, to 3) stay as they are and are separate from tier [D].

**Editions** (after the slice): a drop can carry one modifier, whatever the drive: Tempered (its fire counts one
Heat state higher), Radiant (+5 Heat when it fires), Prismatic (its coefficient is 1.0), Hollow (takes no slot).

### 7.5 Pools by encounter kind

| source | pool | rule |
|---|---|---|
| duel drop | Common 80%, Rare 20% | any family |
| pack drop | Common, weighted to PURPLE and RED | many foes: spreading and chain drives |
| shift room drop | Common and Rare, weighted to GREEN and YELLOW | movement and technique |
| elite | Rare | one of its two drops is **a drive the elite carried** [D]: you take what was used on you |
| boss | Mythic, a pick of three; then the keystone pick [D] | the first boss also offers the first active |
| pick of three (every boundary) | one card from the encounter's pool, one that connects to what the fighter wears, one free | "connects" means: a word it gives or reads, its family (toward a set), or a technique the player has used this run |
| shop | all tiers at their prices | the only place to buy a level |
| pact | Mythic | one stock |
| duo drives | Mythic | offered only when the fighter wears a drive from each of the two families |

A drive declined in a pick is half as likely for the rest of the run [W9]. A level-3 drive is never offered again
[D]. Unlocks stay content-only [D]. The "draw" here is the run's existing seeded pick [R], decided at fight
boundaries, not in a fight.

## 8. The pool: 40 drives, 5 actives, 8 keystones

Kinds: **T** trigger (charges and fires; its coefficient in brackets), **S** shaper, **+H** adds Heat, **xG**
multiplies Heat gain, **xH** multiplies Heat held, **G** grows over the run, **E** economy, **N** neighbour. Tiers:
C, R, M. Numbers are level 1 and first guesses. "V2" marks a drive kept from V2, restated on damage.

How the old hit counts were converted: one old "hit" is taken as 8% [J], so "every 5 hits" became "every 40% dealt".
`sim.py` will move these.

### 8.1 Drives

| # | family | name | tier | kind | rule | job in a build |
|---|---|---|---|---|---|---|
| 1 | RED | **Ember** V2 | C | T (0.5) | Every 40% dealt: a fire burst, 4% and Burning. | chain starter |
| 2 | RED | **Kindle** | C | S | Your smash attacks set Burning. | a word with no charge needed |
| 3 | RED | **Bolt** V2 | C | T (0.6) | Every 30% dealt: lightning strikes the nearest foe, 5%. | the second link |
| 4 | RED | **Nova** V2 | R | T (1.0) | Every 12 seconds: a blast around you, 8%. | timed, launches; the full-strength starter |
| 5 | RED | **Wildfire** V2 | R | T (0.5) | A cash-out on a Burning foe: 10% and Burning to every other foe. | pack clear; each KO lights the next |
| 6 | RED | **Cinder Heart** | R | G | Each Burning KO: Heat fills 2% faster, this run. | scaler |
| 7 | GREEN | **Rush** V2 | C | T (0) | Every 50% dealt: Haste for 4 seconds. | gives Haste |
| 8 | GREEN | **Slipstream** V2 | C | T (0) | Tag in: Haste for 6 seconds. | gives Haste |
| 9 | GREEN | **Tailwind** V2 | R | xG | In Haste, Heat fills x1.5. | was "+25% damage" |
| 10 | GREEN | **Slick** | C | S | Damage within 1 second of a wavedash charges drives half again as fast. | technique as charge |
| 11 | GREEN | **Clean Landing** | C | +H | An L-cancelled aerial that dealt damage: +4 Heat. | technique as Heat |
| 12 | GREEN | **Perfect Form** | R | xG | While your last three aerial landings were L-cancelled: Heat fills x1.5. | goes with Clean Landing |
| 13 | BLUE | **Bulwark** V2 | C | T (0) | Every 15 seconds: an orb. Once struck it blocks everything for half a second. | protects Heat. The half second makes it proof against a foe's multi-hit |
| 14 | BLUE | **Thorns** V2 | C | T (0.4) | Every 20% you take: the attacker takes 4% and is Marked. | a fire that starts a chain while you are losing |
| 15 | BLUE | **Leech** V2 | C | S | A cash-out heals 1% for each 15 Heat, to 10%. | was "every 5th hit" |
| 16 | BLUE | **Riposte** | C | +H | A tech or a power shield: +6 Heat. | defence into Heat |
| 17 | BLUE | **Breaker** | R | S | Shield damage you deal counts as damage, at half. A shield break: +30 Heat. | shield pressure build; needs the shield seam [U] |
| 18 | BLUE | **Insulate** | R | S | Damage you take costs 1 Heat per 1%, not 2. | the Heat keeper |
| 19 | YELLOW | **Comet** V2 | C | T (1.0) | Every 40% of aerial damage: your next landing sends a shockwave, 4%. | player-timed, pops up; full strength |
| 20 | YELLOW | **Updraft** V2 | C | T (0) | Every 20% of aerial damage: an air jump back. | longer air strings |
| 21 | YELLOW | **Feather Storm** V2 | C | T (0.5) | Every 30% of aerial damage: three feathers, 2% each, at three different foes if there are three. | spreads words and damage across a pack |
| 22 | YELLOW | **Skyhook** | R | xG | Aerial damage fills Heat x1.5. | Heat for air play |
| 23 | YELLOW | **Gatekeeper** | R | xG | Damage to an offstage foe fills Heat x2. | edgeguard |
| 24 | YELLOW | **Meteor** | M | xH | A meteor smash: Heat x1.5. | the hardest condition, the biggest single step |
| 25 | PURPLE | **Frost** V2 | C | T (0.3) | Every 40% dealt: 2% and Chilled for 4 seconds. | gives Chilled |
| 26 | PURPLE | **Shatter** V2 | C | T (0.4) | Every 20% dealt to Chilled foes: ice shards, 5%, ending the Chill. | reads Chilled |
| 27 | PURPLE | **Hex** V2 | C | T (0.4) | Every 50% dealt: 2% and Marked for 5 seconds. | gives Marked |
| 28 | PURPLE | **Static** | C | T (0.3) | Every 15% dealt with specials: 2% and Shocked. | gives Shocked, the retrigger word |
| 29 | PURPLE | **Affliction** | R | xG | Heat fills a quarter faster for each word on the foe you damage, to x2. | reads every word |
| 30 | PURPLE | **Contagion** | R | S | A word you apply also lands on foes nearby. | packs; with Blight |
| 31 | WHITE | **Relay** V2 | R | T (1.0) | Tag in: your drives fire in order, as one chain; each keeps its own coefficient. | the tag as a chain starter |
| 32 | WHITE | **Echo** V2 | R | N | The drive to the right fires twice. | was "every 3rd fire repeats" |
| 33 | WHITE | **Conductor** | R | S | Chained fires keep full strength for one more link. | lifts every coefficient |
| 34 | WHITE | **Metronome** | C | +H | Damage from a move that is not one of your last three: +3 Heat. | variety; no technique needed |
| 35 | WHITE | **Crescendo** | M | xH | Each 50% you deal without taking damage: Heat x1.25. | the multiplier for clean play |
| 36 | WHITE | **Snowball** | R | G | A cash-out while Blazing: +2 starting Heat, this run. | scaler |
| 37 | WHITE | **Magpie** | C | E | Cash-outs pay half again in coins. Heat over the cap becomes coins, 1 per 10. | economy |
| 38 | WHITE | **Mirror** | M | N | Copies the drive to its left. | a second Crescendo, or a second Bolt |
| 39 | RED + PURPLE | **Steam** | M duo | xG | Damage to a foe both Burning and Chilled fills Heat x2.5. | needs two givers |
| 40 | YELLOW + RED | **Thunderhead** | M duo | T (0.4) | Every 20% of aerial damage to an offstage foe: a lightning bolt, 5%. | edgeguard into chains |

What changed and why, beyond swapping counts for percent:

- **Shatter, Static, Thorns, Steam** fired "on a hit" in the first draft. With a word on every hit (Winter Oath)
  that made a drill fire them seven times. They are charged by damage now, or (Steam) are a gain multiplier with
  nothing to re-trigger.
- **Comet, Updraft, Feather Storm** counted landings, jumps and aerial hits. They count aerial damage now. A jump is
  not a hit, but a count the player can run up without fighting is the same problem.
- **Feather Storm** was "the best feeder" because three small hits were three hits. Under damage it is worth its 6%.
  Its job is now spreading.
- **Crescendo** was "every 5th hit of a combo". It is the same idea on the new terms: a multiplier for damage dealt
  without being hit.
- **Anchor** (a hit out of a shield drop) is replaced by **Insulate**. The design needed a drive whose job is
  keeping Heat; the shield drop is still read by Bastion.
- Not carried from V2: **Spire** and **Surge**, as before.

Checks the lint keeps or gains (11): every word read has two givers (Burning: Ember, Kindle, Plague Bearer; Chilled:
Frost, Winter Oath; Marked: Thorns, Hex; Haste: Rush, Slipstream; Shocked: Static, and Bolt at level 3). Every `xG`
drive is Rare or Mythic; every `xH` is Mythic. Every technique drive has a drive of the same family with no
technique condition. **No rule counts hits**: the lint rejects a charge source that is a count of hits, landings or
jumps.

### 8.2 Actives

The five of 6.3: Overdrive, Volley, Detonate, Recall, Double Down.

### 8.3 Keystones (8)

Prices are V2's [R `codes.rs` `PRICE_TEXT`].

| # | name | rule | price | what it does to Heat |
|---|---|---|---|---|
| 1 | **Plague Bearer** V2 | Each 8% you deal to a foe adds a Burning stack, to 3. Burning ticks charge your drives (except those that give Burning). | Fragile | ticks at 0.2 keep drives filling while you chase. Fragile means 25% more damage taken, so 25% more Heat lost |
| 2 | **Echo Oath** V2 | Every drive fire repeats once at half strength. | Weak | a retrigger on everything; your own moves make 20% less Heat |
| 3 | **Overclock** V2 | Drives charge twice as fast. | Bleed | more chains, each fire costing 1%. Bleed is self-damage, so it costs no Heat |
| 4 | **Twin Soul** V2 | The benched fighter's drives join your row and fire too. | Lock | a ten-slot line: two sets, long chains, few tags |
| 5 | **Winter Oath** V2 | All your damage Chills for 2 seconds. | Slow | a word always on; Shatter, Steam and Affliction are always live |
| 6 | **Glass Core** V2 | Heat fills twice as fast. You take 60% more. | own | Blazing in half the damage, and 60% more Heat lost per hit taken, because loss follows damage |
| 7 | **Grand Finale** | Your smash attacks launch harder with Heat: +1% knockback per 3 Heat, to +50%. A KO from a smash attack pays half again and keeps no Heat. | Weak | the only knockback scaling. Build Heat, then spend all of it on the kill |
| 8 | **All In** | A KO keeps all your Heat. Damage you take costs 3 Heat per 1%, and losing a stock takes all of it. | own | the one way to stay at the top; one bad exchange ends it |

## 9. Five builds, and how each snowballs

All numbers here are **hand-worked** on the reference stock of 3.4 (the foe is KO'd at 120% total; the player takes
10% for every 30% of move damage dealt before the KO), with level-1 values and five slots. They are rough and exist
to show the shape of each build. Where a build's fires do part of the 120%, the moves needed drop, and so does the
damage taken. With no drives: 120% of moves, 30% taken, **60 Heat** at the KO.

| build | moves needed | taken | Heat at the first KO | state | first cash-out |
|---|---|---|---|---|---|
| no drives | 120% | 30% | 60 | Hot, only just | 6 coins |
| Firestorm | about 72% | 25% | about 38 | Warm | 4 coins, and Wildfire |
| Tech Monk | 120% | 40% | about 140 | Blazing | 21 coins; the KO comes early |
| Frozen Clockwork | about 64% | 20% | about 80 | Hot | 16 coins |
| Sky Pirate | about 87% | 20% | about 127, 150 on a meteor | Blazing | 15 coins + 10 |
| Twin Engine | about 56% | 10% | about 108 | Hot | 16 coins |

The table shows the two axes. Chain builds turn Heat into speed: the foe dies to far fewer moves. Heat builds turn
play into Heat: the meter is high, and the pay and the keystone do the rest. Fires feed Heat at their coefficient,
below 1, so a build cannot make Heat from fires faster than it makes damage.

### 9.1 Firestorm (proc chain)

- **Pieces:** Plague Bearer; Ember | Bolt | Wildfire | Cinder Heart, then Conductor in the fifth slot. Four RED
  drives make **Inferno**.
- **How it runs:** every 8% dealt adds a Burning stack. Say Burning averages 2.5 stacks for 12 seconds: 30% from
  ticks. Ember fires on moves only (it gives Burning, so ticks do not charge it); Bolt is charged by moves, by
  Ember's fire and by ticks.
- **The sum (hand-worked):** moves 72% + ticks 30% + Ember once (5% at Warm) + Bolt twice (12.5%) = 119.5%. Bolt's
  charge: 72 + 2.5 from Ember + 6 from ticks = 80.5, two fires. Heat: 72 + 6 (ticks at 0.2) + 2.5 + 7.5 = 88
  gained; two exchanges at Fragile cost 50; **38 at the KO**. The KO fires Wildfire: 12.5% and Burning on every
  other foe.
- **Snowball:** Inferno lifts Burning to 6 stacks and its ticks to 0.5 for Heat (the 6 above becomes 15 or more).
  Each Burning KO makes Heat fill 2% faster through Cinder Heart. Conductor keeps the second link at full strength.
  By act 3 the same play is Hot, so every fire is half again as big.
- **Weak to:** single foes with long invulnerability; the Fragile price, which costs Heat as well as percent.

### 9.2 Tech Monk (technique into Heat)

- **Pieces:** Grand Finale; Clean Landing | Metronome | Perfect Form | Crescendo. Two GREEN, two WHITE: no set.
- **How it runs:** 120% of moves, of which six L-cancelled aerials (60%) and a rotation of four or more moves. Weak
  makes the stock take longer: four exchanges, 40% taken.
- **The sum (hand-worked):** moves 120. Perfect Form, live for the last 80%: +40. Clean Landing, six times: +24.
  Metronome, eight changes of move: +24. One clean 50% streak for Crescendo, at about 60 Heat: +15. Gained 223, lost
  80: **about 140 at the KO**, Blazing for the last third. With no drives the same play ends at 40.
- **Grand Finale:** at 140 Heat a smash launches 47% harder, so the KO really comes well before 120% (not
  estimated here; the knockback seam is [U]). That KO pays half again (21 coins, 2.1 pips) and keeps nothing.
- **Snowball:** the coins buy Mirror next to Crescendo: x1.25 twice for every clean 50%. A cap-raiser later lets the
  meter pass 150.
- **Weak to:** a player who drops L-cancels. Metronome and Crescendo need none, so the build has a floor.

### 9.3 Frozen Clockwork (words and retriggers)

- **Pieces:** Winter Oath; Echo | Shatter | Static | Hex | Affliction. Four PURPLE: **Blight**.
- **How it runs:** all damage Chills, so every 20% dealt fires Shatter, and Echo makes it fire twice: 12.5% of
  shards at Warm. Static Shocks the foe every 15% of special damage, so the next Shatter fires twice again.
  Affliction reads the words: with one to two on the foe, Heat fills about x1.4.
- **The sum (hand-worked):** moves 64% + three Shatters (37.5%) + one Shocked repeat (12.5%) + Hex, Static and the
  Marked extra (about 9%) = 123%. Heat: moves 64 x 1.4 = 88; shards 50 x 0.4 x 1.4 = 28; the rest about 4. Gained
  120, lost 40: **about 80 at the KO**. Blight pays half again per word on the KO'd foe: with two words, 16 coins.
- **Snowball:** Steam (the duo, if a RED giver is added) lifts gain to x2.5 on a Burning, Chilled foe. Contagion
  spreads the words across a pack, and Feather Storm delivers them.
- **Weak to:** bosses, where Chilled and Marked act at half strength [R `codes.rs`]; the Slow price.
- **Tuning note:** this needs the fewest moves of any build but one, even after Shatter was cut from 6% every 15%
  to 5% every 20% and its coefficient to 0.4 (to pass the lint of 4.3). It is the first thing `sim.py` should check.

### 9.4 Sky Pirate (air and edgeguard)

- **Pieces:** All In or Overclock; Feather Storm | Skyhook | Gatekeeper | Meteor | Updraft, all YELLOW:
  **Skybound**. Active: Recall.
- **How it runs:** three quarters of the damage is aerial and a quarter of it lands offstage. Skybound doubles
  aerial charge, so feathers come every 15% of aerial damage.
- **The sum (hand-worked):** moves 87% (22 on the ground, 43 in the air over the stage, 22 offstage) + four Feather
  Storms (30% at Warm) = 117%. Heat: 22 + 43 x 1.5 + 22 x 1.5 x 2 + 30 x 0.5 = 167 gained, 40 lost: **about 127 at
  the KO**. If the KO is a meteor smash, Heat x1.5 reaches the cap: 15 coins, plus 10 for the edgeguard KO.
- **Snowball:** edgeguard KOs are the best coin source in the game, so this build reaches the shop's Mythics first.
  With All In the next stock starts at the cap. Thunderhead (the duo) turns offstage aerial damage into lightning.
  Recall is the way back from deep.
- **Weak to:** stages with walls or no ledges, and the boss fights with no ledge play. All In makes one bad trade
  offstage cost the meter.

### 9.5 Twin Engine (the tag, and co-op)

- **Pieces:** Twin Soul; fighter A: Relay | Ember | Bolt | Mirror; fighter B: Slipstream | Tailwind | Snowball |
  Magpie. Active: Volley.
- **How it runs:** with Twin Soul both rows are one line of eight. A tag fires Relay: Ember, Bolt and Mirror's copy
  of Bolt in order, 17.5% at Warm. Slipstream gives Haste on the same tag, and Tailwind makes Heat fill x1.5 for
  those six seconds. Lock allows a tag every 10 seconds: two in the stock.
- **The sum (hand-worked):** moves 56% + two Relay volleys (35%) + Ember once, Bolt twice, Mirror twice (30%) =
  121%. Heat: moves 56, three quarters of it in Haste: 77. Fires 65 at about 0.57, mostly in Haste: about 51. Gained
  128, lost 20: **about 108 at the KO**, two points under Blazing. Magpie: 16 coins.
- **Snowball:** one more level anywhere tips the first KO into Blazing, and from then Snowball adds 2 starting Heat
  per cash-out. Volley is a second Relay on a button. In co-op one player builds the chain row and the other the
  Heat row; each has their own meter and they meet only through the words on a foe.
- **Weak to:** the Lock price, which makes each tag a commitment.

A sixth shape exists without being named as a build: the banker (Magpie, Snowball, interest), which is weak for an
act and then buys whatever it wants.

### 9.6 Edge cases, one by one

"Counts" means: feeds Heat and charges drives.

| case | rule | why |
|---|---|---|
| **drill-type multi-hits** (Fox's down air) | counts by the percent each hit deals. Seven hits of 2% are one 14% | the owner's rule; exact in the sim's integers (3.7) |
| **rapid jabs** | the same: each is worth about 1%, and retail's stale-move decay [K] makes each repeat worth less | spamming one move is the worst way to build Heat |
| **projectiles and lasers** (no hitstun) | count in full, by the percent they really deal | a laser is 1-3% [K] and stales with repeats, so chip is slow Heat; the foe's own hits cost 2 for 1. The attacker lookup already resolves a projectile to its owner [R]. If the playtest shows a camping pattern, projectiles get a coefficient (question 6) |
| **grab** | nothing: a grab deals no damage | - |
| **pummel** | counts by its percent, like a jab | no special case |
| **throw** | counts by its percent; a thrown foe that damages another foe counts if the engine credits the thrower [U] | no special case |
| **shield damage you deal** | does **not** count | no percent was dealt. Otherwise safe pressure on a shield is free Heat. Breaker (Rare) makes it count at half; that needs the shield seam [U] |
| **a hit on your shield** | costs no Heat | you were not hit. Whether the take-damage hook is silent on a shielded hit is [U]; it adds no percent, so it should be |
| **self-damage** (Pichu's recoil, your own explosive, the Bleed price) | neither feeds nor costs Heat, and charges nothing | the attacker is the victim [R `on_damage`, the `a == v` test]. Whether every recoil reaches the hook that way is [U] |
| **stage hazards on a foe** | no Heat, no charge | the player did not deal it. This changes V2, which credits a lone player with hazard hits [R lib.rs:1424]. It still adds to nobody's tally, so a foe killed only by the stage pays nothing |
| **stage hazards on you** | costs Heat like any damage | one rule for damage taken |
| **Burning ticks** | feed Heat at 0.2; charge nothing | it is damage your build dealt, so it shows on the meter; three stacks are 0.6 Heat a second. Charging from ticks is the one place a loop could form, so it is a keystone's privilege, with the giver exclusion (4.3). Ticks go through `gd_envoy_percent`, not the damage hook [R `envoy.rs:338`], so the sim decides this and nothing is counted twice |
| **the extra a Marked foe takes** | the same as a tick: Heat at 0.2, no charge | one rule for all word damage |
| **team attack on** | a hit on a teammate gives the attacker nothing and costs the victim nothing | a partner cannot be farmed, and a deliberate friendly hit [D Q11] is not punished twice |
| **co-op** | a meter each. Every player with 10% or more on a foe's stock cashes out on its KO | no reason to steal a KO or to hold one back for a partner |
| **Petey (a boss with HP)** | counts by HP removed. No overkill line. Every quarter of its HP is a **break**: a cash-out, then half kept | without breaks a long boss fight is the "full meter forever" case. **Engine piece:** the boss word reports a hit count per player today [R lib.rs:1557], and Heat needs damage per player |
| **Master Hand, Crazy Hand** | the same as Petey | whether their HP and per-player damage can be read is [U] |
| **packs of weak foes** | one swing through three foes is three lots of damage. Each KO pays at x0.5 and halves Heat | Heat settles at about twice the damage one minion takes, so a pack holds Warm to Hot and the meter saws up and down on every KO |
| **a foe above 150%** | damage above the line counts half | farming a foe that should already be dead is slow |
| **a foe KO'd by its own mistake** | pays whoever has 10% or more on it | the tally, not the last hit, decides |
| **Ice Climbers** | the second climber's damage should count for its player, and damage to it should cost that player Heat | [U]: how the hook reports the second climber was not read |
| **foes' drives** | charge by damage too. Foes have no meter: a foe's fires use Cold, an elite's Warm, a boss's Hot | one less thing on screen; a foe meter can come later |

## 10. HUD: what must be readable in a fight

This section lists elements and states only. The look is a separate UI pass ("Party pack" sticker style). The Heat
meter replaces the first draft's two numbers and timer bar.

**The Heat meter (one per player, by that player's percent)**

- how full it is, 0 to the cap
- three marks on it: Warm, Hot, Blazing
- the current state, one of four, readable without reading the fill. The fighter also shows it
- a gain: sized by the amount, so a drill reads as a smooth rise and a smash as one jump
- a loss: the piece that was lost is seen leaving
- a state going up, and a state going down (two different moments)
- full (at the cap; more damage is being discarded)
- a raised cap (a longer meter, same marks)
- frozen (tag cinematic)
- the sim's thresholds are exact; the look may hold a state a few frames at a boundary so it does not flicker

**Cash-out**

- the moment itself, in three sizes (Warm, Hot, Blazing)
- what it paid: coins, pips
- half the meter kept
- the flare reaching other foes
- a boss break: the same moment, and the four quarters marked on the boss's HP

**Drives**

- each orbiter's charge as a continuous fill (charge is percent now, not pips), and "ready"
- a fire, sized by the Heat state
- a chain: the link flash runs drive to drive along the orbit, one link every 6 frames (ruled "awesome" [O])
- a retrigger: the drive goes twice
- `+Heat`, `xGain` live, `xHeat`: which drive just touched the meter
- a scaler's run-long count; a set's persistent look

**Foes**

- the words on each foe, and Burning's stacks
- a foe's own chain, told apart from the player's

**Other**

- a technique that registered (also a training aid: the player sees which L-cancels counted)
- the active: its pips, "ready", and the D-pad right prompt
- coins (changes only at a cash-out and at boundaries)
- co-op: two meters, each clearly its owner's

**No digits in a fight.** The first draft showed two numbers. The meter needs none, so the "no text on a fire, a
pickup or a level-up" ruling [O] now holds everywhere in a fight. Sound should carry the state as well (a tone per
state), so the player need not look.

Budget, on top of core v2 3.3: one chain at a time per fighter, four links, six frames apart; one set look per
fighter; the cash-out is the only effect larger than a fire, at most 30 frames.

## 11. From V2: what is reused and what changes

| V2 piece [R `envoy-drives.md`] | V3 |
|---|---|
| `pool.toml` -> `build.rs` -> `pool_gen.rs`, `POOL_DIGEST` | **Reuse.** New fields: `tier`, `kind`, `coef`, `heat_add`, `gain_mul`, `heat_mul`, `mark` (a technique), `set`, `duo`. `need` becomes a percent x1000 for damage sources |
| the lint | **Reuse and extend.** New: every trigger has a coefficient; every `xG` is Rare or Mythic and every `xH` Mythic; every technique drive has a same-family drive without one; no charge source counts hits, landings or jumps; a fire returns less charge than it cost (4.3); the whole pool in any order ends a chain inside 4 links |
| the charge sources (hit, taken, landing, jump, time, tag_in, aerial_hit, hit_100, ko, shift, fire, passive) | **Change.** `hit` -> `dealt`, `taken` -> damage taken, `aerial_hit` -> `aerial_dealt`; `landing`, `jump`, `hit_100` go. `time`, `tag_in`, `ko` (as `cashout`), `passive` stay. New: `special_dealt`, `offstage_dealt`, `chilled_dealt`, and the technique marks `tech`, `lcancel`, `wavedash`, `ledge`, `shield_drop`, `pshield`. `fire` becomes the chain rule for every drive |
| charges as integer counts | **Change:** damage, in millionths of a percent, with the remainder kept (3.7) |
| the take-damage post hook | **Reuse.** It already carries the damage [R]; V2 ignores the amount and counts the call |
| the lone player credited with hazard hits [R lib.rs:1424] | **Change:** no credit (9.6) |
| the boss word (`gd_envoy_boss`): hits per player | **Change:** damage per player, and the boss's HP as a fraction for breaks. Small engine piece |
| the fire queue, one fire per fighter per 20 frames | **Change:** the chain queue (4.1); the 20 frames now sit between chains |
| the fire log (12 entries) in the view | **Reuse**, with link depth and parent, so the UI can draw chains |
| levels x1.0 / x1.6 / x2.4, no bag, duplicates level | **Reuse** [D] |
| slots 3 then 4, per fighter | **Change:** ordered; a fifth after the second boss |
| the four words and their windows | **Reuse**, plus Shocked. Burning: 1% a second per stack; stacks by damage |
| `gd_envoy_blast` and the "only timed drives launch" test | **Reuse** [D]. Chained fires use the percent path |
| `gd_envoy_attacker`, `gd_envoy_absorb`, `gd_envoy_speed`, `gd_envoy_jumps` | **Reuse** |
| action-state polling for landings and jumps | **Reuse and extend** to the technique table (2.3) |
| keystones: six, three per run, the seven prices | **Reuse** the timing and prices [D]; six rules restated, two new |
| foes carry drives | **Reuse** [D]; charged by damage; no foe meter |
| per-player drops, picks, pool 5 / 4, co-op | **Reuse** [D]; Heat, coins and the active are per player |
| the pick of three's "connects to a word" rule | **Change:** connects to a word, a family toward a set, or a technique used this run |
| the run record v4 | **Change:** v5 adds order, scaler counters, coins, the active and its pips. Heat is not in it: it starts at 0 each fight |
| `sim.py` (seeds x policies) | **Reuse and extend:** a damage model per policy and the measures of 12 |
| the UI's orbiters, halo, link flash, Brawl sounds | **Reuse**; new: the Heat meter, continuous charge, the chain line, set looks |
| the 20 drives | 18 kept (all restated on damage), 2 set aside (Spire, Surge), 22 new |
| "the full build is 0.6 s between fires" tuning | **Change:** the dials are the build spread and the character spread (12) |

New host work, all small: `gd_envoy_events` (the L-cancel bit first, the meteor's angle later), the boss word
carrying damage, the knockback write for Grand Finale, `gd_envoy_shape` later (element, shield damage, hitlag,
size), a stage ledge table in the sim, raising `state_bytes`.

## 12. The V3 milestone: a slice to judge depth

Core v2's milestones V3-V11 each move down one; this slice takes the number V3. Off `port/v2`.

**In the slice.**

| piece | scope |
|---|---|
| Heat | section 3 in full: gain by damage, the overkill line, 2-for-1 loss, the four states and their scale, the cash-out (coins, pips, flare), half kept, the caps, boss breaks |
| damage charge | every trigger's need in percent, remainders kept; the hit counter removed |
| chains | 4.1 in full: coefficients, rightward only, 4 links, 6 frames |
| order | ordered slots; reorder on the reward screen; Echo as the one neighbour drive |
| technique | L-cancel (the one event bit), wavedash, tech, offstage damage (polled; a ledge table for the 5 Short-run stages) |
| drives (16) | Ember, Bolt, Nova, Wildfire; Rush, Tailwind, Clean Landing, Perfect Form; Riposte, Insulate; Feather Storm, Gatekeeper; Frost, Shatter; Echo, Metronome |
| keystones (3) | Plague Bearer, Glass Core, Grand Finale. If the knockback write slips, All In takes Grand Finale's place: it needs no engine work |
| the active (1) | Volley, on D-pad right, given at the run's start |
| a set (1) | Inferno, so the three-of-a-family moment can be felt |
| coins | earned as in 7.3; spent on one plain screen after fight 3: three drives, a level, a reroll |
| tiers | the three tiers as data and as a beam colour; no editions |
| HUD | the Heat meter with its three marks and four states, gain and loss, the cash-out moment, continuous charge on the orbiters, the chain line, the technique spark, the active's pips, stand-in sounds |
| shapers | words and bonus damage only: element, shield damage, hitlag and size wait |
| engine pieces | the L-cancel bit; the boss word carrying damage (if the Short run's boss is in); the knockback write |
| checks | the lint; `sim.py` with the measures below; synctest depth 4 and 7 over 3600 frames, 0 mismatches, across a chain, a cash-out, a state change, a technique mark, the active and a shop buy |

The slice's drive list changed in two places from the first draft: **Insulate** in (a build about keeping Heat
needs one drive), **Crescendo** out (the slice has no other Mythic, and `xHeat` can be judged after `+Heat` and
`xGain` have been).

**Out of the slice:** co-op (V2's works and is re-tested after; the per-player meter and the tally are built
co-op-ready), the rest stage, the pact, editions, duo drives, `gd_envoy_shape`, four of the five actives, five of
the six sets, `xHeat` drives, cap-raisers, a foe meter.

**The measures before the playtest** (`sim.py`):

1. **Build spread.** The percent a build deals (fires, word damage, flares) per 100% of move damage: the best
   five-drive build over a random five-drive build, per policy. Band **2.5 to 6**. Under 2.5 the builds are still
   shallow; over 6 one build has solved the pool. V2's pool would score near 1 [I].
2. **Heat spread.** Median Heat at a KO, best build over random build. Band **1.5 to 2.5** (the cap bounds it).
3. **Character spread** (new: the owner's first objection as a number). The same build, the same total damage, four
   damage profiles: a few big hits, many small hits, a multi-hit move, projectile chip. The highest build output
   over the lowest: at most **1.25**.
4. **Split test** (new: the owner's second objection as a test). Replace any one piece of damage by n pieces with
   the same total. Heat, every drive's charge, coins and pips must be **identical**. This runs in the lint's test
   suite, not only in `sim.py`.
5. **Use.** The share of the 16 drives that appear in at least one of the top ten builds: at least three quarters.
6. **KO delay.** How much a policy that delays KOs to cash out higher gains over one that does not: under 15% of
   coins.

**The playtest.** Two Short runs. The reward screen is nudged toward Firestorm in one and Tech Monk in the other,
on **two different characters, one of them with no real combo game**. The questions for the owner: "did those two
runs play differently, did the meter build on both characters, and did you want a third?".

**Size and models.** M (one to two weeks). Sonnet: pool data, lint, the sim's Heat and chain state, `sim.py`,
synctests. Opus: the Heat meter and its feel, the chain line, the shop screen.

**Risks.**

| risk | answer |
|---|---|
| the meter is ignored: Heat feels like a background number | the state changes the size and sound of every fire, and the fighter shows it; the cash-out is the loudest moment in a fight |
| 2-for-1 feels punishing against hard foes | one number; Insulate and Bulwark exist; tune in the playtest (question 1) |
| a strong player sits at Blazing | KOs and breaks halve it; overflow is discarded; the cooling rule is the reserve (3.2) |
| delaying KOs to cash out higher | the cap, the overkill line and measure 6 |
| laser camping | stale moves and 2-for-1; a projectile coefficient is the reserve (question 6) |
| Frozen Clockwork looks too strong by hand | first target for `sim.py`; Shatter's need and Echo's second fire are the dials |
| technique drives lock out players who do not L-cancel | every technique drive has a same-family drive without one (lint); a pick shows at most one technique card unless the player has used that technique this run |
| chains bring the October noise back | four links, one chain at a time, rising pitch so a chain reads as one event |
| the L-cancel bit slips | Clean Landing and Perfect Form take wavedash conditions for the slice |
| CPU-proxy tuning misleads | the spreads are properties of the pool, not of the pace; the owner's play sets the dials |

## 13. Questions for the owner (one at a time; each with my recommendation)

The Heat decision answered the first draft's question 1 (the combo as the centre) and question 5 (knockback: your
moves stay retail; only Grand Finale changes it). The first draft's other questions (slot order, 3/4/5 slots, D-pad
right, how much technique matters, coins and the pact, tiers now and editions later) were not part of the decision.
I have kept working on my recommendation for each (yes to all) and will not ask them again unless you want to
reopen one.

1. **How much Heat should a hit you take cost?** *Recommendation: 2 Heat for every 1% you take.* It counts damage,
   not hits, so a foe's drill costs what one hit of the same percent costs. A normal hit takes about an eighth of a
   full meter; the worst single hit about 40%. The alternatives: 1 for 1 (gentler, but a hit barely shows), or a
   flat chunk per hit (simple, but it counts hits).
2. **What should a KO do to your Heat?** *Recommendation: pay on all of it, then keep half.* Keeping none makes
   packs of weak foes feel cold; keeping all lets a good player sit at the top.
3. **Should Heat ever fall by itself over time?** *Recommendation: no.* It moves only when you deal damage, take
   damage or someone is KO'd. A slow decay would bring the combo timer back in another form. The reserve, if you see
   camping at full Heat: lose Heat after 10 seconds with no damage dealt.
4. **Does Heat carry from one fight to the next?** *Recommendation: no, every fight starts at 0.* Then how fast a
   build heats up is part of the build. The alternative is to carry half.
5. **In co-op, who cashes out on a KO?** *Recommendation: every player who dealt 10% or more to that foe on that
   stock, each on their own meter.* Nobody steals a KO. The alternative is only the player who landed it.
6. **Do lasers and other no-flinch projectiles fill Heat in full?** *Recommendation: yes, by the percent they
   really deal.* They are small and they stale. The alternative is to count projectiles at half from the start.
7. **Timed and technique drives: fine as they are?** Your rule is that all charge is by damage. I read it as "never
   by hit count", and kept three things that are not damage: a clock (Nova, Bulwark), a tag (Slipstream, Relay) and
   technique, which only pays through damage dealt after it. Landings and jumps were converted to aerial damage.
   *Recommendation: keep it this way.* The alternative is the strict reading: every drive charges by damage and
   nothing else.

## 14. Sources

### Web (read 2026-11-10, for the first version of this note; not re-read for the revision)

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

Not reached, so the claims that rest on them are tagged [K]: a Risk of Rain 2 page on the proc chain rule that
stops an item triggering twice in one chain; the Hades duo boon pages; the Slay the Spire relic pages. The Melee
facts tagged [K] in this revision (stale-move decay, laser and smash percents, the ice element freezing) were not
looked up either.

### Repo (this branch unless named)

- `docs/rust-port/notes/envoy-drives.md`; `envoy-core-v2.md` on `pub/port/envoycore2`.
- `rust/mods/envoy_data/pool.toml`, `src/codes.rs` (the Burning constants, `PRICE_TEXT`, `FIRE_GAP`);
  `rust/mods/envoy_sim/src/lib.rs` (imports 39-81, action-state ranges 117-123, `x100` 937, the damage hooks
  1378-1481 with the hazard credit at 1424, the boss hit count 1556-1571, `foe_ko` 1589, the tag 1779-1801, the hook
  table 2137-2149).
- `rust/melee_modes/src/tu/envoy.rs` (`percent` 338, `absorb` and the knockback field 422, `attacker` 427-441,
  `blast`, the snapshot note in the header); `rust/melee_modhost/src/lib.rs:536` (`gd_fighter_read`'s fixed record).
- `rust/melee_fighter/src/tu/ftCo_LandingAir.rs:65` (the L-cancel test), `ftCo_Landing.rs:140`
  (LandingFallSpecial), `ftCo_AppealS.rs:76` (the taunt on D-pad up), `ftcoll.rs` (510, 840: the element at a
  connect), `fighter.rs:2912` (hitlag), `ft_0763.rs:26` (the attack id).
- `rust/melee_types/src/gen.rs` (motion-state ids, `HitCapsule`); `docs/specs/hooks-v1.tsv` (the action hook rows).
- Memory: `envoy-lab-answers-2026-11-09.md`.
