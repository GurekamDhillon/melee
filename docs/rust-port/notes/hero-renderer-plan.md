# Hero's renderer as the endgame: plan

Status 2026-11-08, branch `port/heroren` (off `pub/rust-port` 6a1fae236). Scoping only; no code moved.

Subject: Hero88go's melee-unlocked, read-only clone `scratch/ext/melee-unlocked` (upstream 0.9.86 271402e plus our local
`ours/rust-engine` commits; the core v1 header lives on the local branch `gd/core-v1` in `_build/agents/corev1/wt`). His
permission covers his code; nothing of it is copied here, and nothing was sent to him or built. Everything below about his
renderer was read from his sources, with file names given so it can be re-checked. Sizes: S under a day, M one to three days,
L a week or more, all agent-days.

Direction set by the user: deprecate our renderer (`melee_gx` on wgpu) as the endgame, contribute our renderer improvements
into his renderer, and run our Rust engine on top of his renderer, as part of making the Rust engine a third engine in his
launcher (notes/launcher-seam.md, notes/unlocked-survey.md, notes/core-v1.md).

Where we already stand (notes/core-v1.md): our engine builds as `melee_game.dll` (`rust/melee_corev1`, feature `corev1`)
against his frozen `mu_core_v1.h` boundary and runs inside his unmodified `melee_source.exe`, with his renderer drawing the
game: Battlefield VS at 59.9 fps, hashes identical to our own build, his audio dump byte-identical to our mix, his state test
with 0 mismatches. The renderer part of "run on his renderer" therefore exists as a working spike; what remains is the gap
between a spike and the thing we ship and test against, which is this plan.

## 1. What Hero's renderer actually is

### Language and layout

C++17, Windows first. `port/runtime/gx/` is about 41,500 lines across 80 files (`wc -l`), shared by both of his engines
(Static Recomp and Source Port) and by the Linux build. The large pieces:

| part | files | lines |
|---|---|---|
| GX command processor (FIFO decode, register state, frame recording) | `gx_core.{h,cpp}`, `gx_regs.h`, `texture_snapshot.h`, `gx_texture.cpp` | ~2,100 |
| shader generation (GX state to HLSL) | `gx_shader.{h,cpp}` | ~1,130 |
| D3D12 backend (the default, the only one with DLSS/XeSS/DXR) | `gx_d3d12.cpp` | 4,051 |
| D3D11 backend (mirror of D3D12 for old drivers; no DLSS) | `gx_d3d11.cpp` | 2,509 |
| Vulkan backend (Windows and Linux; HLSL to SPIR-V through the shipped `dxcompiler.dll`, glslang on Linux) | `gx_vulkan.cpp` | 1,733 |
| OpenGL 4.5 backend (Linux only) | `port/linux/renderer_gl.cpp`, `renderer_presenter.cpp` | 1,470 |
| threaded presenter and frame queue | `threaded_backend.cpp`, `frame_queue.h` | ~850 |
| sub-frame solver (frame interpolation) | `subframe.{h,cpp}`, `authored_pose.{h,cpp}`, `native_pose_bridge.cpp`, `temporal_history.h` | ~2,500 |
| upscalers and NVIDIA/Intel integration | `gx_streamline.{h,cpp}`, `gx_dlss5.{h,cpp}`, `gx_dlss5_scaling.{h,cpp}`, `gx_xess.{h,cpp}`, `gx_dxr_*.{h,cpp}` | ~2,900 |
| present pass, CRT model, RetroArch presets | `present_shader.h`, `crt_shader.h`, `user_shader.{h,cpp}` | ~830 |
| settings panel (ImGui) and visual options | `pc_settings.cpp` (10,177), `pc_settings_visuals.inl`, `render_options.h` | ~11,800 |
| looks, texture packs, video backdrops, overlays, Lab view, Practice menu, Phob menu | the rest | ~9,000 |

### The GX layer is not Aurora

Our `melee_gx` is a port of Aurora's GX (the C build's GX, WGSL byte-identical). His is a different lineage:

- `gx_core.cpp` is a **console command processor**: `write_fifo_bytes` takes the GameCube write-gather byte stream
  (big-endian BP/CP/XF register writes, vertex data, display-list calls) and decodes it into register state
  (`BPMemory`, XF matrices/lights, VAT/VCD). Each primitive becomes a `DrawCall` (`gx_core.h:113`): a ~5 KB snapshot of
  BP state, all 256 position / 96 normal / 256 post matrices, 8 light blocks, XF regs, TEV colours, 8 `TextureRef`s with a
  shared `TextureSnapshot`, plus a stable `identity` (display-list address + ordinals) used to pair draws across frames,
  `owner_player`, `skinned`, `frustum_backdrop`. Consecutive primitives of one display list that share state are kept as
  `DrawSegment`s inside one `DrawCall`. He notes "~1400 per simulation frame" and opts out of zero-initialisation for speed.
- Vertex arrays, textures, TLUTs and display lists are read **from game memory by physical address** (`host::ptr`,
  `host::try_ptr`, `gx_core.cpp:290-308, 452`), with a `TextureSnapshotCache` (`texture_snapshot.h`) that memcmp-compares a
  texture against the previous copy of the same source pointer and hashes only when it changed (kept 3 frames). TLUT loads go
  through a 1 MB software TMEM (`g_tmem`). Nothing in the renderer holds a guest pointer after record time.
