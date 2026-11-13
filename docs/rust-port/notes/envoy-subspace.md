# Envoy: Subspace enemies and bosses (branch `port/envoy-subspace1`)

Started 2026-11-12. Lane: more of Brawl's Subspace Emissary in Envoy beyond Petey. Petey's own files are not
touched (`petey.rs`, `petey_file.rs`; `petey_model.rs` gains one added function, see "Shared files").

## 1. What the disc offers (inventory)

Read from the owner's own extracted Brawl files (names and counts only; nothing derived is in git).

- Every enemy is a pair of archive members: a parameter file `Enm<Name>` (the same relocatable container as Petey's
  boss file: `rust/brawl_boss` parses it unchanged, `brawl_boss_dump` lists it) and `Enm<Name>Brres` (one MDL0, its
  textures, one CHR0 per member, effects). `petey_model.rs`'s readers load all of them as they are.
- The Primid family lives in `stage/adventure/adventure_common.arc` (one rig, 90 clips, weapon models: beam sword,
  boomerang, Super Scope); everything else sits in the adventure stage archives that use it.
- Common enemies found (bones / clips): Goomba 33 / 14, Giant Goomba 28 / 18, Koopa Paratroopa 34 / 15, Hammer Bro
  32 / 10 (+ hammer model), Bullet Bill 19 / 4, Primid 50 / 90, Spaak 39, Glire family 24, Jyk 9, Mite 12, Bytan 12,
  Feyesh, Poppant, Towtow, Bucculus, Roturret, Floow, Auroros, Greap, Shellpod, Armight, Cymul, Nagagog, Buckot, Borboras,
  Ticken, Gamyga (5 parts), R.O.B. (3 kinds), Bombed, Shaydas, Puppit, Autolance, Roader, Trowlon.
