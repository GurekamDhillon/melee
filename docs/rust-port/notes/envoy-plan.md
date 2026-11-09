# Envoy on the Rust port: a plan (2026-11-08)

Planning only. Nothing here was built or run. Branch `port/envoyplan`, worktree `worktrees/envoyplan`, off `pub/rust-port`
(`db9d048e4`). Paths are repo-relative to the game checkout unless they start with `_research/`, `docs/` (workspace) or
`tools/`. Tags: **[R]** read from source or a note (cited), **[I]** inferred, **[J]** my judgement, **[U]** unverified.

Owner inputs folded in (via the coordinator, 2026-11-08): seamless **stage switching** is a primary Envoy feature and
builds on the `stage_switch` wasm mod being ported on `port/stagesw` (nothing committed there yet at `db9d048e4`
[R]); **character tag-switching** in the spirit of Joyastick's melee-pc-TagFighter (idea only, nothing copied), with the
rule that on a tag the game **pauses and a cinematic plays**, the same treatment as a stage switch, so both are one
**transition** system; and Envoy is **multiplayer-native**: designed for rollback netplay with 2+ players from the first
slice, solo being the one-peer case.

Rules that bound the plan (memory): LAB and custom menus are rebuilt as wasm mods, never the old Lua script host; the C
build is a frozen reference (features go Rust-only); no old GD art is reused; design decisions are taken with the owner one
question at a time (the questions are collected in section 7); everything stays deterministic for rollback.

---

## 1. What Envoy is today (the C build)

### 1.1 Two eras, one name

| era | dates | what it was | where it stands |
|---|---|---|---|
| **SuperTime Envoy** (mazes, genes, missions) | 2026-09-30 to 10-03 | A traversal roguelite: generated levels, a Chao-like gene economy (collection, breeding, fusion), a stage/mission builder. Lua in `pc/scripts/examples/roguelite/` (997 KB), director 1,752 lines | **Rejected by the owner** ("the level sends me back around", "I do not understand the game loop", infinite gene duplication). `docs/ENVOY-PROBLEMS-2026-10-02.md` [R]. The gene idea itself was *kept* as a later layer (`docs/PLAN-ENVOY-EDITOR-MISSIONS-2026-10-03.md` sections 10-11; `_research/chao-genetics-for-envoy-2026-10-03.md`) [R] |
| **Envoy: loot and modifiers** | 2026-10-04 to 10-08 | A roguelite on top of retail Classic and Adventure: drives (rolled loot), keystones, a bag, scaled opponents, New Game+, online adversarial sets. Spec `docs/superpowers/specs/2026-10-04-envoy-loot-and-modifiers-design.md`; mod `pc/scripts/examples/envoy/` (2.5 MB; 60 Lua files, 13,057 lines) [R] | **Shipped on by default** (README: "SOLO > ENVOY"; "public test; look and balance still being tuned"). Online: adversarial Versus sets with passive and triggered builds; "Online Envoy has only the passive drives" is the README's user-facing status [R] |

Everything below is the second era unless stated.

### 1.2 The loop [R: `envoy/README.md`, `run_host.lua`, `mod_progression.lua`, spec sections 2, 5-7]

1. Pick a fighter, start retail Classic (or Adventure) under the Envoy rule host (`envoy rules on`; the menu path
   historically started the older companion HUD instead, audit Q11 [R `_research/envoy-gameplay-ui-audit-2026-10-05.md`]).
2. Every opponent has a **build** rolled from the same pool, targeted at the player's build strength x (1 + min(.25,
   .01 x effective depth)) x role (boss 1.15, last boss 1.3); the roll accepts [.8, 1.2] of the target
   (`_research/envoy-strength-decision-2026-10-07.md` section 1) [R]. A nameplate shows the opponent's modifiers.
3. Fight. Statuses, triggers, crits and technique rewards fire from a Lua event engine (`mod_engine.lua`, 806 lines) with
   looks on the fighters (surface shader, afterimages, tracers, link flashes) [R audit section 1a].
4. Opponents **drop drives** on the floor; a stage-end hold ("Collect the drives") lets you pick them up; the bag (4) auto-merges
   duplicates; the build has 4 slots, 6 from depth 10 [R synergy analysis section 1].
5. Every third stage a **reward screen**: pick one of three drives (or keep). At effective depth 5, 10, 15, ... a **keystone**
   screen: one of three, permanent for the run, with a drawback [R strength decision section 5].
6. Beat Master Hand: **New Game+**, same fighter and build, deeper tiers [R README].
7. Save: the run profile (`save.lua`, format ENVOY4) and, online, the `RN2` run record [R `envoy/ONLINE.md`].

### 1.3 Systems and data (the Lua mod)

| group | files (lines) | does |
|---|---|---|
| modifier engine | `mod_schema` 205, `mod_registry` 64, `mod_pool` 79, `mod_engine` 806, `mod_status` 63, `mod_skill` 78, `mod_techniques` 64, `mod_tuning` 76, `mod_codec` 161, `mod_budget` 251, `mod_graph` 228, `mod_progression` 191 | records = `{trigger, conditions, effects, tags, tiers, text, visual}`; event matching, statuses and stacks; the strength number; the synergy graph; the online set as pure functions |
| loot | `drive_loot` 131, `drive_bag` 168, `drive_merge` 79, `drive_text` 232, `drive_drop` 235, `drive_models` 146, `drives` 106, `keystones` 261 | roll, name, bag, merge, text, floor drops, models |
| opponents | `foe_roll` 222, `foe_driver` 227, `foe_lab` 213 | roll a build to a strength target; drive the CPU's technique (`gd.cpu_assist`); the opponent plate |
| run | `run_host` 1182, `run_screen` 497, `run_hud` 173, `classic` 260, `retail_app` 480, `app` 341, `run` 214 | the 1P adapter: stage hooks, holds, rewards, keystones, NG+, HUD |
| Atlas screens | `atlas_bag` 250, `atlas_reward` 224, `atlas_swap` 176, `atlas_hud` 170, `atlas_setup` 134, `atlas_kit` 133, `atlas_results` 120, `atlas_pause` 108, `atlas_netpick` 94 | the `gd.ui` versions, behind `uxatlas on` |
| looks | `synergy_fx` 427, `earned_fx` 177, `visual`, `recolour` 109; `shaders/modifiers_surface.wgsl`; `fx/DriveLoot_*`, `fx/PickupJuice_*`; `items/drive` | fighter surface layers, tracers, afterimages, link flash, drop effects |
| co-op | `coop` 361, `coop_synth` 233 | two local players, one team, drop ownership, team scaling (`COOP.md`) |
| older layer | `companion` 185, `genetics` 58, `hub` 72, `campaign` 58, `hud` 59, `save` 195 | the companion stats, garden, DNA: the first era's remnants, still wired on the menu path |

Pool after the readability split: **72 records, 34 passive, 38 triggered**; 5 learnable statuses proposed (Burn, Chill, Haste,
Guarded, Marked) plus a Momentum counter and a private Shock [R `ONLINE.md` "Which records are online-safe";
`_research/envoy-readability-pass1-2026-10-05.md` section 0].

### 1.4 Native side (the part that is not Lua) [R `ONLINE.md`, `gw_matchbuild.h`, `script_mods_core.h`]

- `pc/gameworld/script_build.inc`: the applied build table, in snapshotted memory, one word in `RB_GameHash`.
- `pc/gameworld/script_mods_core.h` + `script_mods.inc`: the **native triggered evaluator** (stage 4, 2026-10-08): the Lua
  compiles equipped records into a numeric program, C runs it inside the simulation, one tick per logic frame, also on
  resimulated frames; state in game BSS; its own hash word. Parity test replays Lua-generated event streams natively.
- `pc/platform/gw_matchbuild.h`, `gw_script_netbuild.inc`, `gw_netplay.c`, `gw_net.c/.h`: protocol 5, mode word, lobby
  reward phase, `E`/`W`/`X` messages, `envoy=` scene token.
