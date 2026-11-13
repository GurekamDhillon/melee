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

## Asset service slice 2 (2026-11-12, port/zerolag-assets1; light-slot runs, single runs, not a quiet pass)
Design as built: the recording heap logs every alloc/free/realloc in order; a plan is applied by REPLAYING those calls
in the game arena (temporaries included). No plan ready -> the file decodes in place on the asset path (recycled
blocks and in-place realloc tails zeroed there only) and is queued so a later load replays. A plan whose live blocks
the result does not reach is dropped; its file length is then never prepared again (6d16e9315).

### Proof (measured)
- Arena digest as proof (`MELEE_ASSET_DIGEST=1` + `MELEE_ASSET_DIGEST_DUMP=<dir>`, compare with
  `tools/rust-port/perf/asset_digest_diff.py A B`; build/dg_fod.sh, build/dg_menu.sh). Each live block of a load is
  dumped as classified 8-byte words: own block (number, offset), file image offset, exe offset, arena offset, file
  image or arena minus block start, host pointer (Rust heap/stack: value not compared). Equal bytes always compare equal.
  The per-run values were host pointers and a byte-sliding classifier, plus reclaim freeing in HashMap random order
  (now always address order).
  - FoD --match: off vs off2 293/293 loads identical; on vs off 293/293 (7 replays).
  - Menu flow (retail_full): off vs off2 218/219, on vs off 218/219 (13 replays). The one word left is in a 42 KB file
    (under the 64 KB threshold, so it is always decoded directly, never replayed): a dat_custom particle bank slot whose
    (target - block) lands outside the image, the arena and committed memory. It is a per-run value, also off vs off2.
- Replay bugs found by the digest and fixed (626b1879e): (1) dat_custom particle texture banks store (file image
  address - block start); a replay copied the worker's value, so the game's bank pointed into nothing (texture data
  read from a wrong address; NHV1 cannot see it). These words are now registered while recording and rewritten at
  apply (`Tgt::RawRel`). (2) `sparse_on()`'s first env read allocated in the arena inside the first apply. (3) reclaim
  flags read at boot.
- Gate (goldens vs C references: pad_falcon_fox_fd, pad_marth_sheik_bf, cpu_fox_falco_dl, cpu_peach_samus_ys_items all
  frames; synctest depth 4 0 MISMATCHES; audio hash = C; snapguard depth-7 synctest clean, save 0.235 / load 0.385 /
  total 5.113 ms vs baseline 0.271 / 0.432 / 5.302): all ok on 8572ecac0. Hero replay Game_20251031T213013:
  SLPCHECK match (12844 fighter-frames). NHV1 service on = off: menu flow 3847 ticks, armed Envoy fight 7000 ticks
  (both on 8572ecac0 and again on 6d16e9315).

### Hints
Boot hints take effect for the title (GmTtAll hit), the menus (MnMaAll 2.1 MB apply 4.2 ms), CSS (MnSlChr 3.7 MB apply
3.9 ms), EfCoData (1.2 ms), and the fighter files hinted at CSS (0.3-1.5 ms). They do NOT take effect for ItCo (2.9 MB)
or the stage (1.6 MB): their plans are dropped because 1-3 live blocks are reachable only from dat_custom's
thread-local side lists, not from the decode result (see "Remaining"). The stage-kind hint reads the right grkind, but
the stage plan is dropped for the same reason. With --match (FoD, Envoy) there is no lead: the boot hints are still
queued at the match-setup frame.

### Table (frame ms, worst frame of that scene; one run each)
| scene | pre-service (slice 1) | service on, with lead | service off (= no lead) |
| title / first frame | 68-77 | 52.1 | 50.9 |
| CSS enter | 23.8 | 18.7 | <= 19.8 (below the listed offenders) |
| VS scene change | 78.9 (named baseline 49) | 43.0 | 46.2 |
| match setup (menu flow) | 45.8 | 34.1 | 34.3 |
| FoD --match first frame | ~70 | (no lead) 72.6 | 67.8-71.5 (slice-2 agent) |
| Envoy armed fight first frame | 86 | (no lead) 90.3 | 80.0 |
Before 6d16e9315 the VS scene change with the service on was 80.7 ms. The worker was re-preparing the dropped ItCo
plan (a ~30 ms decode) at the same time. That cause is inferred from the fix, not profiled.
Verdict: with no lead, every scene's worst frame is at or below pre-service, except possibly the Envoy first frame with
the service on (90.3 vs 86, one run; the off run is 80.0). That needs repeated quiet runs. No scene is under 16.7 ms
yet; CSS enter (18.7) is closest.

