# Hero's renderer as the endgame: plan (Vulkan first)

Status 2026-11-08, branch `port/heroren` (off `pub/rust-port` 6a1fae236). Scoping only; no code moved. Revision 2: the
user decided that the Rust engine forces Vulkan (done on our side: `melee_gx` defaults to `Backends::VULKAN`, `WGPU_BACKEND`
overrides for tests) and that our contributions to Hero's renderer target his Vulkan backend (`gx_vulkan.cpp`) first; a
Vulkan path tracer is to be planned against his DXR one; DLSS 5 is "watch the SDK", not near-term.

Subject: Hero88go's melee-unlocked, read-only clone `scratch/ext/melee-unlocked` (upstream 0.9.86 271402e, a single squashed
commit, so his files have no history to mine; plus our local `ours/rust-engine` commits; the core v1 header lives on the local
branch `gd/core-v1` in `_build/agents/corev1/wt`). His permission covers his code; nothing of it is copied here, and nothing
was sent to him or built. Everything below about his renderer was read from his sources, with file names so it can be
re-checked; vendor SDK claims come from the headers and docs he vendors under `port/third_party`, and anything not in those is
marked unverified. Sizes: S under a day, M one to three days, L a week or more, all agent-days.

Where we already stand (notes/core-v1.md): our engine builds as `melee_game.dll` (`rust/melee_corev1`, feature `corev1`)
against his frozen `mu_core_v1.h` boundary and runs inside his unmodified `melee_source.exe`, with his renderer drawing the
game: Battlefield VS at 59.9 fps, hashes identical to our own build, his audio dump byte-identical to our mix, his state test
with 0 mismatches. The renderer part of "run on his renderer" therefore exists as a working spike (on his D3D12 backend; his
Vulkan backend is selected with `backend vulkan` in `port-settings.ini`, untested by us). What remains is the gap between a
spike and the thing we ship and test against.

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
| **Vulkan backend** (Windows and Linux, Vulkan 1.3 + dynamic rendering; HLSL to SPIR-V through the shipped `dxcompiler.dll`, glslang on Linux) | `gx_vulkan.{h,cpp}` | 1,764 |
| OpenGL 4.5 backend (Linux only) | `port/linux/renderer_gl.cpp`, `renderer_presenter.cpp` | 1,470 |
| threaded presenter and frame queue (backend-neutral) | `threaded_backend.cpp`, `frame_queue.h` | ~850 |
| sub-frame solver (frame interpolation, backend-neutral) | `subframe.{h,cpp}`, `authored_pose.{h,cpp}`, `native_pose_bridge.cpp`, `temporal_history.h` | ~2,500 |
| upscalers and NVIDIA/Intel integration (D3D12-typed) | `gx_streamline.{h,cpp}`, `gx_dlss5.{h,cpp}`, `gx_dlss5_scaling.{h,cpp}`, `gx_xess.{h,cpp}` | ~2,000 |
| DXR path tracer (D3D12-typed GPU side; CPU scene extraction is backend-neutral) | `gx_dxr_scene.{h,cpp}`, `gx_dxr_gpu_scene.{h,cpp}`, `gx_dxr_path_tracer.{h,cpp}`, `shaders/dxr_pathtrace.hlsl` | ~900 |
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

### Feature list, per backend (verified in code)

"D3D12" = `gx_d3d12.cpp`, "D3D11" = `gx_d3d11.cpp`, "VK" = `gx_vulkan.cpp`, "GL" = `port/linux/renderer_gl.cpp`. "yes" means
a code path exists, not that we ran it.

| feature | where | D3D12 | D3D11 | VK | GL |
|---|---|---|---|---|---|
| Internal resolution 1x-8x (`efb_scale`, 0 = auto), MSAA 1/2/4/8 (resolved before every read), SSAA 1-4 per side (box filter in present), anisotropy 1-16 | `render_options.h:321,360-362` | yes | yes | yes (`msaa_color_`, `pick_scale()*ssaa`, `samplerAnisotropy`) | yes |
| Present pass: box downsample, CAS sharpening, SSAO (depth-based), brightness/contrast/vibrance, DLSS HUD composite | `present_shader.h` | yes | partly (blit/sharpen shared) | yes (`gx_vulkan.cpp:1429-1482`, SSAO reads `efb_depth_` at 1381) | yes (sharpen; no SSAO) |
| CRT display model (studio monitor / home TV, curve) | `crt_shader.h` | yes | yes | **yes** (`present_crt_` pipeline, `crt_fit`, `crt_look`), although his header comment and the settings label say it is not drawn on Vulkan; the comments are stale | yes |
| Border art with soft shadow | `present_shader.h` | yes | ? | no (0 references) | ? |
| RetroArch `.slangp` presets via librashader | `user_shader.{h,cpp}` | yes | yes | no entry point (`d3d12_*`, `d3d11_*`, `gl_*` only) | yes |
| Custom texture packs (Dolphin names), cosmetic companions, texture dump | `texture_pack.*` | yes | yes | yes (`texpack::load` at 1158) | yes |
| MP4 video backdrops on CSS/SSS | `video_background.*` | yes | ? | no (0 references) | yes (3 refs) |
| True widescreen, aspect modes, looks / lightweight / effects filters, Arcade look, shield colours, tints | `gx_shader.cpp build_projection`, `gx_core.h`, `gx_retro.h` | yes | yes | yes (`presented_aspect`, `frame_widenable_`, `look_filter`) | yes |
| Settings panel (ImGui) and overlays | `pc_settings.cpp` | yes | yes | yes (`imgui_impl_vulkan`) | yes (SDL) |
| Sub-frame animation (all five modes) | `threaded_backend.cpp`, `subframe.*` | yes | yes | yes: `submit_frame(frame, overrides)` applies matrix and vertex overrides (435-461) | yes (`renderer_presenter.cpp`) |
| DLSS Super Resolution / DLAA (Streamline 2.14.1) | `gx_streamline.*` | yes | no | no (`gx_backend.cpp:44` zeroes `dlss_mode`) | no |
| DLSS Frame Generation 2x-6x, Reflex / PCL markers | `gx_streamline.*` | yes | no | no | no |
| DLSS 5 neural rendering (NGX DLSS-NR through a helper DLL) | `gx_dlss5.*`, `port/dlss5_forwarder` | yes (build flag `GX_DLSS5`) | no | no | no |
| Intel XeSS SR 3.0 | `gx_xess.*` | yes | no | no | no |
| DXR diffuse path tracing + DLSS Ray Reconstruction | `gx_dxr_*`, `shaders/dxr_pathtrace.hlsl` | code present, **forced off** (`kPathTracingAvailable = false`) | no | no | no |
| Frame pacing hooks on `Backend`: `variable_refresh` (VRR verdict from DXGI frame statistics), `display_counts_readable`, `display_timing_line`, `set_display_refresh` (tear-free sync on VRR), `vblank_next` (scanline sync), `reflex_pacing`, `frame_generation_multiplier` | `gx_core.h:637-688`, `gx_d3d12.cpp:373-418` | yes | partly | **none overridden** (VK implements only `presentation_options`, `set_skip_present`, `resize`, `presentation_stats`, `submit_frame`); vsync is a FIFO / MAILBOX / IMMEDIATE present mode chosen at swapchain build (651-660) | none |
| Exclusive fullscreen with a paced refresh mode; waitable swapchain latency 1; `ALLOW_TEARING` | `dxgi_paced_mode.h`, `gx_d3d12.cpp:683,1048-1062,3701` | yes | yes | no | n/a |
| Captures: `--capture*` PPM of the presented back buffer, F2 blink captures, flicker scan | `render_options.h:363-367` | yes | yes | yes (back buffer, `record_capture`/`write_capture`) | yes, plus `capture_efb` (EFB-exact) |
| Display latency test fill | `gx_core.h:252` | yes | yes | yes (`test_fill`) | ? |
| GPU texture / EFB-copy eviction | | ? | ? | by age only: `trim()` drops entries unused for 1800 frames every 600; a byte budget exists for pack replacements only | ? |

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
- **Render thread** (`ThreadedBackend::present_loop`, `--threaded-renderer`, which the launcher always passes; without it
  the backend presents on the sim thread at the XFB copy): holds the previous and current `Frame`, runs the `SubFrameSolver`
  for the phase `t` of each present, and calls `submit_frame(frame, overrides)` which **re-executes the whole frame's command
  list** with per-draw matrix overrides (and replacement vertex streams for effects), including its EFB copies, then presents
  at the XFB command. Phase comes from the tick's schedule (`current.tick.target`), not hand-over time (measured 2026-10-23:
  steps over 2 ms per 40 s fell from 262 to 15). A stale in-between present is discarded when a newer game frame arrived while
  its re-pose was being built. Options: present-on-hand-over on VRR displays, frame-locked grid, keyframe modes, pin-phase for
  deterministic development runs, a frame-time CSV, a latency trace, pacing spike logs. This loop is backend-neutral; it
  only asks the backend the `Backend` virtuals above, and on Vulkan every one of them answers the default (no VRR verdict, no
  vblank, no Reflex).
