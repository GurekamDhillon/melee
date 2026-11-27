# nudge: does a tiny engine difference heal, stay small, or end the game? (2026-11-27, branch port/nudge)

No replay names or player data here; every number is an aggregate. Rows live untracked in the workspace `_build/nudge-out/`.

## Method
- `melee slpcheck --full --slippi-checksum-out` writes Slippi's desync checksum per frame (column 2). New hook
  `SLPCHECK_NUDGE=<frame>:<slot>:<field>:<amount>` (off unless set) changes one field of the active fighter in a slot once,
  before the tick after that frame. Fields: `x`, `y` (cur_pos and collision position), `percent`, `kb_vel.x/y`, `self_vel.x/y`.
  Amount `16u` = 16 ULPs away from zero, `0.01` = that value added.
- `tools/rust-port/slippi/nudge_sweep.py` runs the clean engine and each nudged engine on the same recorded inputs and
  compares the two checksum streams (engine vs engine, not vs the replay) with `melee_core::slippi_online::compare`:
  low halves differing by more than 1 (s16) = Hard (the game would end), high halves differing = Soft, else Same.
  Per run: first Soft, first Hard, whether it returns to Same for good (healed), max low-half delta.
  Classes: invisible (never differed), healed, soft (still differing at the end of the window, never Hard), hard.
  `tools/rust-port/slippi/nudge_summary.py` makes the tables.
- Grid: 20 online replays (2.5 to 6 MB, evenly spaced), one port each (alternating), 3 frames each (first neutral frame
  at or after 600; mid-game; 80% in; fighters alive), fields x, y, percent, kb_vel.y, self_vel.x, amounts 1u, 16u, 0.01
  (percent: 0.1). 900 runs plus 20 no-op controls (all 20 identical to clean: the engine is deterministic).
  Each run is followed for 900 frames (15 s) after the nudge; later healing or a Hard is not seen.
- A ULP step on an exact 0 is a denormal (idle velocity, percent 0). 294 of the 900 runs were from zero; they are
  reported apart. The tables below are the 606 runs that started from a non-zero value.

## Results (non-zero start, 606 runs)
| | invisible | healed | stayed Soft | Hard |
|---|---|---|---|---|
| all | 27.6% | 54.6% | 15.8% | 2.0% |
| 1 ULP (202) | 36.1% | 54.5% | 9.4% | 0.0% |
| 16 ULP (202) | 31.2% | 48.5% | 19.8% | 0.5% |
| 0.01 / percent 0.1 (202) | 20.1% / 0% | 71.4% / 27.1% | 5.2% / 60.4% | 3.2% / 12.5% |

Median frames from nudge to first Hard: 296 (12 runs; range from 67 to a few hundred frames; late nudges 500).
Median frames from first Soft to healed: 160.

By field (invisible / healed / soft / Hard):
- percent (144): 0 / 41.0 / 54.9 / 4.2. Most dangerous: the offset never goes away on its own (it heals only when the
  stock is lost or a reset happens), and a different percent changes knockback on the next hit. 0.1 percent: 12.5% Hard.
- self_vel.x (87): 71.3 / 26.4 / 0 / 2.3. Mostly invisible (friction or the action change eats it); the Hard ones were 0.01.
- x (180): 2.2 / 87.8 / 8.3 / 1.7. Visible at once, usually heals when the fighter lands, is hit or is pushed by a wall.
- y (180): 54.4 / 45.0 / 0 / 0.6. Heals fast (median 5 frames; grounded y snaps to the floor).
- kb_vel.y (15 non-zero): no Hard; the other 165 kb_vel.y runs started from zero and were invisible.

From zero (294 runs): kb_vel.y and self_vel.x nudges (denormals) 257 of 258 invisible; percent from 0: 23 healed, 11 soft, 2 Hard.

1-ULP nudges that became Hard: none of the 202 non-zero-start runs (and none of the from-zero ones). The two smallest Hard
cases were 0.01 y at 67 frames and percent 0.1.

## What it means
- A 1-ULP error in a position or velocity is, in this sample, never a game-ender by itself: about 36% never show in the
  checksum, 55% show as Soft warnings and heal (median 90 frames from first Soft), 9% (almost all percent) stay Soft
  until the end of the 15 s window.
- The Soft/Hard split is mostly about percent. A percent error persists and feeds knockback; at 0.1 one in eight went Hard,
  within a few hundred frames (median 411 for percent). So a bug that changes damage dealt (even by rounding) is the
  one to fear online; position/velocity rounding is not.
- Caveats: one port per replay, 15 s window, 2% Hard overall means the confidence interval is wide (12 Hard runs); a
  Hard needs a hit or a platform/edge event to amplify the nudge, so fights with more contact would show more.
