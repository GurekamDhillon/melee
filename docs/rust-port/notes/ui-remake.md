# UI remake: squished and smushed Envoy UI (port/uiremake, 2026-11-14)

The owner on candidate 10: "a lot of the UI probably needs to be remade, we're still getting some smushed and
squished UI elements in game". The uiquality lane (ui-quality.md) had already made the renderer exact: analytic
shapes, a mipmapped atlas, distance-field text, square pixels in the band. This lane measured every Envoy surface on
candidate 10, fixed what was left in the kit and the screens, and added a check that keeps it fixed.

## What was wrong

The band's pixels are square and the host never scales text sideways (a line that does not fit is cut, not
squeezed), so a piece can only come out wrong when a mod asks for it. Measured over every surface, at 1280x720 and
1920x1080, widescreen and 4:3:

| # | What | Where | Cause |
|---|---|---|---|
| 1 | The bench fighter's head squashed sideways, down to 15% of its width | fight HUD, on a tag | the flip animation scaled the sprite's width alone |
| 2 | One-letter hint caps ("A", "B") were ovals, 22.5 x 20.5 | every hint line | the cap was the letter's width plus 12, 18 high |
| 3 | Words cut off: "Optio", "Word", "Comb" | pause tab bar | six tabs each a sixth of a 216-unit bar, the words cut to the tab |
| 4 | "CAPTAIN FALCON" cut to "CAPTAIN" | fight HUD bench sticker | 44 units for the name in a fixed 84-unit sticker |
| 5 | Tab names abbreviated to fit: "Kit", "Fire", "FX", "State" | Item Lab | six tabs each a sixth of a 284-unit bar |
| 6 | The room-choice sentence cut by a character | room choice | a fixed 186-unit line |
| 7 | Every item sprite ran over its frame's border | everywhere an item shows | the sprite was drawn at 3/4 of the frame's cell; the frame's opening is about 2/3 of it and the sprites fill their own cells |
| 8 | Frames overlapping each other, "x2" written over the next icon | build rows (camp cards, run end) | a step of the icon's size + 4 for frames 4/3 of that size; the count was loose text |
| 9 | The new-item card half under the Item Lab panel | Item Lab | the card is centred on the screen, the panel covers the left of it |
| 10 | The equipped icons ran off the panel's foot onto the HUD | Item Lab, Loadout | four rows at 26 a row in a fixed-height panel |
| 11 | The trigger log on top of the stock row | Item Lab | both at the bottom centre |
| 12 | "on while shielding / stack: more" written over the item's name | shop item card | name left-aligned, trigger right-aligned, one line |
| 13 | "+5" payouts written over each other and over the room label | fight HUD | every payout drawn at one spot |
| 14 | Map rows sitting on each other | pause, Run page | eight rows of radius-7 nodes in 118 units |
| 15 | The map cursor as "a few red specks" | act map | a coral burst barely larger than the node, behind it |
| 16 | The Envoy HUD (0%, the bench sticker, a heart) over the results screen | after the last stock | the HUD drew whenever the view said "live" |
| 17 | The HUD showing through the shop as smudges | shop | the shop's backdrop is a tenth see-through |
| 18 | "MR. GAME & WATCH" off both sides of its card | camp fighter cards | a 66-unit card and a name 79 wide at the smallest size |
| 19 | The fighter portrait 3.7% too wide | camp fighter picker | the disc's 136x188 portrait drawn into a typed 60 x 80 box |

Rows 1, 2 and 19 are the only true proportion errors; the rest is layout. Checked and not a cause: every kit
sprite and every `em_*` item sprite, frame and heart is drawn at its art's own proportions (0.0% off); the camp
backdrop (427x240 art over 854x480) is exact; the disc's fighter icons (64x56) sat within 1.3% of their typed boxes.
Fighter art is now placed by its real size (`gd_ui_tex_size`) instead of typed boxes.

A capture artefact that no run from the menus shows: a match started straight from the command line with the arm
knob (`--match` with `MELEE_ENVOY_ARM`: the checks) kept the retail HUD on screen under the Envoy HUD, because
`envoy::tick` looked at the camp's flag only. It now uses `armed()`, so the checks see what the owner sees.

## How it is built now

**Kit (envoy_ui kit.rs).**
- `contain`, `tex_fit`, `tex_in`: art is placed inside a box at its own proportions, centred. `spr` contains: a kit
  sprite can no longer be stretched by its box. `tex_in` asks the host for a texture's real size.