- Cap and sync: `fps_cap` grid; vsync on by default; VRR verdict from DXGI `PresentCount`/`SyncRefreshCount` with
  thresholds 75 Hz / 30 fps / 0.96 / 0.98 / 0.04 (`vrr::judge`; ours in `main_loop.rs judge_vrr` are the same);
  `vblank_next` scanline sync; Reflex sleep when Reflex paces; frame-generation multiplier divides the render rate; exclusive
  fullscreen re-modes the output to a refresh the cap divides.
- Rollback: the host calls `gx::mark_discontinuity(rollback)` when a state is loaded (`source_host.cpp:1346, 1381, 2686`);
  the next frame carries `discontinuous` (fighters may still be smoothed) or `hard_cut` (nothing carried); frames produced
  during re-simulation are not drawn by the game (`g_resim`).

### Why D3D12 first: what his code and comments say

No file states a design decision in so many words (and the clone has no history), so this is what is written, then what we
infer:

- `render_options.h:22`: "D3D12 is the default and the only one with DLSS; D3D11 is for machines whose driver cannot start
  D3D12. Switching takes effect at the next launch."
- `gx_backend.cpp:44`: "DLSS, XeSS and frame generation are Direct3D 12 features" (Vulkan options get `dlss_mode = 0`,
  `frame_generation_mode = 0`).
- `pc_settings.cpp:6341`: "Linux: OpenGL, or Vulkan where a Vulkan 1.3 GPU is present. Direct3D, DLSS and XeSS are Windows
  parts." `:6354`: "Vulkan is offered only where a Vulkan 1.3 GPU is present; it draws without DLSS, XeSS and the CRT model."
  Both OS lists label it "Vulkan (experimental)"; Linux defaults to OpenGL. `:6363`: Vulkan is treated like D3D11 in the
  settings ("no upscalers on either").
- `gx_vulkan.cpp:1-9`: "one backend for Windows and Linux ... What it draws follows the OpenGL renderer ... NVIDIA
  DLSS/Reflex, XeSS, the CRT model and shader presets are Direct3D or OpenGL features and are not drawn here." The CRT
  sentence is stale (the CRT pipeline is built at 1434-1459).
