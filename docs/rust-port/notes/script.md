# Scripting (melee_script)

> **Removed 2026-11-09.** The Lua scripting host described below was taken out of the Rust port (no Lua: scripts and the
> LAB are wasm mods, notes/modhost.md; fighter code is the geno 12 `wasm` block, notes/geno-wasm-plan.md). This note
> stays as the record of what it was; the C build keeps its Lua.

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 1343-1413 -->
## Platform track: scripting, `melee_script` (2026-10-24, port/plat-script, round 1)
- Crate `rust/melee_script` on mlua (lua54, vendored). **Same Lua as the C build**: `rust/third_party/lua-src` is a patched `lua-src` that
  compiles the unmodified `pc/third_party/lua-5.4.7` sources (`[patch.crates-io]` in `rust/Cargo.toml`); stock lua-src ships 5.4.9.
- `engine.rs` ports gw_script.c's plumbing: `gs_build_base` / `gs_new_env` (sandbox: base subset, private copies of every library table
  and `gd`, `string.dump` removed, string metatable locked, `math.randomseed(0)`), script slots (128) with up to 16 coroutine tasks,
  `-- @key:` headers and `mod.json` fields (`manifest.rs` keeps the `strstr`-style reader and its quirks), `gs_pcall` (per-call 2,000,000
  instruction / 500 ms budget through a **raw** C count hook), `gs_run_tasks` (`gd.wait(n)` yields n), 20 errors switch a script off,
  `load_named` / `scan_scripts_dir` / `scan_mods` / `boot` (`MELEE_SCRIPT`, `MELEE_SCRIPTS`), the console (`exec`: `= expr`, help, scripts,
  load/unload/reload, script commands, Lua), FramePre/FramePost/Tick/SceneBegin hook order, camera-completion hook.
