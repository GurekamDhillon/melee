# Demo catalogue

- [Warm](warm/) (`demo_warm`): seven enemy descriptor declarations, readiness
  progress, then one visible Goomba. Source/Lua stub checked; native acceptance pending.

- [Zones](zones/) (`demo_zones`): two outlined rooms and a doorway, cached native
  membership and enter/exit/none/some logs. Lua stub checked; native acceptance pending.

One feature at a time, followed by complete small tools/games. New mods use only text,
kit names and primitives; optional model/material examples accept your own exports.
**This packet never built or launched the game.** New demos are source/syntax/stub checked,
not accepted in game. Existing verification belongs to `docs/HANDOFF-2026-10-03-ENGINE-DAY.md`.

**Fix2 supersedes the original tour acceptance description.** The first external audit ran all
39 original mods on vanilla and reported 36 working; see `_build/audit-20261003/demo-tour/`.
The repairs and stronger visual defaults here have not been rerun in game. A tour PASS now also
requires audit-derived key/state scenarios, owner-local state assertions and fully decoded PNGs,
CPU proof and launcher OK. It is functional evidence for the exercised steps; the Gauntlet tour
checks staging, not a full victory, and all visuals still need human acceptance. No acceptance is
inferred from a screenshot arriving. Scenario evidence is `<id>-scenario.json` with A/B shots.
Only the current demo's packages are mounted under `entries/<id>/mods/`; copied boot scripts are
omitted, and the original mod entry loads once. Unattended tours are muted (`MELEE_VOLUME=0`).

Runtime behaviours exposed by that audit:

- `gd.stage_hide(false)` refuses while any stage slot is loaded. Retire the arena and restore
  the host before loading a destination slot; owner unload later releases live slots.
- A camera claim made in a coroutine task ends with the task. Claim persistent cameras from
  `on_frame` after staged construction completes.
- `gd.press` requires a task (`gd.run`); it cannot be called directly from an ordinary hook.
- Reading `gd.camera_params()` errors while another script owns the parameters. Inspect through
  that owner's command; the tour avoids foreign reads.
- The audit observed `gd.fly_attack` accepting damage 40 despite the documented 1â€“30 range.
  That is a documentation/validation discrepancy, not a promised contract; this demo stays in range.
- `gd.scene_launch` stalled logic for about ten seconds in the audit. The tour waits for a fresh
  active scene with a bounded timeout; it does not assume a short fixed sleep is sufficient.

Stage tour temporarily changes only its surface shader after the `after` event, on the next frame.
It never clears post passes or removes a live transition cover. Restoring owned post-handle updates
awaits the separate engine hang repair and native retest. Six-slots reports FAIL if recycled P6 is
still invisible; a successful native recycle return alone is not visibility proof.

Start an offline vanilla FD LAB: `mode=lab;stage=fd;p1=fox/hu;p2=marth/cpu0`.
Use console `load <absolute-folder>` then `unload <manifest-id>`; replace the folder with the
linked folder below. Demos catch up on their first completed frame. Start a fresh match between
demos: cleanup releases owned resources, but damage, motion/KO and real combat are not reversible.
The tour always starts a fresh scene. Shared keys only act while the game has focus and the console
is closed. Do not load multiple gameplay examples together.

Each demo that needs an idle opponent must set its own CPUs to `gd.cpu_mode(port,"stand")`;
`cpu0` is a difficulty, not an idle mode. Reassert after match start, respawns and stage switches.
The tour adds a continuous guard and records proof per demo. `acting_cpu: true` in the catalogue
explicitly permits demos that require a fighting CPU (fighter-mode controls and Gauntlet).

The tour uses one owned, bounded launcher run per demo: `--demo-max-seconds 90` and a whole-tour
`--max-seconds 600` by default, with unattended watchdog exit and a unique owner tag. Each result
records the launcher's verdict; missing/non-OK verdicts fail even if a screenshot exists.
Window dimensions honour `MELEE_WINDOW_W/H` (default 1024x576; width above 1080 is rejected),
with fixed second-monitor position (-1080,-360) and a visible window.

