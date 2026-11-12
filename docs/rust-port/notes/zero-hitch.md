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

## Asset service slice 1 (2026-11-12, branch port/zerolag-assets1)
- Step 1 (measured, FoD --match, da49ada82): "items init" 19.1 ms is all `match setup: items: ItCo load` (ItCo.usd 2.9 MB
  read+parse); `items: pools init` and `items: spawner setup` are ~0 ms. Nothing else to remove; ItCo goes through the
  asset service.
- Step 2 design as built (cd7c09828, 5f1bbfd6b): recording heap (`recheap.rs`, a reserved range at 0x5C7A_0000_0000,
  1 GB slots, bump, never reused, frees mark dead) + `asset.rs`: `prepare` runs `archive::decode` on a copy of the file
  inside a slot (any thread), keeps the blocks reachable from the `Decoded` result in allocation order plus fixups
  (block->block, block->raw image, stale in-slot words -> null); `apply` allocates the same blocks in the same order
  from the game arena, copies, patches. EVERY load >= 64 KB goes prepare(ahead or now)+apply, so service on/off is the
  same arena sequence by construction; < 64 KB keeps the in-place decode in both modes. Plans are cached by content
  (length + memcmp + FTDATA_LOADING_KIND compatibility): a reloaded file is an apply only. dat_custom's side lists are
  thread-local and its static writes (KINOKO_JOINTS, FOODS_JOINTS, MEX_MOTION_ROWS) are effects replayed at apply.
  Hints: every preload-cache registration (`lbDvd_80017740`) -> worker reads the disc entry with its own handle
  (no hints while a mods overlay is mounted). Refusal at hint time: target lbHeap's largest free block < file, or the
  service budget (42 plans / 1.5 GB) spent. Note: the service never allocates game heap; the game still reads the file
  into its heap at the load as before, so the refusal only stops wasted prefetch.
- Byte-identity selftest (`MELEE_ASSET_SELFTEST=1`: each load is also prepared on a fresh foreign thread and the
  plans compared by digest): first run 5/10 FoD files SAME, 5 DIFFERENT by 2-176 bytes, all the upper 4 bytes of
  8-byte slots after a u32 = uninitialised `Root::Raw(u32)` payloads. Fixed in 5f1bbfd6b (repr(u64), usize payload);
  NOT yet rebuilt/re-run (owner started playing; build stopped).
- Measured with the selftest on (times inflated by it): FoD apply ms: ItCo 2.9 MB 15.2 (57,708 blocks, 152k fixups),
  stage 1.1 MB 3.4, EfCoData 1.3 MB 0.8, fighters 0.2-0.7. ItCo's apply is itself near the budget: apply now
  allocates straight from the arena (uncommitted-at-measurement change, in cd7c09828+); needs re-measuring.
- Open: lead time. In --match the preload registrations happen in the match-setup frame itself (no lead); the
  retail_full route (CSS/SSS before the match) still has to be measured. Rematches/second matches hit the cache.

### Slice 1 results (2026-11-12 ~09:00, port/zerolag-assets1; light-slot runs, not a quiet pass)
Byte identity:
- Plan selftest (`MELEE_ASSET_SELFTEST=1`; every load is prepared again on another thread at the load and the digests are compared):
  FoD 10/10 SAME; menu flow 34/35 until the stage fix below. The causes of the differences were all
  uninitialised bytes (Root::Raw(u32) payload, DatSymbol padding, Effect variant tails) and one game-state read in the
  decode: dat_custom's yakumono params read `stage_info.grkind`. That value is now a plan input: a plan fits only the
  stage it was decoded for.
- Whole-run NHV1 (MELEE_HASH_LOG), service on vs off: menu flow (retail_full, 3847 match ticks) EQUAL; Envoy armed
  fight (7000 ticks) EQUAL. Before the grkind fix, the menu flow diverged at tick 746 and crashed in Corneria's
  material anim (rc 139): the selftest found it (stage plan block 144 vs 152 bytes).
- Gate goldens (4 scenarios), synctest depth 4 (0 mismatches), snapguard (depth-7 synctest, 0 mismatches): see the
  commit message of the run below.
Numbers (frame ms; vanilla, service off = every load prepared on the spot -> same arena as on):
| scene | pre-service (baseline) | service off | service on |
| VS scene change (fighter files) | 78.9 (s5 off) / 49 named baseline | 96.2 | 33.0 |
| match setup (menu flow) | 45.8-49 | 49.2 | 46.1 (stage plan not ready: grkind unknown at hint, prepared on the spot 18 ms) |
| CSS enter | 20-23.8 | 23.8 | 23.0 |
| FoD --match first frame (worst case, no lead) | 67-70 | 107.3 | 81.2 |
Apply cost (measured): ItCo 2.9 MB -> 42.9 MB of blocks, 57.7k blocks: 15 ms first, 7.5-8.6 ms after batching
(one arena lock), sparse copy into known-zero bump memory and fixups (0.5 ms). Fighter files 0.2-1 ms, stage 1.6 MB 3.5-4.4 ms.
Open:
- On the spot (no lead) is SLOWER than the old decode: ItCo prepare 33 ms + apply 8 ms vs the old 19 ms parse. The
  owner's rule says the worst case must not be slower than today. Falling back to the old decode when a plan is not
  ready would make the arena layout depend on worker timing (the plan apply and the old decode allocate differently),
  so the old decode is not an option for a single load. Ways out: make the old decode go away, i.e. make prepare
  on the spot as cheap as the old decode (the recording heap's first touch of fresh pages and the reachability scan
  over 43 MB are the measured extras), or shrink ItCo's decoded image (40 MB of it is the decoder's whole-data shared
  tables).
- Stage hints: worker plans need the coming stage's grkind (stkind -> grkind at the SSS hint).
- Title (first frame 68-77) and CSS (MnSlChr 3.7 MB) have no lead: the title loads at boot; CSS plans could be
  hinted at the main menu.
