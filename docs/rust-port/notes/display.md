# Display options (`melee play`)

`melee play --help` prints the same list. Each option resolves as: **command-line flag**, else
**`MELEE_<KEY>`** in the environment, else **`key=value` in `settings.cfg`** beside the executable
(`MELEE_SETTINGS_CFG` moves it; the C build's settings file, read through `melee_data::settings`), else
the default. `--save-display` writes the resolved values back to settings.cfg.

| flag | env / settings key | values | default |
|---|---|---|---|
| `--widescreen`, `--no-widescreen` | `MELEE_WIDESCREEN` / `widescreen` | 0, 1 | 0 |
| `--fullscreen[=borderless\|exclusive]`, `--windowed` | `MELEE_FULLSCREEN` / `fullscreen` | off, borderless, exclusive | off |
| `--window WxH` | `MELEE_WINDOW` / `window` | e.g. 1920x1080 | 1280x960 (1280x720 with widescreen) |
| `--render-scale N\|auto` | `MELEE_RENDER_SCALE` / `render_scale` | 1..8, auto | auto (follows the window, at most 4) |
| `--msaa N` | `MELEE_MSAA` / `msaa` | 1, 2, 4, 8 | 1 (off) |
| `--vsync on\|off` | `MELEE_VSYNC` / `vsync` | on, off | on |
| `--widescreen-mode renderer\|camera` | `MELEE_WIDESCREEN_MODE` / `widescreen_mode` | renderer, camera | renderer |
| `--fps-cap N\|off` | `MELEE_FPS_CAP` / `fps_cap` | 20..1000, off | off |
| `--vrr auto\|on\|off` | `MELEE_VRR` / `vrr` | auto, on, off | auto |
| `--interp auto\|on\|off` | `MELEE_INTERP` / `interp` | auto (on above 65 fps), on, off | auto |
| `--interp-mode interpolate\|extrapolate` | `MELEE_INTERP_MODE` / `interp_mode` | interpolate, extrapolate | interpolate |
| `--frame-latency 1\|2\|3` | `MELEE_FRAME_LATENCY` / `frame_latency` | 1..3 | 2 |

Alt+Enter toggles fullscreen at any time (to the configured kind, or borderless). Exclusive picks the
monitor's largest video mode, then the highest refresh. The content keeps its aspect: the presenter
letterboxes or pillarboxes it inside the window.

Example `settings.cfg`:

```
widescreen=1
fullscreen=borderless
render_scale=auto
msaa=4
vsync=on
```

## Widescreen, camera mode (`widescreen_mode=camera`, the C build's way)

This follows the C build's view policy (`pc/platform/shim_vi.c` `gw_video_sync_viewport_policy`,
`gw_view_math.h`). Here it lives in `melee_engine/src/view.rs`.

- It applies only in the wide scenes, `gw_view_scene_wide` (GameSceneKind 2 VS, 3 sudden death,
  4 training, 44, 46). The title, the menus, the CSS, the SSS and the results keep their authored
  4:3 and are pillarboxed. Hero's Source Port does the same: "native 2D main menus retain their
  authored proportions".
- Content aspect: the window's, clamped to 4:3..32:9 (`gw_view_aspect`). A 4:3 window with
  widescreen on stays 4:3.
- Perspective cameras: `HSD_CObjUpdateView` scales each authored aspect by
  `View_ProjectionScaleBits` = `aspect * 60/73` (Hor+; 16:9 gives 320/219).
- The EFB target is widened to `480*scale*aspect` x `480*scale`. GX viewports and scissors (logical
  640x480) are stretched across it in x, as Aurora's `AURORA_VIEWPORT_ASPECT` does
  (`melee_gx::render::Renderer::set_target`). Ortho 2D therefore stretches as it does in C. The
  match HUD (damage, stock icons, timer) draws through perspective cameras, so it stays centred
  and keeps its proportions. `HSD_SisLib_SetWideCanvas` text canvases widen with
  `View_AspectBits`.
- The simulation does not read any of this. The C keeps logic on the authored aspect
  (`Camera_LogicToScreen`, `HSD_CObjGetAuthoredAspect`), and the headless engine (goldens, gate)
  installs its own fixed values and never these hooks (`engine.rs` `boot_host`).
  The FD and Stadium shots taken at the same tick with 4:3 and 16:9 show identical damage, timer
  and positions.

## MSAA

The EFB pass draws into a multisampled colour target, which resolves into the single-sample EFB at
the end of every pass. EFB copies, the blit and screenshots therefore read the resolved image.
Depth copies (`GX_TF_Z*`) read sample 0 of the multisampled depth. If the adapter does not support
the requested count for both Rgba8Unorm and Depth32Float, the next lower count is used (logged as
`gx: MSAA ...`).