### Remaining offenders (named causes)
- ItCo 2.9 MB in-place decode 20-22 ms + stage 1.6 MB 11 ms: plans dropped (blocks held by dat_custom's thread-local
  side lists, e.g. 16/24-byte list headers). Fix: make those lists decode results (or effects) so the plan reaches them.
- Fighter create 14-35 ms for the first fighter (common fighter data, PlCo): task 4 of slice 2. Not started.
- Title on_enter "engine step (unsplit)" ~25 ms (not a file cost): needs spans.
- --match has no lead by construction (Envoy/FoD first frame 80-90 ms): needs the next fight's files hinted at the
  shop/pick stop.
- Not merge-ready: the zero-lag requirement is not met in any scene; the byte-identity proofs above all pass.

## Asset service slice 3 (2026-11-12, port/zerolag-assets1; light-slot runs, not a quiet pass)
Changes (3bffb7b8d .. 39dd4e9bf):
- ItCo and stage plans replay. The "live blocks the result does not reach" were leaks of the decode itself (an
  article's dynamics / hurt-bone list or special attributes written twice: the first block loses its only pointer).
  When dat_custom's side lists are all empty again after the decode nothing outside holds them, and the replay
  reproduces them (every call is replayed, every live block written), so the plan is kept.
- `prepare` says why it gave no plan (NoSlot / NotRouted / Unsafe); only Unsafe/NotRouted mark a length unplannable.
  A no-slot failure is retried at the next hint or load (seen on the menu flow: 9 such retries).
- Envoy: `gd_envoy_plan` calls `asset::hint_envoy_plan` (stage file for the plan's grkind, data / costume / animation
  bank of main, partner, co-op pair, CPUs and next-wave characters; pure name-table reads, once per distinct plan).
  With `--match` the plan is published in the fight-start frame: no lead (FoD and Envoy `--match` stay no-lead).
- Arena: a recycled block of a trimmed class (trim mode, >= 64 KB) is not re-zeroed (its pages were decommitted at the
  free and read back zero); the replay skips writing a dropped temporary's decommitted pages; plans with >= 8 MB of
  live blocks write them on 4 threads (allocation order unchanged, same bytes).
- ARAM (seams.rs `ar_addr`) grows to what a transfer touches instead of zero-filling 32 MB at the first ARAM read:
  that read was the first fighter create's first figatree (`ft create: inline2` 13-14 ms -> < 1 ms).
- Spans: `title enter: *`, `title demo: *`, `ft create: *`, `figatree: disc read / parse`.
- Debug: `MELEE_ASSET_POISON=1` (freed recorded blocks poisoned, 16-byte gaps checked: writes after free / past a
  block's end in a recorded decode). Found one: an 8-byte write one past a 136-byte block in ItCo (in the arena it
  lands in the 144-byte class slack, harmless; not yet traced).

Named causes found:
- Title first frame ~52 ms: `title demo: preload scene (lbDvd_80018254)` 32 ms. The port completes DVD reads at once,
  so the preload queue's completion callback (lbDvd_80017E64 -> 80017CC4 -> next file) reads the whole demo match's
  file set in that one frame. Fix options (not done): prefetch those bytes on the worker at boot (the demo pick is game
  state), or make preload reads complete over frames (sim-visible: needs its own proof).
- VS scene change ~25 ms: ItCo apply 9.8 ms (58k blocks, 351k fixups; replaying temporaries 3.7 ms) + fighter files.
  ItCo is decoded again for every match; reusing the decoded commons across matches is the next lever.
- CSS enter: frame 14.5-18.5 with tick 12-14 ms: the long intervals are pacing sleep after a 13 ms tick.
- FoD / Envoy `--match` first frame: no lead by construction (ItCo in place 21 ms, fighter files, scene preload 11 ms).

### Table (frame interval / tick ms, worst of the first 4 frames of the scene, WORST of 3 runs; build 39dd4e9bf, no digest)
| scene | pre-service (slice 1) | slice 2 (on) | slice 3 service on | slice 3 service off |
| title / first frame | 68-77 | 52.1 | 55.1 / 43.1 | 50.9 / 37.2 |
| CSS enter | 23.8 | 18.7 | 17.3 / 13.8 | 18.5 / 12.6 |
| VS scene change | 78.9 | 43.0 | 25.6 / 24.4 | 38.6 / 37.1 |
| match setup (menu flow) | 45.8 | 34.1 | 14.9 / 12.8 | 16.6 / 15.3 |
| FoD `--match` first frame (no lead) | ~70 | 72.6 | 56.0 / 54.8 | 54.8 / 52.5 |
| Envoy armed `--match` first frame (no lead) | 86 | 90.3 | 110.5 / 106.6 (93-110) | 94.1 / 90.7 (87-94) |
Under 16.7 now: match setup (menu flow, service on: all 3 runs; off: tick 15.3 but one interval 16.6); CSS enter by
tick (12-14 ms) but not by presented interval (pacing sleep after the tick, 14.5-18.5). Not under: title (title-demo
preload chain, 32 ms), VS scene change (ItCo apply ~10 ms + fighter files), and the `--match` entries (no lead; in
Envoy the service is WORSE than off: the worker prepares the same files at the same time as the in-place decode,
contention; a no-lead load should not queue a worker prepare while the game decodes the same file).

### Proof (39dd4e9bf)
- Digest FoD `--match`: on vs off 293/293 identical, off vs off2 293/293 (ItCo and the stage now replay here).
- Digest menu flow: off vs off2 218/219 (the known 42 KB word); on vs off 217/219: the same 42 KB word plus ONE word in
  ItCo (load 65, live block 97, 130 bytes = a decoder issue string "non-zero word ..."): in the in-place decode of the
  off runs its first 4 bytes are 0x00000113 (deterministic, off = off2), in the replay they are the text. A probe
  showed the bytes are already written when `archive::decode` returns; MELEE_ASSET_POISON shows no write after free
  and no write past a block end in the recorded decode except the 136-byte one above. So something in the in-place
  decode on the game thread writes a u32 into memory the arena has handed to that string; not found yet. It was
  invisible before (ItCo never replayed in the menu flow). The text is never read by the game; the stray write is the
  concern.
- NHV1 on = off: menu flow 3847 ticks EQUAL, armed Envoy fight 7000 ticks EQUAL.
- Gate: 4 goldens vs C all frames, synctest d4 0 mismatches, audio hash = C, snapguard (d7) save 0.206 / load 0.348 /
  total 4.72 ms vs baseline 0.271 / 0.432 / 5.302: GATE OK. Hero replay Game_20251031T213013: SLPCHECK match (12844).

### Verdict
Not merge-ready: title (~52), VS scene change (~25) and the `--match` fight starts (55-110) still miss 16.7 ms, and
the menu-flow digest has the one unexplained ItCo word. The branch does clearly help the retail path (VS scene change
38.6 -> 25.6, match setup 16.6 -> 14.9 vs service off; vs slice 1: 78.9 -> 25.6 and 45.8 -> 14.9).
Next: trace the 0x113 write (hardware watch on that string); reuse decoded commons (ItCo, EfCoData) across matches;
prefetch the title-demo files at boot; no worker prepare of a file the game is decoding in place right now.

## Asset service slice 4 (2026-11-12, port/zerolag-assets1; light-slot runs, not a quiet pass)
Changes (04f4a5529 .. e915a717f):
- ROOT CAUSE of the ItCo 0x113 word and the 8-byte write past a 136-byte block, one bug: dat_custom `rtable_articles`,
  the Warp Star attribute rebuild (since 8d104891c, Nov 2) allocated the header + `count - 1` entries. The last 16-byte
  `itWstarAttrEntry` was written past the block, and its `x4_sfx` (0x113) landed in the first word of the next block in
  the arena's size class (in the menu flow, a decoder issue string). Found with `MELEE_HWWATCH_ISSUE=1` (new: hardware
  write watch on issue strings, `MELEE_HWWATCH_NTH` arms one). Fixed: 84e682052. Also on its own branch for the owner's
  build: `port/itco-overrun` (e126a38a8 + fix + regression test `wstar_block_holds_every_entry`, gate OK).
  It is very likely the "u32 0x113 stored at match 1's load" of 9cdb42af8 (the MELEE_RECLAIM_LEAK mitigation): same value,
  same load. Not proven: 10 + 3 armed-Envoy fight-start runs with MELEE_RECLAIM_LEAK unset gave 0 crashes / 0 poison
  reports BEFORE the fix as well. Keep the mitigation.
