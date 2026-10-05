# Vanilla Striker demo

One capability: a Geno `define` with its own move set (slice 2, `"geno": 7`): own attributes, own attack rows, eight specials on its own Geno states.
Enable the fighter folder `pc/geno/mods/vanilla-striker/` and `geno-lab`, then start the LAB with `p1=geno:vanilla-striker/hu;p2=mario/cpu0`
(`gd.cpu_mode(2,"stand")`). Keys 1-4 press the four specials, 5 a jab, 6 a forward smash, 7 a grab; S/L save and load, P/N/R pause, step, resume.

The log must show `interpreter attempts 0`; `python -m tools.geno.report pc/geno/mods/vanilla-striker` lists the eight specials bound to its states.
Looks and feel are unreviewed (it wears Mario's model and clips). Lesson: `docs/learn/geno-fighters/11-defined-fighter.md`.
