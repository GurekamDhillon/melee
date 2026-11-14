# Envoy effects layer (port/fx)

> 2026-11-13 (port/envoy-vfx): the flash, the Heat screen treatment and the screen-wide rings described below are
> retired, and every effect shape is a shader quad now. See `envoy-vfx.md`; this note still describes the drive
> materials, the bloom, the UI quad effects and the menu camera.

A small, fixed set of GPU effects for Envoy, built from `scratch/envoy-notes/shader-effects-study.md`. Mods pick
built-in shaders by id; they never ship WGSL. Everything is presentation only.

## What it is

| part | where | what the player sees |
|---|---|---|
| UI quad effects | `melee_gx/src/ui.rs` (`UiQuad::fx`, `FX_*`), `melee_modhost/src/ui/cmd.rs` (`OP_EFFECT`), `nodes.rs` | Epic stickers shimmer with a foil sweep; Unique (tier 3, no family) get a holographic face with rainbow bands and glints; the hovered sticker has a soft breathing halo (the host's pre-blurred glow slice) |
| post | `melee_gx/src/fx.rs` (`POST_WGSL`), fused into `Renderer::blit_to` before the UI batch | Heat: nothing at Cold, warm vignette at Warm, ember tint at Hot, edge heat-haze with rising embers at Blazing; a cash-out sends a gold ring out from the player with a short flash and colour split; a boss break a larger shockwave |
| world layer | `melee_gx/src/fx.rs` (`WORLD_WGSL`), `Renderer::fx_world` at the end of `draw_prepared` | drives as real materials: glass with a fresnel rim, an inner glow that breathes faster with charge and brightens with the wearer's Heat, iridescent foil on Epic/Unique drives, dissolves for a drop's fly-in and pickup, light trails behind orbiters; the fire shapes too |
| bloom | `BLUR_WGSL`, composited in the post blit | the drives' emissive output only, at half resolution |

Rules kept:
- **The sim never sees it.** Inputs are the published list (`drive_draw::SHOW`), the fight camera copied by the
  fighter pass hook (`drive_draw::record_camera`: a static copy of the cobj view matrix, `GXGetProjectionv`,
  `GXGetViewportv`; no GX commands, no allocation) and the post queue. Time is a render clock. The old
  `drive_draw::draw`/`emit` (GX immediate mode inside the game's fighter pass, three `Vec`s a call) is gone: no drive
  or fire goes through the game-scope FIFO any more.
- **Off means today's frame.** `Renderer::fx` is `None` until a run arms it (`envoy_fx.rs`: Envoy armed, a drive
  queued, or a post queued); disarmed or armed with nothing active, the blit uses the plain pipeline and the world
  pass is skipped. UI effect 0 is the old fragment path.
- **Readability.** Every post weight is masked to zero in the centre ellipse (60% of each axis), their sum capped at
  15% x intensity; flashes at most 3 a second.
- **Prewarm.** `fx_arm` creates every buffer and pipeline (world for the current MSAA, post for each swapchain format
  seen and RGBA8) at once.

Intensity: `MELEE_FX` = `off` | `subtle` | `full` (default full offline, subtle in netplay). Off: no quad effects,
no post, no bloom (materials keep their base look). A settings entry can set the same value later.

## Imports (documented in `docs/specs/gd-ui-lab-v1.h`)

- `gd_ui_node_effect(node, effect, rgba)`; immediate drawing: the `OP_EFFECT` record (`gd_ui_enc::Enc::effect`).
  0 none, 1 foil, 2 holo, 3 glow. Text is never affected.
- `gd_fx_post(effect, strength, x, y, rgba)`: 1 vignette, 2 ember, 3 heat-haze (held: re-post each present); 4 flash,
  5 radial ring, 6 colour split, 7 shockwave (one-shots); `| 0x100` world position. At most 4 a present.
- `gd_ui_world_model_ex(model, x, y, z, yaw, scale, alpha, material, p0..p3, rgba)`: 0 body, 1 glass, 2 glow,
  3 foil, 4 dissolve, 5 unlit additive, `| 0x80` trail. p0 glow, p1 charge, p2 Heat state, p3 rarity / dissolve.
  96-instance cap as before. `gd_ui_world_model` still works (body; model 6 becomes glass).

The glass material json (`drive_glass.material.json`) is wired: opacity caps the shell's alpha, roughness sets the
fresnel exponent, tint scales the glass colour (`fx::glass_params`).

## Mod side (minimal, behind helpers)

`envoy_ui/src/fx.rs` (new): `heat_screen`, `moments` (cash-out, boss break; each event once), `model`,
`drop_model` / `drop_ghosts`. `kit::sticker` glows the hovered face; `kit::face_fx_next` + `pick::rarity_fx` give
Epic / Unique offers their face effect; `lib.rs` calls them (three call sites and the orbiter / drop draws).
envoy_sim is untouched.

## Ink edge on drive bodies

Material flag `MAT_OUTLINE` (0x40) on `gd_ui_world_model_ex`: the body is first drawn pushed out along its
screen-space normals by 1.2 px at 640x480 (scaled with the internal resolution) in Party-pack INK, depth-writing,
then the body over it at equal depth, so only the rim stays: a constant-width edge at any size (no shimmer at
21-26 px). Tag bodies only (not tier rings or the ascended overlay, DRIVE-MODELS.md section 3); additive draws never
get ink. The drive models' drawer (port/drvmodels) sets it; port/fx merges first.

## Menu camera (C-stick tilt)

Retail (`mn/mnmain.c` `mn_8022BA1C`): the first controller whose C-stick is outside the 0.4 dead zone orbits the
menu camera about the interest point, up to 30 degrees per axis, linear past the dead zone, no smoothing. Ours
(`melee/src/envoy_fx.rs` `MenuCam`, `ui.rs` vertex shader): the same range and dead zone from the larger deflection
of any controller (the pads the UI host already copied for the present: display only), a spring-damper (zeta 0.42:
lags, overshoots a little, wobbles back), the batch drawn as a plane seen from the turned eye with parallax by draw
order (backdrop deepest, cursor nearest; hard shadows shift), a full-band backdrop overscanned so no black shows at
full turn. At rest the transform is skipped: pixel-identical text. Intensity: Off none, Subtle half.
Applies to custom UI scenes (the camp, the party picker) and an armed run's reward, shop and between-fights screens;
not the fight HUD, not the settings overlay (its own batch); the main menu's Envoy stickers stay flat (retail turns
its own 3D menu; matching its transform for our quads is left for later). The pad script takes optional C-stick
columns: `<frames> <buttons> [sx sy [tl tr [cx cy]]]`.
