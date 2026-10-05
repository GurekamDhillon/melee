CPU technique assist

Offline LAB demo of `gd.cpu_assist(port, {lcancel, perfect_shield, tech, tech_dir, wavedash, fast_fall, seed})`.
Launch Final Destination with two CPUs:

    MELEE_SCENE="mode=lab;stage=fd;p1=fox/cpu0;p2=fox/cpu9"

Port 2 is a level 9 retail-AI CPU that stays in fight mode (the AI is never reset). The engine adds technique to
the AI's own virtual pad, with each value the probability that a technique is performed when its opportunity arises:
an L-cancel press before a lagged aerial lands, a ground tech with the stick held for the tech window (the direction
from `tech_dir`: `in`, `away`, `toward`, `random`), a shield press when a foe hitbox is about to reach it, an air
dodge on the first frame of a ground jump (a wavedash), and a down flick on a descent. Port 1 is a script-mode CPU
attacker (`gd.cpu_goto`, `gd.cpu_macro`, and a knockdown every fifth cycle). The screen shows the engine's counters
(opportunities / performed) beside the count of the skill events the same fighter produced, so the assist is visible
as ordinary input. Console: `assist <0..1>` sets every probability. State is game memory: savestates and rewind
carry it. Offline only.