- Shaders: `gx_shader.cpp` generates HLSL for each `VSUid`/`PSUid`, "transcribed from Dolphin's shader generators"
  (file header). Constant buffers are Dolphin's layout (`VSConstants`, `PSConstants`) with his additions (previous-pose
  matrices and `unjittered_projection` for DLSS motion vectors, `tint`, `mvscale`). The same HLSL feeds D3D11, D3D12 and
  Vulkan (DXC to SPIR-V) and, through glslang, OpenGL.
- A `Frame` (`gx_core.h:199`) is the unit handed to a backend: vertices (108-byte `Vertex`), segments, draws, `EfbCopy`s,
  an ordered `commands` list interleaving them, HUD snapshots, scene ids, host time, `TickTiming`, and the rollback flags
  `discontinuous` / `hard_cut`. The frame ends at the **XFB copy** (`EfbCopy::to_xfb`): `submit_and_recycle` pushes it.
- `sourceport/extern/aurora` exists as a path but is empty in the clone and nothing in `port/` references it. His Source
  Port does not use Aurora; it feeds the same decoder the Static Recomp feeds.

### Feature list (verified in code; where "offered" means a RenderOptions field with a code path)

| feature | where | status in his code |
|---|---|---|
| Backends: D3D12 (default), D3D11 (`--backend d3d11`), Vulkan (Windows + Linux), OpenGL (Linux) | `gx_backend.cpp`, `render_options.h` `RenderApi`, `port/linux/` | all four shipping; D3D12 falls back to D3D11 when `d3d12_usable()` fails |
| Internal resolution 1x-8x (`efb_scale`, 0 = auto integer scale covering the window), MSAA 1/2/4/8 (EFB multisampled, resolved before every read), SSAA 1-4 per side (box filter in present), anisotropy 1-16 | `render_options.h:321,360-362`, `gx_d3d12.cpp` | yes |
| DLSS Super Resolution / DLAA (5 quality modes), NVIDIA Streamline, run-time `sl.interposer.dll`, optional at build (`GX_STREAMLINE`) | `gx_streamline.h` | D3D12 only; degrades to native when the DLL/GPU is absent |
| DLSS Frame Generation 2x-6x fixed or Dynamic; NVIDIA Reflex (low latency, boost, PC-latency markers, latency breakdown) | `gx_streamline.h:204-241`, `Backend::frame_generation_multiplier`, `reflex_pacing` | D3D12 only; FG needs the upscaler running; plugin loaded only when enabled at start |
| DLSS 5 "neural rendering" (NGX `nvngx_dlssnr.dll`, Model A/B/C, tuning presets, resolution scale) | `gx_dlss5.{h,cpp}`, `gx_dlss5_scaling.*` | D3D12, build flag `GX_DLSS5`; EXPERIMENTAL in his header |
| Intel XeSS SR 3.0 (any SM 6.4 GPU) | `gx_xess.{h,cpp}` | D3D12; loaded from `libxess.dll` at run time |
| DXR path tracing + DLSS Ray Reconstruction | `gx_dxr_path_tracer.*`, `gx_dxr_scene.*` | present but `kPathTracingAvailable = false` (hidden, forced off: rebuilds the stage BVH every frame, "a few fps") |
| AMD FSR | | **not present** (no FSR/FidelityFX reference anywhere in `port/`) |
| Present pass: box downsample, contrast-adaptive sharpening, DLSS HUD composite (2D draws from the current frame), screen-space AO (depth-based, display only), brightness/contrast/vibrance, border art with soft shadow | `present_shader.h` | yes, shared by D3D12 and Vulkan; D3D11 shares the blit/sharpen shader |
| CRT display model (studio monitor / home TV, curve), modelled from the 640-column raster | `crt_shader.h` | yes |
| RetroArch `.slangp` shader presets via librashader (run-time DLL) | `user_shader.{h,cpp}` | D3D12, D3D11, OpenGL entry points |
| Sub-frame animation: `Off / Extrapolate / Interpolate / Authored / AuthoredInterpolate` | `subframe.*`, `authored_pose.*`, `threaded_backend.cpp` | yes; `AuthoredInterpolate` is the kept default (`LowSpecPrevious`); matrix pairing by `identity`; authored FObj track re-sampling via `MuNativePoseSnapshot`; menu mode; rollback discontinuity handling; endpoint / phase-flip audits |
| Frame cap: fixed, monitor (-1), uncapped (0, held at 2x refresh unless `MELEE_PRESENT_UNCAPPED=1`); vsync default on; VRR detection from DXGI frame statistics; G-Sync cap `hz*(1-hz/3600)`; scanline sync (`MELEE_SCANLINE_SYNC`); exclusive fullscreen picks a refresh that is a whole multiple of the cap | `threaded_backend.cpp`, `vrr_detect.h`, `vblank_model.h`, `vblank_clock.h`, `dxgi_paced_mode.h` | yes |
| Aspect: Auto (73:60, or 16:9 with Slippi widescreen), Native 73:60, 4:3, 16:9, Stretch; **True widescreen** (renderer-side projection widening, 16:9 / 16:10 / custom / fit window, clamped 4:3..64:27) with exclusions | `render_options.h` `AspectMode`, `TrueAspect`, `presented_aspect`; `gx_shader.cpp build_projection`; `DrawCall::frustum_backdrop` | yes; mutually exclusive with the Slippi Gecko widescreen |
| Visual effects levels (0-3), Lightweight ladder 0-5 (low poly, no backdrops, flat, lowest image, wireframe), Looks (retro 64, flat, white, posterised, wire; port colours; backgrounds), Arcade look (hitboxes, 2D camera, combo counter, grid), shield colours, player tints | `gx_core.h:382-620`, `gx_retro.h`, `render_options.h` | yes, display-only filters on the recorded frame |
| Lab view (Slippi-Lab-style silhouettes over or instead of the scene), training/net/replay overlays, input overlay, Practice menu, settings panel | `lab_view.*`, `*_overlay.*`, `replay_hud.cpp`, `practice_menu.*`, `pc_settings.cpp` | yes (ImGui) |
| Custom texture packs (Dolphin names/hashes), texture dump, prefetch; MP4 video backdrops on CSS/SSS | `texture_pack.*`, `video_background.*` | yes |
| EFB copies: to texture (kept on the GPU, keyed by guest destination address, half-scale, depth formats, alpha forced to 1 when the EFB format has none) and to XFB (ends the frame) | `EfbCopy`, `gx_d3d12.cpp execute_copy` | yes; never written back to RAM |
| Pipeline caching: PSOs compiled on worker threads, a generic fallback PSO while one compiles, `ID3D12PipelineLibrary` on disk, a shipped `recipes.bin` prewarm list | `gx_d3d12.cpp` (`prewarm`, `fallback`, pipeline library) | yes |
| Captures: `--capture <ppm>` at a presented frame, `--capture-every`, `--capture-burst`, `--capture-sim-frame`, F2 blink captures, flicker scan | `render_options.h:363-367,435`, `flicker_scan.*` | back-buffer (presented, letterboxed, scaled) PPMs; Linux has `capture_efb` |
| Display latency test (black/white fill on X+D-down for photodiode testers) | `gx_core.h:252` | yes |

