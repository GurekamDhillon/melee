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

## Widescreen: what it does (as the C build)

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
