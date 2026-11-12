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

## Run 1 (2026-11-11 21:27, commit fd1b40750 + spans, gate with MELEE_RECLAIM_LEAK=1) — build/zerohitch/base
- vanilla_bf: 727 over budget. 722 of them: from tick 1815 of the first match EVERY tick costs 30-40 ms (catch-up,
  300+ ms frames) to the end (tick 9000; the match never reached results). Ticks before were 1-2 ms; arena allocs per
  tick unchanged (~1400-1800), no files/parses at the onset. Not the retired-list walk (LEAK keeps the lists short).
  Under investigation (A/B LEAK on/off with MELEE_SAMPLE). Other offenders: match load 50 ms (file+parse 16, fighter
  create 16, unsplit 17), fighter files load at scene change 51 ms (parse 40), menu 21 ms, CSS 22 ms + two 40 ms
  draw-only frames, first frame 84 ms.
- vanilla_fod: 11 over budget: 4 from a window resize/focus loss (88 ms unaccounted: someone else's window event,
  ~tick 559/673), 5 short engine-step frames right after them, 2 pacing; first frame 72 ms (parse 34, fighter 18).
- envoy: stopped early by the coordinator (only ~18 s, boot/title/menu): menu GPU pipelines 75 ms (163 ms frame),
  title parse 26 ms (50 ms frame). The gate's mods dir lacks the envoy_ui PNGs (warnings only).
- Lock rules from the coordinator: at most 10 minutes per hold; one scenario per hold; 4-min gaps (20 after an
  overrun); quiet passes at most hourly and announced. Gate now has GATE_TIMEOUT (540 s) and envoy 9000 ticks.

## In-match slowdown (found 2026-11-11 22:30): GX data cache thrash, NOT MELEE_RECLAIM_LEAK
- A/B (build/ab/noleak, build/ab/leak; vanilla 4-CPU BF, retail_full, MELEE_SAMPLE): both runs: ticks avg 1.2 ms to
  tick ~1815, then 20 ms (1800-2000) and 37-38 ms (2000-2700). LEAK makes no difference.
- Sampler (PDB beside the exe): 78% inclusive in display: HSD_JObjDispAll -> PObjDispSimplePrimitive ->
  `gxbridge::GXSetArray` -> `data_at_ex` (74%), plus 11% NtFreeVirtualMemory.
- Mechanism (melee_engine/src/gxbridge.rs): `GXSetArray` calls `data_at(base, usize::MAX)`, so `extent` returns the
  REST OF THE CONTAINING BLOCK (to the end of the archive raw image / host range) and each distinct array pointer gets
  its own copy of up to several MB, cached by (ptr, len). The cache is capped at 48 MB (`DATA_CAP`) and on overflow it
  is CLEARED ENTIRELY. Once the scene's arrays need more than 48 MB of such copies (here at tick ~1815, a new
  model's arrays come on screen), every frame clears and re-copies: MBs of memcpy + big Arc alloc/free per
  GXSetArray, inside the tick (display runs in the logic step), ~35 ms per tick.
- Grows with: the number of distinct vertex arrays on screen x their offset from the end of their file (fighters,
  stage, items, effects); not with match length as such. A 4-player match crosses it; Envoy (tag pair + up to 3 foes
  + stage shifts) very likely does too.
- Fix proposal (render-only, the sim does not read these copies; NHV1 unaffected): cache ONE copy per containing block
  (key = block base/len, invalidated by forget_range as now) and hand the renderer (block Arc, offset) instead of a
  per-pointer copy; plus LRU eviction instead of clear-all. Needs `melee_gx` set_array to take an offset. Check: are
  the copies allocated in the game arena (alloc_fresh shows in the profile)? if so they also sit inside snapshots;
  the fix should mark them system-heap.

