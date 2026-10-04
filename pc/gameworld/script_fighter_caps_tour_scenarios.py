"""Integrator data for the shared demo_scenarios.SCENARIOS table.

Not imported by the tour yet: that file belongs to the concurrent catalogue job.
These assertions inspect engine getters; they do not replace human gameplay QA.
"""
SCENARIOS = {
    "demo_fighter_air_jumps": [
        ("assert", "gd.fighter_caps(1).air_jumps==3"),
        ("k", "J", .3), ("assert", "gd.fighter_caps(1).air_jumps==8"),
        ("k", "R", .3), ("assert", "gd.fighter_caps(1)==nil"),
    ],
    "demo_fighter_restrictions": [
        ("assert", "gd.fighter_caps(1).shield==true"),
        ("k", "F", .3), ("assert", "gd.fighter_caps(1).air_dodge==true"),
        ("k", "R", .3), ("assert", "gd.fighter_caps(1)==nil"),
    ],
    "demo_fighter_armour": [
        ("assert", "gd.fighter_armour(1).damage==12"),
        ("k", "A", .3), ("assert", "gd.fighter_armour(1).knockback==60"),
        ("k", "R", .3), ("assert", "gd.fighter_armour(1)==nil"),
    ],
    "demo_fighter_give_item": [
        # The tour must enable/load portable retail items for this case.
        ("probe", "Initial setup applied"),
        ("assert", "(function() for _,it in ipairs(gd.items()) do if it.owner_port==1 then return true end end return false end)()"),
    ],
    "demo_fighter_effects": [
        ("k", "E", .1),
        ("assert", "gd.fighter_effect(1,'invincible')~=nil"),
        ("k", "R", .1), ("assert", "gd.fighter_effect(1,'invincible')==nil"),
    ],
    "demo_fighter_targeting": [
        ("k", "T", .1),
        ("assert", "gd.nearest_opponent(1)~=nil"),
        ("assert", "gd.fighter_timed_status(gd.nearest_opponent(1),1)~=nil"),
    ],
}
