# UI quality: causes and construction (port/uiquality, 2026-11-13)

The owner's report: render artifacts, squished and smudged UI pieces, micro tears in the seams, small circles that are
not circles. Two earlier rounds (uiseams 8d8b9f87a, 7b9e20efe) fixed the cases they targeted. This pass measured every
screen at 720p, 1080p, 1440p and 4K, in 4:3 and widescreen, and fixed the causes in the renderer and the kit.

## What was wrong

| # | Cause | Where | What it looked like |
|---|---|---|---|
| 1 | A sticker face was a 9-slice of a 48-unit tile with 14-unit corners, used for stickers lower or narrower than two corners (28 units). The corner tiles were pushed into each other. | `envoy_ui` `kit::face`: the P1 tag, name pills, the coin pill, tabs, hint caps, the "x2" badge, every small ink chip (11 to 24 units) | lumpy, pinched pills; doubled outlines; "squished" |
| 2 | Every rounded shape was a bitmap magnified by bilinear filtering: host slices and kit pieces at 1 texel per unit (magnified 1.5x at 720p to 4.5x at 4K), sticker art at 2 texels per unit (magnified above 960p band height). | `melee_gx::ui` atlas tiles; `bake_stickers.py` | soft, smudged edges and corners, worse the higher the resolution |
| 3 | The atlas had no mip chain. Minified art was box-filtered over at most 4x4 taps, which undersamples anything drawn more than 4x smaller than its texels (a 64-texel disc as a 4.5 px ring dot). | `ui.rs` fragment shader | small circles not round; dots of one ring differing from each other |
| 4 | A shape was up to nine quads. Their shared sides only stay closed while every piece is transformed identically and the pieces are found by the join pass (12 quads back, exact matches). | 9-slices rotated or under the menu tilt | hairline seams; fixed case by case before |
| 5 | Text with an ink edge was nine draws of the string (eight ink copies at offsets). The copies do not make a round outline and their antialiased edges stack. | `kit::text_edge`: titles, percent, names | a soft, lumpy ink edge on text |
| 6 | Glyph distance fields were 24 texels per em: large text (a 40-unit percent at 4K is 180 px) is built from 24 texels, and small text minified the field without mips. | `ui_font.rs` | rounded corners on large text, uneven stems on small text |
| 7 | In a window narrower than 4:3 the band took the window's full height but only its width: non-square pixels. | `Renderer::blit_to` band | every shape squeezed sideways (only at such window shapes) |

Checked and not a cause: the blit pass is single-sampled and the UI is drawn at window resolution after the EFB
resolve, so MSAA and the render scale do not touch it; the atlas is premultiplied and the blend state is premultiplied
(no fringe); PNG texels and the target are both non-linear 8-bit and blended as such, which matches how the art was
drawn; widescreen keeps the band 4:3 with square pixels.

## How it is built now

**Analytic shapes.** `Q_SHAPE` instances (`ui.rs`, flags bits 16..19) carry no texture. The fragment shader evaluates
the signed distance to a rounded box (a pill or a circle when the radius reaches half the short side), a diamond or a
glow, in device pixels, and turns it into coverage one pixel wide. A border is a second distance inside the first, in
its own colour; a clear fill with a border is a ring; a dashed border is measured along the perimeter. One quad is the
whole shape at any size, so:

- there are no pieces, and so no seams, by construction, at rest, rotated and under the menu tilt;
- corners are true arcs at every resolution; nothing is magnified;
- a sticker smaller than its corner radius is a pill, not two corners overlapping.

Host slices (`OP_SLICE`) and the round kit pieces (`ui.dot`, `ui.ring`, `ui.knob`, `ui.diamond`) are now one shape
quad each, so every existing mod (the LAB, the menus) gets this without a change. Mods draw their own with
`OP_SHAPE` (gd-ui-v1.4, `Enc::shape`, `round_rect`, `circle`). The Envoy kit's `face`, `sticker`, `disc`, and the
disc / ink dot / stock dot sprites are shapes with the numbers of the tiles they replace.

**Pixel policy.** An unrotated, untilted shape with straight sides is snapped: origin and size land on whole device
pixels (the size is rounded on its own, so it does not change as the shape moves) and its border is a whole number of
pixels. Straight edges are crisp and a border of one width is the same number of pixels everywhere. Circles, rotated
and tilted shapes are not snapped: they are pure coverage. Flat and textured quads keep the rasterizer's rule (an
edge lands on the nearest pixel boundary), which is watertight between abutting quads; rotated ones keep the uiseams
edge ramp with inner sides marked.

**Atlas.** 4096 x 4096, premultiplied, with 4 mip levels built on the CPU when an entry is written (at load, not at
present). Every entry starts on an 8-texel boundary and is followed by an 8-texel clear gutter, so no mip texel mixes
two entries. Sampling is trilinear, half a level sharp. A quad that shows a whole entry (`Q_FULL`) reaches one pixel
past its rect and may filter into the gutter, so minified art fades out at its edge instead of being cut. The CPU
page is allocated by rows as the shelves fill.

**Art.** `bake_stickers.py` renders at 4x (was 2x): 1:1 or better up to 1440p, 0.9 texel per pixel at 4K. The faces,
disc and dots are no longer baked.

**Text.** 48 texels per em, 6 texels of spread, the same 96 px supersampled raster as before (same build time);
minified through the mips. `OP_OUTLINE` draws an outline pass from the glyphs' own distance field under the fill:
two draws of the string instead of nine, a round outline.

**Band.** `ui::band_in`: the largest centred 4:3 rect of the blit rect, square pixels at every window shape.

## Checks

- `cargo test -p melee_gx --test ui_quality` renders on the real pipeline, offscreen, at the band scales of 720p,
  1080p, 1440p and 4K, and reads the pixels back:
  - circles and rings from 2 to 32 units: area within 2% of pi r^2 (10% under 6 px across), width equals height
    within 2% (5% under 6 px), centroid within 0.08 px, every edge pixel within 0.3 px of the circle;
  - small pills: mirror-symmetric within 3% coverage, straight sides exactly on pixel boundaries;
  - seams: no partly covered pixel between fully covered ones, for a panel, a pill, a sticker (shadow under face)
    and a three-piece flat bar, at rest, rotated 0.6 and 8 degrees, tilted, rotated and tilted;
  - a 128-texel sprite disc drawn down to 18 px stays round (the mip chain covers 8x minification).
- `cargo test -p melee_modhost --lib ui::`: a slice is one shape quad at any size; `OP_SHAPE` / `OP_OUTLINE` records.
- Screens: `scratch/morning/uiquality/index.md` (local): each defect with a before / after crop.

## Limits

- Art minified more than 8x reaches past the mip chain (4 levels). Nothing in the kit is drawn that small.
- Sprite-sheet cells that abut with no gutter (the drive model sheets) can pick up a neighbouring cell's edge when
  minified. Their cells are clear at the edges today.
- The first present with UI still uploads the whole used atlas once (about 13 ms, as before this change).