### How his Source Port feeds it

1. The game library is compiled with the console's GX API (`sourceport/game/gxnative/GXFifo_native.c`): `GXW_*` encoders
   append command bytes to a 64 KB staging buffer; `mu_wg_flush` hands them to the host through the host table
   (`MuHostApi.gx_fifo`, `source_host.cpp:1000 h_gx_fifo` -> `gx::write_fifo_bytes`). Same bytes a console would see.
2. The game library is linked at **image base 0x82800000** and the host maps MEM1 (40 MB) at **0x80000000**, so every
   pointer the GX registers carry (26-bit physical addresses, 64 MiB window; the Linux layout script asserts `.bss` ends
   under 0x84000000) is a real address in the process, and the decoder reads vertex arrays, textures and display lists
   straight out of game memory (`main_source.cpp:25-32`, `sourceport/game/CMakeLists.txt:475-479`).
3. Display-only data goes back and forth through the host table rather than the FIFO: `hud_player` / `hud_combo` (the
   HUD snapshot in `Frame`), `native_pose_snapshot` + `native_render_scope_event` (authored poses for sub-frames; the game is
   patched to publish JObj/FObj data, `abi/mu_native_pose.h`), FIFO tokens `0xF0/0xF1` for draw owners, and `display_options`
   / `hud_scales` / `picture_widen` / `shield_color` read by the game each draw (`source_host.cpp:1016-1105`).
4. `h_vi_set_next_framebuffer` is a no-op: "presentation follows the EFB copy". The game's `GXCopyDisp` is the frame end.

### Frame pacing and present model

- **Sim thread** (the game thread, inside `gx_fifo`): decode, record, snapshot textures. At the XFB copy the `Frame` is
  moved into a `FrameQueue` (`frame_queue.h`): bounded at 32 frames or 256 MB, **never drops a frame** (every frame's EFB
  copies feed later ones), hands back a recycled, pre-reserved buffer so the sim thread never reallocates. Backlogs are
  drained by the renderer with `set_skip_present(true)` (copies and clears run, no blit).
- **Render thread** (`ThreadedBackend::present_loop`, `--threaded-renderer`; without it the backend presents on the sim
  thread at the XFB copy): holds the previous and current `Frame`, runs the `SubFrameSolver` for the phase `t` of each
  present, and calls `submit_frame(frame, overrides)` which **re-executes the whole frame's command list** with per-draw
  matrix overrides (and replacement vertex streams for effects), including its EFB copies, then `present_efb` at the XFB
  command. Phase comes from the tick's schedule (`current.tick.target`), not hand-over time (measured 2026-10-23: steps
  over 2 ms per 40 s fell from 262 to 15). A stale in-between present is discarded when a newer game frame arrived while
  its re-pose was being built. Options: present-on-hand-over on VRR displays, frame-locked grid, keyframe modes, pin-phase for
  deterministic development runs, a frame-time CSV, a latency trace, pacing spike logs.
- Cap and sync: `fps_cap` grid; vsync on by default; VRR verdict from DXGI `PresentCount`/`SyncRefreshCount` with
  thresholds 75 Hz / 30 fps / 0.96 / 0.98 / 0.04 (`vrr::judge`; ours in `main_loop.rs judge_vrr` are the same);
  `vblank_next` scanline sync; Reflex sleep when Reflex paces; frame-generation multiplier divides the render rate; exclusive
  fullscreen re-modes the output to a refresh the cap divides.
- Rollback: the host calls `gx::mark_discontinuity(rollback)` when a state is loaded (`source_host.cpp:1346, 1381, 2686`);
  the next frame carries `discontinuous` (fighters may still be smoothed) or `hard_cut` (nothing carried); frames produced
  during re-simulation are not drawn by the game (`g_resim`).

## 2. Inventory of our melee_gx / platform-loop improvements, mapped onto his renderer