- A queued (not started) hint of a file the game decodes in place is dropped; kept-copy (`Src::Bytes`) prepares wait
  until the game's decodes were quiet for 150 ms; disc hints go first.
- Plans with >= 100k fixups (ItCo) write live blocks on 4 threads (ItCo apply 11.2 -> 8.8 ms).
- Boot hints: MnExtAll (CSS: 1.3 MB decoded in place, 2.9 ms) and Kirby's copy-hat files (title demo with Kirby: ~22 ms of
  in-place decodes in the first frame).
- Pacing (main_loop.rs, presentation only): a tick due in the last half of a present slot is preceded by the present, and
  a tick due within 1.5 ms runs at once instead of sleeping to it (the clock counts it at its due time). A long tick no
  longer adds the slot phase to the presented interval (CSS 18.0/11.8 -> 12.6/11.8).
Not done: reusing applied commons across matches (would change allocation order); title disc reads / random demo files
(the remaining title cost is reading the randomly picked demo match's ~77 files in the first frame).

### Table (interval / tick ms, worst of first 4 frames, WORST of 3 runs; build e915a717f)
| scene | slice 3 on | slice 3 off | slice 4 on | slice 4 off |
| title / first frame | 55.1 / 43.1 | 50.9 / 37.2 | 62.0 / 46.9 | 53.9 / 39.4 |
| CSS enter | 17.3 / 13.8 | 18.5 / 12.6 | 15.6 / 14.7 | 13.2 / 12.4 |
| VS scene change | 25.6 / 24.4 | 38.6 / 37.1 | 29.0 / 27.9 (2 of 3: 23.7-24.3) | 39.1 / 37.7 |
| match setup | 14.9 / 12.8 | 16.6 / 15.3 | 17.0 / 15.6 (2 of 3: 11.6-12.5) | 17.3 / 15.4 |
| FoD `--match` (no lead) | 56.0 | 54.8 | 56.6 / 55.5 | 56.7 / 50.4 |
| Envoy `--match` (no lead) | 110.5 | 94.1 | 105.5 / 101.8 | 96.7 / 93.1 |
Under 16.7: CSS enter (on and off). Match setup is under in 2 of 3 runs only. Not under: title, VS scene change and the
no-lead fight starts. Envoy no-lead is still slower with the service on (100-105 vs 88-97). The queued-hint drop did not
fix that.

### Proof (e915a717f)
Digest FoD on = off 293/293, off = off2 293/293. Menu flow on vs off: only the known per-run 42 KB word (load 35)
differs, the same as off vs off2 (the ItCo word is gone). NHV1 on = off: menu flow 3847 EQUAL, Envoy fight 7000 EQUAL.
Gate: 4 goldens, synctest d4 0 mismatches, audio = C, snapguard 0.208 / 0.35 / 4.73 under baseline: GATE OK.
Synctest d7: 1638 hash checks, 0 mismatches. Hero replay Game_20251031T213013: SLPCHECK match (12844).
Verdict: correctness proofs all pass; zero-lag NOT met (title, VS scene change, no-lead fight starts).

## Asset service slice 5 (2026-11-12, port/zerolag-assets1): the COST of decode and apply
Profiled with new env-gated diagnostics (`MELEE_DECODE_PROF=1`, archive.rs; game-thread decodes only for the loop /
alloc timers; it drains the fill queue per root, so never use it for digests): per decode the phases (parse: data copy,
relocs, symbols, reloc set; bounds; decoder setup; roots; finish/drop; article fixes), the time and count of zeroed
allocations, the element loops of big arrays per type, top types by bytes, and the process page-fault delta. The apply
log line (`MELEE_ASSET_LOG`) now also prints page faults.

### Breakdown (menu flow, service off = decode in place, before the fixes)
| file | total | where |
| ItCo 2903 KB | 20.6-22.6 ms | parse 1.6 (copy 1.0, reloc set 0.4), bounds 0.9, roots (itPublicData) 17.4; 58k blocks / 36 MB (VtxDescList 22 MB, TlutDesc 6 MB); allocations 2.2 ms; TlutDesc shared loop 9.1 ms (276k elems), VtxDescList shared 3.7 ms (178k); 6.6k page faults |
| stage (Corneria) 1610 KB | 6.9-8.2 | map_head 5.9; TlutDesc 2.7, VtxDescList 1.4, AnimJoint 0.7; 2.9k faults |
| Kirby copy hat 103 KB | 6.1-6.7 | 15 MB of blocks from a 103 KB file: `Vec` arrays 1.0M elements 6.0 ms; 3.1k faults |
| ftDataMario 190 KB | 5.6-6.0 | `Vec` arrays 824k elements 4.7 ms; 2.8k faults |
| MnSlChr 3722 KB | 2.4 | data copy 1.2, roots 0.7 |
| menu panels 2104 / 1284 KB | 2.8 / 1.8 | VtxDescList shared, copy |
Apply (service on): ItCo 8.5 ms with 9.0k page faults (events 5.1 of which temporaries 3.4, live writes 3.4).
Not the cost: allocation calls (2.2 ms of ItCo), hash maps / memo, endian conversion, reachability, article fixes
(0.06), parse (<= 1.6). The cost is (1) the element loops of the overlapping shared-array windows: each window rescanned
every "done" flag of elements other windows had decoded, and (2) first-touch page faults: in `melee play` the arena trims
(decommits) big freed blocks, so every reuse refaults (ItCo with MELEE_ARENA_NO_TRIM: 16.8 vs 20.8 ms).

### Changes (13596b880)
- gcdecode `shared_array_slack`: a run of already-decoded elements is skipped 8 flags at a time. Same elements, same
  order, same writes. TlutDesc loop 9.1 -> 3.9 ms, VtxDescList 3.7 -> 2.3; ItCo in place 22.6 -> 17.3 ms; Kirby hat 6.3 -> 3.7.
- arena pre-faulter (`arena::start_prefault`, started by asset init; `MELEE_ARENA_PREFAULT=off`): a below-normal thread
  keeps 96 MB past the bump pointer committed (under the arena lock, as alloc would) and resident by READING one byte per
  page. No writes, so bytes / blocks / write-watch state are unchanged. Gain is small (ItCo apply 8.5 -> 6.4 in one
  run, decode ~2 ms): most faults are on trimmed (decommitted) recycled blocks, which it cannot pre-touch.
- 44242f989: asset init reads `MELEE_DECODE_PROF` once, outside the asset path (the first read allocated inside the
  first decode: 2 extra recorded calls in load 0 of an off run; found by the digest proof).

### Table (interval / tick ms, worst of first 4 frames, WORST of 3 runs; build 44242f989; light-slot runs, heavy lock
FREE before and after the batch, not otherwise verified quiet)
| scene | slice 4 on | slice 4 off | slice 5 on | slice 5 off |
| title / first frame | 62.0 / 46.9 | 53.9 / 39.4 | 59.2 / 44.6 | 41.1 / 29.4 |
| CSS enter | 15.6 / 14.7 | 13.2 / 12.4 | 12.1 / 11.4 | 12.2 / 11.5 |
| VS scene change | 29.0 / 27.9 | 39.1 / 37.7 | 25.5 / 24.2 | 33.5 / 32.1 |
| match setup | 17.0 / 15.6 | 17.3 / 15.4 | 13.3 / 11.9 | 15.7 / 14.3 |
| FoD `--match` (no lead) | 56.6 / 55.5 | 56.7 / 50.4 | 51.5 / 44.8 | 43.8 / 37.7 |
| Envoy `--match` (no lead) | 105.5 / 101.8 | 96.7 / 93.1 | 69.7 / 65.1 | 65.0 / 63.8 |
Under 16.7: CSS enter and match setup (on and off, all 3 runs). Not under:
- title: on 57-59, off 40-41. The service makes the title WORSE by ~17 ms: the boot hints are prepared on the worker
  during the title's first frame (contention with the game thread's demo preload). Remaining off cost: ~10 ms of in-place
  decodes of the demo files + ~17 ms of the demo preload outside the decodes (reads + preload bookkeeping, not split).
