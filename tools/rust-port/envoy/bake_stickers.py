#!/usr/bin/env python3
"""Envoy V3 (Party pack) sticker sprites for envoy_ui (docs/rust-port/notes/envoy-v3-ui.md).

Renders original vector art from the design generators in `menu/out_envoy/` (build_step*.py, build_final.py: our own
SVG, no disc data) with headless Chromium into PNGs the mod loads with `gd_ui_tex_load`. Every sprite is rendered at
4x its size in game pixels (640x480 units; the big flat ones at 2x) and the mod draws it at 1x: a game pixel is 4.5
device pixels at 4K, and the host's UI atlas is mipmapped, so the same art is sharp at 4K and clean at 720p
(uiquality; it was 2x, which was soft above 1080p).

Not baked any more: the sticker faces, the disc, the ink dot and the stock dots. The mod draws those as analytic
shapes (rust/mods/envoy_ui/src/kit.rs `face`, `spr`), exact at every size; the numbers there are the ones the tiles
had (face radius 10, 3 px ink edge centred on the rectangle, hard ink shadow offset (3, 4)).

  python tools/rust-port/envoy/bake_stickers.py [OUT_DIR]   (default rust/mods/envoy_ui/art)
Needs playwright (Chromium) and the design folder (GW_ENVOY_DESIGN, default E:/Projects/Melee Workspace/menu/out_envoy).
"""
import os
import sys
from pathlib import Path

DESIGN = Path(os.environ.get("GW_ENVOY_DESIGN", "E:/Projects/Melee Workspace/menu/out_envoy"))
sys.path.insert(0, str(DESIGN))
os.chdir(DESIGN)
from playwright.sync_api import sync_playwright  # noqa: E402

import build_step2 as s2  # noqa: E402
import build_step3 as s3  # noqa: E402
import build_step4 as s4  # noqa: E402
import build_step5 as s5  # noqa: E402
import build_final as bf  # noqa: E402

# V3 family ruling (coordinator, 2026-11-11): BLUE is Guard, drawn in the spare clay colour with a shield glyph
# (the "stone" badge row); the others keep their design glyphs (RED fire, PURPLE frost, YELLOW Sky = wind's gust,
# GREEN Tempo = technique's bullseye, WHITE utility).
_glyph = s4.glyph
s4.glyph = lambda f: s4._solid("M0,-9.5L8,-6.2V0.4C8,4.9 4.6,8 0,10C-4.6,8 -8,4.9 -8,0.4V-6.2Z") if f == "stone" else _glyph(f)

SUN, INK, WHITE, CORAL, LAGOON = s2.SUN, s2.INK, s2.WHITE, s2.CORAL, s2.LAGOON
svg = s2.svg
REPO = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else REPO / "rust/mods/envoy_ui/art"

S = {}  # name -> (w, h, inner html)


def add(name, w, h, inner):
    S[name] = (w, h, inner)


# ---- small pieces (the faces, discs and stock dots are analytic shapes in kit.rs)
add("coin", 16, 16, s5.coin(16))
add("star_on", 12, 12, s2.star_svg(12))
add("star_off", 12, 12, s2.star_svg(12, on=False))
add("spark", 14, 14, svg(14, 14, "-7 -7 14 14", f'<path d="{s4.SPARK}" fill="{SUN}" stroke="{INK}" stroke-width="1.5" stroke-linejoin="round"/>'))
add("lock", 14, 14, s3.lock(14))

# ---- bursts, rays, flame
for k, col in (("coral", CORAL), ("sunny", SUN), ("lagoon", LAGOON), ("white", "#fff")):
    add(f"burst_{k}", 58, 58, svg(58, 58, "-29 -29 58 58", f'<path d="{s2.BURST}" fill="{col}" stroke="{INK}" stroke-width="3.5" stroke-linejoin="round"/>'))
add("burst16", 40, 40, svg(40, 40, "-20 -20 40 40", "".join(
    f'<path transform="rotate({i * 22.5})" d="M-3,-6 L0,-19 L3,-6Z" fill="{SUN}" stroke="{INK}" stroke-width="1.2"/>' for i in range(16))
    + f'<circle r="8" fill="{SUN}"/>'))
add("rays", 200, 200, svg(200, 200, "-100 -100 200 200", "".join(
    f'<path transform="rotate({i * 20})" d="M-7,-100 L0,0 L7,-100Z" fill="{SUN}"/>' for i in range(18))))
for k, col in s5.HEATC.items():
    core = {"cold": "", "warm": "", "hot": f'<g transform="scale(.45) translate(0 14)"><path d="{s2.FLAME}" fill="{SUN}"/></g>',
            "blaze": '<g transform="scale(.45) translate(0 14)"><path d="' + s2.FLAME + '" fill="#fff"/></g>'}[k]
    fill = "#B9B2DE" if k == "cold" else col
    add(f"flame_{k}", 24, 30, svg(24, 30, "-40 -52 80 108", f'<path d="{s2.FLAME}" fill="{fill}" stroke="{INK}" stroke-width="9" stroke-linejoin="round"/>{core}'))

# ---- pointers and cursors, D-pad, emblem, no-disc mark
for k, col in (("coral", CORAL), ("lagoon", LAGOON)):
    d = "M3,9 H14 V3 L25,13 L14,23 V17 H3Z"
    add(f"ptr_{k}", 28, 26, svg(28, 26, "0 0 28 26", f'<path d="{d}" fill="{col}" stroke="#fff" stroke-width="6" stroke-linejoin="round"/>'
                                 f'<path d="{d}" fill="{col}" stroke="{INK}" stroke-width="2.5" stroke-linejoin="round"/>'))
    dd = "M9,3 V14 H3 L13,25 L23,14 H17 V3Z"
    add(f"ptrdown_{k}", 26, 28, svg(26, 28, "0 0 26 28", f'<path d="{dd}" fill="{col}" stroke="#fff" stroke-width="6" stroke-linejoin="round"/>'
                                     f'<path d="{dd}" fill="{col}" stroke="{INK}" stroke-width="2.5" stroke-linejoin="round"/>'))