Sources: notes/perf.md, notes/render.md, notes/display.md, `git log -- rust/melee_gx rust/melee_platform/src/{gfx,main_loop}.rs`,
and today's `port/perfspike` commit 5813a8554. "His" refers to the files above. Sizes are for porting into his C++.

| # | ours (commit) | what it does | does his have it | would it port | size |
|---|---|---|---|---|---|
| 1 | Aurora-exact GX port with oracles (ea458449b, 59e97c611): WGSL byte-identical to Aurora, texture decode / state / shim oracles, 0-diff render test vs Aurora's screenshot | correctness reference | no: his generator is Dolphin-transcribed HLSL, a different pipeline. Pixel equality between the two is not expected | the code does not port; the **method** does (a GX-API-level golden image set with a fixed scene, see section 4) | M for a test harness in his tree |
| 2 | GX decode on its own thread (30f171eff): the FIFO and handle table go to a decode thread, the tick keeps only recording | tick p50 5.5 -> 2.0 ms on Temple 4P | no: his decode and texture snapshot run inside `gx_fifo` on the game thread. Inside his host our tick measured 2.60 ms of which his decode was **1.47 ms** (notes/core-v1.md) | partly. His decoder reads live game memory, so an async decoder needs the bytes it will read to be stable until it runs (the Source Port rewrites textures in place; he documents a one-frame-stale bug from that). For our engine the transcoder already copies everything into a MEM1 arena, so a double-buffered arena plus calling `gx_fifo` from our decode thread is possible without his code changing; for his own game it is a larger change | M on our side (arena fencing), L on his |
| 3 | prepare / draw_prepared split, persistent GPU buffers, uniform dedupe, state-change elision (cf565dd1e): render CPU 11 -> 1 ms per frame | per-present cost | mostly yes: `FrameGeometry geometry_[4]` reuses a sim frame's buffers across its sub-frame presents, constants are built in CPU memory and uploaded once, `vs_constants_bytes` trims unread matrices, `cached_pipeline` per draw | nothing large left; his per-present cost is a full re-record of ~1400 draws, as ours is | S (measure first) |
| 4 | draw batching across commands (655cc1328): identical-state consecutive draws appended into one call, 2520 -> 1255 calls on Temple | fewer draw calls | partial: `DrawSegment`s merge consecutive primitives *within one display list*; cross-command merging of equal-state draws is not done | yes, at `record_draw`/`append_segment_indices` level (compare BP/XF/textures with the previous DrawCall; respect the sub-frame pairing identity, which would have to stay per-source-draw) | M |
| 5 | pipelines created on the decode thread / `PipeFactory::warm` (655cc1328) | no first-use stalls | yes and more: worker-thread PSO compile, fallback PSO instead of a skipped draw, pipeline library, shipped `recipes.bin` | nothing to port | - |
| 6 | content-keyed decoded-texture cache, 256 MB budget, 120-frame expiry, address->key alias (547866504, 3729f4d84, 8ec7e5c2a) | no re-uploads when the same image lands in a new buffer; no VRAM growth | partly: the CPU side (`TextureSnapshotCache`) is content-hashed with a 3-frame keep; the GPU-texture cache in the backends is keyed by snapshot and we did not find a byte budget or an expiry audit | check his GPU-side eviction; if unbounded, port the budget | S |
| 7 | texture bind-group sharing between unmerged draws with equal textures (5813a8554, 72% of Temple draws) | fewer binds | n/a: D3D12 descriptor tables already make this a pointer write; D3D11 has a small SRV state cache (header of `gx_d3d11.cpp`) | no | - |
| 8 | **present split** (5813a8554): below a 2.5 ms present interval the adopting present only prepares the new frame and re-presents the previous EFB; the next present draws it. p99 3.89 -> 3.50 ms FD, 7.20 -> 4.72 ms Temple | frame-interval tail | not as such. His render thread already takes the whole frame off the sim thread, but each present still records every draw, so a fresh frame's upload + full pass lands in one interval exactly as ours did | yes, in `present_loop`: on a new frame, upload geometry and re-present the old EFB at its newest pose, draw on the next slot; measure with his `--frame-times` | M |
| 9 | free retired frames off the present path (e1b3abf79) | adopt 1.4 -> 0.15 ms | yes: `FrameQueue::recycle` reuses buffers, nothing is freed | no | - |
| 10 | EFB copies keep a sim frame's exact pose (eeabe05ed): in-between presents skip the copies; a screenshot redraws the previous sim frame exactly first | Pokemon Stadium screen no longer phase-dependent; shots deterministic | **no**: `D3D12Backend::submit_frame(frame, overrides)` executes `execute_copy` for every non-XFB copy on every sub-frame present (`gx_d3d12.cpp:3645-3649`), so a copy sampled by a later draw (Stadium's screen, FoD's reflection) is taken at the re-posed phase | yes: skip non-XFB copies when `overrides` is set and `t != 1`, or run the copies once at phase 1 per sim frame. Small, self-contained, a visible correctness fix | S |
| 11 | deterministic screenshot tick (5813a8554): no tick runs while a shot is pending | shot = the tick asked for | different model: his `--capture-sim-frame N` counts XFB copies and captures the presented back buffer; a capture is never of an EFB and the sim never waits | relevant to section 4 (EFB-exact capture), not a port | - |
| 12 | render-only frame interpolation (f879201a6, `interp.rs`, `pose_capture.rs`): authored FObj channels replayed with a bit-exact phase-0 check, camera slerp + lerped projection, cut detection, ortho and HUD cameras held, menus interpolate | sub-frames | yes, and his is the larger implementation (envelopes, stage lock, menu mode, discontinuities, endpoint audit, phase-flip audit). His authored check uses tolerance 0.002 where ours requires bit-exact phase 0 | our "bit-exact at phase 0 or the channel is not authored" rule is a candidate tightening; otherwise nothing | S |
| 13 | pacing (f879201a6): fixed grid, phase from the tick schedule, stale drop, VRR detect, G-Sync cap | pacing | yes, same formulas (his are the original; ours were written to the same thresholds) | no | - |
| 14 | renderer-side true widescreen with exclusions (f879201a6): RTT viewports, authored 4:3 flashes, backdrop planes (decoded vertices, up to 64), corner-pinned layers, nametag ortho layer widened | 16:9 without touching the sim | yes: `build_projection` + `frustum_backdrop` + `is_authored_fullscreen`, nametag layer re-spaced | cross-check the two exclusion lists against each other on the stage set; each list probably has a case the other lacks | S |
| 15 | MSAA resolve before reads, depth copies from sample 0 (91b4c8071, 5fae7d882) | | yes | no | - |
| 16 | GPU fault handling (8ec7e5c2a): uncaptured-error handler + device-lost callback, OOM evicts and retries | clean stop, no panic | yes (`disable_after_resource_error`, device fault paths) | no | - |
| 17 | rollback-safe renderer state: no statics the snapshot restores; `Detacher` keyed by content hash (8ec7e5c2a, notes/net.md) | | not applicable to his memory model (host-side raw snapshots exclude the renderer) | no; the transcoder already follows the rule (`ROLLBACK` mode, system heap) | - |
| 18 | UI batch renderer + SDF fonts (6a0d9e95f, e0ddfd0f6), settings overlay, HUD overlay, wasm presentation mods (`ui.rs`, `ui_font.rs`, melee_modhost) | our presentation layer | he has ImGui panels and overlays; ours is a different system | not a port into his renderer. Under his host these have no surface today (section 3, "overlays") | decision |
| 19 | texture overrides for cosmetics (`tex_override.rs`, d9b272fd4) | skins | yes, differently: Dolphin-named packs and cosmetic companions | our cosmetics would be applied before the FIFO (bytes in MEM1) or exported in his pack naming | S-M |
| 20 | in-process sampling profiler (`prof.rs`), `MELEE_FRAME_TIMES`, `MELEE_PRESENT_SPIKE`, `MELEE_GX_PROF` | diagnostics | yes: `--frame-times`, pacing spike log, latency trace, `profile_line`, Reflex breakdown | no | - |
| 21 | `corev1::Transcoder` (d9a8b2095, 1923f3677): Aurora FIFO -> console FIFO, MEM1 arena, EFB copy pool, four encoder fixes, Geno palette to console matrices | the seam itself | n/a | stays ours (section 3) | - |
| 22 | perf.md sim-side work (snapshots, jobj, boot) | not renderer | n/a | n/a | - |

