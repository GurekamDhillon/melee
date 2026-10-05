Turbo zero-to-death combo

A scripted combo in one match with the Turbo rule on. Fox (P1, a CPU in script mode, driven only through the CPU
virtual controller: `gd.cpu_mode(1, "script")` and `gd.cpu_pad`) takes an idle Falco (P2, `cpus=idle`: no DI, no
tech, no SDI, no escape) from 0% to a stock loss in one unbroken combo on Final Destination. The victim being idle
is what makes it deterministic; a real player could DI out of it. Frame timing and state were checked by readback
only: how it looks has not been seen by the agent that made it (the owner watched the first version).

Launch (LAB; the scene turns Turbo on):

    MELEE_SCENE="mode=lab;stage=fd;turbo=on;p1=fox/cpu0;p2=falco/cpu0/idle"
    MELEE_SCRIPT=<absolute path of this folder>

The mod places both fighters, waits 25 frames, then plays a table of one pad sample per logic frame
(`scripts/main.lua`). It starts about 2.5 s into the match. The R key (when it has finished) or the console command
`turbo_combo [waveshine|classic]` restarts the match and plays it again; `turbo_combo_here` replays in the same match
(positions reset, stale moves and damage not, so the numbers differ). Nothing in the game is edited: every input goes
through the CPU's virtual controller inside the simulation.

## Variants

`waveshine` (default): starts at the left edge (Fox x=-72, Falco x=-63) and carries Falco across the whole stage
to x=100 (past the right edge) in 10 hits, 0 to 44.5%, KO 217 logic frames after the start of the sequence:
shine, jab, shine, then six times (jump, wavedash forward, shine), then a down tilt that sends him off the right
edge. About 17 frames from one shine hit to the next (human waveshine: about 33). The finisher is an edge kill: the
down tilt hits him as he is off the stage, in hitstun, and an idle CPU cannot recover.

`classic`: in the middle (Fox x=-9, Falco x=0), 6 hits, 0 to 68.7%: shine, jab, up smash, forward smash, down tilt,
(jump, wavedash) up smash, which launches him off the top.

## What is Turbo and what is vanilla

Turbo cancels (the game logs `turbo: window open` and `turbo: cancel entity=1 from key=K (motion A) into motion B`
for each): classic shine to jab, jab to up smash, up smash to forward smash, forward smash to down tilt, down tilt to
jump; waveshine shine to jab, jab to shine, and shine to jump in each of the six loops, plus shine to down tilt at
the end. The window is 30 free frames after hitlag; the same move is not allowed twice (jab and shine alternate),
and smashes may not cancel into jump (in this build they cannot, so classic goes smash to smash).
Vanilla frame-perfect input, no Turbo: the wavedash (air dodge on the first airborne frame, stick about 20 degrees
below horizontal) and the shine out of it, and every attack that starts after Fox has landed from a wavedash.

## Result of the in-mod readback

`TURBO-COMBO hit N` lines and a final `TURBO-COMBO RESULT variant=... hits=.. victim=..% hitstun_unbroken=.. ko=..`
line: whether the victim was in hitstun on every frame between the first and last hit. Reproduced 6 of 6 times
(waveshine, real time) and 1 of 1 (classic, real time); on the turbo virtual clock 6 of 6 (classic), 3 of 3 (waveshine).

## How it was found, and its limits

An in-game depth-first search from savestates (`tools/combos/search.lua`; what it measured, including links that
broke or were refused, is in `tools/combos/tree/`). It only works against this victim, this start and this
build's Turbo rules (window 30 frames, smashes cannot cancel into jump, crouch or dash, no same-move cancel, walking
and turning never take the window). Changing the placement or the rules breaks the table.
