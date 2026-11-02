# Survey of melee-unlocked (v0.9.86, 271402e): what to port, and how to plug in as a third engine

2026-11-02. Read-only survey. Subject: `scratch/ext/melee-unlocked` (hero88go; GPL-3.0-or-later overall, per-file
GPL-2.0-or-later headers on the ported parts). The author gave explicit permission to use his repo as we see fit.
That permission covers HIS code only; section 8 lists what it does not cover. Nothing was built or run, nothing in his
repo or in the user's own checkout was touched, no OS input. Nothing of his is copied here; names are for reference.

How this was done: the seam (section 2) and the shared kit (section 3) were read by hand against the files named.
Sections 4 to 9 come from four parallel read-only passes (display, replays/netplay, menus/audio/input/launcher,
validation/shims/licences); I spot-checked file names, the ABI and the launcher, not every claim. Guesses are marked
"(guess)". Caveats that matter:
- `sourceport/extern/melee` (the doldecomp submodule) and `sourceport/extern/aurora` are EMPTY in this checkout. The
  game as he builds it is doldecomp plus the 8 MB `sourceport/patches/melee-native.patch` (1583 files; 64-bit pointer
  port, 133 `MU_NO_CONTRACT` tags, 218 `MU_P()` wraps, native hooks). So his Source Port is NOT bit-identical to
  vanilla decomp by construction; it is "the decomp plus his edits", validated against replays.
- His validation is looser than ours (section 7). Do not treat his Source Port as an oracle for bit-identity.

## 1. Port first (ranked)

Two sorting keys, per the coordinator: (A) soonest to get our engine running inside his launcher, then (B) what makes it
worth choosing over his two engines. Items marked "free" come from his host once the seam works; we only have to feed
them, not rebuild them. Effort S/M/L is for the Rust side.

### A. Get running inside his launcher (do in this order)
1. **Engine DLL shell: `melee_rust.dll` exporting `mu_game_entry`** (S). Loaded by his unchanged `melee_source.exe`
   through the existing `MELEE_GAME_DLL` env var (`source_host.cpp` ~3442). Must load at the fixed image base
   `0x82800000` (`/BASE /FIXED /DYNAMICBASE:NO`), fill `MuGameApi`, check `MuHostApi.version == 21`. Detail: section 2.
2. **Console-memory arena** (M). The host reserves MEM1 (40 MB) at `0x80000000` and the host's GX decoder resolves
   every texture, vertex array and display-list pointer in the FIFO as a 32-bit address inside it (`gx_core.cpp`
   ~290/304/630: `host::ptr(0x80000000 | ...)`). Anything the renderer or mixer must read has to live in that arena.
3. **GX bridge: emit a hardware-faithful FIFO via `host->gx_fifo`** (M-L; the single biggest item). Our `melee_gx` writes
   Aurora's dialect (`GX_AURORA` 0x50 subcommands, pointers as resource handles). His decoder parses real BP/CP/XF
   registers with real addresses. Either switch the Rust encoders to real register writes plus arena addresses (our
   data loader then allocates DAT/texture bytes from the arena), or send him a small PR adding a handle-resolving
   subcommand. Also required: the XFB copy trigger at frame end (his presenter "follows the EFB copy").
4. **Frame loop on his clock** (S). `run()` blocks on the game thread; per tick call `host->pad_read`, step the sim, then
   `host->vi_wait_retrace()`. The host then calls back `retrace()`, `fire_alarms()` and `ai_dma_done()` (all can be
   near-empty for us). Never sleep or pace ourselves; during rollback re-sim bracket with `host->resim_phase(1/0)`.
5. **Input, disc, card through the host table** (S-M). `pad_read` (`MuPadStatus` is the PADStatus layout), `disc_entrynum/
   disc_file/disc_read` (completion arrives from `poll()`; gives us his mod/cosmetic overlays free), `card_*`
   (Dolphin GCI folder; replaces our `melee_data` card path).
6. **`state_snapshot` = MuStatePod** (S). 12 raw words for each of 6 slots plus rng/scene/major/match_frame. We already
   hash this state. Gives immediate access to his `--state-digest`, `validate_native.py` mode matrix and
   `lockstep_compare.py` against his Source Port: a second cross-check of our engine for free.
7. **Dummy-valid snapshot API + own rollback** (S). `state_regions` must return exactly 3 regions (region 0 = MEM1, two
   non-empty image sections outside MEM1), `state_exclusions`, `snapshot_ranges` must validate. Host-side raw-memory
   snapshots cannot see Rust state, so our `melee_net` keeps its own saves; we call `resim_phase` and use his transport
   (`slippi_command`). A future ABI v7 with opaque `save_state/load_state` callbacks would let his replay viewer
   jump-back and Training Lab savestates work on our engine (PR candidate).
8. **Audio: write mixed PCM into the AI DMA buffer** (M). Keep our `melee_audio` AX mixer, give the host the buffer
   (`ai_init_dma`, `ai_start_dma`, `ai_dma_done`), skip his `ax_ucode.cpp` path. Output, resampling, underrun policy,
   music packs, volume: free from the host.
9. **Launcher: third engine segment `ENGINE_RUST = 2`** (S). `launcher_engine.h` (enum, `parse_engine_line`, probing
   function), `launcher.cpp` (GAME BUILD row ~801/1245, exe choice ~432, crash and lobby labels). Detail in section 2.

### B. What makes it worth choosing (after A works)
10. **Speed claim, measured** (S): 0.30 to 0.50 ms/tick headless vs 3.4 to 3.6 ms for his Static Recomp (bench-unlocked.md;
    his Source Port was not built there). Rollback re-sim at 4.9 ms/frame is our weak spot (bench.md); his snapshot design
    (section 7) is worth studying only if we move to an arena.
11. **Vanilla-exact mode is our home ground** (S). His launcher already has a "Vanilla" segment (`launcher_engine.h`
    `VanillaChoice`: `--slippi-menus off --vanilla-only`). Our engine is bit-identical to vanilla decomp, so that is
    the honest label for us at first; his Source Port needs an FMA tag audit to claim it.
12. **Sub-frame tier 2: pose capture** (L). Tier 1 (matrix interpolate/extrapolate) works from the draw stream alone.
    Tier 2 needs the private FIFO tokens `0xF0`/`0xF1` plus `native_pose_snapshot` (ABI `mu_native_pose.h` v7). Our Rust
    HSD joint/AObj/FObj code is the thing he had to patch the decomp to expose; filling the snapshot from Rust is cleaner.
13. **Native Slippi codes in the engine** (L, GPL-3 derived on his side; see section 8): UCF 0.84, widescreen camera hooks,
    replay start/frame application, online forced codes, `camera_tasks` between re-sims. Gate for frame-exact `.slp`
    and Slippi online. Re-derive from the Slippi code list and our decomp, do not transliterate his C.
14. **Replay start-block conversion** (M). Apply ports, costumes, stocks, timer, teams, stage from the `.slp` Game Start
    block (0x138 bytes) the way his `mu_replay_start_melee` does; read the replay's own Gecko list to decide UCF and
    other codes (not the per-port toggle words). Closes the frame-0 divergence in replays-ext.md.
15. **Hardware-op exactness policy** (M-L audit). His tagged FMA lists (in the patch) are a census of where GCC fused
    versus where the console did. Useful as an independent cross-check of our `fmadds` sites (STATUS: clang `-ffp-contract=on`).