Perf context (notes/bench-live.md, 2026-11-02, same machine, both windows hidden/unfocused): CPU per rendered frame ours
0.99 ms vs his 0.95 ms (FD 2P), 2.8 vs 3.0 ms (Temple 4P); his 1% low was better (152 vs 54 fps on Temple) before our
perfspike work; his sim tick 2.2-4.4 ms vs our 4.1-7.5 ms *with our GX recording in the tick* (headless we are 0.3-0.5 ms).
Inside his host our tick is 2.6 ms of which 1.47 ms is his decoder, so item 2 is where the shared win is.

Contribution candidates, in order of value for cost: **10** (EFB copies at phase; S, correctness), **8** (present split; M),
**4** (cross-command merge; M), **6** (GPU texture budget; S after checking), **14** and **12** (cross-checks; S), **2** (his
decoder off the game thread; L, the biggest number).

## 3. The seam: our Rust engine driving his renderer

**FFI direction.** His renderer is internal C++ (`gx::init(Backend*)`, `gx::write_fifo_bytes`), not a library with a C
export; the only stable boundary he offers is the game side: a DLL loaded by `melee_source.exe`. So the direction is
**our engine as the game library** (`melee_game.dll`, `mu_game_entry_core_v1`, image base 0x82800000), his process owning the
window, swapchain, settings and presenter. This is what core v1/v1.1 already does. The reverse (his renderer as a DLL inside
`melee.exe`) would need a new C ABI around `gx::Backend` plus his window/settings code, and gains nothing for the launcher
goal; not proposed.

**What we emit, what he consumes.** The game->host call is `gx_fifo(bytes)`: the console's write-gather stream. Our
encoders write Aurora's dialect (`GX_AURORA` subcommands, data by handle, sized/indexed draws), so `melee_gx::corev1::Transcoder`
rewrites each drained buffer (notes/core-v1.md): every handle's bytes are copied once into a MEM1 bump arena (cached by shared
buffer; 238 MB staged and 8,702 texture copies over 3,400 ticks; arena wrap mid-frame handled by a draw-done + re-bind),
registers get console addresses, `GXCopyTex` becomes the EFB copy registers into a 4 MB MEM1 pool, `GXCopyDisp` becomes the
XFB copy that ends his frame, PC-only subcommands (viewport/scissor/projection overrides, polygon offset, pose tags) are
dropped. Measured transcode 0.125 ms per tick. Display lists: his decoder reads them by address too; ours are recorded by the
shim and sent inline, which already works. Not yet exercised: `DRAW_SIZED`/`DRAW_INDEXED` paths, depth-format EFB copies
(encoded, unverified), the Geno palette conversion beyond a unit test.