## Render scale

There are N x 480 lines internally (8 at most, further capped by the device's 2D texture limit).
`auto` picks the smallest scale that covers the window, up to 4. The log line
`gw: video: render scale N (auto|MELEE_RENDER_SCALE), MSAA Nx` reports the choice.

## Testing

- `MELEE_WINDOW_UNFOCUSED` (every test run) disables fullscreen. Logged as "fullscreen ignored",
  so an automated run never takes the user's screen. Fullscreen and Alt+Enter are covered by unit
  tests only (`melee_platform` `display_tests`: parsing, the toggle, the exclusive mode choice). A
  visible check needs a person.
- Unit tests: `melee_engine` `view::tests` (the C view maths: the cases in
  `pc/tests/view_aspect_cases.h`) and `melee` `display::tests` (precedence and the settings round
  trip).

## Unlocked frame rate and pacing (port/display2)

The simulation stays at exactly 60 Hz (`FixedStep`, integer ns). Rendering is decoupled
(`melee_platform/src/main_loop.rs`):

- **vsync** off: Immediate (else Mailbox) present; on: FIFO.
- **fps cap**: presents on a fixed grid (`next_present` + period). Between slots the loop sleeps (`sleep_until`: a
  timer sleep, then yields for the last ~1 ms), or wakes earlier when a tick is due.
- **VRR**: with vsync on and `vrr=auto`, the first 6 s run at 0.75 x refresh and read DXGI's frame statistics of
  wgpu's own D3D12 swapchain (`Presenter::frame_stats`: PresentCount against SyncRefreshCount). A fixed display keeps
  refreshing at its full rate for fewer frames; a variable one refreshes once per frame (`judge_vrr`, two equal
  verdicts in a row). Detected (or `vrr=on`): frames are capped at `hz * (1 - hz/3600)` (144 Hz: 138.2). The
  statistics need the D3D12 backend (`WGPU_BACKEND=dx12`). On Vulkan (the adapter wgpu picks on this machine) the
  probe ends with "fixed refresh assumed". This machine's 144 Hz panel, windowed, measured fixed:
  `VRR probe: 107.5 frames/s shown, 144.0 refreshes/s -> fixed`.
- **phase from the tick schedule**: the presenter gets the fixed-step clock's fraction at the time the present is
  *scheduled for*, not at the time the call happens to run, nor when the frame was handed over.
- **stale presents dropped**: a redraw that finds a tick due runs the tick first, since the in-between of the older
  pair would be shown after the frame it leads to exists. The exit line `gw: pacing: N presents dropped stale` counts them.
- `--frame-latency` sets the swapchain's maximum frame latency.

## Frame interpolation (render-only)

Capture (`melee_runtime/src/pose_capture.rs`) runs during the render pass, only when interpolation is on. The pobj
matrix setup (`SetupRigidModelMtx`, `SetupSharedVtxModelMtx`, `SetupEnvelopeModelMtx`) and `HSD_CObjSetCurrent` call
in after loading a matrix or making a camera current. They copy what the matrix was made of into a
`melee_core::pose::PoseFrame`: cameras; joints with their SRT, parent and world matrix; and per loaded slot either
`view * world(joint)` or the skinned `view * sum(w * world * envelope) * right`. They also tag the GX stream
(`GX_AURORA` 0x2000 POSE_TAG slot/record, 0x2001 camera). Each record is checked against the matrix the game actually
loaded; anything else stays untagged.

A joint is `Srt` only when rebuilding its matrix from its SRT with the game's own `HSD_MtxSRT` / `HSD_MtxSRTQuat`
reproduces `jobj->mtx`. Constraints, IK, user matrices and path-driven joints are `Opaque`: their local matrix is held.
Authored channels: each animated channel's `HSD_FObj` is **copied** and run forward with the module's own keyframe
interpreter (fobj.c's rules) at 0, 1/4 .. 1 of the next tick's frame rate. A channel counts as authored only if phase 0
lands bit-exactly on the joint's current value and the tick stays inside the animation (no loop or end in the window).
No game object is written and no game function is called. Capture is off in every headless run.

Solve (`melee_gx/src/interp.rs`): once per new frame pair, then per present at phase t.
- **interpolate** (default): between the last two ticks, one tick behind. A channel uses the previous tick's authored
  samples when they end where this tick actually is; otherwise it is interpolated (angles wrapped; steps over 30 units
  or 1.5 rad are held). **extrapolate**: the newest tick's authored samples forward, the rest extrapolated from the
  last delta with the same caps.
- Joints are rebuilt parent first; skinned slots are re-summed from the bones; the right-hand node matrix is rebuilt
  in the same three forms as `_HSD_mkEnvelopeModelNodeMtx`.
- **Cameras**: the eye moves on a line, the rotation on an arc (slerp); the projection is lerped (zoom), and a change
  over 25% is held. **Cut detection**: when the main camera (the perspective camera with the most tagged draws) moves
  over 120 units, turns over 1.2 rad, or is new, the pair is a cut and the newest frame is presented as is.
- Untagged perspective draws move with their camera only (`view' * inverse(view) * M`).
- **Held**: orthographic layers and, in a match, the HUD's own perspective cameras (every camera but the main one).
- Menus interpolate: their panels are animated joints under a perspective camera.

It never touches sim state or the hash (proof below).

## True widescreen (renderer side, default `widescreen_mode=renderer`)

The game's view hooks (`View_ProjectionScaleBits`, `View_AspectBits`) keep answering 4:3. The EFB target is widened to
the content aspect as before, and the presenter scales row 0 of each **perspective** draw's projection by
`(73/60) / aspect`: 219/320 at 16:9, since Melee's cameras are authored at 73:60 (cm/camera.c `1.2173333`).
Not widened:
- draws into a viewport smaller than the screen (render-to-texture: the Stadium screen, FoD's reflection);
- a frustum authored for a 4:3 screen fill (no off-centre shift, y/x scale exactly 4/3: lbbgflash's flash and wipes);
- backdrop planes whose vertices cover the whole 4:3 frustum (decoded from the draw: f32/s16/s8, direct or indexed
  positions, up to 64 vertices);
- corner-pinned perspective layers (off-centre shifts at +-1 on both axes);
- the orthographic HUD and 2D layers. The exception is the P1/P2 nametag layer (ifnametag.c's 640x480 ortho camera,
  near 0.1 / far 32768): it is placed from the widened world, so it is widened with it.

This applies to matches only (the scene policy above); menus stay 4:3.

The mod UI's window-space mode (`melee_gx/src/ui.rs`, the SPACE record) is untouched: the UI batch is still drawn after
the EFB blit at window size, and the `melee_modhost` ui tests pass.

## Checks (2026-11-04; Core Ultra 7 265F, RTX 5070, 144 Hz)

Determinism: `MELEE_HASH_LOG=<file>` writes the NHV1 hash of every match tick. Setup: FD, 4 CPUs at level 9, 1800
ticks. Three runs: interp off at 4:3; interp on with renderer widescreen 16:9, vsync off, cap 240; extrapolate with
widescreen. All three logs are identical (same file hash). `widescreen_mode=camera` was also identical over these 1800
ticks.

Frame pacing setup: FD, Fox/Marth/Falco/Peach CPU 9, 3600 ticks, vsync off, window 1280x960 (render scale 2), Vulkan.
Figures cover match ticks 300..3600, from `MELEE_FRAME_TIMES`:

| cap | interp | fps | present interval p50 / p99 (ms) | present call p50 / p99 | tick p50 / p99 |
|---|---|---|---|---|---|
| 60 | on | 60.0 | 16.67 / 17.42 | 1.43 / 1.95 | 1.54 / 2.30 |
| 144 | on | 144.0 | 6.94 / 8.76 | 1.44 / 3.37 | 1.56 / 2.16 |
| 240 | on | 240.0 | 4.17 / 6.22 | 1.19 / 3.47 | 1.61 / 2.28 |
| 60 | off | 60.0 | 16.67 / 17.35 | 2.77 / 3.59 | 1.24 / 1.93 |
| 144 | off | 144.0 | 6.94 / 8.43 | 0.50 / 3.41 | 1.25 / 1.88 |
| 240 | off | 240.0 | 4.16 / 5.86 | 0.34 / 3.23 | 1.26 / 1.84 |

Interpolation cost:
- capture: 0.25 ms per tick (420 joints, 1640 matrix slots a frame);
- per re-posed present at the 240 cap: evaluate p50 0.08 / p99 0.20 ms, uniform patching p50 0.06 / p99 0.20 ms.

The rest of the per-present difference is re-drawing the frame instead of re-presenting the last image. At a 60 cap,
interpolation re-poses almost nothing, because each present lands right after its tick; `interp=auto` turns it on only
above 65 fps. With vsync on (FIFO, 144 Hz): 144.0 fps, interval p50 6.90 / p99 9.78 ms.

Not verified: a VRR display (there is none here; detection was only seen to answer "fixed"), and exclusive fullscreen
pacing.