- **mlua trap**: an error raised by a Rust callback reaches Lua's `pcall` as a userdata, and a hook error likewise; the C build raises
  strings with the caller's position. So natives never raise: `Engine::native` returns `(sentinel, msg)` and a small Lua trampoline does
  `error(msg, 2)`; the budget hook is a raw `lua_sethook` + `luaL_error` (threads inherit it; mlua's hook callback does not reach coroutines).
- The game is reached through `game::ScriptGame`, the `gw_ScriptGame_*` scalar interface with the C field numbers (script_lab.h, SI_/SF_),
  every method defaulting to "nothing there". The platform through `host::Host`. `pad::ScriptPads` is gw_script_pad.c section 3.
- gd functions ported (~120 of 440): core (log/print, frame, time, rng, rgb, label, text/box/fill/line, key/key_pressed/mouse, command,
  run + prelude wait/wait_until/press/tilt, gd.comm, data_read/exists/write/write_atomic, script, quit, pause/resume/step/paused,
  safe_area), mod_read/list/stamp, input/release/pad/input_mask/input_chord/mirror_pad, scene/match/players/player/char_name,
  hitboxes/hurtboxes/joints/dobjs/project/attrs/motion_name/debug_draw/debug_stage, camera_* (9), set_percent/set_damage/set_stocks/hit/
  impulse/cpu_mode(s)/fly/teleport/fly_speed/fly_solid/hold_hitbox/play_sound, timeline/motion_list/set_motion/anim_rows/anim_preview,
  lab_request/lab_mode/lab_leave/lab_now/lab_common/lab_env/floor_below/set_shield/set_scale/rollbacks(_clear).
- Round 2: `rewind.rs` + `gd/history.rs` port the long rewind (gd.history/step_back/rewind_to/rewind_live/rewind_test(_result), input log with pad/sfx/audio replay, keyframe cadence, `iterations`, FramePre/Post kinds 0/1/2), hot_reload(_status), the `gd.state_*` library (own 384-byte header, index.txt) and gd.kb_preview. Keyframes/snapshot files are `ScriptGame::rw_*`/`snap_file_*` (the game half does not exist in Rust yet; tests/history.rs uses an in-memory mock, 9 tests). Found+fixed: gd.set_percent lacked rw_branch.
- Geno registry (round 3, `rust/melee_script/src/geno/`): `json.rs` (the registry's JSON reader, strtod and `%.9g`), `registry.rs`
  (geno_registry.c: attach entries with attributes/scale/toon/jumps/hooks, v1 special_attributes/on_land/motion_anims/subaction overlays,
  v2 states/params/specials, v5 counter windows/sounds/articles, stable ids, kind table, hot-reload layout check), `consts.rs`/`tables.rs`
  generated from pc/geno/geno.h and geno_game*.c by `tools/gen_geno_tables.py`. `tests/geno_registry.rs` ports test_geno_registry_*; the pinned
  stable id `39ebc1bd1de2fc58` equals the C build's. Found: the C `geno_registry_v2` test asserts phys "none" < 0 but the table returns
  GENO_CBID_NONE (4); the port follows the table. **Not ported**: native `define` entries (geno_define_*.inc; every shipped mod is one, so
  the shipped geno.json files parse to 0 entries), fighter Lua (geno_lua_registry.inc), combo/meter, controls schema, the game half.
- Round 4: `gd/combo.rs` (gd.combo, gd.combo_ki; reads through `ScriptGame::combo_*`), `gd/perf.rs` (gd.perf incl. shader block and
  profiler block, gd.prof; via `Host::perf_snapshot/prof_snapshot/prof_command`), `gd/rec.rs` (all gd.rec_* Lua halves: gr_gate, gr_name_ok, argument
  shapes, status/list tables; the recorder state machine and .gdlp files stay behind `ScriptGame::rec_*`, NOT ported, default impls refuse).
  `tests/combo_perf_rec.rs` (4 tests). Every `gd.*` that lab.lua calls is now registered; 279 names are in the C table, ~150 remain (see below).
- Round 5: `gd/arena.rs` (arena-hooks: stage_set_spawn/stage_spawn/stage_set_origin/stage_set_camera_bounds/stage_set_blast_bounds/
  stage_restore_bounds/stage_collision_group(s), and the merged `gd.stage_bounds` = largemap set/restore + arena origin/frames + model-gaps
  main_floor/surface_top), `ScriptGame::arena_*`/`stage_bounds_i`/`map_bound(s)`, arena release on script unload. `tests/arena.rs` (4 tests,
  mock arena; headless, no input sent). ~140 of the 349 C table names still unregistered (stage_add_*, models, enemies, items, fx, cpu_*, caps...).
- Round 6: `gd/caps.rs` (fighter_caps, fighter_effect, fighter_armour, fighter_armor, give_item, nearest_opponent, opponents_in_radius,
  fighter_timed_status, fighter_interrupt), `gd/bench.rs` (fighter_bench/call/benched), `gd/crit.rs` (crit, crit_force, crit_seed,
  cpu_technical, cpu_assist; the on_crit/armor event queues are not ported). New `ScriptGame` hooks default to "nothing there"; unload calls
  `caps_release(owner)`. Tests `tests/caps.rs` (3), `bench.rs` (1), `crit.rs` (1) run over mocks; `cargo test -p melee_script` all green.
  Headless only, no OS input sent. Still unregistered: cpu_ctl scripts (cpu_pad/cpu_script_done), hit_rules, zones, deadline, echo/afterimage/tracer,
  fx_*, item_define/item_kinds, model_*, stage_add_*/stage_switch/slot, 1P (spawn_1p...), area_*, sim_*, contact_*, skill_*, warm.
- Round 7 (port/plat-script, this session; every item below has a `rust/melee_script/tests/*.rs` test over a `ScriptGame`/`Host` mock, all green):
  `gd/hit_rules.rs` (hit_rule_add/remove, hit_rules_clear, fighter_status, hit_rules), `events.rs` (the engine event queue of gw_Script_GameEvent and
  HitContext/HitRuleContext/EnemyGameEvent/TargetBroken/Enemy*/Skill/Crit/Armor event rows, the one epoch/frame/sequence spine clock, post-frame dispatch
  of on_action_change/on_hit/on_hitlag/on_land/on_target_broken/.../on_action_signal/on_armor/on_skill/on_crit plus the common `on_event` envelope; hook order as
  `gs_dispatch_events`), `gd/entity.rs` (entity_ref/resolve/valid, source_ref/resolve, entity_call), `zones_core.rs` + `gd/zones.rs` (the C zone core with its
  pc/tests/zones_test.c fixture ported as a unit test, zone_add/set/remove, zones, point_zones, zones_at, zone_members, enter/exit/none/some dispatch),
  `gd/deadline.rs`, `gd/skill.rs` (skill_history/state/kinds/thresholds, 25 kinds, on_<technique> hooks), `gd/shock.rs`, `gd/contacts.rs` (the contact observer,
  contacts, contact_events, contact_trace with the JSONL rotation, contact_overlay, wait_until{...}/wait_status watchers, `contacts`/`trace`/`wait` console commands),
  `gd/flycam.rs` (fly_target/attack/clear/state, camera_params), `gd/echo.rs` (echo_add/remove, echoes, fighter_history(_depth); the echo parse/prepare/pair/apply
  helpers are pub for sim_commit), `gd/cpu_ctl.rs` (cpu_pad/script/dest/target/goto/cancel/commands/attrs/macro and the status readers, with the 52 retail CpuCmd
  codes and the eleven technique macros), `gd/present.rs` (parts/parts_id/parts_clear, dobj_solid/tint, the fx_* effects lab, lab_peek), `gd/kit.rs` (gd.kit over
  `Host::kit_*`; without a kit `available()` is false and the drawing calls raise the C build's error, so the LAB loads and runs headless: `gdscript pc/geno/mods/geno-lab`
  prints "the kit is not available, the Lab UI is not drawn" and no error), `gd/frontend.rs` (scene_launch/clear, training_select, screenshot, hud_visible, launch_request/cancel),
  `gd/fmod.rs` (fighter_mod, hitstop/hitstop_cancel and the presentation timers). `ScriptGame` grew the scalar hooks for all of these (every one defaults to "nothing there"),
  `Host` the fx/kit/frontend services; a new `ScriptGame::owner_release` is called on script unload. ~50 gd.* names added: 349 in the C table, **124 still unregistered**
  (afterimage/tracer/echo_afterimage/motion_* need the gw_motion trail service; area_*, boss_*, enemy_*, spawn_*, 1P, item_*, model_*, stage_*, shader/post_*, sim_*, warm_*,
  netbuild/netplay/modsync/mods_install, mode_blob, menu, light_set, render_test, fighter_lua/fighter_recycle/fighter_shader, match_end_*).
  Proof run this session: `cargo test -p melee_script` (all test binaries green, 14 lib tests incl. the zones fixture), `gdscript` over pc/geno/mods/geno-lab (clean) and
  every pc/tests/*.lua that needs the engine loads. Still **not** proven: any of this against the real game (the Rust game half of these hooks does not exist yet).
- **Not ported yet** (next rounds, in order): the .gdlp recorder itself, hitstop, cpu_* scripts, hot_reload, kb_preview, `gd.kit` / `gd.ui` (tables are empty), entity refs (`entity_ref` is nil), items/fx/echo/
  afterimage/tracer, stage_*/model_*/shader/area/warm/launch/1P/netplay/modsync/sim/hit_rules/enemy*, the Geno registry (geno_registry.c,
  geno_lua_core.h, fighter Lua), `lab.lua`'s remaining calls. `gdscript pc/geno/mods/geno-lab` lists what lab.lua still lacks at load.
- Proof so far: `cargo test -p melee_script` (7 unit + 18 engine + 6 lab_api tests); `rust/melee_script/tools/standalone_parity.sh` runs
  every engine-less script of pc/tests, pc/geno/tools, pc/gameworld, pc/scripts under the system `lua` 5.4.7 and under `luarun`
  (this host's interpreter): 82 identical exit status and output (incl. atlas_ui_stub_test, envoy_*, campaign-save-unit, lab_stage_d_check's
  own 9 pre-existing data-dependent failures), 0 different, 58 need the engine (`gd` global) and are not run.
- Traps: write files with the Write/Edit tools (a bash heredoc dropped a backslash and wrote NUL bytes); target dir is `build/cargo` at
  the repo root, not `rust/target`.