- VS scene change: ItCo apply 6-8 ms (most of it page faults on trimmed, recycled arena blocks) + the fighter files.
- no-lead fight starts: FoD 44-52, Envoy 62-70: ItCo decode in place (~17 ms) + stage + fighters in one frame. The
  service is still slower on than off (FoD +8, Envoy +5): not profiled further (the worker's boot/kept-copy prepares).

### Proof (44242f989; gate / d7 / hero replay / NHV1 Envoy on 13596b880, which differs only in when an env flag is read)
Digest FoD on = off 293/293, off = off2 293/293. Menu flow on vs off: only the known per-run word (load 35), the same as
off vs off2. Digest off vs the profiling build before the decode change (allocation sequence of 8686ed93b): 293 loads,
the only differing words are exe-image offsets (class 3, the code moved) and the 2-call artifact of that build's load 0.
NHV1 on = off: menu flow 3847 EQUAL, FoD 1500 EQUAL, Envoy fight 7000 EQUAL. Gate: 4 goldens, synctest d4 0 mismatches,
audio = C, snapguard 0.234 / 0.385 / 5.1 under baseline: GATE OK. Synctest d7: 1638 checks, 0 mismatches. Hero replay
Game_20251031T213013: SLPCHECK match (12844).

### Verdict and next approach
Correctness proofs pass; the changes help every path (off mode too). Zero-lag NOT met: title, VS scene change, no-lead
fight starts. Merge: the decode change and the pre-faulter are safe and help the owner's build (they also speed up the
service-off path); the asset service itself still makes the title worse and the no-lead starts worse, so merging with
the service ON by default is not justified yet.
Honest view: cheaper decode/apply alone will not close the gap. ItCo alone is ~17 ms in place and 6-8 ms applied, and
a scene change loads several files in one frame. What would close it:
1. Stop the arena trimming big blocks that are reused every match (the decommit/refault per reuse is most of the apply
   time): keep a per-size-class pool of trimmed blocks resident, zeroed off the game thread. Needs care: alloc_zeroed's
   "recycled trimmed block reads zero" rule and the memory ceiling the trim exists for.
