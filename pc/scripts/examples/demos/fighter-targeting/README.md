# Opponent queries and timed status

Offline LAB demo. Load this folder, then enter a match with P1 and an opponent. Initial setup runs on the first active frame, including loading during a match.

T: mark nearest opponent for 120 frames; radius 80 units. The HUD reads native state. Refusals are shown rather than claimed as success.

Syntax checked only; no build or game run has been performed.

`scripts/rewind-proof.lua` is a standalone offline LAB fixture (load without the demo or a gameplay director). It activates all capabilities together, observes 30-frame effects/statuses expire during a 120-frame forward run, then requires `rewind_test_result().pass` and `.diff == 0` plus restored timer/capability readbacks. Armour thresholds are checked after restore. Start with empty hands and enabled/loaded Home-Run Bat data for the deterministic item grant. This is an unexecuted fixture, not a runtime proof or gameplay acceptance result.