- `text_fit`, `fit_role`, `wrap`, `wrap_lines`: a line that is too long is set smaller (down to a floor) or wrapped
  on whole words. Nothing passes a width to the host's text op any more, so nothing is cut.
- `pill`, `pill_w`: a pill is as wide as its words; a one-letter pill is a circle, never an oval.
- `tab_row`, `tab_rows`: tabs as wide as their words, the room left over shared out; two rows when one cannot hold
  them.
- `rich_lines`: a card measures its keyword text first and is as high as that.
- `item_icon` puts the sprite inside the frame's opening; `item_step` is the step at which frames stand clear;
  `count_chip` is the stack count as a pill on the icon's foot.
- The clear span (`claim_left`, `claim_right`, `mid`, `clear_w`): side panels claim their side of the screen, and
  the centred cards (new item, room banners, callouts, synergy banner) centre in what is left.

**Layout (envoy_ui ly.rs, melee_modhost ui/layout.rs).** A screen can describe itself as a flexbox tree: rows and
columns, gap, padding, fixed and percentage sizes, grow, align and justify, aspect-ratio boxes, wrapping rows, text
leaves measured with the font they are drawn in. `St` is the style builder (`St::row().gap(6.0).pad(10.0)`), `Lay`
the tree, `P` what a node draws. The pause panel (every page), the Item Lab panel's frame and the run-end card are
laid out this way; the act map and the fight HUD are not (they are positions, not boxes).

The compute is Taffy 0.15, run natively by the host behind one import, `gd_ui_layout`: the mod sends the tree (a
few hundred words) and gets four floats a node back. It first ran inside the mod, Taffy compiled to wasm: the
interpreter needed 2.8 ms a present for the pause panel, and with a cache 3 to 4.5 ms on every present whose
content changed, with four times the presentation fuel. The mod still hashes its tree and only asks when the hash
changes, so a present that changes nothing makes no call. Layout is draw-only: no value reaches the sim, and the
same request and font give the same rectangles on every machine (the host computes in f32 with no rounding).

Cost of the layout (hidden 1280x720 widescreen runs, the mod's frame call in microseconds; the pause run's pad
script changes page or cursor every few presents, so its p99 is a present that was laid out again):

| | p50 | p99 | max |
|---|---|---|---|
| pause panel, hand-laid (before) | 94 | 190 | 370 |
| pause panel, Taffy inside the mod, every present | 2846 | 3969 | 7905 |
| pause panel, Taffy inside the mod, cached | 132 | 3133 | 4496 |
| pause panel, Taffy in the host, cached | 137 | 360 | 710 |
| Item Lab panel, Taffy in the host, cached | 62 | 193 | 492 |

The Lab's own perf run (34,393 presents, every item fired): inside the Lab session no present over budget, all wasm
mods together p99 0.62 ms, 1.29 ms at worst. The mod is 309 KB (280 KB before any layout code, 381 KB with Taffy
inside it). `UI_FUEL` is unchanged at 4M.

**Renderer (melee_gx ui.rs).**
- `SHAPE_SHADOW`: a Gaussian soft shadow of a rounded box in closed form, written from Raph Levien's "Blurred
  rounded rectangles" (2020). One quad at any blur. Under the pause and Lab panels and the run-end card.
- `SHAPE_BOX4` / `OP_SHAPE4`: a box with a radius for each corner (the run-end card's head).
- UI supersampling: see below.

**Screens.** The pause panel is 312 wide (two players on a 4:3 screen: 298 each, with the tabs on two rows); the Lab
panel 300 to 380 with its pages' full names; the fight corner is one column (tag and name, percent, bench sticker,
hint); the bench sticker is as wide as its name; payouts rise beside the purse; the map's node radius is what a row
and a lane leave, its cursor a ring with the pointer beside the node.

## The check

`MELEE_UI_ASPECT=1` turns on the host's audit (`melee_modhost` ui/cmd.rs `Audit`). Every draw is looked at as it is
encoded, and each distinct finding is logged once, the bad ones marked `BAD`:

- a sprite whose drawn width / height is more than 2% off its art's (a cell of a sheet: off its cell's);
- a box with fully round ends whose sides differ by 2% to 25% (a circle that is not round);
- a line cut at its width;
- a line that starts on a sticker and runs past its side, a line that leaves the screen, a line over another line;
- two sprites of one layer that cross over more than an eighth of the smaller one.

