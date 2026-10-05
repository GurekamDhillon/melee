# Grid inventory

A reusable grid-inventory menu component (`scripts/grid.lua`) and a demo (`scripts/main.lua`, fake data in
`scripts/layouts.lua`). Blocks of cells read at a glance without text: main colour (drive family), border colour plus
corner notches (rarity: none / 1 / 2 / 4), pips (1-4), flags (NEW, can merge, equipped, locked). One detail panel for
the focused cell, an action bar (A/X/Y/B), a compare mode (before -> after) and an optional countdown in the title.

Launch (offline LAB, idle CPU; the demo only draws, it consumes P1's D-pad while shown):

    MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"
    MELEE_SCRIPT=<absolute path of this folder>

Controls: D-pad / stick move (wraps), A/X/Y/B act on the focused cell (logged, nothing changes), pad L or TAB cycles
the layouts BAG (4/5/6 slots), REWARD (2/3 offered, compare), FULL SLOTS (swap out or keep in bag); F6 hides it.
Console: `gi_layout <bag4|bag5|bag6|reward2|reward3|swap>`, `gi_press <left|right|up|down|A|B|X|Y|L>...`,
`gi_state`, `gi_cost [reset]`, `gi_bench [frames]`, `gi_skip <mask>` (profiling).

Focus rule: the nearest focusable cell strictly in that direction (distance plus twice the sideways offset); holes
(missing cells of uneven rows) are skipped, blocks are crossed in the same row/column; at the edge it wraps to the far
side keeping the row/column (`wrap = false` turns that off).

Layout: computed from `gd.safe_area()`; composition at most 760 wide inside the title-safe margins. The cell edge is
the largest of 56..24 px that fits. Minimum: the 640x480 canvas (every window is at least that wide); the largest layout
(BAG with 6 slots) keeps 51 px cells there; below 24 px the layout reports `fit = false`.

Cost: no table or string is built per frame (layout, wrapped detail text and action bar are rebuilt when data, focus or
screen size change). Measured about 0.3-0.4 ms of script time per `on_draw` (frozen build, 1024x576 and 768x576).

Icons: the default is a flat colour block (kit-free primitives); `icon = "crown"` uses a kit icon; a table-valued
`icon = {kind = "model", ...}` is handed to `view.icon_draw(desc, x, y, w, h, focused, locked, cell)` inside the
reserved square rectangle that `view:cell_rect(block, index)` reads back (the demo's placeholder draws a squeezing bar).

Tests: `lua melee/pc/tests/grid_inventory.lua` from the workspace root. `scripts/embed.py` copies grid.lua and
layouts.lua into main.lua (the engine loads one entry file per mod); `--check` verifies it.

Verification: loaded and driven in the frozen audit build on the vanilla LAB, layouts read back by console and looked
at in window captures at 16:9 and 4:3. No disc-derived art; every pixel is a fill, box or kit panel/text/glyph/icon.
Credit: GD scripting reference and the Envoy mod's menu conventions; no third-party code or assets.
