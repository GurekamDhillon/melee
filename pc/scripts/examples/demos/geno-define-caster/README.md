# Vanilla Caster demo

One capability: a Geno `define` with its own article (a projectile), effect and named sounds (slice 3, `"geno": 8`). Enable `pc/geno/mods/vanilla-caster/` and `geno-lab`, then start the LAB with
`p1=geno:vanilla-caster/hu;p2=mario/cpu0` (`gd.cpu_mode(2,"stand")`). Key 1 presses the neutral special; S/L save and load, P/N/R pause, step, resume.

The log shows `geno: article sound: spawn of article 0 ...`, `article 0 spawned as item kind 4096` and the despawn reason; the hit reads `move_tag=projectile`.
Looks and sounds are unreviewed. Lesson: `docs/learn/geno-fighters/11-defined-fighter.md` section 5.