All tour writes stay under its new `--output` folder: `runtime/` holds read-only baseline EXE/map
copies, and `runtime/runs/<id>/` holds launcher EXE/DLL/cache copies, memory cards, native logs,
heartbeat/run/verdict JSON and possible crash/hang diagnostics. Top-level files contain the plan,
results, summary, guard scripts, per-demo logs and contact sheet; `mods/` contains isolated mounts
and `data/` contains saves/screenshots. The baseline build root is read only. The runner never builds.
Disc paths are redacted as `<disc>` in public output. To keep the configured path out of native
logs/metadata too, the runner uses a temporary `disc.iso` hard-link alias (no bytes copied), removed
on exit. Choose an output on the same hard-link-capable volume as the disc; failure stops before launch.

For effects and Gauntlet, put the folder directly under a mods parent, enable its manifest id in
`enabled.txt`, set `MELEE_MODS_DIR` to that parent and boot. Definitions/packages are mounted at
boot. Other new demos can be loaded by absolute folder. Mission model reads work on Windows.
Do not move existing examples. `catalogue.json` supplies ids, paths, setup and tour admission;
reference-only rows are lessons, not launchable mods. Bench requires the benchmark runner's
generated config; Envoy has its own acceptance guide. Both are indexed but skipped by default tour.

Run `python tools/port/test_demo_mods.py` and `python tools/port/test_demo_smoke.py` from the workspace.
See `tools/port/demo_tour.py --help` for the integrator's visible second-monitor tour.

