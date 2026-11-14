# Run modes simulate the same game (headless = hidden window = shown window)

## The report (2026-11-13, port/envoy-combobreak 9c2b519bb)
The same scripted Envoy run (envoy/camp_pad.txt, P1 CPU, Petey boss, drives noshop, turbo) fired its first combo
break at live frame 563 headless and at about frame 6263 in a hidden window; each mode repeated its own result.

## Cause: a real divergence, from the wall clock
Per-tick logs (`MELEE_TICK_LOG`: engine frame, scene, the pads the sim got after the UI host, the logic seed) of both
modes were equal (same pads, same frames, same scenes) until tick 302, the title's first frame. There the logic seed
parted. `MELEE_RAND_TRACE`: headless drew 10 numbers, the window 12, from the same seed.
`gmTitle_801A165C` (the retail title) burns one `HSD_Rand()` per second of `lbTime_GetTimeInSeconds()`'s calendar
second, and the port's `OSGetTime` was the host's monotonic clock since start. A turbo headless run reaches the
title in a fraction of a second (0 burns); a window runs the boot at 60 Hz and gets there ~5 s in (4 burns).
Everything after (demo picks, menus, the run's seeds, the CPUs) followed the different stream. Each mode was
self-consistent because its own boot speed is.

Inputs and setup were otherwise identical: same pad script per PADRead, same turbo, same arming path (camp door,
no MELEE_ENVOY_ARM / START_FIGHT), same card copy.

## Fix
`melee_engine::seams::os_get_time` counts logic frames (`SIM_FRAMES`, advanced by every `Engine::tick`;
675000 ticks of 40.5 MHz per frame) instead of reading the host clock. Every sim reader of OSGetTime (lbTime,
lbsnap, lbcardgame, perf) now sees game time. The C build is untouched (frozen reference).

## Check
`tools/rust-port/run_modes_check.sh` runs the camp route headless and in a hidden window and compares the tick
and NHV1 logs line by line. After the fix, 9000 ticks: tick.log equal (9000 lines), hash.log equal (8233 match
frames); both modes fire the same combo breaks at the same frames.