Text has no horizontal scale to check: the host's text op has none (`Batch::text` places glyphs at their advances
at one uniform size). What it can do to a line is cut it, and that is flagged.

`tools/rust-port/envoy/ui_aspect.sh` puts every surface on screen with scripted hidden runs (pause, lab, camp, room,
shop, devil, angel, secret, synergy, coop, pickup, run) at the configurations asked for and fails on any `BAD`
line that `ui_aspect_allow.txt` does not list (two entries: bursts behind bursts, the cash-out's coins in flight)
and on a surface that never showed. `lab_check.sh` and `pause_check.sh` run with the audit on and fail the same way.
`cargo test -p melee_modhost --lib ui::` covers the audit, the layout request and the v1.5 records.

## UI supersampling

The owner's idea: render the UI at 4K and scale it down. `MELEE_UI_SS` draws the UI batch into its own target at a
whole multiple of the window (the smallest that reaches 2160 lines, at least 2: 720p 3x, 1080p 2x, 4K 2x), box
filters it down (an exact area average) and blends it over the game. It changes how edges, glyphs and minified
sprites are sampled. It does not move or reshape anything: squished pieces are the layout's business, above.

Measured (`cargo test -p melee_gx --test ui_quality supersampled -- --nocapture`: a frame the size of the pause
overlay, timed to GPU completion, RTX 5070):

| Window | Factor, target | Target memory | Straight | Supersampled | Added |
|---|---|---|---|---|---|
| 1280x720 | 3x, 3840x2160 | 31.6 MB | 0.102 ms | 0.233 ms | +0.131 ms |
| 1920x1080 | 2x, 3840x2160 | 31.6 MB | 0.119 ms | 0.232 ms | +0.113 ms |
| 3840x2160 | 2x, 7680x4320 | 126.6 MB | 0.205 ms | 0.560 ms | +0.354 ms |

The CPU side does not change (frame profiler, pause overlay up: draw 0.72 ms and present 0.15 ms p50 either way).
The same test checks the picture: no pixel differs by more than a third of full coverage and under 0.3% of the
frame's coverage moves (measured: 0.05% at 720p, 0.02% at 1080p).

So it is on by itself only where the target stays within 3840x2160 (windows up to 1080p: 32 MB, about 0.1 ms) and
off at 1440p and 4K, where the target would be 56 to 127 MB for edges the display's own pixel density already
resolves. `MELEE_UI_SS=0` turns it off, `MELEE_UI_SS=2` (1..4) forces a factor at any size.

What it buys is small, because the shapes were already analytic and the text a distance field: glyph stems and
diagonal edges get a true area average instead of a one-pixel coverage ramp, and minified sprites are filtered from
more texels. Crops: `scratch/morning/uiremake/supersampling.png`.

## Results (tip 88a114a8b and after)

- `ui_aspect.sh`, 1280x720 widescreen and 960x720: 24 of 24 scenarios in proportion (candidate 10's mods under the
  same audit: 105 findings at 1280x720 widescreen over the twelve scenarios, 17 of them the allowed bursts).
- `pause_check.sh`: nav and abandon ok, with the audit on.
- `lab_check.sh`: flow ok with the audit on; sync depth 4, 3000 frames, 0 mismatches. perf: no present over budget
  inside the Lab session; the leg still exits non-zero on frames before the Lab opens (a 23 ms OS/loop frame in the
  boot scene on one run; match-setup engine steps on a second run the check itself marked "not quiet, re-run").
- `items_check.sh` depth 4: 27 ok. `gate.sh`: GATE OK (13 ok).
- `cargo test -p melee_modhost --lib ui::`: 20 passed. `cargo test -p melee_gx --test ui_quality`: 7 passed.
- Sheets: `scratch/morning/uiremake/index.md` (local).

## Limits

- The Lab's row pages and the fight HUD are still placed by hand (with the kit's fitting helpers). They pass the
  check; they are not flexbox trees.
- The audit sees what is drawn, so a surface the scenarios never reach is not checked. The scenarios cover the
  surfaces listed in ui_aspect.sh; the Petey boss bar and the older drive HUD (no item rules) are not among them.
- The retail pause banner's box glyphs around "P1 Pause" are the game's own text, not the Envoy UI.
