# Envoy redesign: getting in, the reward layer, and diegetic play (2026-11-09)

Design only. Nothing here was built or run. Branch `port/envoyloot`, worktree `C:/melee-wt/envoyloot`, off `pub/port/envoy1p`
(`52a5933d4`). It continues `envoy-plan.md` (port/envoyplan) and the owner's answers of 2026-11-09 (memory
`envoy-lab-answers-2026-11-09.md`), and it designs on top of what `envoy-1p.md` built. Tags as in the plan: **[R]** read from
source or a note (cited), **[I]** inferred, **[J]** judgement, **[U]** unverified, **[V]** source verified to exist.

**Owner instructions folded in (2026-11-09, via the coordinator):**

1. **Do not use the old Envoy "drive" design**: no prefix/suffix drives, colour families, rarity tiers, keystones, merging, no
   reuse of the `envoy_drives` emblem models, and the readability-pass numbers are not a spec. Those documents were read for
   what existed and what failed (section 2.1), nothing more. This partly supersedes answer **E19** ("keep the earlier
   numbers"): the *limits* that pass kept (one sentence per rule, few status words, no price paid with a status on yourself,
   no "assembled" toast) survive as lessons; the objects they were limits *on* do not (question Q3).
2. **A proper way into Envoy from inside the game.** Today Envoy is reached by launching a `.cmd` with the mods loaded, after
   which every VS match silently becomes an Envoy fight (`envoy-1p.md` "Play") [R]. Section 1 is the main answer.
3. **A more diegetic way to play**: fewer menus and overlay cards, more of the run carried by the world. Section 5, secondary.

Rules from memory that bound this note: wasm mods only (never the old script host); the C build is frozen; no old GD art; the
owner designs visuals one step at a time (section 6 gives directions and questions, no art); everything in the sim is
deterministic and rolls back; commit nothing disc-derived.

---

## 0. One page

- **In:** a row in the game's own menu (`melee play`'s Solo menu, beside Classic / Adventure / All-Star) and the same row in
  the in-game browser's MODES catalog (where Envoy already sits as "unavailable" [R]); it opens an **Envoy camp scene** (a
  presentation-mod scene) with Continue / New run / Party / Abandon. The sim mod only publishes a match plan while the run
  is **armed** from that scene, so VS stays retail with the mod installed (section 1).
- **The reward object is the Crest.** One rule, one number, worn by **one fighter of the pair**. Most crests fire at the
  **moments** the new Envoy already makes loud and rare: tag-in, tag-out (the bench), a stage shift, a stock lost from the
  shared pool, the fight's opening, a boss. Six per player (three per fighter), plus at most two **heavy** crests (rule and
  price) earned at bosses. No rarity, tier, affix, merge, bag or strength percent (section 2).
- **Offers:** three per player at each fight boundary, one of which always connects to something held, one from a family the
  party lacks, one random or a "hone" of a held crest; never a dead card by construction. Both players pick with their own
  pads inside the sim freeze, so the pick is rollback-safe with no new command channel (section 3).
- **Measured, not felt:** a headless harness (the `envoy-1p` win path already runs a whole run with pad scripts) scores
  fires per moment, damage inside windows, offer quality, pair completion and the CPU-vs-CPU win band 0.4-0.6 (section 3.6).
- **Built where the 1P slice is:** the six placeholder upgrades become the five ALWAYS crests and the POOL "+1 stock" offer;
  the pool is a generated table shared by `envoy_sim` and `envoy_ui`; new host imports are few and small (section 4).
- **Diegetic flow, later:** chained matches with a rest stage between fights, rewards as objects you walk to, the route as
  exits you stand in; three directions sized and ranked (section 5). Visual directions for crests and pickups, three distinct
  mechanisms, no art (section 6). Eight owner questions (section 7).

---

## 1. Getting into Envoy from inside the game

### 1.1 What exists to build on [R]

| fact | where |
|---|---|
| The **retail main menu runs** in the port (`mnmain.rs`: a mode pick writes `MenuExitData.pending_mode` and ends the scene); its layout is the disc's menu data. The C build's kit menus (its own menu tree, settings pages, lobby, select screens) are **not ported**; `melee play` draws stand-ins for screens that are not retail scenes | `rust/melee/src/browser.rs` 7-8, 304-313; `notes/menus.md` |
| The in-game browser mod (`rust/mods/browser`, Z on the main menu) lists a **MODES** catalog with `id, group, title, description, available, note`; a `GD_UI_REQ_LAUNCH` request starts a retail mode in place "with the menu's own two writes" (`MenuExitData.pending_mode`, `gm_801A4B60`), offline; boot-time modes relaunch `melee play`. The catalog already has a row `envoy` (group "Online", "Best-of sets with an Envoy build per game", `Launch::Unavailable("Envoy is not in this build yet.")`): the text is the old C-build Envoy's and the row is dead | `notes/wasm-ui-lab-api.md` A6; `docs/specs/gd-ui-lab-v1.h` 139-150; `rust/melee/src/browser.rs:218` |
| A presentation mod can register its own scene (`gd_ui_scene_register`, layer SCENE, retail GX off while on top), **replace a retail scene** (`gd_ui_scene_replace(major, minor, scene)`: "entering that retail scene, for example mnMain, CSS or SSS, enters the custom one instead", one replacer per scene), and jump (`gd_ui_scene_goto`, `gd_ui_scene_back`) | `wasm-ui-lab-api.md` A5; implemented in `rust/melee_modhost/src/ui/mod.rs:695` |
| `gd_ui_match_start` returns `GD_ESTATE`: "gd_match_setup is not defined yet" | `ui/mod.rs:737` |
| The Envoy match rewrite: `melee_modes::tu::envoy::match_begin` at the top of `fn_8016E730` rewrites `StartMeleeData` from the mod's **plan** (pair, stocks, ratios, CPU line-up, stage); **inert while `PLAN[0] == 0`**; VS only, never on replay playback. The sim mod publishes the plan in `gd_mod_loaded()`, which is why every VS match is an Envoy fight today | `rust/melee_modes/src/tu/envoy.rs`; `rust/mods/envoy_sim/src/lib.rs` `make_plan`, `gd_mod_loaded` |
| Netplay carries agreed per-session words in the scene token (`;np_rules=s<stocks>,t<secs>,i<items>`), host-authoritative; the wasm mods loaded are part of the handshake identity (`play_net::mods_identity`) | `notes/net.md` 281; `rust/melee/src/play_net.rs:669` |
| The retail flow after a fight today: results -> CSS -> SSS -> next fight; the run's pair is "P1's pick = main, P2's slot = partner" on the retail CSS | `envoy-1p.md` |

### 1.2 Where Envoy lives: three options