- Bosses found (bones): Rayquaza 76, Porky 61 (+ statue, robots), Galleom 92 (+ missile), Duon 65 (+ 4 weapon models),
  Ridley 115, Meta Ridley 106 (+ the Falcon Flyer, 25), Tabuu 146, Master Hand 36 / Crazy Hand 38 (Brawl's).

What the existing importer handles: MDL0 v9 with one texture per material, CHR0 (all key formats), CMPR / plain /
palette textures, skinning by bone or weighted mix. All of the above parse. Not handled: effects (REFF / REFT),
environment / indirect texture stages (the Primid's sheen, the Bullet Bill's shine), texture pattern / SRT clips
(the Goomba's eyes do not blink), visibility clips, sounds per enemy.

## 2. Ranked candidates (value to the roguelite per unit of effort)

Common enemies first: a handful of varied ones changes every room; one more boss changes one fight.

| rank | candidate | why | effort |
|---|---|---|---|
| 1 | Goomba | the simplest full enemy (walk, charge, stomp); proves the pool, the hit paths and the room wiring | S |
| 2 | Primid | Subspace's signature enemy; a real melee threat (three attacks with wind-ups) | M |
| 3 | Bullet Bill | an aerial threat that crosses the arena and explodes: rooms read differently at once | S |
| 4 | Giant Goomba | elite-weight variant (HP 18 + 6 L, tantrum stomp): a mini-boss for elite rooms from data already read | S |
| 5 | Hammer Bro | the first thrower (needs a projectile record: the hammer model is in its archive) | M |
| 6 | Koopa Paratroopa | flyer that becomes a walker / shell; three modes | M |
| 7 | Primid variants (Sword, Boomerang, Fire, Big, Metal, Scope) | one rig, per-variant attacks and a weapon model | M each |
| 8 | Spaak, Jyk, Mite | lightning cloud, static hazard, swarm | M / S / S |
| B1 | Galleom | ground boss: jumps, slams, punches; fits a flat side-view arena like Petey | L |
| B2 | Duon | ground boss, two faces, turns; flat arena | L |
| B3 | Porky | ground walker with lasers and robots | L |
| B4 | Rayquaza | flies, burrows, long body; needs a wide arena and many attacks | XL |
| B5 | Ridley / Meta Ridley | fliers; Meta Ridley needs the Falcon Flyer as the floor | XL |
| B6 | Tabuu | teleports, 20+ screen-filling attacks | XL |

Built this night, in order: 1, 2, 3, 4 (section 3). Bosses: not started; the parameter files of B1-B4 are decoded by
the same tool and are the next step (section 7).

## 3. How an enemy is built

| piece | where |
|---|---|
| the pool, behaviour, hit tests, placeholder draw | `rust/melee_modes/src/tu/subspace.rs` (statics: snapshot / rollback covered; own LCG) |
| the look (async load, never blocks, never feeds the sim) | `rust/melee_modes/src/tu/subspace_model.rs` on `petey_model::load_rigs` / `Rig` / `draw_tris` |
| spawn / read API for the sim mod | `gd_envoy_enemy_spawn(which, x, y, facing, handle)` with `which = 100 + kind + 32 x tier`; `gd_envoy_enemy_i`, `gd_envoy_enemy_status` answer for these handles too |
| spawn API for engine code (a room, a map spawn point) | `melee_modes::tu::subspace::sub_spawn(100 + kind + 32 * tier, x, y, facing, handle) -> handle or -1`; `sub_status(handle)` 1 alive / 2 beaten; `sub_enemy_i(handle, 2 / 6 / 7 / 8 / 9)` damage taken / hits / last attacker slot + 1 / x / y; `alive()`; `clear_all()`; `preload(kind mask)` |
| per frame | `subspace::sub_tick` from `envoy::tick` (after the post chain); returns at once when the pool is empty |
| draw | `ftdrawcommon::BOSS_DRAW = sub_draw`, set by the first spawn of a match (Petey fights never spawn these) |

Kinds: 0 Goomba, 1 Primid, 2 Bullet Bill, 3 Giant Goomba. Tier = Envoy loops cleared; it picks Brawl's difficulty
scalar L (2.5, 4.0, 5.5, 7.0, 9.0), and every scaled number is the original's `base + slope x L`.

Rules that hold for every kind:
- HP is separate from percent. A hit takes `damage x the player's attack ratio`; the knockback uses Melee's formula
  with the damage taken so far as the percent and the kind's weight. Under 60 the enemy flinches; from 60, or when
  its HP is gone, it is launched. An enemy that leaves the blast zone is beaten.
- Only the players' side hurts enemies (fighter hitboxes, their items and projectiles, ownerless items). A foe
  fighter's attacks pass through them, and enemies never hit foe fighters.
- An enemy's attack lands as an environment hit through the engine's damage log (`ftcoll_8007a06c`): percent,
  knockback, hitlag and the damage state are Melee's own. A raised shield takes the damage instead. A player who
  is invincible or intangible is not hit.
- Deterministic: the sim reads no model, file or clock. The same inputs give the same run with or without the
  owner's Brawl files and whatever the load timing.

## 4. The enemies

Numbers marked [D] were read from the enemy's parameter file on the owner's disc; [T] are our tuning.

### Goomba (Brawl: Kuribo)
- [D] HP 4 + 0.8 L. One attack, the charge: 6 + 2.5 L, angle 63, growth 16 + L, base 110, live for the whole run.
  A walking Goomba does not hurt. Clips: walk, run hold (17), run, run brake (30), damage, fall, landing, collapse.
- [T] walk 0.28, run 0.95 per frame; sees a player within 55 x 14; turns at ledges; after a charge it brakes and
  waits 45-90 frames. The charge box is a circle of 4 at its front (the file's box is 1.6 on the head bone).
- Landing on its head squashes it (Brawl: the same) and bounces the player.

### Primid
- [D] HP 10 + 1.4 L. Punch: live frames 10-16 of 25, 4 + 2 L, angle 30, set knockback 24. Kick: 10-21 of 34,
  5 + 2.1 L, angle 65, set knockback 33. Hard punch: start 8, hold 41, then live 8-11 of 33: 12 + 2.5 L, angle 40,
  growth 25 + 0.5 L, base 120.
- [T] walk 0.38; closes to 10, then picks punch / kick / hard punch 45 / 35 / 20 among those that reach; waits
  35-90 frames between attacks; jumps to a player above it or across a ledge. Box positions are single circles in
  front of it (the file puts two or three on the arm / leg bones).
- Not done: its other moves in the file (rush, headbutt, air kicks, slide, step jump), the Big Primid's stronger set.

### Bullet Bill (Brawl: Killer)
- [D] HP 4 + 0.7 L. Contact in flight: 3 + 2 L, angle 70, growth 50, base 106. It then explodes: 6 + 3.2 L,
  angle 90, growth 35, base 120 - 5 L, radius 14, live 15 frames.
- [T] flies in from the side at 1.25 per frame, drifts toward the player's height while approaching, never turns
  back, explodes on contact or after 8 seconds; a hit that empties its HP drops it.
- Not done: the cannon it comes from, the explosion effect (the model vanishes; the blast is the hit only).

### Giant Goomba (Brawl: Dekakuribo)
- [D] HP 18 + 6 L. Charge: 14 + 2.4 L, angle 80, growth 28 + 0.5 L, base 113 - 1.6 L. Tantrum stomp: live from
  frame 39 of 121, 12 + 1.5 L, angle 70, growth 30, base 117 - 0.5 L. Landing on its head does not squash it.
- [T] walk 0.22, run 0.8; the tantrum when a player is within 20, else the charge; a stomp on its head costs it
  10 HP and bounces the player; weight 200 (it is launched only when beaten).

## 5. Where they appear in a run

See section 6 of this note once the sim wiring is in ("Rooms").

## 6. Checks

`tools/rust-port/envoy/subspace_check.sh` (DISC=vanilla|ace): per kind, a synctest of an enemy room with a scripted
player; passes when three spawned, an enemy attack landed, player hits landed, all three were beaten, the room was
won and the synctest has 0 mismatches. Results are appended below, dated.

## 7. Next (not built)

- Hammer Bro (projectile record), Paratroopa, the Primid variants.
- A boss: Galleom first. Its parameter file decodes with `brawl_boss_dump`; it needs the multi-part hurt set and a
  bone-following hit path as Petey has, and an arena.

## Shared files touched

| file | change |
|---|---|
| `rust/melee_modes/src/tu/petey_model.rs` | added `pub fn load_rigs` (the generic form of `load`; `load` itself and everything Petey uses are unchanged) |
| `rust/melee_modes/src/tu/mod.rs` | two `pub mod` lines |
| `rust/melee_modes/src/tu/envoy.rs` | `subspace::begin_match` at the top of `match_begin`, `subspace::sub_tick` at the top of `tick`, the test knob `MELEE_ENVOY_TEST_MAIN` |
| `rust/melee_modhost/src/lib.rs` | the three `gd_envoy_enemy_*` imports route Subspace handles / `which >= 100` to `subspace.rs` |
| `rust/mods/envoy_sim/src/lib.rs` | the `~N` test knob accepts 100 and up; room wiring (section 5) |