Alternative worth asking him for later (user decides when): a core v1.2 host call that **maps a region we already own** into
the GX window (or a handle-resolving subcommand), removing the 238 MB of copies. Until then the arena is correct and cheap.

**Threading.** His decoder runs on whichever thread calls `gx_fifo`; the full frame is decoded within the call that carries
the XFB copy, and he presents on his render thread (`--threaded-renderer`, which the launcher always passes). Our async GX
decode thread is pointless under his host (our decode is replaced by his), so the 1.47 ms of his decode sits in our tick. Two
ways out, both ours: (a) call `gx_fifo` from our existing decode thread with a double-buffered MEM1 arena (his rule: the
bytes of a frame may be reused once the XFB-copy call returns), (b) item 2 on his side. (a) is M and needs nothing from him;
pads/retrace stay on the game thread since `vi_wait_retrace` is where his host calls `state_*`.

**Interpolation (sub-frames).** Tier 1 (matrix pairing by `identity`) is free and already running (his `SubFrameMode`
applies to our frames). Tier 2 (authored poses, his default `AuthoredInterpolate`) needs `native_pose_snapshot` /
`native_render_scope_event`, which exist in the full `MuHostApi` but **not in core v1**. Proposal for a v1.2 append: those two
host calls plus the `0xF0/0xF1` owner tokens in the FIFO; our `pose_capture.rs` already has the joints (SRT, parent, world),
the FObj channels and the camera per tagged draw, so filling `MuNativePoseSnapshot` is a translation (M). Until then his
solver falls back to matrix blending for our draws (the mode he calls Interpolate/Extrapolate), which is our tier 1 today.

**Rollback re-pose.** His solver must be told about a state load so it does not blend across it (`gx::mark_discontinuity`);
in his Source Port the host knows because it performs the load. Under core v1.1 the host calls our `state_load`, so the host
side can mark it itself (one line on his side, S), and when his Slippi rollback drives our `state_*` the same hook fires. Our
re-simulated frames already send no draws and no copies (`set_suppress_draws`, 1923f3677). Our own netplay (`melee_net`,
`play_net::Detacher`) is reachable only from our exe today; under his launcher the online path is his (Slippi/mu_net), which
is outside this plan.

**Screenshots for our tests.** His captures are PPMs of the presented back buffer (letterboxed to 73:60, scaled, after the
present pass), counted by XFB copies (`--capture-sim-frame`), never of the EFB. Our shot tooling (`melee play --shots /
--shot-ages / --shot-ticks`, `view-golden`, `MELEE_RETAIL_SHOT_TICKS`, `shotcmp*.py`) reads the 640x480 EFB at an exact tick.
`compare_shots.py` (tools/rust-port/corev1) registers his capture to ours and reports mean abs 2.1-11.6/255, PSNR 18-32 dB,
dominated by his 73:60 presentation and rescaling, not by content. For a gate we need **EFB-exact capture in his host**: the
Linux renderer has `capture_efb`; Windows does not. Proposal: a `--capture-efb <path>` on his D3D12/D3D11 backends (dump the
resolved EFB at the XFB copy of sim frame N; S-M on his side, local branch first) and, for the game library, a core v1.2 host
call `efb_readback(buffer)` so our `--shots` can keep its contract under his host.

**Overlays, UI, mods.** Our settings overlay, HUD overlay, LAB and the wasm presentation mods draw with `melee_gx::ui` after the
EFB blit; under his host they have no surface (notes/core-v1.md "Gaps"). Options: (1) emit them as GX draws on an ortho 2D
layer at the end of our frame, so his renderer draws them, his captures include them and they survive his present pass (M;
his DLSS HUD composite treats flat 2D draws as UI, which is what we want); (2) ImGui hooks on his side (C++ in his tree);
(3) nothing. (1) is the only one that keeps our UI ours and renderer-independent. Decision for the user (section 7).

**Linux.** His Source Port runs on Linux (SDL window, OpenGL or Vulkan backend, the ELF game library at a console-compatible
base through `game-linux-layout.ld`). Our cdylib would need the same base on Linux (`-Wl,-Ttext-segment=0x82800000`) and his
sandbox limits do not apply there. Not started; S-M once Windows is gated.

## 4. What must stay, and when melee_gx may go

**What depends on our renderer today.**
- `melee_gx` tests: `tests/render.rs` (0-diff against Aurora's own screenshot), `tests/oracle.rs` (texture decode, WGSL text,
  GX state, 9,000 shim cases), the Geno palette unit test. These are the only bit-for-bit renderer proofs we have.
- Render proof vs the C build (notes/render.md): `view-golden` shots compared with `shotcmp3.py`/`shotcmp_dir.py` at mean abs
  1.1-3.4/255 per frame, 88-99% within 16. This is our definition of "looks like the C build".
- Scenario scripts that take and compare shots: `melee play --shots/--shot-ages/--shot-ticks`, `MELEE_RETAIL_SHOT_TICKS`,
  `tools/rust-port/play/full_loop.txt`, `tools/rust-port/menus/gen_lua.py`, `tools/rust-port/netplay/gen_css_scripts.py`,
  the Stadium determinism check (notes/perf.md), transition captures, the gd-ui and modhost UI tests, `bench-live`.
- The oracle tooling is sim-side (goldens, synctest, Dolphin oracle, `melee diff`) and does not depend on the renderer, but
  every scripted visual check above does.
