# Envoy drives: V1 (the drive slice) and V2 (the full drive core) (2026-11-10)

Built from `envoy-core-v2.md` (port/envoycore2) section 6. V1 (branch `port/v1`, merged into rust-port after the
owner played it: "seems fine") replaced milestone 1's crests with drives and keystones. V2 (branch `port/v2`, off
rust-port, not merged until the owner has played it) is the full core: the 20 drives and 6 keystones, launching
blasts, elites and bosses with drives, local co-op, the balancing harness and V1's known issues. The owner approved
the note's eight recommendations (memory `envoy-lab-answers-2026-11-09.md`); this note says how they were built.

## What is in

| piece | where |
|---|---|
| pool | `rust/mods/envoy_data/pool.toml` -> `build.rs` -> `pool_gen.rs` (`POOL`, `POOL_LEN`, `POOL_DIGEST`): 20 drives (RED Ember, Nova, Bolt, Spire; GREEN Rush, Slipstream, Tailwind; BLUE Bulwark, Thorns, Leech; YELLOW Comet, Updraft, Feather Storm; PURPLE Frost, Shatter, Hex, Wildfire; WHITE Relay, Echo, Surge) and 6 keystones (Plague Bearer, Echo Oath, Overclock, Twin Soul, Winter Oath, Glass Core) |
| vocabulary | `envoy_data/src/codes.rs`: 6 families, 12 charge sources (hit, taken, landing, jump, time, tag_in, aerial_hit, hit_100, ko, shift, fire, passive), 25 effects (19 drive, 6 keystone), `LAUNCHES` (nova, shockwave, spire), 10 looks, the four words, the seven prices, level scale x1.0 / x1.6 / x2.4, the 20-frame gap, the boss half-strength numbers |
| lint | `envoy_data/src/spec.rs`, `FULL_POOL` on: <= 12 words, <= 2 numbers, no duplicate key / name / rule, a word in the rule is one it gives or reads, every read word has two givers, every family has three drives, a keystone has exactly one price (or own), a drive has a look, a sound and a drive effect, an estimated fire period inside 2-16 s (a drive gated on a word or on an event is exempt), only Tailwind is passive. A test checks that only timed drives launch (Q6) |
| view | `envoy_data/src/view.rs`: the one layout both mods use (512 words: the run, a block per player, a block per match slot, a 12-entry fire log, the run record) |
| sim | `envoy_sim`: drives `[2 players][2 fighters][4]` of (id, level), 3 slots open then 4 after the first boss; charges and cooldowns; the fire queue per set; foe drives; statuses on every match slot; per-player drops; per-player picks; the shared pool; all in the one `#[repr(C)]` state hashed into `gd_hash_word` every frame |
| engine | `melee_modes::tu::envoy`: co-op plan words (27-29) and the second human's pair (`ROSTER2`), `speed` (walk / dash from a stored base), `absorb`, `attacker`, `jumps` / `air_jump`, `blast` + `tick` (the drive blast on the engine's hit path); `petey::damage(amount)`, `petey::carrier`, `petey::HITS`; `drive_draw` draws from any fighter's pass; `VIEW_WORDS` 512; run record version 4 (48 words) |
| host imports (sim) | `gd_envoy_coop` (`MELEE_ENVOY_COOP`), `gd_envoy_speed`, `gd_envoy_absorb`, `gd_envoy_attacker`, `gd_envoy_jumps`, `gd_envoy_air_jump`, `gd_envoy_blast`, `gd_envoy_boss`; `gd_envoy_roster` 2-3 (player 2's pair) |
| UI | `envoy_ui`: per-player orbiters (Twin Soul's bench set on a wider, dimmer ring), the keystone halo, orbs, Haste, foes' orbiters and every slot's statuses (Burning, Chilled, Marked), drops in the owner's colour, every drive's effect by its look, Brawl sounds (`gd_ui_brawl_sfx`); the HUD names an elite's or boss's drives; co-op reward screens (both players pick at once, a row each); the camp shows both pairs. Only the existing kit and the drive models: no new art |
| harness | `tools/rust-port/envoy/sim.py`: seeds x policies (brawler, aerial, tagger, turtle) over a model of the sim's rules; the fire-rate, drive-share and win bands |

## The run (the Short run)

| fight | kind | foe | carries |
|---|---|---|---|
| 1 | duel | 1 CPU, level 3, 1 stock | - |
| 2 | elite (act boss 1) | 1 CPU, level 6, 2 stocks, x1.15 | 1 drive, level 1 |
| 3 | pack | 2 CPUs, levels 5-6 | - |
| 4 | elite (act boss 2) | 1 CPU, level 8, 2 stocks, x1.2 | 2 drives, level 2 |
| 5 | boss | Ganondorf level 9 (or Petey with `MELEE_ENVOY_BOSS=petey`), 3 stocks | 2 drives, level 2 |

Keystones: one at the run's start and one after each of the first two bosses (fights 2 and 4 here), so three a run;
the keystone pick opens the next fight, before its drive pick. Drive slots: 3, then 4 after the first boss. Co-op
adds a CPU to a duel or a pack (CPUs are slots 2-3 then) and a stock to an elite or a boss.

## The loop

- **Charges** (the fighter in play's set; the bench's too with Twin Soul): hits landed (attributed through the
  victim's damage source, `gd_envoy_attacker`, so in co-op the hitting player's drives charge), aerial hits (the
  attacker in AttackAir*), hits on a foe at 100%+, hits taken, landings and jumps (polled: a landing or a
  jump state entered, aerial landings included), time, tag-in, a KO dealt (Wildfire only on a Burning foe), a stage shift, another fire of the
  same set (Echo). Shatter only counts hits on a Chilled foe. Overclock adds 2. Cooldowns: Spire 6 s, Thorns 2 s.
  Hits on Petey come from his own hit counters (`petey::HITS`: the carrier never takes the take-damage path).
- **Fire queue**: per set, a full drive fires on the next frame the set has not fired for 20 frames, slot order.
  Relay's volley fires the set's other drives 6 frames apart. Echo Oath repeats each fire 12 frames later at half
  strength; Echo (the drive) repeats the set's last fire at half strength every 3rd fire.
- **Effects**: Ember (6% and Burning; level 3 splashes), Nova, Comet and Spire (blasts that launch, below), Bolt (8%
  to the nearest; level 3 chains), Rush and Slipstream (Haste), Tailwind (always on: in Haste, hits deal +25% x
  level), Bulwark (an orb absorbs the next hit), Thorns (the attacker takes 5% and is Marked), Leech (heal 5%),
  Updraft (one air jump back, banked until one is used), Feather Storm (three hits of 3%), Frost (Chilled), Shatter
  (10%, the Chill ends), Hex (Marked), Wildfire (15% to every foe), Surge (every drive of the set charged), Relay.
  Words: Burning 2% a second a stack; Chilled walk / dash x0.7; Marked +30% taken; Haste x1.3. On a boss (an elite,
  the final boss, Petey) Chilled is x0.85 and Marked +15%. Drive effects only ever reach the other side.
- **Launching** (Q6: only the drives you time yourself): `gd_envoy_blast(slot, x, y, r, damage, angle, kbg, bkb)`
  arms a hitbox on a capsule slot the owner's fighter is not using, on the engine's hit path as Petey's attacks
  are, for 3 frames (`envoy::tick` keeps it on the capsule and switches it off). It launches with the game's own
  knockback, hits Petey's body and cages, its damage goes through the take-damage hook (so it counts as a hit). The
  owner's teammates are pre-registered as victims: a blast never hits a teammate, whatever the friendly-fire rule.
  Nova: radius 16, Sakurai angle, kbg 70 / bkb 35; Comet: 13, 80 degrees (pops up), 55 / 30; Spire: 9 at the foe,
  straight up, 75 / 40; level 3 widens them by a quarter. No free capsule: the fire falls back to percent.
  `gd_envoy_blast_cap(slot, cap x1000, tag)` puts a cap and a log tag on the blast the slot armed this frame: the
  engine clamps that capsule's damage on each victim to the cap in `ftColl_80076ED8`, after every multiplier and
  right before it joins the victim's damage of the frame (`melee_fighter::hooks::ENVOY_BLAST_LAND`, None in a
  vanilla match). Foe blasts carry the foe fire cap (envoy-balance.md, round 3). `gd_envoy_landed(victim, out)`
  reads what blasts landed on a victim this frame (per capsule: landed, before the cap, owner, capsule, tag, and
  whether the blast's knockback won the frame), for the sim's take-damage hook and `kblog`.
- **Prices**: Fragile defence x1.25, Weak attack x0.8, Bleed 1% per fire, Slow walk / dash x0.85, Lock tag cooldown
  10 s; Glass Core deals and takes x1.6. Vulnerable (statuses last twice as long) is coded, no keystone uses it yet.
- **Foes**: an elite or boss carries 1-2 drives from the foe list (Ember, Nova, Bolt, Frost, Rush, Bulwark, Hex,
  Comet), rolled from the run seed. They charge from the foe's own hits, hits taken, landings and time, fire at the
  players with the same rules, orbit the foe and are named on its card in the HUD.
- **Earning**: KOs drop drives (a duel 1, a pack 1 per two foes, an elite or boss 2), one per player each time, on
  a floor point, in the owner's colour; only the owner takes it (Q8). After the last KO the fight holds up to 10 s
  while a drop is on the floor, then sweeps it up. Fights 2-5 open with three drive cards per player (none the pair
  cannot take; one connects to a word the pair has when it can; `+1 stock` third at pool 1), then the fighter. In
  co-op both players pick in one freeze with their own pads.
- **Co-op** (`MELEE_ENVOY_COOP=1`): P1 + P3 on the CSS are player 1's pair, P2 + P4 player 2's. Pool 5, shared: a
  stock one player loses is lost by both. Each player has their own drives, keystones, picks, drops and tag.
- **Shared knob**: `MELEE_ENVOY_SHARED_DRIVES=1` makes each pair's set 0 the pair's.

## V1's known issues, resolved

| issue | V2 |
|---|---|
| orbiters vanish when slot 0 is KO'd | `drive_draw::draw` runs in one fighter's pass per render, any fighter: the drawer is kept, and when it is not drawn for a whole render (a fighter seen twice first) that fighter takes over |
| Chilled / Haste attribute drift | `envoy::speed(slot, active, bench)` sets walk / dash to `base * m`, the base stored per slot and entity and re-taken whenever the character there changes; the sim writes the product of Haste, Chilled and Slow every live frame |
| statuses under the in-place tag | the same: the swapped-in or respawned fighter is re-based from its own attributes and gets the multiplier on its first frame; statuses live per match slot in the sim, not on the fighter |
| Bulwark lets knockback through | decided with the note's intent ("an orb absorbs the next hit"): the orb takes the whole hit. The pre hook skips the percent and `gd_envoy_absorb` zeroes the hit's knockback, which `ftCo_8008EC90` takes as no reaction (no launch, no flinch; hitlag stays). The rule text now reads "absorbs the next hit" |

## Tuning (sim.py, the first tune)

The first harness pass found the full build too loud (0.60 s between fires with four drives at levels 2-3 and three
keystones) and two drives outside the 2-16 s band alone (Updraft 18 s; Spire 17 s, which is gated on a foe at 100%
and now counts as event-gated). Changed: Ember 4 -> 5 hits, Frost 4 -> 5, Hex 5 -> 6, Rush 5 -> 6, Leech 5 -> 6,
Updraft 4 -> 3 aerial hits. After, 200 seeds:

| policy | early (3 drives, level 1) s/fire | full build s/fire | drive share of damage | Short run cleared |
|---|---|---|---|---|
| brawler | 2.14 | 0.62 | 0.55 | 0.73 |
| aerial | 2.00 | 0.64 | 0.59 | 0.72 |
| tagger | 2.14 | 0.69 | 0.61 | 0.54 |
| turtle | 3.16 | 1.13 | 0.61 | 0.28 |

Level-1 periods alone (brawler pace): Ember 4.8 s, Rush 4.6, Thorns 4.7, Frost 4.8, Leech 5.8, Hex 5.8, Comet 7.1,
Bolt 8.2, Feather Storm 10.8, Nova 12.9, Updraft 13.1, Bulwark 15.0. Bands: early 2.0-3.6 s, full 0.6-1.4 s (the
note's "about one a second", the low end open while it starts loud, Q5), win 0.45-0.9 and drive share 0.25-0.65 for
the standard policy. The full build sits at the loud end on purpose; the owner's play sets the dial.

## Checks

`tools/rust-port/envoy/v2_drives.sh`: the lint and its tests, the harness bands, then six synctests at depths 4 and
7 over 3600 frames, each 0 mismatches: a pad run (time / landing / jump / tag drives, a blast, an echo, a card, a
tag, the elite's drive), a CPU run (hit drives, Plague Bearer, Winter Oath, Shatter, a drop), Twin Soul (both sets
fire), the shared knob, co-op (both players fire and pick) and the Petey boss.

`tools/rust-port/envoy/camp_run.sh`: the whole flow from the camp door with no `MELEE_ENVOY_ARM` (title, 1-P,
Envoy row, New run, the CSS, then Start / A through the results, the camp's Continue, the CSS and the cards, the
player's fighters level-9 CPUs), asserting fights 1, 2 and 3 are rewritten Envoy matches, 1 and 2 won, no plain VS
match and no disarm in between.

## Fixed after the first playtest

- **A won fight read as RUN OVER.** The collect hold (a drop still on the floor after the last KO) waited inside the
  match, but the retail game end that follows the last KO tears the players down and their stock reads go to 0: the
  sim saw "no stocks" and ended the run, the camp showed RUN OVER, and its Back row disarmed the run onto a plain VS
  CSS. The win now latches when the last foe falls (pool kept from that moment) and stands if the match ends inside
  the hold. After a fight the camp ignores B (a B mashed through the results no longer leaves the run).
- **Co-op player 2 read player 1's controller.** `match_begin` set player 2's `slot` (the 1-based port, `getPort`)
  to 1, port 1. It is 2 now; with one controller player 2 stands idle and the HUD says "P2: CONNECT A SECOND
  CONTROLLER" (`gd_ui_pad_present`).

## Known limits

- Hits on Petey charge drives but Marked does not apply to him (his damage is not the take-damage path); Chilled
  does nothing to him (he does not walk).
- A blast is a hitbox on the owner's fighter: when it connects the owner gets the hit's hitlag, and its damage
  counts as a hit landed (it charges hit drives).
- Attribution: a hit with no fighter or item source (a stage hazard) credits player 1 in solo and nobody in co-op.
- The team-attack toggle is not built; drive effects ignore it either way.
