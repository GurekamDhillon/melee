# Vanilla Charger demo

One capability: a Geno `define` whose neutral special is a Lua callback with typed per-fighter state (slice 5, `"geno": 9`; reference `docs/geno.md` section 23).
Enable the fighter folder `pc/geno/mods/vanilla-charger/` and `geno-lab`, then start the LAB with `p1=geno:vanilla-charger/hu;p2=mario/cpu0`
(`gd.cpu_mode(2,"stand")`). Hold B on a controller, or press key 1 (hold B 45 frames) or 2 (15 frames); S/L save and load, P/N/R pause, step, resume.
The HUD reads the Lua state with `gd.fighter_lua(1)`; the fighter's Lua is not a `gd` script and cannot call `gd`.

`scripts/proof.lua` is the headless proof (no keys): it checks that the charge counts one per held frame, that the release enters the Release state with the
charge kept and the hitbox damage `6 + 0.15 x charge`, that a savestate load brings the charge back, and that `gd.rewind_test` across the charge, the 60-frame cap and
the release reports `diff_compared == 0`. Run it as a script mod of its own (a folder with a `mod.json` whose `entry` is `scripts/proof.lua`, beside `vanilla-charger` and
`geno-lab`) with `MELEE_SCENE="mode=lab;p1=geno:vanilla-charger/hu;p2=mario/cpu0;stage=fd"`; the log ends with `PROOF RESULT: PASS`.
`scripts/cycle.lua` repeats charge and release for 2400 frames to drive the bench SyncTest (`MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1 MELEE_SYNCTEST=12`).

Looks and feel are unreviewed (it wears Mario's model and clips). Lesson: `docs/learn/geno-fighters/13-fighter-lua.md`.
