# The full game: Envoy 1.0 on the Rust port (plan, 2026-11-09)

Planning only. Nothing here was built or run. Branch `port/fullplan`, worktree `C:/melee-wt/fullplan`, off `pub/rust-port`
(`e0e53381c`). Written by Fable at the owner's request; the research papers were re-ingested from the web (section 10).
Tags: **[R]** read from source or a note (cited), **[I]** inferred, **[J]** judgement, **[U]** unverified, **[V]** source
verified today, **[D]** an owner decision already taken (memory `envoy-lab-answers-2026-11-09.md`: E1-E30, S1-S10, P1-P9,
O1-O5, L1-L7 and the eight redesign approvals).

What this plan builds on: `envoy-plan.md` (port/envoyplan, the original plan), `envoy-redesign.md` and `envoy-1p.md`
(port/envoyloot, port/envoydoor: crests, the door, the camp), `lab-theorycraft.md` and `lab-ui-spec.md` (rust-port),
`petey-boss.md` / `petey-behavior.md` / `petey-extract.md` (port/petey), and the `_research/` notes (section 10.3). What is
**already built** is listed in section 6.1 so the milestones start from fact, not from the earlier plans' intentions.

Rules that bound everything below (memory): wasm mods, never the old script host; the C build is frozen; **no old GD art, a
fresh look, designed with the owner one question at a time**; one playtest per milestone (E26); Opus for UI and advanced
work, Sonnet for plain work; nothing disc-derived is committed and no disc path is printed; everything in the sim is
deterministic and rolls back.

---

## 1. The vision

Envoy is a one- or two-player co-op party roguelite built on Melee's own fighting: you bring a pair of fighters, tag between
them mid-fight with a one-second cinematic, and take a seeded run of fights through arenas that shift under you every
thirty seconds, toward a boss. Between fights you pick **Crests**: one rule, one number, worn by one fighter of the pair,
firing at the loud, rare moments the run already makes (tag in, tag out, a stage shift, a stock lost from the shared pool,
the opening, a boss), so a build is something you can feel and name, not a wall of modifiers. Two players share one stock
pool and pick their own crests with their own pads, locally or over the port's rollback netplay, with the sim bit-identical
on both machines. Bosses are real enemies (Petey first, the Hands as the finale) with their own HP, phases and rules,
never playable. Everything the player sees outside the retail fight is ours and new: the door in the 1-P menu, the camp,
the roster, the lanterns on the arena, the rest stage. The LAB is the other half of the same product: the place to learn
the fighters, and the harness that keeps the run's numbers honest.

---

## 2. The player journey

Each step says what exists today **[R]** and what 1.0 adds. "Today" means `pub/port/envoydoor` + `pub/port/petey` unless
stated.