16. **Menus**: retail `mn*`/`gm*` scenes run unchanged in his engines (the decomp's own code). For us the gap is still
    ours (STATUS: stand-ins). Nothing to take except the hardware shim list (`mu_vi/mu_dvd/mu_card/mu_pad/mu_audio/mu_os`).
17. **Lab view + Practice** (host side free once `mu_look_snapshot_v1` / `mu_look_items_v1` are exported by the DLL (S-M)
    and `practice` op bridge implemented (M)). Training mods (TM-CE, 20XX) are third-party content: skip.
18. **Test-harness imports** (S-M each): per-frame POD digest + mode matrix, parity-table twin harness, replay_compare field
    list and negative-frame ULP rule, net_fault_proxy.py.

Skip: DLSS/XeSS/DXR/Reflex (host-only, platform), PhobGCC (GPL-3), ghost AI, Akaneia/ACE/rogue/Maker/20XX content.

## 2. Engine seam: how a Rust engine becomes a third engine

### 2.1 How his launcher and runtime pick an engine
- Two engines, two executables. Static Recomp = `melee_port.exe` (translated PPC). Source Port = `melee_source.exe` +
  `melee_game.dll` (decomp C, MinGW GCC 14+). CMake: `port/CMakeLists.txt` defines `melee_source` (L85, from
  `app/main.cpp`, `app/source_host.cpp`, `app/source_guest_boundary.cpp`, `app/settings_window.cpp`, `runtime_ns`
  static lib) and the launcher `melee_unlocked` (L199). `MELEE_NO_SLIPPI` builds a Slippi-free variant.
- Launcher: `port/app/launcher_engine.h` is the testable pure part: `enum Engine { ENGINE_LEGACY = 0, ENGINE_SOURCE = 1 }`;
  `source_exe_dir()` offers Source only when `melee_source.exe` AND `melee_game.dll` sit together (or under a checkout's
  `build-sourceport/port/Release`); `parse_engine_line()` reads `engine=` in `launcher.ini` and clamps to `<= ENGINE_SOURCE`;
  `VanillaChoice` / `vanilla_launch_args()` add `--slippi-menus off --vanilla-only [--vanilla-game]`.
  `launcher.cpp`: `g_engine` (L164), exe choice (~L432 to 438, 1385, 1424), GAME BUILD segments (~L801 to 812, the click
  handler ~L1245), lobby/crash tags `:source`/`:recomp` (L1873, 1916, 2022, 2066, 2141) which also keep players of
  different engines from pairing. Test: `tests/launcher_engine_test.cpp`.
- Runtime: `source_host.cpp` (`source_port::run`, ~L3340 to 3545) reserves memory (`reserve_memory`, L3344), loads the
  library (`g_dll = "melee_game.dll"`, overridable by env `MELEE_GAME_DLL`, L3442), checks `module == GAME_IMAGE_BASE`
  (0x82800000, L3482), resolves `mu_game_entry` (+ optional `mu_look_snapshot_v1`, `mu_look_items_v1`, `mu_user_gecko_write/read`,
  `mu_ak_native_build`), calls `entry(&api, &g_game)`, validates versions and state regions, then `g_game.run()` on the game thread.
  It also refuses a non-vanilla DOL (`host::disc_has_vanilla_dol`).

### 2.2 The interface (all in `port/runtime/abi/mu_host.h`, plain C, shared by MSVC host and GCC game)
`MU_HOST_API_VERSION 21`, `MU_GAME_API_VERSION 6`. One export, `int32_t mu_game_entry(const MuHostApi*, MuGameApi*)`;
the host fills the first table, the game fills the second; the library must return nonzero on a version mismatch.
- **Game must provide (`MuGameApi`)**: `run` (the blocking game main), `retrace` (one 60 Hz tick, called by the host from
  inside `poll`/`vi_wait_retrace`), `fire_alarms(now)`, `ai_dma_done`, `state_snapshot(MuStatePod*)`, `lcancel_view`,
  `state_regions` (exactly 3), `state_exclusions`, `snapshot_ranges`, `practice(op,args,n)` (ops `MU_PRACTICE_*`).
- **Host provides (`MuHostApi`)**: `log`, `panic`; time (`ticks` = 40.5 MHz timebase, deterministic under `--time-base`,
  `boot_time`); `poll` (called from every game spin; delivers disc completions, audio, alarms, retrace); graphics
  (`gx_fifo`, `vi_configure`, `vi_set_next_framebuffer`, `vi_flush`, `vi_retrace_count`, `vi_next_field`, `vi_set_black`,
  `vi_wait_retrace`, `vi_idle_step`); input (`pad_read`, `pad_rumble`); disc (`disc_entrynum/file/read/status/id`); card (14
  `card_*`); audio (`ai_init_dma`, `ai_start_dma`, `ai_set_sample_rate`, `ai_set_stream_volume`, `dsp_mail`,
  `dsp_mail_pending`, `aram_base/size/dma`); `native_alloc/free`; machine (`mem1_size`, sound/progressive mode, reset,
  `stop(reason)`); scripted-run hooks (`match_override`, `rng_seed_override`, `mark_match_start`); option words
  (`game_options`, `game_options2`, `game_options3`, `display_options`, `audio_options`, `mod_flags`, `music_volume`,
  `hud_scales`, `picture_widen`, `shield_color`, `grid_color`); display-only feeds the game pushes (`hud_player`, `hud_combo`,
  `native_pose_snapshot`, `native_render_scope_event`, `native_draw_identity`, `native_owner_tracking`); replay
  (`replay_start`, `replay_frame`, `replay_stock_steal`, `replay_event`, `replay_finished`); Slippi/online
  (`slippi_command`, `resim_phase`, `online_test_match`). Every function pointer added after v1 can be NULL on old hosts; a
  game checks `host->version` and NULL.
- **Renderer handoff**: his renderer consumes GX hardware command bytes at the FIFO layer, the same decoder the Static
  Recomp uses (`gx_core.cpp`: `write_fifo_bytes` -> `drain_fifo` -> `parse_command`). The Source Port's own GX library
  (`sourceport/game/gxnative/GXFifo_native.c`) stages bytes in a 64 KB buffer and flushes through `mu_host_gx_fifo_bytes`.
  Textures, arrays and display lists are read from game memory by address. Draws are recorded per frame into
  `Frame` objects and pushed at the XFB copy to the render thread (`frame_queue.h`). Sub-frame, widescreen, MSAA, DLSS,
  packs all live downstream of this boundary, so a faithful FIFO inherits them all.
- **Save states / rollback**: host-driven raw memory snapshots (`native_savestate.cpp`: MEM1 + image .data/.bss minus
  `MU_EXCLUSIONS`, 4 KB chunks, write-watch dirty tracking, 7 slots). The game triggers them through
  `slippi_command(CMD_CAPTURE_SAVESTATE / CMD_LOAD_SAVESTATE)` which the host intercepts (`source_host.cpp` ~L1595); the
  game brackets re-simulation with `resim_phase`. During `g_resim` the host's `poll` and `vi_wait_retrace` return at once.
- **Slippi playback/online**: game calls `replay_start`/`replay_frame` (Source) and `slippi_command` 0xB0 to 0xE5
  (online); the host owns matchmaking, ENet transport, `.slp` writing, spectate.
- **Replay validation / checkpoints**: `--state-digest <csv>` (Source) writes `MuStatePod` per retrace from
  `g_game.state_snapshot`; `tools/validate_native.py` runs 5 render modes and diffs.
- **Settings UI**: entirely host (`gx/pc_settings.cpp`, ImGui). Reaches the game only via the option words above.

### 2.3 What the Source Port does to plug in (the template)
`sourceport/game/shim/mu_entry.c`: `mu_game_entry` calls `__sinit_trigf_c()`, stores `mu_host`, fills `MuGameApi`
(`mu_run` -> `mu_fill_from_dol(); mu_general_codes_boot(); main()`). Hardware shims forward to the table:
`mu_vi.c` (framebuffer/retrace), `mu_dvd.c`, `mu_card.c`, `mu_pad.c`, `mu_audio.c`, `mu_os.c` (interrupt-disable is a
counter; alarms; time from `ticks()`), `mu_cache.c`; the AX library runs natively (`axnative/AXOut.c`) and the host mixes
(`hle/ax_ucode.cpp`); GX via `gxnative/GXFifo_native.c`. Everything else in `shim/` is Slippi-code replacement
(`mu_online*.c`, `mu_replay*.c`, `mu_gecko.c`, `mu_record_start.inc`) or mod content (`mu_te*`, `mu_hp*`, `mu_tmce*`...).
The image is linked at a fixed base with disc structs in big-endian storage (`DISC_STRUCT`/`DISC_PTR`) so that pointers
the GX registers hold are physical addresses below 64 MB.

### 2.4 What a Rust engine has to implement or expose
Shape: a `cdylib` (`melee_rust.dll`) beside `melee_source.exe`. Static lib is not viable (the host `LoadLibrary`s it);
an exe-only engine would duplicate the whole host. No change to his repo is needed for a first run
(`MELEE_GAME_DLL=melee_rust.dll melee_source.exe --iso ...`).
Obligations and traps:
1. Exports: `mu_game_entry` (required); optional `mu_look_snapshot_v1`, `mu_look_items_v1` (Lab view), `mu_user_gecko_*`
   (skip), `mu_ak_native_build` (do not export).
2. Load address: link with `/BASE:0x82800000 /FIXED /DYNAMICBASE:NO`; the host holds the range and frees it just before
   `LoadLibrary` (`g_game_image_reservation`). The reservation size was not checked (to verify in the spike). Provide two
   non-empty writable image sections for `state_regions` regions 1 and 2 (e.g. two `static mut` arrays).
3. Memory: use the host's MEM1 as an arena for GX-visible and AX-visible data only; the sim state can stay in Rust types.
   `mem1_size()` is 40 MB while the game believes 24 MB (extra only on heap exhaustion), so keep the arena small.
4. Threads: `run()` is called on the game thread under a vectored exception handler (`on_game_exception`); never unwind
   a panic across the boundary: `catch_unwind` then `host->panic(file,line,msg)`.
5. Floating point: the host is AVX2/FMA; our code is independent of that, bit-identity is ours to hold. His `MU_NO_CONTRACT`
   knowledge only matters if we compare against his engine.
6. Gaps in the ABI for us (PR candidates): opaque save/load callbacks (to replace raw-memory snapshot); handle-resolving
   GX subcommand; a "no MEM1" capability flag. Everything else exists.
7. Launcher change (S): add `ENGINE_RUST = 2`; extend `parse_engine_line` bound; `rust_exe_dir()` probing `melee_source.exe` +
   `melee_rust.dll`; third segment in the GAME BUILD row; exe choice + `MELEE_GAME_DLL` env at spawn; `:rust` tag for the
   lobby and crash report; packaging scripts (`tools/package_release.py`) ship the DLL; `tests/launcher_engine_test.cpp`.
   Or add a `--game-dll` flag next to the `MELEE_GAME_DLL` env in `source_host.cpp`.

## 3. Shared menu kit (our `menu/out_kit` inside his UI)

**What he took.** Provenance in `port/runtime/gx/ui_sources/gd_melee/ORIGIN.md` and `docs/third-party-ui-attribution.md`:
workspace `GurekamDhillon/gd-melee-workspace` at `477aff14eb26...` (our commit of 2026-09-22) and runtime
`GurekamDhillon/melee` branch `pc-port` at `efc484806c41...`; an earlier adaptation used `9b6753cd2969`. Imported by
`tools/import_gd_kit.py <workspace> <runtime>` into `gd_melee/assets/kit/`:
`kit.json`, `chrome_layout.json`, `list_layout.json`, `widgets_layout.json`, `kit_motion.json` (copied from `menu/out_kit/`),
`font_manifest.json` (from `_build/ui`), `SourceSans3-{Bold,Black}.otf` + OFL text, button/icon PNGs decoded from our `.gxtex`
(`ico_options`, `glyph_a`, `glyph_b`, `glyph_dpad`), eight Source Sans 3 atlas pages decoded with our `pc/tools/png2gx.py`, and
a generated `font_metrics.h` (glyph advance/offset/size/UV + kerning per role: caption, body, row, label, title, heading,
hero, display). Also older `menu/out/ora/frame_*.ora`, `panel_bg`, `row_ng/row_sel`, cursor art (kept "for history").
Plus `menu/pipeline/hub_motion.py`'s `ev_slide_in`/`ev_slide_out` keyframes, in `motion.h` (Hermite spline, tangents in px per
60 Hz frame). He cites `gmfrontend_kit.inc` and `gmfrontend_kitlist.inc` as the runtime reference.