| ID / folder | Shows | API lesson | Run / controls | In game |
|---|---|---|---|---|
| [`demo_input`](input) | Input and pad reads | Input; Tasks (scripts that wait) | I: hold right 30 frames; R: release; mouse/pad readout | No (this packet) |
| [`demo_kit`](kit) | Text, kit and safe area | Drawing (call from on_draw); Kit drawing | K: toggle kit; resize the window to see safe-area layout | No (this packet) |
| [`demo_fighters`](fighters) | Fighter state and control | State; Gameplay | T: teleport P1; P: P2 percent +25; C: stand/fight CPU | No (this packet) |
| [`demo_modifiers`](modifiers) | Passive fighter modifiers | Passive fighter modifiers (engine batch 2) | M: toggle P1 speed/attack buff; baseline is 1.0 | No (this packet) |
| [`demo_reserve`](reserve) | Bench and call a reserve | Reserve fighters (engine batch 2) | B: bench P2; C: call P2 onto floor; refused states shown | No (this packet) |
| [`demo_fly`](fly) | Debug fly cursor | Gameplay | F: fly toggle; T: target 30,40; A: native debug attack; R: clear | No (this packet) |
| [`demo_contacts`](contacts) | Contacts, trace and wait | Contacts and traces | W: land near x=0 within 180 frames; O: overlay; trace auto on | No (this packet) |
| [`demo_camera`](camera) | Camera control | Camera (offline, gameplay mods) | F: follow P1; M: tween; S: shake; R: return to retail camera | No (this packet) |
| [`demo_camera_params`](camera-params) | Retail camera parameters | Camera parameters (engine batch 2) | G: zero yaw/pitch gains; R: stage baseline; walk away from origin | No (this packet) |
| [`demo_collision`](collision) | Collision lines and platforms | Stage content (offline, gameplay mods) | P: move cyan platform; walls red, ceilings violet; jump/drop through | No (this packet) |
| [`demo_models`](models) | Model handles and labels | Stage content; Mission model paths and reloads; Contacts and traces | R: reload optional models/demo.gxmesh; cyan slab is the no-export fallback | No (this packet) |
| [`demo_materials`](materials) | Custom material, glass and light | Custom shaders, post passes and model materials | L: warm/cool light; optional lit/glass/custom meshes; fallback slabs | No (this packet) |
| [`demo_chunks`](chunks) | Mission folder, chunks and reload | Mission model paths and reloads; Stage content | R: reread tiny level; arrows choose chunk; fresh handles per area | No (this packet) |
| [`demo_enemies`](enemies) | Enemy waves | Adventure enemies (offline gameplay mods) | Fight two goombas, then one redead; N: next wave after clear | No (this packet) |
| [`demo_pickups`](pickups) | Standalone Geno pickup | Standalone items and model rotation | I: drop a native coloured pickup by P1; touch it to collect | No (this packet) |
| [`demo_items`](items) | Unified item list | State; Standalone items and model rotation | I: spawn common capsule (kind 0); inspect layer, name and owner | No (this packet) |
| [`demo_events`](events) | Native game events | Hooks; Clank observation and timed presentation; Stage slots and switching | Fight/clank; E: spawn enemy; events log after logic; no fabricated events | No (this packet) |
| [`demo_hitstop`](hitstop) | Timed hitstop | Clank observation and timed presentation | H: 30 nominal frames of freeze; C: cancel own request | No (this packet) |
| [`demo_post_grade`](post-grade) | Colour grade | Custom shaders, post passes and model materials | P: enable/disable pass; compare the same view | No (this packet) |
| [`demo_post_vignette`](post-vignette) | Vignette | Custom shaders, post passes and model materials | P: enable/disable pass; compare the same view | No (this packet) |
| [`demo_post_outline`](post-outline) | Depth outline | Custom shaders, post passes and model materials | P: enable/disable pass; compare the same view | No (this packet) |
| [`demo_post_bloom`](post-bloom) | Whole-scene bloom | Custom shaders, post passes and model materials | P: enable/disable pass; compare the same view | No (this packet) |
| [`demo_post_custom`](post-custom) | Custom WGSL live reload | Custom shaders, post passes and model materials | Edit/save shaders/pulse.wgsl; U: change tint parameter | No (this packet) |
| [`demo_surface_fighter`](surface-fighter) | Fighter surface shader | Fighter and original-stage surface shaders | S: toggle surface tint | No (this packet) |
| [`demo_surface_stage`](surface-stage) | Stage surface shader | Fighter and original-stage surface shaders | S: toggle surface tint | No (this packet) |
| [`demo_parts`](parts) | Per-part texture-preserving tint | Offline character-part inspection | T: tint first supported P1 part; R: clear; refresh before indexing | No (this packet) |
| [`demo_six_slots`](six-slots) | Six fighter launch and recycling | Six-fighter direct matches | L: launch six-slot LAB; K: send P6 below blast; R: recycle after KO | No (this packet) |
| [`demo_data`](data) | Saving script data | Console and files; Data and fighter-state readback | S: add one and atomically save; R: reread; data stays after unload | No (this packet) |
| [`demo_profiler`](profiler) | Profiler and zone readout | State (gd.perf); docs/profiling.md | O: prof on/off; console prof report / prof trace 8 / prof hitch 20 | No (this packet) |
| [`demo_lab_inspection`](lab-inspection) | LAB hitboxes and frame timeline | LAB inspection and control | J: request jab frame 1 (pauses); SPACE: resume; B: debug draw toggle | No (this packet) |
| [`demo_rewind`](rewind) | Save, step and rewind | Rewind, reload and persistent states | S: snapshot 1; L: load; P: pause; N: step; R: resume | No (this packet) |
| [`demo_callouts`](callouts) | Text-only comm callout | Comms callouts (offline) | C: queue a text callout with fallback portrait initial | No (this packet) |
| [`stage_switch_demo`](../stage_switch_demo) | Static slots, switches and queue | Stage slots and switching | load linked mod folder;  | See engine-day handoff; not run here |
| [`missions`](../missions) | Mission folders, streaming, camera modes and maze | Mission model paths and reloads; mission camera modes; maze generator | load linked mod folder; mission maze 42 8; mission maze map | See engine-day handoff; not run here |
| [`bench`](../bench) | Existing measured-workload bench | docs/profiling.md | load linked mod folder; bench status | See engine-day handoff; not run here |
| [`shader-demo`](../../../geno/mods/shader-demo) | Existing combined post sample | Custom shaders, post passes and model materials | Mount shader-demo at boot; F6 toggles; scripts/demo.lua is auto-discovered (manifest has no entry) | See engine-day handoff; not run here |
| [`surface-shaders`](../surface-shaders) | Existing surface shader techniques | Fighter and original-stage surface shaders | load linked mod folder;  | See engine-day handoff; not run here |
| [`envoy`](../envoy) | Supertime Envoy (fourth showcase) | Combined mission, fighter, item, UI and save systems | load linked mod folder;  | See engine-day handoff; not run here |
| [`geno-fighter-tutorial`](../../../../../docs/learn/geno-fighters) | Geno fighter overlay tutorial | melee/docs/geno.md; docs/learn/geno-fighters | load linked mod folder;  | No (this packet) |
| [`demo_effects`](effects) | Native effects and emitter controls | State (gd.fx); LAB effect control | E: play square sparks; C: slow/brighten; R: fade out; F: attach package | No (this packet) |
| [`demo_mission_cameras`](mission-cameras) | Mission camera modes | Camera; missions README Camera | F: follow; C: chunk; V: shaft; R: retail; jump along primitive route | No (this packet) |
| [`demo_gauntlet`](gauntlet) | Gauntlet | Combined missions, maze, waves, reserve, pickups, cameras, slots, post and data | Fight room 1; reach maze goal in room 2; KO boss in room 3; ENTER retries | No (this packet) |
| [`demo_training_card`](training-card) | Training card | Combined LAB timeline, hitboxes, contacts, trace and wait_until | LEFT/RIGHT: move; M: inspect chosen move; D: landing drill; SPACE: resume | No (this packet) |
| [`demo_stage_tour`](stage-tour) | Stage tour | Combined stage slots, queue, transition events, surface shaders and HUD | N: next stage; cycles FD/BF/YS every 8s; wipe/flash/morph | No (this packet) |
| [`demo_tasks`](tasks) | Coroutine input task | Tasks (scripts that wait); Input | Q: jump, drift, wait for landing; task releases its pad claim when complete | No (this packet) |
| [`demo_console_socket`](console-socket) | External console socket | The console (localhost socket) | Python client.py <port> demo_ping; external commands read player state | No (this packet) |
| [`mission-first`](../missions/missions/first) | Existing tiny mission folder | missions README; mission schema | load linked mod folder;  | See engine-day handoff; not run here |
| [`maze-generator`](../missions) | Seeded generator and ASCII map | missions README Generated maze extension; tools/maze/README.md | Load missions, then mission maze 42 8; mission maze map | No (this packet) |