- The gate (`tools/rust-port/gate.sh`) has no renderer step; the renderer is proven by the items above, run by hand.

**Reference path.** Keep both builds from the same sources, as now: `melee.exe` (wgpu, our presenter) and `melee_game.dll`
(core v1, his renderer). `melee.exe` stays the reference for every shot-based check until his path passes them. The
transcoder is the only renderer-side code that is core-v1-only, so nothing forks.

**Criteria for deleting melee_gx (render / shader / interp / ui / texture; the GX API, shim, state decode, resources and the
transcoder stay, since the transcoder encodes against them).** All of:
1. EFB-exact capture exists in his host (section 3) and a shot gate `tools/rust-port/corev1/gate_shots.sh` runs the
   view-golden set (FD, BF, YS, GG, FoD, DL64) plus the `play --shot-ages` scene set through both renderers at the same ticks,
   with the same bar we hold against the C build: mean abs <= 2.5/255 and >= 97% of pixels within 16, per shot, and the
   Stadium ages 300-1100 identical run to run.
2. Every scripted shot consumer above has a core-v1 variant and passes.
3. Our overlays, LAB and wasm presentation mods render through his renderer (section 3, option 1 or a decided alternative).
4. Sub-frame tier 2 under his host is at least as good as our interp (his audits: endpoint check, phase flips, insane holds).
5. Frame-interval p99 and 1% low under his host are no worse than `melee.exe` on the bench-live scenarios.
6. Linux runs (his GL or Vulkan) for the same gate, or Linux is explicitly dropped by the user.
7. The melee_gx oracle data (`tests/data/*.txt`, `render_aurora.png`) is kept in a small test crate or archived with a note,
   so the Aurora-exactness proof is not lost.
Until then melee_gx is "frozen reference": bug fixes that the gate needs, no features.

## 5. Licence

Facts, read from the files:
- His repo: `LICENSE` is the GPL-3.0 text; `NOTICE` says releases are **GPL-3.0-or-later since 0.8.5** (GPL-2.0-or-later
  up to 0.8.1), that files whose header names another project keep that notice, and grants a GPL-3 section 7 additional
  permission for linking with the NVIDIA Streamline / NGX / Reflex and Intel XeSS SDKs (his own code only). Per-file SPDX
  counts under `port/` and `sourceport/game`: 379 `GPL-2.0-or-later` (this is most of the renderer: `gx_core.h`,
  `gx_shader.h`, `render_options.h`, `subframe.h`, `threaded_backend.cpp`, the backends, the Linux GL renderer), 16
  `GPL-3.0-or-later`, 8 `GPL-3.0-only` (all PhobGCC: `phob*.{h,cpp}`, `phob_menu.*`, `phob_stickmaps.h`), 29 Apache-2.0, 3 MIT,
  4 BSD-2/CC0. `gx_shader.cpp` is Dolphin-derived (GPL-2.0-or-later, not his to relicense); `texture_pack.cpp` is Ishiiruka-
  derived. Third-party list in `NOTICE` (ENet, ImGui, stb, nlohmann, Vulkan-Headers MIT/ISC/Apache; libusb LGPL-2.1+;
  SlippiLib GPL-2.0+; Streamline headers MIT; NGX and XeSS under their vendor licences, shipped separately).
- Ours: `README.md` "Licence and legal": `pc/` is GPL-2.0-or-later (`pc/LICENSE`); Geno and the interop specs are MIT
  (`docs/specs/LICENSE-MIT`, `pc/geno/LICENSE-MIT`); the decomp itself has no licence of ours. The Rust workspace
  (`rust/Cargo.toml`) says `license = "CC0-1.0"` on every crate, from the skeleton commit b551c8dc2, and there is no `LICENSE`
  file under `rust/`. STATUS.md records the user's 2026-11-02 decision that implementing Slippi codes from slippi-ssbm-asm
  (GPL-3.0) "makes the combined work GPL-3, which the user accepts". `mu_core_v1.h` is his header (GPL); our `corev1.rs` was
  written from the header alone and carries his `#[repr(C)]` layout and offsets.

What linking means (not legal advice; the conservative reading of the GPL FAQ, for the user to weigh):
- **Running our DLL inside his exe** (the current direction) is one program at run time: both sides call each other through
  function tables and share MEM1. Distributed together (the launcher bundle with `rust/melee.exe` or `melee_game.dll`), the
  conservative view is that the bundle is a combined work under GPL-3.0-or-later, so our library must be under GPL-3-compatible
  terms. GPL-2.0-or-later is compatible (the "or later" lets it be used under 3); MIT is compatible; CC0-1.0 is compatible but
  the FSF discourages it for software (patent clause), and it is almost certainly not what the user meant. **Our source files
  do not become GPL-3**: the combination does; we can keep publishing the Rust port under its own compatible licence.
- **Contributing our improvements into his renderer** (section 2): the code we write in his tree is given under his file's
  terms (GPL-2.0-or-later for the renderer files, GPL-3.0-or-later for new files) and his NOTICE's section 7 permission is his
  to extend to it. It does not change our repo's licence, since the original Rust is ours and we may offer a C++ rewrite under
  any terms.
- **Copying his renderer code into our repo** (not proposed; the task forbids it here): the copied parts stay GPL-2.0-or-later
  / GPL-3.0-or-later and the Rust build that contains them is bound by that; it would end the possibility of a non-GPL Rust
  engine.