**Revision check (I compared bytes).** The five JSONs he copied are byte-identical to our `menu/out_kit/` today. His
`font_manifest.json` is 358 KB, ours (`menu/out_kit/font/font_manifest.json`) is 822 KB: ours gained the Atlas roles
(Barlow Condensed, Source Sans 3 Semibold, Hasklug; commit d312091, 2026-10-06). We have 30 `menu/` commits since his
revision, and files he does not have: `css_tokens_layout.json`, `cursor_layout.json`, `dialog_layout.json`,
`glyphs_manifest.json`, `manifest.json`.

**How his C++ consumes it: it does not parse the JSON.** Grep finds the JSON names only in the import script. The numbers
are hard-coded in `pc_settings.cpp` (`settings_gd_chrome` ~L2680 to 2720, `settings_gd_home` ~L2725 to 2790): header slab
`(84,24)-(120+title_width,52)`, rule `(116+w,46)-(550,52)`, footer bars `(90,398)-(596,422)`/`(90,430)-(596,456)`, row
`x=96`, `y=84+34i`, `444x30`, baselines 45.92/68.62/447.62/20.28, colours as literals (`#2e3640` face, gold selection). The
shear is the same affine as ours: `x' = x + (240-y)*0.25` (`point` lambda), scale = window width / 640. Text goes through
`text.inl` (`gd_kit_text`, `gd_kit_text_width`): per the manifest's `setting` rules (kerning[prev+ch], quad at pen+offset,
advance, `?` fallback, caps-only roles upper-cased, `max_width` ellipsis), per-vertex shear, atlas PNG as an ImGui texture.
So what is "contract" for him is the generated `font_metrics.h` and the PNGs; geometry and colours are copies.

**What he built on it.** Only the Options section (KS_OPTIONS palette): `settings_gd_home` (the 8 category rows VIDEO, AUDIO,
GAME, CONTROLS, OVERLAYS, CUSTOMIZE, GECKO CODES, MODS) with a help line, `settings_gd_chrome` (header, breadcrumb, hint bar
with A / D-pad / B glyphs), detail pages, tab buttons (`overlay_style == 2` branch ~L3200 to 3245) and the panel slide
(`panel_slide_x`, `slide_in/slide_out`, ~L5107). Sliders and toggles are ImGui widgets (`settings_slider`, `settings_toggle`)
with colour pushes; I found no consumer of `widgets_layout.json` or `list_layout.json`. It is one of six selectable
appearances; the others do not use the kit.

**Changed or added beyond our version.** Row entrance stagger (12 frames, 2-frame offset, cubic ease, 48 px) which is his,
not in `kit_motion.json`; selected row lifts exactly (-5,-5) with a gold-dark shadow (our `row_select` curve peaks near
-3.9 then settles at -3; deviation); `slide_in` retarget from a live offset (his comment: the source's -120 px/frame tangent
raced and clipped at high refresh), so panels interrupted mid-slide ease back over 17 frames; high-refresh-rate time
(`DeltaTime*60`); ImGui nav/hover integration; `settings_palette_tint`/`settings_accent` so the user can recolour; hit
testing on the sheared shape (`settings_shaped_hit_button`). The kit is explicitly marked "visual and interaction validation
required" in his ORIGIN.md.

**Licence.** Both repos state GPL-2.0-or-later for this material (his ORIGIN.md: "Both repositories state GPL-2.0-or-later";
NOTICE lists "GD's Melee (GPL-2.0-or-later): settings motion and art"). It is the user's own work, so no permission
question, but note his repo as a whole is GPL-3.0-or-later and NOTICE says GPL-2-or-later parts keep their notices. Fonts:
Source Sans 3 is SIL OFL 1.1 (not GPL). The art is called placeholder art upstream.

**Recommendation: one kit contract, read by both ports.**
1. Canonical source stays `menu/out_kit/` in the workspace; the JSON is the contract and the PNG/gxtex atlases are generated
   from it (as today). Add a top-level `"schema": N` and a `kit_version` to every JSON (and to `manifest.json`) so a consumer
   can refuse or warn on mismatch; keep file names stable (his importer hard-codes them).
2. Rust side: one crate, e.g. `melee_kit`, with `serde` structs for the five layout files + `font_manifest` + `kit_motion`,
   parsed at startup like our C does (`gmfrontend_kit.inc` parses once into static tables), no codegen of geometry. Layer
   on it: (a) tokens (`@face`, `@face_hi`, section palettes, `bands`); (b) the shear as one `Affine` (`x' = x + 60 - 0.25 y`,
   applied last, as the kit says), which makes Vello a natural fit (flat quads as fills, textured quads as image brushes
   under one transform); (c) text layout from the manifest, NOT from Parley shaping, for kit text: copy the C rules
   (kerning, offset, advance, caps-only roles, step down one role then ellipsis, `?` fallback) so widths match the C and
   the C++ to the pixel; Parley is for non-kit text; glyphs can be drawn either from the atlas pages or as outlines from
   the OTFs at manifest pen positions; (d) motion: evaluate `kit_motion.json` events (Hermite `SPL` keys with `slope`,
   `from_current`, string `value_expressions` over `dir,value,max,delta,target`) in a tiny interpreter; his `slide_in` fix is the
   one deviation worth upstreaming into the JSON as an event.