Technique credits: GD scripting/mission/shader examples and native registration/implementation.
The post bodies and Gauntlet's pure maze generator/recipes retain the existing project techniques.
No external techniques, third-party assets or disc-derived data were imported. See workspace CREDITS.md.

- [Pickup with juice](pickup-juice/) (`demo_pickup_juice`): I plain, J the same native pickup with glow, particles and sound; R clear. Source/stub checked; native visual/audio/timing acceptance pending.

## Retail 1P source demos (native acceptance pending)

- `demo_1p_awareness` - [Retail 1P lifecycle](1p-awareness/README.md): Read lifecycle snapshots during a manually started Classic/Adventure run.
- `demo_1p_hold` - [Bounded interstage hold](1p-hold/README.md): Clear a stage; press A on controller 1 or wait 180 host ticks. No retail logic runs during the panel.
- `demo_1p_spawn` - [Retail opponent spawn template](1p-spawn/README.md): Enter Classic and reach the wireframe team: each replacement receives the same port template before its first logic frame. Enemy CPU AI is unchanged.
- `demo_1p_loop` - [Scripted 1P launch and New Game+](1p-loop/README.md): Press C to start Classic Mario Normal/3 stocks; B cancels. Completion restarts. Scripted runs do not write retail progression records and skip trophy/credits/congratulations.
