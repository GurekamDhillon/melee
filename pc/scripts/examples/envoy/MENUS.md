# Envoy rule-host run screens - 2026-10-04 (ux pass; supersedes the older contracts below for `envoy rules on`)

Controller only. Every screen states its bindings at the bottom. Text is plain words (`drive_text.lua`): no ids,
tier labels or internal points. Layout uses existing `gd.kit` panels/lists/text and `g.fill` pips: the look is unverified.

## Reward screen (the hold after a stage clear)
Opens inside the engine's barrier hold (`gd.hold_1p(1800)`). The hold is counted in host ticks, which run at the render
rate: about 15 s at 120 Hz, 30 s at 60 Hz. The title row shows the countdown ("Time left Ns, then new drives are
sorted for you"). Left: rows. Right: the focused drive in words, then `YOUR BUILD` totals (or `IF YOU DO THIS`, before -> after).

Rows: `STAGE REWARD: take one of N` (N = 2, or 3 after a bonus stage and the final boss: two Rare + one Unique) with the
offered drives and `Skip the reward (keep none)`; `EQUIPPED n/slots` (Slot i: name or Empty); `BAG n/12` (`NEW` marks
drives collected this stage); `KEYSTONES n/allowed` (benefit and drawback in the detail pane; "A keystone is one
powerful rule with a built-in drawback"); `Continue to the next stage`.

| Focus | A | X | Y | B |
|---|---|---|---|---|
| offered drive | take it and equip in the first free slot (one press); if all slots are full, take it and ask which to swap | take it, keep in bag | - | jump to Continue |
| bag drive | equip in the first free slot, else ask which to swap | - | discard (asks twice) | jump to Continue |
| equipped slot | move to bag | move to bag | - | jump to Continue |
| keystone | choose / remove (the allowance limits it) | - | - | jump to Continue |
| Skip | asks; a second A skips all offered drives | - | - | cancels the ask |
| Continue | leaves (refused while an offer is unresolved) | - | - | - |
Up/Down move, Left/Right turn the text page.

Swap view (`SWAP OUT WHICH DRIVE?`): the slots plus `Keep it in the bag instead`; the detail pane shows
`Build strength a -> b` and the totals before -> after (green better, red worse) and which drive goes to the bag. A swaps,
B backs out (the taken drive stays in the bag).

Ways out: Continue (A), `Skip` (A, A), the countdown, or the engine ending the hold first. Countdown / release: the first
offer is taken (never discarded), new drives fill free slots in order, the rest stay in the bag. No hold available: same
auto-sort at once. Everything logs: `equipped into slot N`, `swapped out X`, `bagged X`, `discarded X (player choice)`,
`skipped the stage reward: none kept (...)`, `timeout: took X automatically`.

## Bag screen (Z+START in a fight, or `bag`)
Same screen with `YOUR DRIVES`, no offers, no countdown, `Close` instead of Continue. Pauses the game while open
(B or START closes). Z+START is also START to the retail pause: closing may need one more START if retail paused too.

## Build strip (during fights)
Top-left: one pip per slot (fill = drive colour, border = rarity: grey common, blue magic, gold rare, orange unique;
dark = empty), `Strength x.xx`, `Depth n` (`NG+n`), and `Keystone: name`. It flashes ~1.5 s when a slot changes, a drive is
gained or a modifier fires (the modifier name appears under it). The companion stat bars are not drawn on this route.

## Announcements and cards
Centred panel, ~6 s each, once: starter drive, "Fifth slot unlocked", "Keystone allowance: n", "Drive tier n", "New Game+ n".
Bottom card on a drop ("A drive dropped! Walk over it") and on pickup (name + effect + "It is in your bag").
Opponent plate (stage start, ~4 s): `Name (opponent strength x.x)` then its modifiers in plain words, three per page.

## Drops
An opponent drops a drive when it first passes 50% damage and again when it loses a stock (a one-stock fight ends at the
KO, so the drive must be reachable before). It appears on the floor a few steps from you, with the existing hover/spin/glow/beam.
Battle/giant/metal: the first trigger always drops, later ones 25% (max 2). Team: one per opponent (max 3). Bonus: none (the
clear offers 3). Final boss: none on the floor (the clear offers 3). Seeded by run seed and stage. Left on the floor at
stage end (or faded): collected into the bag. Switches in `run_host.lua` `H.tuning`: `drops`, `auto_collect` (false = lost, and logged),
`drop_percent`, `drop_chance`, `team_drops_max`, `offer_count`, `bonus_offer_count`.

Console: `uxdump` logs slots, bag, offers, HUD and the open screen as text; `uxpress <up|down|left|right|accept|back|x|y|start>`; `uxbag`.

---

# Envoy retail 1P menu contract - 2026-10-04 fix1

This retail contract supersedes the default maze/campaign flow below. The old
mission flow remains parked for diagnostics; it is not extended by this packet.

- Asset-free menu and setup default to Classic; Adventure is the second run choice.
  Fighter, difficulty and stocks carry into each NG+ playthrough.
- A selects a drive reward after each cleared retail stage. Three coloured/size
  choices appear in an existing kit panel; the final clear has a larger reward.
  Each choice states its concrete percentage effect. The selected reward plays
  existing drive pickup sounds/FX and animates before/after growth across level-ups.
  It applies after the atomic save succeeds. Coloured
  drive glyphs use existing kit text; retained 3D models have no kit preview API. B cannot
  bypass the reward. A bounded engine hold keeps a broken panel from stranding
  retail progression.
- Opponent leaning-stat tags appear at stage start. Retail teams, giant/metal
  handicaps and AI levels retain their normal behavior; drive multipliers stack
  with those handicaps. Physical in-stage drive drops default off.
- The HUD retains Power, Speed, Guard and Jump bars. Jump changes ground/air jump
  height and air speed; Guard reduces damage/knockback and briefly flashes on a hit.
- Results return to a working menu without assets. The walkable garden appears
  only after its model resolves; failed spawning returns to the menu.
- Master Hand completion (Adventure: its own final boss) evolves a young
  companion, then NG+ continues with the same stats and age. Continue retries use
  the same seeded opponent spread. Game over settles growth and returns to results.
- Older engines show an unavailable-mode message; they must never silently start
  a maze in response to Classic or Adventure.

See the Classic packet report for exact engine API, ending/record policy and
source-only validation. Controller feel and retail presentation need native play.

## Parked mission contract (historical)

# Envoy slices 2?4 menu and garden contract

Supersedes the slice1 menu contract. All panels, text and lists use existing
`gd.kit` components. No new art. Safe-area layout covers 640, 853 and 1140px
canvases. A selects, B returns, START pauses; C-stick retains attacks.

| Surface | Behavior |
| --- | --- |
| Title/Profile | Explicitly open Envoy; Continue enters the physical garden. Corrupt profiles refuse without replacing the file. |
| Garden | Walk freely on the authored mission level. Move near a station and tap A. START opens the garden menu. |
| Run exit, x=-180 | Opens setup; selected fighter launches an offline LAB scene, then the seeded campaign starts after readiness. |
| Fighter, x=-90 | Existing fighter selection list; selection is used at the next run launch. |
| Companion, x=0 | Power, Speed, Guard and Jump grades, nonlinear bars, age/remaining runs, named passive, colours and DNA. The tinted kit placeholder has an Envoy id/type label. |
| Records, x=90 | Runs started, settled runs, wins, best winning time, companions raised and highest observed grade. Unknown legacy history is labelled. |
| Nest, x=180 | Present with a slice5 closed label; no breeding. |
| Playing | Power, Speed, Guard and Jump bars fill with collected points. Receiving bar flashes; LEVEL UP lasts 48 logic frames. START pauses. |
| Interlude | Current drive and level gains, next theme. A explicitly continues; B cannot skip or advance. |
| Pause | Resume, Companion, Abandon Run with safe-default confirmation. |
| Results | Before/after grades, levels and points; boss outcome; evolution/type/passive moment. A/B returns to the garden after cleanup/settlement. |
| Garden return | Sixth settlement shows the reincarnated egg with a notice; the next started run hatches it. |

Menus own only their own pause and D-pad mask. Garden walking and staged cleanup
remain unpaused. A station opens once per press within 22 units horizontally;
release A before activating another. Screens opened by a station return to the
physical garden. Closing Envoy disposes its garden and restores host ownership.
Normal matches remain untouched until the player explicitly enters Envoy.

`envoy hub`, `menu`, `menudump`, `dump`, `status`, `start`, `retry`, `give`, `stop`
and `reset-profile confirm` provide diagnostics. `retry` reuses the last saved
campaign seed; a saved interrupted start resumes its pending seed without aging
again. It restarts traversal, rather than restoring unsaved room/item state.
