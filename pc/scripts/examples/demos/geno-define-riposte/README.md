# Vanilla Riposte demo

One capability: a Geno `define` whose down special is a counter answered in Lua, with a follow-up (slice 5, second move, `"geno": 10`; reference `docs/geno.md`
section 23.7). Enable the fighter folder `pc/geno/mods/vanilla-riposte/` and `geno-lab`, then start the LAB with `p1=geno:vanilla-riposte/hu;p2=mario/cpu0`.
Key 1 holds down+B for the full stance, 2 lets go early (the stance cancels), 3 makes the CPU jab once, 4 puts the CPU back in front of you; S/L save and load, P/N/R pause, step,
resume. The HUD reads the Lua state with `gd.fighter_lua(1)`; the fighter's Lua cannot call `gd`.

`scripts/proof.lua` is the headless proof (no keys). It drives P2 as a script-mode CPU and checks: the stance enters and leaves on time; a jab inside the window is countered
(P1's percent does not move), the answer's hitbox damage is 1.5 x the countered hit clamped to 9..30 plus the streak bonus, a second counter raises the streak, a whiff and an early
cancel break it, A inside the answer chains into the follow-up (damage 0.6 x), and `gd.rewind_test` across a counter reports `diff_compared == 0` with the typed state restored. Run it
as a script mod of its own (a folder with a `mod.json` whose `entry` is `scripts/proof.lua`, beside `vanilla-riposte` and `geno-lab`) with
`MELEE_SCENE="mode=lab;p1=geno:vanilla-riposte/hu;p2=mario/cpu0;stage=fd"`; the log ends with `PROOF RESULT: PASS`.
`scripts/cycle.lua` repeats stance, counter, answer and follow-up for 2400 frames to drive the bench SyncTest.

Looks and feel are unreviewed (it wears Mario's model and clips). Lesson: `docs/learn/geno-fighters/13-fighter-lua.md`.
