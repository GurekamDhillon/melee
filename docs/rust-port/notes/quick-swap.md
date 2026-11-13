# Quick-swap cast (Envoy; branch port/quickswap1, 2026-11-13)

Builds on `envoy-tag-swap.md` (the in-place tag, `MELEE_TAG_INPLACE=1`) and Envoy Quick Tag (`tag_swap::quick`). Those
already hold the mechanism; this lane adds a plain button mod for it, measures it, and decides the open points.

## Mechanism (recommended: A)

- **A. In-place despawn / spawn, no freeze (recommended).** The port owns one fighter object. On the swap, between logic
  frames (after the post chain): `Fighter_Create` the benched character as a sleeping sub-fighter in the free entity
  index, run the retail transform swap onto it (`ftCommon_8007EFC8`), free the outgoing fighter, bench its character
  and % in plain statics (`TAG_BENCH_CK`, `TAG_BENCH_PCT`). Code: `tag_swap::quick` + `swap_inplace`.
- B. Sleeping sub-fighter (the default tag): faster wake, but a whole live fighter per partner and it takes the port's
  sub-fighter slot (no Ice Climbers / Zelda / Sheik in a party).
- C. Swap behind a freeze cinematic (tag_fighter): hides any cost, but it is not "almost instant".

## Preload

- When: at scene preload (`Player_80031CB0`), every configured partner's files are loaded with the slot's, so the swap
  never reads the disc.
- What is created at the swap: only the fighter object (GObj, joints, its heap blocks); the archive is shared.
- Memory per preloaded partner: the partner's three character files stay resident for the match (measured from the
  profiler's load events: Marth PlMs 170 KB + PlMsNr costume 581 KB + PlMsAJ animations 1897 KB = 2.6 MB; Falcon
  2.2 MB; Fox 2.0 MB). The live fighter costs its object only while it is the active one. heap0 free moves by about 7.7 KB between the two characters (Falcon vs Marth), i.e. the object.
- A warm-up spawn at match start is not needed: the first swap costs 2.3-2.5 ms (cold), later ones 0.4-0.8 ms.

## State carried across a swap

- Carried (retail transform field list): position, velocity, facing, ground / air, input, camera, jumps used.
- Per character (benched): its own %; restored on the way back in.
- Per port (shared): stocks, Envoy drives / buffs (they live on the port in Envoy data, not on the fighter).
- Not carried: invulnerability frames (the incoming fighter enters Wait / Fall clean), hitlag, charge states.
- Held item: the swap is refused while holding one (`quick` rule); recommended later: drop it in place at the swap.

## Input

- D-pad down, as a press (not a hold). Not a retail in-match control (D-pad up is the taunt), same as tag_fighter's
  classic tag. Envoy keeps its own tag button setting (`gd_envoy_tag_button`: Y / X / D-pad down); the mod is for
  plain versus.
- Refused (quietly, the press is dropped) while not free to act: dead / entering, hitstun / tumble, down / tech, own
  hitlag, grabbing / grabbed, holding an item, on the ledge, another swap queued.

## Cooldown

- 30 logic frames (0.5 s) per port after an accepted swap. Long enough that mashing cannot flicker characters; short
  enough to feel tag-style.

## Co-op and CPU partners

- Each port has its own partner and bench; ports never share a bench. P2 in co-op swaps on its own D-pad down.
- A CPU port: the mod reads the CPU's synced input like any port; CPUs do not press D-pad down today, so they never
  swap. Recommended: leave CPU swapping to Envoy's own rules (it already queues tags for CPUs: `tagauto`).

## Rollback netplay and replays

- Everything is sim state: the bench statics, the mod's cooldown (its linear memory), the queued swap. The swap reads
  only the synced input, so peers and replays swap on the same frame.
- Synctest at depth 7 across ~176 swaps: 0 mismatches (rollback passes resimulate the swap: 352 swap executions).
- Cost inside rollback: a resimulated swap costs the warm figure (~0.5 ms) per pass; at depth 7 that stays well under a
  frame.

## Measured (release build, windowed real time, MELEE_PROFILE=1; `tools/rust-port/quickswap/check.sh`)

Falcon <-> Marth vs a level-1 CPU Fox on FD, D-pad down every 40 frames, 9000 ticks (210 presses).

| | vanilla | ACE |
|---|---|---|
| swaps applied / presses refused (not free to act) | 177 / 33 | 177 / 33 |
| swap cost, first (cold) | 2.53 ms | 2.53 ms |
| swap cost, worst after the first / median | 0.77 / 0.57 ms | 0.79 / 0.56 ms |
| worst over the first 50 swaps | 2.53 ms | 2.53 ms |
| frames over 16.7 ms (zero_hitch_check) | 1 + startup: tick 11, GPU pipelines, before any swap | 2 + startup: tick 3 (pacing) and tick 658 (frame 20.8 ms with a 0.0 ms tick: not game work) |
| heap0 free, swap 2 -> swap 177 | 26.82 -> 26.75 MB | 26.66 -> 26.59 MB |
| heap1 free | flat (6.88 MB) | flat |
| synctest depth 7, 2400 frames (352 swap executions incl. resimulation) | 0 mismatches | 0 mismatches |

- Leak: heap0 free is exactly flat across 32 consecutive swap pairs (64 swaps) in the middle of the match; the 66 KB
  total drop comes in a few steps that do not repeat per swap. Not a per-swap leak; a no-swap control run would
  attribute the steps.
- Envoy room (`v3_slice.sh qtagcoop` with `MELEE_TAG_INPLACE=1`, depth 7, 3600 frames): 80 in-place Quick Tags, 0
  mismatches. First spawn there 8.4 ms (cold, synctest), ~3 ms during resimulation. `qtag` (solo) shows no tag with or
  without in-place on the base build: a pre-existing pad / tag-button mismatch, not this lane.
- ACE extra-slot fighters as partners: not tried.

## Risks

- Ice Climbers / Zelda / Sheik as the outgoing character: refused today (the free entity index is taken). Fix is the
  "despawn everything on the port, spawn the incoming set" step in `envoy-tag-swap.md` section 4.
- Effects / projectiles owned by the outgoing fighter: freed with it (as a KO would); a projectile in flight vanishes.
- First swap on a slow machine: 7.6 ms was seen headless on two peers; still under a frame, watch it.
- Per-swap heap0: flat over long stretches, with a few small unrelated-looking steps (see results); a dedicated
  no-swap control run would settle it.
