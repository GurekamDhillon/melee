# Zero hitch (branch port/zerolag, worktree C:\melee-wt\zerolag)

Owner's rule: no presented frame interval over 16.7 ms and no tick over its 16.7 ms slot from the game's own work, in any
scene. The OS scheduler, GPU driver, compositor, antivirus and other programs can still delay a frame; the profiler
(`melee_core::frameprof`, `MELEE_PROFILE=1`) says which it was. A late tick is a failure even if the presenter re-shows
an old frame.

## Gate
- `tools/rust-port/perf/zero_hitch_check.py PROFILE.csv` reads one profile (csv + the jsonl beside it) and exits 1 if
  any frame interval or tick_max is over 16.7 ms; offenders are grouped by their largest measured component (file
  read+parse, GPU pipelines, texture upload, big allocation, fighter create, pacing sleep, wasm mods, ui, draw,
  unaccounted, else "engine step (unsplit)") with the events inside the frame (scene/stage/envoy/file names). Also
  counts presents over 1.5x the display interval (the 144 Hz goal; reported, not failed). Only the very first frame
  (window creation) is exempt, and it is still listed as "startup".
- `tools/rust-port/perf/zero_hitch_gate.sh [vanilla_bf vanilla_fod envoy]` runs the scenarios real time, windowed,
  with audio, a copy of the card, `MELEE_PROFILE=1`, and checks each profile. It refuses to run if any
  melee/cargo/rustc/link process is up. It takes no lock: wrap it in buildlock + runlight. `INTERP=off`, `FPS=60`
  env give the variants.
  - vanilla_bf: `play/retail_full.txt` (title -> menus -> 4 CPUs on Battlefield -> results -> CSS -> 2nd match), 9000 ticks.
  - vanilla_fod: `--match fox/cpu9,falco/cpu9,FOUNTAIN OF DREAMS --quit-at-results` (no menus), 3000 ticks.
  - envoy: `envoy/camp_pad.txt`, P1CPU, Petey boss, drives `ember,bolt,tagauto,wave2,shift,dropall/ember,bolt`, 30000 ticks.
  - Not covered yet: borderless (owner to check), m-ex disc boot, 1-P modes.

## Baseline and work queue
See the commit log of this branch and the section below (filled by each run).
