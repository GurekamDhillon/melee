Crits and on_crit

Offline LAB single-feature demo of the engine's crit decision. `gd.crit(entity, {chance, multiplier[, multiplier_max], launch,
min_percent, force, tags})` configures an attacker, a seeded generator decides each eligible hit, a crit adds
damage*(multiplier-1) through the percent-only path, and `on_crit` reports base and final damage, strength 0..1, the
contact point and the victim's hitlag frames. See docs/scripting.md, "Crits".