| step | today [R] | 1.0 |
|---|---|---|
| **Boot** | `melee play` runs the retail opening, title and main menu from the user's disc (`retail_flow.rs`; STATUS.md "runs the retail menus, matches, 1P modes"). Mods load from `MELEE_WASM_MODS`; `envoy_sim` and `envoy_ui` are inert until armed | Launched from GD's Qt launcher (memory: stay in GD's launcher) with the Envoy mods mounted by default. Boot unchanged. A **wordmark** on the title's mod strip (O1: wordmark first, designed with the owner) |
| **Menus and the door** | Main menu > **1-P > Envoy**: a native cursor row past the last retail entry, drawn by `envoy_ui` as a kit panel (`envoy::door_1p`); the browser's MODES catalog row `envoy` (group Solo) opens the same camp. VS stays retail when not armed (verified: a 2-minute retail time match with both mods loaded) | Same two doors. The row gets its own look once the retail cursor can light it (today it stays on Training while Envoy is hovered: placeholder) |
| **The camp** | `envoy_ui` scene `envoy.camp`: Continue / New run / Party / Abandon (asks twice) / Back; after a fight: Continue / Party / Abandon / Leave, or RUN CLEAR / RUN OVER. Replaces the CSS only while armed and a fight has happened (`gd_ui_scene_replace_when`, cond 1) | The camp is the hub: run record ("act 2, fight 5/12, pool 3, Fox + Marth"), the **roster** (section 4.1), run history and records (section 2's "meta"), an online row ("New run (online)", "Continue (online)"), the result card naming what ended the run (Juul [V]: failure implicates the player, so name the cause) |
| **Party** | S1: P1's pick is the main, the P2 slot (HMN or CPU, any character) is the partner; Zelda / Sheik / ICs fall back to Fox / Marth [R `envoy-1p.md`] | A2: the **roster screen** replacing the CSS while armed, 23 taggable characters, 2 players side by side, lockstep online (D: S1 now, S2 next). "Recruit" (the beaten opponent joins) as a run **variant**, not the default (D) |
| **A run** | 5 fights on a fixed schedule, pool of 4, stage shift every 30 s from fight 2, reward hold at the start of fights 2-5 | Section 3: **3 acts x (3 fights + a boss) = 12 encounters**, about 20-25 minutes; encounter kinds (duel, pack, elite, shift room, boss, rest); act stage sets; a run seed; a short-run option that is today's 5 fights |
| **Fights** | Teams on (you vs the CPUs), items off, no timer; tag on D-pad down (60-frame cinematic, 5 s cooldown, air OK); the pool carries over; `match_begin` rewrites `StartMeleeData` from the plan | Same core. Crests fire at moments; the opponent card at the ready moment (two lines at most); the HUD loses the TAG READY / STAGE SHIFT lines once the world carries them (D Q8) |
| **Rewards and crests** | 3 of 6 stat upgrades with before -> after rows, 20 s, offer 0 on timeout (E8, E22) | Crests (section 4.2): three offers per player (connect / new / spread-or-hone; `+1 stock` when the pool is at 1), the card names the wearer (D Q4), both players pick with their own pads inside one freeze; then **pickups in the arena** after the last KO (D3), then the **rest stage** (D2) |
| **Bosses** | Fight 5: level-9 Ganondorf stand-in, or Petey on FD with `MELEE_ENVOY_BOSS=petey` (carrier-CPU trick, HP separate from %, super armour, two cages, phase 2 at 50% [P4], HP x(1 + 0.6 (players - 1)) [P3], partner caged, tags and shifts refused) | Section 4.5: one boss per act. Act 1 an **elite** (a retail fighter under boss rules: scale, ratios, a crest set, HP bar); act 2 **Petey** (P1 party rescue, P2 both freed, P5 original arena later, P7 stand-in effects, P8 enemy only); act 3 **Master Hand**, Crazy Hand joining at half HP in 2-player (P6 says the Hands are the fallback; here they are also the finale, question Q2). Heavy crests are offered only at bosses (D Q5) |
| **The run's end** | RUN CLEAR / RUN OVER shown in the camp; the record is deleted | The result card: fights won, crests worn, what ended it, time; **New Game+** lap (heavier foes, a third heavy crest, same party) or a new run; the profile records the run |
| **Meta-progression** | none | Section 4.8: a **profile** per player (run history, best runs per pair, technique counters), **crest discovery** (a crest you have worn is shown in the camp's crest book with its text; nothing is gated in 1.0, question Q7), NG+ laps. No currency, no breeding, no companion in 1.0 (E21 keeps the companion for later) |
| **Saves** | 16-word run record saved by the host to `melee_data/envoy/run.bin` at RUN LIVE boundaries, digest-checked, deleted on clear / over / abandon; offline only | The run record grows (act, encounter graph path, crests, heavy crests, boss state flags; ~300 B) and stays boundary-only (question Q8); the profile is a second file; online both peers hold the same record bytes (the `envoy=` token). A snapshot of a mid-fight state never leaves the machine (disc-derived) |

---

## 3. Run structure

### 3.1 Length and shape

**Default run (1.0): 3 acts, each 3 fights then a boss; 12 encounters; a rest stage before each boss (once D2 lands).**
Reasoning: today's 5-fight run plays in 6-10 minutes [R `envoy-redesign.md` 3.6]; a crest build needs about 8 picks and a
few hones to come together (3.4 "a staircase of at most 8 picks + 6 hones"); 12 encounters give 9 offers + 3 heavy-crest
screens, which is the whole staircase once. At ~1.5 min a fight, 20 s an offer, 3 min a boss, a run is **20-25 minutes**
[I], the length of a Hades run and shorter than a Slay the Spire run (design observation, not a source). The **Short run**
(today's 5 fights, one boss) stays as a camp option for a quick session and as the harness's cheap case.

| act | fights | stage set (retail only, E18) | boss | what the act teaches |
|---|---|---|---|---|
| 1 | 3 (duel, duel or pack, pack) | BF, YS, FoD | elite (a retail fighter under boss rules) | the tag, the shift, the first crests |
| 2 | 3 (pack, elite, shift room) | DL, FD, + one more audited stage (Q6) | Petey | the pool under pressure, synergy words |
| 3 | 3 (pack, elite, shift room) | all five + the audited extras | Master Hand (Crazy Hand joins at half HP with 2 players) | the build at full size, heavy crests |

Fight 1 is always a 1v1 duel with no shift (the tag and the arena are learnt one at a time; shifts start at fight 2 as
today [R]).

### 3.2 Encounter kinds

| kind | CPUs | stocks | rules | needs beyond today |
|---|---|---|---|---|
| **duel** | 1 | 2 | one strong CPU | nothing (fight 2's shape) |
| **pack** | 2-3 | 1 each | several weaker CPUs; stage shifts every 30 s | nothing (fights 3-4's shape); the 4-player results-screen text pool fix is in [R `envoy-1p.md`] |
| **elite** | 1 | 2-3 | a CPU with scale x1.1-1.25, ratios, and **ALWAYS/OPENER crests** shown on its card; a Geno attach entry later for a changed special [J] | foe crests (C5); Geno attach for CPUs (later) |
| **shift room** | 2 | 2 each | the arena shifts every **15 s** instead of 30; SHIFT crests shine | a per-encounter shift period in the plan (S) |
| **boss** | the carrier (Petey) or the Hand kinds | HP | section 4.5 | the boss framework (B1) |
| **rest** | 0 | - | the rest stage: pickups, exits, the pool as pedestals | `gd_match_setup` (D2) |
| **spar** (2-player only, later) | 0 | - | the two players duel for a crest, no stock loss, first to 100% [J] (E5: teammate duels OK) | a custom end condition in the sim mod (S-M); not in 1.0 unless asked (Q5) |

Waves ("CPUs arrive in sequence") are **not** a kind: nothing is spawned or freed mid-match (the TagFighter lesson [R
`_research/tagfighter-insights`]: all fighters exist from frame 1; mid-match `Fighter_Create` produced collapsed models),
and the engine can sleep/wake only sub-fighters today [R `envoy-redesign.md` 5.2].

### 3.3 Stage order and the shift schedule

- The five legal stages today: FD, BF, YS, DL, FoD (`st_retail_supported`; both source and destination must be in the list,
  "unusual/streaming stages deliberately fail closed") [R `stage_hop.rs`]. Act sets above use only these until the audit in
  Q6 adds hazard-light stages (candidates: Kongo Jungle 64, Pokemon Stadium **frozen** (the Frozen PS word already exists in
  the `.slp` scenario converter [R slpcheck.md]), Yoshi's Island 64, Final Destination variants are not needed).
- Shift period per encounter: 30 s default, 15 s in shift rooms, none in fight 1 and in boss fights (the engine refuses a
  shift during Petey [R]). Stage Morph's rules are the model [V: SmashWiki]: trigger on a logic frame, a 3 s warning (the
  cover's colour creeping in from the edges, D Q8), fighters kept in place when standing on the new floor, otherwise placed
  on the nearest floor (kinder than Ultimate's immediate KO), stage-bound items despawn (items are off anyway).
- The order inside an act is seeded (`seed_for(seed, act, encounter)`), never repeating the stage you are on, and the first
  shift of a fight never goes to the stage the fight started on.
- **Blocker to fix first:** each switch leaks ~0.7 MB of heap 0 [R `envoy-1p.md`]; a 12-encounter run with shift rooms is
  ~50-70 switches, 35-50 MB. Milestone 0.

### 3.4 Difficulty curves and target bands

Two curves, both set at fight boundaries only (never inside a fight; section 10 notes this is our rule, not Hunicke's).

**Foe level curve** (the retail CPU level, the only difficulty lever that exists today [R `SCHEDULE`]):

| encounter | 1 | 2 | 3 | B1 | 4 | 5 | 6 | B2 | 7 | 8 | 9 | B3 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| CPU level | 3 | 4-5 | 5 | elite 6 | 5-6 | 6 | 6-7 | Petey L from the budget | 7 | 7-8 | 8 | Hands |

**Foe budget** (from `envoy-redesign.md` 3.5, kept): budget = party weight x (1 + min(0.25, 0.05 x (encounter - 1))) x
role (elite 1.15, boss 1.15, last boss 1.30); a foe spends it on ALWAYS and OPENER crests first; the +25% cap is the C
build's one balance result that held (win rate 50-60% through loop 2, 43 fights [R synergy 5.3, strength 3.3]). With two
players the budget is per party (both players' weights summed and halved [J]) and CPU **count** rises before level does
(a pack gets its third CPU; a duel becomes a pack), because count is what the shared pool feels.

**Target bands** (the harness of 4.9 measures them; CPU-tuned first, then corrected from the owner's recorded play, E25):

| metric | band | source |
|---|---|---|
| CPU-proxy win rate, fights 1-9 | 0.40-0.60, rising slightly with the act [J: a run should get harder, not easier] | Rupp et al. 2024 [V]: simulation-balanced content is perceived as balanced for most scenarios; the band is **ours**, their target is equal win rates (0.5); the C build's 50-60% [R] |
| CPU-proxy win rate, bosses | 0.30-0.50 | [J] |
| run clear rate, CPU proxy | 0.05-0.15 for a full run (product of the above) | [I] |
| pool left entering each boss | median 1-2 | [J] |
| tags per fight | 2-4 | [J] |
| fires per family per fight | RELAY 2-5, SHIFT 1-3, POOL 0-2, OPENER 1, BENCH 1-3 | redesign 3.6 [J] |
| Fresh-window damage uplift | +25% to +60% inside the window | synergy proposal 2 raised for a short window [R] |
| offer quality | dead 0 (by rule); coin flip < 25%; tension > 35% | synergy 4's definitions [R] |
| pair completion by B2 | > 60% (connect policy), > 35% (random) | synergy 3 [R] |
| run length | 20-25 min full, 6-10 min short | [I] |

Two personas run the harness (the retail level-9 and a recorded-input or LAB-reaction dummy that performs technique), so
technique-gated numbers are not scored by a proxy that never performs technique (Holmgård et al. [V]; the C lane's blind
spot: one hit-confirmed L-cancel per 450 s [R synergy 7]).

---

## 4. Systems

### 4.1 The roster

- **23 taggable characters**: the 26 retail kinds minus Ice Climbers, Zelda and Sheik (`envoy::taggable`), because the tag
  rides the Zelda/Sheik transform path and the sub-fighter slot [R `tu/envoy.rs`, `tag_swap.rs`]. Non-taggable picks fall
  back to Fox / Marth today; the roster screen simply does not offer them.
- 1.0 opens the whole roster from the first run (no fighter unlocks; the retail unlock bits are not read [J], question Q7).
- **Roster screen (A2)**: each player picks main then partner on a grid of 23 names (portraits when a texture path for the
  disc's CSS portraits is confirmed [U]); 2 players side by side; lockstep online like `--np-css`. Opus, designed with the
  owner one step at a time (the owner's second Envoy screen after the camp).
- **Local 2-player mapping**: P1 + P3 are player 1's pair, P2 + P4 player 2's (`match_begin` reads only P1/P2 today: S)
  [R `envoy-redesign.md` 1.4].
- Later (not 1.0): Zelda/Sheik as a pair (their transform *is* a tag), ICs (two GObjs), the "Recruit" variant.

### 4.2 Crests (the reward object)

As approved in `envoy-redesign.md` section 2 (D Q3-Q5); restated only where 1.0 extends it.

- Fields: `family` (ALWAYS, RELAY, BENCH, SHIFT, POOL, OPENER, HEAVY), `rule` (one sentence, <= 12 words, <= 2 numbers),
  `number` (one i32 x1000, the one thing a hone changes, +50% once), `weight` (1-3, the foe budget unit, never shown),
  `gives` / `reads` (0-1 word each from **Fresh, Benched, Marked, Burning**), `wearer`, `price` (HEAVY: one of the seven
  Slow, Low, Fragile, Weak, Vulnerable, Hurried, Bleed).
- Pool size: the redesign's ~34 for the first milestone; **1.0 pool ~48**: 7 per family (RELAY, BENCH, SHIFT, POOL, OPENER),
  5 ALWAYS, 8 HEAVY. The lint (`envoy_lint`, build-time) enforces the readability limits the owner kept (E19 as limits
  only): one trigger, one effect, <= 12 words, every `reads` word has >= 2 givers, every family >= 4 members, no duplicate
  rule text. The two hit-reading crests wait for the hit view (H1).
- Slots: 3 per fighter, 6 per player, at most 2 heavy per run (3 on an NG+ lap). No rarity, tier, affix, merge, bag,
  strength percent (the C build's measured failures: 17% dead and 44% coin-flip drive screens vs 48% tension on the one
  trade-off screen [R synergy 4]).
- Growth comes from more crests and heavier foes, never bigger numbers: the tier curve that made late builds x3.5 is gone
  [R synergy 0.4].
- Offers: slot A connects (a `reads`/`gives` match with something held; else the family of the fighter with fewer crests),
  B new (a family the party lacks), C spread or hone, alternating; `+1 shared stock` replaces C when the pool is at 1
  (the comeback valve at a boundary). Never a crest a fighter wears, never the same crest on both fighters, never two
  offers from one family: dead cards are impossible by construction.
- Synergy is a word written twice ("Marked" on the giver and the reader), cross-fighter synergy is the party (Fox's
  tag-out Marks, Marth's Fresh crest hits the Marked foe), the theme line per fighter counts words, a four-entry glossary.
- Data: `rust/mods/envoy_data/pool.toml` -> `build.rs` -> `pool_gen.rs`, `include!`d by both `envoy_sim` and `envoy_ui`;
  an FNV digest of the pool is in the sim hash and the run record, so peers with different pools refuse each other and a
  record from an older pool is dropped with a notice.

### 4.3 The shared pool, tag, Second Wind

- **Pool**: 4 stocks shared by the pair across the run [R], carried fight to fight; `+1 stock` as an offer kind; POOL crests
  key on the pool falling and on the last stock. Two players: the same pool (E4). Question Q4 asks whether co-op starts at 5.
- **Tag** (E10-E15, built): D-pad down when actionable, ground or air, 5 s cooldown, 60-frame frozen cinematic (D-pad down
  skips its *presentation* after 6 frames; the freeze keeps its length, E14), each fighter keeps its own percent, the new
  fighter appears in place; no assists or stock donation in v1 (E15). The benched fighter is the sleeping sub-fighter.
  1.0 adds: `gd_fighter_set_intangible` for the RELAY "20 more frames intangible" crest; the bench handle
  (`gd_tag_partner_handle`) so BENCH crests can heal the bench; refusal rules unchanged (captured, thrown, dead,
  respawning, during another transition).
- **Second Wind** (E20 in): a run resource, once per fight, rewinding the fight 3 s for everyone; one heavy crest doubles it.
  Offline it is the LAB ring's rewind under a sim command (`gd_match_rewind(frames)`); online it needs the host to apply
  the rewind at a frame all peers have confirmed (the online LAB already applies rewind on both peers with 0 desyncs, but
  in lockstep, not under rollback) [R lab-ui-spec status]. **1.0 ships Second Wind offline; online Second Wind is
  post-1.0 unless the probe in milestone 9 is cheap** (question Q9).

### 4.4 Transitions (one mechanism)

Built [R `tag_swap.rs`]: REQUESTED -> FREEZE (retail pause mask row 7) -> SWAP -> RELEASE; kinds TAG 1 (60 frames),
STAGE 2 (90 frames, switch at the midpoint), HOLD 3 (released by the mod: rewards). The UI's cinematic clock is the sim's
frames-left, so both peers see the reveal on the same sim frame. 1.0 keeps this and adds covers beyond wipe and flash only
through the effects system (section 5.4; E17 wipe and flash first, E27 covers are the first gallery round).

### 4.5 The boss framework (generalised from Petey)

What Petey built that generalises [R `petey.rs`, survey]: the **carrier-CPU** trick (the fight's one CPU slot is `cpu_kind`
0, invisible, intangible; the boss's attacks are its capsule hitbox slots on the engine hit path; a win is the carrier's
bottom-blast KO so Envoy's win check runs), the `BOSS_DRAW` render hook, the `P_BOSS` plan word, the
`arm / disarm / tick / view / draw` interface, hit latching per hitbox lifetime and per item, the partner-caged gate in
`tag_swap`, the stage-shift refusal, a 12-word read-only HUD view, statics in the snapshot with an own LCG, animation
chosen from sim state and the match frame only.

What is Petey-specific and must become data or a trait: the spec tables (`L_TABLE`, 12 motions, 15 statuses, cage HP,
phase thresholds, rules 0-30), `petey_model.rs` (reads the owner's local Brawl files at runtime, never committed, capsules
otherwise), `BOSS_DRAW` as a single slot, `P_BOSS` as a bool, the `MELEE_ENVOY_BOSS` env switch, the FD arena constant,
`gd_ui_petey`.

**B1, the framework:**

```
melee_modes::tu::boss
  trait Boss { arm(ctx), disarm(), tick(), view(&mut [i32; 16]), draw(), allows_tag() -> bool, allows_shift() -> bool }
  registry: boss id (u8) in the plan word P_BOSS; dispatch for tick / draw / view; BOSS_DRAW becomes a dispatch
  common: HP (separate from %), phases (thresholds, a phase-change beat), objectives (0-2 sub-bars: cages, shields),
          the carrier slot, a camera box, the arena (stage kind + x clamp), the player-count scale x(1 + 0.6 (n - 1)) [P3],
          the ready card (name, one rule line), the result (RUN CLEAR / next act)
  gd_ui_boss(out, 16): the generic HUD view (HP, phase, up to two sub-bars, a flag word, the id)
```

**Three bosses in 1.0** (question Q2):

| boss | act | body | HP and rules | status |
|---|---|---|---|---|
| **Elite** | 1 | a retail fighter kind chosen by the seed, scale x1.25, ratios, super armour off, 3 stocks, a crest set shown on the card | retail stock rules; the HP bar shows stocks x percent [J] | needs B1's common HUD and the elite crests (C5); S-M on top of B1 |
| **Petey** | 2 | the clean-room behaviour (petey-behavior.md): rules 0-30, waits, basket swings f70-114/160, jump 20 + T + 90 + 71, Angry at 50% and 25%; the owner's Brawl files draw the real model when present, capsules otherwise | HP 300 / cage 70 **stand-ins** (petey-extract.md: no field confirmed as HP); cage break or HP drained wins; party rescue (P1), both freed (P2) | built on port/petey; HP numbers open; **the model depends on the player's own Brawl files** (risk R3, question Q3) |
| **Master Hand (+ Crazy Hand)** | 3 | the retail boss fighter kinds (`Ft_Kind_MasterH` 0x1B, `Ft_Kind_CrezyH` 0x1C) with their own motion tables and boss AI; stamina-style HP (`Player_SetOtherStamina`, `Player_GetRemainingHP`); Crazy Hand's half-HP join is gated in retail to Classic difficulty >= 2 [R `_research/boss-fights.md`] | HP from the Hand's difficulty attribute (300 is the setup value, not the live one [R]); in Envoy: Master alone for 1 player, Crazy joins at half HP with 2 players [J]; the FD `0xB0` backdrop phases follow average Hand HP | the Hands are retail data on the player's own Melee disc, so they always draw. **M-L**: the 1P controller `fn_8017C1A4` owns the Hand beats; a VS-flow boss needs the stamina flag, the entry/death beats and the termination replicated in the framework (boss-fights.md section 3: "M: native mode/controller state and a once-only trigger") |

Later bosses (post-1.0, from the porky branch's plan): Porky (needs a scrolling arena), Galleom/Duon, Tabuu last; each
under the same clean-room method (a reader agent writes a spec, an implementer never sees the code) [D: Petey method].

### 4.6 Arenas, pickups, the rest stage

In the approved order (D Q6): **cards now -> D3 arena pickups -> D2 rest stage; D1 (one long match) never unless D2 feels
like loading screens.**

- **D3 pickups**: after the last KO a 15 s collect hold on the arena; three zones per player (a sim-side rectangle with a
  payload), rolled with the offers from the seed so both peers agree; standing in one for 30 frames and pressing A takes it
  and **the standing fighter wears it** (the diegetic answer to Q4, D); drawn by `envoy_ui` through a new `gd_ui_project`
  (camera read-only) as a **lantern** (D Q7: lanterns first, standards second; an SDF glyph with an additive core and a
  soft occlusion test; the `envoy_drives` bodies from `make_drives.py` as the lantern's core, re-generated per family,
  rings as the hone mark and the crown as the heavy mark; user OK). No items, nothing new in the rollback set.
- **D2 rest stage**: a match with no foes between fights (needs `gd_match_setup` to chain matches without results / CSS /
  SSS, plan item 5): rewards as lanterns at fixed points, the route as **exits** you stand in (each marked by its reward
  kind: Hades' door symbol as an idea), the pool as lit pedestals, the partner recruit (variant). Candidate stage: a legal
  retail stage dressed by our own geometry (the owner's Blender parametric pipeline), or Melee's All-Star **rest area**
  if `st_retail_supported` can admit it [U, audit in Q6].
- What stays on the HUD: stocks and percent (exact numbers), per Llanos and Jørgensen [V] and Peacocke et al. [V] (health
  reads best on a HUD; counts can live in the world); the theme line in fights until the owner has seen it both ways.
  Iacovides et al. [V] found HUD removal raised immersion for **experts** and did little for novices (10 experts, 14
  novices, Battlefield 3), so the world-carried lines are the default for the lines that are redundant (TAG READY, SHIFT
  IN), never for the numbers.

### 4.7 Co-op and online

- **Local co-op**: two humans, one team, one pool, P1+P3 / P2+P4 pairs (S). Both pick in one freeze with their own pads;
  the UI shows the other cursor. The `+1 stock` valve and the pool-at-1 crests are the comeback levers; no friendly fire
  (teams on; team attack off [J], question Q11).
- **Online** (2 peers; 3-4 later, E3): the camp's online rows hand the room flow an `envoy=` token (armed, seed, record
  digest, pool digest), host-authoritative like `np_rules`; the wasm mods are already in the handshake identity
  (`play_net::mods_identity`); `gd_hash_word` is already mixed into the compared session hash [R survey 4]. Rules: no
  mid-run joins (E6); on a disconnect the survivors continue and the leaver rejoins at the next fight (E7); a desync ends
  the encounter and the run falls back to its last boundary record; a run can always be continued solo offline from its
  record. Spectators via `--np-spectate` as today.
- **What is unproven**: CPUs under **two-peer** rollback. The survey found CPU seats wired in the scene string (`--cpu`,
  "CPU seats on ports 3 and 4, at most two") but no documented 2-peer synctest or loopback proof; what exists is
  single-process synctest with CPUs (envoy1p depth 4 over 2400 frames, 0 mismatches; Petey depth 4 over 8143 passes;
  stage switch `play_pair.sh` 398 rollbacks 0 desyncs, humans only). Memory records E2 as "moot, since CPUs passed under
  rollback"; **that pass is not in the notes I can cite**, so milestone 0 runs the two-peer probe and writes it down.
- Items stay off in 1.0 (no rollback proof for items either; pickups are zones).

### 4.8 Meta-progression and saves

- **Profile** (`melee_data/envoy/profile.bin`, per player, never agreed online): run history (seed, pair, result, acts
  cleared, crests worn, time), best runs per pair and overall, technique counters (L-cancel, wavedash, perfect shield,
  tech rates per minute: the player model the C build never collected; Yannakakis and Togelius [V]), the **crest book**
  (every crest ever worn, with its text and the fights it fired in), settings. Nothing gates play in 1.0 (Q7).
- **NG+**: beating act 3 offers a lap: same pair and crests, foes gain count and weight, a third heavy slot, elite CPUs
  carry RELAY crests too (they still never tag) [J].
- **Run record**: boundary-only saves (today's host-side write, widened); a run that references a missing crest or a
  changed pool is dropped with a notice; the record never depends on the profile.
- Melee's memory-card save is untouched (retail unlocks and records stay retail).

### 4.9 The LAB's role

1. **The balance harness host.** `melee play --match` headless already runs a whole run with pad scripts and a CPU player
   (`MELEE_ENVOY_TEST_P1CPU=1`, `MELEE_ENVOY_START_FIGHT=N`) at ~0.2-0.5 ms a frame [R perf.md]; the LAB's `LabRun`
   supplies the second persona (recorded inputs via `gd_lab_rec_*` / `gd_lab_play`, reactions DI / tech / OoS, CPU
   behaviours) once the harness can run it headless (the LAB imports are offline-only, which the harness is). `sim.py`
   (Sonnet) runs seeds x policies x personas, 8 jobs at most (memory: PC memory limits), and writes the report attached
   to every balance hand-back.
2. **A Crest lab preset (post-M1, S-M):** load an Envoy pair and build into the LAB against a dummy, so a player can feel a
   crest's window with the frame meter; the LAB's `lab.rs` gains a "match setup from an Envoy record" entry when
   `gd_match_setup` lands.
3. **The house style.** The LAB mod's look is the style Envoy's screens extend (no old GD art); the kit nodes, SDF fonts
   (Inter, JetBrains Mono) and theme tokens are shared.
4. **Replay takeover of a run** (theorycraft section 1): `.mrpl` of an Envoy fight opened in the LAB, take over a port,
   auto-rewind: post-1.0.

---

## 5. Content list for 1.0

Under the owner's rules: design with the owner one question at a time; a fresh look; no old GD art; nothing from a disc
is shipped; (a) what we build ourselves first, (b) what needs a person decided after it is played (E29).

| content | count | source | state |
|---|---|---|---|
| **fighters** | 23 taggable retail kinds | the player's own Melee disc | built; Zelda/Sheik/ICs post-1.0 |
| **bosses** | 3: elite (retail fighter), Petey, Master Hand (+ Crazy) | retail data for the Hands and the elite; Petey's behaviour is ours (clean room), its model is the player's own Brawl files, else our stand-in | Petey built (numbers open); Hands and elite M-L |
| **stages** | 5 legal + 2-3 audited (Q6); the rest stage dressing | retail DATs; our geometry for the rest stage (Blender parametric pipeline) | 5 built |
| **crests** | ~48 (5 ALWAYS, 7 x RELAY / BENCH / SHIFT / POOL / OPENER, 8 HEAVY), 7 prices, 4 words | `pool.toml`, ours | ~34 designed on paper |
| **encounter kinds** | duel, pack, elite, shift room, boss, rest (+ spar later) | ours | duel/pack built as fights 1-4 |
| **screens** | door row, camp, roster, reward cards, opponent card, boss ready card, result card, run history, crest book, glossary, online rows | `envoy_ui`, kit nodes; Opus with the owner | door, camp, cards built as placeholders |
| **HUD** | fight n/12 + act, pool, theme line per fighter, boss HP + sub-bars, tag ring at the feet (projected), shift creep | `envoy_ui` | placeholder text today |
| **cinematics** | tag (60 f), stage cover (90 f: wipe, flash; dissolve and displacement once `gd_fx_*` exists), boss entry and defeat beats, run clear | presentation scenes reusing the disc's models and camera **at runtime, never shipped**; no story cutscenes, no voice (E30) | wipe and flash built |
| **effects** | covers (4), statuses (4 words), reward moment, heavy-crest halo, boss telegraphs (stand-in, P7), KO / failure | the `melee_gx` effects system, composable primitives, one core technique per set, the contact-sheet gallery, 3 candidates per round (E28) | not started |
| **pickup bodies and icons** | 7 family silhouettes + hone ring + heavy crown | `make_drives.py` re-run for the families (user OK), lanterns first | the generator exists (C repo tooling) |
| **art that needs a person (b)** | wordmark/logo (O1: designed with the owner first), portraits for the roster and the tag card, Petey's stand-in body if he ships without Brawl files, a rest-stage hero prop | commission or the owner, decided after milestone 3 is played (E29) | open |
| **audio** | retail stage music per arena (the destination's music on a shift, Q10), retail Hands music for act 3, stand-in SFX for Petey (P7), kit UI sounds | retail at runtime; our UI sounds | music does not change on a switch today [R stage_hop] |
| **text** | crest texts (<= 12 words), prices (7), glossary (4), camp and card strings, a 6-line how-to-play | ours; English first | partial |

---

## 6. Tech dependencies on the port

### 6.1 What is built (facts the milestones start from) [R survey]

- `envoy_sim` 578 lines, `envoy_ui` 682, `tu/envoy.rs` 265, `tag_swap.rs` 276, `stage_hop.rs` 225, `envoy_store.rs` 94
  (port/envoydoor; 5 commits ahead of rust-port, 5 behind). `petey.rs` 1190, `petey_model.rs` 1189 (port/petey, 8 ahead).
- Host: 64 sim imports on rust-port + 13 Envoy-only (`gd_envoy_*`, `gd_hold_*`, `gd_stage_transition`, `gd_tag_begin_ex`,
  `gd_transition_kind`); UI imports incl. scenes (`register / replace / goto / back`) + `gd_ui_scene_replace_when` and
  `gd_ui_envoy_*` on envoydoor; `gd_hash_word` exists and is compared by netplay, synctest and the LAB; `gd_match_setup`
  / `gd_ui_match_start` **not defined** (returns `GD_ESTATE`).
- Limits: wasmi, 1 M fuel per sim hook, 4 M per present, 1 MiB fixed memory, 16 KiB stack, state <= 256 KiB.
- Netplay: 2 humans (host port 1, guest port 2), rooms with hole punching and relay fallback, lockstep CSS/SSS/results,
  rematches, spectate; proofs: synctest depth 12 (1802 frames, 0 mismatches), loopback 70 ms / 5% loss 78 rollbacks 0
  desyncs; CPU seats wired (`--cpu`, ports 3-4) with no documented 2-peer proof.
- Fidelity: four main goldens 100%; coverage goldens 182 pass / 30 fail / 10 missing; `.slp` corpus **86 of 125**
  frame-exact (round 5), Hero set 392/428, local 125/125, own 917/917, **4-player set 106 of 206** (remaining: x 55, stocks
  43, percent 2) [R dolphin-oracle.md round 11; STATUS.md still quotes 38/125].
- Perf: 0.20 ms (pad) / 0.53 ms (CPU) per tick headless; snapshot save 0.24-0.29 ms, load 0.35-0.44 ms; depth-7 resim
  ~3.3 ms; render p99 3.05-3.78 ms uncapped; UI mods 43 us p50 / 111 us p99 per present.
- Geno: 64/64 (defines hosted natively); sprite fighters: planning note only (S1-S10 decided).
- LAB: M0-M3 done, M4 part, M8 online done, M9 part; CPU behaviours and reactions; record/play.

### 6.2 What 1.0 needs from the port

| need | why | size | milestone |
|---|---|---|---|
| **Fix the stage-switch heap leak** (~0.7 MB per switch) | 50-70 switches per run | S | 0 |
| **Two-peer CPU rollback proof** (loopback + synctest depth 12 with 2-3 CPUs, 3600 frames) and the Envoy plan + `gd_hash_word` under it | every co-op fight online | S-M | 0 (probe), 6 (ship) |
| **4-player sim fidelity** (`.slp` 4-player set 106/206: x and stocks classes) | Envoy fights are 3-4 slot matches with teams; divergences from Melee are feel bugs even if deterministic | M (ongoing oracle lane, not Envoy's) | parallel |
| `gd_envoy_apply` per-fighter bit, `gd_tag_partner_handle`, `gd_fighter_set_intangible`, `VIEW` 64 -> 160 words | crests per fighter, bench heals, RELAY intangibility | S | 1 |
| per-encounter plan words: kind, shift period, CPU count/levels/crests, boss id, act | run structure | S | 3 |
| the boss framework (`tu::boss`, dispatch, generic HUD view) and the Hands under VS flow (stamina flag, beats, termination) | three bosses | M-L | 4 |
| `gd_ui_project(x, y, z)` | pickups drawn in the world | S | 7 |
| `gd_match_setup` through the lockstep lobby (party, stage set, items off, rules, teams) | camp -> fight without SSS; chained matches; the rest stage | M | 8 |
| `gd_hit_view` + wire `hit.fighter_process_hit`; `gd_fighter_mod_set/get` with engine floors | the two hit-reading crests, knockback prices, Marked as knockback | M (Opus semantics) | 9 |
| `gd_match_rewind(frames)` under a sim command (offline), the confirmed-frame variant online | Second Wind | M offline, L online | 9 / post-1.0 |
| `gd_data_read/write` per-peer files | profile, crest book | S | 10 |
| the `melee_gx` effects system + `gd_fx_*` + `fx-gallery` contact sheets + `fx_lint` | covers beyond wipe/flash, statuses, boss telegraphs, reward moments | L | 11 (started in parallel from 7) |
| the sislib text pool scale (done), results-screen stability with 4 slots | packs | done | - |
| launcher entry (GD's Qt launcher) and a disc-free package | release | S-M | 12 |

### 6.3 Perf envelope [I]

A 2-player co-op pack is 2 pairs (4 fighter objects, two asleep) + 2-3 CPUs = 6-7 fighter objects; at ~0.6-0.8 ms a tick
a depth-7 resim is 4-6 ms, inside the 8.3 ms frame but close; the shift room's extra switches add disc reads (1-2 ms warm
[R seamless-stage-switch]). Measure in milestone 2 (`snapguard.sh` baseline + an Envoy pack scenario) before adding
anything per-frame to the sim mod; keep the sim tick under 10,000 wasm instructions.

---

## 7. Milestones

Sizes: **S** = days, **M** = 1-2 weeks, **L** = 3+ weeks of one agent. Each ends in **one playtest with the owner** (E26:
loopback with the owner on one pad, named windows, volume 3, no OS input from agents; a real-network check at 6 and 12).
Model per memory: Opus for UI, design-sensitive and cross-cutting host work; Sonnet for data, wiring, tests, harness.

| # | milestone | playable checkpoint (the playtest) | size | model | depends on |
|---|---|---|---|---|---|
| **0** | **Merge and probes.** Merge port/envoydoor and port/petey into rust-port (both 5 behind); fix the switch heap leak; two-peer CPU rollback probe written into net.md; `gate.sh`, `stage_switch.sh`, `tag/check.sh` green; STATUS.md fidelity numbers refreshed | today's 5-fight run from the 1-P door, Petey on fight 5, from rust-port | S-M | Sonnet | - |
| **1** | **Crests core** (redesign C1 + C2). `envoy_data` + lint; the five ALWAYS crests and the stock offer replace the six upgrades; moments (tag in/out, shift, pool, opener, boss); the four words; RELAY/BENCH/SHIFT/POOL/OPENER (~34 crests, minus the two hit-readers); per-fighter wearer; full FNV digest in `gd_hash_word`; synctest depth 7 across tags, shifts and picks; goldens | a 5-fight run where crests fire at moments and the card names the wearer | M | Sonnet (sim, data, tests); Opus reviews window semantics and the card | 0 |
| **2** | **Offers, two players, the harness** (C3 + C4). A/B/C/stock offer rules, hone, connect footer, theme line, glossary; local co-op (P1+P3 / P2+P4), two pads in one freeze, the other cursor; `sim.py` with knobs (`MELEE_ENVOY_SEED/POLICY/TAG_EVERY`) and `fire / window / offer` log lines; the first tune to the bands of 3.4; a perf baseline for a 2-pair pack | two people on one machine play a 5-fight run, each picking their own crests; a balance report with numbers | M | Sonnet (rules, harness); Opus (cards, theme line) | 1 |
| **3** | **The 1.0 run.** 3 acts x (3 + boss), encounter kinds (duel, pack, elite, shift room), seeded act stage sets, the per-encounter plan words, foe budget and foe ALWAYS/OPENER crests, the opponent card at the ready moment, **heavy crests at bosses** with the seven prices (C5), the Short run option, the camp shows act/fight; bosses are stand-ins except Petey in act 2 | a full 12-encounter run, solo and local co-op, with the harness bands re-tuned | M | Sonnet (data, budget); Opus (cards) | 2 |
| **4** | **Bosses** (B1). The `tu::boss` framework from `petey.rs`; the generic boss HUD view; the act-1 elite under boss rules; **Master Hand** under VS flow with Crazy Hand joining at half HP for 2 players; Petey's HP/cage numbers settled (the owner's Dolphin save state unlocks observation, else our stand-ins tuned by the harness); boss entry and defeat beats as presentation scenes | three different bosses in one run | M-L | Opus (framework, Hands); Sonnet (elite data, tests) | 3 |
| **5** | **Roster and camp 1.0** (A2). The roster screen replacing the CSS while armed (offline, then lockstep); party, run record, run history and the result card in the camp; the crest book (read-only) | the pair picked on an Envoy screen; the camp tells the run's story | M | Opus, with the owner one step at a time | 3 |
| **6** | **Online** (N1). `envoy=` token in the handshake (armed, seed, record and pool digests); camp online rows; disconnect / rejoin rules (E7); a two-peer synctest over a run; the first **real-network** session with the owner and a friend | two people play a run online | M | Sonnet (token, tests); the room-flow owner | 0 (probe), 2 |
| **7** | **Pickups in the world** (D3). `gd_ui_project`; zones in `envoy_sim`; the 15 s collect hold; lanterns (SDF glyph, the `make_drives.py` bodies per family from a contact sheet the owner picks from); the tag ring at the feet; the shift creep; the TAG READY / SHIFT lines dropped (D Q8). The effects system lane starts in parallel (one Opus agent) | rewards taken by walking to them; the HUD quieter | M | Opus (projection draw, lantern shader); Sonnet (zones, hold); the generator: Sonnet | 3 |
| **8** | **The rest stage** (D2). `gd_match_setup` through the lockstep lobby; camp -> fight under a cover (no SSS); chained matches; the rest stage before each boss with exits as the route, the pool as pedestals, the partner "Recruit" variant; the audited extra stages (Q6) | a run with no retail menu between fights | M-L | Opus (flow, `gd_match_setup`); Sonnet (zones, stage audit) | 5, 7 |
| **9** | **Hit view and Second Wind** (H1 + E20). `gd_hit_view`, `gd_fighter_mod_set/get`, the two hit-reading crests, knockback prices, Marked as knockback, the pool to ~48; Second Wind offline via `gd_match_rewind`; the online probe (confirmed-frame rewind) decides 1.0 or later | the pool complete; a 3-second shared rewind once per fight, offline | M (+ L if online) | Opus (semantics, rewind); Sonnet (wiring, data) | 2, 6 |
| **10** | **Profile, saves, NG+.** `gd_data_read/write`; profile with run history, bests, technique counters, the crest book; record migration rules; the NG+ lap; the `Continue` path hardened (boundary-only) | runs remembered; a second lap | M | Sonnet; Opus for the history screen | 5 |
| **11** | **Looks and sound.** The effects families through the gallery (covers: dissolve, displacement; the four words; the reward moment; heavy halo; boss telegraphs; KO/failure), the tag cinematic with runtime disc models and our camera, the wordmark (O1), music policy (Q10), UI sounds, Petey's stand-in effects (P7); commissions decided (E29) | the game the owner can show | L | Opus (effects, scenes); Sonnet (gallery harness, lint) | 7 |
| **12** | **Release 1.0.** GD's Qt launcher entry; a disc-free package (mods, data, no disc-derived bytes); the how-to-play; a final balance pass against the owner's recorded play (E25); second real-network session; STATUS.md and the notes | 1.0 in the launcher | M | Sonnet; Opus for the how-to-play screen | all |

Order: 0 -> 1 -> 2 -> 3 are strictly serial (each playtest shapes the next). 4, 5, 6, 7 can run in parallel (one Opus
each, memory limits permitting: one big build at a time, cargo jobs capped at 8). 8 waits for 5 and 7. 9 and 10 fit
between. 11 is a long lane that starts at 7 and ends before 12. Rough total: **5-7 months of parallel agent work**; the
playable core (0-3) is about **5-7 weeks**.

---

## 8. Risks

| # | risk | effect | mitigation |
|---|---|---|---|
| R1 | **CPUs under two-peer rollback are not documented as proven** (survey found wiring, no proof; memory says "passed") | every online co-op fight | milestone 0 probe; if it fails, fix CPU determinism as its own lane and ship online with the shift-room / spar kinds first |
| R2 | **The switch heap leak** (~0.7 MB per switch) | a 12-encounter run leaks 35-50 MB; crashes on long sessions | milestone 0; a `heap0 free` assertion in `stage_switch.sh` |
| R3 | **Petey's model is the player's own Brawl files**: without them capsules draw; the fight cannot ship with the model and our stand-in is capsules; HP/cage HP are unconfirmed | act 2's boss looks like a debug shape for most players | Q3: gate Petey on "Brawl files present" with the Hands or the elite as act 2's fallback (P6), **or** commission/own an original body (section 5 (b)); tune HP by the harness and the owner's play |
| R4 | **The Hands under VS flow** (the 1P controller owns the beats; termination, camera, HUD toggles, music cues) | act 3 slips | boss-fights.md section 5's plan (offline first, latch once, replicate the beats); the elite boss as the fallback finale |
| R5 | **4-player fidelity** (106/206 `.slp` 4-player files; x and stocks classes) | Envoy fights are 3-4 slot team matches: subtle feel differences from Melee | the oracle lane continues in parallel; Envoy's determinism does not depend on it |
| R6 | **`gd_match_setup` is not defined**; the SSS detour stays until milestone 8 | the rest stage and "camp -> fight under a cover" wait; every fight passes a retail stage select | the plan overrides the stage today; 8 is M-L and sits after the playable core |
| R7 | **Balance by CPU proxy** (one hit-confirmed L-cancel per 450 s; n small) | technique-shaped crests mistuned; bosses too easy or too hard for people | two personas; the owner's recorded play corrects (E25); bands are starting points; Giovannetti [V]: metrics plus judgement |
| R8 | **Second Wind under rollback** | a 3-second rewind for all peers at a confirmed frame is new machinery | offline first; online decided by the milestone-9 probe (Q9) |
| R9 | **Art needs a person** (wordmark, portraits, a boss body) | placeholders in the owner's hands for months | O1 first with the owner; E29 decides after milestone 3; the roster uses names until portraits exist |
| R10 | **Readability regressions** (the C build's failure: 86 pieces, per-hit fires, queued toasts) | the new pool grows past what a player holds in mind | the lint; one announcement at a time; the harness's fires-per-family band; the four words only |
| R11 | **Scope creep** (companion/garden E21, mazes, Recruit as default, waves, more bosses) | the core never gets its playtests | the milestone gates; post-1.0 list in section 9 |
| R12 | **Save migration** (pool digest changes drop saved runs) | a tuning pass kills a run in progress | migration notice (the readability pass's Q11 answer); tune between the owner's runs, not during |
| R13 | **Perf with 6-7 fighter objects and shift rooms** | resim depth 7 near the 8.3 ms frame | milestone 2 baseline; the sim tick budget |
| R14 | **Zelda / Sheik / ICs excluded** | three fan favourites missing from a 23-roster | post-1.0 engine work; say so on the roster |

---

## 9. Open questions for the owner, ranked (one at a time; each with my recommendation)

1. **Run shape.** 1.0 default = 3 acts x (3 fights + a boss) = 12 encounters, ~20-25 min, with today's 5 fights kept as
   "Short run"? **Recommend yes.** The alternative (8 encounters, two bosses) is cheaper but gives a build only 5 picks.
2. **Bosses in 1.0.** Act 1 an elite retail fighter under boss rules, act 2 Petey, act 3 Master Hand with Crazy Hand
   joining at half HP in 2-player? **Recommend yes**; a fourth, original boss (Porky needs a scrolling arena) after 1.0.
3. **Petey without Brawl files.** For players without their own Brawl dump: (a) gate Petey behind "files present" and use
   the Hands / elite in act 2, (b) ship an original stand-in body (commission or the owner's Blender pipeline), (c) ship
   the capsule placeholder. **Recommend (a) now, (b) when E29's commission round is decided.** Never ship the model.
4. **Shared pool in co-op.** 4 (as solo) or 5 with two players? **Recommend 5**, with CPU count rising before level; the
   harness checks the pool-at-boss band for both.
5. **Encounter kinds in 1.0.** Duel, pack, elite, shift room, boss, rest; the 2-player "spar" (teammate duel for a crest,
   E5) as a later variant? **Recommend the six now, spar later.**
6. **More stages.** Audit Kongo Jungle 64, Pokemon Stadium (frozen), Yoshi's Island 64 for `st_retail_supported`, and
   Melee's All-Star rest area for the rest stage? **Recommend the audit (Sonnet, S)**; ship whatever passes; hazard stages
   never in 1.0.
7. **Meta-progression.** Nothing gated in 1.0 (all 23 fighters and all crests available from run 1); the profile keeps
   history, bests, counters and a crest book? **Recommend yes.** Unlock gates can come with the companion layer later.
8. **Mid-run saves.** Resume only at fight boundaries (as built), never mid-fight? **Recommend boundary-only**; mid-fight
   snapshots are disc-derived and offline-only, not worth the save-file rules.
9. **Second Wind online.** Ship it offline in 1.0 and decide online by the milestone-9 probe? **Recommend yes.**
10. **Music.** On a stage shift the music follows the destination stage (today it does not change); boss music = the
    retail Hands theme for act 3; Petey's music is the destination stage's (no Brawl audio ever)? **Recommend yes.**
11. **Team attack in co-op.** Off (you cannot hit your partner) in 1.0? **Recommend off**; the tag pair already shares a
    pool, friendly fire adds noise the harness cannot read.
12. **NG+ rules.** Same pair and crests, foes gain count and weight, a third heavy slot, elites get RELAY crests?
    **Recommend yes**, numbers from the harness.
13. **Local 2-player ports.** Players on ports 1 and 2, partners on 3 and 4 (P1+P3 / P2+P4)? **Recommend yes** (S in
    `match_begin`).
14. **Release target.** 1.0 ships in GD's Qt launcher as the Rust engine's headline mode; a YouGame listing only after a
    disc-free package is proven (memory: Patches listing, independence from Hero)? **Recommend launcher first.**
15. **Elite boss body.** The act-1 elite is any retail fighter at x1.25 scale with a crest set (Adventure's Giant/Metal
    idea), chosen by the seed; or a fixed cast of three? **Recommend seeded from a curated list of 8** (readable
    silhouettes at scale: DK, Bowser, Ganondorf, Samus, Falcon, Marth, Link, Mewtwo).
16. **The roster's missing three.** Say "Zelda, Sheik and the Ice Climbers join later" on the roster screen? **Recommend
    yes**, in one line.
17. **Harness personas.** The second persona is a recorded set of the owner's own inputs (E25) or the LAB's reaction
    dummy? **Recommend the LAB dummy first** (no session needed), the owner's recording when E25's session happens.
18. **Crest book look.** Lantern glyphs as the crest book's icons (one shape per family, hone ring, heavy crown)? **Recommend
    yes**, from the same `make_drives.py` round.

Post-1.0 list (not questions now): companion/garden (E21), Recruit as default, spar, Zelda/Sheik and ICs, Porky /
Galleom / Tabuu, 3-4 players, online Second Wind, replay takeover of a run, items, sprite-fighter elites (S1-S10).

---

## 10. Sources

### 10.1 Papers and books cited by `envoy-plan.md` section 6 and `envoy-redesign.md` section 8, re-verified today

[V] = the source was located today and the claim we use was checked against it; [V, partial] = the source exists, the
claim is checked only against an abstract, a preprint or coverage; [U] = not verified. **Corrections** are what the earlier
docs got wrong or over-read.

| source | status | what it supports here | correction |
|---|---|---|---|
| Togelius, Yannakakis, Stanley, Browne, "Search-Based Procedural Content Generation: A Taxonomy and Survey", IEEE TCIAIG 3(3):172-186, 2011, doi 10.1109/TCIAIG.2011.2148116 | [V] via Semantic Scholar metadata and the 2010 EvoApplications text (the journal PDF is paywalled): "Three key classes of fitness functions ... direct, simulation-based and interactive"; simulation-based = "an artificial agent playing through some part of the game"; constructive vs generate-and-test defined | the harness is a simulation-based evaluation; run graphs are constructive, their contents tested | none |
| Shaker, Togelius, Nelson, *Procedural Content Generation in Games*, Springer 2016, doi 10.1007/978-3-319-42716-4 | [V] pcgbook.com: author-version chapters free; ch. 2 search-based, 3 constructive, 12 evaluating generators | constructive templates, offline search | none |
| van der Linden, Lopes, Bidarra, "Procedural Generation of Dungeons", IEEE TCIAIG 6(1):78-89, 2014 | [V] abstract: current methods need "richer, more accurate control"; gameplay-related criteria as the source of control | the run is authored at encounter/reward level, not geometry | none |
| Yannakakis, Togelius, "Experience-Driven Procedural Content Generation", IEEE T-AFFC 2(3):147-161, 2011 | [V] metadata + extended abstract: player experience model, content quality, representation, generator | the profile's technique counters as the player model | none |
| Hunicke, "The Case for Dynamic Difficulty Adjustment in Games", ACE 2005, pp. 429-433, doi 10.1145/1178477.1178573 | [V, partial] OpenAlex abstract; the Hamlet companion paper (Hunicke and Chapman) | DDA must not disrupt the core experience | **Correction:** the earlier plan said Hunicke's DDA "acts between events rather than mid-event". The abstract does not say that, and Hamlet adjusts inventory **during play** (reactively and proactively). Boundary-only adjustment is **our** rule (the readability lesson), not Hunicke's. Semantic Scholar's venue for this DOI ("IFAC") is wrong; ACE 2005 is right |
| Chen, "Flow in Games (and Everything Else)", CACM 50(4):31-34, 2007 | [V] the author's text: "Embed DDA choices into the core gameplay mechanics and let player make their choices through play" | the tag and the route as in-play difficulty choices | none |
| Juul, *The Art of Failure*, MIT Press 2013 | [V, partial] reviews and the publisher: the paradox of failure; failure "implicates the player"; an internal/external attribution axis | the result card names what ended the run | **Correction:** "attributable" is Juul's; "recoverable" is not a claim I could find in his text; it is our design rule |
| the Berlin Interpretation (IRDC 2008) | [V] RogueBasin: high-value factors (random environment generation, permadeath, turn-based, grid-based, non-modal, complexity, resource management, hack'n'slash, exploration and discovery), low-value (single player character, monsters similar to players, tactical challenge, ASCII, dungeons, numbers); "Missing some points does not mean the game is not a roguelike" | Envoy keeps random generation, run permadeath, resource management; drops turn-based, grid, single character knowingly | none |
| Giovannetti, "'Slay the Spire': Metrics Driven Design and Balance", GDC 2019 | [V, partial] GDC Vault listing confirms the session and speaker; details from Game Developer and Seattle Indies coverage ("with 200 cards ... tough for two people to just say it's perfectly balanced") | the harness and the profile counters as our metrics server; metrics plus judgement | none; no transcript verified |
| Yu, *Spelunky*, Boss Fight Books, 2016 | [V] publisher: released 2016-03-29, 222 pages (Boss Fight Books #11); the 4x4 room grid with hand-built templates whose parts are randomised is from GMTK/Killscreen coverage of the generator | hand-authored templates, each played in isolation | none |
| Rupp, Puddu, Becker-Asano, Eckert, "It might be balanced, but is it actually good? An Empirical Evaluation of Game Level Balancing", IEEE CoG 2024, doi 10.1109/CoG60054.2024.10645642 | [V] Semantic Scholar abstract (arXiv 2407.11396): PCGRL-balanced levels, a survey over four scenarios, "positively influences players' perceived balance for most scenarios", effects differ per scenario | simulation-balanced content, confirmed with people | **Correction:** the 0.4-0.6 band is **ours**; the paper balances toward equal win rates. The related IEEE Trans. Games 16(4):903-913, 2024 paper is titled "Simulation-Driven Balancing of Competitive Game Levels With Reinforcement Learning" (Rupp, Eberhardinger, Eckert, doi 10.1109/TG.2024.3399536); "Balancing of competitive two-player Game Levels with RL" is the 2023 CoG paper |
| Holmgård, Green, Liapis, Togelius, "Automated Playtesting with Procedural Personas through MCTS with Evolved Heuristics" | [V] IEEE Trans. Games 11(4):352-362, **2019**, doi 10.1109/TG.2018.2808198 (arXiv 1802.06881, 2018): personas as synthetic playtesters for PCG | two harness personas | **Correction:** the earlier plan dated it 2018 and marked it [U]; it is the 2019 journal paper |
| Fagerholt, Lorentzon, *Beyond the HUD: User Interfaces for Increased Player Immersion in FPS Games*, Chalmers MSc thesis, 2009 | [V] the thesis PDF: a design space of **diegesis x spatiality** (a four-field chart), plus an alternative fiction / rules / space model; four guidelines | the vocabulary for in-world vs overlay | **Correction:** the labels "diegetic, non-diegetic, spatial, meta" are **not** the thesis's; it has the two dimensions. The four-label version is Marcus Andrews' 2010 Gamasutra article popularising it [U: not fetched]. Re-attribute in `envoy-redesign.md` 5.1 |
| Jørgensen, *Gameworld Interfaces*, MIT Press 2013 | [V, partial] publisher and reviews: gameworlds are governed by game mechanics, interfaces are hybrids of in-world and overlay | integration, not purity | the "too coarse" critique of diegetic/non-diegetic is explicit in her 2011 chapter on game sound; the book treats it as a spectrum |
| Llanos, Jørgensen, "Do Players Prefer Integrated User Interfaces? A Qualitative Study of Game UI Design Issues", DiGRA 2011 | [V] DiGRA library abstract: "no necessary connection between a transparent interface and involvement ... in many cases, overlay interfaces are preferred due to the clear information they present" (Assassin's Creed without its UI, observed and interviewed) | keep exact numbers on the HUD | **Correction:** the redesign said players "prefer integrated interfaces when they cost nothing in usability". The paper's emphasis is the reverse: overlays are often preferred. Our rule survives; the phrasing did not |
| Iacovides, Cox, Kennedy, Cairns, Jennett, "Removing the HUD: The Impact of Non-Diegetic Game Elements and Expertise on Player Involvement", CHI PLAY 2015, pp. 13-22 | [V] (abstract and citing summaries; the UCL PDF refused the fetch): Battlefield 3 with and without HUD, 10 experts and 14 novices; HUD removal raised immersion for experts, little effect on novices | world-carried lines only for redundant information; numbers stay | the redesign marked the findings [U]; now known |
| Peacocke, Teather, Carette, MacKenzie, "Evaluating the effectiveness of HUDs and diegetic ammo displays in first-person shooter games", IEEE GEM 2015 | [V] Carleton PDF listing and the 2018 follow-up (Entertainment Computing): the in-game **number** display gave 35% fewer wasted shots and 26% faster reloads than icons-on-HUD, preferred by 70%; the 2018 study: ammo best diegetic/spatial, **health best on a HUD** | counts and cooldowns can live in the world; percent and stocks stay on the HUD | the redesign marked the result [U]; now known and it cuts both ways |
| Supergiant / Kasavin on Hades | [V, partial] the GDC 2021 session was on dialogue (Kasavin and Korb); the "bosses remember" idea is from the GDC Podcast, January 2021 | not adopted; noted for the companion layer | as the plan said |
| SmashWiki, "Stage Morph" (CC BY-SA 4.0) | [V] Off / Random / Every 1:00-5:00 in 30 s steps; both stages loaded at once; fighters outside the new blast lines KO'd; stage-bound items despawn; not in First-to-1 | the shift rules of 3.3 | none |
| Joyastick, melee-pc-TagFighter (README, idea only) | [V] D-pad down calls the assist, D-pad down again swaps point; up to three tags per call with cooldowns; the assist is promoted when the point is eliminated; a sole survivor may donate a stock to revive the partner; `src/melee` unlicensed, port code GPL-3.0-or-later | the tag idea; the bench-not-spawn lesson | none; no code read or copied |

### 10.2 Claims the sources do not support (summary for the owner)

1. "DDA acts between events, never mid-event" is not Hunicke's position; it is ours.
2. "Players prefer integrated interfaces when they cost nothing" over-reads Llanos and Jørgensen, whose finding leans
   toward overlays for clear information.
3. The four-label HUD taxonomy is misattributed to Fagerholt and Lorentzon (they have diegesis x spatiality).
4. "Failure must be recoverable" is not Juul's claim; "attributable" is.
5. The 0.4-0.6 win-rate band is not from Rupp et al.; they target 0.5 and show perception follows simulation balance for
   most scenarios. The IEEE Trans. Games 2024 paper's title in `envoy-plan.md` is the 2023 CoG paper's.
6. Holmgård et al. is the 2019 journal paper (not 2018).
7. Memory's "CPUs passed under rollback" (E2) has no note or test I could cite; treated as unproven until milestone 0.

### 10.3 Internal notes read (`E:\Projects\Melee Workspace\_research\`)

`boss-fights.md`, `adventure-arena-lock.md`, `adventure-building-blocks.md`, `roguelite-ai-2026-09-30.md`,
`envoy-strength-decision-2026-10-07.md`, `frontend-menus.md` (read directly); `envoy-readability-pass1`,
`envoy-synergy-analysis`, `envoy-visual-identity`, `envoy-netplay-scoping`, `envoy-gameplay-ui-audit`,
`chao-genetics-for-envoy`, `camera-for-large-levels`, `large-levels-and-maze-stitching`, `seamless-stage-switch`,
`tagfighter-insights`, `menu-kit-pieces` (summarised by a Sonnet agent, numbers quoted exactly). The chao-genetics,
camera and large-levels notes belong to the retired maze era; their carry-overs here are: a persistence currency needs a
natural brake (chao), tween the camera origin 30-60 frames on a shift and set min distance per stage (camera), and
preload stages as resident chunks (large levels, seamless switch). Every numeric claim from these notes is a C-build,
CPU-proxy, small-n measurement: calibration, not spec.

### 10.4 Repo files read (via `git show pub/<branch>:<path>`)

port/envoyplan: `docs/rust-port/notes/envoy-plan.md`. port/envoyloot, port/envoydoor: `envoy-redesign.md`, `envoy-1p.md`,
`rust/mods/envoy_sim/src/lib.rs`, `rust/mods/envoy_ui/src/lib.rs`, `rust/melee_modes/src/tu/{envoy,tag_swap,stage_hop}.rs`,
`rust/melee/src/envoy_store.rs`. port/petey: `petey-boss.md`, `petey-behavior.md`, `petey-extract.md`,
`rust/melee_modes/src/tu/{petey,petey_model}.rs`. port/porky: `brawl-boss-petey.md`. rust-port: `STATUS.md`,
`notes/{lab-theorycraft,lab-ui-spec,net,spectate-replay,modhost,wasm-ui-lab-api,slpcheck,golden,bench-unlocked,perf,
dolphin-oracle,coverage,geno,geno-wasm-plan,menus,play}.md`, `docs/specs/{gdmod.h,gd-ui-lab-v1.h}`,
`rust/melee_modhost/src/{lib.rs,ui/mod.rs}`, `rust/melee/src/play_net.rs`, `rust/mods/lab`. Memory:
`envoy-lab-answers-2026-11-09.md`, `launcher-roguelite-decisions.md`, `design-user-in-loop.md`, `melee-rust-port.md`,
`lab-menus-wasm.md`, `c-build-frozen-reference.md`, `agent-model-choice.md`, `hero-renderer-endgame.md`.