| | **A. A Solo-menu row** (recommended) | **B. The browser row only** | **C. Reached from VS: an "Envoy" tile on the stage select** |
|---|---|---|---|
| what the player does | Main menu > 1-P > **Envoy** (beside Regular Match, Event Match, Stadium) opens the Envoy camp scene. The browser's MODES row becomes `available` when both Envoy mods are loaded and launches the same scene | Z on the main menu, Modes, Envoy. Nothing in the menu tree itself | VS as usual; on the stage select a custom tile "Envoy" (the SSS is replaced by a custom scene while the mods are loaded) starts the run with the CSS picks |
| how it is added | two ways, pick by cost once probed: **(a)** `envoy_ui` replaces the retail **1-P submenu scene** with a custom one (`gd_ui_scene_replace`) that draws the retail rows and one more, routing the retail rows back to the retail scene (`gd_ui_scene_goto`) and "Envoy" to the camp; the API's intended path, mod-side only, but the submenu's look is then ours; **(b)** a native row in the retail menu (`mnmain.rs`; the menu tree is disc data, so this is a native edit of the menu scene, M **[U]** until someone opens it); either way **browser**: `available` + LAUNCH(envoy) -> arm + `gd_ui_scene_goto(camp)`; **envoy_ui**: the camp scene | browser only; envoy_ui's camp scene | envoy_ui: `gd_ui_scene_replace(SSS)` with a scene that draws the retail SSS's choices plus one tile (a rebuilt stage select, not a tile added to the retail one [I]) |
| coexists with VS | yes: VS from the menu never arms the run | yes | yes, but the SSS is now a mod screen for every VS match, online too |
| discoverable | yes: it sits where Classic sits (the C build's README said "SOLO > ENVOY" [R]) | no: Z is a hidden key | odd: a mode hiding as a stage |
| size, model | (a) S-M, Opus (the submenu and the camp are the owner's first Envoy screens); (b) M, Sonnet after a probe | S (Sonnet) | M (Opus) and it rebuilds the SSS first |
| netplay | the camp's "New run (online)" hands the room flow an `envoy=` token (as `np_rules` is carried today); both peers' camp scenes agree on it; **stub until the plan is in the handshake** (`envoy-1p.md` "Offline only") | same | the lockstep SSS would have to be the custom one on both peers |

**Recommendation: A, way (a) first, with B's row kept as the second door** (B is a few lines once the camp exists: the
row's text, group and `Launch` change; the camp arrives through `gd_ui_scene_goto`). Way (b) replaces (a) if opening
`mnmain.rs` shows the menu data is cheap to extend. C is the most "in the game" of the three but it bends the wrong thing: a
stage select that is secretly a mode select, on every VS match. If the owner wants a *world* door later, it belongs in
section 5 (the rest stage), not on the SSS.

### 1.3 Start, continue, quit: the camp scene

The camp is one `gd_ui` scene owned by `envoy_ui` (kit nodes, so focus and repeat are the host's). Rows:

| row | what happens | sim side |
|---|---|---|
| **Continue** (shown when a run record exists: "fight 3/5, pool 2, Fox + Marth") | arms the run and goes to the next fight's setup (today: retail CSS with the pair pre-chosen, then SSS, Start; once `gd_match_setup` lands: straight into the fight under a cover) | the sim mod gets the record and publishes the plan |
| **New run** | party select (1.4), then the first fight | a fresh run record |
| **Party** | change the default pair without starting | record only |
| **Abandon run** | asks once; deletes the record; disarms | plan withdrawn (`PLAN[0] = 0`) |
| **Back** | main menu | nothing |

- **Arming.** A host flag `envoy::ARMED` (game-side static, menu-time state, not match state) is set by the camp's Start
  and by the browser's LAUNCH(envoy), cleared by Abandon, by RUN CLEAR / RUN OVER (after the camp has shown the result)
  and by leaving to the main menu. New sim import **`gd_envoy_armed() -> i32`**; `make_plan` publishes only when armed.
  Everything else in `match_begin` stays as it is. With the flag clear the mods are loaded and inert: **VS is retail**.
- **Quit from inside a fight.** The retail L+R+A+Start quits the match; today "quitting a fight retries it" [R], which is
  the right rule (the run stands at its last boundary; Juul: failure attributable, the run recoverable from the boundary
  [V]). The results screen then lands in the camp (1.5), where Continue and Abandon are the two choices. No pause-menu work.
- **Run record.** Today the run lives in the sim mod's memory for the session [R]. The record (seed, fight, pool, pair,
  crest slots, picks; ~200 B) is published in the view each frame already; the host persists it: on RUN LIVE boundary
  frames the host copies the view's record words to `melee_data/envoy/run.bin` (atomic replace), and hands them back on
  arm: new imports **`gd_envoy_record(out, cap) -> len`** (sim, at arm time) and the host-side save (no import; the host
  reads the published view). A digest word in the record refuses a stale or edited file with a notice. Online: the record
  rides the agreed token; both peers hold the same bytes (plan 3.4).
- **Results after the last fight.** RUN CLEAR / RUN OVER are shown in the camp (the retail results screen still plays
  for the fight itself), with "what ended the run: fight 4, pool emptied by the Falco" (Juul [V]; plan 2.4).

### 1.4 Party and partner select: three options

| | **S1. Retail CSS, as today** | **S2. An Envoy roster screen** (recommended next) | **S3. Recruit the partner in the run** |
|---|---|---|---|
| how | P1's pick is the main, P2's slot (HMN or CPU, any character) is the partner [R]. Two players: P1 + P3 are player 1's pair, P2 + P4 player 2's (`match_begin` reads only P1/P2 today; S work) | a custom scene replacing the CSS while armed: each player picks main, then partner, on a grid of the 23 taggable characters (`envoy::taggable`), names first, portraits when a texture path for the disc's CSS portraits is confirmed [U]; 2 players side by side; online in lockstep like `--np-css` | the run starts with one fighter; fight 1 is 1v1 with no tag; **the beaten opponent joins as the partner** from fight 2 (`TAG_PARTNER_CK` is set per match in `match_begin`, so this is data, not engine work). The route choice before fight 1 (section 5) is then also the partner choice |
| cost | 0 (built); the 2-player mapping S | M, Opus (design-sensitive; the owner's second Envoy screen) | S once chained matches exist; a run rule, not a screen |
| for | nothing to build; the lockstep CSS already works online | says what it is; no "P2's slot means partner" riddle; shows crests per fighter later | the most in-world; makes fight 1 a choice with a consequence; fits Direction C's "party as the rhythm" |
| against | confusing; in 2-player the retail CSS has no notion of a pair; Zelda / Sheik / ICs silently fall back to Fox / Marth [R] | one more screen to design and to lockstep | you cannot pick your favourite pair; a run variant, not the default |

**Recommendation:** ship S1 under the camp now (one sentence on the camp explains the slots), build S2 at the camp
milestone, and ask the owner about S3 as a **run variant** ("Recruit"), not the default (Q2).

### 1.5 How the flow fits together (and what each step needs)

```
main menu > 1-P > Envoy   ──>  camp (envoy_ui scene)  ──Continue/New──>  [party: S1 CSS | S2 roster]
                                                                                │
      ┌─────────────────────────────────────────────────────────────────────────┘
      v
   SSS (any stage, Start; the plan overrides the stage)  ──>  fight (envoy_sim rules)  ──>  retail results
                                                                                                │
                                        camp (next fight / result / abandon)  <── scene_replace(CSS while armed) ──┘
```

| step | today | with section 1 | later (needs `gd_match_setup`, plan 4.2 item 5) |
|---|---|---|---|
| entry | launch with mods; any VS | Solo row / browser row -> camp -> arm | same |
| party | retail CSS trick | S1 under the camp; S2 replaces the CSS while armed | S2 feeds `gd_match_setup` |
| stage select | SSS, any stage | SSS, any stage (plan overrides) | skipped: camp -> fight under a cover |
| between fights | results -> CSS -> SSS | results -> **camp** (the camp replaces the CSS while armed and offers Continue) -> SSS | results replaced by the camp or by the rest stage (section 5) |
| quit | quit the fight = retry | same, then camp: Continue / Abandon | same |
| VS with the mod installed | becomes Envoy | retail unless armed | retail unless armed |

**Replacing the CSS only while armed.** `gd_ui_scene_replace` is permanent once registered (one replacer per retail scene
[R]). The camp scene handles it: when it is entered through the CSS replacement and the run is **not** armed, it calls
`gd_ui_scene_goto(retail CSS)` on its first present (a one-frame pass-through [I]). If that frame is visible, the host gets
a `gd_ui_scene_replace_when(major, minor, scene, flag)` variant later; not needed to start.

**Netplay (2 players).** The camp's "New run (online)" and "Continue (online)" go through the room flow with an `envoy=`
token carrying the armed flag, the seed and the record digest; the host's token wins (as `np_rules` does [R]); the CSS is the
lockstep CSS (S1) until S2 is lockstepped. The plan is then in the handshake, which closes `envoy-1p.md`'s "Offline only"
stub. Sized in section 4.5.

---

## 2. The reward object: the Crest

### 2.1 What the drives taught (read for lessons; none of the structure is reused)

| the drive system had | measured result [R] | the lesson this design keeps |
|---|---|---|
| prefix/suffix affixes stacked per drive; names as sentences | a rare drive carried 4.2 rules and 50 words; 38 of 86 pieces failed the one-sentence rule (readability pass 0, 3 row 1, 8) | **one rule per object**, a 12-word ceiling, two numbers at most, checked by a lint |
| colour families, rarity tiers, merging, a bag | duplicates were dead (11 of 22 rules in the owner's build), the tier curve was the real power (x3.5 by stage 25), rarity meant "more rules" (readability 3 row 2; synergy 0.4, 4) | no tiers, no rarity, no merge, no bag; **a second copy is never offered**; growth is more crests and a once-only hone |
| keystones with an unbounded allowance and prices paid with a status on yourself | 12 keystones at depth 56; 13 of 42 paid with Chill/Curse/Burn, so a status look meant two things (readability 3 rows 3-4) | at most **two** heavy crests per run, at bosses only; a price from a fixed list, **never a status** |
| seven statuses touched by up to 18 pieces each | "Chill" meant both "you slowed them" and "you paid" (readability 3 row 4) | **four words**, each meaning one thing, each given by few crests |
| triggers that fire per hit | 3.9 per-hit rules live per build; one hit started 3-4 effects; a "Modifier!" flash on each (readability 3 row 5; audit 2a) | rules fire at **moments** (tag, shift, stock, opener, boss), a few times a fight, never per hit |
| synergy by shared tags, announced on assembly | pairs fired but the uplift was inside fight noise (n=4 cells detect +-30%); technique / shock / armour sets completed in 4-9% of runs (synergy 0.1-0.2, 3) | synergy is a crest **reading a word another gives**, in the same word; effects large and rare enough to measure; **offers connect** |
| a strength number shown as `+16,800%`, used to scale foes | a progress meter sold as difficulty; Echoes counted x2.20, measured x1.05 (strength decision 0, 2) | an integer **weight** per crest for the foe budget; the player sees **compare rows only** (E22) |
| 17% dead and 44% coin-flip drive screens | the keystone screen was the real choice (tension 48%, dead 1%) (synergy 4) | the offer rule forbids dead cards; the heavy-crest choice at a boss keeps the one screen that worked |
| the floor drop and pickup "felt nice"; the models "10/10" | spec 5; visual identity 0 | **pickups in the world** come back (section 5), with **new** objects (section 6); the old models stay retired |

### 2.2 Definition

A **Crest** is worn by one fighter of a player's pair. It has:

| field | values | notes |
|---|---|---|
| `family` | ALWAYS, RELAY, BENCH, SHIFT, POOL, OPENER, HEAVY | the moment it keys on (ALWAYS has none; HEAVY carries a price) |
| `rule` | one sentence, <= 12 words, <= 2 numbers | lint-checked at build |
| `number` | one i32 (x1000) | the one thing a hone changes (+50%, once) |
| `weight` | 1, 2 or 3 | the foe budget unit (3.5); shown nowhere but the harness |
| `gives` / `reads` | 0 or 1 word each from {Fresh, Benched, Marked, Burning} | the synergy edge; same word in both texts (readability R11 kept) |
| `wearer` | main or partner | assigned when taken (Q4) |
| `price` (HEAVY only) | one of seven | section 2.4 |

**The four words** (the whole status vocabulary):

| word | on whom | what | how it is implemented in v1 [I] |
|---|---|---|---|
| **Fresh** | the fighter who just tagged in (also granted by some SHIFT / POOL / OPENER crests) | a 3 s window (180 frames) in which RELAY crests act | a frame counter per fighter in `envoy_sim`; ratio and attribute writes at the window's start and end (`gd_envoy_apply`) |
| **Benched** | the fighter on the bench | the state BENCH crests read (heal per second, charge the next Fresh) | the bench is `Player.transformed`'s sleeping sub-fighter [R tag_swap]; its percent is written with `gd_fighter_set_percent` on its handle |
| **Marked** | a foe | takes +20% damage for 4 s | the foe slot's defense ratio for the window (`Player_SetDefenseRatio` through `gd_envoy_apply(slot, UP_DEFENSE)`); knockback instead when `gd_fighter_mod_set` exists |
| **Burning** | a foe | 1% per second for 5 s | `gd_fighter_set_percent` every 60 frames; capped so it never KOs by itself (a Burning KO reads as unfair; the last point is left) |

Moments the sim already sees without new hooks [R `envoy_sim`]: **tag-in / tag-out** (transition kind 1 release),
**shift** (kind 2 release), **pool stock lost** (`gd_player_stocks(0)` falls), **KO dealt** (a CPU slot's stocks fall),
**opener** (the match `seq` edge, first 600 frames), **boss** (fight 5, and the boss's percent crossing thresholds via
`gd_fighter_get_percent`). Only two crest texts need a hit event ("the foe you last hit", "your first hit while Fresh");
they wait for the hit view (section 4.3) and are not in the first pool.

### 2.3 The first pool (shape, not final numbers; ~34 crests)

Numbers are placeholders the harness tunes (3.6). One rule each; the word in **bold** is the synergy word.

| family (count) | examples |
|---|---|
| ALWAYS (5) | Power: this fighter's hits deal +20%. Guard: takes -15% (floor x0.40). Heavy: weight +20%. Swift: walk and dash +12%. Spring: jump +12%. (The five placeholders, re-homed to one fighter each.) |
| RELAY (7) | Tag in: **Fresh** hits deal +35%. Tag in: heal 12%. **Fresh**: speed +30%. Tag in: the nearest foe is **Marked**. Tag in: 20 more frames intangible. **Fresh** lasts 6 s instead of 3. Tag cooldown 5 s -> 3.5 s (party). |
| BENCH (5) | **Benched**: heal 1% per second. **Benched** 10 s or more: the next tag-in is **Fresh** 6 s. Tag out: foes within reach are **Burning**. Tag out: your partner's first 5 s are +15% damage. **Benched** fighter's shield is full when it returns. |
| SHIFT (6) | Shift: both fighters heal 8%. Shift: every foe is **Marked**. Shift: the active fighter is **Fresh**. Shift: the next shift comes 10 s later (party). Shift: foes are **Burning**. Shift: +1 shared stock, once per fight (weight 3). |
| POOL (5) | Pool stock lost: the respawn is **Fresh** 5 s. Last stock: hits deal +25%. Pool stock lost: every foe is **Burning**. KO dealt: heal 15%. Pool at 1: Guard x0.7 (both). |
| OPENER (4) | Opener: **Fresh** for 10 s. Opener: foes start at 20%. Opener: the first shift comes at 15 s (party; pairs with SHIFT). Opener: both fighters' shields are oversized for 15 s. |
| HEAVY (6, bosses only; rule + price) | Hits deal +40%; price **Fragile**. Tag cooldown 2 s; price **Slow**. +2 shared stocks; price **Hurried** (shifts every 20 s). Second Wind twice a fight; price **Weak**. Marked lasts 8 s; price **Vulnerable**. Fresh never ends while the pool is full; price **Bleed** (lose 3% on each tag). |

The old spec's test is kept as the pool's acceptance test: a tester can name five two-crest pairs and two three-crest
chains nobody wrote as a set (spec 8 [R]). Here the obvious ones: Shift-Marked + Fresh-damage; Bench-heal + Fresh-6s;
Opener-shift-early + any SHIFT crest; Burning from two sources + "Last stock +25%".

### 2.4 Heavy crests and the price list

The only trade-off object. Earned at a boss (fight 5 today; every boss once runs are longer), one offer of three, at most two
held per run, never offered twice. The price is one of seven, each one line, learned once: **Slow** (walk/dash -15%),
**Low** (jump -15%), **Fragile** (take +25%; knockback once `gd_fighter_mod_set` exists), **Weak** (deal -20%),
**Vulnerable** (Marked lasts twice as long on you), **Hurried** (shifts every 20 s), **Bleed** (lose 3% on each tag).
Never a status on yourself (readability R9 lesson). Why keep a trade-off object at all when keystones are out: it was the one
screen that measured as a real choice (tension 48%, dead 1%) [R synergy 4], and a boss is the one moment a run can afford a
20-second decision (Hunicke: adjust between events, never inside one [V]).

### 2.5 How crests meet the new systems

| system | how a crest meets it | why this beats the drives |
|---|---|---|
| **tag** (E10, E12: own %, in place, D-pad down, 5 s cooldown, air OK) | RELAY and BENCH crests make the tag a *build* decision: tag to the fighter whose crests fit the next 3 s; the bench is a resource (heal, charge). Fresh is granted on the sim frame of the release, which is also the frame the cinematic reveals the incoming fighter on every peer (plan 3.5) | a drive fired 3.9 times per hit and could not be seen; a tag happens 2-5 times a fight, is already the loudest thing on screen, and the cinematic *is* the budget's one screen-wide effect (visual identity 0: "loudness inversely proportional to duration") |
| **shared stock pool** (E4) | POOL crests key on the pool falling and on the last stock; "+1 stock" is an offer kind, not a crest; "Pool at 1" crests are the comeback lever | the pool is the one resource both players share; crests on it make the co-op state readable ("we are at 1, Guard is on") without a second economy (the old bag) |
| **stage shift** (E16: run-triggered, every 30 s from fight 2) | SHIFT crests fire on the switch frame; two crests change the schedule itself (later / earlier); Hurried is a price | the switch is a moment both peers see on the same frame under a cover; a rule that fires there reads as the arena's doing |
| **bosses** (Petey as boss-only enemy; Hands as fallback [memory]) | heavy crests are earned only there; the boss's own crests (CPU kinds, 3.5) are shown on its card at the ready moment; a boss phase (50%, P4) is a moment POOL/OPENER-like crests may key on later | the keystone allowance grew with depth and nobody could hold it in mind; two heavy crests per run is a count a player remembers |
| **two fighters per player** (E11) | crests are per fighter; the HUD's theme line is per fighter ("FOX: Fresh 2 · MARTH: Burning 1") | gives each half of the pair an identity, which is what the old "variety without identity" lacked [R synergy 3] |
| **Second Wind** (E20) | one heavy crest doubles it; it stays a run resource, not a crest | keeps the riskiest mechanic out of the common pool |

### 2.6 Why this is better than the drives, by source

- **Rare, loud moments instead of per-hit rules.** Readability diagnosis rows 5-7 and the visual identity budget ("bursts at most
  4 a second per fighter", "one screen-wide effect at a time") [R]. A rule anchored to a tag fires at a rate the harness can
  count exactly and a player can attribute (Juul's attributable failure applies to success too [V]).
- **Measurable effects.** The synergy lane could not see a +5% payoff with n=4 fights [R synergy 5.1, 7]. A Fresh window with
  +35% damage for 3 s after an event that happens 3 times a fight is a +-30%-scale effect inside a known window: the harness
  attributes damage per window and the number is tuned to a target band (Rupp et al. 2024: simulation-balanced content is
  perceived as balanced, confirm with people [V]; Giovannetti: metrics plus judgement, never one alone [V]).
- **Per-fighter ownership.** Yannakakis and Togelius' experience-driven PCG wants the content to adapt to a model of *this*
  player [V]; with two fighters per player the crests on each are the smallest unit the model can act on, and a tag is the
  player steering between two half-builds mid-fight (Chen: adaptive choice embedded in the core activity, not in menus [V]).
- **Trade-offs at boundaries only.** Hunicke's DDA acts between events [V]; heavy crests are offered only at a boss and the
  foe budget is set at fight boundaries from the party's weight (3.5), never inside a fight.
- **A bounded build.** The Berlin Interpretation's "resource management" is kept as the pool and six slots [V]; the tier
  curve that made late builds x3.5 (and the number x169) is gone, so New Game+ growth must come from more fights and heavier
  foes, not bigger numbers (3.5).

---

## 3. Pool structure, offers, synergy, power, balance

### 3.1 Pool as data

`rust/mods/envoy_data/pool.toml` (hand-authored) -> `build.rs` -> `pool_gen.rs` (`#[repr(C)]` records, text tables, a
FNV digest) `include!`d by **both** `envoy_sim` (rules, numbers, weights, words) and `envoy_ui` (names, texts, family, word
colours). One source, two consumers, no runtime parsing in a `no_std` wasm module. The digest is a word of the sim's hash
(4.2) and of the run record, so two peers with different pools refuse each other in the camp, and a record from an older
pool is dropped with a notice (plan 3.10 rule).

**`envoy_lint`** (build-time, Sonnet): one trigger family, one effect, <= 12 words, <= 2 numbers, `gives`/`reads` from the
four words only, a HEAVY has exactly one price from the seven, no two crests with the same rule text, every `reads` word has
at least two givers in the pool, every family has at least four members. The readability pass's R1-R6 and R11 as code.

### 3.2 Offer rules (per player, at each fight boundary from fight 2; at the boss, heavy crests instead)

Three offers, rolled from `seed_for(seed, fight, player)` (the C build's rule, kept: a retry is the same, two peers agree
[R plan 1.6]):

| slot | rule | fallback |
|---|---|---|
| A **connects** | a crest whose `reads` matches a word a held crest `gives` (or whose `gives` matches a held `reads`); among those, the family the party holds fewest of | if nothing held: the family of the fighter with fewer crests |
| B **new** | a crest from a family the party does not hold yet | a random crest the party does not hold |
| C **spread or hone** | alternately: a random crest the party does not hold, or a **hone** of a held crest (+50% of its number, once per crest) | if every held crest is honed: a random crest |
| **stock** | when the pool is at 1 stock, slot C is replaced by **+1 shared stock** | the comeback valve, at a boundary (Hunicke) |

Rules that make a dead card impossible: never a crest a fighter already wears; never the same crest on both fighters (a second
copy has nowhere to go: the one place the old bag's "dead duplicate" came from); never two offers from the same family.
Who wears it: the roll names the wearer on the card (so slot A can connect across the pair), Q4. Timer 20 s, offer A on
timeout (E8). Two players: each picks with their own pad inside the same freeze; the hold releases when both have picked or
the timer ends; the UI shows the other player's cursor. No new channel: pads are already in the rollback input stream
(`envoy-1p.md`: "the reward pick is made with the synced sim pad inside envoy_sim" [R]).

### 3.3 Synergy without the readability problems

- **Synergy is a word, written twice.** A crest that gives Marked says "Marked"; a crest that reads it says "Marked". The
  card's footer says "fits: FOX's Shift-Marked" when slot A connected. No archetype table, no "assembled" toast, no link
  flash (readability 4e kept as a lesson [R]).
- **Cross-fighter synergy is the party.** Fox's tag-out crest Marks; Marth's Fresh crest hits the Marked foe. That is the
  "party as the rhythm" of Direction C and it is readable because the tag between the two halves is a cinematic.
- **The theme line.** One HUD line per fighter, counts by word ("FOX  Fresh 2  Marked 1"), nothing else persistent (plan 3.3).
- **The glossary.** Four entries, reachable from the focused card; numbers of the words live there, not in rule text (R11).

### 3.4 Power curve

| source of power | bound | NG+ |
|---|---|---|
| crests worn | 3 per fighter, 6 per player | unchanged |
| hones | one per crest, +50% of its number | unchanged |
| heavy crests | 2 per run | 3 on the second lap |
| numbers | flat, no tier | unchanged; **foes** gain weight and count instead |
| shared pool | 4 + stock offers | unchanged |

The run's growth is a staircase of at most 8 picks + 6 hones, each visible in a compare row. A pair's "weight" (sum of
crest weights) is an integer from 0 to about 18; it drives the foe budget and is never shown as a strength.

### 3.5 Opponent budget (CPU kinds) and the +25% cap

Foes are rolled to a **budget** = party weight x (1 + min(0.25, 0.05 x (fight - 1))) x role (boss 1.15, last boss 1.30),
the C build's cap kept as a lesson (win rate 50-60% through loop 2 with it [R strength 3.3]) but on an integer budget, not
a multiplicative strength. In the 1P slice foes have ratios and levels only (`SCHEDULE` [R]); when CPU crests land (4.5
milestone C5) a foe spends its budget on ALWAYS and OPENER crests first (readable at the ready moment: "this Falco opens
Fresh"), never on RELAY/BENCH (foes do not tag in v1). The card shows at most two lines (audit Q4 lesson).

### 3.6 Measurement and the simulation plan

**What exists [R `envoy-1p.md`]:** `melee play --match` headless runs a whole run: `MELEE_ENVOY_TEST_P1CPU=1` makes the
player a level-9 CPU, `MELEE_ENVOY_START_FIGHT=N` starts at fight N, pad scripts (`MELEE_PAD_SCRIPT`) press D-pad down for
tags and left/right/A for picks, `envoy:` log lines record every event (fight, tag, stock key, reward, outcome); synctest
depth 4 over 2400 frames passed; headless frames cost ~216 us (`bench-unlocked.md`).

**Added knobs (sim, Sonnet, S):** `MELEE_ENVOY_SEED=n`, `MELEE_ENVOY_POLICY=random|weight|connect` (the sim picks for the
CPU player: random; highest weight; the connecting offer), `MELEE_ENVOY_TAG_EVERY=frames` (the CPU player tags on a
schedule; later the LAB's CPU behaviours as a second persona, Holmgård et al. [U]). Log lines gain: `envoy: fire <crest>
<fighter> <frame>`, `envoy: window <word> <fighter> <start> <end> <damage dealt in window>`, `envoy: offer <A> <B> <C>
<pick>`.

**The harness** `tools/rust-port/envoy/sim.py` (Sonnet, S): seeds x policies x personas -> one process each, **8 jobs at
most** (memory: PC memory limits), ~8 s per 5-fight run -> 600 runs in about 10 minutes; a CSV per run and a report:

| metric | definition | target band | source of the band |
|---|---|---|---|
| win rate per fight index | CPU-player wins / runs reaching that fight | 0.40-0.60 for fights 1-4, 0.30-0.50 for the boss | Rupp et al. 2024 [V]; the C build's 50-60% [R] |
| pool left at the boss | median stocks on entering fight 5 | 1-2 | [J]: the boss must be a fight |
| fires per family per fight | count of `fire` lines | RELAY 2-5, SHIFT 1-3, POOL 0-2, OPENER 1, BENCH 1-3 | [J]: below 1 a family is dead weight; above 6 it is per-hit noise again |
| window uplift | damage per second inside Fresh / outside, per crest | +25% to +60% | synergy 6 proposal 2 ("+20 to 30% on uptime") [R], raised because the window is short |
| offer quality | dead (nothing better than held: must be **0** by rule), coin flip (top two within 5% of expected uplift), tension (weight-best differs from connect-best) | dead 0; coin flip < 25%; tension > 35% | synergy 4 definitions [R] |
| pair completion by the boss | runs holding a gives/reads pair | > 60% for `connect`, > 35% for `random` | synergy 3's 30-72% for the webs that worked [R] |
| tags per fight | tag count | 2-4 | [J]; also a check that the schedule knob is sane |
| run length | frames | 5 fights in 6-10 minutes | Classic's length (plan 2.4) |

Each balance change is checked against the CPU proxy **and** the owner's own recorded play (E25: CPU-tuned first, then
corrected), never one alone (Giovannetti [V]; synergy 7 "a CPU is not a person" [R]). The report is attached to every balance
hand-back, like the gallery sheet for effects (plan 3.12c).

---

## 4. Implementation in the current architecture

### 4.1 Who holds what

| piece | module | notes |
|---|---|---|
| pool records, texts, digest | `envoy_data` (generated, `include!`d by both) | 3.1 |
| crest slots, windows, statuses, offers, picks, moment detection, foe budget, run record, hash digest | `envoy_sim` (hooks-v1, `frame.run_procs` post as today) | all in linear memory or engine statics, rolled back |
| camp scene, roster scene (S2), HUD, cards (then pickups), compare rows, glossary, theme line, the other player's cursor | `envoy_ui` (gd-ui-v1) | reads the view; never writes the sim; the arm/record flow goes through host imports |
| arm flag, record persistence, Solo-menu row, browser MODES row | host: `melee_modes::tu::envoy`, `rust/melee/src/{ui,browser}.rs`, `melee_modhost` | 1.3 |

### 4.2 Sim state (deterministic, rollback-safe, hashed)

```
Run      { state, seed, fight, pool, pair[2], picks, record_digest }                      (~40 B)
Player x2 { crests[2 fighters][3] (id u8, honed u8), heavy[2], weight }                  (2 x 16 B)
Fighter x (4 slots x 2) { fresh_until, benched_since, marked_until, burning_until, burn_tick }  (8 x 20 B)
Match    { seq, frames, tag_cd, stage_clock, outcome, last_stocks[4] }                     (~32 B)
Offer x2 { active, offers[3] (id, wearer), cursor, timer, picked, last_pick, since_pick }  (2 x 24 B)
```

Under 1 KiB; the pool is `.rodata`. Every frame the mod folds **an FNV-1a over the whole struct** into `gd_hash_word` (the
host mixes it into the compared netplay hash [R `melee_modhost` 661]). Today's slice hashes five fields packed into one word
[R]; a divergence in an unhashed field (an offer id, a window end) would surface only when it changed a fighter. The full
digest closes that (the C build's exact lesson, plan 3.2).

Writes to the game are all through existing paths that are game state and so roll back: `gd_envoy_apply` (ratios, stocks,
`co_attrs`), `gd_fighter_set_percent`, the tag and hold imports. Windows re-apply at their start and end frames; a rollback
that undoes a tag undoes the window and its writes together because both are re-simulated from the snapshot.

### 4.3 Engine imports: exist, new small, new later

| need | import | status |
|---|---|---|
| ratios, stocks, weight/speed/jump on both fighters | `gd_envoy_apply(slot, which, v)` | exists [R] |
| heal / Burning ticks / percent on the bench | `gd_fighter_set_percent`, `gd_fighter_from_slot`, the sub-fighter handle | exists; the bench handle needs `gd_fighter_from_slot_sub(slot)` or `gd_tag_partner_handle` (**new, S**) [I] |
| moments | `gd_transition_*`, `gd_player_stocks`, `gd_envoy_seq`, `gd_fighter_get_percent` | exist |
| picks with two pads | `gd_pad_pressed(port)` | exists |
| intangible frames on tag-in | `gd_fighter_set_intangible(h, frames)` | **new, S** (today only `gd_fighter_write` raw) |
| arm, record | `gd_envoy_armed()`, `gd_envoy_record(out, cap)` | **new, S** (1.3) |
| view, more words | `gd_envoy_publish` / `gd_ui_envoy` with `VIEW_WORDS` 64 -> 160 | exists; widen (S) |
| "the foe you last hit", "first hit while Fresh", foe crests that read hits | `gd_hit_view` + wire `hit.fighter_process_hit` | **new, M**, plan 4.2 item 7 (Opus: semantics) |
| knockback prices and Marked as knockback | `gd_fighter_mod_set/get` | **new, M**, plan item 7 |
| pickups drawn in the world | `gd_ui_project(x, y, z, out[3])` (presentation; camera read-only) | **new, S** (section 5) |
| straight from the camp into a fight, no SSS | `gd_match_setup` / `gd_ui_match_start` | **new, M**, plan item 5 (stub today [R]) |
| per-peer files | `gd_data_read/write` | later; the host-side record save (1.3) is enough first |

### 4.4 Replacing the six placeholder upgrades

| today [R `envoy_sim`] | becomes |
|---|---|
| `UPGRADES: [(stat, step); 6]`, `N_UPGRADES`, `UP_STOCK` | `envoy_data::POOL` records; the five stat upgrades are ALWAYS crests with `wearer`; "Extra stock" is the **stock** offer kind of 3.2 |
| `RUN.stats[5]` (party-wide) | `Player.crests[fighter][3]`; ALWAYS numbers applied per fighter via `gd_envoy_apply` on that fighter's handle (today `apply` writes both tag fighters [R]: it gains a `which_fighter` bit, S) |
| `roll_offers()` (3 distinct ids) | the A / B / C / stock rules of 3.2 over the pool, per player |
| `compare(id)` (before -> after of a stat) | per record: ALWAYS shows the stat row as now; moment crests show "fires on: tag in" and the number; a hone shows number -> number |
| `pick(id)` (apply at once) | ALWAYS: apply at once to the wearer; others: store; `reward_frame` runs per player |
| `VIEW[13..22]` three offers + compare | offers x 2 players with wearer and connect flag; crest slots; windows; theme counts |
| `envoy_ui` `UP_NAMES/UP_STAT/UP_TEXT` | `envoy_data` text tables; the card gains a wearer line and a "fits:" footer; the HUD gains the theme line and a Fresh/Benched indicator per fighter |

Nothing of the transition, hold, plan or outcome code changes.

### 4.5 Milestones (S = days, M = 1-2 weeks, L = 3+; model per memory: Opus for UI / advanced, Sonnet for plain work)

| # | milestone | playable at the end | size | model |
|---|---|---|---|---|
| **A1** | **The door.** `gd_envoy_armed`, host arm flag, the 1-P submenu replacement with the Envoy row (way (a); a one-day probe of `mnmain.rs` first decides (a) or (b)), browser MODES row made available + LAUNCH route, camp scene (Continue / New / Party / Abandon), the camp replaces the CSS while armed, record save/restore (`gd_envoy_record`, host-side file), S1 party under the camp, RUN CLEAR / OVER in the camp, VS retail when not armed | Envoy started, continued and quit from the game; VS unaffected | M | host + sim: Sonnet; submenu and camp scenes: Opus |
| **C1** | **Pool as data.** `envoy_data` + `build.rs` + `envoy_lint`; the five ALWAYS crests and the stock offer replace the six upgrades (same behaviour); per-fighter wearer; `VIEW` widened; cards show wearer and compare rows | identical play, new structure | S | Sonnet |
| **C2** | **Moments.** Fresh / Benched / Marked / Burning; RELAY, BENCH, SHIFT, POOL, OPENER families from 2.3 (minus the two hit-reading texts); windows via apply; `gd_fighter_set_intangible`, the bench handle; the full FNV digest in `gd_hash_word`; synctest depth 7 across tags, shifts and picks; goldens | builds that fire at moments, hashed | M | Sonnet (sim, tests); Opus reviews the window semantics |
| **C3** | **Offers and two players.** A / B / C / stock rules, hone, connect footer, two pads in one freeze, the other cursor, theme line, glossary; the 2-player S1 mapping | two players pick their own crests online-shaped (loopback) | M | Sonnet (rules); Opus (cards, theme line) |
| **C4** | **Harness and first balance.** Knobs, log lines, `sim.py`, the report; tune numbers to the bands of 3.6 | numbers with a reason | S-M | Sonnet (harness); Opus reads the report and tunes |
| **C5** | **Heavy crests and foe crests.** Heavy offers at the boss, the seven prices; foe budget; CPU ALWAYS/OPENER crests; the opponent card at the ready moment | the boss choice; foes that read | M | Sonnet (data, budget); Opus (card) |
| **A2** | **Roster screen (S2)** replacing the CSS while armed; offline first, then lockstep | the pair picked on an Envoy screen | M | Opus |
| **N1** | **Plan in the handshake.** `envoy=` token (armed, seed, digest), camp online rows, synctest 2-peer over a run | two people play a run | M | Sonnet (token, tests); depends on the room flow owner |
| **H1** | **Hit view** (`gd_hit_view`, wire `hit.fighter_process_hit`) and `gd_fighter_mod_set`; the two hit-reading crests; knockback prices | the pool complete | M | Opus (semantics), Sonnet (wiring) |
| **D1-D3** | the diegetic flow of section 5 | see 5.4 | S to L | see 5.4 |

Order: A1, C1, C2, C3, C4 (the playable core, all Sonnet-heavy), then C5 / A2 / N1 in any order, H1 when plan item 7 lands.

---

## 5. Diegetic run flow (secondary): fewer menus, more world

### 5.1 What the research says, in one paragraph

Fagerholt and Lorentzon's taxonomy (diegetic, spatial, meta, non-diegetic) is the vocabulary [V: Chalmers MSc thesis, 2009].
Jørgensen argues the diegetic / non-diegetic split is too coarse for games: interface information is part of the gameworld
whenever the player needs it to act, and the question is *integration*, not purity [V: *Gameworld Interfaces*, MIT Press
2013]. Llanos and Jørgensen's player study found players prefer integrated interfaces **when they cost nothing in
usability**, and accept a HUD for information that must be exact [V: DiGRA 2011]. Iacovides et al. removed the HUD from a
shooter and reported effects on involvement that depended on expertise [V: CHI PLAY 2015; the direction of the effect per
group is **[U]**, re-read before leaning on it]. Peacocke et al. compared HUD and in-world ammo displays on performance [V:
IEEE GEM 2015; result **[U]**]. The practical rule we take: **move the run's structure into the world; keep exact numbers
(stocks, percent, the timer) where they are**; let the world carry *what comes next, what you may take, where you may go*.
Run-structure precedents (design observation, not papers): Hades shows the next room's reward on the door before you choose
it; Slay the Spire's map is the route (Giovannetti [V]); Melee's own **All-Star rest area** is a retail stage where the
next opponents stand as trophies and heart containers lie on the ground, between fights, with no menu [R: retail content].

### 5.2 What the engine can and cannot do for this today [R]

| can | cannot (yet) |
|---|---|
| switch the stage under a 90-frame freeze and cover inside a match (`stage_hop`, `gd_stage_transition`; retail DATs that `st_retail_supported` allows) | spawn or tear down a fighter mid-match (plan 3.5: "heavier and unproven") |
| freeze the sim with a hold and read pads while frozen (`gd_hold_begin`, `gd_pad_pressed`) | items in a rollback match: "CPUs/items in the scene" not ported for netplay (`net.md` 59); CPUs since passed (E2), items **[U]** |
| put a fighter to sleep and wake it (the tag partner path: `Fighter_Create` sleeps the sub-fighter, the swap wakes it) | sleep a *CPU slot's* fighter on request (no import; the mechanism exists for sub-fighters only) |
| rewrite the next match from a plan (`match_begin`) | chain matches without the results / CSS / SSS (needs `gd_match_setup`, or scene replacement of each) |
| read fighter positions (`gd_fighter_get_pos_*`) and draw overlay quads | project a world point to the screen from a presentation mod (`gd_ui_project`, new, S) |

### 5.3 Three directions

| | **D1. One long match** | **D2. Chained matches with a rest stage** (recommended) | **D3. Arena pickups, no hub** (first step) |
|---|---|---|---|
| the run is | one VS match: the party, three CPU slots (asleep until their fight), the hub and the arenas all stage switches | a chain of short matches, no menus between: fight -> cover -> rest stage (a match with no foes) -> cover -> fight | a chain of fights; rewards appear in the arena after the last KO; the route is which reward you take |
| moving between fights | a stage switch into the rest stage; walking to an exit switches into the next arena and wakes that fight's CPUs at the spawns | the rest stage is its own match; walking into an exit zone (both players standing in it, or 20 s then the host's) ends it and the next match starts under the cover | the last KO starts a 15 s collect hold on the same stage; the next match follows |
| rewards | objects on the rest stage; walk into one and press A (zones first, items when proven online) | same | same, on the arena |
| party pick in world | the rest stage before fight 1: S3 recruit, or S2 | same | S1 / S2 before the chain |
| HUD | retail stocks and percent stay; the pool shows as lit pedestals on the rest stage; tag cooldown as a ring at the feet (projected quad); crests as small marks in the tag cinematic only | same | same, minus the rest stage |
| needs | sleep / wake a CPU slot (**new, M-L**), suppress the retail "one team left" match end while the hub is up (**new, M**, risky), re-stock CPUs between fights (`Player_SetStocks` exists), the rest stage DAT allowed by `st_retail_supported` [U], six opponent characters per run (3 slots x main + sub) | `gd_match_setup` to chain matches without menus (plan item 5), the rest stage DAT allowed [U], `gd_ui_project`, zones in `envoy_sim` | `gd_ui_project`, the collect hold (exists as KIND_HOLD) |
| rollback | everything is match state; the hub is just more frames | each match is a match; the rest stage is one too | same |
| size, model | L, Opus (engine: fighter sleep, match-end rule) | M, Opus (flow, rest stage) after `gd_match_setup`; Sonnet for zones | S-M, Sonnet (zones, hold), Opus (projection draw) |
| risk | the match-end rule and sleeping CPUs' AI/camera under rollback: unproven | the results screen and the SSS must both be skipped; the rest stage match must start with items off and no CPUs (the plan does this) | the pickup moment adds 15 s per fight; fine if the objects are worth looking at |

**Recommendation: D3 now, D2 when `gd_match_setup` lands, D1 never unless D2 feels like loading screens.** D3 brings back
the one thing the C build's players liked about loot (the floor pickup, spec 5 [R]) with no engine risk and replaces the card
overlay; D2 gives the rest stage, the route as a door and the partner as a recruit; D1 is the purest and the most expensive,
and its gain over D2 is a cover you do not see.

### 5.4 What the world carries (and what stays on the HUD)

| information | world | HUD |
|---|---|---|
| next fight / route | exits on the rest stage, each marked by its reward kind (Hades' door symbol as an idea) | none |
| rewards | objects at fixed points (zones), three per player, colour-safe silhouettes (section 6) | the compare row appears only while standing on one |
| shared pool | lit pedestals on the rest stage; in a fight the retail stock icons (exact numbers stay, Llanos and Jørgensen) | retail |
| tag cooldown | a ring at the active fighter's feet (projected quad; later `gd_fx_*`) | the "TAG READY" line goes |
| crests worn | marks in the tag cinematic's name card; the theme line only on the rest stage | theme line in fights until the owner has seen it both ways |
| stage shift | the last 3 s before a shift: the cover's colour creeps in from the edges (presentation, from `v[27]`) | the "STAGE SHIFT IN" line goes |
| run end | the rest stage with no exits and a closed gate; RUN CLEAR: an open gate | the camp line |

### 5.5 Pickups and route as sim zones (D3 / D2 mechanics)

A **zone** is a sim-side rectangle on the stage (x range, platform y) with a payload (a crest offer or an exit). `envoy_sim`
checks `gd_fighter_get_pos_x/y` of each player's active fighter each frame; standing inside for 30 frames and pressing A takes
the offer (the wearer is the fighter standing there: the diegetic answer to Q4). Zones are rolled with the offers from the
same seed, so both peers agree; the UI draws each zone through `gd_ui_project` as a projected quad with the crest's glyph. No
items, no spawns, nothing new in the rollback set. Real items (the engine's pickup, with hands and a throw) come later if the
items-under-rollback probe passes; the zone stays as the fallback.

---

## 6. Presentation: directions only (no art, no old models)

Rules carried over from plan 3.12: composable primitives, one core technique per set, two named references per effect,
the gallery contact sheet before anything is "done", the owner picks per round, and no reuse of the `envoy_drives` models.
The grammar: WHO (silhouette and colour), WHAT KIND (carrier), WHEN (motion state), HOW MUCH, WHERE (visual identity 0 [R]).
Family must read in greyscale (silhouette, not colour, carries it).

| | **P1. Standards** | **P2. Lanterns** | **P3. Tablets** |
|---|---|---|---|
| the crest as an object | a small banner on a short pole, planted in the floor; the family is the banner's cut-out glyph; a heavy crest is a taller, darker standard with a second pennant (the price) | a floating light the size of a fist, a glyph lit from inside; heavy: two lights bound by a thread | a slab that rises from the stage, the glyph carved; heavy: a cracked slab |
| pickup moment | the banner unfurls as you approach and is taken down as you take it | the lantern drifts to the fighter's hand and goes out | the slab sinks; the glyph stays as a floor mark for a second |
| worn (tag cinematic name card) | a row of tiny pennants under the name | three points of light orbiting the name | glyphs stamped on the card's edge |
| core technique (one each; plan 3.12c) | cloth: vertex displacement by a flow field | SDF glyph with an additive core and a soft occlusion test against scene depth | erosion/dissolve mask on a solid, normal-lit |
| references (ideas only, credited) | banners as loot markers in strategy games; signal flags | Hades' boon orbs (an orb per god, glyph inside); Journey's glyph lights | Shadow of the Colossus' sigils; Breath of the Wild's shrine pedestals |
| HUD glyph | the cut-out as a flat icon | the glyph as a flat icon | the carving as a flat icon |
| cost | geometry from the owner's Blender parametric pipeline (memory); one shader | one SDF shader, no geometry | geometry (one slab) and one mask shader |

The three differ in mechanism (cloth, light, stone), so no two read alike at a glance; within a direction the seven families
differ by silhouette (glyph) and the four words by colour. **Questions for the owner (section 7, Q6-Q7):** which direction;
whether crests are ever visible on the fighter in a fight (recommend no: the tag cinematic and the pickups are their stage;
visual identity's "passive rules get no look of their own" [R]); whether the family glyphs are drawn by the owner or
commissioned later (plan 3.12b).

---

## 7. Open questions for the owner (plain, few; each with a recommendation)

| # | question | recommendation |
|---|---|---|
| Q1 | Envoy's door: a row in the 1-P menu beside Regular Match (plus the browser row), opening an Envoy camp screen with Continue / New run / Party / Abandon? | Yes (1.2 option A, mod-side first). |
| Q2 | Partner pick: keep "P1 = main, P2 slot = partner" now, build an Envoy roster screen next; and should "the beaten opponent joins you" exist as a run variant? | Keep S1 now, build S2; yes to Recruit as a variant later, not the default. |
| Q3 | Crests replace drives and keystones entirely; the only thing kept from the readability pass is its limits (one sentence, few words, no status as a price, no "assembled" toast). OK, and does this close E19? | Yes; E19 stands only as those limits. |
| Q4 | Who wears a crest you take: the card names the fighter (so offers can connect across the pair), or the fighter standing on the pickup wears it once pickups are in the world? | The card names it now; the standing fighter once pickups are in the world. |
| Q5 | Heavy crests (one rule, one price from a list of seven), at most two per run, only at a boss: in? | In. It was the one choice that measured as real. |
| Q6 | Rewards in the world: cards now, then objects on the arena after the last KO (D3), then a rest stage between fights (D2)? | Yes, in that order; D1 (one long match) only if D2 feels like loading screens. |
| Q7 | Pickup look: standards (cloth), lanterns (light) or tablets (stone)? And crests never visible on the fighter during a fight? | Lanterns first for the gallery (no geometry needed), standards second; not visible in fights. |
| Q8 | The HUD in a fight: drop the TAG READY and STAGE SHIFT lines once the world carries them (ring at the feet, cover creeping in), keep the retail stocks and percent? | Yes. |

---

## 8. Sources

Plan section 6 sources used here [V unless marked]: Togelius et al. 2011 (generate-and-test: the harness); Shaker, Togelius,
Nelson 2016; van der Linden et al. 2014 (control over geometry: zones and exits, not mazes); Yannakakis and Togelius 2011
(experience-driven PCG: per-fighter crests, the profile later); Hunicke 2005 (adjust at boundaries: offers, heavy crests,
foe budget); Chen 2007 (choice inside the activity: the tag as a build decision); Juul 2013 (attributable failure: the camp
names what ended the run); Berlin Interpretation (resource management: the pool and six slots); Giovannetti GDC 2019
(metrics plus judgement); Yu 2016 (hand-checked templates: every crest played in isolation by the harness); Rupp et al. 2024
(the 0.4-0.6 band, confirm with people); Holmgård et al. 2018 [U] (two personas).

Added for sections 5-6: Fagerholt and Lorentzon, *Beyond the HUD* (Chalmers, 2009) [V]; Jørgensen, *Gameworld Interfaces*
(MIT Press, 2013) [V]; Llanos and Jørgensen, "Do Players Prefer Integrated User Interfaces?" (DiGRA 2011) [V]; Iacovides,
Cox, Kennedy, Cairns, Jennett, "Removing the HUD" (CHI PLAY 2015) [V, findings U]; Peacocke, Teather, Carette, MacKenzie,
"Evaluating the effectiveness of HUDs and diegetic ammo displays" (IEEE GEM 2015) [V, findings U]. Design observations
(shipped games, ideas only, to be credited in CREDITS.md): Hades (door rewards, boon orbs), Slay the Spire (the map),
Journey (glyph lights), Shadow of the Colossus and Breath of the Wild (sigils, pedestals), tag windows in team fighters
(Marvel vs. Capcom, Dragon Ball FighterZ: tag-in as a moment with rules), Melee's All-Star rest area (retail content).

Old Envoy documents read for lessons only: `_research/envoy-readability-pass1-2026-10-05.md`,
`_research/envoy-synergy-analysis-2026-10-05.md`, `_research/envoy-strength-decision-2026-10-07.md`,
`_research/envoy-visual-identity-2026-10-05.md`, `_research/envoy-gameplay-ui-audit-2026-10-05.md`,
`docs/superpowers/specs/2026-10-04-envoy-loot-and-modifiers-design.md`, `pc/scripts/examples/envoy_drives/README.md`.

Rust port files read: `docs/rust-port/notes/{envoy-1p,modhost,wasm-ui-lab-api,menus,net,tag-fighter}.md`,
`docs/specs/{gdmod.h,gd-ui-lab-v1.h,hooks-v1.tsv}`, `rust/mods/envoy_sim/src/lib.rs`, `rust/mods/envoy_ui/src/lib.rs`,
`rust/melee_modes/src/tu/{envoy,tag_swap,stage_hop}.rs`, `rust/melee_modhost/src/lib.rs`, `rust/melee_modhost/src/ui/mod.rs`,
`rust/melee/src/play_net.rs`.
