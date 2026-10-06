# Atlas screens (gd.ui)

Shows `gd.ui`, the Atlas screen binding, on two screens. The list has a toggle, a choice, a slider, a text value and a disabled row. The grid has locked, empty, merge and new cells, an origin mark and pips. Both show the explainer, the key hints, a corner note (X) and a dialog (Y). Each `on` handler is used once: `change` (toggle, choice and slider rows), `accept` (the text row and grid cells), `back`, `alt` X and Y, `page` (L or R, and Tab, switch between the list and the grid), `start` (a note), `focus`, `open` and `close` (log lines). Presentation only: it reads and writes no game state and works online.

Launch with `MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"`, then in the console `load <absolute path of this folder>`. F7 opens or closes it; D-pad or arrows move, A changes a toggle or choice row, left and right also step a slider or choice, X shows a note, Y a dialog, START a note, L or R (Tab on the keyboard) switch between the list and the grid, B closes. Mouse: hover focuses, left click is A, right click is B, the wheel moves the focus on the list.

No disc-derived art; every pixel is a part drawn by the engine. Needs the Atlas font roles in ui/ (menu/pipeline/font_atlas.py); without them gd.ui.available() is false and the demo does nothing. Not yet seen in the game.