2. Keep ItCo's decoded result across matches (identical bytes every match) instead of re-decoding/replaying it: an
   allocation-order change, so it needs the goldens/synctests/digest-off-before proof route.
3. Title: do not prepare boot hints while the game thread is inside the title's first frame (gate the worker on
   GAME_DECODE_AT quiet time, as kept-copy prepares already are), and split the ~17 ms of demo preload outside decodes.
4. Remove the garbage decoding at its source (decoding 1M `Vec` records from a 103 KB Kirby hat file, 711k vertex-list
   records per VS load): output-visible today, so this is a behaviour change to prove unobservable.

## Asset service slice 6 (2026-11-12, port/zerolag-assets1 83086f7f7; merge candidate port/zerolag-merge1)
Changes:
- The asset worker waits for game I/O quiet: `host_dev_com_request` stamps `asset::game_io()`. A disc hint starts only
  after 40 ms without a game read or decode, and a kept-copy prepare only after 150 ms. The clock is stamped at service
  start, so boot hints never run inside the title's first frame or a no-lead fight start.
- Trim mode keeps big freed blocks resident (`arena::keep`, `MELEE_ARENA_KEEP=off|<MB>`, default cap 256 MB, table of
  512). Before, these blocks were decommitted. Now exactly the bytes a decommit would drop (whole pages inside
  `[p+16, p+size)`) are zeroed: by the pre-faulter thread in 64 KB steps under the arena lock, or by the allocating
  thread for what that thread has not reached yet. A kept block needs no recommit.
  - ItCo apply: 6-8 -> 4.4 ms (page faults 9.0k -> 2.1k).
  - ItCo in place: ~17 -> 9-13 ms.