add("dpad_down", 16, 16, s5.dpad("down", 16))
add("dpad_right", 16, 16, s5.dpad("right", 16))
add("dpad_right_lit", 16, 16, s5.dpad("right", 16, lit=True))
add("emblem", 30, 30, svg(30, 30, "10 4 30 30", f'<circle cx="25" cy="19" r="14.5" fill="{INK}"/><circle cx="25" cy="19" r="11.5" fill="#fff"/>'
                                                 f'<circle cx="21.4" cy="19" r="5" fill="{CORAL}" stroke="{INK}" stroke-width="2"/>'
                                                 f'<circle cx="28.6" cy="19" r="5" fill="{LAGOON}" stroke="{INK}" stroke-width="2"/>'))
add("nodisc", 22, 22, s2.disc_badge(22))
add("swap", 22, 22, svg(22, 22, "-11 -11 22 22", f'<circle r="9.5" fill="{SUN}" stroke="{INK}" stroke-width="2.4"/>'
                                                 f'<path d="M-5,-2 H4 L1,-5 M5,2 H-4 L-1,5" fill="none" stroke="{INK}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>'))
add("head", 22, 22, svg(22, 22, "-11 -11 22 22", f'<circle r="9.5" fill="#E9E6F5" stroke="{INK}" stroke-width="2.4"/>'
                                                 f'<circle cy="-2" r="3.6" fill="{INK}"/><path d="M-6,7 C-5,2 5,2 6,7Z" fill="{INK}"/>'))

# ---- the main menu's Envoy label (L3), 222 x 38
for st in ("idle", "on", "miss"):
    add(f"label_{st}", 222, 38, s2.label(3, st))

# ---- drive badges: 8 families x 3 tiers (Common, Rare, Epic), 32 px (drawn 19 to 58)
FAMS = ["fire", "bolt", "frost", "wind", "tech", "util", "stone", "rose"]
for i, f in enumerate(FAMS):
    for r in range(3):
        add(f"badge_{i}_{r}", 40, 40, s4.pog(f, 40 * 16 / 16, rar=r))

# ---- keystone emblems (white disc, 42 units) and the wedge faces
KEYS = ["Plague Bearer", "Echo Oath", "Grand Finale", "Long Fuse", "Winter Oath", "Glass Core"]
for i, k in enumerate(KEYS):
    add(f"kem_{i}", 34, 34, bf.kem(k, 34))
for k, face in (("idle", INK), ("on", SUN)):
    add(f"wedge_{k}", 186, 276, svg(186, 276, "-4 -4 194 284",
        f'<path d="M26,10 H170 L192,282 H8Z" fill="{INK}"/>'
        f'<path d="M22,2 H164 L186,274 H0Z" fill="{face}" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>'
        f'<path d="M26,8 H160 L180,268 H6Z" fill="none" stroke="{"#fff" if face == INK else INK}" stroke-width="2.5" stroke-linejoin="round"/>'))

# ---- backdrops (1x is enough: flat shapes)
add("bg_camp", 854, 480, s3.scene(1))
add("bg_road", 854, 480, s3.scene(2))


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    css = s2.CSS.replace('url("fonts/', f'url("{DESIGN.as_uri()}/fonts/').replace('url("../SourceSans3/', f'url("{DESIGN.parent.as_uri()}/SourceSans3/')
    body = "".join(f'<div id="{k}" style="position:absolute;left:0;top:0;width:{w}px;height:{h}px;display:none">{inner}</div>'
                   for k, (w, h, inner) in S.items())
    defs = ('<svg width="0" height="0" style="position:absolute"><defs><linearGradient id="holo" x1="0" y1="0" x2="1" y2="1">'
            + "".join(f'<stop offset="{i / 5:.2f}" stop-color="{c}"/>' for i, c in enumerate(s4.HOLO.split(","))) + '</linearGradient></defs></svg>')
    page_html = f'<!doctype html><html><head><style>{css} html,body{{background:transparent !important;width:auto}}</style></head><body>{defs}{body}</body></html>'
    tmp = OUT / "_bake.html"
    tmp.write_text(page_html, encoding="utf-8")
    with sync_playwright() as p:
        b = p.chromium.launch(args=["--force-color-profile=srgb"])
        for k, (w, h, _) in S.items():
            scale = 1 if k.startswith("bg_") else 2 if k.startswith("wedge_") or k == "rays" else 4
            pg = b.new_page(viewport={"width": max(w, 16), "height": max(h, 16)}, device_scale_factor=scale)
            pg.goto(tmp.as_uri())
            pg.evaluate("document.fonts.ready")
            pg.evaluate(f"document.getElementById('{k}').style.display='block'")
            pg.locator(f"#{k}").screenshot(path=str(OUT / f"{k}.png"), omit_background=True)
            pg.close()
        b.close()
    tmp.unlink()
    # backdrops at half size (flat shapes scale up cleanly; the shared UI atlas is 2048 square)
    from PIL import Image
    for k in S:
        if k.startswith("bg_"):
            im = Image.open(OUT / f"{k}.png")
            im.resize((im.width // 2, im.height // 2), Image.LANCZOS).save(OUT / f"{k}.png")
    print(f"baked {len(S)} sprites to {OUT}")


if __name__ == "__main__":
    main()