## GX cache fix (2026-11-11/12): branch port/zerolag-gxfix2 (on port/v3x), also merged into port/zerolag
- Fix: one copy per archive image / host range for vertex arrays, renderer gets (block, offset) (LOAD_ARRAYBASE gains
  an offset word; GX state, ArraySnap, pack, interp, core v1 honour it); LRU eviction in both caches; forget_range
  drops overlapping block copies. Copies stay in the game arena (shim handles are arena state; see report).
- Proof: NHV1 + offscreen PNGs identical on pad_falcon_fox_fd / cpu_fox_falco_dl (frames 300/900/1500); hero replay
  Game_20251031T213013 frame-exact; melee_gx tests incl. fx_disarmed_is_the_plain_frame; snapguard save/load
  unchanged (0.25/0.40 ms). Vanilla 4-CPU BF match ticks p50 1.21 / p99 1.66 (before avg 37 ms after tick 1815).
  Envoy armed fight (tag pair + 2 CPUs, wave respawn, stage switch 10 -> 28), 7000 ticks: 0 frames over budget after
  the first; cache 9.6 MB, 0 evictions.
- Scripts (build/): ab_inner.sh (one run), envoy_fight.sh (armed Envoy fight via --match), vs4_run.sh (4 CPUs to
  results), verify_inner.sh (render-only proof), ticks.py (p50/p99/max per scene).
- camp_pad.txt no longer reaches Envoy on v3x (new native menu row): the gate's envoy scenario needs a new pad
  script or the armed --match form; dd5825a68 + camp_pad hangs at tick 1173 in HSD_PadRumbleInterpret1 (not in v3x).

## Named baseline (2026-11-12 ~01:40, port/zerolag 9ffcb603a = a07f7a03b + GX fix + spans; light-slot runs, not a quiet pass)
Scripts: build/vs_run.sh (retail_full, 5000 ticks), ab_inner --match fod, build/envoy_fight.sh (armed Envoy fight, tag
pair + 2 CPUs, wave respawn, stage switch 10 -> 28). Every offender is a load or a scene transition; nothing in fights.
- vanilla_bf: 3 + first frame.
  - first frame / title on_enter 76 ms (parse 45 of 72 files: PlKb*/EfKb*, GmTtAll; scene on_enter kind 0 = 57 ms).
  - CSS enter 20 ms (MnSlChr.usd 3.7 MB parse 14).
  - VS scene change (fighter files PlFc/PlFcNr/PlFcAJ 1.5 MB, EfFxData) 49 ms: parse 41, "scene on_enter kind 32" 40.
  - match setup 37 ms: stage load 11, fighter create 15 (kind 0), parse 13.
  - match ticks p50 1.18 / p99 1.68 (GX fix in).
- vanilla_fod (--match, first frame is the match setup): 67 ms (parse 33, items init 18, fighter create 16,
  scene preload 10); one 21 ms pacing frame at tick 6 (draw 6 + texture upload 2: first draws).
- envoy_fight: 0 over budget after the first frame; first frame 86 ms (parse 43, items init 21, fighter create 29
  incl. Falcon 19, scene preload 10). In-fight: stage switch 7.7 ms, wave-respawn fighter create 1.4 ms. p99 1.90.
- Treatments: title/CSS/menu archives and the fighter sets chosen at CSS -> parse ahead on the worker (decode plan)
  with the apply at the scene's first tick; commons (EfCoData, ItCo, LbRb, PlKb*/EfKb*) -> already-loaded reuse
  across scenes; "items init" 18-21 ms -> split (next step: spans inside it) then parse-ahead of ItCo; fighter create
  15-20 ms for the first fighter (common fighter data) -> parse-ahead + reuse; GPU first-draw costs -> prewarm.
  Envoy lead times: next fight's setup is known at the shop/pick stop (seconds ahead); stage shift and wave spawn
  are already under 8 ms but get N = 60 ticks of prefetch so they stay flat with heavier stages/characters.
- Not covered: camp_pad Envoy route (hangs at tick 1173 with Envoy mods loaded, any exe; rumblehang agent has it),
  borderless (owner), m-ex boot, 1P modes.
