Shock status

Offline LAB single-feature demo of the native Shock status. `gd.shock(entity, {frames, charges, hitstun, bonus, stack})` shocks
a fighter: the next `charges` hits it takes that put it into a damage reaction get extra hitstun frames
(`round(base * (hitstun - 1)) + bonus`), then the status is spent; it also ends when its `frames` run out. Damage and
knockback are untouched. `on_shock`, `on_shock_hit` and `on_shock_end` (and `on_skill`) report each step. See docs/scripting.md,
"Shock".
