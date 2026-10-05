Skill events

Offline LAB single-feature demo of the engine's technique telemetry: `on_skill(info)` for every technique, `on_<kind>`
for one, `gd.skill_history(entity)` for the recent ring, `gd.skill_thresholds()` for the numbers. P2 is a CPU in script
mode; `skill_do <macro>` runs a technique on it, and P1 shows the same events for your own play. Kinds: lcancel,
lcancel_miss, lcancel_hit, auto_cancel, wavedash, waveland, ledge_dash, air_dodge, perfect_shield, tech, tech_miss,
dash_dance, short_hop, full_hop, fast_fall, shield_drop, jump_cancel_grab, jump_cancel_usmash, sdi, combo, combo_end.
See docs/scripting.md, "Skill events".
