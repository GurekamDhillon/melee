# Envoy synergy architecture, V3: Heat, and drives that feed each other (design, finalised 2026-11-10)

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
> 7-hit drill totalling 14% is worth what one 14% hit is worth. It carries through neutral, drops by what you take
> when you are hit, and cashes out on a KO. Drives add to it or multiply it. **All drive charge is by damage, not by hit
> count**, everywhere.
>
> **Kept:** proc chains, slot order, sets, technique, the active drive on D-pad right, coins and the shop, rarity
> tiers, scaling drives, keystones as devil deals. Each was re-derived on damage; the numbers are new.

> **Finalised on 2026-11-10 (the owner's answers; the rest decided under delegation).**
>
> The owner answered the five Heat questions and handed the remaining decisions to the author, to be made in the
> owner's pattern. All of them are in the table of 13.1. What moved because of them:
>
> 1. **Taking a hit costs 1 Heat per 1% taken, not 2** [O: "trades build slowly"]. Heat is now, plainly, what you
>    dealt minus what you took.
> 2. Because the same play now nets half again as much Heat, **the Heat scale is multiplied by 1.5**: the marks are
>    at 40 / 90 / 165 and the cap at 225 (they were 25 / 60 / 110 and 150). The marks sit at the same places along
>    the bar, so the HUD's three marks and a cap do not move (3.4). Pays per Heat are divided by the same 1.5.
> 3. **Insulate** has a new effect (its old one became the default rule), **All In** costs 2 per 1% (was 3), Glass
>    Core was checked and kept, and every pacing table and build sum was redone by hand.
> 4. The top tier, Mythic in the first revision, is renamed **Epic**. Coins go into **one party purse**. Editions
>    and duo drives are parked.
> 5. The question list is gone: section 13 is the decisions and the open risks. Section 12 is now a build spec.

Tags: **[R]** read from the repo (cited), **[W]** read on the web (section 14), **[K]** the author's own knowledge
of the game, not fetched, **[I]** inferred, **[J]** judgement, **[U]** unverified in code, **[D]** an owner decision
already taken (memory `envoy-lab-answers-2026-11-09.md`, the Heat decision above, or 13.1), **[O]** the owner's
own words.
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
- **Heat is the shared number.** One meter per player. 1% dealt is 1 Heat. 1% taken costs 1 Heat. No decay.
- **Heat has four states**: Cold, Warm (40), Hot (90), Blazing (165), full at 225. The state scales every drive
  fire: x1, x1.25, x1.5, x2.
- **A KO cashes out.** It pays coins, charge for the active drive and a flare at the other foes, all sized by Heat.
  Then you keep half.
- **Drives charge by damage.** "Every 40% dealt", not "every 5 hits". A drill and a single hit of the same total
  charge the same.
- **Fires are damage.** A drive's fire charges the drives **to its right** by its damage times its **coefficient**,
  and feeds Heat the same way. That is a proc chain; position, the coefficient and a four-link limit are the brakes.
- **Slot order matters.** Heat steps apply left to right, chains run left to right, and two drives act on a
  neighbour.
- **Technique is a bonus path, never a requirement.** L-cancel, wavedash, tech, ledge grab, shield drop, power
  shield and edgeguard give Heat or speed up charge, but only through damage that follows them or a hit they
  answered, so none can be farmed in a corner.
- **Three of a family on one fighter is a set**: a visible transformation with its own rule.
- **One active drive per player on D-pad right**, charged by cash-outs, kept through a tag.
- **Scaling drives** grow for the whole run. **Keystones are the devil deals**: eight, each bends a Heat rule.
- **V3 is a slice**: Heat, damage charge, chains, order, four techniques, 16 drives, 3 keystones, 1 active, 1 set,
  a plain shop screen, in the 5-fight Short run (section 12 is its build spec). Its question is "do two runs feel
  different?", and now also "does every character build Heat?".

Kept as decided [D]: drives per fighter; floor drops plus a pick of three; a duplicate levels a drive (to 3); no bag;
keystones at the start and after the first two bosses; fires are loud; only timed drives launch; elites and bosses
carry drives; co-op drops per player; content-only unlocks; NG+; the Brawl disc required; the drive models. Decided
on 2026-11-10: the table of 13.1.

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
| editions | a modifier on any joker [W10] | editions, parked until the slice is judged (7.4) |
| slot limit | five jokers, so every pick is also a cut [W10] | 5 slots per fighter (5.3) |
| economy | money per round, interest, a shop between rounds [W12] | coins and the shop (7.3) |

### 1.4 Two others

- **Hades.** A duo boon appears only when the player already holds a qualifying boon from each of two gods [K].
  Envoy takes two **duo drives** (7.5), parked until the slice is judged.
- **Slay the Spire.** Balance was driven by pick-rate and win-rate metrics from play [W15]. Envoy takes: the elite
  drops what it carried (7.5), and `sim.py` measures build spread and character spread before a playtest (12.10).

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

**Technique is a bonus path, never a requirement** [D 13.1]. Every technique drive has a drive of the same family
with no technique condition, and no build in section 9 needs one to work.

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
| **overkill** | the part of the damage that lands above 150% on a foe counts half | the line is placed on the sim's own integer sum of what that foe took this stock (12.2), not on the engine's float; the damage is split at 150, so it is still exact for multi-hits. Bosses with HP have no such line |
| **decay** | none [D, owner] | argued in 3.2 |
| **loss when hit** | 1 Heat per 1% you take [D, owner] | any source except yourself and a teammate (9.6); never below 0 |
| **you lose a stock** | Heat halves, no pay | "any KO halves it; yours pays" |
| **a KO (cash-out)** | pays on all the Heat you hold (3.5), then you keep half [D, owner] | every player who dealt 10% or more to that foe on that stock cashes out [D, owner] |
| **a tag** | nothing changes | Heat belongs to the player [D] |
| **a fight starts** | Heat is 0 [D, owner] | each fight has its own arc; scaling drives can raise the start (7.1) |
| **cap** | 225 | drives and keystones can raise it; never above 375 |
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

What stops "sit on full Heat forever" is not time but events: a KO halves it (which is also the pay-off) and a
boss "break" halves it (9.6). Being hit takes it back at the rate it came, which at 1 for 1 is a weaker brake than
the first revision had. If the playtest shows a player parked at Blazing while timed drives do the work, the fix is
a cooling rule (after 10 seconds with no damage dealt, lose 8 Heat a second). It is not in the slice, and it is an
open risk (13.2).

**Loss scaled by damage taken, 1 for 1** [D, owner, 2026-11-10]. The owner's reason: "trades build slowly" [O].
The candidates were:

| loss rule | what happens |
|---|---|
| a flat chunk per hit (say 25) | it counts hits. A foe's drill or rapid jab would strip the meter; a 1% laser would cost as much as a smash. This is the second objection, turned on the player |
| a fraction per hit (say a quarter) | the same: seven small hits leave 13% of the meter, one big hit leaves 75% |
| 2 Heat per 1% taken (the first revision's recommendation) | nothing counts hits, but an even trade loses Heat, so a player who trades never leaves Cold. Rejected by the owner as too harsh |
| **1 Heat per 1% taken** | nothing counts hits. Heat is what you dealt minus what you took, then the cap and the KO's halving. A jab (3%) costs 3. A typical hit (10%) costs 10. The hardest single hits in the game (20-30% [K]) cost 20-30, at most an eighth of a full meter |

So trading two for one (deal 30, take 15) gains 15 each time, an even trade holds still, and only a losing exchange
falls. Every character can trade.

What this gives up, said plainly: a single hit is a small step on the meter (a 10% hit is about 4% of a full one),
so the meter is no longer where a lost exchange is felt. Percent is. The loss is still drawn (10), and an exchange
lost badly (60%) still takes more than a quarter of a full meter.

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

When a player **takes** damage from a foe or the stage: Insulate's allowance, if the player wears it, absorbs the
damage first (the first 20% taken in a fight, and again after each cash-out, costs nothing); Heat falls by what is
left, 1 for 1; drives charged by damage taken gain all of it; "without taking damage" streaks reset.

Three kinds of Heat drive, then:

| kind | what it does | where it lives |
|---|---|---|
| **+Heat** | a flat amount on an event (an L-cancelled aerial that dealt damage: +6) | Common |
| **xGain** | multiplies how fast damage becomes Heat while a condition holds (in Haste: x1.5) | Rare and up |
| **xHeat** | multiplies the Heat you hold, once, on a hard event (a meteor smash: x1.5) | Epic |

### 3.4 What Heat does while you hold it

| state | Heat | drive fires deal | what else |
|---|---|---|---|
| **Cold** | 0-39 | x1.0 | - |
| **Warm** | 40-89 | x1.25 | - |
| **Hot** | 90-164 | x1.5 | drives and sets can read "Hot or hotter" |
| **Blazing** | 165 and up | x2.0 | the top state; drives can read it |

**Why the marks moved** (delegated; the first revision had 25 / 60 / 110 and a cap of 150). At 2 for 1 the
reference stock below ended at 60 Heat. At 1 for 1 it ends at 90: exactly 1.5 times as much. Left alone, the old
marks would put a player with no drives at Blazing on the second KO and at the cap on the third. Two things would
break. The states would stop telling builds apart, because everybody would be at the top. And the Heat spread
measure (12.10, band 1.5 to 2.5) could not be met: the most any build could beat a driveless player by would be
150 / 90 = 1.67 on the first KO and nothing after.

So every number on the Heat scale is multiplied by 1.5 and rounded to 5: marks at **40 / 90 / 165**, cap **225**.
The marks sit at the same places along the bar (0.18 / 0.40 / 0.73 of the cap, against 0.17 / 0.40 / 0.73), so the
HUD still draws three marks and a cap, where it drew them. What a full meter pays is unchanged, because the pays
per Heat are divided by the same 1.5 (3.5). The cost of this choice is in 13.2: a hit is a smaller share of a
longer bar.

- The **state scales every drive fire**. That is where the build-up shows: the same Ember is a puff at Cold and a
  blast at Blazing. It also makes chains reach further when you are hot (4.2).
- The player's **own moves are unchanged**: retail percent and knockback. A move that launched differently
  depending on a meter could not be followed up. One keystone breaks this on purpose (8.3).
- States are thresholds, not a curve, so there are four things to read, and a drive rule can say "while Hot" in
  two words.

Hand-worked pacing with **no drives at all** [J]. The "reference stock": the foe is KO'd at 120% total; the player
deals it in four lots of 30% and takes damage between lots.

| play | Heat after each step | first KO | second KO | third KO |
|---|---|---|---|---|
| **reference** (deal 30, take 10) | 30, 20, 50, 40, 70, 60, **90** | pays on 90 (Hot, only just), keeps 45 | reaches **135**, keeps 67 | reaches **157**, keeps 79 |
| **trading two for one** (deal 30, take 15) | 30, 15, 45, 30, 60, 45, **75** | pays on 75 (Warm), keeps 37 | reaches **112** (Hot), keeps 56 | reaches **131** |
| **trading evenly** (deal 30, take 30) | 30, 0, 30, 0, 30, 0, **30** | pays on 30 (Cold), keeps 15 | reaches 30 | reaches 30 |

So ordinary good play reaches Hot at the first KO and stays just under Blazing through the third (the fourth
reaches 169). That is where the first revision put it. A player who trades two for one builds, slowly, as the
owner asked. An even trade goes nowhere. Blazing early in a fight has to come from the build. That is the room
drives have.

### 3.5 The cash-out

When a foe loses a stock, every player whose tally on that foe (this stock) is 10% or more cashes out. Who landed
the last hit does not matter, so a foe that falls to its own mistake after you damaged it still pays, and co-op
partners never fight over a KO.

| pays | amount | cap |
|---|---|---|
| **coins** | Heat / 15 | none (the meter is capped) |
| **the active's charge** | 1 pip per 150 Heat cashed, the remainder carried | - |
| **the flare** | Heat / 15 percent to every **other** foe in play, no launch. It feeds no Heat, charges nothing and joins no tally | 15% each |
| **drives that read a cash-out** | Wildfire, Leech, Snowball, Magpie (8.1) | each its own |

Then the player keeps half. The pay is multiplied by what the foe was: a pack minion x0.5, a fighter x1, an elite
x1.5, a boss break x1, the boss's last break x2 [J]. The divisors are the first revision's (10, 100, 10) times 1.5,
so a full meter pays what it did: 15 coins, a pip and a half, a 15% flare. Coins go to the party's one purse; the
sim also keeps what each player earned (7.3).

There is a small reason to delay a KO: to cash out at more Heat. It is bounded three ways: the cap (at most 15
coins from a fighter), the overkill rule (a foe above 150% gives half), and what a hit takes back if the plan goes
wrong. `sim.py` reports it (12.10).

### 3.6 Caps

| thing | cap | why |
|---|---|---|
| Heat | 225; 375 with every cap-raiser | the meter has an end the player can see |
| the gain multiplier | x4 | a full meter in about 56% dealt at most |
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
- **Heat** is an `i64` in ticks, 2,000,000,000 to a point (12.2 has the exact table). Gain is a product of
  integers (the damage, the overkill half, the coefficient, the gain multiplier) **with no division**, clamped to
  the cap. Because nothing is divided or rounded per record, seven records of 2% and one of 14% add exactly the
  same Heat. The only grain is the rounding at the hook: at most 0.0005% per hit.
- **Charge** is an `i64` too, 1,000,000,000 units to a percent. When a drive fires, its need is subtracted and
  **the remainder stays**. Without that, one big hit would waste what a multi-hit move keeps.
- The **gain multiplier** is a product of x1000 values, rounded down after each step, left to right, then capped.
- **Coins and pips** carry their remainders the same way, so no cash-out loses a fraction.
- The overkill split, the tally per foe and player (4 x 2 `i32`), the Burning damage counter per foe, technique
  marks, streak counters, the chain queue, the active's pips and coins are all plain integers in the same struct.
- A coefficient scales **charge**, not probability, so there is no dice roll to desync and the player can plan a
  chain.

The manifest's `state_bytes` is 4096 today [R lib.rs:2149] and will need raising (12.3).

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

1. **Heat steps apply left to right** (3.3). From 60 Heat, `+6` then `x1.5` gives 99; the other order gives 96.
   `+Heat` goes left of `xHeat`.
2. **Chains run left to right** (4.1). Starters go left, finishers right.
3. **Neighbour drives** act on a position: **Echo** makes the drive to its right fire twice; **Mirror** copies the
   drive to its left.
4. **The leftmost element shaper** sets the element shown (2.2).
5. With **Twin Soul** the benched fighter's row continues the active one: slots 6-10 of one line.

### 5.1 Reordering

Slot order matters [D 13.1], so moving a drive is a real choice and is rationed:

- **At the rest stop: free**, any drive, either fighter, as often as the player likes.
- **On the reward screen: only while placing a new drive.** The player puts the new drive in any slot of that
  fighter's row and may rearrange that row as part of placing it. Skipping the pick moves nothing.
- Nowhere else, and never during a fight.

Order is drawn in the orbit: the first slot leads, and a chain's link flash runs along the orbit in order.

### 5.2 What the pick tells the player

A card shows what the drive would do **in the slot it would take**, with the neighbours it would act on lit. The
"compare rows only" rule [D E22] stays: no strength number.

### 5.3 Slots

**3, then 4 after the first boss, then 5 after the second** [D 13.1], per fighter. V2 has 3 then 4 [R]. Five
matches the limit that makes every pick a cut in Balatro [W10]. The orbit budget was 4 [R core v2 3.3]; shapers are
drawn smaller and closer to the body than triggers, so the loud objects stay near four.

### 5.4 Full rows and duplicates

- A **duplicate** of a drive the fighter wears levels it up (to 3) and takes no slot [D].
- A **new drive onto a full row** replaces one drive the player chooses, or the pick is skipped [D 13.1]. The
  replaced drive is gone: there is no bag [D]. The new drive takes the old one's slot unless the player rearranges
  the row while placing it (5.1).

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
- **Charge:** pips. One pip is 150 Heat cashed out (3.5). An active needs 2-4 pips. Pips are kept between fights.
- **Use:** on the ground or in the air when actionable, as the tag is. It is the player's own timing, so by the
  standing rule an active may launch [D].
- Got from the first boss, or as a Rare card; in the slice, given at the run's start. Not sold in the shop. Taking
  a new one replaces the old.

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
| **Snowball** | you cash out while Blazing | +3 Heat at the start of every fight, per cash-out, to +60 |
| **Magpie** | (not a scaler; it feeds them) | cash-outs pay half again in coins; Heat over the cap becomes coins |

Scalers are weak when found and strong when found early, so the same card is a different decision in act 1 and
act 3.

### 7.2 Keystones as devil deals

Timing as decided: one at the run's start, one after each of the first two bosses, so up to three in force
[D 13.1]. Each is one rule and one price. Every keystone now bends one rule of section 3 or 4, so a keystone picks
the kind of run. The eight are in 8.3.

### 7.3 Coins, the shop and the pact

| coins from | amount |
|---|---|
| a cash-out | Heat / 15, times what the foe was (3.5) |
| a fight cleared | 20; +10 with no stock lost |
| an edgeguard KO | +10 |
| a taunt inside 2 seconds of a KO | +5 (the retail taunt; you stand still for it) |
| interest at each fight's start | +1 per 10 held, to +5 |

Hand-worked: a fight with three fighter KOs at 90, 135 and 157 Heat pays 25 coins from cash-outs and 20-30 for the
clear.

**One purse** [D 13.1]. Coins are **earned per player** (each player's own cash-outs and bonuses) and go into
**one shared party purse**, as the stock pool is shared [D]. The sim keeps each player's earned total for the end
screen. Either player may spend at the shop; what is bought goes to the buyer's fighter.

**The rest stop sits just before each act's boss** [D 13.1], and it is the shop. Until the rest stage exists as a
place (core v2's milestone V8) it is a screen at the same point in the run.

| on sale | price |
|---|---|
| **buy a drive**: three on offer, from the shop pool | Common 30, Rare 80, Epic 150 |
| **level a drive** you wear | 60 to level 2, 120 to level 3 |
| **restore a stock** to the pool | 100 |
| **reroll** the three | 10, then +10 each time |
| **reorder** | free (5.1) |

A bought drive onto a full row replaces one the player chooses (5.4). Actives are not sold. A Common was 40 in the
first revision; it is 30 so that the first rest stop of a run, which holds about 30 coins by hand (12.9), can buy
one [J].

**The pact**: once per act the rest stop offers one Epic drive for **one stock from the pool**. In this game the
shared stock pool [D] is the health a devil deal would take. A stock costs 100 to restore and an Epic 150, so the
pact is a good price for a player who can spare the stock.

### 7.4 Rarity tiers and editions

Tiers ration the multipliers.

| tier | share of the pool | holds | mark |
|---|---|---|---|
| **Common** | about half | triggers, shapers, `+Heat`, technique | the plain glass shell |
| **Rare** | about a third | `xGain` on a condition, retriggers, scalers, set-makers | a coloured beam on the floor drop (the old game had a beam per rarity [R core v2 1.1]) |
| **Epic** | the rest | `xHeat`, copy, the duo drives | a beam and a ring |

Levels (a duplicate, to 3) stay as they are and are separate from tier [D].

**Editions** (parked: they and the duo drives stay out until the slice is judged [D 13.1]): a drop can carry one
modifier, whatever the drive: Tempered (its fire counts one Heat state higher), Radiant (+8 Heat when it fires),
Prismatic (its coefficient is 1.0), Hollow (takes no slot).

### 7.5 Pools by encounter kind

| source | pool | rule |
|---|---|---|
| duel drop | Common 80%, Rare 20% | any family |
| pack drop | Common, weighted to PURPLE and RED | many foes: spreading and chain drives |
| shift room drop | Common and Rare, weighted to GREEN and YELLOW | movement and technique |
| elite | Rare | one of its two drops is **a drive the elite carried** [D]: you take what was used on you |
| boss | Epic, a pick of three; then the keystone pick [D] | the first boss also offers the first active |
| pick of three (every boundary) | one card from the encounter's pool, one that connects to what the fighter wears, one free | "connects" means: a word it gives or reads, its family (toward a set), or a technique the player has used this run |
| shop | all tiers at their prices | the only place to buy a level or a stock |
| pact | Epic | one stock |
| duo drives (parked) | Epic | offered only when the fighter wears a drive from each of the two families |

A drive declined in a pick is half as likely for the rest of the run [W9]. A level-3 drive is never offered again
[D]. Unlocks stay content-only [D]. The "draw" here is the run's existing seeded pick [R], decided at fight
boundaries, not in a fight.

## 8. The pool: 40 drives, 5 actives, 8 keystones

Kinds: **T** trigger (charges and fires; its coefficient in brackets), **S** shaper, **+H** adds Heat, **xG**
multiplies Heat gain, **xH** multiplies Heat held, **G** grows over the run, **Ec** economy, **N** neighbour.
Tiers: C (Common), R (Rare), E (Epic). Numbers are level 1 and first guesses. "V2" marks a drive kept from V2,
restated on damage.

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
| 11 | GREEN | **Clean Landing** | C | +H | An L-cancelled aerial that dealt damage: +6 Heat. | technique as Heat |
| 12 | GREEN | **Perfect Form** | R | xG | While your last three aerial landings were L-cancelled: Heat fills x1.5. | goes with Clean Landing |
| 13 | BLUE | **Bulwark** V2 | C | T (0) | Every 15 seconds: an orb. Once struck it blocks everything for half a second. | protects Heat. The half second makes it proof against a foe's multi-hit |
| 14 | BLUE | **Thorns** V2 | C | T (0.4) | Every 20% you take: the attacker takes 4% and is Marked. | a fire that starts a chain while you are losing |
| 15 | BLUE | **Leech** V2 | C | S | A cash-out heals 1% for each 20 Heat, to 10%. | was "every 5th hit" |
| 16 | BLUE | **Riposte** | C | +H | A tech or a power shield: +10 Heat. | defence into Heat |
| 17 | BLUE | **Breaker** | R | S | Shield damage you deal counts as damage, at half. A shield break: +45 Heat. | shield pressure build; needs the shield seam [U] |
| 18 | BLUE | **Insulate** | R | S | The first 20% you take in a fight, and after each cash-out, costs no Heat. | the Heat keeper: an allowance in percent, refilled by a KO |
| 19 | YELLOW | **Comet** V2 | C | T (1.0) | Every 40% of aerial damage: your next landing sends a shockwave, 4%. | player-timed, pops up; full strength |
| 20 | YELLOW | **Updraft** V2 | C | T (0) | Every 20% of aerial damage: an air jump back. | longer air strings |
| 21 | YELLOW | **Feather Storm** V2 | C | T (0.5) | Every 30% of aerial damage: three feathers, 2% each, at three different foes if there are three. | spreads words and damage across a pack |
| 22 | YELLOW | **Skyhook** | R | xG | Aerial damage fills Heat x1.5. | Heat for air play |
| 23 | YELLOW | **Gatekeeper** | R | xG | Damage to an offstage foe fills Heat x2. | edgeguard |
| 24 | YELLOW | **Meteor** | E | xH | A meteor smash: Heat x1.5. | the hardest condition, the biggest single step |
| 25 | PURPLE | **Frost** V2 | C | T (0.3) | Every 40% dealt: 2% and Chilled for 4 seconds. | gives Chilled |
| 26 | PURPLE | **Shatter** V2 | C | T (0.4) | Every 20% dealt to Chilled foes: ice shards, 5%, ending the Chill. | reads Chilled |
| 27 | PURPLE | **Hex** V2 | C | T (0.4) | Every 50% dealt: 2% and Marked for 5 seconds. | gives Marked |
| 28 | PURPLE | **Static** | C | T (0.3) | Every 15% dealt with specials: 2% and Shocked. | gives Shocked, the retrigger word |
| 29 | PURPLE | **Affliction** | R | xG | Heat fills a quarter faster for each word on the foe you damage, to x2. | reads every word |
| 30 | PURPLE | **Contagion** | R | S | A word you apply also lands on foes nearby. | packs; with Blight |
| 31 | WHITE | **Relay** V2 | R | T (1.0) | Tag in: your drives fire in order, as one chain; each keeps its own coefficient. | the tag as a chain starter |
| 32 | WHITE | **Echo** V2 | R | N | The drive to the right fires twice; the second time at half strength. | was "every 3rd fire repeats" |
| 33 | WHITE | **Conductor** | R | S | Chained fires keep full strength for one more link. | lifts every coefficient |
| 34 | WHITE | **Metronome** | C | +H | Damage from a move that is not one of your last three: +5 Heat. | variety; no technique needed |
| 35 | WHITE | **Crescendo** | E | xH | Each 50% you deal without taking damage: Heat x1.25. | the multiplier for clean play |
| 36 | WHITE | **Snowball** | R | G | A cash-out while Blazing: +3 starting Heat, this run. | scaler |
| 37 | WHITE | **Magpie** | C | Ec | Cash-outs pay half again in coins. Heat over the cap becomes coins, 1 per 15. | economy |
| 38 | WHITE | **Mirror** | E | N | Copies the drive to its left. | a second Crescendo, or a second Bolt |
| 39 | RED + PURPLE | **Steam** (parked) | E duo | xG | Damage to a foe both Burning and Chilled fills Heat x2.5. | needs two givers |
| 40 | YELLOW + RED | **Thunderhead** (parked) | E duo | T (0.4) | Every 20% of aerial damage to an offstage foe: a lightning bolt, 5%. | edgeguard into chains |

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
- **Insulate's effect is new** (2026-11-10). It was "damage you take costs 1 Heat per 1%, not 2", which is now
  everybody's rule. The replacement keeps the job and counts nothing but damage: an allowance of 20% taken (32% and
  48% at levels 2 and 3) that costs no Heat, refilled at a fight's start and by each cash-out. A foe's drill and
  one hit of the same percent use up the same amount of it. Halving the cost again (0.5 per 1%) was the other
  candidate; at 1 for 1 a hit is already small, so half of it would be hard to see, and an allowance can be drawn.
- **Flat Heat amounts** (Clean Landing, Riposte, Metronome, Breaker, Snowball) are 1.5 times the first revision's,
  with the scale (3.4). Each pays **once per event** (a landing, a tech, a move), never per record, so a multi-hit
  move is one event.
- **Echo's** second fire is at half strength, V2's number [R `pool.toml`], and scales with level (x0.5 / x0.8 /
  x1.2). The first revision left the strength unsaid and its hand sums took it as full.
- Not carried from V2: **Spire** and **Surge**, as before.

Checks the lint keeps or gains (11): every word read has two givers (Burning: Ember, Kindle, Plague Bearer; Chilled:
Frost, Winter Oath; Marked: Thorns, Hex; Haste: Rush, Slipstream; Shocked: Static, and Bolt at level 3). Every `xG`
drive is Rare or Epic; every `xH` is Epic. Every technique drive has a drive of the same family with no
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
| 6 | **Glass Core** V2 | Heat fills twice as fast. You take 60% more. | own | checked against the 1-for-1 loss and left as it is. Hand-worked on the reference stock: 240 gained, 48 lost, **192 at the first KO** (Blazing; 0.85 of the cap). Under the old rule it was 144 of 150 (0.96). Its price on the meter halved with everyone's, so the price is now mostly the percent: 60% more taken is a stock lost sooner, and a lost stock halves Heat |
| 7 | **Grand Finale** | Your smash attacks launch harder with Heat: +1% knockback per 4.5 Heat, to +50% at a full meter. A KO from a smash attack pays half again and keeps no Heat. | Weak | the only knockback scaling. Build Heat, then spend all of it on the kill |
| 8 | **All In** | A KO keeps all your Heat. Damage you take costs 2 Heat per 1%, and losing a stock takes all of it. | own | the one way to stay at the top. Twice the base rate, where it was one and a half times: at 3 per 1% the reference stock nets 30 a stock (30, 60, 90 at three KOs), under what no keystone gives (90, 135, 157) for the whole of a three-stock fight, so nobody would take it. At 2 it nets 60 (60, 120, 180): behind on the first KO, ahead on the third, and well ahead for a player who takes less than the reference |

## 9. Five builds, and how each snowballs

All numbers here are **hand-worked** on the reference stock of 3.4 (the foe is KO'd at 120% total; the player takes
10% for every 30% of move damage dealt before the KO), with level-1 values and five slots. They were redone on
2026-11-10 for the 1-for-1 loss and the 40 / 90 / 165 / 225 scale. They are rough and exist to show the shape of
each build. Where a build's fires do part of the 120%, the moves needed drop, and so does the damage taken. A fire
is counted at the Heat state the build holds when it goes off. With no drives: 120% of moves, 30% taken, **90
Heat** at the KO.

| build | moves needed | taken | Heat at the first KO | of the cap (was) | state | first cash-out |
|---|---|---|---|---|---|---|
| no drives | 120% | 30% | 90 | 0.40 (0.40) | Hot, only just | 6 coins |
| Firestorm | about 75% | 25% | about 65 | 0.29 (0.25) | Warm | 4 coins, and Wildfire |
| Tech Monk | 120% | 40% | about 221 | 0.98 (0.93) | Blazing | 22 coins; the KO comes early |
| Frozen Clockwork | about 78% | 20% | about 113 | 0.50 (0.53) | Hot | 15 coins |
| Sky Pirate | about 87% | 20% | about 147, 221 on a meteor | 0.65 (0.85) | Hot; Blazing on a meteor | 10 coins, or 15; + 10 |
| Twin Engine | about 56% | 10% | about 118 | 0.52 (0.72) | Hot | 12 coins |

"Was" is the same build under the first revision (2 for 1, cap 150). Coins per KO are about what they were,
because the divisor moved with the scale.

The table shows the two axes. Chain builds turn Heat into speed: the foe dies to far fewer moves. Heat builds turn
play into Heat: the meter is high, and the pay and the keystone do the rest. Fires feed Heat at their coefficient,
below 1, so a build cannot make Heat from fires faster than it makes damage.

It also shows what the gentler loss did. The player with no drives gained half again as much Heat and sits where
they sat on the bar. Builds that make Heat by multiplying gain (Sky Pirate, Twin Engine) gained only what they no
longer lose, so they sit lower on the bar than they did, and closer to the driveless player: 1.6 and 1.3 times the
driveless Heat, where they were 2.1 and 1.8. Tech Monk's flat amounts moved with the scale, so it did not slip.
Whether the gain multipliers need raising is for `sim.py` (13.2).

### 9.1 Firestorm (proc chain)

- **Pieces:** Plague Bearer; Ember | Bolt | Wildfire | Cinder Heart, then Conductor in the fifth slot. Four RED
  drives make **Inferno**.
- **How it runs:** every 8% dealt adds a Burning stack. Say Burning averages 2.5 stacks for 12 seconds: 30% from
  ticks. Ember fires on moves only (it gives Burning, so ticks do not charge it); Bolt is charged by moves, by
  Ember's fire and by ticks.
- **The sum (hand-worked):** moves 75% + ticks 30% + Ember once (4%, Cold) + Bolt twice (5% Cold, 6.25% Warm) =
  120%. Bolt's charge: 75 + 2 from Ember + 6 from ticks = 83, two fires. Heat: 75 + 6 (ticks at 0.2) + 2 + 6.75 =
  about 90 gained; two exchanges at Fragile cost 25; **about 65 at the KO**, Warm. The KO fires Wildfire: 12.5% and
  Burning on every other foe.
- **Snowball:** Inferno lifts Burning to 6 stacks and its ticks to 0.5 for Heat (the 6 above becomes 15 or more).
  Each Burning KO makes Heat fill 2% faster through Cinder Heart. Conductor keeps the second link at full strength.
  By act 3 the same play is Hot, so every fire is half again as big.
- **Weak to:** single foes with long invulnerability; the Fragile price, which costs Heat as well as percent. The
  build is cold early: its fires come before the meter has anything in it.

### 9.2 Tech Monk (technique into Heat)

- **Pieces:** Grand Finale; Clean Landing | Metronome | Perfect Form | Crescendo. Two GREEN, two WHITE: no set.
- **How it runs:** 120% of moves, of which six L-cancelled aerials (60%) and a rotation of four or more moves. Weak
  makes the stock take longer: four exchanges, 40% taken.
- **The sum (hand-worked):** moves 120. Perfect Form, live for the last 80%: +40. Clean Landing, six times: +36.
  Metronome, eight changes of move: +40. One clean 50% streak for Crescendo, at about 100 Heat: +25. Gained 261,
  lost 40: **about 221 at the KO**, just under the cap, Blazing for the last quarter. With no drives the same play
  ends at 80.
- **Grand Finale:** at 221 Heat a smash launches 49% harder, so the KO really comes well before 120% (not
  estimated here; the knockback seam is unbuilt, 12.5). That KO pays half again (22 coins, 2.2 pips) and keeps
  nothing.
- **Snowball:** the coins buy Mirror next to Crescendo: x1.25 twice for every clean 50%. A cap-raiser later lets the
  meter pass 225.
- **Weak to:** a player who drops L-cancels. Metronome and Crescendo need none, so the build has a floor: without
  one L-cancel it still gains 120 + 40 + 25 and ends near 145.

### 9.3 Frozen Clockwork (words and retriggers)

- **Pieces:** Winter Oath; Echo | Shatter | Static | Hex | Affliction. Four PURPLE: **Blight**.
- **How it runs:** all damage Chills, so every 20% dealt fires Shatter, and Echo makes it fire again at half
  strength: 9.4% of shards at Warm. Static Shocks the foe every 15% of special damage, so the next Shatter fires
  twice. Affliction reads the words: with one to two on the foe, Heat fills about x1.4.
- **The sum (hand-worked):** moves 78% + three Shatters with their echoes (about 30%) + one Shocked repeat (about
  6%) + Hex, Static and the Marked extra (about 9%) = 123%. Heat: moves 78 x 1.4 = 109; shards 36 x 0.4 x 1.4 = 20;
  the rest about 4. Gained 133, lost 20: **about 113 at the KO**, Hot. Blight pays half again per word on the KO'd
  foe: with two words, 15 coins.
- **Snowball:** Contagion spreads the words across a pack, and Feather Storm delivers them. A second level on Echo
  lifts its repeat to x0.8.
- **Weak to:** bosses, where Chilled and Marked act at half strength [R `codes.rs`]; the Slow price.
- **Tuning note:** the first revision had this at 64% of moves, the fewest of any build but one, because its sum
  took Echo's second fire at full strength. At V2's half strength it is 78%. It still stacks a gain multiplier on a
  retrigger, so it stays the first thing `sim.py` should check (13.2).

### 9.4 Sky Pirate (air and edgeguard)

- **Pieces:** All In or Overclock; Feather Storm | Skyhook | Gatekeeper | Meteor | Updraft, all YELLOW:
  **Skybound**. Active: Recall.
- **How it runs:** three quarters of the damage is aerial and a quarter of it lands offstage. Skybound doubles
  aerial charge, so feathers come every 15% of aerial damage.
- **The sum (hand-worked, with Overclock):** moves 87% (22 on the ground, 43 in the air over the stage, 22
  offstage) + four Feather Storms (30% at Warm) = 117%. Heat: 22 + 43 x 1.5 + 22 x 1.5 x 2 + 30 x 0.5 = 167 gained,
  20 lost: **about 147 at the KO**, Hot. If the KO is a meteor smash, Heat x1.5 reaches 221: 15 coins, plus 10 for
  the edgeguard KO.
- **Snowball:** edgeguard KOs are the best coin source in the game, so this build reaches the shop's Epics first.
  With All In the next stock starts where this one ended. Recall is the way back from deep.
- **Weak to:** stages with walls or no ledges, and the boss fights with no ledge play. All In doubles what a trade
  offstage costs, and a lost stock takes the whole meter.

### 9.5 Twin Engine (the tag, and co-op)

- **Pieces:** Twin Soul; fighter A: Relay | Ember | Bolt | Mirror; fighter B: Slipstream | Tailwind | Snowball |
  Magpie. Active: Volley.
- **How it runs:** with Twin Soul both rows are one line of eight. A tag fires Relay: Ember, Bolt and Mirror's copy
  of Bolt in order, 17.5% at Warm. Slipstream gives Haste on the same tag, and Tailwind makes Heat fill x1.5 for
  those six seconds. Lock allows a tag every 10 seconds: two in the stock.
- **The sum (hand-worked):** moves 56% + two Relay volleys (35%) + Ember once, Bolt twice, Mirror twice (30%) =
  121%. Heat: moves 56, three quarters of it in Haste: 77. Fires 65 at about 0.57, mostly in Haste: about 51. Gained
  128, lost 10: **about 118 at the KO**, Hot. Magpie: 12 coins.
- **Snowball:** the second KO of a fight starts from 59 and reaches about 187, which is Blazing, and from then
  Snowball adds 3 starting Heat per cash-out. Volley is a second Relay on a button. In co-op one player builds the
  chain row and the other the Heat row; each has their own meter, they share the purse, and they meet in the fight
  only through the words on a foe.
- **Weak to:** the Lock price, which makes each tag a commitment.
- **Tuning note:** this now needs the fewest moves of the five (13.2).

A sixth shape exists without being named as a build: the banker (Magpie, Snowball, interest), which is weak for an
act and then buys whatever it wants.

### 9.6 Edge cases, one by one

"Counts" means: feeds Heat and charges drives.

| case | rule | why |
|---|---|---|
| **drill-type multi-hits** (Fox's down air) | counts by the percent each hit deals. Seven hits of 2% are one 14% | the owner's rule; exact in the sim's integers (3.7) |
| **rapid jabs** | the same: each is worth about 1%, and retail's stale-move decay [K] makes each repeat worth less | spamming one move is the worst way to build Heat |
| **projectiles and lasers** (no hitstun) | count in full, by the percent they really deal | a laser is 1-3% [K] and stales with repeats, so chip is slow Heat [D 13.1]. The attacker lookup already resolves a projectile to its owner [R]. If the playtest shows a camping pattern, projectiles get a coefficient (13.2) |
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
| **co-op** | a meter each, one purse. Every player with 10% or more on a foe's stock cashes out on its KO [D, owner]. Player 2 must have joined at the camp; P1 is on port 1 and P2 on port 2, each with the in-place tag [D 13.1] | no reason to steal a KO or to hold one back for a partner |
| **Petey (a boss with HP)** | counts by HP removed. No overkill line. Every quarter of its HP is a **break**: a cash-out, then half kept | without breaks a long boss fight is the "full meter forever" case. **Engine piece:** the boss word reports a hit count per player today [R lib.rs:1557], and Heat needs damage per player |
| **Master Hand, Crazy Hand** | the same as Petey | whether their HP and per-player damage can be read is [U] |
| **packs of weak foes** | one swing through three foes is three lots of damage. Each KO pays at x0.5 and halves Heat | Heat settles at about twice the damage one minion takes, so a pack holds Warm to Hot and the meter saws up and down on every KO |
| **a foe above 150%** | damage above the line counts half | farming a foe that should already be dead is slow |
| **a foe KO'd by its own mistake** | pays whoever has 10% or more on it | the tally, not the last hit, decides |
| **Ice Climbers** | the second climber's damage should count for its player, and damage to it should cost that player Heat | [U]: how the hook reports the second climber was not read |
| **Second Wind** | one stock, and the same fight resumes: no restart, no rewind [D 13.1]. Heat is halved as for any lost stock | how a fight carries on after the pool's last stock is lost was not read [U]; it is not in the slice |
| **between fights** | fight -> pick of three -> (keystone pick after a boss) -> map -> next fight. No character select [D 13.1] | the pair is picked once, at the camp |
| **foes' drives** | charge by damage too. Foes have no meter: a foe's fires use Cold, an elite's Warm, a boss's Hot | one less thing on screen; a foe meter can come later |

## 10. HUD: what must be readable in a fight

This section lists elements and states only. The look is a separate UI pass ("Party pack" sticker style). The Heat
meter replaces the first draft's two numbers and timer bar.

**The Heat meter (one per player, by that player's percent)**

- how full it is, 0 to the cap
- three marks on it: Warm, Hot, Blazing
- the current state, one of four, readable without reading the fill. The fighter also shows it
- a gain: sized by the amount, so a drill reads as a smooth rise and a smash as one jump
- a loss: the piece that was lost is seen leaving. At 1 for 1 a loss can be a few points, so the look may give it
  a least size
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
- coins: the party's one purse (changes only at a cash-out and at boundaries)
- co-op: two meters, each clearly its owner's; one purse
- Insulate's allowance, while it lasts (a shell on the meter)

The exact data behind each of these is the table of 12.8.

**No digits in a fight.** The first draft showed two numbers. The meter needs none, so the "no text on a fire, a
pickup or a level-up" ruling [O] now holds everywhere in a fight. Sound should carry the state as well (a tone per
state), so the player need not look.

Budget, on top of core v2 3.3: one chain at a time per fighter, four links, six frames apart; one set look per
fighter; the cash-out is the only effect larger than a fire, at most 30 frames.

## 11. From V2: what is reused and what changes

| V2 piece [R `envoy-drives.md`] | V3 |
|---|---|
| `pool.toml` -> `build.rs` -> `pool_gen.rs`, `POOL_DIGEST` | **Reuse.** New fields: `tier`, `kind`, `coef`, `heat_add`, `gain_mul`, `heat_mul`, `mark` (a technique), `set`, `duo`. `need` becomes a percent x1000 for damage sources |
| the lint | **Reuse and extend.** New: every trigger has a coefficient; every `xG` is Rare or Epic and every `xH` Epic; every technique drive has a same-family drive without one; no charge source counts hits, landings or jumps; a fire returns less charge than it cost (4.3); the whole pool in any order ends a chain inside 4 links |
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
| per-player drops, picks, pool 5 / 4, co-op | **Reuse** [D]; Heat and the active are per player; coins are earned per player into one purse |
| the pick of three's "connects to a word" rule | **Change:** connects to a word, a family toward a set, or a technique used this run |
| the run record v4 | **Change:** v5 adds order, scaler counters, the purse, the active and its pips (12.5). Heat is not in it: it starts at 0 each fight |
| `sim.py` (seeds x policies) | **Reuse and extend:** a damage model per policy and the measures of 12 |
| the UI's orbiters, halo, link flash, Brawl sounds | **Reuse**; new: the Heat meter, continuous charge, the chain line, set looks |
| the 20 drives | 18 kept (all restated on damage), 2 set aside (Spire, Surge), 22 new |
| "the full build is 0.6 s between fires" tuning | **Change:** the dials are the build spread and the character spread (12.10) |

New host work, all small: `gd_envoy_events` (the L-cancel bit first, the meteor's angle later), the boss word
carrying damage, the knockback write for Grand Finale, `gd_envoy_shape` later (element, shield damage, hitlag,
size), a stage ledge table in the sim, raising `state_bytes` and `VIEW_WORDS`. The three the slice needs are
specified, with their seams, in 12.5.

## 12. The V3 slice: build spec

Core v2's milestones V3-V11 each move down one; this slice takes the number V3. Off `port/v2`. Nothing in this
section has been built or run. Line numbers are this branch at `40e750a48`; each was re-read for this revision
unless tagged [U].

### 12.1 Scope

| piece | in the slice |
|---|---|
| Heat | section 3 in full: gain by damage, the overkill line, the 1-for-1 loss, four states and their scale, the cash-out (coins, pips, flare), half kept, the caps, boss breaks |
| damage charge | every trigger's need in percent, remainders kept; the hit counter removed |
| chains | 4.1 in full: coefficients, rightward only, 4 links, 6 frames |
| order | ordered slots, 3 / 4 / 5; reorder at the rest stop and while placing a new drive; full rows replace or skip (5.4) |
| technique | L-cancel (one engine event bit), wavedash, tech, offstage damage (polled; a ledge table for the 5 Short-run stages) |
| content | 16 drives, 3 keystones, 1 active, 1 set (12.6) |
| coins | one purse; the plain shop at each rest stop (12.9) |
| tiers | Common and Rare as data and as a beam colour (the 16 hold no Epic) |
| flow | fight -> pick of three -> (keystone pick after a boss) -> map -> next fight; no character select between fights (12.9) |
| co-op rules | the sim holds two meters, two tallies and one purse from the start, and every player with 10% on a foe's stock cashes out. V2's co-op synctest must still pass. Co-op HUD polish is not in the slice |
| HUD | the data of 12.8, drawn with stand-in art and sounds |
| engine | the three hooks of 12.5 |

**Out of the slice:** the rest stage as a place (the shop is a screen), the pact (there is no Epic to offer), Second
Wind (decided in 13, but how a fight resumes after the last stock is lost was not read [U]), editions, duo drives,
`gd_envoy_shape` (element, shield damage, hitlag, size), four of the five actives, five of the six sets, `xHeat`
drives, scalers, cap-raisers, a foe meter, the meteor's angle.

### 12.2 Numbers: the fixed-point representation

Everything is an integer in the sim's one hashed struct. No floats after the hook, no chance rolls. This replaces
the "millionths in an `i32`" sketch of the first revision: Heat needs a finer unit so that the overkill half, a
chain coefficient and a gain multiplier can all be applied **without a division**.

| quantity | type | unit | notes |
|---|---|---|---|
| damage `dmg_m` | `i32` | 1% = 1,000 | made once, at the hook: `(d * 1000.0 + 0.5) as i32` from the engine's `f32` [R `on_damage(h, d: f32)`]. V2 converts at the same place [R `x100`, lib.rs:937] |
| a foe's percent this stock `foe_m[slot]` | `i32` | 1% = 1,000 | the sim's own sum of every `dmg_m` that foe took (any source) plus what the sim itself added through `gd_envoy_percent`. Used for the 150% line, so the line never depends on a float. Reset when that foe loses a stock |
| coefficient `coef_m` | `i32` | x1 = 1,000 | a move: 1,000. A fire: drive coefficient x parent / 1,000, rounded down once per link. Word damage: 200 (500 under Inferno) |
| gain multiplier `gain_m` | `i32` | x1 = 1,000 | 1,000, then x each live `xGain` value / 1,000, left to right, rounded down after each; Glass Core's x2 is one more step; capped at 4,000 |
| **Heat** `heat[p]` | `i64` | **1 Heat = 2,000,000,000 ticks** | 2 (the overkill half) x 1,000 (`dmg_m`) x 1,000 (`coef_m`) x 1,000 (`gain_m`) |
| Heat gain | `i64` | ticks | `(2 * below_m + above_m) * coef_m * gain_m`, where the record's `dmg_m` is split at 150,000 on `foe_m` into `below_m` and `above_m`. A boss with HP has no line: all of it is `below_m` |
| Heat loss | `i64` | ticks | `paid_m * loss_m * 2,000`, where `loss_m` is 1,000 (All In: 2,000) and `paid_m` is the damage left after Insulate's allowance. Never below 0 |
| `+Heat` | `i64` | ticks | amount x1000 `* 2,000,000` |
| the cap | `i64` | ticks | 225 Heat = 450,000,000,000. Heat is clamped after every step |
| marks | `i64` | ticks | Warm 40, Hot 90, Blazing 165 (each x 2,000,000,000). State = the highest mark at or under `heat[p]` |
| **charge** `charge[set][slot]` | `i64` | **1% = 1,000,000,000 units** | gain `dmg_m * coef_m * rate_m`. `rate_m` is 1,000 (the slice has no charge-rate drive; Overclock and Skybound will use it). The need is `need_m * 1,000,000`. On a fire the need is subtracted and **the remainder stays**. The 150% line does not apply to charge |
| timed charge | `i32` | frames | +1 a frame while the fight runs; need in frames (12 s = 720) |
| a fire's damage | `i32` | 1% = 1,000 | `number_m * level_m / 1000 * state_m / 1000`, rounded down; level 1,000 / 1,600 / 2,400 [R `LEVEL_SCALE`]; state 1,000 / 1,250 / 1,500 / 2,000; clamped to 30,000 (a launching timed fire 40,000) |
| the tally `tally_m[p][slot]` | `i32` | 1% = 1,000 | every record of player p on that foe this stock; a cash-out needs 10,000 |
| Insulate's allowance `free_m[p]` | `i32` | 1% = 1,000 | 3.3; refilled at a fight's start and after each cash-out |
| Burning's stack counter | `i32` | 1% = 1,000 | a stack per 8,000 dealt to that foe, remainder kept |
| the purse `purse` | `i64` | 1 coin = 30,000,000,000,000 | a cash-out adds `heat * pay_m` (15 Heat a coin, and `pay_m` x1000: minion 500, fighter 1,000, elite 1,500, break 1,000, last break 2,000; Grand Finale's smash KO x1.5 on top, rounded down). Whole coins are `purse / 3e13`; flat coin awards and prices add or subtract whole coins. One purse for the party; `earned[p]` keeps each player's share for the end screen |
| pips `pips[p]` | `i64` | 1 pip = 300,000,000,000,000 | the same product; 150 Heat a pip |
| the flare | `i32` | 1% = 1,000 | `min(15,000, heat * pay_m / 30,000,000,000)` |
| Grand Finale's launch bonus | `i32` | x1 = 1,000 | `1,000 + min(500, heat / 900,000,000)`: +1% per 4.5 Heat |

What is exact and what is not:

- **Exact:** Heat gain, charge, the tally, coins and pips are sums of products with no division, so n records adding
  up to one record's `dmg_m`, at the same coefficient and multiplier, give identical results (the split test,
  12.10). The 150% line is exact too, because it is placed on the sim's own integer.
- **Rounded, once each:** the damage at the hook (at most 0.0005% a hit, and the same on both peers, or rollback
  would already be broken [I]); a chain coefficient per link; the gain multiplier per `xGain` step; a fire's damage;
  the KO's halving (`heat / 2`); the flare.
- **Not split-proof by design:** the frame a drive fires on (3.7).
- **Unverified:** that `foe_m` tracks the percent the player sees closely enough. It ignores healing and anything
  that changes a foe's percent outside the hook and the sim's own writes [U].

### 12.3 State added to the sim

Per player (2): `heat`, `state`, `pips`, `earned`, `free_m`, `tally_m[4]`, the technique marks (frames left:
`lcancel`, `wavedash`, `tech`), the last three aerial landings (3 bits) and the last three attack states (for
Perfect Form and Metronome), the active's id. Per set (4): `drive[5]`, `level[5]`, `charge[5]`, timed counters.
Per match slot (4): `foe_m`, the Burning counter and stacks, the word timers, the last player to add a Burning
stack. Global: `purse`, the frame's record buffer (24 records), the chain queue per fighter (the owner, the next
slot, the link number, the parent's `coef_m`, frames to the next link, frames to the next chain), the event ring
of 12.8, the boss's last damage words and break count.

The manifest's `state_bytes` is 4096 [R lib.rs:2149]; raise it to 16384 [J]. The host allows up to 256 KiB
[R `melee_modhost` lib.rs:860].

### 12.4 The frame: update order

**During the engine's frame** (the take-damage hooks, in the engine's own order):

- `on_damage_pre` [R lib.rs:1391]: Bulwark's orb, as V2. Not used by the slice's 16.
- `on_damage` [R lib.rs:1412]: make `dmg_m`; resolve the attacker [R `gd_envoy_attacker`]; read the tags that only
  exist now (the attacker's action state for AERIAL / SMASH and for Metronome; the victim's position against the
  ledge table for OFFSTAGE); add `dmg_m` to `foe_m`; **append one record** to the frame's buffer. Grand Finale: if
  the attacker is a player in a smash attack state, call `gd_envoy_kb_scale` here (12.5). Nothing else is decided
  in the hook. A hit with no fighter source is a hazard: recorded with no owner (this removes the credit at
  lib.rs:1424).
- If the buffer is full, the record's `dmg_m` is added to the last record with the same owner and victim. That
  keeps the totals; only its tags are lost. With four fighters it should not happen [I].

**At frame end** (`on_frame_end` [R lib.rs:2028]), only while the fight runs and is not frozen (tag cinematic,
reward freeze, hold). A frozen frame clears the buffer (its damage is already in `foe_m`) and steps nothing else.

1. **Read the frame's inputs**: `gd_envoy_events(slot)` for each player (read and clear), every fighter's action
   state, position and stocks, the boss damage words, `gd_pad_pressed`.
2. **Technique marks**: count the live marks down by one; then set new ones from this frame's inputs (the L-cancel
   bit: 20 frames; LandingFallSpecial from EscapeAir: 60 frames; a Passive state: this frame). Shift the landing and
   attack-state histories. Each new mark writes a spark event (12.8).
3. **The records, in buffer order.** For each:
   - a player was hit by a foe or a hazard: Insulate's allowance, then the Heat loss; a foe's drives charge by the
     damage (as V2, on damage);
   - a foe was hit by a player's side: the Heat step of 3.3, steps 3 to 9 (eligible damage, gain multiplier, gain,
     `+Heat` drives left to right, charge, the tally). Drives that fill are **marked full**, not fired yet;
   - anything else (self, friendly, a hazard on a foe): nothing.
4. **The boss**: each player's new boss damage (the delta of the words of 12.5) is one more "foe was hit" record.
   If the boss's HP crossed a quarter, that is a break: a cash-out (step 8).
5. **Word damage**: a foe whose Burning tick is due (every 60 frames [R `BURN_TICK`]) takes stacks x 1% through
   `gd_envoy_percent`; that is a record at 200 (Inferno: 500) for the player who added the last stack. It feeds
   Heat. It charges only under Plague Bearer, and then never Ember or Wildfire.
6. **Timed charge**: +1 frame on each timed drive.
7. **The active**: D-pad right pressed, the fighter actionable, 3 pips held: spend them; Volley queues every
   trigger drive of the fighter in play, left to right, as one chain with no link limit.
8. **KOs.** For each foe that lost a stock this frame [R `foe_ko`, lib.rs:1589]: every player with
   `tally_m >= 10,000` on it cashes out, player 1 first: purse, pips, the flare on every other foe (it feeds no
   Heat, charges nothing and joins no tally), Wildfire marked full if the foe was Burning; then `heat /= 2`
   (Grand Finale's smash KO: 0), `free_m` refills, and that foe's tallies, `foe_m` and Burning reset. For each
   player who lost a stock: `heat /= 2`.
9. **Chains**, per fighter: if no chain is running, 20 frames have passed since the last one ended, and a drive is
   full, start one at the leftmost full drive (link 1, this frame). If a link is due, fire it: subtract its need,
   compute its damage at **this frame's** state, apply it (`gd_envoy_percent`; `gd_envoy_blast` for Nova), set its
   word, then run 3.3 on the fire's own record at once: Heat, and charge **only to its right**. Echo to its left:
   the same drive fires again 6 frames later at Echo's strength; that repeat is not a link. Then queue the next
   full drive to the right, 6 frames on, up to 4 links. Each fire writes a log entry with its link number and
   parent.
10. **State**: recompute each player's state from `heat`; a change writes an event.
11. **Publish** the view (12.8) and hash the state [R lib.rs:1896].

Three consequences worth knowing. A move's damage charges and can start a chain on the same frame it lands (step 3
then step 9). A KO's halving happens before that frame's chain link fires, so a KO-frame fire is sized by the
halved Heat. Grand Finale's launch bonus is written in the hook, from the Heat of the frame before.

### 12.5 What the sim needs from the engine

Three new host imports. Each is inert unless an Envoy run is armed, so goldens and replays do not change
[R `envoy.rs` header].

| hook | what | the seam | status |
|---|---|---|---|
| **the L-cancel event bit** | `gd_envoy_events(slot) -> u32`, read and clear; bit 0 = "this fighter L-cancelled a landing this frame" | The engine's own test is `(f.x67F as i32) < cdat.xE4`, in the branch that sets `skill_mode = 1`: `rust/melee_fighter/src/tu/ftCo_LandingAir.rs:65-66`. `melee_fighter` cannot call `melee_modes` (it is the other way round in both `Cargo.toml`s), so the site calls an `Option<fn(&Fighter)>` in `rust/melee_fighter/src/hooks.rs`, the pattern of `MOD_TAKE_DAMAGE` (:14) and `GENO_GROUND_EDGE` (:86). `melee_modes::tu::envoy` installs it and sets the bit in a new `static mut EVENTS: [u32; 4]` beside `VIEW` (`envoy.rs:49`); the import goes beside `gd_envoy_absorb` (`melee_modhost/src/lib.rs:676`) | S. Not built. [U] that line 65 is the only way into an L-cancel; it is the only test of `x67F` against `xE4` I found. A sub-fighter (the second Ice Climber) is ignored |
| **the boss word carrying damage** | `gd_envoy_boss_dmg(out: *mut u32) -> i32`: six words: damage dealt to the boss by each of the four slots (x1000, running totals that wrap), the boss's HP, its full HP. A new import, so the old six-word `gd_envoy_boss` and its callers are untouched | Today's word carries a hit **count**: `HITS` at `rust/melee_modes/src/tu/petey.rs:201`, cleared at :190, bumped at :1048, sent by the closure at `melee_modhost/src/lib.rs:684-690`, read by the sim at lib.rs:1556-1571 and 1737-1739. The hit's damage is in hand one line above the bump (`let dmg = (*h).damage * ratio;`, petey.rs:1045). A second `static mut DMG: [u32; 4]` is added there. HP: the words `gd_ui_petey` already publishes (petey.rs:1117) | S. Not built. [U] that `dmg` at :1045 is exactly what the boss loses. Master Hand and Crazy Hand were not read. Only needed if the Short run's boss is Petey (`MELEE_ENVOY_BOSS=petey`); the stand-in boss is a fighter and uses the ordinary hook |
| **the knockback write** | `gd_envoy_kb_scale(slot, mul_milli) -> i32`: multiply the knockback of the hit that fighter is taking right now; `mul_milli` clamped to 1,000-1,500 | A new `kb_scale` beside `absorb` (`rust/melee_modes/src/tu/envoy.rs:417-424`), which already writes the same field (`(*f).dmg.kb_applied = 0.0`, :422); the import beside `gd_envoy_absorb` (`melee_modhost/src/lib.rs:676`). Timing: the take-damage hook wraps `Fighter_TakeDamage_8006CC7C` (`fighter.rs:2633-2640`), which the hit path reaches from `fighter.rs:2821` with `kb_applied` already set; `ftCo_Damage_CalcKnockback` reads the field on the next line (:2822; `ftCo_Damage.rs:233-262`). So a write made in the hook lands before the crouch, charge and armour steps, as a stronger hit would | S. Not built. [I] from reading; V2's orb proves a write there takes effect (it zeroes the launch). [U]: whether the attacker is still in the smash state when a late hit or a smash's projectile lands. If it slips, All In replaces Grand Finale in the slice: it needs no engine work |

Also changed, but not hooks:

- **The view grows**: `VIEW_WORDS` 512 to 768 (`envoy.rs:21`, `envoy_data/src/view.rs:4`); a new magic "ENV3".
- **The run record becomes v5** inside the same 48 words (`REC_WORDS`, `envoy.rs:80`): drives as `[4 sets][5]` in
  slot order (20 words, 20-39), the pool digest (40), the purse (41-42), the active (43), player 1's pips (44-45),
  one spare, the record's digest last. Player 2's pips do not fit and start at 0 on a resumed run [J]. The host's
  `rec_valid` (:102) was not read for a version check [U].
- **No engine work**: wavedash, tech and offstage damage are polled from action states and positions the sim
  already reads; the ledge table for the five Short-run stages [R lib.rs:115] is sim data. D-pad right is already
  in the pad word (`DPAD_RIGHT`, lib.rs:99; `gd_pad_pressed`, :60). Fires use `gd_envoy_percent`
  (`envoy.rs:338`) and `gd_envoy_blast`. Restoring a stock uses the path V2's stock offer uses (`OFFER_STOCK`).

### 12.6 The content, with final numbers

Level 1. A duplicate levels a drive (to 3): its **number** is x1.0 / x1.6 / x2.4. For an `xGain` drive the number
is the part above x1, so x1.5 / x1.8 / x2.2. All are first guesses [J] for `sim.py` to move.

**Drives (16)**

| # | key | family | tier | kind | charge | need | number | coef | words | rule (the card's text) |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `ember` | RED | C | trigger | dealt | 40% | 4% | 0.5 | gives Burning | Every 40% dealt: a fire burst, 4% and Burning. |
| 2 | `bolt` | RED | C | trigger | dealt | 30% | 5% | 0.6 | - | Every 30% dealt: lightning strikes the nearest foe, 5%. |
| 3 | `nova` | RED | R | trigger | time | 12 s | 8% | 1.0 | - | Every 12 seconds: a blast around you, 8%. It launches. |
| 4 | `wildfire` | RED | R | trigger | cashout | on a Burning foe | 10% | 0.5 | reads and gives Burning | A cash-out on a Burning foe: 10% and Burning to every other foe. |
| 5 | `rush` | GREEN | C | trigger | dealt | 50% | 4 s | 0 | gives Haste | Every 50% dealt: Haste for 4 seconds. |
| 6 | `tailwind` | GREEN | R | gain_mul | - | - | x1.5 | - | reads Haste | In Haste, Heat fills x1.5. |
| 7 | `clean_landing` | GREEN | C | heat_add | - | - | +6 Heat | - | - | An L-cancelled aerial that dealt damage: +6 Heat. |
| 8 | `perfect_form` | GREEN | R | gain_mul | - | - | x1.5 | - | - | While your last three aerial landings were L-cancelled: Heat fills x1.5. |
| 9 | `riposte` | BLUE | C | heat_add | - | - | +10 Heat | - | - | A tech: +10 Heat. |
| 10 | `insulate` | BLUE | R | shaper | - | - | 20% | - | - | The first 20% you take in a fight, and after each cash-out, costs no Heat. |
| 11 | `feather_storm` | YELLOW | C | trigger | aerial_dealt | 30% | 2% x 3 | 0.5 | - | Every 30% of aerial damage: three feathers, 2% each, at three different foes if there are three. |
| 12 | `gatekeeper` | YELLOW | R | gain_mul | - | - | x2.0 | - | - | Damage to an offstage foe fills Heat x2. |
| 13 | `frost` | PURPLE | C | trigger | dealt | 40% | 2% | 0.3 | gives Chilled, 4 s | Every 40% dealt: 2% and Chilled for 4 seconds. |
| 14 | `shatter` | PURPLE | C | trigger | chilled_dealt | 20% | 5% | 0.4 | reads Chilled, ends it | Every 20% dealt to Chilled foes: ice shards, 5%, ending the Chill. |
| 15 | `echo` | WHITE | R | neighbour | - | - | x0.5 | - | - | The drive to the right fires twice; the second time at half strength. |
| 16 | `metronome` | WHITE | C | heat_add | - | - | +5 Heat | - | - | Damage from a move that is not one of your last three: +5 Heat. |

Notes that the table cannot hold:

- **Once per event, not per record.** Clean Landing pays once per landing, when the mark is set and the aerial just
  ended dealt damage. Metronome pays once per move: on the first record of an attack state that is not one of the
  last three. Riposte pays once per tech. A multi-hit move is one event for all three.
- **Echo** acts on a trigger only; next to anything else it does nothing. Its strength is V2's number, carried
  [R `pool.toml`: 500]: x0.5 / x0.8 / x1.2 by level. The repeat never launches.
- **Riposte** reads a tech only in the slice. The power shield joins it once the GuardReflect poll is checked [I].
- **Perfect Form** needs three aerial landings in a row; landing without an aerial does not break it. With no
  L-cancel bit it and Clean Landing take the wavedash mark instead (damage within 60 frames of a wavedash).
- **Two-giver rule.** In the full pool every word read has two givers. In these 16, Chilled has one (Frost) and
  Haste has one (Rush). The slice's lint asks for one; the pick of three's "connects" card (7.5) covers the gap.
- **The lint of 4.3** by hand: the tightest are Bolt and Shatter at 0.1 (level 1, Cold) and 0.48 (level 3,
  Blazing). With Echo at level 3 on its left, Shatter's worst is 0.48 x 2.2 = 1.06. That is over 1, and still
  cannot loop, because charge only moves right and a drive fires once per chain; the lint's bound for an echoed
  drive is 1.1.

**Keystones (3)**

| key | rule | price [R `PRICE_TEXT`] |
|---|---|---|
| `plague_bearer` | Each 8% you deal to a foe adds a Burning stack, to 3. Burning ticks charge your drives at 0.2, except Ember and Wildfire. | Fragile: you take 25% more. |
| `glass_core` | Heat fills twice as fast. | own: you take 60% more. |
| `grand_finale` | Your smash attacks launch harder with Heat: +1% knockback per 4.5 Heat, to +50% at a full meter. A KO from a smash attack pays half again and keeps no Heat. | Weak: you deal 20% less. |

In the slice a run holds up to three, so all three by the end: one at the start, one after each of the first two
bosses, each a pick from what is left. If the knockback write slips, `all_in` (8.3) takes Grand Finale's place.

**The active (1):** `volley`, 3 pips, D-pad right. Every trigger drive on the fighter in play fires now, left to
right, as one chain with no link limit. Given at the run's start with its 3 pips full, so it is used in the first
fight. Hand-worked: a Short run cashes out about 1,150 Heat with no drives (ten foe stocks at the reference pace),
so about seven more pips, two more uses.

**The set (1):** `inferno`, three different RED drives on one fighter (four of the 16 are RED). Burning stacks to
6, and its ticks feed Heat at 0.5 instead of 0.2.

**Words in the slice:** Burning (1% a second per stack, 4 seconds, refreshed [R `BURN_FRAMES`]; one stack from
Ember or Wildfire, more only under Plague Bearer or Inferno), Chilled, Haste. Marked and Shocked have no giver in
the 16.

### 12.7 The data format for a drive

`rust/mods/envoy_data/pool.toml` -> `build.rs` (the lint) -> `pool_gen.rs`, as V2 [R]. One `[[drive]]` table per
drive. All numbers are integers, x1000.

| field | values | meaning |
|---|---|---|
| `key`, `name`, `family`, `rule`, `look`, `sound` | as V2 | `rule` is the card's text; `look` and `sound` are never read by the sim |
| `tier` | `common` \| `rare` \| `epic` | new |
| `kind` | `trigger` \| `shaper` \| `heat_add` \| `gain_mul` \| `heat_mul` \| `neighbour` \| `scaler` \| `economy` | new; replaces V2's `passive` charge |
| `charge` | `dealt` \| `aerial_dealt` \| `special_dealt` \| `offstage_dealt` \| `chilled_dealt` \| `taken` \| `time` \| `tag_in` \| `cashout` | triggers only. The lint rejects any source that counts hits, landings or jumps |
| `need` | x1000 percent, or milliseconds for `time`; absent for `tag_in` and `cashout` | 40% is `40000` |
| `effect` | one of `codes.rs` `EFFECTS` | what the fire does |
| `number` | x1000: a percent, a time, a Heat amount, a multiplier | the one number a level scales |
| `coef` | 0-1000 | required on every trigger; 1000 only with `launch = true` |
| `launch` | `true` \| absent | only with `charge = "time"` [D] |
| `when` | `haste` \| `aerial` \| `offstage` \| `lcancel` \| `lcancel_streak` \| `tech` \| `new_move` \| `burning_foe` | the event or condition of a `heat_add` or `gain_mul` drive, or the filter on a `cashout` trigger |
| `gives`, `reads` | a word | as V2 |
| `set` | absent | a set is the family; nothing to write |

Two examples:

```toml
[[drive]]
key = "ember"
name = "Ember"
family = "RED"
tier = "common"
kind = "trigger"
charge = "dealt"
need = 40000
effect = "burst"
number = 4000
coef = 500
gives = "Burning"
look = "burst"
sound = "fire_red"
rule = "Every 40% dealt: a fire burst, 4% and Burning."

[[drive]]
key = "tailwind"
name = "Tailwind"
family = "GREEN"
tier = "rare"
kind = "gain_mul"
when = "haste"
number = 1500
reads = "Haste"
look = "line"
sound = "fire_green"
rule = "In Haste, Heat fills x1.5."
```

Keystones keep V2's `[[keystone]]` table (a rule, an effect, a price). The active and the set are `[[active]]`
(key, name, `pips`, effect, rule) and `[[set]]` (family, name, effect, rule). The Heat constants (the marks, the
cap, the loss rate, the divisors of 3.5) live in `codes.rs`, not in the pool, and are part of `POOL_DIGEST`.

### 12.8 What the HUD reads each frame

The sim publishes one view [R `gd_envoy_publish`; `envoy_data/src/view.rs`]; the UI mod only reads it. V2's
layout stays where it can; these are the additions. Offsets are assigned in `view.rs`; the names and units are the
contract.

| field | per | unit | for |
|---|---|---|---|
| `P_HEAT` | player | Heat x1000 | the fill |
| `P_HEAT_CAP` | player | Heat x1000 | the bar's length (225,000 in the slice) |
| `P_HEAT_MARKS` (3) | player | Heat x1000 | the three marks (40,000 / 90,000 / 165,000), so the UI holds no constants |
| `P_HEAT_STATE` | player | 0-3 | Cold, Warm, Hot, Blazing: exact, from the sim |
| `P_HEAT_FULL` | player | 0 / 1 | at the cap this frame: damage is being discarded |
| `P_HEAT_FROZEN` | player | 0 / 1 | the meter is not stepping |
| `P_GAIN_MUL` | player | x1000 | the gain multiplier live right now |
| `P_GAIN_SRC` | player | bits, by slot | which `xGain` drives are live |
| `P_FREE` , `P_FREE_MAX` | player | percent x1000 | Insulate's allowance |
| `P_DRIVES`, `P_LEVELS` | player, 2 fighters x 5 slots | id + 1; level | the row, in order (V2 has 4 slots [R `SLOTS`]) |
| `P_CHARGE` | player, 2 x 5 | 0-1000 | **each drive's charge fraction**; 1000 = ready. Timed drives use the same field. Insulate shows its allowance; a non-trigger shows 0 |
| `P_SET` | player, 2 fighters | set id or 0 | the persistent look |
| `P_ACTIVE`, `P_PIPS`, `P_PIPS_NEED`, `P_PIP_PART` | player | id + 1; whole pips; pips needed; 0-1000 toward the next | the active, "ready", and the D-pad right prompt |
| `P_MARKS` (3) | player | frames left | L-cancel, wavedash, tech: a live mark |
| `P_EARNED` | player | coins | each player's share |
| `V_COINS` | run | coins | the purse |
| the fire log | 12 entries | V2's eight words [R `V_LOG`]; the flags word gains the link number (bits 8-11) and the parent's log index + 1 (bits 12-15) | **chain events**: a fire, its size (the level and the state at the fire), a repeat (`FL_ECHO`), the link flash from parent to child |
| the event ring `V_EV` | 16 entries x 4 words: frame, kind, who (player, or a foe slot), amount | - | everything that is a moment: see below |
| `S_BURN`, `S_CHILL`, `S_HASTE` | match slot | as V2 | the words on each foe |
| `S_BREAKS` | match slot | 0-4 | a boss's quarters, if the boss is in |

Event kinds in the ring: `EV_SPARK` (**a technique registered**; amount = which; it is written whether or not a
drive paid for it, so it works as a training aid), `EV_HEAT_ADD` (a `+Heat` drive paid; amount = slot and Heat),
`EV_STATE_UP`, `EV_STATE_DOWN` (amount = the new state), `EV_CASHOUT` (amount = Heat cashed, x1000), `EV_PAY`
(amount = coins paid), `EV_PIP` (amount = thousandths of a pip paid), `EV_FLARE` (who = the foe hit),
`EV_BREAK`, `EV_ACTIVE` (the active was used), `EV_LOSS` (amount = Heat lost, x1000, summed over the frame).

Gains need no event: the UI draws them from the change in `P_HEAT`, which is why a drill reads as a smooth rise.
Losses get one because the UI must show the piece leaving, and at 1 for 1 a loss can be small; the look may give
it a least size (10).

### 12.9 The run and the shop in the slice

The Short run is five fights: a duel, an elite (the first act's boss), a pack, an elite (the second act's boss),
the final boss [R lib.rs:1-20, `SCHEDULE` :136].

| after | then |
|---|---|
| the camp | P2 joins here or not at all; the first keystone pick; Volley given; 3 slots |
| fight 1 (duel) | pick of three -> map -> **shop** -> fight 2 |
| fight 2 (boss 1) | pick of three -> keystone pick -> the 4th slot -> map -> fight 3 |
| fight 3 (pack) | pick of three -> map -> **shop** -> fight 4 |
| fight 4 (boss 2) | pick of three -> keystone pick -> the 5th slot -> map -> **shop** -> fight 5 |
| fight 5 (final boss) | the run's end |

There is no character select after the camp. The map in the slice is a plain line of five marks; whether V2 has
any map screen to reuse was not read [U]. The shop is one screen: three drives from the 16, a level for a worn
drive, a stock, a reroll, and the rows to reorder, at the prices of 7.3.

Hand-worked coins, no drives, reference pace: fight 1 pays 6 from its one cash-out and 20 to 30 for the clear, so
the first shop holds about 30: one Common, or nothing and the interest. A two-stock fight pays about 15 and 20 to
30; the run about 220 before interest.

### 12.10 Acceptance checks

One script, `tools/rust-port/envoy/v3_slice.sh`, in the shape of `v2_drives.sh` [R `envoy-drives.md`]. Every
game run in it is driven by a pad script, at volume 3, in a titled window.

**A. The lint and its tests** (`envoy_data` `build.rs`, no game):

- every trigger has a coefficient; 1000 only on a launching timed drive
- every `gain_mul` is Rare or Epic; every `heat_mul` is Epic
- every technique drive has a same-family drive with no technique condition
- no charge source counts hits, landings or jumps
- every word read has a giver (two in the full pool)
- a fire returns less charge than it cost (4.3): at most 0.1 at level 1 Cold, 0.5 at level 3 Blazing, 1.1 for a
  drive with Echo on its left
- every order of every five of the 16 ends a chain inside 4 links
- **the split test**: for each of the 16 drives alone and for ten fixed five-drive rows, feed one record of N
  (N = 2%, 14%, 30%, 160% across the 150% line) and then the same total as 2, 7 and 30 records. `heat`, every
  `charge`, `tally_m`, `purse` and `pips` must be **bit-identical**. The same again for damage taken, with and
  without Insulate

**B. `sim.py`** (`tools/rust-port/envoy/sim.py`, extended with a damage model per policy; exit 1 on a missed
band):

| # | measure | band |
|---|---|---|
| 1 | **Build spread**: the percent a build deals (fires, word damage, flares) per 100% of move damage; the best five-drive build over a random one, per policy | 2.5 to 6 |
| 2 | **Heat spread**: median Heat at a KO, best build over random build | 1.5 to 2.5 |
| 3 | **Character spread**: one build, one total of damage, four profiles (a few big hits, many small hits, a multi-hit move, projectile chip); highest build output over lowest | at most 1.25 |
| 4 | **Split**: the lint's split test, repeated on the model | identical |
| 5 | **Use**: the share of the 16 drives in at least one of the top ten builds | at least 12 of 16 |
| 6 | **KO delay**: coins gained by a policy that delays KOs to cash out higher, over one that does not | under 15% |
| 7 | **Pace** (new, the owner's answer as a number): no drives. At the reference pace (deal 30, take 10) Heat at the first KO is 80 to 100, and the player is not Blazing before the fourth KO. Trading two for one (deal 30, take 15) the player is Warm by the first KO. Trading evenly the player never leaves Cold | all three hold |

**C. Synctests**: depth 4 and depth 7, 3600 frames each, **0 mismatches**, over these runs:

1. a chain of three links, with Echo
2. a cash-out, the flare, and Heat halved
3. every state change, up and down, and a player's stock lost
4. each technique mark (L-cancel, wavedash, tech) paying a drive, and offstage damage
5. Volley on D-pad right
6. a shop buy, a level, a reroll, a reorder, and a full row replaced
7. Grand Finale's launch bonus (the knockback write)
8. co-op: two meters, both players cashing out on one foe, one purse
9. the Petey boss with breaks, if the boss word is in

V2's six synctests [R `envoy-drives.md`] must still pass on the new rules.

**D. Inert when off**: with the mods loaded and no run armed, and with no mods loaded, the goldens and replays
are unchanged.

### 12.11 The playtest, size and models

Two Short runs. The reward screen is nudged toward Firestorm in one and Tech Monk in the other, on **two different
characters, one of them with no real combo game**. The questions for the owner: "did those two runs play
differently, did the meter build on both characters, and did you want a third?".

M (one to two weeks). Sonnet: pool data, the lint and the split test, the sim's Heat and chain state, `sim.py`,
the synctests. Opus: the Heat meter and its feel, the chain line, the shop screen.

## 13. Decisions and open risks

### 13.1 Decisions (all dated 2026-11-10)

"Owner" is the owner's own answer. "Delegated" was decided by the author, on the owner's instruction, in the
owner's pattern: nothing counted by hit number or by strict combos; charge by damage dealt; gentler penalties;
every character can use it; plain over clever; otherwise the recommended option.

| # | decision | who |
|---|---|---|
| 1 | Taking a hit costs **1 Heat per 1% taken**. "Trades build slowly" [O] | owner |
| 2 | A KO pays out on all the Heat held, then the player keeps half | owner |
| 3 | Heat never falls over time | owner |
| 4 | Heat starts at 0 in every fight | owner |
| 5 | Co-op: every player with 10% or more on that foe's stock cashes out | owner |
| 6 | Lasers and other no-flinch projectiles fill Heat in full | delegated |
| 7 | Timed, tag and technique drives stay. Charge is never by hit count | delegated |
| 8 | Slot order matters. Reordering is free at the rest stop, and on the reward screen while placing a new drive; nowhere else. Slots: 3, then 4 after boss 1, then 5 after boss 2 | delegated |
| 9 | The active drive is on D-pad right, one per player, kept through a tag. Technique is a bonus path, never a requirement | delegated |
| 10 | Coins are earned per player into one shared party purse. A plain shop at the rest stop: buy a drive, level a drive, restore a stock, reroll, reorder. The pact: one top-tier drive for one stock, once per act. The rest stop sits just before each act's boss | delegated |
| 11 | Three tiers: Common, Rare, Epic (Epic was Mythic). Editions and duo drives stay out until the slice is judged | delegated |
| 12 | Keystones: one at the run's start and one after each of bosses 1 and 2, so up to three in force | delegated |
| 13 | Full rows: a new drive replaces one the player chooses, or is skipped. A duplicate levels up (to 3) | delegated |
| 14 | Second Wind: one stock, and the same fight resumes. No character select between fights: fight -> pick of three -> (keystone pick after a boss) -> map -> next fight | delegated |
| 15 | Co-op needs player 2 joined at the camp; P1 on port 1 and P2 on port 2, with the in-place tag | delegated |

Worked out from those (all delegated; each is argued where it is cited):

| # | consequence | where |
|---|---|---|
| 16 | The Heat scale is multiplied by 1.5: marks at 40 / 90 / 165, the cap at 225 (375 with every cap-raiser). The bar's proportions do not change | 3.4 |
| 17 | Pays per Heat are divided by 1.5, so a full meter pays what it did: a coin per 15 Heat, a pip per 150, the flare Heat / 15. Flat Heat amounts are multiplied by 1.5 (Clean Landing +6, Riposte +10, Metronome +5, Breaker +45, Snowball +3 to +60) | 3.5, 8.1 |
| 18 | Insulate's old effect was the rule the owner has now made the default. New effect: the first 20% taken in a fight, and after each cash-out, costs no Heat | 3.3, 8.1 |
| 19 | All In: damage taken costs 2 Heat per 1% (was 3) | 8.3 |
| 20 | Glass Core: checked and left as it is | 8.3 |
| 21 | Echo's repeat is at half strength and scales with level (V2's number). The first revision's hand sums took it at full strength | 12.6, 9.3 |
| 22 | Heat is an `i64` at 2,000,000,000 ticks a point | 12.2 |
| 23 | The flare feeds no Heat, charges nothing and joins no tally | 12.4 |
| 24 | Shop prices: Common 30 (was 40), so the first shop can buy one; a stock 100; actives are not sold | 7.3 |

### 13.2 Open risks

Things only the sim or a playtest can settle. None blocks the build.

1. **Frozen Clockwork may still be too strong.** With Echo at half strength it no longer needs the fewest moves by
   hand (78%, against Firestorm's 75% and Twin Engine's 56%), but it stacks a multiplier on a retrigger. First
   target for `sim.py`; the dials are Shatter's need and Echo's number.
2. **Twin Engine needs the fewest moves by hand** (56%): two Relay volleys do a third of a stock. Relay is not in
   the slice, so this waits, but it is the next thing to measure.
3. **A hit may not read on the meter.** At 1 for 1 on a 225 scale a 10% hit is about 4% of the bar. The meter is
   then not where a lost exchange is felt. The HUD can give a loss a least size; whether that is enough is a
   playtest question.
4. **A strong player may park at Blazing.** Being hit no longer drains Heat faster than hitting fills it. KOs and
   boss breaks still halve it. The reserve is the cooling rule of 3.2.
5. **The 1.5 scale is a delegated call.** It keeps the driveless player where the reviewed design had them (Hot at
   the first KO, under Blazing through the third). If the owner would rather keep 25 / 60 / 110 / 150, a player
   with no drives is Blazing on the second KO and full on the third, and measure 2 cannot be met.
6. **Gain multipliers are worth less than they were.** The driveless player gained 50% from the gentler loss; an
   `xGain` build did not. By hand Sky Pirate is 1.6 times the driveless Heat at the first KO (it was 2.1) and Twin
   Engine 1.3 (was 1.8). Measure 2 decides; the dial is the multipliers.
7. **Laser camping.** Chip fills Heat in full and the foe's replies now cost only what they deal. Stale moves are
   the brake; a projectile coefficient is the reserve.
8. **Delaying a KO to cash out higher.** Bounded by the cap and the 150% line; measure 6.
9. **Technique may still feel required.** Three of the 16 read a technique. The rule (a same-family drive without
   one, at most one technique card in a pick unless the player has used it) is untested on a player who does not
   L-cancel.
10. **The first shop is thin** (about 30 coins by hand). It may read as an empty stop.
11. **Insulate's allowance may be too small to notice** (20 Heat a stock out of about 90).
12. **Unverified engine facts**, any of which can change a rule: the L-cancel hook path; the knockback write's
    timing and the attacker's state on late smash hits; Petey's per-player damage and HP words; whether a shielded
    hit, every recoil and the second Ice Climber reach the take-damage hook as assumed (9.6); that the engine's
    `f32` damage is the same on both peers; `foe_m` against healing; how a fight resumes for Second Wind.
13. **CPU-proxy tuning misleads.** The spreads are properties of the pool, not of the pace; the owner's play sets
    the dials.

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
- Read for the 2026-11-10 finalising: `rust/melee_fighter/src/hooks.rs` (:14, :86, the `Option<fn>` pattern);
  `fighter.rs` 2625-2640 and 2817-2822 (the take-damage hook and the knockback step);
  `ftCo_Damage.rs` 233-262 (`ftCo_Damage_CalcKnockback`); `rust/melee_modes/src/tu/petey.rs` (190, 201, 215,
  1045-1048, 1117); `melee_modhost/src/lib.rs` (676, 684-690, 860); `envoy_data/src/view.rs`; `pool.toml` (Echo's
  number); `tools/rust-port/envoy/sim.py` (its header and bands). Read, not run.
- `rust/melee_fighter/src/tu/ftCo_LandingAir.rs:65` (the L-cancel test), `ftCo_Landing.rs:140`
  (LandingFallSpecial), `ftCo_AppealS.rs:76` (the taunt on D-pad up), `ftcoll.rs` (510, 840: the element at a
  connect), `fighter.rs:2912` (hitlag), `ft_0763.rs:26` (the attack id).
- `rust/melee_types/src/gen.rs` (motion-state ids, `HitCapsule`); `docs/specs/hooks-v1.tsv` (the action hook rows).
- Memory: `envoy-lab-answers-2026-11-09.md`.
