# Envoy shader effects (port/envoy-vfx)

Every Envoy effect is a quad shaded by a built-in procedural fragment shader. No flat coloured polygons, no
full-screen flash, no full-screen Heat treatment. Presentation only.

## The pieces

| part | where |
|---|---|
| the shaders, the instanced pipeline, the offset (refraction) target | `rust/melee_gx/src/vfx.rs` |
| the passes (world pass, bloom, blit) the quads ride in | `rust/melee_gx/src/fx.rs` (`fx_world`, `POST_WGSL`) |
| the host import `gd_vfx_batch` | `rust/melee_modhost/src/ui/mod.rs` |
| the presenter hand-over | `rust/melee/src/envoy_fx.rs` |
| the mod's effect API and recipes | `rust/mods/envoy_ui/src/vfx.rs` |
| the items, pedestals, pick-up | `rust/mods/envoy_ui/src/items_ui.rs` |
| the combo breaker, cash-out, boss break | `rust/mods/envoy_ui/src/fx.rs` (`moments`) |
| drive fires, statuses, tag pops, the stage rift | `rust/mods/envoy_ui/src/lib.rs` |
| the gallery (tune a shader without the game) | `rust/melee_gx/tests/vfx_gallery.rs` |

## How a quad is drawn

`VfxInst` (64 bytes): centre, rotation, size, age in seconds, seed, four numbers, two colours, kind, flags. One
instanced draw for all of a present's quads, inside the effects layer's world pass: after the GX frame's last EFB
pass, into the EFB colour target and the emissive target the bloom blurs. World quads stand in the stage plane at
their own depth and are depth-tested against the scene (no depth write), so a fighter in front of an effect stays in
front; `BACK` (z -9) is behind the fighters, `FRONT` (z +9) in front. `F_SCREEN` quads are placed on the 640x480
canvas (the rift). Blending is premultiplied: light adds, solid parts (ice, a mine, a coin) cover. Light past full
brightness runs to white, so cores are white-hot without clipping to a flat colour.

Refraction: quads flagged `F_DISTORT` are drawn a second time into a half-resolution RG16F offset target; the blit
samples the EFB through it with a slight colour fringe. It is local to the quad (a shockwave's ring, the air over
fire, the rift); the old screen-wide ring posts are gone.

Time is the age the mod passes (frames / 60 from the sim's view, or the mod's present counter for loops): there is
no clock in the shader, so a frame's picture depends only on the view. Lightning is redrawn from `seed` and
`floor(age * rate)`, so it jitters every other frame and a capture of the same frame is the same picture.

## Kinds

| id | kind | p0 | p1 | p2 | p3 |
|---|---|---|---|---|---|
| 1 | GLOW soft light | falloff | hot core | | |
| 2 | SHOCK ring running out | progress | width | electric crackle | refraction |
| 3 | BOLT lightning along x | jaggedness | branches | redraws / s | thickness |
| 4 | CIRCLE magic circle | drawn-in | cracks | charge | spin |
| 5 | FLAME fire on the bottom edge | height | lean | heat | |
| 6 | EMBERS rising sparks | density | speed | | |
| 7 | ICE shard cluster | growth | shatter | | |
| 8 | TRAIL comet streak | wobble | head size | | |
| 9 | SPARKS impact burst | progress | count | gravity | spread |
| 10 | STAR four-point sparkle | sharpness | spin | second cross | |
| 11 | WISP spectre | facing | strike flash | | |
| 12 | SLASH crescent | progress | arc span | thickness | |
| 13 | CRACK split ground | spread | glow | dust | |
| 14 | BEAM pillar of light | core | motes | | |
| 15 | ORB energy ball | plasma | crescent | pulse rate | |
| 16 | DOME hexagonal shield | progress | cells | | refraction |
| 17 | MINE spiked charge | light | warning ring | armed | |
| 18 | COIN turning coin | turns / s | sparkle | | |
| 19 | HAZE heat shimmer (refraction only) | strength | | | |
| 20 | AURA flames round a fighter | strength | | | |
| 21 | RING thin standing ring | radius | line width | dashes | turn rate |

## Adding an effect (the mod API)

```rust
use crate::vfx;
// a ring that crackles and bends the air, a pop of light, sparks: what Spark Ring's burst is made of
vfx::burst_ring(x, y, radius, k, colour, 1.0, age);
vfx::pop(x, y, 7.0, age, 6.0, WHITE);
vfx::sparks(x, y, radius, k, 20.0, colour, seed);
// or a raw quad
vfx::fx(vfx::ICE, x, y + h * 0.5, w, h).cols(c0, c1).p(grow, shatter, 0.0, 0.0).age(age).seed(seed).emit();
// lightning between two points
vfx::bolt(x0, y0, x1, y1, width, age, life, colour, seed);
```

`Fx::emit` applies the pause overlay's options: effect intensity scales the opacity (Off queues nothing), screen
shake Off drops every refraction. `vfx::flush()` runs once at the end of `gd_ui_frame`. Rules for a new item: one
silhouette and one colour pair of its own, its own sound, and `vfx::impact` (or sparks) where it hits.

## What was removed

- `POST_FLASH`, `POST_EMBER`, `POST_HEAT_HAZE`, `POST_RADIAL_WAVE`, `POST_CHROMA`, `POST_SHOCKWAVE`: accepted by
  `gd_fx_post` for old mods, never drawn. The blit's post shader keeps only the bloom composite, the refraction
  and one edge vignette (the combo breaker's pulse in the player's colour, 12 frames, capped at 15% x intensity,
  never over the centre ellipse).
- The UI rectangles that flashed the screen white: the stage shift's switch flash, the rift's hand-off flash and
  its 20% veils.
- `gd_ui_world_fx` is no longer called by envoy_ui (the import stays for other mods).

## Readability and cost

Effects in front of fighters are light (additive) and short; solid kinds sit behind or beside. A quad's light
covers what is behind it only where it is bright, which keeps colours true on a bright stage.
Measured (2026-11-14, this machine, `vfx_cost` in `tests/vfx_gallery.rs`: 168 quads, 8 of every kind, GPU waited):
+0.23 ms a frame at 1280x960 internal, +0.43 ms at 1920x1440 (1080p output), +0.99 ms at 3200x2400 (4K output); that
is the world pass, the bloom, the refraction pass and the blit together. Real-time 1080p runs with the frame
profiler (every effect in turn, and an all-items fight): 0 frames over 16.7 ms in 2400 after the match load.

Pre-warm: the effect pipelines (one shader module, two colour pipelines, one refraction pipeline, beside the drive
material ones) are built on the third present of the process whenever a UI mod is loaded (`envoy_fx.rs`,
`play: envoy fx: armed (.. ms .., pre-warmed at start-up)`), so a fight never compiles one; 215-250 ms the first
time on a machine, 15-19 ms once the driver has them cached. `tools/rust-port/perf/pipe_seed.sh` runs every item
in its Envoy pass and checks that line.

Development aids: `MELEE_VFX_WGSL=<file>` replaces the shader source without a rebuild; `MELEE_VFX_LOG=1` prints
the effect kinds on screen whenever they change; the gallery test renders a spec of quads over a backdrop to PNGs.