- `pc/platform/gw_netrun.h`: `RN2` run record (seed, stage, loop, scores, flags, picks, stocks, continues, digest).
- Stage slots and switching (a separate feature Envoy did not yet use): `pc/gameworld/script_stage_slots.inc`,
  `script_stage_slot_carry.inc`, `pc/platform/gw_script_stage_slots.inc`; Lua `gd.stage_slot_load`, `gd.stage_switch(slot,
  {transition='wipe'|'flash'|'morph', place='keep'|'nearest'|'spawns'})`, `gd.stage_queue*`. **Offline only** ("stage
  slots require an offline gameplay match (no rollback/replay)"), save/load/rewind across a switch refused, six legal retail
  DATs [R `gw_script_stage_slots.inc:28-47`, `stage_switch_demo/README.md`].

### 1.5 Netplay status [R `ONLINE.md`, `_research/envoy-netplay-scoping-2026-10-05.md`]

| slice | state |
|---|---|
| adversarial Versus set, 2 humans, private room, passive builds, host-validated reward pick between games (stages 2-3) | done, loopback proof |
| triggered builds online through the native evaluator (stage 4) | done, loopback proof, parity test |
| run record, resume, abandon, continue offline, co-op flag (stage 5) | done, 16/16 loopback set proof |
| co-op online (CPUs in a netplay match, teams) (stage 6) | **not done; CPU AI exactness under rollback never proven in the C build** |
| online Classic director (two humans play Classic rows, shared stock pool, no builds, no CPUs) (stage 7a/b) | done as a slice; exit barrier "safely aborts" on a pending rollback correction |
| over the internet, by two people | never |

### 1.6 What worked and what did not (per the research notes)

Worked [R]:
- The **floor drop and pickup** ("felt nice", spec section 5); the **drive models** ("10/10 S+, charming", visual identity
  section 0); the **link flash** ("awesome") and fighter tints (audit 1a #12, #16).
- **Opponent scaling holds**: win rate 50-60% through loop 2 with the +25% edge cap (synergy analysis 5.3; strength decision 3.3).
- **Keystone screens are real choices** (tension 48%, dead 1%) (synergy analysis section 4).
- The native evaluator design: compile in Lua, run in C, hash word from day one, parity test (ONLINE.md stage 4).
- Rolls as pure functions of `seed_for(seed, stage, loop, port)`, so a retry is the same and two peers agree (scoping 0.6).

Did not [R]:
- **Too much at once.** 86 pieces, a rare drive carrying 4.2 rules and 50 words, 7 statuses with shared looks, up to 12
  keystones, three toasts queued for 18 s, a "Modifier!" flash on every fire (readability pass section 3; audit section 2a).
- **Synergy fires but cannot be felt**: the damage uplift of a completed pair over the sum of its halves is inside fight noise for
  every pair; only a DoT keystone moves the fight, and it does so alone (synergy analysis section 5.1).
- **The strength number misleads**: Echoes counted x2.20, measured x1.05; Gambler x0.94 vs x1.25; strength is a progress meter,
  not a difficulty meter, and it was shown as `+16,800%` (strength decision sections 0, 2).
- **Drive screens are mostly not choices**: 17% dead, 44% coin flips; technique, shock and armour archetypes complete in 4-9% of
  12-stage runs (synergy analysis sections 3-4).
- **Debug UI leaked** into play; two different HUDs depending on how the run was started (audit sections 0, 3).
- **Hub/gene/maze era** confused the owner and was dropped from the default path (ENVOY-PROBLEMS).
- **The CPU cannot stand in for a person** on technique (an L-cancel after a hit once per 450 s), so every technique number is a
  lower bound and the owner's own counters were never collected (synergy analysis section 7).
- Netplay: the scene flow of Classic/Adventure under a networked session is "not a small step" and was never attempted beyond the
  director slice; CPUs online are unproven (scoping section 4; ONLINE.md 7a/b).

---

## 2. A reimagined Envoy

### 2.1 What the Rust engine offers that the C build did not have

| strength | fact [R] | what it buys Envoy |
|---|---|---|
| bit-exact deterministic sim, hashed per frame (NHV1) | STATUS.md orientation; `notes/golden.md` | any rule that lives in the sim is netplay-safe by construction; a divergence is caught the frame it starts |
| wasm sim mods roll back for free | `modhost.md` "Rollback": host state lives in `GameScope`, snapshotted page by page; mods have no save/load code | the whole Envoy rules engine can be one wasm module with zero rollback code |
| presentation mods separate from the sim | `wasm-ui-lab-api.md` section 0: a second store, harness-owned, never snapshotted, may use wall time | HUD, menus and cinematics can be rich and non-deterministic without touching a hash |
| cheap snapshots and a rewind ring | `lab-ui-spec.md`: slot save 1.8 ms, load 1.4 ms, ring capture 1.25 ms every 8 frames, 600-frame depth; online LAB applies rewind on both peers (lockstep commands, 0 desyncs) | run checkpoints that resume *mid-fight*; rewind as a spendable mechanic (section 2.4) |
| headless speed | `bench-unlocked.md`: about 9x cheaper per sim frame than a static recomp; the LAB harness measured 216 us/frame headless | the generator can play a candidate encounter to the end with CPUs and keep only good ones (section 6) |
| rollback netplay with rooms, lobby, CSS netplay, spectate, lockstep menus | `net.md` sections on `--np-css`, `np_lockstep.rs`, `spectate-replay.md` | between-stage screens run in lockstep like the netplay CSS; a spectator can watch a run |
| Geno defines (data-defined fighter states and articles) | `notes/geno.md`, `docs/specs/geno-v5.md` (defines not yet hosted in Rust) | elite opponents and "echo" variants with altered specials as data, no code |
| stage slots and switching as a wasm mod | `port/stagesw` (in progress) | the arena changes under the fight |

### 2.2 Three directions

All three keep what worked (drops, drive models, scaled opponents, keystone choices, seeded rolls, bite-size rules) and drop
what did not (the maze era, the gene economy as a required layer, stacked affixes, the strength percent, announcement spam).
All three are multiplayer-native: the run is a sequence of **encounters**, every encounter is a match both peers agree on, and
solo is the one-peer case.

**A. Envoy Set (versus run).** Two to four players, each with a build. The run is a best-of set of encounters; between
encounters each player takes a reward from their own seeded offer; opponents are the other players. Stage switches are
rule-driven (a keystone "the arena turns every 90 s", a stage pick as a reward) and tags are a party mechanic (section 3.6).
No CPUs anywhere.
- For: nothing unproven is needed (the C build's online slices did this; the Rust netplay already runs human-vs-human with
  lockstep menus). First online play in weeks.
- Against: it is a fighting-game set with loot, not a roguelite; no run "against the game", no progression toward a boss; the
  owner's "Envoy" word has always meant a journey.

**B. Envoy Expedition (co-op gauntlet).** One to four players on one team against rolled CPU opponents through a run of
encounters to a boss, with shared stocks, per-player builds, a shared keystone vote. Stage switches are the room transitions
of the expedition; tags are each player's party of two fighters.
- For: the roguelite the owner described; co-op is what he asked for first ("two player, cooperative or adversarial").
- Against: **CPUs in a rollback match are unproven in both builds** (`net.md` "Not ported: CPUs/items in the scene";
  scoping 11). Without them nothing plays online. One feasibility probe decides the schedule.

**C. Envoy Relay (party roguelite, encounter-agnostic).** Each player brings a **party of two fighters** (the tag pair) and a
build per fighter. The run is a seeded graph of encounters of several *kinds*: a CPU fight (needs B's CPUs), a duel between
two players (A), a team fight vs CPUs (B), a survival room (stage switches every N seconds), a boss. The run engine does not
care which kind an encounter is; each kind is a `match setup` both peers agree on. Rewards, keystones, saves, transitions are
one set of rules over all kinds.
- For: ships with A's encounter kinds first (human vs human) while the CPU probe runs; adds B's kinds the day CPUs are proven;
  the tag pair and the stage switch are the *rhythm* of every encounter rather than features bolted on; the roguelite
  identity (a journey to a boss, permadeath of the run, a persistent profile) is intact from the first slice because the run
  graph, rewards and saves exist from the start.
- Against: more design surface at once (encounter kinds, party builds); the versus-only first slice can read as A until CPUs land.

### 2.3 Recommendation: C, with a strict order

Build **C**, with the run engine, the transition system, rewards and saves specified once for all encounter kinds, and ship
encounter kinds in this order: (1) duel and 2v2 between humans (A's kinds), (2) CPU fights and team-vs-CPU (B's kinds) as soon
as milestone 0's CPU probe passes, (3) survival rooms and the boss. Reasons: it is the only direction where the first
playable slice already *is* the final game's structure (so nothing gets rebuilt), it gives the owner the co-op roguelite he
asked for without betting the schedule on one unproven feature, and the tag/stage transition is load-bearing in every kind.
Solo is a one-peer run whose encounters are all CPU kinds; that is why milestone 0's probe comes first.

### 2.4 The design in one page (what a player meets)

- **Party.** You bring two fighters (a tag pair, section 3.6). Each has its own four drive slots; keystones belong to the
  party. Tagging is part of play (D-pad down by default; owner question 7.12).
- **Drives** are one-bite rules (readability pass R1-R11): one rule early, two from depth 5, no drawback on a drive, a keystone
  is one rule line and one price line from a fixed price list. Pool from the C build's 72, re-authored as data (section 3.2).
- **Run.** A seeded graph of 8-12 encounters to a boss (Classic's length), with a *path choice* at two or three forks (the
  Slay the Spire / Hades room choice; section 6): "a duel for a rare drive" vs "a team fight for a keystone" vs "a survival room".
  Every encounter's arena may switch mid-fight (section 3.5). Lose all party stocks: the run ends (permadeath of the run).
- **Rewards** between encounters, one screen, three offers, 20 s in lockstep; one offer always *connects* to something you
  hold (synergy analysis proposal 7); the strength index is shown as `x1.8`, never as a percent, and never sold as difficulty.
- **Second Wind** (uses the engine's rewind): a keystone or run resource that rewinds the fight 3 s once per encounter, for
  everyone (a deterministic lockstep command like the online LAB's; section 3.7). Owner question 7.16.
- **Opponents** (CPU kinds) are rolled to the party's strength with the +25% cap; elites carry a Geno define that changes one
  special (section 3.8). Their build is shown as a small card at the ready moment, keystones first (audit Q4; readability Q14).
- **Profile** (per player, local): unlocked drives and keystones, best runs, counters of your own technique rates (the number
  the C build never collected, section 6). **Run save** (agreed, digest-checked, a copy on every peer): resume from the last
  encounter boundary or, offline, from a mid-fight snapshot.
- **Companion / garden**: not in this plan's slices. The owner kept the idea (PLAN-ENVOY sections 10-11); it is a later
  layer on top of the party and profile (question 7.20).

---

## 3. Architecture as wasm mods

### 3.1 Three modules, one package

```
rust/mods/envoy/
  envoy_sim    (sim mod: hooks-v1, rolls back with the game, hashed)      <- rules, run state, transitions
  envoy_ui     (presentation mod: gd-ui-v1, never rolled back)            <- HUD, screens, cinematics, saves
  envoy_data/  (pool.bin, run graphs, text, looks: data compiled into both modules' .rodata)
depends on: rust/mods/stage_switch (port/stagesw) through its host API, never by duplicating it
```

The split follows `wasm-ui-lab-api.md` section 0: "UI never writes sim state unless explicitly allowed"; the LAB's sim half
(`rust/melee/src/lab.rs`) and presentation half (`rust/mods/lab`) are the precedent. One package may ship both kinds of module
("a mod may be presentation-only or have both kinds" [R A3]).

### 3.2 Sim side (`envoy_sim`): what rolls back and is hashed

| state (all in the module's linear memory, <= 256 KiB) | size [I] |
|---|---|
| run record: seed (u64), encounter index, graph path taken, loop, per-player scores/stocks pool, flags | < 200 B |
| per player (4): party (2 x fighter kind, costume), 2 x 4 drive slots (record id u8, tier u8), keystones (<= 6), bag (4) | < 100 B |
| per fighter entity (6 x [main, sub]): status slots (5 x {expires, stacks, amount, next_tick}), Momentum counter, recent-event frames (16 kinds), crit/armour flags | 12 x ~160 B |
| event queue (bounded, drained each frame), chain depth counter | 2 KiB |
| transition state: kind, phase, frames left, source/target (stage slot or party index), carry rule | 32 B |
| compiled pool (read-only `.rodata`): 72-100 records x ~64 B, text ids, look ids | ~8 KiB |

Rules evaluator: a port of `mod_engine.lua`'s `matches / conditions / apply / drain` (the C build already did this once in
`script_mods_core.h`, ~600-900 lines of C; the Rust twin is the same size). Fixed evaluation order (slot 0..5, record order,
queue order), integer and f32 only, `gd_f2i`, `gd_div`, no NaN branches (mod-c-v1 section 5). Fuel: one frame hook tick for
12 entities and a drained queue is a few thousand wasm instructions, far under the 1,000,000 per hook [I].

**Hash coverage** is the one thing the host does not give a sim mod today: linear memory is snapshotted but **not mixed into
NHV1 / the rollback hash**, so a divergence inside Envoy's state would be seen only when it changed a fighter field (the C build's
exact lesson, scoping 0.4). New import `gd_hash_word(u32)` (section 3.9) fixes that; `envoy_sim` folds a digest of its
state into it every frame.

**Events.** The vocabulary of the C spec (hit dealt/taken, KO, stock lost, shield hit, perfect shield, clank, jump, air jump,
landing, ledge grab, grab, throw, every N seconds). Sources on the Rust host [R `modhost.md` wired rows; `hooks-v1.tsv`]:

| event | source | status |
|---|---|---|
| hit dealt / taken, damage, knockback | `damage.fighter_take_damage` (wired: pre / replace / post), `hit.fighter_process_hit` (in the table, **not wired**) | damage scaling via replace is possible today; knockback and element need `hit.*` wired plus a hit-context view (new) |
| KO / stock lost | `fighter.process_death` (table, **not wired**) | wire |
| landing, jump, air jump, ledge, grab, throw, shield, perfect shield | `action.*.enter` rows (19 wired: Landing, Jump, ... ), 846 in the table | wire the few more Envoy needs; perfect shield = `GuardReflect` entry (the LAB derives it the same way) |
| clank | no row | new row or a flag in the hit view |
| every N seconds, stage start | `frame.run_procs`, `stage.on_start` (wired) | ok |
| pad input (tag request) | `gd_fighter_get_input` (exists) | ok: a tag is an input, so it is rollback-safe for free |

**Effects.** Today a sim mod can write percent, pos, vel, facing, request a handful of action states, and raw-write fighter
fields (`gd_fighter_write`). Envoy's families (damage dealt/taken, knockback dealt/taken, run/air speed, jump height, air
jumps, fall speed, weight, shield size, armour, intangibility, hitstop, element) need a **fighter-mod table** with engine
floors (the C build's `hr_percent_cap` floors damage taken at x0.15 and launch at x0.05 [R strength decision 3.2]): new
imports `gd_fighter_mod_set/get` (section 3.9). Statuses are Envoy's own state; their *looks* are the UI mod's business.

### 3.3 Presentation side (`envoy_ui`): what never rolls back

- HUD during a fight: build squares and keystone cells per party (audit 1a #1, shrunk), the synergy pill, a corner toast lane
  (one at a time, 2 s), the opponent card at the ready moment, the transition cinematic (section 3.5). Nothing wide, nothing
  over the timer, no strength percent, no "Modifier!" flash (audit section 5 answers).
- Screens: run start (party and difficulty), the reward and bag grid (MENUS.md grid, re-laid on the kit nodes), the path
  fork, results, the run map. All through `gd_ui_node*` so focus, wrap and repeat are the host's (A2).
- Looks on fighters: the C build's surface layers, tracers, afterimages and link flashes need a presentation **effects**
  API that does not exist in the Rust host yet (`gd_fx_*`, section 3.9). Until it does, looks are HUD-side (chips on the
  fighter's tag, a halo quad projected from `gd_fighter_get_pos_*`). The visual identity note's grammar (WHO / WHAT KIND /
  WHEN / HOW MUCH / WHERE, loudness inversely proportional to duration; budget 3 loud looks on you, 2 on an opponent, one
  screen-wide effect at a time) is adopted as the spec for that API [R `_research/envoy-visual-identity-2026-10-05.md` 0].
- Reads the sim through the existing read-only getters plus one new channel: the sim mod's **published view**
  (`gd_sim_view_publish` / `gd_ui_sim_view`, section 3.9). The view is a versioned struct the sim mod writes once per
  frame (statuses, build, transition, run record, last events); the UI reads it after the logic frame. Under rollback the
  view is simply the current frame's; the UI must be idempotent on it (section 3.5 says how for cinematics).
- Writes to the sim only through the **command channel** (`gd_ui_cmd`, section 3.9): a reward pick, a path choice, a
  keystone vote, a Second Wind. Commands ride the netplay stream and land on an agreed frame on every peer, the way the online
  LAB's `LabCmd`s do (`wasm-ui-lab-api.md` half B: "commands ride the lockstep stream ... host's commands first on both peers").
- Fuel 4,000,000 per present, one `gd_ui_submit` per present, no allocation: the LAB mod's budget (p99 141 us with 663 quads)
  is the bar.

### 3.4 Multiplayer model (everything is specified for N peers; N = 1 is solo)

| layer | deterministic, agreed by all peers | per-peer, presentation only |
|---|---|---|
| run record | seed, graph, path, encounter index, every player's party, builds, bag, keystones, stocks pool, scores, flags; digest | which panel is open, toasts, camera of the cinematic |
| encounter setup | `gd_match_setup` (players, kinds, costumes, stage slot set, items off, rules) agreed in the lockstep lobby phase before frame 0 | nothing |
| in-match rules | `envoy_sim` state and hash word; transitions (3.5); tag requests as pad input | looks, HUD, cinematic playback |
| between encounters | reward offers = `seed_for(seed, encounter, loop, player)`; picks and votes = commands with a host-counted tick deadline (1800 lockstep ticks in the C build); timeout = a fixed rule (take offer 0 / keep) | the screen itself |
| profile | none (never agreed; a peer's unlock set is *advertised*, section 3.10) | everything |

- **Session shape.** Rooms as today (`melee_net::app::room`, room codes, relay). The run lobby is the netplay CSS flow in
  lockstep (`np_lockstep.rs`) extended with Envoy's phases; matches run under rollback. 2 peers first; 3-4 peers when the
  transport supports more than two ports (open on the Rust side: `net.md` lists "4-player" nowhere as done [U]).
- **Joining.** Never mid-match. A new peer may join at an encounter boundary with the host's consent; they get a starter party
  and a build *scaled to the run depth* (the same roll the game gives a late-joining CPU), and the run record gains a player
  entry (question 7.6).
- **Disconnect.** Mid-match: that encounter is void for the pair (rollback cannot recover a missing peer); the run stays at
  the last boundary. The survivors may continue (the run record is complete on every peer), the leaver may **rejoin** at the
  next boundary if the host waits. A run can always be continued **solo offline** from its record (the C build's stage 5
  contract, kept) [R `ONLINE.md` stage 5].
- **Desync.** Hash mismatch ends the encounter with a message, the run falls back to its last boundary record; nothing is
  scored. With `gd_hash_word` the mismatch is caught the frame Envoy's state diverges.
- **Spectators** see the run through `--spectate` as today (presentation-only mods are free to load on a spectator).
- **Mixed progress.** The run record never depends on a profile; it carries the pool digest and every record id it uses. A
  peer whose profile has not unlocked an offered record still sees it offered *in this run* (unlocks gate what the generator
  may roll for *your* offers, not what you may hold; question 7.9). A missing optional mod (e.g. a cosmetic) is per-peer;
  the `envoy_sim` module itself is in the handshake identity (`play_net::mods_identity`), so both peers run the same rules.

### 3.5 The transition system (stage switch and tag, one mechanism)

A transition is **a sim state machine with a fixed length, and a presentation cinematic that follows it**.

Sim side (`envoy_sim`, agreed, rolled back):

```
IDLE -> REQUESTED(kind, src, dst, carry, at_frame) -> FREEZE(n frames) -> SWAP -> RELEASE(k frames) -> IDLE
```

- **Request**: from a rule (a keystone fires; the room's clock; an encounter script) or from a pad input (a tag request on
  D-pad down, read with `gd_fighter_get_input`, so it is an input like any other and rolls back like any other). Refused
  while the fighter is captured, thrown, dead or respawning (the C carry code's rule [R `gw_script_stage_slots.inc` rc -9])
  or during another transition. One transition at a time per match; a stage switch and a tag never overlap.
- **Freeze**: the engine stops running fighter / item / stage procs but **keeps consuming inputs and advancing the frame
  counter and hash**: new import `gd_match_freeze(frames)` (section 3.9). n is a fixed number in the pool data (a tag 60, a
  stage switch 90 frames, owner question 7.13); since it is sim state, a mispredicted request rolls back cleanly and both
  peers freeze on the same frame.
- **Swap** (one frame, deterministic):
  - *stage*: `gd_stage_switch(slot, place)` of the `stage_switch` mod's API (port/stagesw): the destination DAT is already in a
    slot (preloaded at match start from the encounter's stage set), fighters are carried by rule (`keep` / `nearest` /
    `spawns`, the C carry code `script_stage_slot_carry.inc`), items are cleared, camera re-bounded.
  - *tag*: the engine already swaps a fighter's kind mid-match for Zelda/Sheik (the sub-fighter slot `[1]` that Geno also
    tracks [R `geno-v5.md` 8.2 "`Geno_StateBlock[6][2]`: 0 = main, 1 = sub-fighter (Nana, the inactive Zelda/Sheik form)"]).
    A tag is a **generalised transform**: the benched fighter is the sub-fighter, preloaded at match start; the swap hands the
    slot's control, percent-by-rule, stocks and position to it: new import `gd_fighter_swap(port, carry_flags)` (3.9). Party
    size is therefore **2** by construction; a bigger party needs teardown/spawn of fighters mid-match, which is heavier and
    unproven (question 7.11).
- **Release**: k frames of intangibility and no inputs for the swapped fighter, then play.

Presentation side (`envoy_ui`, per peer, idempotent):

- Reads `transition{id, kind, phase, frames_left, src, dst}` from the published view every present. On seeing a new `id` in
  FREEZE it starts the cinematic: a cover (wipe / flash / morph are the C build's three; the morph was render-only offsets
  that "never move collision during freeze" [R `gw_script_stage_slots.inc:5`]), the party portraits for a tag, the destination
  name for a stage. The cinematic's clock is the sim's `frames_left`, so it ends exactly when SWAP happens and both peers see
  the reveal on the same sim frame even if their wall clocks differ.
- **Rollback**: a request that is undone by a rollback makes the `id` vanish from the view; the UI cancels the cinematic
  (a brief cover flicker at worst). To avoid that, the UI may start the cover only once the start frame is confirmed on all
  peers (`gd_ui_confirmed_frame()`, 3.9); with input delay 2 and depth <= 7 this is a 2-7 frame latency on a 60-frame freeze.
  Recommended: start at the confirmed frame.
- **Skip**: two options. (a) Presentation-only: a peer may fast-forward *their* cinematic to its end card; the sim freeze keeps
  its fixed length (fair, simple). (b) Shortening: when all players press a button, the freeze ends early; that is a pad
  input, so deterministic. Recommended (a) first; (b) is a pool-data flag. Question 7.14.
- **Desync or a peer skipping**: cannot happen in the sim (the freeze is sim state). A peer who *cannot draw* the cinematic
  (mod missing) sees a frozen game for n frames; acceptable, and impossible when `envoy_ui` ships in the same package.

Save/load across a transition: the C build refused it. In Rust the stage slot contents must live in `GameScope` (or be
re-applied idempotently on load) so a rollback or a savestate across a switch is sound; this is the one requirement Envoy
places on `port/stagesw` beyond its API (question for that lane; section 4.2).

### 3.6 Tag design (the TagFighter idea, our rules)

From melee-pc-TagFighter (read for the idea only [R its README via fetch]): D-pad down calls the partner, both fighters are
on screen as point and assist, the tagged-in fighter takes control at once unless in a stuck state, a stock can be donated to
revive the partner. Our version differs by the owner's rule: **a tag pauses the game and plays a cinematic**, then the swap.
Design choices for the owner (section 7.10-7.15): what carries (percent, stocks, position, action), who may trigger (point
player only; a rule; both), cooldown, whether an eliminated partner can be revived by a stock donation (TagFighter's idea),
and whether the bench fighter is ever visible.

### 3.7 Second Wind (rewind as a mechanic) [J]

The online LAB already applies "rewind" on both peers as a synced command with 0 desyncs (lockstep). For Envoy a Second Wind
is a command (`gd_ui_cmd`) that the host stamps for an agreed future frame; at that frame every peer loads the ring snapshot
from N frames before the stamp and re-sims with the logged inputs (`gd_lab_rewind`'s machinery, exposed to a sim mod as
`gd_match_rewind(frames)` under the same gating). Under rollback (not lockstep) this needs the host to apply the rewind at a
frame all peers have confirmed; the cost is a 3-second visible rewind for everyone. It is the one mechanic no other fighting
roguelite can offer; it is also the riskiest item here (milestone 6), hence optional.

### 3.8 Geno defines for elites [J]

An elite or boss opponent carries a `geno.json` attach/define entry that alters one special (an echo projectile, a longer
glide, a changed hitbox) as data, so the fight reads differently without any Envoy code. Defines are not hosted in Rust yet
(`geno.md` status: 50/64 tests, the 14 that need a `define` are open). Attach-only behaviours (which pass today) are enough
for the first elites. Shown on the opponent card as one line.

### 3.9 Host APIs: exist vs must be added

Exist today [R `modhost.md`, `wasm-ui-lab-api.md`, `gdmod.h`]: hooks (frame, take_damage, fighter procs, 19 action enters, items,
stage), read accessors (pos, vel, percent, facing, action state, input, players, items, stage kind, RNG, trig),
`gd_fighter_set_percent/pos/vel/facing`, `gd_fighter_request_action_state` (3 states), `gd_fighter_write` (raw), `gd_ui_*`
(submit, nodes, focus, capture, inject offline, scenes, textures, fonts, `gd_ui_session`, `gd_ui_scene`), `UI_HOST` catalogs
and requests, `gd_lab_*` (slots, ring, rewind, record/play, cpu behaviours, overlays, queries, LabNetplay commands).

Must be added (each a small, general import other mods will use too; section 4.2 shows who else needs them):

| import | side | what | needed by |
|---|---|---|---|
| `gd_hash_word(u32 word)` | sim | mix a word into the per-frame hash (NHV1 extra column + rollback hash), like the C's `ScriptMods_HashWord` | every sim mod with state: Envoy, Turbo-style rules, stage_switch |
| `gd_sim_view_publish(ptr, len)` / `gd_ui_sim_view(mod_id, out, cap) -> len` | sim / pres | a sim mod publishes a versioned struct each frame; presentation mods (any) read it after the logic frame | Envoy HUD, LAB reading rules mods, stage_switch HUD |
| `gd_ui_cmd(kind, ptr, len)` / export `gd_cmd(kind, ptr, len, frame)` in the sim mod | pres -> sim | a decision from a screen delivered to every peer's sim mod on an agreed frame (lockstep: next tick; rollback: host-stamped frame), the generalisation of `LabCmd` | Envoy picks and votes, stage_switch's manual switch, any menu that changes a match |
| `gd_ui_confirmed_frame() -> i32` | pres | last frame confirmed on all peers | cinematics, toasts that must not lie |
| `gd_match_freeze(frames)` | sim | stop fighter/item/stage procs for n frames while inputs, frame count and hash advance | transitions (Envoy, stage_switch), hitstop demos |
| `gd_fighter_mod_set(port, family, f32)` / `_get` | sim | the fighter-mod table with engine floors (damage dealt/taken, kb dealt/taken, run/air speed, jump, air jumps, fall speed, weight, shield, armour frames, intangible frames, hitstop add) | Envoy, Turbo, handicap mods, LAB cheats (GAP-6) |
| `gd_hit_view(out)` in `hit.fighter_process_hit` pre/replace | sim | attacker, victim, damage, element, move tag (action state class), is_smash/aerial/projectile; writable damage, element, kb scale | Envoy triggers and conversions |
| wire `hit.fighter_process_hit`, `fighter.process_death`, the shield / ledge / grab / throw `action.*.enter` rows | host | already in hooks-v1.tsv; wiring only | Envoy, LAB events |
| `gd_fighter_swap(port, carry_flags)` | sim | the generalised Zelda/Sheik transform: hand the slot to the preloaded sub-fighter | tag |
| `gd_stage_slot_load / gd_stage_switch / gd_stage_queue*` | sim | **the `stage_switch` mod's API (port/stagesw)**: Envoy calls it, never reimplements it. Requirements Envoy adds: callable from a sim mod on an agreed frame, slot contents in `GameScope` (rollback / savestate safe), a `place` rule, a per-switch event in the view | stage transitions |
| `gd_match_setup` (already named "not done" in A5) with party (main + sub kind per port), stage slot set, items off, rules | pres -> flow | the only way a mod starts a match; in netplay it goes through the lobby so both peers agree | every mode mod |
| `gd_session_blob_set/get(ptr, len)` | pres / flow | a small agreed blob carried across scenes by the netplay session (the C's `RN2`), host-authoritative, digest-checked; offline it is just saved | run records of Envoy, stage runs, tournaments |
| `gd_data_read/write(name, ptr, len)` | pres | per-peer save files under the mod's own data dir (`melee_data`), atomic replace | profiles, run saves, LAB settings |
| `gd_fx_*` (surface layer params per fighter, a joint-following emitter, a tracer, a post pass with a budget) | pres | the declarative, rollback-safe effect binding the visual identity note asked for (its spec C) | Envoy looks, stage_switch covers, LAB tint (GAP-10) |
| `gd_match_rewind(frames)` | sim/host | Second Wind; the LAB ring under a command | optional |
| `gd_cpu_assist(port, skill)` or a `gd_lab_cpu`-like synthesiser available to a sim mod | sim | opponents that *use technique* (the C's `foe_driver`) | CPU encounter kinds |

### 3.10 Save data

| file (per peer, `melee_data` mod dir) | content | agreed? |
|---|---|---|
| `profile.bin` | unlocked records, best runs, technique counters (L-cancel, wavedash, perfect shield, tech rates), settings | no; the unlock mask is advertised in the lobby and intersected for rolls (the C director's rule) |
| `run_<seed>.bin` | the run record (3.4) + digest + the last encounter's mid-fight snapshot reference (offline only; a snapshot is disc-derived data and must never leave the machine) | the record: yes (every peer holds the same bytes); the snapshot: offline only |
| `pool.digest` | FNV of the compiled pool | yes: a run names its pool; a peer with another pool is refused in the lobby |

Migration rule: a run holding a record that no longer exists is dropped with a notice (readability Q11's suggested answer).

### 3.11 Performance limits

- Sim module: linear memory 1 MiB fixed, state <= 256 KiB, fuel 1,000,000 per hook call, re-entrancy depth 8. Envoy's state
  is ~5 KiB; the pool ~8 KiB `.rodata`; the per-frame tick is bounded by the queue cap (64 events) and chain depth (4).
  Resimulated frames run the same tick; at depth 7 that is 7 ticks in one display frame, well inside the 8.3 ms target [I].
- Presentation module: fuel 4,000,000 per present, one submit, no allocation; the LAB's measured 42-47 us p50 is the bar; the
  reward grid (~60 nodes) is the CSS-sized case the budget table allows 0.15 ms for.
- Preloading: a tag pair and an encounter's stage set (2-3 DATs) are loaded before frame 0, never mid-match; the stage slot
  pool is 16 in the C build with a 512 KiB headroom rule; the Rust `stage_switch` lane sets its own.

### 3.12 Visuals (approved direction, 2026-11-08)

**(a) What we build ourselves**

| layer | how | who |
|---|---|---|
| UI (HUD, screens, run map) | wasm presentation mods on `gd_ui` (section 3.3): kit nodes, our own theme tokens, SDF fonts already in the host. No old GD art (memory rule); the LAB mod's look is the house style to extend, not the C kit's | Opus, designed with the owner one step at a time |
| transition cinematics | presentation-only scenes (layer `SCENE`, GX hidden or composited) that **reuse the disc's own models, animations, effects and camera, loaded at runtime from the user's disc and never shipped**, plus our camera moves and post effects; the cinematic's clock is the sim freeze (3.5) | Opus |
| 3D geometry we add (arena dressing, reward pedestals, a hub later) | the owner's **Blender parametric pipeline** (memory: platform-fighter stage assets, Blender 5.2 geometry nodes) exporting to the engine's model path | owner's pipeline + Sonnet for export glue |
| procedural effects and shaders | **an effects system in `melee_gx`** (wgpu; Vulkan on Windows/Linux, DX12 verified pixel-equal [R `render.md`]) exposed to mods as `gd_fx_*` (3.9) | Opus |
| 2D (sprite fighters, 2D elites, drive icons) | the **Geno sprite pipeline** (`port/genosprite`, `notes/geno-sprite-fighters.md`: indexed PNG, palette rows per costume, stage-plane quads) | that lane; Envoy consumes |

**(b) What needs a person**: illustration, key art, portraits, a real logo, distinctive hand-made 3D models (a boss, a hub).
Commission or the owner. **The first playable slice (milestone 1) uses only (a)**; commissions are decided after it is played.

**(c) Against sameness** (the owner's real concern: the C build's many "separate, specific" shader effects came out looking alike)

1. **Composable primitives, not one-off shaders.** The `melee_gx` effects system is a small library of passes and primitives
   with parameters, combined per effect: SDF shapes (ring, capsule, polygon, glyph), noise and flow fields (simplex, curl,
   domain warp), voronoi/cellular, dissolve and erosion masks, screen-space displacement and refraction, chromatic and temporal
   effects (ghost, smear, strobe-safe flicker), feedback buffers (trails that live in a texture), compute particles with
   fields, reaction-diffusion and a small 2D fluid, palette and colour-grade LUTs, toon/ink/halftone stylisation, scene-depth
   occlusion (the post pass can occlude against fighters and stage; `_research/envoy-visual-identity-2026-10-05.md` 0). Two effects
   differ in **mechanism**, not just colour and speed.
2. **Purpose first, technique second, and a written rule.** Every effect record in `envoy_data` states (i) what it
   communicates: *telegraph, reward, danger, transition, status, synergy, failure*; (ii) its core technique from the list above;
   (iii) two named references (a game or a known technique, credited in CREDITS.md as ideas only). **Rule: within one set
   (the statuses; the transition covers; the reward moments; the keystone halos) no two effects share a core technique.** The
   check is mechanical: a `fx_lint` over the records refuses a set that repeats a technique.
3. **A visual review loop.** `melee fx-gallery` (headless, `melee_gx` offscreen) renders every effect record to stills at
   three moments and a 2-second clip, on a neutral stage with one fighter, and lays them out as a **contact sheet** (one row
   per set) so sameness is visible at a glance. The owner picks or rejects per round; a rejected effect's record keeps the
   reason. Agents attach the sheet to every effects hand-back; nothing is "done" from source alone. The sheets are rendered
   from the user's disc and stay out of the repo (disc-derived); the records and shaders are committed.
4. **Agents.** Opus for shader work; **one effect family per agent** (statuses; covers; reward and level-up moments; keystone
   and party identity; crits and technique; failure and KO), each with the gallery sheet as its deliverable and the lint
   passing. Sonnet writes the gallery harness and the lint once Opus has fixed the record format.

**Sizing inside the milestones** (added to section 5): the gallery harness and the record format land with milestone 3 (the
first cover needs it); primitives arrive family by family in milestones 3 (covers: wipe, flash, dissolve, displacement),
5 (statuses and opponent identity) and 6 (reward moments, crits, failure, the rest). The effects system itself is item 9 of
section 4.2, started in parallel with milestone 2 by one Opus agent so milestone 3 does not wait on it.

---

## 4. Everything else as wasm mods

### 4.1 Survey and order

| C-build feature | becomes | state | needs (from 3.9) | order |
|---|---|---|---|---|
| LAB | `rust/mods/lab` | done (M0-M3, M4 part, M9 part) | - | - |
| in-game browser, Nucleus | `rust/mods/browser` | done | - | - |
| custom menus (kit screens) | menu mod(s) on gd-ui | in progress (`menus.md`) | `gd_match_setup` | 1 |
| stage slots / switching / stage tour | `rust/mods/stage_switch` | in progress (`port/stagesw`) | `gd_match_freeze`, `gd_hash_word`, cmd channel, `gd_fx_*` covers | 1 |
| **Envoy** | `rust/mods/envoy` | this plan | all of 3.9 | 2 |
| Turbo / match rules (`gw_matchrules.h`) | `rust/mods/turbo` (sim) + a rules page | not started | `gd_fighter_mod_set`, `gd_hash_word`, `gd_match_setup` rules | 3 |
| hitstop / events / gauntlet demos | small sim mods | not started | `gd_match_freeze`, cmd channel | 3 (test vehicles for the APIs) |
| shader-demo, crit shader, mod surface shaders | effects through `gd_fx_*` | not started | `gd_fx_*` | 4 |
| co-op (local, one team) | an Envoy encounter kind, not a mod | - | `gd_match_setup` teams | with Envoy |
| tm_lite / training | LAB presets | LAB | - | - |
| combo tools, frame-data export | LAB tools | LAB M-later | - | - |
| Geno fighters / m-ex import | data + native (not wasm) | Geno 50/64; m-ex importer plan | - | - |
| online lobby UI (room code, delay, how-to-play) | menu mod page | with menus | `UI_HOST` ONLINE catalog (exists) | 1 |
| settings overlay, cosmetics | native today; a menu page later | - | `UI_HOST` | later |
| old Envoy era (mazes, mission builder, gene economy) | **not ported** | - | - | - |

### 4.2 The shared host APIs, built once

In dependency order, each with its first consumer so it is tested by a real mod:

1. `gd_hash_word` (consumer: `stage_switch`, then Envoy) - half a day of host work plus the NHV1 column note.
2. `gd_sim_view_publish` / `gd_ui_sim_view` (consumer: `stage_switch`'s HUD, LAB "rules" readout) - 1-2 days.
3. `gd_ui_cmd` / `gd_cmd` over lockstep and rollback (consumer: `stage_switch` manual switch, Envoy picks) - the lockstep half
   exists as `LabCmd`; the rollback half (host-stamped frame in the input stream) is new: 3-5 days.
4. `gd_match_freeze` (consumer: `stage_switch` transition, hitstop demo) - 1-2 days.
5. `gd_match_setup` with party and stage set (consumer: menus, Envoy) - already planned in A5; 3-5 days with the lobby path.
6. `gd_session_blob_*` and `gd_data_*` (consumer: Envoy run record and profile) - 2-3 days.
7. `gd_fighter_mod_set/get`, `gd_hit_view`, wire the hit/death/shield rows (consumer: Envoy rules, Turbo) - 1 week.
8. `gd_fighter_swap` (consumer: tag) - 1-2 weeks: the transform path generalised, preloading, Geno's sub-fighter block.
9. `gd_fx_*` (consumer: Envoy looks, stage covers) - 2-3 weeks; design from the visual identity note.
10. `gd_match_rewind`, `gd_cpu_assist` (optional / CPU kinds) - sized in milestones 6 and 2.

---

## 5. Milestones

Sizes: S = days, M = 1-2 weeks, L = 3+ weeks. "Opus" = UI, design-sensitive or cross-cutting host work; "Sonnet" = plain
ports, data authoring, tests, wiring with a clear spec (memory: bulk workflow agents on Sonnet).

| # | milestone | what is playable at the end | size | risk | model |
|---|---|---|---|---|---|
| 0 | **Probes.** (a) CPUs in a rollback netplay match: two peers, CPU on port 3, synctest depth 12, 3600 frames, 0 mismatches. (b) `gd_hash_word` + a sim mod with state under synctest. (c) items off, stage slot load before frame 0 under rollback (with the stagesw lane). | nothing new to play; the schedule's two biggest unknowns answered | S-M | (a) is the gate for every CPU encounter kind; if it fails, order (1) duels first and fix CPU determinism as its own lane | Sonnet (a, b), with the stagesw agent (c) |
| 1 | **Smallest playable slice: a duel run, offline and 2-peer.** `envoy_sim` with 12 passive drives (fighter-mod families only), a run of 5 duels on a fixed graph, reward screen between (3 offers, lockstep, host-counted timer), run record over `gd_session_blob`, `envoy_ui` HUD squares + reward grid, pool as data. Solo = the same run against a level-9 CPU *offline only*. | two people play a 5-encounter run online with builds that matter; one person plays it offline against CPUs | M | the cmd channel's rollback half; `gd_match_setup` through the lobby | host APIs: Opus; sim rules port + data: Sonnet; UI: Opus |
| 2 | **Triggered rules and the event vocabulary.** Wire hit/death/shield rows, `gd_hit_view`, statuses (5), Momentum, the 38 triggered records re-authored one-bite; parity harness replaying recorded event streams against the C evaluator's fixtures (`pc/tests/script_mods_core_test.c` event files are reusable as data) | builds with "when X, do Z" rules online, hashed | M-L | parity with the C engine where the owner liked the feel; the technique rates are still guesses until profiles collect them | Sonnet (port, tests), Opus (hit view semantics) |
| 2b | **Effects system in `melee_gx` (parallel with 2).** The composable primitives library (3.12c item 1), the effect record format with purpose / technique / references, `fx_lint`, the `melee fx-gallery` contact-sheet harness, `gd_fx_*` imports for presentation mods | a contact sheet of the first primitives the owner can judge | M-L | scope creep; the gallery must come before any effect is "done" | Opus (primitives, record format), Sonnet (gallery harness, lint) |
| 3 | **Transitions: stage switch.** On top of `stage_switch`'s API: freeze, cover cinematic driven by the sim clock, switch on an agreed frame, carry rules; a "survival room" encounter kind that switches every N s; synctest across a switch; savestate across a switch. Effects family **covers** (wipe, flash, dissolve, displacement; one technique each) through the gallery; cinematic scenes reuse runtime-loaded disc models and camera (3.12a) | the arena changes under a fight, online, with a cinematic | M | depends on `port/stagesw` making slots `GameScope`-resident; without 2b the covers stay wipe/flash | Opus (transition state machine + cinematic + covers), Sonnet (tests) |
| 4 | **Transitions: tag.** `gd_fighter_swap` via the transform path, party of two in `gd_match_setup`, tag request as input, carry rules from question 7.10, the tag cinematic (party portraits are placeholders until (b) art exists), tag cooldown and refusal rules | party play: swap fighters mid-fight, online | L | the heaviest host change (fighter kind swap outside Zelda/Sheik); preloading two fighters per port on the disc seam; HUD stock display for a pair | Opus |
| 5 | **CPU encounter kinds (if probe 0a passed).** Opponent rolls to the party's strength (+25% cap), elites with Geno attach entries (and 2D elites from the sprite pipeline when it lands), team-vs-CPU with shared stocks pool, `gd_cpu_assist` for technique-using foes; path forks in the run graph; boss. Effects families **statuses** and **opponent identity** through the gallery | the roguelite proper: a run against the game, solo or co-op, to a boss | L | CPU determinism online; opponent card readability; balance (section 6's harness) | Sonnet (rolls, data, harness), Opus (encounter kinds, boss, effects) |
| 6 | **Looks, saves, polish.** Remaining effect families (reward and level-up moments, keystone and party identity, crits and technique, failure and KO) under the budget of the visual grammar; profile with technique counters; run resume from a mid-fight snapshot offline; results and run map; Second Wind (optional); spectator view; decide commissions (3.12b) | the game the owner can show | L | Second Wind under rollback; art that needs a person | Opus (effects, UI), Sonnet (saves, counters) |

Milestone 1 is the smallest playable slice. It needs host items 1-6 of section 4.2 and nothing from 7-10, and it uses only
section 3.12(a) visuals (kit-node UI, no effects). Everything after it is additive; nothing in 1 is rebuilt later.

---

## 6. Research basis (verified citations; one Envoy decision each)

Verified 2026-11-08 by web lookup (publisher or author pages); marked [V]. Where the owner's recollection differed from
the record, the record is given.

| source | what it says that we use | the Envoy decision it supports |
|---|---|---|
| Togelius, Yannakakis, Stanley, Browne, "Search-Based Procedural Content Generation: A Taxonomy and Survey", *IEEE TCIAIG* 3(3):172-186, 2011, doi 10.1109/TCIAIG.2011.2148116 [V] | generate-and-test: content is represented, searched (evolutionary or otherwise) and scored by a fitness function, which may be **simulation-based** (an agent plays the content) | **The encounter generator is generate-and-test over the headless sim.** A candidate encounter (stage set, switch schedule, opponent build, tag setup) is rolled from the seed, played out by CPUs headless (the engine runs 30-80x real time; `bench-unlocked.md`), scored (fight length, damage ratio, stocks, switch count), and kept only inside the target band. The *seed* stays the agreed thing between peers; the generator runs offline when authoring the pool and the graph templates, so no peer has to run it live |
| Shaker, Togelius, Nelson, *Procedural Content Generation in Games*, Springer 2016, doi 10.1007/978-3-319-42716-4, chapters free at pcgbook.com [V] | the constructive vs search-based split; dungeon and level chapters; "evaluation functions" chapter | Run graphs are **constructive** (templates with forks, cheap, seeded) and their contents **search-tested** offline; we do not search live in a lobby |
| van der Linden, Lopes, Bidarra, "Procedural Generation of Dungeons", *IEEE TCIAIG* 6(1):78-89, 2014, doi 10.1109/TCIAIG.2013.2290371 [V] | what dungeon generators lack is **control**, not speed: designers need gameplay-level knobs | The run graph is authored at the level of *encounter kinds, forks and reward kinds*, not geometry; the maze era failed exactly on control (ENVOY-PROBLEMS) |
| Yannakakis, Togelius, "Experience-Driven Procedural Content Generation", *IEEE Trans. Affective Computing* 2(3):147-161, 2011, doi 10.1109/T-AFFC.2011.6 [V]; and *Artificial Intelligence and Games*, Springer 2018 (paid; companion site gameaibook.org) [V: not free, contrary to the brief] | content adapts to a model of the player's experience built from play data | The **profile's technique counters** (L-cancel, wavedash, perfect shield, tech rates per minute) are the player model; they replace the guessed `B.human` rates the C build never measured and drive both the strength of technique records *for that player* and what the generator offers them |
| Hunicke, "The Case for Dynamic Difficulty Adjustment in Games", ACE 2005, pp. 429-433, doi 10.1145/1178477.1178573 [V] | DDA as an explicit, probabilistic policy over inventory and encounters, acting between events rather than mid-event | **Mixed-skill co-op**: difficulty is adjusted at *encounter boundaries only* (opponent strength target, offer quality, stocks pool), never inside a fight, and it is per run not per player, so the agreed record stays the single source; the per-player lever is each player's own build and offers |
| Chen, "Flow in Games (and Everything Else)", *CACM* 50(4):31-34, 2007, doi 10.1145/1232743.1232769 [V] | keep each player in their own flow zone by embedding adaptive choices in the core activity, not in menus | The **path fork** (a harder room for a better reward) is the player-steered difficulty choice; the opponent roll follows the party's strength, so the choice is real |
| Juul, *The Art of Failure*, MIT Press 2013 [V] | failure must be attributable and recoverable to be satisfying | Run death ends the run but the **profile keeps the counters and unlocks**; the results screen names what ended the run (which encounter, which rule); Second Wind is a bounded, visible recovery |
| the Berlin Interpretation (IRDC 2008) [V: a community document, factor counts differ between sources] | random environment generation, permadeath, non-modal play, resource management, "hack'n'slash", exploration | We take random generation, run permadeath and resource management (the bag, stocks pool); we drop turn-based and single-character (party of two, multiplayer) knowingly: Envoy is a roguelite by the sources' own vocabulary |
| Giovannetti, "'Slay the Spire': Metrics Driven Design and Balance", GDC 2019 (GDC Vault; Game Developer coverage) [V: talk exists; no transcript verified, claims are from coverage] | balance thousands of cards by a metrics server fed by every run; data misleads when conflated | The **headless harness** (section 6 row 1) plus the profile counters are our metrics server; every balance change is checked against the CPU proxy *and* the owner's own recorded counters, never one alone (the synergy lane's lesson) |
| Supergiant Games, Hades: Kasavin's talks and interviews on roguelike narrative (GDC 2021 session was on dialogue; run structure is from Game Developer / GDC Podcast coverage) [V: partial] | persistent narrative across deaths; bosses that remember | Not adopted now: Envoy has no narrative layer in these slices; noted for the companion/garden layer (question 7.20) |
| Yu, *Spelunky*, Boss Fight Books 2016 [V] | level templates plus randomised fill, every element hand-checked to be fair in isolation | Encounter templates are hand-authored and each is **played in isolation by the harness before entering the pool** (the PLAN-ENVOY chunk rule, kept) |
| Rupp, Puddu, Becker-Asano, Eckert, "It might be balanced, but is it actually good? An Empirical Evaluation of Game Level Balancing", IEEE CoG 2024, doi 10.1109/CoG60054.2024.10645642 [V] (newer, found during verification); related: Rupp, Eberhardinger, Eckert, *IEEE Trans. Games* 16(4):903-913, 2024 | simulation-balanced levels (agent win rate at 0.5) are perceived as more balanced by people, with differences per scenario | Target band for a generated **duel** arena/switch schedule: CPU-vs-CPU win rate 0.4-0.6 over the schedule; and confirm with people, since the perceived effect differed by scenario |
| Holmgård, Green, Liapis, Togelius, "Automated Playtesting with Procedural Personas through MCTS with Evolved Heuristics", *IEEE Trans. Games* 2018 [U: known paper, not re-verified today] | several agent *personas* (aggressive, cautious) evaluate content, not one agent | The harness runs at least two CPU profiles (the retail level-9 and an assisted technique-user) so technique-gated records are not scored by a proxy that never performs technique (the C lane's blind spot) |

---

## 7. Open questions for the owner (all at once; each gets one short answer)

**Direction and mode**
1. Direction C (party roguelite, encounter kinds shipped duels-first) as recommended, or A (versus set) or B (co-op gauntlet) alone?
2. If the CPU-in-rollback probe (milestone 0a) fails, do we ship the human-vs-human run first and fix CPU determinism as its own lane, or hold Envoy until CPUs are proven?
3. Players per run: 2 first, 3-4 later (needs transport work), or design for 4 from day one?
4. Co-op friendly fire and shared stocks pool (the C director's pool of 12 for two humans), or per-player stocks?
5. Versus encounters inside a co-op run (a duel for a drive between teammates): wanted, or keep a run purely co-op or purely versus?

**Netplay rules**
6. Joining at an encounter boundary with a depth-scaled starter build: allowed (host consent) or runs are closed once started?
7. Disconnect: survivors continue and the leaver may rejoin at the next boundary (recommended), or the run pauses until they return (time limit)?
8. Reward timer in lockstep ticks: 20 s (1200) or the C build's 30 s (1800)? Timeout rule: take offer 0, or keep the build?
9. Unlocks: gate only what *you* can be offered (recommended), or gate what anyone may hold in a run you join?

**Tag (character switch)**
10. What carries on a tag: (a) percent stays with each fighter (the pair has two health bars), (b) the pair shares one percent, (c) the incoming fighter heals a fixed amount? Stocks: shared pool per player (recommended) or per fighter? Position: the incoming fighter appears where the outgoing one was (recommended) or at a spawn point?
11. Party size: two per player via the engine's transform path (recommended; fits the sub-fighter slot), or more (needs mid-match fighter teardown/spawn, L and unproven)?
12. Who triggers a tag: the controlling player (D-pad down), a rule (a keystone "tag on every KO"), or both? Cooldown in seconds? Allowed while airborne?
13. Cinematic length: a tag 1.0 s (60 frames), a stage switch 1.5 s (90), both fixed in the pool data? Or shorter?
14. Skip: presentation-only fast-forward with the sim freeze kept (recommended), or all players pressing a button ends the freeze early?
15. Is the benched fighter ever visible (an assist that appears and leaves, TagFighter-style), or only in the cinematic? Can a stock be donated to revive an eliminated partner (TagFighter's idea)?

**Stage switch**
16. Who triggers a stage switch in Envoy: rules and the run only (recommended), a player action as well (a drive "pull the arena"), or both?
17. Which transition covers first: wipe and flash (no effects API needed), with morph once `gd_fx_*` exists?
18. Stage set per encounter: 2-3 preloaded retail stages (the C build's six legal DATs) or also mod stages (needs the stagesw lane's loader)?

**Loot, rules, feel**
19. Keep the readability pass's decisions as made (two rules per drive, 30 keystones, seven prices, five statuses, Momentum as orbs, no "assembled" toast)? Any veto?
20. Second Wind (a shared 3-second rewind, once per encounter): in the plan (milestone 6) or out?
21. The companion / garden (Chao-like) layer: still wanted later, on top of the party and profile, or retired with the maze era?
22. The strength index `x1.8`: show it at all in the first slice, or only the compare rows on the reward screen?
23. Name: keep "Envoy" / "SuperTime Envoy", or rename with the rebuild?

**Process**
24. Visual identity: start with HUD-side looks only (no fighter effects until `gd_fx_*`), yes?
25. Do you want the technique counters collected from your own play *before* milestone 2 tunes technique records (a short recording session), or tune from the CPU proxy first and correct later?
26. Playtest cadence: one short two-peer session per milestone over loopback with you on one pad (the memory rules: no OS input in agent runs, volume 3, named windows), then one real-network check (<= 2 unranked) at milestone 1 and 5?

**Visuals**
27. First gallery round: the transition covers (needed by milestone 3, recommended) or the five statuses?
28. Contact-sheet format: stills at three moments plus a 2-second clip per effect, one row per set; how many candidates per effect per round (3?)
29. Commissions (3.12b): decide after milestone 1 as planned, or brief an illustrator for portraits and a logo earlier so the tag cinematic has real art at milestone 4?
30. Transition cinematics reuse the disc's models and camera at runtime (never shipped): any limit you want (no story cutscenes, no voice), or free use?

---

## 8. Files cited

Workspace: `_research/envoy-gameplay-ui-audit-2026-10-05.md`, `_research/envoy-netplay-scoping-2026-10-05.md`,
`_research/envoy-readability-pass1-2026-10-05.md`, `_research/envoy-strength-decision-2026-10-07.md`,
`_research/envoy-synergy-analysis-2026-10-05.md`, `_research/envoy-visual-identity-2026-10-05.md`,
`_research/chao-genetics-for-envoy-2026-10-03.md`, `docs/ENVOY-PROBLEMS-2026-10-02.md`,
`docs/PLAN-ENVOY-EDITOR-MISSIONS-2026-10-03.md`, `docs/superpowers/specs/2026-10-04-envoy-loot-and-modifiers-design.md`,
`docs/HANDOFF-2026-10-05.md`, `docs/NEXT-SESSION.md`, `README.md`, `CREDITS.md`.
Game checkout (branch pc-port): `pc/scripts/examples/envoy/{README,ONLINE,COOP,MENUS}.md`, `scripts/*.lua`,
`pc/gameworld/script_mods_core.h`, `script_build.inc`, `script_stage_slots.inc`, `script_stage_slot_carry.inc`,
`pc/platform/gw_matchbuild.h`, `gw_netrun.h`, `gw_script_stage_slots.inc`, `pc/scripts/examples/stage_switch_demo/README.md`.
Rust port (`pub/rust-port`): `docs/rust-port/STATUS.md`, `notes/modhost.md`, `notes/wasm-ui-lab-api.md`, `notes/lab-ui-spec.md`,
`notes/geno.md`, `notes/net.md`, `notes/menus.md`, `docs/specs/mod-c-v1.md`, `docs/specs/gdmod.h`, `docs/specs/hooks-v1.tsv`,
`docs/specs/gd-ui-lab-v1.h`, `docs/specs/geno-v5.md`, `rust/mods/{lab,browser,moves_mod,world_mod,ui_demo}`.
Outside: Joyastick/melee-pc-TagFighter README (idea only), and the sources in section 6.
