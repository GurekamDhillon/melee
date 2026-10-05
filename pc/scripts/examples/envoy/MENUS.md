# Envoy rule-host run screens: the grid (2026-10-05; supersedes the older contracts below for `envoy rules on`)

Controller only. One component draws both screens: the grid from `demos/grid-inventory` (embedded into the bundle as
`D.grid`; `tools/port/envoy_bundle.py` wraps `grid.lua` unedited). A drive is a square cell that reads without text: fill = drive colour
(red damage, green speed, blue defence, yellow air, purple status, white wild), border and corner notches = rarity,
pips = how many modifiers, a dot = new, a plus = "the focused drive merges into this one", a lock = slot not yet unlocked.
Text is plain words (`drive_text.lua`): no ids, tier codes or budget numbers; ONE panel describes the focused cell.

## Layout (reward screen and bag screen share it)

| block | cells | shown |
|---|---|---|
| TAKE ONE | the offered drives (3) | reward screen only, while offers remain |
| KEYSTONE: ONE | the offered keystones (3, three different colours) | reward screen only, when the keystone allowance has a step owed |
| EQUIPPED n/slots | always six cells: drive, empty, or locked | always |
| BAG n/4 | four cells | always |
| KEYSTONES n/allowed | the keystones you hold, six per row | when you hold any (a run starts with one) |

The detail panel (right) shows the focused cell: name in words, the base line, one line per modifier, what A would do
("Merges into Green Drive: Lingering: Lingering got stronger.", "Goes into slot 2.", "Goes into your bag.", "Your bag is full:
you will pick a drive to give up."), then `Build strength +49% -> +56%` (any other total only when it changes). The bar at the bottom
shows A / X / Y / B for the focused cell. The title row carries the countdown in the reward screen and short notices.

## Buttons

| Focus | A | X | Y | B |
|---|---|---|---|---|
| offered drive | the obvious thing: merge into a matching drive, else equip into the first free slot, else the bag, else ask which to replace | to the bag (only when the bag has room) | - | skip the rest (asks; a second B skips) |
| offered keystone | take it | - | - | skip (the allowance stays owed) |
| bag drive | merge into a matching drive, else equip into a free slot, else ask which equipped drive to swap with | - | discard (asks twice) | close / continue |
| equipped drive | to the bag (when the bag has room) | to the bag | - | close / continue |
| empty / locked cell | says what it is | - | - | close / continue |
| held keystone | read only: you keep keystones for the whole run | - | - | close / continue |

D-pad or stick moves one cell (it wraps; Up/Down jump between blocks). B with nothing left on offer continues; in the bag screen B or START closes.

Swap layout ("BAG FULL" / "SWAP"): the incoming drive, the six equipped cells and the four bag cells as targets. A replaces the focused
drive (an equipped drive goes to the bag when it has room, otherwise it is gone, and the panel says which before you press); B backs out.
When a picked-up drive has nowhere to go this opens by itself and pauses the game; B twice leaves the drive behind (logged).

## Rules the screens enforce

- Taking a drive into a free slot never needs bag space (`bag:place`), so a full bag with an empty slot 6 still takes the reward.
- A gained drive always resolves to merge / equip / bag / an explicit choice; nothing is dropped silently, and every outcome logs.
- Keystones come one per allowance step (1 + one per five depth): a run starts with one random keystone (announced with the starter
  drive in one panel); at a stage clear with a step owed, a pick of three from different colours that are legal with what you hold.
- Rewards: a pick of three at every third stage (2, 5, 8...), every bonus stage and the boss; the final boss offers two Rare and one Unique.
- Ways out of the reward screen: B (continue / confirmed skip), the 45 s countdown, or the engine ending the hold first. On a countdown
  the first offer is taken (merge, free slot or bag, never discarded when it can be kept), new drives fill free slots.
- Script cost: the blocks and the detail text are rebuilt only when the bag, the offers or the focus change; a drawn frame replays the
  component's cached layout. `uxcost` logs the measured cost (`uxcost reset` first).

## Bag screen (Z+START in a fight, or `bag`)
The same grid with `YOUR DRIVES`, no offered blocks, no countdown, `Close` for B. It pauses the game while open. Z+START is kept from the game
by `gd.input_chord`, and the START that closes it is hidden until released, so it does not also pause.

## Build strip (during fights)
Top-left: one pip per slot (fill = drive colour, border = rarity; dark = empty), build strength as a percentage (`+56%`), `Depth n`
(`NG+n`), and the keystones you hold as small cells (an initial on the family colour, six then `+n`). It flashes about 1.5 s when
a slot changes, a drive is gained or a modifier fires.

## Announcements and cards
Centred panel, about 6 s, once: the starter drive and starting keystone (one panel), "Fifth slot unlocked", "Keystone allowance: n.
A keystone is offered at the next stage clear.", "Drive tier n", "New Game+ n". Bottom card: "A drive dropped!" plus its one short line, "Picked up: <short
name>", "Merged!" with the drive and what got stronger, "Bag full: <name>". Opponent plate (stage start, about 4 s): its modifiers one line each
(keystones: name and effect), three per page.

## Drops
At most one drive per stage on the floor: battle/giant/metal 70% (at the first trigger: an opponent passes 50% damage or loses a stock),
team one, bonus and boss none. Opponents that spawn after the stage began (Adventure side-scrollers) are registered when they appear,
rolled once per port per stage, and can drop. The match does not end while a drive is on the floor: pick it up (or it fades, or 30 s
after the last opponent) and then the stage ends. The numbers are `drive_economy.lua` `E.tuning`; the switches `drops`, `auto_collect`,
`drop_percent` are in `run_host.lua` `H.tuning`. Left on the floor at stage end: gathered through the same gain rule.

Console: `uxdump` logs slots, bag, keystones, offers, HUD and the open screen as text; `uxpress <up|down|left|right|accept|back|x|y|start>`;
`uxbag`; `uxcost [reset]`; test hooks `uxgain [n] [merge]` (gain rolled drives through the pickup rule) and `uxpreview [keys]` (open the reward grid with real offers).

---

## Technique modifiers in the grid (skill layer, 2026-10-05)

A technique modifier reads "<when>: <what>" on the first line of its detail panel (for example "L-cancel a landing after the aerial hit: Haste for
2 seconds."), and a second line says what to look for ("You earn it by technique: a blue afterimage shows while it lasts."). A crit modifier says
"Your hits crit 5% of the time." and "Crits are rare: the engine has none until a modifier grants a chance." There are no ids, tiers or numbers
from the budget in either line. Technique and crit modifiers never roll before their depth (see PLAYTEST.md), so a stage-1 drive stays one plain
effect. The first time a technique rule fires in a run, the build strip shows one toast ("Technique rule fired: ..."). A strong crit also toasts
("Critical hit x2.0"); a weak one only plays the light impact frame.

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
