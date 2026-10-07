# Armour types

Offline LAB with P1 and P2. First active frame applies knockback armour to P1 and sets P2 to stand. A CPU P1 is also set to stand; a human P1 can remain neutral. Loading during a match works.

- A cycles knockback, damage threshold, knockback threshold, super, hit count, damage pool.
- R clears and reapplies the selected type for 600 logic frames.
- W/M/S inject weak (3 damage), medium (10), strong (25) collision hits through `gd.hit(1, {from=2, ...})`.
- SPACE toggles one scripted collision hit every 90 logic frames.

These are collision hits attributed to P2, not P2 attack animations. No direct percent writes are used. Percent accumulates, so computed knockback can change; start a fresh match for a fresh percent baseline. HUD reads typed armour and shows the latest `on_armor` event, including absorption, break, selected damage and computed knockback. Typed expiry/budget behavior comes from the native API.

Syntax/stub checks do not establish native armour reactions, events or rewind exactness. No build or game launch was performed.

`scripts/rewind-proof.lua` is a separate unexecuted offline LAB tester. Load it alone, keep human controls neutral, and use standing CPUs. It grants all six types for 30 frames, observes expiry over a 120-frame forward run, then requires the existing native rewind comparison to return `pass` and `diff_compared == 0` (simulation bytes only), with restored timer/type/value/budget readbacks. This source fixture is not a successful runtime proof.