- `gx_streamline.h:112-115`: Streamline is integrated **as the interposer over DXGI/D3D12 creation** ("D3D12/DXGI creation
  goes through its proxies so Streamline can manage the swap chain and command lists"); `gx_streamline.cpp` and `gx_xess.cpp`,
  `gx_dlss5.cpp` are typed on `ID3D12Device`/`ID3D12Resource`/`ID3D12GraphicsCommandList`.
- `gx_d3d12.cpp:373-418, 683, 1048-1062, 1806, 3701`: every pacing instrument he built (VRR verdict, display timing line,
  tear-free vs tearing presents, waitable swapchain, exclusive-fullscreen mode picking, vblank) is DXGI: `GetFrameStatistics`,
  `DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING`, `SetMaximumFrameLatency(1)`, `IDXGIOutput::WaitForVBlank` (vblank_clock.h).
- `gx_backend.cpp:18-21`: `NvOptimusEnablement` / `AmdPowerXpressRequestHighPerformance` exports and a DXGI adapter pick.
- `NOTICE`: the GPL section 7 permission is written for the NVIDIA Streamline / NGX / Reflex and Intel XeSS SDKs.

Inference (ours, marked as such): the renderer grew on D3D12 because that is where his players are (Windows), where DLSS
shipped first for him and where DXGI hands out frame statistics, tearing control and exclusive fullscreen for free; Vulkan was
added later as the backend shared with Linux and so was built to the OpenGL renderer's feature level, not the D3D12 one. Nothing
in the code says Vulkan was avoided for a technical reason; the gaps are "not done", not "cannot be done".

## 2. Vulkan gap inventory

Every feature that his D3D12, D3D11 or GL backends have and his Vulkan backend lacks, with whether the vendor SDK he vendors
supports Vulkan today (read from `port/third_party/*` headers and docs; nothing else consulted), and the size to close the
gap in `gx_vulkan.cpp` (plus shared code where named). Our side is now Vulkan by default too (`melee_gx` on wgpu Vulkan), so
the shot gate (section 5) compares like with like on one driver stack.

| gap | vendor SDK on Vulkan | what the VK backend needs | size |
|---|---|---|---|
| **DLSS Super Resolution / DLAA** | **Verified in his vendored Streamline 2.14.1 docs**: `ProgrammingGuide.md:16,32,130,466-472,574,595-614` (Vulkan 1.2+, `sl.interposer.dll` replaces `vulkan-1.dll` or manual hooking with `slSetVulkanInfo`, `FeatureRequirementFlags::eVulkanSupported`, required instance/device extensions and 1.2/1.3 features returned by `slGetFeatureRequirements`); `ProgrammingGuideDLSS.md:53`; `sl_helpers_vk.h` exists. His `gx_streamline.cpp` is D3D12-typed throughout | (1) jittered projection + per-draw motion vectors + unjittered previous-pose constants (D3D12 has `mvec_`, `prev_*` constants; VK has none), (2) `sl::Resource` tagging with `VkImage`/`VkImageView`/`VkDeviceMemory` and the depth aspect rule (`ProgrammingGuide.md:935`), (3) Streamline init before `vkCreateInstance` or `slSetVulkanInfo` after, (4) HUD composite in the present pass (exists in `present_shader.h`) | L (motion vectors M, Streamline-VK plumbing M) |
| **DLSS Frame Generation** | **Verified in docs**: `ProgrammingGuideDLSS_G.md:116,180-186,522-548,792-849` (intercepts `vkQueuePresentKHR` / `vkAcquireNextImageKHR`; optical flow in interop mode by default, native with driver >= 527.64 and Vulkan 1.1+; a dedicated section on host/SL synchronisation on Vulkan; `queueParallelismMode`) | DLSS SR first; present/acquire through the interposer; the presenter's `frame_generation_multiplier` hook | L (after SR) |
| **Reflex / PCL markers** | **Verified in docs**: `ProgrammingGuideReflex.md:67`, `ProgrammingGuidePCL.md:64` (`slSetVulkanInfo`) | Streamline-VK init; `reflex_pacing` / `reflex_sleep` / `pcl_marker` are already backend-neutral calls in `gx_streamline.h` | S-M (after the Streamline-VK plumbing) |
| **DLSS Ray Reconstruction** | **Verified in docs**: `ProgrammingGuideDLSS_RR.md:53` (`slSetVulkanInfo`); the RR guide buffers (diffuse albedo, normal/roughness, specular albedo) are what his DXR pass already writes | the Vulkan path tracer (section 4) | L (with section 4) |
| **DLSS 5 neural rendering** | **Unverified**: the vendored `port/third_party/ngx` holds only `LICENSE.txt` (no headers); his `dlss5_forwarder/core_api.h` is an opaque C ABI and `gx_dlss5.h` is D3D12-typed ("D3D12 only", EXPERIMENTAL). Whether NGX DLSS-NR has a Vulkan evaluate path is not shown by anything he vendors | - | **watch SDK**; not planned |
| **Intel XeSS SR** | **Verified in headers**: `port/third_party/xess/inc/xess_vk.h` and `xess_vk_debug.h` are vendored beside `xess_d3d12.h`; his `gx_xess.cpp` is D3D12-typed | the same jitter + motion vector + depth inputs as DLSS; a VK init/evaluate pair | M (after motion vectors) |
| **DXR path tracing** | n/a (own code); Vulkan ray tracing is core-extension (`VK_KHR_acceleration_structure`, `VK_KHR_ray_tracing_pipeline` / `VK_KHR_ray_query`) | section 4 | L |
| **CAS sharpening, SSAO, colour sliders, box downsample** | n/a | **already present** on VK (`gx_vulkan.cpp:1429-1482`); verify visually and fix the stale comments | S |
| **CRT model** | n/a | **already present** on VK (`present_crt_`); the header and the settings label say otherwise; fix both and verify | S |
| **`.slangp` shader presets** | **Verified in header**: `librashader.h:49-51,151,181,370-377` has the `LIBRA_RUNTIME_VULKAN` API (filter chain on a `VkInstance`/`VkDevice`/queue). **Unverified**: whether the `librashader.dll` he ships was built with the Vulkan runtime (`README.txt` only says the `.so` is OpenGL-only) | `vk_prepare` / `vk_draw` / `vk_release` in `user_shader.cpp` on the model of the GL/D3D12 ones, and a call in the VK present pass | M |
| **Video backdrops** | n/a (host-side decode) | an upload path for decoded frames into the learned backdrop texture, as GL does | M |
| **Border art** | n/a | the border texture and the `crt2.w` branch of the shared present shader | S |
| **MSAA / SSAA / anisotropy** | n/a | **already present** on VK | - |
| **Texture packs / cosmetics** | n/a | **already present** on VK | - |
| **VRR verdict (`variable_refresh`)** | no DXGI statistics for a Vulkan swapchain. Candidates: `VK_KHR_present_wait` + `VK_KHR_present_id` (present completion times; on a fixed display they quantise to refresh multiples while frames come slower than the refresh, on VRR they follow the frames) or `VK_GOOGLE_display_timing` (**unverified** on Windows NVIDIA/AMD; typically Android/Linux) | a `vrr::Detector`-compatible sample source from present-wait timings; the judge is reusable as is | M (design, then S) |
| **Tear-free on VRR (`set_display_refresh`)** | present-mode switch without a swapchain rebuild: `VK_EXT_swapchain_maintenance1` (**unverified** availability on his target drivers); otherwise rebuild on change (he already has `swapchain_dirty_`) | implement the hook: FIFO when tear-free is asked, MAILBOX/IMMEDIATE otherwise | S-M |
| **Scanline sync (`vblank_next`)** | `vblank_clock.h` is a thread on `IDXGIOutput::WaitForVBlank`, which is per output and independent of the drawing API | implement `vblank_next` on VK by reusing his vblank thread for the window's output (Windows); Linux: `VK_KHR_present_wait` timings or none | S (Windows), M (Linux) |
| **`display_timing_line`, `display_counts_readable`** | as VRR | | S after VRR |
| **Exclusive fullscreen with paced refresh** | `VK_EXT_full_screen_exclusive` (NVIDIA/AMD Windows drivers expose it; **unverified** against his minimum driver) plus the mode pick from `dxgi_paced_mode.h` (DXGI, API-independent) | | S-M |
| **Waitable-swapchain latency 1** | `VK_KHR_present_wait` or a fence on acquire (he already fences per frame slot) | measure first | S |
| **EFB-exact capture** | n/a | our need (section 5): copy `efb_color_` at the XFB copy of sim frame N to a PPM, as GL's `capture_efb` does | S |
| **GPU texture / copy byte budget** | n/a | `trim()` is age-only; add the 256 MB-style budget we use (inventory item 6 in section 3) | S |
| **Our two fixes** (section 3 items 8 and 10) | n/a | EFB copies not re-run on in-between presents: the VK `submit_frame` loop at 435-461 runs `copy()` with overrides set, same defect as D3D12. Present split: lands in the backend-neutral `present_loop` and is verified on VK | S + M |
| **First-use textures created inside submit** (from the Melee Unlocked dev-chat handoff of 2026-11-08, `YOUGAME_FIXES_HANDOFF_1108.md` part 2 section 6: D3D12 `get_texture()` decodes, `CreateCommittedResource`s and uploads a never-seen texture on the render thread inside submit; a Squad Strike fighter swapped in mid-match misses the cache for its whole costume: 34-84 ms freezes measured) | n/a | **VK has the same path**: `gx_vulkan.cpp:1143-1191` on a `textures_` miss decodes (`decode_texture`), `make_image`s and records the upload on the frame's command buffer, inside `draw()`. Proposed in the handoff and endorsed here: (1) a host entry `prewarm_textures(const TextureRef*, count)` that runs the miss path and draws nothing, called before match frame 1 (and at a swap) for every TObj image and TLUT of each loaded costume; (2) a per-frame cap on new texture creation with the rest finished next frame or on a worker. For VK the natural form is a transfer queue / worker thread with its own command pool and a timeline semaphore, which D3D12 lacks a direct analogue for. Both backends share the `TextureRef` key, so one host call serves all | S (cap) + M (prewarm entry + VK worker upload) |
| **Independent flip and VRR when embedded** (same handoff, part 1: his VRR detector reads the swap chain's `GetFrameStatistics`; inside a desktop-composed child window those read as fixed refresh and the cap never engages; measured input latency 27.0 vs 22.7 ms, "8% of presents never shown") | n/a | not a Vulkan gap as such, but a constraint on any Vulkan pacing design: present-wait timings are only meaningful on an independent-flip top-level window too. For our engine in his host: nothing on our side; the window is his, and the handoff's remedy (own top-level `WS_OVERLAPPEDWINDOW`, no owner, optionally exclusive fullscreen) applies to the host process. On Vulkan, independent flip on Windows needs a plain top-level window plus `VK_PRESENT_MODE_IMMEDIATE_KHR` / `MAILBOX_KHR` for tearing or `FIFO_KHR` for sync (he picks these at 651-660) and, for the exclusive case, `VK_EXT_full_screen_exclusive` (unverified against his minimum drivers). PresentMon ("Hardware: Independent Flip") is the check, as the handoff uses | S (document and test) |

What Vulkan gives his project back, besides parity: one backend for Windows and Linux (his stated goal in the file header),
one SPIR-V path for every feature including ray tracing (DXC compiles his HLSL ray tracing shaders to SPIR-V with
`-fspv-target-env=vulkan1.2` and `SPV_KHR_ray_tracing`; **unverified** on his exact shader), and a path for our Rust engine that
matches our own default.

## 3. Inventory of our melee_gx / platform-loop improvements, mapped onto his renderer

Sources: notes/perf.md, notes/render.md, notes/display.md, `git log -- rust/melee_gx rust/melee_platform/src/{gfx,main_loop}.rs`,
and today's `port/perfspike` commit 5813a8554. "Lands in" names the file on his side when the item is a Vulkan-first
contribution.

| # | ours (commit) | what it does | does his have it | would it port / lands in | size |
|---|---|---|---|---|---|
| 1 | Aurora-exact GX port with oracles (ea458449b, 59e97c611): WGSL byte-identical to Aurora, texture decode / state / shim oracles, 0-diff render test vs Aurora's screenshot | correctness reference | no: his generator is Dolphin-transcribed HLSL, a different pipeline. Pixel equality between the two is not expected | the code does not port; the **method** does (a GX-API-level golden image set with a fixed scene) | M for a harness in his tree |
| 2 | GX decode on its own thread (30f171eff) | tick p50 5.5 -> 2.0 ms on Temple 4P | no: his decode and texture snapshot run inside `gx_fifo` on the game thread. Inside his host our tick measured 2.60 ms of which his decode was **1.47 ms** (notes/core-v1.md) | partly; his decoder reads live game memory. For our engine the transcoder copies into a MEM1 arena, so a double-buffered arena plus calling `gx_fifo` from our decode thread works without his code changing; for his own game it is a larger change in `gx_core.cpp` (backend-neutral) | M ours, L his |
| 3 | prepare / draw_prepared split, persistent GPU buffers, uniform dedupe (cf565dd1e) | per-present cost | mostly yes on D3D12 (`FrameGeometry geometry_[4]`, one upload, `vs_constants_bytes`); **VK: check** whether its per-frame ring re-uploads geometry on every sub-frame present (`f.ring.mapped` at 449-450 suggests per-present vertex upload) | if so, per-sim-frame geometry caching in `gx_vulkan.cpp` as D3D12 has | S-M |
| 4 | draw batching across commands (655cc1328): 2520 -> 1255 calls on Temple | fewer draw calls | partial: `DrawSegment`s merge within one display list only | `gx_core.cpp record_draw` (backend-neutral); the pairing identity must stay per source draw | M |
| 5 | pipelines created on the decode thread / warm (655cc1328) | no first-use stalls | D3D12: worker PSO compile, fallback PSO, pipeline library, `recipes.bin`. **VK: check** whether pipelines compile synchronously on first use (`build_pipeline` at 1021-1050 keys by state; no worker pool seen) | if synchronous: a VK pipeline cache file (`VkPipelineCache` save/load) + the fallback-pipeline idea | M |
| 6 | content-keyed decoded-texture cache, 256 MB budget, expiry (547866504, 3729f4d84, 8ec7e5c2a) | no re-uploads, no VRAM growth | CPU side yes (`TextureSnapshotCache`); VK GPU side age-only `trim()` | a byte budget in `gx_vulkan.cpp trim()` | S |
| 7 | texture bind-group sharing (5813a8554) | fewer binds | VK: descriptor sets per texture set; check for a set cache keyed by the 8 textures | if absent, a small descriptor-set cache | S |
| 8 | **present split** (5813a8554): the adopting present only prepares and re-presents the previous EFB; the next present draws | frame-interval tail; p99 7.20 -> 4.72 ms Temple | not as such; each of his presents records every draw, so the fresh frame's upload + full pass lands in one interval | `threaded_backend.cpp present_loop` (backend-neutral), measured on VK with `--frame-times` | M |
| 9 | free retired frames off the present path (e1b3abf79) | adopt 1.4 -> 0.15 ms | yes (`FrameQueue::recycle`) | no | - |
| 10 | EFB copies keep a sim frame's exact pose (eeabe05ed) | Stadium screen not phase-dependent | **no**, on every backend: VK `submit_frame` runs `copy()` for non-XFB copies with overrides set (`gx_vulkan.cpp:455-461`), D3D12 the same (`gx_d3d12.cpp:3645-3649`) | skip non-XFB copies on a re-posed present (`overrides && t != 1`), or run them once per sim frame at phase 1; VK first, D3D12/D3D11/GL after | S each |
| 11 | deterministic screenshot tick (5813a8554) | shot = the tick asked for | his captures are presented-frame PPMs counted by XFB copies | section 5 (EFB-exact capture), not a port | - |
| 12 | render-only frame interpolation (`interp.rs`, `pose_capture.rs`): authored FObj channels with a bit-exact phase-0 check, camera slerp, cut detection | sub-frames | yes, larger (envelopes, stage lock, menu mode, audits); his authored check uses tolerance 0.002 | our bit-exact rule as a tightening in `authored_pose.cpp` (backend-neutral) | S |
| 13 | pacing (f879201a6): grid, phase from schedule, stale drop, VRR detect, G-Sync cap | pacing | yes on D3D12 (same formulas); **VK answers none of the hooks** | the VRR / vblank / tear-free hooks in `gx_vulkan.cpp` (section 2) | M |
| 14 | renderer-side true widescreen with exclusions (f879201a6) | 16:9 without touching the sim | yes, all backends | cross-check the two exclusion lists | S |
| 15 | MSAA resolve before reads, depth copies from sample 0 | | yes, VK included | no | - |
| 16 | GPU fault handling (8ec7e5c2a) | clean stop | yes | no | - |
| 17 | rollback-safe renderer state, `Detacher` by content hash | | not applicable to his memory model | no | - |
| 18 | UI batch renderer + SDF fonts, settings overlay, HUD overlay, wasm presentation mods | our presentation layer | ImGui on his side | not a port; section 4 "overlays" | decision |
| 19 | texture overrides for cosmetics (`tex_override.rs`) | skins | his packs and cosmetic companions | applied before the FIFO (bytes in MEM1) or exported in his pack naming | S-M |
| 20 | sampling profiler, `MELEE_FRAME_TIMES`, `MELEE_PRESENT_SPIKE`, `MELEE_GX_PROF` | diagnostics | yes (`--frame-times`, spike log, latency trace); VK has no `profile_line` | a `profile_line` for VK | S |
| 21 | `corev1::Transcoder` (d9a8b2095, 1923f3677) | the seam | n/a | stays ours (section 4) | - |
| 22 | perf.md sim-side work | not renderer | n/a | n/a | - |
| 23 | first-use textures: ours are decoded, created and uploaded in `Renderer::prepare` on the presenting thread (`render.rs:1432-1452`, `gxprof newtex`); perf3's `PipeFactory::warm` prewarms **pipelines** on the decode thread, not textures | we have **the same defect** as his `get_texture()` (section 2, last rows), unmeasured on our side because nothing swaps a fighter mid-match in our tests yet | his D3D12 and VK both; the handoff's `prewarm_textures` + per-frame cap | on our side: move texture creation to the decode thread beside `warm` (S-M); on his: section 2 |

Perf context (notes/bench-live.md, 2026-11-02, same machine, both windows hidden/unfocused, his D3D12): CPU per rendered frame
ours 0.99 ms vs his 0.95 ms (FD 2P), 2.8 vs 3.0 ms (Temple 4P); his 1% low was better (152 vs 54 fps on Temple) before our
perfspike work; his sim tick 2.2-4.4 ms vs our 4.1-7.5 ms with our GX recording in the tick (headless we are 0.3-0.5 ms).
Inside his host our tick is 2.6 ms of which 1.47 ms is his decoder. No number exists yet for his Vulkan backend; the first
measurement of the plan is bench-live on his VK vs his D3D12 vs ours (now wgpu Vulkan).

## 4. A Vulkan path tracer on par with his DXR path

### What his DXR path tracer is, and why it is off

- Design (`shaders/dxr_pathtrace.hlsl:1-2`, `gx_dxr_path_tracer.cpp:1`): "Hybrid one-bounce GX path-tracing pass. The raster
  image remains the material-color source so Melee's TEV textures and authored lighting survive; DXR traces diffuse secondary
  transport." `RayGen` reads the rasterised EFB pixel as the primary albedo, casts a primary ray through the GX frustum
  (`frustumLeft/Right/Top/Bottom`, `nearPlane`) to find the hit point and normal, then up to `maxBounces` cosine-weighted
  bounces (dispatched with 1 sample, 3 bounces at EFB resolution, `gx_d3d12.cpp:2737`); a secondary hit's colour is the
  **interpolated vertex colour** (`ClosestHit`, `payload.vertexColor`; `sampled_material_color`), a miss a fixed sky constant.
  It writes `PathColor` (HDR, then fed to DLSS as the colour input in R16G16B16A16, `gx_d3d12.cpp:1606-1614`) plus the three
  DLSS-RR guide buffers (`DiffuseGuide`, `NormalGuide`, `SpecularGuide`).
- Scene (`gx_dxr_scene.{h,cpp}`, **backend-neutral C++**): once per present, for every draw of the main camera
  (`same_camera`, first draw with `xf_regs[0x26] == 0`), every vertex is transformed **on the CPU** through its GX position and
  normal matrices into **view space** (`transform_draw_vertex`, the matrices from `DrawCall::posMatrices`/`normalMatrices`,
  or the sub-frame `overrides`), strips are expanded to a triangle list (`append_draw_indices`), and everything is appended to
  one `DxrScene` (positions, normals, colour 0, UV 0; `DxrSceneGeometry` keeps `draw_index` per range). No texture, TEV or
  blend state crosses; translucent and additive draws (sparks, shields, glass) are treated as opaque geometry; lines are skipped.
- GPU (`gx_dxr_gpu_scene.cpp`): **one BLAS with one geometry** holding the whole frame (`NumDescs = 1`,
  `D3D12_RAYTRACING_GEOMETRY_FLAG_OPAQUE`, `PREFER_FAST_TRACE`), one TLAS with one instance, **rebuilt from scratch every
  present** across 3 frame slots; no `ALLOW_UPDATE`/refit, no reuse across frames.
- Why it is off (`render_options.h:204-207`): "DXR path tracing and Ray Reconstruction: not offered in 0.8. They rebuild the
  whole stage's ray tracing geometry every frame, which dropped the picture to a few frames per second in matches. Hidden and
  forced off (saved values ignored) until that is reworked." `kPathTracingAvailable = false`; `pc_settings.cpp:2321,6475-6478,
  7103` gate the toggles and `dxr_path_tracing_available()` on it. No TODO/FIXME exists in the DXR files; the only other hook
  is a diagnostic `dxr_scene_diag_frame` probe that logs geometry counts.
- What else it costs when on: MSAA is disabled with path tracing (`msaa_on()`, `gx_d3d12.cpp:700,1264`); the DLSS colour
  path switches to HDR; RR requires DLSS on (`rr_available`).

### Why the blockers carry over, and how a Vulkan tracer avoids them

The slowness is not DXR's. It is (a) transforming every vertex of every main-camera draw on the CPU each present (the same
~1400 draws / tens of thousands of vertices the raster path uploads), because positions are baked in **view space**, so a
camera move invalidates all of them; (b) rebuilding one monolithic BLAS from those vertices every present; (c) tracing at
full EFB resolution with no accumulation. A Vulkan port that copies the design inherits all three. The design below changes
them; the same changes would speed up his DXR path and should be proposed to him in the same terms.

1. **World-space BLAS per draw identity, TLAS per present.** Build each draw's triangles in **model space** (the vertex
   stream as recorded, no matrix) and put the draw's position matrix in the **TLAS instance transform**. Rigid draws
   (`!skinned`, which is the whole stage and the HUD) then need **no BLAS rebuild** between frames: cache the BLAS by
   `DrawCall::identity` + vertex count + a hash of the vertex bytes (the `TextureSnapshotCache` idea applied to geometry), build
   it once, and refit nothing. Only the TLAS (one instance per draw, ~1400) is rebuilt per present, which is cheap. The CPU
   per-vertex transform disappears; the camera move becomes a change of instance transforms. This needs one more thing from
   the decoder: GX vertices index **one of up to 64 matrix slots per vertex** (`Vertex::posmtx`), so a draw whose vertices use
   several slots is split into one instance per slot (segments by `posmtx`), or, when it is skinned, handled by rule 2.
2. **Skinned draws (fighters) rebuilt per present, small.** Fighters are a few thousand triangles each; transform their
   vertices (CPU or a compute pre-pass using the posed matrices, which also makes sub-frame `overrides` free) and rebuild
   their BLASes with `PREFER_FAST_BUILD` or refit (`ALLOW_UPDATE` + update) since topology is constant.
3. **Material capture at the decoder.** His scene carries colour 0 and UV 0 but no texture; secondary hits see vertex
   colour, which is why bounces look grey on textured stages. Carry per-geometry `TextureRef[0]` (the snapshot already has
   the decoded image on the GPU for raster) and the TEV's first-stage mode so a hit can sample the base texture through a
   bindless array; blend mode and `zmode` to **skip** additive/non-depth-writing draws (sparks, glows) and to mark translucent
   ones non-opaque. All of that is in `DrawCall` already (`bp.blendmode()`, `bp.zmode()`, `textures[]`); only the scene struct
   grows.
4. **Resolution and accumulation.** Trace at half EFB resolution (or DLSS/XeSS render size when active), 1 spp, temporal
   accumulation with the raster depth/motion vectors (the same `mvec_` work section 2 needs for DLSS), and either DLSS-RR
   (Streamline on Vulkan, docs verified) or a simple spatio-temporal filter when RR is unavailable. The present pass composites
   `PathColor` over the raster image as today.
5. **Ray query, not a ray tracing pipeline, first.** His raygen/closest-hit/miss program is tiny and does its own
   material lookup; the same logic as a **compute shader with `VK_KHR_ray_query`** (`rayQueryEXT`) needs no shader binding
   table or pipeline library and is the shorter path to a first picture. `VK_KHR_ray_tracing_pipeline` can follow if hit
   shaders are wanted. Extensions: `VK_KHR_acceleration_structure`, `VK_KHR_ray_query` (or `_pipeline`),
   `VK_KHR_buffer_device_address`, `VK_KHR_deferred_host_operations`; his backend asks for Vulkan 1.3 + dynamic rendering
   only (`gx_vulkan.cpp:479-530`), so device creation grows a feature chain and a capability probe that falls back cleanly
   (as `vulkan_available()` does).
6. **Shader source.** Keep his HLSL and compile to SPIR-V with the DXC he ships (`-T lib_6_3` for RT pipelines or `-T cs_6_5`
   with `RayQuery<>` for ray query, `-fspv-target-env=vulkan1.2`); **unverified** that his DXC build has the SPIR-V ray
   tracing path enabled. Fallback: a GLSL/Slang copy compiled with glslang (he already uses glslang on Linux).

"On par" means: at his settings (1 spp, 3 bounces, EFB resolution) the Vulkan pass costs no more GPU time than his DXR pass
on the same GPU, measured with his own `profile_line`/timestamp queries ported to VK. With rules 1-2 the per-frame CPU and
BLAS cost drops below his by construction; rule 4 is what makes it shippable at a frame rate, for both APIs.

Sizes: scene changes in the backend-neutral `gx_dxr_scene.*` (world-space + per-slot split + material fields, the BLAS cache
key) M; Vulkan acceleration structures + instance TLAS + ray-query compute pass M; accumulation / RR hookup M; porting the
same BLAS caching back to his D3D12 path S-M once the scene code is shared. Total L. Prerequisite: the motion-vector work
from section 2. Open question for him: whether the shipped `dxcompiler.dll` emits SPIR-V ray tracing (else glslang).

## 5. The seam, what must stay, and the shot gate

**FFI direction.** His renderer is internal C++ (`gx::init(Backend*)`, `gx::write_fifo_bytes`), not a library with a C
export; the only stable boundary he offers is the game side: a DLL loaded by `melee_source.exe`. So the direction is
**our engine as the game library** (`melee_game.dll`, `mu_game_entry_core_v1`, image base 0x82800000), his process owning the
window, swapchain, settings and presenter. This is what core v1/v1.1 already does. The reverse (his renderer as a DLL inside
`melee.exe`) would need a new C ABI around `gx::Backend` plus his window/settings code, and gains nothing for the launcher
goal; not proposed.

**What we emit, what he consumes.** The game->host call is `gx_fifo(bytes)`: the console's write-gather stream. Our
encoders write Aurora's dialect, so `melee_gx::corev1::Transcoder` rewrites each drained buffer (notes/core-v1.md): every
handle's bytes are copied once into a MEM1 bump arena (238 MB staged and 8,702 texture copies over 3,400 ticks; arena wrap
mid-frame handled by a draw-done + re-bind), registers get console addresses, `GXCopyTex` becomes the EFB copy registers into a
4 MB MEM1 pool, `GXCopyDisp` becomes the XFB copy that ends his frame, PC-only subcommands are dropped. Measured transcode 0.125
ms per tick. Not yet exercised: `DRAW_SIZED`/`DRAW_INDEXED`, depth-format EFB copies, the Geno palette beyond a unit test.
Later ask (user decides when): a core v1.2 host call that maps a region we own into the GX window, or a handle-resolving
subcommand, removing the copies.

**Backend selection.** His host reads `backend vulkan` from `port-settings.ini` (`pc_settings.cpp:2362`); the launcher does
not pass it. For "the Rust engine forces Vulkan" under his launcher, either our launcher patch writes `backend vulkan` when the
Rust engine is picked, or (cleaner, his side) the engine entry can state a preferred backend. Decision for the user.

**Threading.** His decoder runs on the thread that calls `gx_fifo`; he presents on his render thread. Our async GX decode
thread is pointless under his host, so the 1.47 ms of his decode sits in our tick. Ways out: (a) call `gx_fifo` from our
decode thread with a double-buffered MEM1 arena (M, ours), (b) inventory item 2 on his side (L).

**Interpolation.** Tier 1 (matrix pairing) is free and runs on every backend including VK. Tier 2 (authored poses) needs
`native_pose_snapshot` / `native_render_scope_event` and the `0xF0/0xF1` tokens, which are in the full `MuHostApi` but not in
core v1; propose a v1.2 append and fill it from `pose_capture.rs` (M).

**Rollback re-pose.** The host must `gx::mark_discontinuity` around our `state_load` (one line on his side); our re-simulated
frames already send no draws and no copies.

**Texture prewarm (seam requirement).** Under his host every texture we draw for the first time is decoded, allocated and
uploaded on his render thread inside submit (section 2, last rows); his match intro hides it for a normal match, a mid-match
load (Squad Strike swap, our m-ex and cosmetic swaps, a Lab reload) does not. The core v1.2 append should carry the handoff's
`prewarm_textures(const TextureRef*, count)` (or, simpler for a transcoded engine, "prewarm every texture referenced by this
FIFO buffer" since our transcoder already knows each texture's console address, format, size and TLUT), and our engine calls it
when it loads a costume or stage. Until the host has it, the transcoder can emit a draw-nothing prewarm itself: bind each new
texture in a zero-area scissor draw at scene load, which runs his miss path early at the cost of a few dummy draws.

**Independent flip / VRR.** Pacing quality is the host window's: his VRR verdict and tear-free logic need an independent-flip
top-level window (handoff part 1: composed, the statistics read fixed refresh, +4.3 ms input latency). Our engine has no say in
the window; the requirement lands on the launcher integration (phase 8): launch as its own top-level process, never as an
embedded child, and on Vulkan check with PresentMon that the whole run is "Hardware: Independent Flip".

**Screenshots for our tests.** His captures are PPMs of the presented back buffer on every backend; only Linux GL has
`capture_efb`. Our shot tooling reads the 640x480 EFB at an exact tick. `compare_shots.py` registers his capture to ours and
reports mean abs 2.1-11.6/255, PSNR 18-32 dB today, dominated by his 73:60 presentation and rescaling. The gate needs
**EFB-exact capture in his Vulkan backend** (section 2, S) and, for the game library, a core v1.2 host call `efb_readback`.

**Overlays, UI, mods.** Ours draw with `melee_gx::ui` after the EFB blit; under his host they have no surface. Options:
(1) emit them as GX draws on an ortho 2D layer at the end of our frame (M; his DLSS HUD composite treats flat 2D draws as UI);
(2) ImGui hooks on his side; (3) nothing. (1) is the only one that keeps our UI ours and renderer-independent. Decision.

**Linux.** His Source Port runs on Linux with GL or Vulkan; our cdylib would need the console base there
(`-Wl,-Ttext-segment=0x82800000`). With Vulkan on both OSes this is the same backend code. Not started.
Checked against his public v0.9.107 (6d5e2e4, read-only, 2026-11-08): the Linux route needs three things on his side, none
of them build config. (1) The core v1 header and its host side: `mu_core_v1.h` / `mu_game_entry_core_v1` are not in his
tree; they exist only on our local `gd/core-v1` branch, for the Windows `source_host.cpp`. (2) A core v1 entry in his Linux
host: `port/linux/host_services.cpp` `initialize()` dlopens `melee_game.so` and requires his full native `mu_game_entry`
table (`state_snapshot` ...). (3) An image-size exemption: `inspect_image()` refuses a game ELF that ends past 0x84000000
("64 MiB GX physical address window", 24 MB from the base); our engine's loadable image on Linux is about 41 MB (the
Windows image, 36 MB, passes only because his unsandboxed loader checks the base alone). The transcoder copies every byte GX
reads into MEM1, so core v1 does not need the window; either his check exempts core v1, or our image goes under 24 MB
(the size study in notes/core-v1.md). Until then phase 1 runs on the PC (Windows, `backend vulkan`).

**What must stay.** `melee_gx` tests (`tests/render.rs` 0-diff vs Aurora, `tests/oracle.rs`), the render proof vs the C
build (notes/render.md, mean abs 1.1-3.4/255 per frame), every scripted shot consumer (`melee play --shots/--shot-ages/
--shot-ticks`, `MELEE_RETAIL_SHOT_TICKS`, `tools/rust-port/play/full_loop.txt`, `menus/gen_lua.py`, `netplay/gen_css_scripts.py`,
the Stadium determinism check, gd-ui and modhost UI tests, `bench-live`). `melee.exe` (wgpu, now Vulkan by default) stays the
reference until his path passes them; the transcoder is the only core-v1-only renderer code, so nothing forks. Note for ours:
`display.md`'s VRR probe reads DXGI frame statistics and only works on wgpu's DX12 backend, so with Vulkan as default our own
VRR detection is off unless `WGPU_BACKEND=dx12`; the same gap as his Vulkan backend has (section 2).

**Criteria for deleting melee_gx (render / shader / interp / ui / texture; API, shim, state decode, resources and the
transcoder stay).** All of:
1. EFB-exact capture in his VK backend and a shot gate `tools/rust-port/corev1/gate_shots.sh` over the view-golden set (FD,
   BF, YS, GG, FoD, DL64) plus the `play --shot-ages` scene set, both renderers on Vulkan at the same ticks, at the bar we hold
   against the C build: mean abs <= 2.5/255 and >= 97% within 16 per shot; Stadium ages 300-1100 identical run to run.
2. Every scripted shot consumer has a core-v1 variant and passes.
3. Our overlays, LAB and wasm presentation mods render through his renderer.
4. Sub-frame tier 2 under his host is at least as good as our interp (his audits).
5. Frame-interval p99 and 1% low under his VK backend are no worse than `melee.exe` on the bench-live scenarios.
6. Linux runs (his VK) for the same gate, or Linux is explicitly dropped.
7. The melee_gx oracle data is kept in a small test crate or archived with a note.
Until then melee_gx is "frozen reference": bug fixes the gate needs, no features.

## 6. Licence

Facts, read from the files:
- His repo: `LICENSE` is the GPL-3.0 text; `NOTICE` says releases are **GPL-3.0-or-later since 0.8.5** (GPL-2.0-or-later up
  to 0.8.1), files whose header names another project keep that notice, and a GPL-3 section 7 additional permission covers
  linking with the NVIDIA Streamline / NGX / Reflex and Intel XeSS SDKs (his own code only). Per-file SPDX counts under `port/`
  and `sourceport/game`: 379 `GPL-2.0-or-later` (most of the renderer: `gx_core.h`, `gx_shader.h`, `render_options.h`,
  `subframe.h`, `threaded_backend.cpp`, `gx_vulkan.cpp`, the DXR files, the Linux GL renderer), 16 `GPL-3.0-or-later`
  (`dlss5_forwarder/core_api.h` among them), 8 `GPL-3.0-only` (all PhobGCC), 29 Apache-2.0, 3 MIT, 4 BSD-2/CC0.
  `gx_shader.cpp` is Dolphin-derived; `texture_pack.cpp` Ishiiruka-derived. Vendored: Streamline headers MIT, XeSS headers
  under Intel's licence, NGX under NVIDIA's (licence file only), librashader MPL-2.0, DXC LLVM + MS licences, Vulkan-Headers
  Apache-2.0 OR MIT.
- Ours: `README.md`: `pc/` is GPL-2.0-or-later (`pc/LICENSE`); Geno and the interop specs are MIT (`docs/specs/LICENSE-MIT`,
  `pc/geno/LICENSE-MIT`). The Rust workspace (`rust/Cargo.toml`) says `license = "CC0-1.0"` on every crate, from the skeleton
  commit b551c8dc2, and there is no `LICENSE` under `rust/`. STATUS.md records the user's 2026-11-02 decision that implementing
  Slippi codes from slippi-ssbm-asm (GPL-3.0) "makes the combined work GPL-3, which the user accepts". `mu_core_v1.h` is his
  (GPL); our `corev1.rs` was written from the header alone and carries its layout.

What linking means (not legal advice; the conservative reading of the GPL FAQ, for the user to weigh):
- **Running our DLL inside his exe** is one program at run time. Distributed together, the conservative view is a combined
  work under GPL-3.0-or-later, so our library must be under GPL-3-compatible terms. GPL-2.0-or-later is compatible; MIT is;
  CC0-1.0 is compatible but discouraged by the FSF for software and surely not what was meant. Our source files do not become
  GPL-3; the combination does.
- **Contributing into his renderer** (sections 2-4): code we write in his tree goes under the file's terms (GPL-2.0-or-later
  for the renderer files, GPL-3.0-or-later for new ones) and his section 7 permission is his to extend. It does not change our
  repo's licence. A Vulkan Streamline/XeSS integration on his side falls under his existing section 7 exception.
- **Copying his renderer code into our repo** (not proposed; forbidden in this task) would bind the Rust build to GPL.
- **What stays MIT regardless**: `docs/specs`, `pc/geno`. Asking him to put `mu_core_v1.h` (and any renderer C header we
  propose) under MIT or dual MIT/GPL would make the boundary licence-neutral.

Decisions for the user (none taken here):
- D1. State the Rust port's licence explicitly (replace `CC0-1.0`: GPL-2.0-or-later like `pc/`, or GPL-3.0-or-later given the
  2026-11-02 decision); add `rust/LICENSE`; say in README how the Rust port relates to `pc/`.
- D2. Accept that the shipped third-engine bundle is a GPL-3.0-or-later combined work; or keep the Rust engine distributable
  alone under its own terms as well (both can be true).
- D3. Ask Hero for MIT (or dual) on `mu_core_v1.h` and any renderer-side C ABI header we propose.
- D4. Contribution terms for our patches to his renderer (GPL-2.0-or-later in his files by default) and attribution in headers.
- D5. Whether the Rust port may ever vendor GPL-3-only code (PhobGCC is the only such set; we skip it).

## 7. Phased plan (Vulkan first)

| phase | what | size | needs from him |
|---|---|---|---|
| 0 | this note; licence decisions D1-D5; decide how the Rust engine selects `backend vulkan` under his launcher | S | - |
| 1 | **Measure on Vulkan.** bench-live scenarios on his VK vs his D3D12 vs our wgpu-Vulkan `melee.exe`; core v1.1 in his host with `backend vulkan`; first `gate_shots.sh` table with his existing back-buffer capture (view-golden stages + `play --shot-ages` scenes, registered and scored by `compare_shots.py`) | M | - |
| 2 | **Vulkan parity + our fixes in `gx_vulkan.cpp`** (local `gd/*` branch; PR when the user says): (a) EFB copies not re-run on re-posed presents (item 10, S); (b) present split in `present_loop`, verified on VK (item 8, M); (c) `Backend` hooks for VK: `vblank_next` via his DXGI vblank thread, `set_display_refresh` present-mode switch, VRR verdict from present-wait timings (design first), `display_timing_line`, `profile_line` (M); (d) EFB-exact capture (S); (e) texture/copy byte budget (S); (f) border art, video backdrops, `.slangp` via librashader-VK (S+M+M); (g) fix the stale "no CRT on Vulkan" comment and label, verify CAS/SSAO/CRT on VK (S); (h) VK pipeline cache file / fallback pipeline if first-use compiles are synchronous (item 5, M); (i) **first-use textures off the present**: per-frame cap on new texture creation in the VK `draw()` miss path (S), a worker/transfer-queue upload with a timeline semaphore (M), and the `prewarm_textures` host entry shared with D3D12 (S on the ABI, M across backends); this is the handoff's Squad Strike freeze and the earliest item a player feels | M-L total | review |
| 3 | **Upscalers on Vulkan**: per-draw motion vectors + jitter + unjittered previous projection in the VK backend (M); Streamline-VK plumbing (interposer or `slSetVulkanInfo`, `sl::Resource` tagging) (M); DLSS SR/DLAA, then Reflex/PCL (S-M), XeSS-VK (M), then Frame Generation (L). DLSS 5: watch the SDK, nothing planned | L | his section 7 exception already covers the SDKs |
| 4 | **Vulkan path tracer** (section 4): world-space BLAS cache by draw identity, per-present TLAS, skinned draws rebuilt, material fields in `DxrScene`, ray-query compute pass, half-res + accumulation, DLSS-RR on VK; the BLAS caching ported back to his D3D12 path so both are on par and his can be re-enabled | L | whether his DXC emits SPIR-V ray tracing |
| 5 | **Close transcoder gaps** against the gate: depth-format EFB copies, sized/indexed draws, items/1P/results shots, UI/overlays/LAB/wasm mods as GX 2D draws, cosmetics before the FIFO | M-L | - |
| 6 | **Sub-frame tier 2 and rollback marks**: core v1.2 append (`native_pose_snapshot`, `native_render_scope_event`, owner tokens, `efb_readback`); host marks discontinuity around `state_load` | M | v1.2 header (S his side) |
| 7 | **Perf under his host**: `gx_fifo` from our decode thread with a double-buffered arena; Pokemon Stadium with savestates | M | - |
| 8 | **Launcher and Linux**: the third-engine segment (already drafted on `ours/rust-engine`) points at `melee_game.dll` under his Source Port host with `backend vulkan`; Linux cdylib at the console base against his VK path | M | the launcher patch |
| 9 | **Deprecate**: `melee.exe` keeps wgpu as `--renderer ours` for the gate; when section 5's criteria hold for a release cycle, remove `render.rs`, `shader.rs`, `interp.rs`, `ui*.rs`, `texture.rs` and the presenter; keep the oracle data | S-M | - |

**First concrete step (phase 1, no change to his code):** run core v1.1 in his host with `backend vulkan` in the test
`port-settings.ini` and his `--capture-sim-frame`, on the six view-golden stages plus the `play --shot-ages` scene set; write
`tools/rust-port/corev1/gate_shots.sh` around the existing `run.sh host/ref` and `compare_shots.py`; commit the table
(mean abs / PSNR / % within 16 per shot, his VK vs our wgpu-Vulkan) into notes/core-v1.md, with a bench-live row for his VK
backend beside his D3D12. That table is what phases 2-4 move, and the VK run itself tells us at once which of section 2's
"already present" entries hold.

## 8. Open decisions (for the user)

1. Licence D1-D5 (section 6).
2. How the Rust engine forces Vulkan under his launcher: our launcher patch writes `backend vulkan`, or a preferred-backend
   field in the core v1.2 game table (his side, S).
3. Overlays, LAB, settings UI and wasm presentation mods under his host: GX 2D draws, his ImGui side, or dropped.
4. Threading (section 5): our decode thread calling `gx_fifo` (M, ours) or his decoder off the game thread (L, his), or wait
   for the shot gate.
5. What goes to Hero first and when: the EFB-copy fix (item 10) on VK + D3D12 is small, visible and stands alone; the
   "surprise PR once it has an edge" memory suggests bundling it with core v1.1, the launcher segment and the VK pacing hooks.
6. The VRR verdict on Vulkan: accept a present-wait-timing design (M, unverified on his drivers) or leave Vulkan without VRR
   detection as it is today.
7. Path tracer order: Vulkan-only first (section 4) or the BLAS-cache rework in his shared scene code first so his D3D12 path
   can be re-enabled at the same time.
8. Linux in or out of the melee_gx deletion criteria.
9. Core v1.2 asks (pose snapshot, EFB readback, memory window / handle subcommand, discontinuity mark, preferred backend,
   `prewarm_textures`) as one append or several.
10. Texture prewarm on our side before his ABI has it: transcoder-emitted draw-nothing prewarm draws at scene load (works
    today, costs dummy draws) or wait for the host entry.
11. Launcher integration must launch the Rust engine as its own top-level process (independent flip), never embedded; whether
    that constraint is written into the launcher patch now or left to his handoff's standalone work.