- **What stays MIT regardless**: `docs/specs`, `pc/geno` and anything else under a `LICENSE-MIT`. MIT material inside a GPL
  combined work keeps its MIT licence; only the combination's terms are GPL. The ABI headers are the sensitive spot: if
  `mu_core_v1.h` stays GPL, an engine that implements it is arguably derived from it (interfaces are a grey area; the
  conservative view says a header with layout and semantics is covered). Asking him to put `mu_core_v1.h` (and any renderer
  C header we add) under MIT or dual MIT/GPL would make the boundary licence-neutral.

Decisions for the user (none taken here):
- D1. State the Rust port's licence explicitly (replace `CC0-1.0`; candidates GPL-2.0-or-later like `pc/`, or GPL-3.0-or-later
  given the 2026-11-02 decision), add `rust/LICENSE`, and say in README how the Rust port relates to `pc/`.
- D2. Accept that the shipped third-engine bundle is a GPL-3.0-or-later combined work (it already is for the slippi-ssbm-asm
  codes); or keep the Rust engine distributable alone under its own terms as well (both can be true).
- D3. Ask Hero for MIT (or dual) on `mu_core_v1.h` and on any renderer-side C ABI header we propose, so the seam is
  licence-neutral. Timing is tied to the "surprise PR" plan.
- D4. Contribution terms for our patches to his renderer: plain GPL-2.0-or-later in his files (default), and whether we want
  our names in his per-file headers (his files carry none today).
- D5. Whether the Rust port may ever vendor GPL-3-only code (PhobGCC is the only GPL-3.0-only set in his tree; we skip it).

## 6. Phased plan

| phase | what | size | needs from him |
|---|---|---|---|
| 0 | this note; licence decisions D1-D5 | S | - |
| 1 | **Shot gate.** `tools/rust-port/corev1/gate_shots.sh`: run the view-golden stage set and the `play --shot-ages` scene set through `melee.exe` and through his host (core v1.1, `--capture-sim-frame`), register and score with `compare_shots.py`, write the baseline table into notes/core-v1.md. Then EFB-exact capture on his side (local branch, `--capture-efb`) and a core v1.2 `efb_readback` so our `--shots` keep their contract | M | `--capture-efb`: S on his side, or ours on a local branch |
| 2 | **Close render gaps in the transcoder** against the gate: depth-format EFB copies verified (Stadium, FoD reflection), sized/indexed draws, items and 1P modes, results screen, UI/overlays/LAB/wasm mods as GX 2D draws (section 3 option 1), cosmetics applied before the FIFO | M-L | - |
| 3 | **Contribute to his renderer**, local `gd/*` branches first, PR when the user says: (a) EFB copies not re-run on in-between presents (item 10, S); (b) present split (item 8, M); (c) cross-command draw merge (item 4, M); (d) GPU texture budget if unbounded (item 6, S); (e) true-widescreen exclusion cross-check and the authored-channel phase-0 rule (items 14, 12, S); (f) a GX-level golden-image test harness (item 1, M); (g) his decoder off the game thread (item 2, L, after measuring (a)-(c)) | M each, L for (g) | review and merge |
| 4 | **Sub-frame tier 2 and rollback marks**: core v1.2 append `native_pose_snapshot` + `native_render_scope_event` + owner tokens; fill from `pose_capture.rs`; host marks discontinuity around `state_load`; compare his audits against `melee.exe` interp | M | v1.2 header (S on his side) |
| 5 | **Perf and threading under his host**: `gx_fifo` from our decode thread with a double-buffered arena; bench-live scenarios through his host; Pokemon Stadium with savestates (12.6 ms tick, notes/core-v1.md) | M | - |
| 6 | **Launcher and Linux**: the third-engine segment already on `ours/rust-engine` points at `melee_game.dll` under his Source Port host instead of `melee.exe`; Linux cdylib at the console base against his GL/Vulkan path | M | the launcher patch (already drafted locally) |
| 7 | **Deprecate**: `melee.exe` keeps the wgpu renderer only as `--renderer ours` for the gate; when section 4's criteria hold for a release cycle, remove `render.rs`, `shader.rs`, `interp.rs`, `ui*.rs`, `texture.rs` and the presenter, keep the oracle data in a test crate | S-M | - |

**First concrete step (phase 1, first half, no change to his code):** write `tools/rust-port/corev1/gate_shots.sh` around
the existing `run.sh host/ref` and `compare_shots.py`: six view-golden stages plus the scene set at fixed ticks, both
renderers, one table with mean abs / PSNR / % within 16 per shot, committed as the baseline in notes/core-v1.md. That table is
the measure every later phase moves, and it tells us which transcoder gaps (phase 2) matter before anyone touches his tree.

## 7. Open decisions (for the user)

1. Licence D1-D5 above (section 5).
2. Overlays, LAB, settings UI and wasm presentation mods under his host: GX 2D draws (keeps them ours), his ImGui side, or
   dropped when running under his launcher.
3. Threading (section 3): invest in calling `gx_fifo` from our decode thread (ours, M) or in his decoder off the game thread
   (his, L), or neither until the shot gate is green.
4. What goes to him first, and when: the EFB-copy fix (item 10) is small, visible and stands alone; the "surprise PR once it
   has an edge" memory suggests bundling it with core v1.1 and the launcher segment.
5. Whether Linux is in scope for deleting melee_gx (criterion 6) or dropped.
6. Whether to ask for core v1.2 (pose snapshot, EFB readback, memory-window / handle subcommand, discontinuity mark) as one
   append or several.