- Title: the ~17 ms outside the decodes is about 10 ms of reads of the demo match's ~70 files plus about 7 ms of
  in-place decodes, all in one frame. Not split into named spans further. Not fixed (the demo pick is random; it would
  need read-ahead of the candidates).
- Task 4, keeping ItCo's decode result across matches: NOT done. It changes the allocation order, so the digest proof
  does not apply. It needs a golden/synctest/NHV1 proof route, and the cross-match `dat_custom` side effects and reclaim
  order must be shown unobservable first.
- Task 5, the garbage Vec records: NOT done. Source: `HSD_Spline.cv` (arrays.tsv `numcv.max(1)*3+4`). Kirby hat and
  fighter files hold joints flagged `JOBJ_SPLINE`, and the spline record's `numcv` reads ~27-31k. Each such spline
  decodes 80-95k Vec records past the data (zeros). Found with a temporary backtrace in `GcDecoder::array`, since
  removed. Trimming it changes block sizes, so it needs NHV1/golden proof that the game never reads those cv entries.

### Table (presented interval / tick ms, worst of first 4 frames; 3 runs, worst listed; light-slot runs)
| scene | s5 on | s5 off | s6 on | s6 off |
| title | 59.2/44.6 | 41.1/29.4 | 58.0/46.0 (others 41.4, 38.3) | 40.1/27.7 |
| CSS enter | 12.1 | 12.2 | 11.6/10.5 | 9.3/8.3 |
| VS scene change | 25.5/24.2 | 33.5/32.1 | 22.6/21.3 (others 17.5, 17.8) | 24.1/23.0 |
| match setup | 13.3 | 15.7 | 12.1/11.2 | 11.0/9.8 |
| FoD no lead | 51.5 | 43.8 | 47.7/41.6 | 42.7/38.4 |
| Envoy no lead | 69.7 | 65.0 | 60.0/59.1 | 57.7/53.9 (run 3 produced no profile) |
Merge (port/zerolag-merge1, worker off) vs base 677359edb, worst of 3:
| scene | base | merge |
| title | 56.0/44.0 | 40.9/26.9 |
| CSS enter | 19.1/13.5 | 9.5/8.6 |
| VS scene change | 44.3/42.5 | 24.9/23.5 |
| match setup | 35.8/33.9 | 11.5/10.7 |
| FoD no lead | 67.8/66.5 | 43.5/38.9 |
| Envoy no lead | 88.9/87.8 | 57.0/55.8 |
Under 16.7: CSS enter and match setup (both branches, all modes). Not under: title (~36-41; one 58 outlier with the
service on), VS scene change (on: 17.5-22.6; off: ~24), and the no-lead fight starts (FoD 38-48, Envoy 54-60).

### Proof
Asset branch:
- Digests: FoD on = off and off = off2 (293/293). Menu flow: only the known per-run word at load 35.
- `MELEE_ARENA_KEEP=off` vs on: FoD 293/293 equal; menu flow only that same known word; menu NHV1 equal.
- NHV1 on = off: menu flow 3847 EQUAL, Envoy 7000 EQUAL.
- Gate OK (snapguard 0.23 / 0.373 / 4.996). Synctest d7 1638 checks, 0 mismatches. Hero replay SLPCHECK match (12844).
Merge branch:
- Gate OK (goldens, synctest d4 0 mismatches, audio = C, snapguard 0.255 / 0.413 / 5.639).
- Synctest d7 0 mismatches. Hero replay SLPCHECK match.
- The integrator still has to run the three replay sets on port/zerolag-merge1 before packaging.
