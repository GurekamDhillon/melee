# Envoy crests core (full-game plan milestone 1, 2026-11-10)

Crests replace the six placeholder upgrades (the old drive system stays removed). A crest is one rule and one number,
worn by one fighter of the pair, firing at a moment (envoy-redesign.md 2, full-game-plan.md 4.2). This milestone is
the core: the pool as data with a lint, the moments, the four words, a per-fighter wearer, the full state digest, a
first pool of 26, the offer at the start of fights 2-5. Offers with A/B/C rules, hones, two players and the harness
are milestone 2; heavy crests and prices are milestone 3.

## Pieces

| piece | where |
|---|---|
| pool | `rust/mods/envoy_data/pool.toml` -> `build.rs` -> `pool_gen.rs` (`POOL`, `POOL_LEN`, `POOL_DIGEST`), a `no_std` library used by `envoy_sim` and `envoy_ui` |
| vocabulary | `envoy_data/src/codes.rs`: 7 families, 11 moments, 17 effects, the four words, the glossary, `MARKED_DEFENSE` 1.2, `FRESH_FRAMES` 180, `BURN_TICK` 60 |
| lint (`envoy_lint`) | `envoy_data/src/spec.rs`, run by build.rs (a problem fails every Envoy mod's build) and by `cargo test`: <= 12 words, <= 2 numbers, the moment belongs to the family, gives / reads from the four words and written in the rule, a word in the rule is one the crest gives or reads, no duplicate key / name / rule, every read word has >= 2 givers (Fresh and Benched are given by every tag), every family >= 4 members, weight 1-3, no HEAVY yet (needs a price) |
| sim | `envoy_sim`: worn crests `[2 fighters][3]`, the moments, the windows, the offer; the whole state is one `#[repr(C)]` struct of i32 words, and every frame an FNV-1a over it, the plan and `POOL_DIGEST` goes into `gd_hash_word` |
| host | `gd_envoy_apply` gains a fighter selector (`| 0x100` main, `| 0x200` partner) for weight / speed / jump; `gd_envoy_percent(slot, bench, delta)` heals or burns the active or benched fighter; `gd_envoy_test_crests` (knob); `VIEW_WORDS` 64 -> 160; the run record 16 -> 32 words, version 2 (crests at 14..20, the pool digest at 20; a v1 record or one from another pool is dropped) |
| UI | `envoy_ui`: three crest cards (name, family, WORN BY, rule, the word's glossary line), a `+1 STOCK` card, the "FOX WEARS KEEN" line after a pick, the pair's crests in the between-fights band and the camp. Crests are not drawn in fights (redesign Q7) |

## Moments and the four words

| moment | when the sim sees it |
|---|---|
| tag in / tag out | the frame a tag freeze releases (transition kind 1); the incoming fighter is **Fresh** for 3 s, the outgoing one **Benched** |
| shift | the frame a stage-switch freeze releases (kind 2) |
| stock lost | the pool (`gd_player_stocks(0)`) falls and is still > 0 |
| KO dealt | the CPUs' stock total falls |
| opener | the first unfrozen frame of a fight after its offer |
| boss | the opener of fight 5 (no crest keys on it yet) |
| windows | always (while the wearer is in), Fresh (wearer's window), Benched (the bench, Rest heals 1%/s), last stock (pool at 1) |

Words: **Fresh** (a per-fighter window, RELAY crests act in it), **Benched** (the fighter on the bench), **Marked**
(a foe's defense ratio x1.2 for the window), **Burning** (a foe takes 1% every 60 frames for the window). Every window
counts unfrozen frames only, so a tag or shift freeze does not eat it. The fighter in play is tracked by kind (a
respawn, a refused swap) and by the tag release.

Ratios: the slot's attack and defense ratio are recomputed every frame for the fighter in play (its ALWAYS, Fresh and
last-stock crests, Handoff's boost) and written only when they change; a Marked foe's defense likewise. Weight, speed
and jump crests multiply the wearer's fighter object once per fight (with the legacy two-fighter tag; the in-place
prototype `MELEE_TAG_INPLACE=1` respawns the fighter and loses them).

## The first pool (26)

| family | crests |
|---|---|
| ALWAYS (5) | Power (hits +20%), Guard (takes -15%), Heavy (weight +20%), Swift (walk/dash +12%), Spring (jump +12%) |
| RELAY (5) | Keen (while Fresh, hits +35%), Mend (tag in: heal 12%), Hunter (tag in: nearest foe Marked), Linger (Fresh 6 s), Quick (tag cooldown 3.5 s, party) |
| BENCH (4) | Rest (Benched: heal 1%/s), Charge (Benched 10 s: next Fresh 6 s), Ember (tag out: nearest foe Burning), Handoff (tag out: partner +15% for 5 s) |
| SHIFT (4) | Refit (both heal 8%), Exposure (every foe Marked), Second Breath (active fighter Fresh), Wildfire (every foe Burning) |
| POOL (4) | Rebirth (stock lost: respawn Fresh 5 s), Last Stand (last stock: hits +25%), Spite (stock lost: every foe Burning), Trophy (KO dealt: heal 15%) |
| OPENER (4) | Head Start (Fresh 10 s), Softened (foes start at 20%), Early Shift (first shift at 15 s), Called Shot (every foe Marked 6 s) |

The plan's other ~8 (intangible tag-in, shield crests, the schedule-later shift, +1 stock once per fight, Guard at pool 1)
need `gd_fighter_set_intangible` / shield writes or are better as offer kinds; the two hit-readers wait for the hit view.

## The offer (milestone-1 rule)

At the start of fights 2-5: three crests from three different families, none already worn, rolled from
`seed, fight, picks` (a retry is the same). Wearers: the fighter with fewer crests first, then alternating, never a full
fighter (3 slots each). When the pool is at 1, the third card is `+1 stock`. A takes, D-pad / stick left-right moves,
20 s then the first card.

## View words added (envoy_sim `publish`)

13-15 offers (pool index or 1000 = stock), 16-18 their wearers, 19 stocks now, 22-27 worn crests (index + 1), 38 the
fighter in play, 39 the last pick's wearer, 72 fires, 73 last fire, 74 frames since, 75 pool digest, 76 stage
countdown (moved from 27), 77-78 Fresh frames left.

## Knobs and checks

| knob | effect |
|---|---|
| `MELEE_ENVOY_TEST_CRESTS="key,key/key"` | main / partner wear these from a run's start (fixed for the process) |
| log | `envoy: crest fire <key>` (value: wearer), `crest taken`, `moment ...` |

`tools/rust-port/envoy/m1_crests.sh`: the lint tests; a pad run from fight 2 (tags every few seconds, A takes the offer,
Early Shift brings the first shift to 15 s) under synctest depth 4 for 3600 frames, requiring 0 mismatches and the
opener, tag-in, tag-out and shift crests to fire; a CPU run (`MELEE_ENVOY_TEST_P1CPU=1`) requiring Softened and a
KO-dealt or stock-lost crest.