3. Taffy: the kit screens are absolute 640x480 layouts (`list_layout.panel`: rows_top 84, pitch 34, visible 9), so they
   do not need Taffy. Use Taffy only for new, non-kit screens, and read pitch/margins from the kit so the two styles agree.
4. Conformance: a Python reference (or the C) dumps, for a fixed string set and each of the three list states, the text
   widths and the first quad vertex list; check these golden dumps in once and test them in C (`gmfrontend_kit`), C++ (his
   `gd_kit_text_width`, his `tests/gd_melee_ui_motion_test.cpp` for motion) and Rust. Pixel diffs are optional.
5. Sync: give him a documented `import_gd_kit.py` pin (it already records revisions), and offer to move him from
   477aff1 to the current kit (adds Atlas fonts, dialog, cursor, css tokens). Our rule stays: never hand-edit the
   generated tables.

## 4. Display and render performance (shared host; platform-only unless noted)
All in `port/runtime/gx/`; the solver is shared by both of his engines; only pose capture differs.
| Feature | Where | How | Logic risk | Closes | Effort |
|---|---|---|---|---|---|
| Sub-frame animation (frame interpolation at any fps) | `subframe.cpp` (`SubFrameSolver::set_frames/build/fractional`), `authored_pose.cpp` (`sample_authored`, `sample_authored_envelope`, `carry_camera`), `native_pose_bridge.cpp`, `abi/mu_native_pose.h`, `threaded_backend.cpp` | Sim at 60 Hz; render thread holds prev+current `Frame`, pairs draws by identity (vertex count, primitive, state, textures, matrix bindings), then per presented phase t rebuilds matrices: authored joints re-sample the FObj tracks at `frame + t*rate` (validated against the captured matrix, tolerance 0.002, "endpoint proof"), skinned envelopes rebuilt as `view*sum(w*world*env)`; untracked motion extrapolates the last delta (cap 30 units); camera by screw motion (decline if translation >40 or rotation >1.2 rad; cut if jump >120); one shared "stage lock" camera moves unpaired draws; UVs blend; HUD (ortho) held. Default mode `AuthoredInterpolate`; menus always interpolate. | None. Read-only capture. Source Port: `#ifdef MU_NATIVE` hunks in `pobj.c`; Static: `render_observer.cpp` reads guest RAM. | Frame interpolation, unlocked fps | Free via seam (tier 1). Tier 2 (pose snapshot from Rust): L |
| Pacing, vsync off, VRR | `threaded_backend.cpp`, `vblank_clock.h`, `vblank_model.h`, `vrr_detect.h`, `dxgi_paced_mode.h`, `gx_d3d12.cpp` present | Phase from the tick schedule not hand-over time (steps >2 ms: 262 -> 15 per 40 s); stale in-between presents cancelled; `WaitForVBlank` thread with median period; VRR cap `hz*(1-hz/3600)`; waitable swapchain latency 1; `MELEE_SCANLINE_SYNC` tear-free with vsync off. Default vsync ON because off tears on fixed refresh (docs/hbox-triage-2026-10-28.md). | None | vsync off, pacing | Free; wgpu equivalent for our own window: M |
| True widescreen 16:9 | `gx_shader.cpp::build_projection` (`g_true_widen` = 219/320 at 16:9, first projection row, perspective draws only), `render_options.h` (`AspectMode`, `presented_aspect`), `gx_core.cpp` | Renderer-side widening; excluded: frustum-filling backdrops, non-full viewports (RTT, FoD mirror), authored fullscreen flashes, corner-pinned layers, nametag ortho layer (re-spaced instead); ortho HUD deliberately not widened (measured). Melee's real aspect is 73:60 (`cm/camera.c`, `ty/toy.c`), not 4:3. CSS bands stretched for ultrawide. | Logic-adjacent only in the Source Port's code path (below) | 16:9 camera and HUD | Free via seam |
| Slippi widescreen (Gecko 320/219) | Static: recompiler emits both variants of 3 instructions + 2 hooks behind `gecko::option_widescreen`; Source: `mu_gecko.c`, `CObjLoad` aspect, `ifmagnify`, `ifnametag`, `lbbgflash` constants | Camera aspect x 320/219; HUD constants switch. Source Port draws out-of-4:3 fighters in `ftDrawCommon_80080E18` with `SnapshotTake/Restore` (memcpy of Fighter + JObj tree around the draw because drawing sets bone matrices that gameplay reads). | YES, bit-identity risk if restore is incomplete. Mutually exclusive with true widescreen (would widen twice). | Camera hooks | For us: prefer true widescreen (render-only). If hooks are wanted, draw from a copy |
| EFB copies | `gx_core.cpp` BP trigger -> `EfbCopy{dest_addr,stride,rect,format,to_xfb,clear,half_scale,is_depth,efb_alpha}`; D3D12 `execute_copy` (~L2632) | `to_xfb` ends the frame and submits; texture copies stay GPU textures keyed by guest dest address, sampled by later draws; half-scale = linear 2:1 blit; non-alpha EFB forces alpha 1 (Classic STAGE CLEAR zoom bug); depth via R32_FLOAT blit; never written back to RAM. | None | GXCopyTex | We have a path; the address-keyed registry is M |
| Pipeline caches and prewarm | `gx_d3d12.cpp` (`PsoJob`, `prewarm_fallback_psos`, `save_shader_blob`, `ID3D12PipelineLibrary`), `recipes.bin` | PSO key {vs,ps,blend,zmode,cull,topology,zcontrol,mvec}; compile on worker threads; a draw whose PSO is not ready uses a generic fallback PSO (position+vertex colour+tex0, one per raster state), not skip (skipping caused pop-in/flicker); `PipelineRecipe` list (<=16384) merged across shader versions and prewarmed in parallel before the guest boots (6555 pipelines 10-13 s cold, 0.6 s warm); a warmed `recipes.bin` ships in the zip; even 31k prewarmed did not cover new character/stage pairs, hence the fallback. | None | Pipeline cache | M (wgpu pipeline cache + recipe list + fallback) |
| Draw merging | none found | Only geometry caching per frame (`FrameGeometry geometry_[4]`) so repeated sub-frame presents of one sim frame reuse buffers; constants built in CPU memory then one contiguous upload; per-draw cached pipeline pointer. | None | (Not a feature there) | n/a |
| Threaded renderer | `threaded_backend.cpp`, `frame_queue.h`, `SolverPool` | Sim and render on separate threads, zero-copy hand-over with buffer recycling, cap 32 frames / 256 MB (slow GPU reached hundreds of MB), solver on its own pool polling cancel every 32 draws. | None | Perf | Free |
| MSAA / SSAA / internal res | `render_options.h` (`msaa` 1/2/4/8, `ssaa` 1..4 per side), `present_shader.h` | Multisampled EFB resolved before every read; SSAA = EFB scale x ssaa, box filtered in present; auto scale = integer scale covering the window, clamped so DLSS render height never exceeds output (a 2.7 GB log black-screen bug). Present pass: CAS sharpen, HUD mask, CRT model. | None | MSAA | S |
| Backends | `gx_d3d12.cpp` (4051 lines), `gx_d3d11.cpp`, `gx_vulkan.cpp`, Linux GL (`port/linux/renderer_gl.cpp`, not read) | Same HLSL generated from GX state (Dolphin-derived, section 8) for D3D11/12/Vulkan (DXC to SPIR-V). | None | D3D backends | Free |
| Texture packs | `texture_pack.cpp` | Dolphin-compatible hires names `tex1_<w>x<h>_<hash>_<fmt>`, XXH64 over raw GX bytes, `_mipN`; async replacement. | None | Custom textures | S-M; derived from Ishiiruka code: not covered |
Cleverness here: intangibility flash adds a TEV stage and unpairs fighters (pairing ignores shader identity for tracked
draws); the match camera has a tiny negative depth scale (~-6e-6) so projection smoothing keeps depth terms constant; FoD's
reflection camera is chosen by majority; the timer's hundredths use a perspective HUD camera, not ortho; unpaired stage
draws (51k per interval on Yoshi's Story) must still follow the camera or the stage slides; zero-scale flat foliage cards
make the normal-matrix delta singular (hold normals, keep positions); an RGB8 EFB has no alpha on hardware.

## 5. Replays, Slippi, netplay

Key design: the Source Port does not run Slippi's PPC playback codes. The host parses the `.slp`; the decomp C has
`MU_NATIVE` branches standing in for each code. The Static Recomp runs the real Slippi codes (translated PPC) and answers
over EXI. The host side (`slippi_playback.cpp`, SlippiLib) is shared.
| Feature | Where | How | Logic | Closes | Effort |
|---|---|---|---|---|---|
| Replay start: ports, costumes, stocks, timer, teams, stage from the file | Source: `sourceport/game/shim/mu_replay.c` (`mu_replay_start_melee`, `convert_rules`, `convert_player` x6, `mu_replay_prepare_scene`, `mu_replay_boot_mode` = GM_DEBUG_VS), host `source_host.cpp` ~L3300, `slippi_playback.cpp::prepare_game_info` | The host keeps the raw 0x138-byte Game Start game-info block (console layout, BE). `convert_rules` unpacks timer, stock mode, item freq, SD penalty, stkind (+0xE), time limit (+0x10), game speed; `convert_player` copies ckind, slot type, stocks, color, spawn pos/dir, team, handicap, damage, ratios, scale (players at +0x60 + 0x24*i); nametags forced 0x78; Sheik/Zelda ext ids patched into the header. Whole block => ports, costumes, stocks, timer all come from the file. Stage > 85 refused (mods). | Logic | The frame-0 divergence in replays-ext.md | M |
| Per-frame replay inputs and RNG | `mu_replay.c::mu_replay_input` (~L640), `restore_rng_proc` | Overwrites `fp->input` sticks/triggers/buttons with the file's post-processed values AND writes the RAW stick bytes into `HSD_PadLibData.queue[qread].stat[port]` so UCF sees raw input; per-frame RNG reseed; frame-0 spawn fixup by hand (ftPartSetRotX, cur_pos/last_pos, camera box snap, `Camera_8002F3AC`); `FRAME_RESP_TERMINATE` ends it; stock steal via file. | Logic | Frame-exact `.slp` | M |
| UCF and the other gameplay codes | `port/app/source_host.cpp::replay_codes()` (~L2730), `mu_gecko.c` (`mu_ucf_pad_buffer`, `mu_ucf_dashback`, shield drop, SDI, tumble), `mu_native.h MU_RC_*` | Rules are NOT driven by file fields but by which gameplay injections the replay's own Gecko list contains: bitmask `MU_REPLAY_CODE_*` (UCF 0.84 at 0x800D65EC/C9A44/6B460/8E54C/9A0B8/998A4/93294/908F4, NeutralSpawn 8016E510, FreezeGlitch, InitStage/PlayerData, BrawlOffscreenDamage, DeadUpFall x3, FD BG seed, 4 Stadium codes, WhispyFix, Nana determinism, PS monitor, PreventWobbling x3). UCF = `mu_general_codes() && mu_replay_allows(UCF084)`; a 4-deep raw-stick ring per port with float arithmetic mimicking PPC fused ops. Unknown UCF address sets `UCF_OTHER` (no consumer found). The per-port `ucfToggles` words (off/UCF/Arduino) are parsed and only passed on the Static EXI path; the Source Port ignores them, and `isPAL` has no consumer (as far as I found). | Logic | UCF in every Slippi file; this is the biggest known playback-fidelity item | L (UCF is a decomp-level C change; we port the 0.84 behavior ourselves) |
| Denylist and code list | `slippi_playback.cpp::build_denylist_in`, `gameplay_codes()`, `prepare_gecko_list`, `note_gecko_list_dma` | Drops injections not tagged `[affects-gameplay]` in Slippi's InjectionLists JSON plus 9 hard-coded old addresses, PreventMusicAlarm, MuteMusic. Static build: the replay's own list is baked at recompile time (`--extra-gct <gecko_list.bin> --extra-gct-base 0x80BE0A00`); without it frame 221 diverged; lists of 5728 vs 5632 bytes from 3.19.1 both replay 0-mismatch since only the cave ADDRESS matters. Playback set files in `port/slippi_sys_playback/` are copied from Slippi Ishiiruka. | Logic | Gecko list handling | S for the idea, files not covered |
| Frame protocol | `slippi_playback.cpp::prepare_is_file_ready/prepare_game_info/prepare_frame_data/prepare_is_stock_steal` | WAIT / CONTINUE / TERMINATE / FASTFORWARD; >=3.7.0 waits for `GetLastFinalizedFrame() >= frame`; per-character 52-byte blob (seed, sticks, triggers, buttons, loc, facing, anim, raw sticks, percent). | Platform | n/a | S |
| Replay compare gate | `tools/replay_compare.py`, `replay_batch.py`, `docs/replay-validation.md` | 8 post-frame fields per frame/port/follower, floats by bit pattern: action state, x, y, facing, percent, shield, stocks, internal char. Known baseline: 6 one-ULP Y pairs at frames -118..-108 (spawn animation), allowed only with `--allow-preframe-ulp`; everything from frame 0 must be exact. Batch cache keyed by SHA-256 of exe/DLLs/discs/tools. | Test | Our `golden --slp` | S-M (we compare hash, add field compare) |
| Replay viewer | `host/replay_control.cpp` (`gate(frame)`), `replay_coach.cpp`, `replay_scan.cpp`, `replay_revisions.cpp`, `gx/replay_bar.cpp`, `replay_hud.cpp` | Pause, step, 0.5x/0.25x, fast-forward, jump back (savestate else restart from -123 unpaced), takeover of one port live from frame N, "as experienced" via `<replay>.trace`. | Platform; takeover writes input | Lab | Free (jump-back needs ABI v7 for us) |
| Slippi online (host) | `hle/slippi_online.cpp`, `slippi_net.cpp`, `exi_slippi.cpp`, `slippi_report.cpp`, `slippi_spectate.cpp`, `native_slippi_bridge.h` | Wire-compatible Dolphin SlippiNetplayClient/Matchmaking over ENet to mm.slippi.gg; commands 0xB0..0xE5; rank/ranked deliberately not offered; mods only in Direct/Teams (`native_online_policy.h`); `SLIPPI_SEMVER` 3.6.4; reporting posts to internal.slippi.gg with a user-agent claiming SlippiDolphin 3.6.4 (policy risk). | Platform | Slippi online | Free, but derived from Ishiiruka: not covered |
| Native online codes | `shim/mu_online.c`, `mu_online_rules.c`, `mu_noslippi.c` | Pad frame N consumed by the body whose `gm_80479D58.unk_8 == N` (4746/4746 samples); SyncRNG seed = `((f<<16)|(f>>16)) + rng_offset`; frame-end checksum XOR of motion id, pos x/y, percent bits, spawnNum + float sums (Slippi wire compatible; mu-net treats only >1 diff in low 16 bits as desync); forced-on rollback codes: INIT_STAGE_DATA, INIT_PLAYER_DATA, OFFSCREEN_DAMAGE, DEAD_UP_FALL, FD_BG_SEED, PS_ZERO_BUFFER, PS_IS_VALID, PS_FILE_LOAD, WHISPY_FIX, NANA_DETERMINISM, PS_MONITOR. | Logic | Rollback correctness | L; ours (`melee_net`) already has rollback, add these determinism fixes |
| Re-sim trap | `mu_online.c::camera_tasks()` | Between re-simulated frames run `Camera_8002A4AC`, `HSD_CObjSetCurrent`, `mu_refresh_part_matrices` per awake fighter. Gameplay reads render-derived camera and bone matrices; without it a rollback "lands one rounding step away". | Logic | Our resim cost and correctness | S (verify our engine has this state) |
| Early / sub-frame inputs | `host/early_input.cpp`, `early_input_rules.h`, docs/early-inputs.md | 1 ms sampler; zero-crossing rules (stick 23/64/53/-56, shield-drop -53 while shielding, trigger 43); 13-byte payload (target frame BE + port + 8 pad bytes); receiver keeps 8 per remote, predicts with newest early pad whose target <= N; remote guessing wrong >60% of 64 is ignored; confirmed pad always decides. Credit: SSBM_Arte (third party idea; not covered). | Prediction only | Netplay latency | M |
| mu_net (his own P2P protocol) | `runtime/mu_net/*`, docs/mu-net-protocol.md | ENet framing, XChaCha20-Poly1305 + Ed25519/X25519 via Monocypher (BSD/CC0); COMMIT 0x10 up to 128 pad frames newest first, 7-frame input window anchored at the CONFIRMED frame (anchoring at newest confirmed a wrong pad), stall if newest < f-7, hold limit 7 s, time sync every 30 frames (mean of the middle third of 30 samples, offset = (now - rtt/2) - T + 16683*(L-F)), BLAKE2b transcript with signed results, NAT traversal (STUN, NAT-PMP, UPnP, hole punch). Original, not Slippi-derived. | Platform | Netplay and rollback | L whole, M simplified |
| Fault injection tools | `tools/net_fault_proxy.py`, `online_pair.py`, `online_quad.py`, `p2p_pair.py` | Seeded UDP delay/jitter/loss/dup/reorder by packet position not clock. | Test | Net testing | S |

## 6. Menus, audio, input, launcher, training

### Menus and scenes (headline: nothing is driven by the host)
Neither engine has menu code in the host. Retail `mn*`, `gm*`, results, opening movie, fades and loading are the game's own
code (translated PPC in Static, decomp C in Source). The host only supplies hardware via the table in section 2.
`patches/melee-native.patch` hunks that touch scenes (all Source Port, logic-adjacent): `gmboot.c` (`mu_slippi_boot_mode`,
`mu_lag_test`), `gmmenumode.c` (Slippi submenu), `gmopeningmode.c` (`mu_title_demo_stage` swaps an oversized skin for the
fixed heap), `lbmthp.c` (THP opening movie: u32 frame table, big-endian size), `lbdvd.c` (preload cache pinning,
eviction after many matches), `lbcardserial.c` (new, 212 lines: GmSaveData/NameTag byte swap with `_Static_assert` sizes
0x1790 and 0x1F2C; bitfields need per-byte bit reversal because GCC orders them differently), `mncharsel.c`, `mnmain.c`.
For us this means: results/fades/opening/1P/Toy/Data/Settings stay OUR task (STATUS), and under the seam the shim set
`mu_vi/mu_dvd/mu_card/mu_pad/mu_audio/mu_os` is the shopping list of what the retail scene code needs from hardware.

### Audio
| Feature | Where | How | Logic | Closes | Effort |
|---|---|---|---|---|---|
| AX DSP HLE | `hle/ax_ucode.cpp` (840 lines) | Port of Dolphin DSPHLE AX.cpp/AXVoice.h for ucode 0x4e8a8b21; PB layout, accelerator, command lists, aux A/B, ITD (`MELEE_AUDIO_ITD=0`), needs free `dsp_coef.bin`. | Platform | cross-check of `melee_audio` | M; Dolphin-derived: not covered |
| AI DMA scheduler | `hle/audio_core.cpp` | Period = `dma_length*TB/128000` (5 ms per 160 stereo frames); block pushed at mix time not at DMA completion (saves a period); `awaiting_block` gives one block per call (re-sim after a stall replayed the same block every 5 ms = crackle); stall >20 periods skips ahead; missing block = one fade-out then silence (a repeat was heard as a buzz). | Platform | Underruns | S-M |
| Idle pacing | `source_host.cpp::h_vi_idle_step`, `shim/mu_vi.c VIWaitForRetrace` | Steps the AI clock one 5 ms block at a time during the retrace wait instead of 3 blocks after it. `MELEE_AUDIO_PACING=0` off. | Platform | Underruns | S |
| Faster sound effects | `axnative/AXOut.c`, `MU_AUDIO_OPTION_EARLY_KEYON` | User callback owed at end of frame k runs at the top of k+1 before voices sync: sound mixed 5 ms sooner. Offline only; the host returns 0 online/replay because it changes when the game sees a sound end. | Logic-adjacent | Latency | S |
| Output | `host/audio.cpp` (1488 lines), `audio_buffer_policy.h` | WASAPI shared/exclusive, ASIO (mode 3, GPL-3 SDK), WinMM; SPSC ring of 8192 frames @32 kHz; 48-tap Kaiser sinc resampler (512 phases, beta 6) replacing Catmull-Rom; fill steered by proportional + slow integral skew, cap 0.2% (a +-1.5% loop oscillated +-20 cents); Auto buffer +4 ms per gap <60 ms (max 120), -1 ms per 10 s clean, remembered per device in `audio-auto-v3.txt`; underrun fades to silence and cross-fades back; "Direct" mode writes each block as it arrives; follows online game speed. | Platform | Underruns, mixer | Free; algorithms M if we keep our own cpal path |
| Rollback-safe audio | `mu_online_audio.c`, `melee_game.snapexcl` | Per-frame ring of started sounds (`SFX_RING=16`) so re-executed frames adopt old voices and stop orphans; audio memory excluded from snapshots ("a console savestate leaves out the sound driver"). A stale `.snapexcl` crashed within seconds once (09-29). | Logic-adjacent | Rollback | M |
| Music | `hle/jukebox.cpp`, docs/MusicPacks.md | Songs start muted (`MuteMusic` hunk in `lbaudio_ax.c`); host decodes DSP-ADPCM HPS on a worker thread and mixes it, so sim hitches cannot glitch music; console gain hard-coded 0.700133 L / 0.702887 R (measured to 0.04 dB; Slippi's 0.8 was 1.16 dB too loud); packs by disc path, playlist folders `audio/a+b/`, non-HPS via Media Foundation, 10-min cap. Slippi Rust + hps_decode derived. | Platform | Music | M, clean-room |

### Input
| Feature | Where | How | Effort |
|---|---|---|---|
| GC adapter | `host/gc_adapter.cpp` (839 lines) | libusb + HID fallback (Wine/Proton, stock HID); Dolphin protocol (0x13 starts 37-byte stream on 0x81, rumble 0x11+4); endpoints read from descriptors (third-party adapters differ); reader thread at highest priority (normal priority dropped 1000 Hz to 500); rumble thread; scanner retries every 2 s; `PADRecalibrate` drops the stored origin (UCF shield drop). Dolphin-derived: not covered. Our own port is the C `gc_adapter.c` port (platform.md). | Free; nothing to port |
| Switch Pro / DS4 / DualSense / generic HID | `host/switch_pro.cpp` (needs 0x80 0x02, 0x80 0x04, subcommand 0x03 arg 0x30), `playstation_pad.cpp`, `hid_pad.cpp` (HidP descriptors, B0XX/Frame1/vJoy), `box_melee.h` (SOCD, from published values) | Per-family deadzones (Xbox none, as Dolphin), stick modifier percent, XInput empty-slot poll costs 5 ms (not asked on the game thread) | Free |
| PhobGCC | `host/phob.cpp`, `phobscope.cpp`, `gx/phob_menu.cpp`, `phob_stickmaps.h` | Firmware math in a 1 ms loop | GPL-3, PhobGCC: not covered; skip |
| Auto L-cancel / delay | `host/lcancel.cpp`, `offline_input_delay` | Injects an analog trigger EDGE upstream of the netplay send; skips frames where A or Z is newly pressed; 9-frame offline delay queue | Needs `lcancel_view` from the engine (S) |

### Memory card and disc
`hle_card.cpp` / `source_host.cpp card_*`: Dolphin GCI folder, 8 KiB blocks, slot A = 2043 blocks; write via temp file +
`MoveFileEx` with up to 20 retries on sharing violation (antivirus holds the file in about 1 of 10 saves); ASCII-safe
names (a non-ASCII path crashed saves); same-name save moved out of the folder; backup at mount. `hle_dvd.cpp`: async
reads on a worker, completion delivered at a fixed virtual time (request + 1/4 frame, in request order) so timing is
deterministic; the sim thread copies the bytes (a worker writing straight into RAM landed at a different moment each run
and even after `DVDCancelAsync`); the copy calls `mark_ram_write` so the texture snapshot cache invalidates (0.8.6 bug:
black track names). Closes: our card/disc loading gaps. Free under the seam.

### Training, Lab, Practice
Source Port only: `shim/mu_lab.c` (savestate = rollback snapshot at the top of an engine frame; D-pad right save, left
load, down record, up replay; collision bubbles via the game's own debug draw; dummy DI/tech/react), `mu_practice_cpu.c`
(model in `abi/mu_practice_cpu_impl.inl`), `mu_framedata.c`, `native_practice.cpp` (frame advance: L+R+Z freezes via
`gm_80479D58` pause bits, Z steps; matchmaking while training). `gx/lab_view.cpp`: Slippi-Lab-style silhouettes (Slippi
Lab assets MIT, `tools/build_lab_assets.py` -> `Lab\<id>.lab`, earcut), stage geometry read live from collision, hurtbox
capsules; fed by `MuLookSnapshot` on Source. Our lab task: export `mu_look_snapshot_v1` (S-M) and the `practice` bridge (M).
`mu_tmce*`, `mu_te*`, `mu_hp*`, `mu_20xx_ai.c`, ghost AI: third-party content, skip.

### Launcher, updater, packaging
`app/launcher.cpp` (plain Win32, owner-draw, no dependencies) with `.inl` pieces for lobby/mods/replays/crash;
`host/updater.cpp` (GitHub releases incl. pre-release, verifies the SHA-256 GitHub lists, in-process zip/gzip via stb
inflate because Wine has no tar, hands over to generated `update.bat`/POSIX script, keeps `User\`/saves/replays; a
past `\r\r\n` text-mode bug is noted); crash reporter (`launcher_crash.inl`: on consent, sanitized 8 MB zip to a
Cloudflare worker relay `tools/crash_relay`, privacy rules shared as JSON vectors across C++/Python/JS);
`tools/package_release.py`, `package_source_port.py` (greps the no-Slippi build for the word "Slippi"),
`package_linux_release.py`; Linux preview docs/linux-native-0.9.md (SDL2/GLEW/libusb, 120 FPS default, offline).
Not relevant: `native/` (a small D3D11 asset viewer with four tests).

## 7. Validation, tooling, snapshots, perf tricks
- **Checkpoints across render modes.** `tools/validate_native.py` runs one exe 5 times (`--headless`, `--hidden`,
  `--hidden --threaded-renderer`, plus authored and authored-interpolate at `--fps 240`) with `--volume 0 --fast --frames 2400
  --time-base 1 --script port/scripts/vs_match.txt` and an empty card dir; per retrace the Static trace writes
  `retrace,cpu,ram,aram,events` (FNV over every Context field, 24 MB RAM hash, 16 MB ARAM hash); Source uses `--state-digest`
  (`MuStatePod`: rng, scene, major, match_frame, 12 words x 6 slots), optional `MELEE_DIGEST_RAM=<N>`. It must call
  `hle::dvd_settle()` before hashing (async reads flaked one checkpoint under load). His own docstring: it "checks
  renderer isolation, not equivalence to Dolphin or complete rollback state". His fighter digest (`fighter_digest.py`, 15
  fields) excludes RNG calls, items, projectiles, stage state. Ours (NHV1 goldens) is stronger; take the MODE MATRIX
  idea: run our hash through headless / offscreen render / threaded present and require equality (S-M).
- **FMA exactness (highest relevance).** Two reference semantics: console/Slippi = double FMA then round to f32; his GCC build
  = AVX2 `vfmadd..ss` single rounding, `-ffp-contract=on`. Mitigations: `MU_NO_CONTRACT` on 133 functions where retail had no
  fused op (list computed by `tag_no_contract.py` comparing the recompiler's per-instruction C++ to FMA3 in the DLL via
  `addr2line`); `MU_P(x)` rounding a product alone (218 sites; sqrtf args because the console's inline sqrtf makes the arg a
  variable first; squares of memory-loaded operands were NOT fused by the console but squares of locals were, measured with
  `retail_ops.py`); explicit `MU_FMADDS` (double then f32); `sourceport/game/math/sdk_math.c` rewrites PSMTX/PSVEC with
  `-ffp-contract=off`, sqrt = frsqrte + 3 Newton steps with a 25-bit significand mask; Dolphin's fres/frsqrte tables
  (`estimates.c`; Dolphin-derived); MSL math compiled in (never host libm); `__sinit_trigf_c()` called explicitly because the DLL
  loader skips `.ctors` (otherwise sin/cos range-reduction constants are uninitialised). `gcc_fma_wrap.py` (rewrite FMA3 to
  double-then-single) is NOT wired into CMake (guess: abandoned; likely the origin of the residual 1-ULP spawn baseline).
  AVX2+FMA is mandatory: his SSE2 build matched action/percent/stocks/RNG but drifted up to 192 ULP in positions and "desyncs
  online". Our rule (clang `-ffp-contract=on`, `fmadds`/`fmsubs`) is the same idea; his tag lists in the patch are an
  independent census to diff against ours (M-L audit).
- **Parity harness.** `native_parity.cpp` + `parity_table.c` run each native routine against its translated twin on random
  inputs (mixed magnitudes, exact zeros, +-2, edge mode with denormals/inf/-0/NaN, aliasing cases, every array compared even
  if unwritten); covered: PSMTX/PSVEC, sinf, cosf, tanf, atan2f, acosf, expf, powf, lbVector_*, HSD_MtxSRT, MatToQuat, lbColl_*,
  mpLineIntersection, splGetHelmite. `extract_funcs.py` copies named functions out of the decomp so the library compiles
  exactly the game's code. Our `melee_gx` / `melee_platform` oracles do the same trick; add `edge mode` inputs (S).
- **Compiler traps (fixed real bugs).** `-fno-aggressive-loop-optimizations` (a table read one past the end on a loop's last
  pass was dropped; Peach's third down-special item), `-fno-strict-aliasing`, `-fwrapv`, `-mno-ms-bitfields` (console packs
  u8/u16 bitfields in one run), `-fno-builtin` (no compile-time sin/sqrt folding), `-fexec-charset=CP932`, `-fno-tree-sra`
  (GCC 14 SRA lost big-endian storage order in nested unions), narrow int to pointer casts (addresses have bit 31 set so an
  `s32` sign-extends: `MU_Z()` zero-extends, `tools/cast_census.py`), `tools/disc_offset_audit.py` (every field offset vs the
  decomp's `/* +XX */` comments; a missed 8-byte pointer once shifted Link's attributes from +0x94). The last two suggest a
  check for our `GcLayout` tables against decomp annotations (S).
- **Snapshots.** No pointer fixups: raw bytes at fixed addresses (MEM1 `0x80000000`, image `0x82800000`, DLL
  `--disable-dynamicbase`). Capture = MEM1 + image .data/.bss minus exclusions (ax_*, synth, axdriver, reverb, video, perf,
  pobj_bridge, gx_regs, gx_fifo, os interrupts/events, card, audio, replay_abi, lab, content, practice and the online
  bookkeeping). 4 KB chunks, shadow copy, `GetWriteWatch` dirty detection (memcmp elsewhere), per-checkpoint undo lists
  merged on drop, parallel copy, 7 slots (Slippi semantics: capture keeps, load restores and frees all). Diagnostics:
  `MELEE_ONLINE_WATCH_MEM1=1` hashes MEM1 outside the snapshot at 64 KB granularity to find missed state;
  `MELEE_SNAPSHOT_EXCLUDE`, `CMD_SELFTEST_KEEP/COMPARE (0xF0/0xF1)` report differing byte ranges after re-sim. The game stack
  is NOT saved: rollback happens at an engine frame boundary. Alarms stand in for the decrementer: `mu_alarms_hold/release`
  preserve the alarm list across a load so a fired pad alarm does not re-fire. If the scene ends during a re-sim,
  `mu_online_scene_loop_exit` force-ends rollback (the host skipped retrace waits and the game froze).
- **Headless flags/env.** `--headless` (no window, audio still opens, muted), `--hidden` (renderer on, window hidden), `--fast`
  (no pacing), `--frames N`, `--time-base`, `--rng-seed`, `--script`, `--volume 0`, `--no-music`, isolation dirs
  (`--card-dir --user-dir --shader-cache --replay-dir --settings-path`), `--capture*`, `--frame-times`, `--audio-dump`,
  `--state-trace`, `--state-digest`, `--match <spec>`, `--replay x.slp`, `--record-native`; automated runs set `no_gc_adapter`,
  disable matchmaking unless `--allow-matchmaking`, and refuse `MELEE_TEST_*` outside hidden/headless. `MELEE_SIM_TIMES=<csv>`
  per-frame wall/cpu/SIM_COST slots. Input scripts `port/scripts/*.txt`: `<frame> [p=N] [buttons] [sx= sy= cx= cy= l= r=]`
  with `@match` (frames relative to match frame 1), `@loop N`, `@scene major[:minor]`, `@matchrt`. Same idea as our
  `MELEE_PAD_SCRIPT`; the `@match` anchoring is worth copying (absolute-frame entries stop applying once the match starts).
- **Perf and timing tricks.** Pipelines on worker threads + fallback PSOs; prewarm recipes; async disc at virtual time;
  catch-up threshold 200 ms -> 34 ms (the sprint was fast-forwarded audio); `timeBeginPeriod(1)` (60 Hz instead of 48 on the
  15.6 ms tick); console clock advanced by emulated cycles/timebase not wall clock; `--time-base` presets the RTC; MEM1 is
  40 MB while the game believes 24 MB (`DOLPHIN_RAM_SIZE = 24 MB + 200000` so the arena top matches Slippi Dolphin); frame queue
  cap; A/B timing tools `benchmark_native.py` (caps 120..240, separate startup/match summaries, "CPU timings do not prove
  displayed FPS"), `perf_replay.py` (both engines alternating on the same `.slp`), `frame_scan.py`/`frame_triage.py`/
  `diff_captures.py` (burst captures; judge a frame against its neighbours, not an absolute black threshold),
  `audio_clicks.py` (LPC-residual click/cutoff detector), `snapshot_audio_ranges.py`, `crash_triage.py`.
  Do NOT use `tools/latency_probe/` (SendInput + Desktop Duplication): it needs OS input.
- **Tricks and bug causes worth knowing.** `__longjmp` translated as an ordinary function returned to the host caller
  instead of the `__setjmp` site (the game uses setjmp/longjmp to abort `HSD_ForeachAnim` walks), corrupting the OS alarm
  queue ~30 s into animated-background stages; native = `__builtin_longjmp`, game longjmps all use value 1. Wobbuffet's
  counter damage converts below zero as the console does (test B-281). Early-input peers lying >60% are ignored. The
  `lbdvd` preload cache must pin blocks (archives hold absolute pointers). Temporal history for DLSS was being recorded on
  drained frames (render only).

## 8. Third-party components (NOT covered by the author's permission)
From `NOTICE`, checked against `port/third_party` and file headers. "Can we port it?" assumes we want clean-room or permissive
only, until the user decides the licence question (his repo is GPL-3.0-or-later; ours is not stated in this survey).
| Component | Licence | Where | Verdict |
|---|---|---|---|
| ENet, Dear ImGui, stb, nlohmann json, dht, earcut (ISC), Vulkan-Headers, Monocypher | MIT / ISC / Apache-or-MIT / BSD-2-or-CC0 | `port/third_party/*` | Permissive, usable with notices |
| libusb | LGPL-2.1+ | `third_party/libusb` | Link-only; we use rusb/hidapi anyway |
| Dolphin (GX shader gen, AX emulation, fres/frsqrte tables, GC adapter protocol) | GPL-2.0+ | `gx_shader.cpp`, `ax_ucode.cpp`, `ppc_estimates.inc`, `sourceport/game/math/estimates.c`, `gc_adapter.cpp` | NOT covered. We already did our own shader gen from Aurora and our own AX; do not copy tables from him |
| Slippi Ishiiruka (EXI, online, playback, texture naming), SlippiLib, `port/slippi_sys*` | GPL-2.0+ | `exi_slippi.cpp`, `slippi_online.cpp`, `slippi_net.cpp`, `slippi_playback.cpp`, `texture_pack.cpp`, `third_party/slippilib` | NOT covered. Protocol knowledge only |
| slippi-ssbm-asm (Slippi game codes) and their native forms | GPL-3.0 | Static Recomp codes; Source Port `mu_online.c`, `mu_slippi_*.c`, `mu_replay.c`, `mu_gecko.c`, `mu_record_start.inc` | NOT covered, and these are the files our item 13 would want. Re-derive from the Slippi code list and our decomp |
| Slippi Rust extensions (jukebox, reporting), Slippi Launcher icons | GPL-2 text / GPL-3 | `jukebox.cpp`, `slippi_report.cpp` | NOT covered; the reporter also impersonates the Slippi client |
| hps_decode, Slippi Lab | MIT | `docs/licenses/` | Usable with notice |
| slippi-js landing lag table | LGPL-3.0+ | launcher | NOT covered |
| PhobGCC, Altimor stickmap, venus23gg stickmaps | GPL-3.0 | `phob.cpp`, `phobscope.cpp`, `phob_menu.cpp`, `phob_stickmaps.h` | NOT covered; skip |
| NVIDIA Streamline (MIT headers), NGX/DLSS (RTX SDKs licence), Reflex, Intel XeSS | MIT / proprietary | `third_party/streamline`, `ngx`, `xess` | NOT covered; host-only, no need |
| Steinberg ASIO SDK | GPL-3 option + BSD-style | `third_party/asio` | NOT covered; host-only |
| GD's Melee kit, Source Sans 3, Twemoji | GPL-2+ / OFL / CC BY 4.0 | `ui_sources/gd_melee`, launcher | Our own work (kit); OFL fonts fine |
| TM-CE (UnclePunch) | per `docs/third-party-training-mode-ce.txt` | `sourceport/game/tmce`, `mu_tmce*.c` | NOT covered |
| doldecomp/melee | GPL-2.0+ | `sourceport/extern/melee` (empty here) | The decomp; same as our reference |
| Arte's subframe rollback (idea and original implementation) | credited, licence unstated | `early_input.cpp`, docs/early-inputs.md | NOT covered; clean-room from the doc's description only |
Listed in NOTICE as not GPL and shipped separately: Streamline/DLSS/Reflex/XeSS runtimes, VC runtime.
**Not in NOTICE but third-party in origin (flag for review):** `port/third_party/dxc` (DirectX Shader Compiler, LLVM + MS
licences), `librashader` (MPL-2.0), `slang-shaders` (libretro CRT presets, mixed per-shader licences, provenance only),
`mod_tools` (xdelta3 GPL-2.0, 7zr LGPL/BSD, shipped as separate processes), `sourceport/extern/aurora` submodule (licence
not read), community mod content ported to native C with no per-file credit in the files opened (Akaneia `akaneia/`, ACE
fighters `ace/`, rogueMelee `rogue/`, Melee Maker `maker/`, 20XX TE/Hack Pack `shim/mu_te*.c, mu_hp*.c, mu_20xx_*.c`, the 2D
Arcade mod `mu_arcade.c`, community m-ex stages `mexstages/`, `mu_mex.c`), and `tools/ghost_ai/` (trained on named pros'
replays; provenance and consent unclear, guess). Also: no file header names a copyright holder (grep found "Copyright" only
in third_party); 357 files carry SPDX GPL-2.0-or-later, 15 GPL-3.0-or-later, 8 GPL-3.0-only.

## 9. Things we had not planned for
1. **The seam is a C ABI over 32-bit console addresses.** The renderer and mixer read game memory by address; a Rust engine
   with its own types needs an arena and a faithful FIFO. This is the real cost of "third engine" (item 3).
2. **Fixed image base 0x82800000**, because GX physical addresses are 26 bits and static game textures live in the image.
3. **Frame numbering for rollback**: pad frame N is consumed by the engine body whose unpaused counter equals N, measured
   4746/4746; checksum is XOR/sum over motion id, pos x/y, percent, spawnNum, stocks (Slippi wire compatible).
4. **Re-sim must refresh camera and bone matrices** (`camera_tasks`), or rollback lands "one rounding step away".
5. **Replays: UCF is decided by the replay's Gecko list, not by the file's toggles**, and needs RAW stick bytes in the pad
   queue; per-frame RNG reseed; the playback code list differs by Slippi version but only the cave address matters.
6. **Preframe 1-ULP baseline** at frames -118..-108 is their accepted residue; require exact from frame 0.
7. **Audio pacing is a design problem**: block pushed at mix time, one block per call after a stall, fade-out then silence on
   a missing block, clock skew capped at 0.2%, Auto buffer growth. These are the "underruns" answers.
8. **Disc and card determinism**: async reads with virtual completion time; atomic card writes with retries.
9. **Tearing**: vsync default ON; scanline-synced tear-free option when off.
10. **Fallback pipelines instead of skipped draws**, and prewarm recipes shipped in the release zip.
11. **True widescreen is a renderer-side projection scale** plus a list of exclusions; Slippi's Gecko widescreen is
    logic-adjacent (Fighter snapshot/restore around draws). Prefer the former for bit-identity.
12. **Hardware math audit tooling** (FMA census, parity table, disc offset audit, cast census) as a ready-made model for our
    own regression gates.
13. **Open questions for a spike**: reservation size of the image range; whether the host tolerates `region 0` pointing at an
    arena we barely use; what `native_pcm` audio mode requires of the DMA buffer contents; whether `state_regions` may report
    tiny image sections; the fate of `vi_set_next_framebuffer(xfb)` for an engine with no XFB memory.
