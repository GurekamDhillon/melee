# Nudge experiment: do tiny state differences heal? (2026-11-27)

Question: if one engine's state is off by a tiny amount (the size of a typical port bug), does Slippi's desync
checksum (`melee_core::slippi_online::compare`) see it, and does it heal, stay a warning, or end the game?

## Method
- `SLPCHECK_NUDGE=<frame>:<slot>:<field>:<amount>` (rust/melee/src/slpcheck.rs): at the start of that frame, change
  one fighter field. Fields `x` / `y` (cur_pos and the collision position), `percent`, `kb_vel.x/y`, `self_vel.x/y`.
  `Nu` = N ULPs away from zero, a decimal = that value added. Off unless set.
- `tools/rust-port/slippi/nudge_sweep.py`: per replay, one clean run and the nudged grid; compares the two engines'
  Slippi checksums frame by frame (engine vs engine, not vs the replay) for 900 frames after the nudge.
  Classes: invisible (never differs), healed (differed, back to Same and stays), soft (high half still differs at the
  end), hard (low halves more than 1 apart = Slippi ends the game).
- `tools/rust-port/slippi/nudge_summary.py`: the tables below.
- Grid: 20 online replays (hero set), nudges at early neutral (frame >= 600), mid-game and 80% in, fields x, y,
  percent, kb_vel.y, self_vel.x, amounts 1u, 16u, 0.01 (percent 0.1). 900 nudged runs + 20 zero controls.

## Results
Controls (0u): 20/20 invisible, so the engine is deterministic run to run. Errors: 0.

Runs whose field started non-zero (606; a nudge from exact 0 only makes a denormal, counted apart):

| | n | invisible | healed | soft | hard |
|---|---|---|---|---|---|
| all | 606 | 27.6% | 54.6% | 15.8% | 2.0% |
| 1 ULP | 202 | 36.1% | 54.5% | 9.4% | **0.0%** |
| 16 ULP | 202 | 31.2% | 48.5% | 19.8% | 0.5% |
| +0.01 | 154 | 20.1% | 71.4% | 5.2% | 3.2% |
| percent +0.1 | 48 | 0% | 27.1% | 60.4% | 12.5% |

By field: x heals most (87.8% healed, typically when the fighter lands or is snapped by collision); y is mostly
invisible or heals within a few frames; percent never heals by itself (it persists until a stock is lost) and is
the field most likely to grow into Hard (via different knockback); velocity nudges are invisible until they move
the fighter. Median time from nudge to Hard, when it happened: ~300 frames (5 s).

No 1-ULP nudge became Hard in 900 frames.

## What it means
A one-off 1-ULP engine error is usually harmless online: it either never reaches the checksum or heals, and at
worst shows Slippi's "desync risk" warning. Hard desyncs come from larger or repeated errors, and from percent
differences, which persist and change knockback. So the engine bugs that matter most for online play are the ones
that recur every frame or touch damage/knockback, not single-rounding differences in position.

## Limits
- 900-frame window: later healing or later Hard is not seen.
- 20 replays, one port nudged per replay; one nudge per run (real bugs can fire repeatedly).
- The checksum only contains position, percent, action state and spawn number.
