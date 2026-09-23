"""Geno Lab UI art, in the kit's style and on the kit's pipeline (private: Geno build only).

    python lab_art.py                          build everything, convert, check, preview
    python lab_art.py --install-menu-icon DIR  also copy ico_lab.gxtex into DIR (a run's ui/)

Writes (paths relative to this folder):
    out/2x/*.png, out/1x/*.png   PNG-32 straight alpha, sRGB-tagged (the root pipeline's save_png)
    out/lab_ui.json              every texture: size, GX format, draw size, slices, tints, use
    ../ui/*.gxtex                the 2x textures as GX data (pc/tools/png2gx.py --layout), plus
    ../ui/lab_ui.json            ... the manifest, where gd.kit loads them by name
    preview/lab_sheet.png        ONE composed review sheet (Lab art beside the kit's own) - GD's

Provenance: every pixel comes from the SVG / HTML / CSS in this folder, rendered by headless
Chromium. Nothing is traced, sampled or referenced from Melee or any other game. The root
pipeline (menu/pipeline) is imported by path for its shared helpers and is never modified.

Kit rules kept: white-on-alpha masks tinted by material colour (I4) for everything
single-colour; RGB5A3 only for the multi-colour 9-slice; POT, <= 1024, authored at 2x and drawn
at 1x size; strokes and gaps >= 5 grid units and holes >= 6 on the 64 grid; no letters except
where the letter is the symbol (the '?'); words and key labels come from the font atlas.
"""
import argparse
import io
import json
import os
import shutil
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont
from playwright.sync_api import sync_playwright

import lab_palette as LP                                  # also puts menu/pipeline on sys.path
import icons as I                                         # noqa: E402  (root pipeline, read-only)
from build import downscale_half, save_png               # noqa: E402
import quantise_check as QC                               # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "out")
PREVIEW = os.path.join(HERE, "preview")
MOD = os.path.dirname(HERE)
UI = os.path.join(MOD, "ui")
MENU = LP.MENU
PNG2GX = None
for cand in (os.path.join(MOD, "..", "..", "..", "tools", "png2gx.py"),
             os.path.join(MENU, "..", "melee", "pc", "tools", "png2gx.py")):
    if os.path.exists(cand):
        PNG2GX = os.path.abspath(cand)
        break

P, S, circle, ring, rect, rrect, star = I.P, I.S, I.circle, I.ring, I.rect, I.rrect, I.star

# ============================================================ drawings
# name -> dict(kind, w, h (2x px), min_draw_1x, grid (viewBox w, h), body | shape+knock,
#              use, group, tint)
ART = {}


def mask(name, group, w, h, min_draw, use, body=None, shape=None, knock=None, grid=None,
         tint="bone"):
    ART[name] = dict(kind="mask", group=group, w=w, h=h, min_draw_1x=min_draw, use=use,
                     body=body, shape=shape, knock=knock, grid=grid or (64, 64), tint=tint)


def ico(name, use, min_draw=16, **kw):
    """A Lab control icon: 64x64 @2x on the kit's 64 grid, like ico_host / ico_paste."""
    mask("ico_lab_" + name, "icon", 64, 64, min_draw, use, **kw)


def arc(cx, cy, r, a0, a1):
    """SVG arc path from angle a0 to a1 (degrees, 0 = +x, clockwise on screen)."""
    import math
    x0, y0 = cx + r * math.cos(math.radians(a0)), cy + r * math.sin(math.radians(a0))
    x1, y1 = cx + r * math.cos(math.radians(a1)), cy + r * math.sin(math.radians(a1))
    large = 1 if (a1 - a0) % 360 > 180 else 0
    return "M%.2f %.2f A%g %g 0 %d 1 %.2f %.2f" % (x0, y0, r, r, large, x1, y1)


WHITE = '<g fill="#fff">%s</g>'

# ---- transport
ico("play", "resume (P while paused)", body=P("M14 6 L58 32 L14 58 Z"))
ico("pause", "pause (P while running)", body=P(rect(10, 6, 16, 52)) + P(rect(38, 6, 16, 52)))
ico("step", "step 1 frame forward (N; hold = slow play)",
    body=P("M4 8 L40 32 L4 56 Z") + P(rect(46, 8, 12, 48)))
ico("step_back", "step 1 frame back (B; hold = slow rewind)",
    body=P(rect(6, 8, 12, 48)) + P("M60 8 L24 32 L60 56 Z"))
ico("rewind", "rewind (hold B / CTRL+B = 10 frames back)",
    body=P("M32 8 L2 32 L32 56 Z") + P("M62 8 L32 32 L62 56 Z"))
ico("forward", "fast step (CTRL+N = 10 frames forward)",
    body=P("M2 8 L32 32 L2 56 Z") + P("M32 8 L62 32 L32 56 Z"))
ico("slowmo", "slow play: a snail",
    body=(P(ring(38, 29, 22, 15.5), "evenodd") + P(circle(38, 29, 7)) +
          P("M2 60 V50 C2 42 6 38 12 38 C18 38 20 44 20 50 H62 V60 Z") +
          S("M9 39 L4 26", 5, "round") + S("M16 39 L17 26", 5, "round")))
ico("history", "step-back history (the snapshot ring): a clock with a back-arrow",
    body=(S(arc(34, 32, 23, 200, 470), 6) +
          P("M2 20 L20 20 L11 36 Z") + S("M34 18 V33 H45", 5.5)))
ico("record", "record / history on",
    body=P(ring(32, 32, 29, 23), "evenodd") + P(circle(32, 32, 15)))

# ---- overlays
ico("hitbox", "1: hitboxes (a burst; the hurtbox is the capsule)",
    shape=P(star(32, 32, 31, 20, 10)), knock=P(circle(32, 32, 10)))
ico("hurtbox", "1: hurtboxes (a capsule around a bone)",
    body=S(rrect(17, 3, 30, 58, 15), 6) + P(rect(29.5, 17, 5, 30)))
ico("ecb", "5: ECB (the diamond)",
    body=S("M32 5 L59 32 L32 59 L5 32 Z", 6) + P(circle(32, 32, 6)))
ico("stage", "6: stage collision lines (cycles lines / ledges / terrain)",
    body=(S("M2 42 H14 L22 58 H42 L50 42 H62", 5.5) + P(rect(18, 14, 28, 6)) +
          P(circle(14, 42, 5.5)) + P(circle(50, 42, 5.5))))
ico("ledge", "6: ledges (a stage corner and its grab box)",
    body=P(rect(2, 34, 30, 28)) + S(rect(30, 12, 28, 28), 5))
ico("model", "2: model on/off (a figure)",
    body=(P(circle(32, 11, 9)) +
          P("M14 60 L19 30 C20 25 24 23 28 23 H36 C40 23 44 25 45 30 L50 60 H39 L35 42 H29 "
            "L25 60 Z")))
ico("skeleton", "3: skeleton (joints and bones)",
    body=(S("M32 16 V38 M13 32 L32 23 L51 32 M19 58 L32 38 L45 58", 5) +
          P(circle(32, 9, 7)) + P(circle(13, 32, 5)) + P(circle(51, 32, 5)) +
          P(circle(32, 38, 5)) + P(circle(19, 58, 5)) + P(circle(45, 58, 5))))
ico("joints", "4: joint numbers (a joint with a label tag)",
    shape=(S("M4 62 L14 46", 5, "round") + P(circle(15, 45, 9)) + S("M15 45 L28 28", 5) +
           P("M30 6 H62 V32 H30 L22 19 Z")),
    knock=P(circle(32, 19, 3.5)))
ico("hitlabels", "9: hitbox labels (a hitbox with a label tag)",
    shape=P(circle(20, 44, 18)) + P("M36 4 H62 V28 H36 L29 16 Z"),
    knock=P(circle(20, 44, 7)) + P(circle(38, 16, 3.5)))
ico("timeline", "M: move timeline (track, ticks, playhead)",
    shape=(P(rect(2, 24, 60, 16)) + P(rect(2, 46, 5, 12)) + P(rect(17, 46, 5, 8)) +
           P(rect(32, 46, 5, 12)) + P(rect(47, 46, 5, 8))),
    knock=P(rect(36, 19, 16, 45)) +
    WHITE % (P("M34 2 H54 L44 14 Z") + P(rect(41.5, 12, 5, 34))))
ico("info", "7: info panel (the info mark)",
    shape=P(circle(32, 32, 30)), knock=P(circle(32, 17, 5.5)) + P(rect(27.5, 27, 9, 23)))
ico("log", "8: event log (a console)",
    shape=P(rrect(2, 4, 60, 56, 5)),
    knock=P(rect(8, 12, 48, 42)) +
    WHITE % (S("M13 20 L21 27 L13 34", 5) + P(rect(26, 29, 24, 5)) + P(rect(13, 42, 37, 5))))
ico("attrs", "0: attribute compare (paired bars)",
    body=(P(rect(2, 28, 9, 32)) + P(rect(16, 12, 9, 48)) +
          P(rect(37, 36, 9, 24)) + P(rect(51, 4, 9, 56))))
ico("lockstep", "C: lock-step compare (two tracks on one frame)",
    shape=P(rect(2, 10, 60, 14)) + P(rect(2, 40, 60, 14)),
    knock=P(rect(26, 4, 16, 56)) +
    WHITE % (P(rect(31.5, 2, 5, 60)) + P("M24 2 H44 L34 10 Z") + P("M24 62 H44 L34 54 Z")))
ico("mirror", "R: mirror P1's pad onto P2 (a pad reflected across an axis)",
    body=(P(rect(2, 28.5, 22, 7)) + P(rect(9.5, 21, 7, 22)) +
          P(rect(40, 28.5, 22, 7)) + P(rect(47.5, 21, 7, 22)) +
          "".join(P(rect(29.5, y, 5, 8)) for y in (2, 16, 30, 44, 56)) +
          P("M8 6 H22 L15 14 Z") + P("M42 58 H56 L49 50 Z")))
ico("focus", "TAB: focus the next fighter (brackets on a figure)",
    body=(S("M4 18 V4 H18", 5) + S("M46 4 H60 V18", 5) + S("M60 46 V60 H46", 5) +
          S("M18 60 H4 V46", 5) + P(circle(32, 24, 9)) + P("M17 50 C17 36 47 36 47 50 Z")))
ico("save", "F5: save state (into the slot)",
    body=S("M5 36 V58 H59 V36", 6) + P("M26 2 H38 V20 H48 L32 38 L16 20 H26 Z"))
ico("load", "F6: load state (out of the slot)",
    body=S("M5 36 V58 H59 V36", 6) + P("M32 2 L48 20 H38 V44 H26 V20 H16 Z"))
ico("help", "F3: help (the question mark is the symbol)",
    body=(S("M19 22 C19 6 45 6 45 22 C45 34 32 32 32 44", 8) + P(circle(32, 55, 6))))
ico("power", "X: Lab on / off",
    body=S(arc(32, 34, 23, -55, 235), 7) + P(rect(28, 2, 8, 30)))
ico("slash", "overlay on any Lab icon when its toggle is OFF (tint: off); colour is never the "
    "only cue", body=S("M6 58 L58 6", 6, "square"), tint="off")
ico("slash_gap", "drawn first under ico_lab_slash in the backdrop colour (glass), so the "
    "slash cuts the icon", body=S("M6 58 L58 6", 16, "square"), tint="glass")

# ---- the LAB menu icon (SOLO > LAB), hero-capable like the kit's hub icons
mask("ico_lab", "menu", 256, 256, 16,
     "SOLO > LAB menu tile, LAB breadcrumb: a flask (replaces ico_training on that entry)",
     shape=(P(rect(18, 2, 28, 7)) +
            P("M22 8 H42 V24 L60 55 C62 59 60 62 55 62 H9 C4 62 2 59 4 55 L22 24 Z")),
     knock=(P("M27 9 H37 V26 L50 47 H14 L27 26 Z") + P(circle(25, 54, 3.5)) +
            P(circle(39, 53, 4))))

# ---- timeline markers (colour AND shape per event type), 32x32 @2x = 16 @1x
def mark(name, use, **kw):
    mask("lab_mk_" + name, "marker", 32, 32, 12, use, tint=name, **kw)


mark("iasa", "IASA: interruptible from this frame (a flag)",
     body=P(rect(8, 2, 7, 60)) + P("M15 2 L60 16 L15 31 Z"))
mark("invinc", "body / hurtbox state change (a shield)",
     body=P("M32 2 L59 12 V30 C59 46 47 56 32 62 C17 56 5 46 5 30 V12 Z"))
mark("gfx", "GFX spawned (a spark)", body=P(star(32, 32, 31, 10, 4)))
mark("sfx", "SFX played (a speaker)",
     body=P("M2 22 H17 L36 5 V59 L17 42 H2 Z") + S("M45 18 Q57 32 45 46", 6))
mark("vis", "model visibility change (an eye)",
     shape=P("M1 32 C14 8 50 8 63 32 C50 56 14 56 1 32 Z"),
     knock=P(circle(32, 32, 14)) + WHITE % P(circle(32, 32, 7)))
mark("hitbox", "legend swatch for a hitbox window (windows themselves are flat quads)",
     body=P(rrect(2, 14, 60, 36, 6)))

# ---- timeline track pieces (grid = 2x px)
mask("lab_tl_playhead", "timeline", 32, 64, 16,
     "the frame shown: head + 2px stem; tint playhead (gold), draw once in ink at +1,+1 1x first",
     body=P("M3 2 H29 L16 17 Z") + P(rect(14, 14, 4, 50)), grid=(32, 64), tint="playhead")
mask("lab_tl_cap_l", "timeline", 16, 32, 16,
     "track left end, slanted at the kit shear (0.25); the track body is a flat quad",
     body=P("M8 0 H16 V32 H0 Z"), grid=(16, 32), tint="track")
mask("lab_tl_cap_r", "timeline", 16, 32, 16, "track right end (see cap_l)",
     body=P("M0 0 H16 L8 32 H0 Z"), grid=(16, 32), tint="track")

# ---- key-hint chip: two-layer keycap, 3-sliced (l + flat centre + r); the label is set from
# the font atlas (caption role, ink) on the top face. One-character keys: l + r, no centre.
KEY = dict(h_1x=16, cap_w_1x=8, top=(1, 12), radius_1x=3, lip_1x=4, side_1x=1)
mask("lab_key_l", "key", 16, 32, 16, "keycap body, left end (tint muted; accent when held)",
     body=P("M6 0 H16 V32 H6 A6 6 0 0 1 0 26 V6 A6 6 0 0 1 6 0 Z"), grid=(16, 32), tint="muted")
mask("lab_key_r", "key", 16, 32, 16, "keycap body, right end",
     body=P("M0 0 H10 A6 6 0 0 1 16 6 V26 A6 6 0 0 1 10 32 H0 Z"), grid=(16, 32), tint="muted")
mask("lab_key_top_l", "key", 16, 32, 16, "keycap top face, left end (tint bone)",
     body=P("M6 2 H16 V24 H6 A4 4 0 0 1 2 20 V6 A4 4 0 0 1 6 2 Z"), grid=(16, 32), tint="bone")
mask("lab_key_top_r", "key", 16, 32, 16, "keycap top face, right end",
     body=P("M0 2 H10 A4 4 0 0 1 14 6 V20 A4 4 0 0 1 10 24 H0 Z"), grid=(16, 32), tint="bone")

# ---- compact translucent panel: RGB5A3 9-slice (colour baked; the only non-mask art)
PANEL_PIECES = {
    "lab_frame_corner_tl": ("ch", 0), "lab_frame_corner_tr": ("sq", 90),
    "lab_frame_corner_br": ("ch", 180), "lab_frame_corner_bl": ("sq", 270),
    "lab_frame_edge_h": ("eh", 0), "lab_frame_edge_v": ("ev", 0), "lab_frame_fill": ("fl", 0),
}
for _n, (_cls, _rot) in PANEL_PIECES.items():
    ART[_n] = dict(kind="panel", group="panel", w=32, h=32, cls=_cls, rot=_rot, min_draw_1x=16,
                   use={"lab_frame_edge_h": "top edge; bottom = V-flipped; stretch on X",
                        "lab_frame_edge_v": "left edge; right = U-flipped; stretch on Y",
                        "lab_frame_fill": "centre; stretch both ways (or a flat glass quad)"}
                   .get(_n, "corner, per orientation (no UV flips needed)"))

# ============================================================ rendering
with open(os.path.join(HERE, "lab_style.css"), encoding="utf-8") as fh:
    CSS = fh.read()


def svg_body(a):
    gw, gh = a["grid"]
    if a["body"] is not None:
        return WHITE % a["body"]
    return ('<defs><mask id="k" maskUnits="userSpaceOnUse" x="-2" y="-2" width="%d" height="%d">'
            '<rect x="-2" y="-2" width="%d" height="%d" fill="#fff"/><g fill="#000">%s</g>'
            '</mask></defs><g fill="#fff" stroke-width="0" mask="url(#k)">%s</g>'
            % (gw + 4, gh + 4, gw + 4, gh + 4, a["knock"], a["shape"]))


def render_mask(page, a):
    w, h = a["w"], a["h"]
    gw, gh = a["grid"]
    html = ('<!doctype html><html><head><style>html,body{margin:0;background:transparent}'
            '</style></head><body><svg id="s" width="%d" height="%d" viewBox="0 0 %d %d">%s'
            '</svg></body></html>' % (w, h, gw, gh, svg_body(a)))
    page.set_viewport_size({"width": max(w, 64), "height": max(h, 64)})
    page.set_content(html)
    png = page.locator("#s").screenshot(omit_background=True, type="png")
    img = Image.open(io.BytesIO(png)).convert("RGBA")
    out = Image.new("RGBA", img.size, (255, 255, 255, 0))
    out.putalpha(img.getchannel("A"))            # masks are white + alpha only
    return out


def render_panel(page, a):
    cls = a["cls"]
    inner = {"ch": '<div class="fill ink"></div><div class="fill glass"></div>'
                   '<div class="fill cap"></div>',
             "sq": '<div class="fill ink"></div><div class="fill glass"></div>',
             "eh": '<div class="ink"></div><div class="glass"></div>',
             "ev": '<div class="ink"></div><div class="glass"></div>',
             "fl": '<div class="glass"></div>'}[cls]
    body = ('<div class="el %s" style="width:%dpx;height:%dpx"><div class="rot" '
            'style="transform:rotate(%ddeg)">%s</div></div>' % (cls, a["w"], a["h"], a["rot"], inner))
    page.set_viewport_size({"width": 64, "height": 64})
    page.set_content('<!doctype html><html><head><meta charset="utf-8"><style>%s</style></head>'
                     '<body>%s</body></html>' % (CSS, body))
    png = page.locator(".el").screenshot(omit_background=True, type="png")
    return Image.open(io.BytesIO(png)).convert("RGBA")


# ============================================================ checks
def topo_ok(name, a, img, errs):
    """The root icon check: shapes and holes at full res survive a draw at 2x the smallest."""
    if a["w"] != a["h"]:
        return None
    full = I.topology(img.getchannel("A"))
    hd = I.topology(I.shrink(img, 2 * a["min_draw_1x"]))
    sd = I.topology(I.shrink(img, a["min_draw_1x"]))
    if hd != full:
        errs.append("%s: at %d px shapes/holes %s, full res %s" % (name, 2 * a["min_draw_1x"], hd, full))
    return dict(full=list(full), at_2x_draw=list(hd), at_1x_draw=list(sd))


def pot(n):
    return n > 0 and n & (n - 1) == 0


FORMAT = {"mask": "I4", "panel": "RGB5A3"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--install-menu-icon", metavar="DIR",
                    help="copy ico_lab.gxtex into DIR (e.g. a Geno run's <exe dir>/ui)")
    args = ap.parse_args()

    errs, rep = LP.check()
    if errs:
        print("palette checks failed:", errs)
        return 1

    for d in (os.path.join(OUT, "2x"), os.path.join(OUT, "1x")):
        if os.path.isdir(d):
            shutil.rmtree(d)
        os.makedirs(d)
    os.makedirs(PREVIEW, exist_ok=True)

    imgs, rows = {}, []
    with sync_playwright() as p:
        browser = p.chromium.launch(args=["--force-color-profile=srgb", "--disable-lcd-text"])
        page = browser.new_page(device_scale_factor=1)
        for name, a in ART.items():
            img = render_mask(page, a) if a["kind"] == "mask" else render_panel(page, a)
            if img.size != (a["w"], a["h"]):
                errs.append("%s rendered %s, want %dx%d" % (name, img.size, a["w"], a["h"]))
            imgs[name] = img
            save_png(img, os.path.join(OUT, "2x", name + ".png"))
            save_png(downscale_half(img), os.path.join(OUT, "1x", name + ".png"))
        browser.close()

    # ---- per-texture checks
    for name, a in ART.items():
        img = imgs[name]
        w, h = img.size
        fmt = FORMAT[a["kind"]]
        if not (pot(w) and pot(h)) or w > 1024 or h > 1024:
            errs.append("%s %dx%d not POT / over 1024" % (name, w, h))
        tile = (8, 8) if fmt == "I4" else (4, 4)
        if w % tile[0] or h % tile[1] or (w // 2) % tile[0] or (h // 2) % tile[1]:
            errs.append("%s: %dx%d (1x %dx%d) not whole %s tiles" % (name, w, h, w // 2, h // 2, fmt))
        arr = np.asarray(img)
        alpha = arr[..., 3]
        if alpha.max() == 0:
            errs.append("%s is empty" % name)
        row = dict(name=name, group=a["group"], file_2x="out/2x/%s.png" % name,
                   file_1x="out/1x/%s.png" % name, size_2x=[w, h], size_1x=[w // 2, h // 2],
                   format=fmt, bytes_2x=w * h // 2 if fmt == "I4" else w * h * 2,
                   min_draw_1x=a["min_draw_1x"], use=a["use"])
        if a["kind"] == "mask":
            if (arr[..., :3][alpha > 0] != 255).any():
                errs.append("%s: a mask texel is not white" % name)
            row["tint"] = a["tint"]
            row["alpha_levels_i4"] = int(len(np.unique(np.round(alpha / 255 * 15))))
            leg = topo_ok(name, a, img, errs)
            if leg:
                row["legibility"] = leg
        else:
            q = QC.rgb5a3(img)
            o = arr.astype("int16")
            qa = np.asarray(q).astype("int16")
            row["rgb5a3"] = dict(
                max_rgb_delta_opaque=int(np.abs(o[..., :3] - qa[..., :3])[alpha == 255].max(initial=0)),
                max_alpha_delta=int(np.abs(o[..., 3] - qa[..., 3]).max()),
                colours_before=int(len(np.unique(o.reshape(-1, 4), axis=0))),
                colours_after=int(len(np.unique(qa.reshape(-1, 4), axis=0))),
                alpha_values=sorted(int(v) for v in np.unique(alpha)))
            if row["rgb5a3"]["max_alpha_delta"] > 18:
                errs.append("%s: RGB5A3 alpha delta %d" % (name, row["rgb5a3"]["max_alpha_delta"]))
        rows.append(row)

    # the 9-slice glass must be exact: a centre texel is #111122 at 219, before and after RGB5A3
    c = np.asarray(imgs["lab_frame_fill"])[16, 16]
    qc = np.asarray(QC.rgb5a3(imgs["lab_frame_fill"]))[16, 16]
    if tuple(c) != (17, 17, 34, 219) or tuple(qc) != (17, 17, 34, 219):
        errs.append("glass texel %s -> %s, want (17,17,34,219)" % (tuple(c), tuple(qc)))
    # edges must be constant along their stretch axis
    eh, ev = np.asarray(imgs["lab_frame_edge_h"]), np.asarray(imgs["lab_frame_edge_v"])
    if (eh != eh[:, :1]).any() or (ev != ev[:1, :]).any():
        errs.append("a frame edge is not constant along its stretch axis")
    # corners must meet the edges: the ink/glass boundary at the seam is the same row/column
    tl, tr = np.asarray(imgs["lab_frame_corner_tl"]), np.asarray(imgs["lab_frame_corner_tr"])
    if (tl[:, -1] != eh[:, 0]).any() or (tr[:, 0] != eh[:, 0]).any():
        errs.append("top corners do not meet lab_frame_edge_h at the seam")
    if (tl[-1, :] != ev[0, :]).any():
        errs.append("lab_frame_corner_tl does not meet lab_frame_edge_v at the seam")

    manifest = dict(
        name="geno-lab UI art", private="Geno build only (branch private/geno)",
        provenance="Original drawings in pc/geno/mods/geno-lab/art (SVG on the kit's 64 grid, "
                   "HTML/CSS), rendered by headless Chromium; nothing traced, sampled or "
                   "referenced from Melee or any other game.",
        authoring="2x; draw every texture at its size_1x on the 640x480 script screen",
        naming=dict(
            icons="ico_lab_<name>: gd.kit.icon('lab_<name>') -> ico_lab_<name>.gxtex",
            menu_icon="ico_lab (the kit's ui/, not only the mod's: see art/README.md)",
            markers="lab_mk_<event>", timeline="lab_tl_*", keycap="lab_key_*",
            panel="lab_frame_* (same slots as the kit's frame_*: corner_tl/tr/bl/br, "
                  "edge_h, edge_v, + fill)"),
        masks="I4 white masks: TEV RGB from the material/vertex colour, alpha = texture alpha "
              "x colour alpha (the kit's 'I4 needs a specific TEV setup'); IA4 is the fallback",
        palette=LP.palette_dict(rep),
        panel=dict(piece_1x=16, border_1x=2, chamfer_1x=6, corners="tl, br chamfered (tl "
                   "carries the accent cap); tr, bl square", min_size_1x=[32, 32],
                   content_inset_1x=6,
                   title_slab="optional: a flat accent quad under the top edge, ink text, "
                              "caption/body role"),
        timeline=dict(
            track=dict(height_1x=8, colour="track", caps="lab_tl_cap_l / _r at 8x16 @1x drawn "
                       "8 px tall (scale V) or 16 tall for a double-height track"),
            windows="flat quads in palette.hit[<id>], 6 px tall inside the track; min 2 px wide",
            ticks=dict(colour="tick", minor=dict(every=5, size_1x=[1, 2]),
                       major=dict(every=10, size_1x=[1, 4])),
            playhead=dict(texture="lab_tl_playhead", size_1x=[16, 32], anchor_1x=[8, 0],
                          note="stem centred on the frame's x; ink shadow at +1,+1 first"),
            markers=dict(size_1x=[8, 8], lanes=LP.LANES,
                         note="lab_mk_* at 8 px (or 12 with room); ink shadow +1,+1")),
        key_chip=dict(KEY, label_role="caption", label_colour="ink", body_tint="muted",
                      top_tint="bone", held_body_tint="accent",
                      layout="w = max(16, text advance + 8); body: lab_key_l, flat body quad "
                             "(muted), lab_key_r; top: lab_key_top_l, flat quad over top rows "
                             "1..12, lab_key_top_r; label baseline centres its cap height in "
                             "the top face. Pressed: top face and label +1 px down."),
        textures=rows)
    with open(os.path.join(OUT, "lab_ui.json"), "w", encoding="utf-8") as fh:
        json.dump(manifest, fh, indent=1)

    # ---- convert with the port's own converter, then read every file back
    if PNG2GX is None:
        errs.append("pc/tools/png2gx.py not found - no .gxtex written")
    else:
        if os.path.isdir(UI):
            for f in os.listdir(UI):
                if f.endswith(".gxtex") or f == "lab_ui.json":
                    os.remove(os.path.join(UI, f))
        r = subprocess.run([sys.executable, PNG2GX, "--layout", os.path.join(OUT, "lab_ui.json"),
                            "--outdir", UI, "--res", "2x"], capture_output=True, text=True)
        if r.returncode:
            errs.append("png2gx failed:\n" + r.stdout + r.stderr)
        sys.path.insert(0, os.path.dirname(PNG2GX))
        import png2gx
        worst = {}
        for row in rows:
            path = os.path.join(UI, row["name"] + ".gxtex")
            if not os.path.exists(path):
                errs.append("missing " + path)
                continue
            back = png2gx.decode_gxtex(path)
            src = imgs[row["name"]]
            wr, wa, mean = png2gx.compare(src, back)
            if row["format"] == "I4":
                wr = 0                                # I4 decodes to white; RGB is the tint
            worst[row["name"]] = (wr, wa, os.path.getsize(path))
            imgs[row["name"] + "#gx"] = back
            # half a quantisation step: I4 alpha 17/2, RGB5A3 3-bit alpha 36/2, RGB444 17/2
            if wa > (8 if row["format"] == "I4" else 18) or wr > 8:
                errs.append("%s round trip rgb %d alpha %d" % (row["name"], wr, wa))
        print("png2gx: %d .gxtex in %s" % (len(worst), os.path.relpath(UI, HERE)))
        rep_rt = worst

    if args.install_menu_icon:
        os.makedirs(args.install_menu_icon, exist_ok=True)
        shutil.copyfile(os.path.join(UI, "ico_lab.gxtex"),
                        os.path.join(args.install_menu_icon, "ico_lab.gxtex"))
        print("installed ico_lab.gxtex -> " + args.install_menu_icon)

    sheet(imgs, rows)

    # ---- report
    print("\n%-22s %-9s %-8s %-7s %s" % ("texture", "2x", "1x", "format", "gxtex bytes / checks"))
    for row in rows:
        extra = ""
        if "rgb5a3" in row:
            q = row["rgb5a3"]
            extra = " colours %d->%d, max rgb d %d, alpha %s" % (
                q["colours_before"], q["colours_after"], q["max_rgb_delta_opaque"],
                q["alpha_values"][:6] + (["..."] if len(q["alpha_values"]) > 6 else []))
        if "legibility" in row:
            extra = " shapes/holes %s @%dpx %s" % (row["legibility"]["full"],
                                                   2 * row["min_draw_1x"],
                                                   row["legibility"]["at_2x_draw"])
        rt = rep_rt.get(row["name"], ("-", "-", 0))
        print("%-22s %-9s %-8s %-7s %5d B  rt a%s%s" % (
            row["name"], "%dx%d" % tuple(row["size_2x"]), "%dx%d" % tuple(row["size_1x"]),
            row["format"], rt[2], rt[1], extra))
    total = sum(r["bytes_2x"] for r in rows)
    print("\n%d textures, %.1f KB of texel data @2x" % (len(rows), total / 1024))
    if errs:
        print("\nCHECKS FAILED (%d):" % len(errs))
        for e in errs:
            print("  - " + e)
        return 1
    print("lab art checks ok: POT, whole tiles, non-empty, masks white, legibility at 2x of "
          "each smallest draw, glass exact, 9-slice seams meet, png2gx round trip")
    return 0


# ============================================================ preview sheet
FONT_DIR = os.path.join(MENU, "SourceSans3")


def font(size, weight="Black"):
    return ImageFont.truetype(os.path.join(FONT_DIR, "SourceSans3-%s.otf" % weight), size)


def hexc(h, a=255):
    return LP.rgb(h) + (a,)


def tint_of(t):
    if t in LP.MARK:
        return LP.MARK[t]
    if t in LP.LAB:
        return LP.LAB[t]
    return LP.KIT.get(t, LP.KIT["bone"])


def paste_mask(canvas, m, xy, colour, size=None, shadow=None):
    if size and m.size != size:
        m = m.resize(size, Image.BOX)
    a = m.getchannel("A")
    if shadow:
        canvas.paste(Image.new("RGBA", m.size, hexc(LP.KIT["ink"])), (xy[0] + shadow, xy[1] + shadow), a)
    canvas.paste(Image.new("RGBA", m.size, hexc(colour) if isinstance(colour, str) else colour), xy, a)


def backdrop(w, h):
    """A busy stand-in for gameplay behind the translucent panel: generated stripes and
    blocks from the kit's own colours (never a game capture)."""
    im = Image.new("RGBA", (w, h), (255, 255, 255, 255))
    d = ImageDraw.Draw(im)
    cols = ["#f2efe4", "#2f7cf0", "#f4d23a", "#27b88a", "#e5483b", "#ffffff", "#0a0e18", "#7e8490"]
    for i in range(0, w + h, 36):
        d.polygon([(i, 0), (i + 18, 0), (i + 18 - h, h), (i - h, h)], fill=hexc(cols[(i // 36) % len(cols)]))
    return im


def nine_slice(gx, w, h):
    """Compose a panel from the decoded .gxtex pieces exactly as a 9-slice draws them."""
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    c = 32
    tl, tr, bl, br = (gx["lab_frame_corner_" + k] for k in ("tl", "tr", "bl", "br"))
    eh, ev, fl = gx["lab_frame_edge_h"], gx["lab_frame_edge_v"], gx["lab_frame_fill"]
    sx, sy = w - 2 * c, h - 2 * c
    if sx > 0:
        top = eh.resize((sx, c), Image.NEAREST)
        im.paste(top, (c, 0))
        im.paste(top.transpose(Image.FLIP_TOP_BOTTOM), (c, h - c))
    if sy > 0:
        left = ev.resize((c, sy), Image.NEAREST)
        im.paste(left, (0, c))
        im.paste(left.transpose(Image.FLIP_LEFT_RIGHT), (w - c, c))
    if sx > 0 and sy > 0:
        im.paste(fl.resize((sx, sy), Image.NEAREST), (c, c))
    im.paste(tl, (0, 0)); im.paste(tr, (w - c, 0)); im.paste(bl, (0, h - c)); im.paste(br, (w - c, h - c))
    return im


def key_chip(gx, label, held=False, pressed=False):
    """A keycap at 2x: 3-slice body + 3-slice top face + the label in ink (caption role)."""
    f = font(24, "Black")
    tw = f.getlength(label)
    w = max(32, int(tw + 16 + 0.5) // 2 * 2)
    im = Image.new("RGBA", (w, 32), (0, 0, 0, 0))
    body = LP.LAB["accent"] if held else LP.KIT["muted"]
    L, R = gx["lab_key_l"].getchannel("A"), gx["lab_key_r"].getchannel("A")
    tL, tR = gx["lab_key_top_l"].getchannel("A"), gx["lab_key_top_r"].getchannel("A")
    im.paste(Image.new("RGBA", (16, 32), hexc(body)), (0, 0), L)
    im.paste(Image.new("RGBA", (16, 32), hexc(body)), (w - 16, 0), R)
    if w > 32:
        ImageDraw.Draw(im).rectangle((16, 0, w - 17, 31), fill=hexc(body))
    dy = 2 if pressed else 0
    im.paste(Image.new("RGBA", (16, 32), hexc(LP.KIT["bone"])), (0, dy), tL)
    im.paste(Image.new("RGBA", (16, 32), hexc(LP.KIT["bone"])), (w - 16, dy), tR)
    if w > 32:
        ImageDraw.Draw(im).rectangle((16, 2 + dy, w - 17, 23 + dy), fill=hexc(LP.KIT["bone"]))
    d = ImageDraw.Draw(im)
    d.text((w / 2, 13 + dy), label, font=f, fill=hexc(LP.KIT["ink"]), anchor="mm")
    return im


def kit_png(sub, name):
    for d in ("out_nav", "out_online", "out_kit", "out_lobby", "out_loading", "out"):
        p = os.path.join(MENU, d, sub, name + ".png")
        if os.path.exists(p):
            return Image.open(p).convert("RGBA")
    return None


def sheet(imgs, rows):
    gx = {k[:-3]: v for k, v in imgs.items() if k.endswith("#gx")}
    if len(gx) < len(ART):
        gx = {n: imgs[n] for n in ART}
    ink, bone, muted = hexc(LP.KIT["ink"]), hexc(LP.KIT["bone"]), hexc(LP.KIT["muted"])
    solo_bg = hexc(LP.kit.SECTIONS["solo"]["bg"])
    W = 1760
    canvas = Image.new("RGBA", (W, 2400), solo_bg)
    d = ImageDraw.Draw(canvas)
    H1, H2, SM = font(34), font(20, "Bold"), font(16, "Bold")

    def head(y, text, sub=None):
        d.rectangle((24, y, 24 + d.textlength(text, font=H1) + 28, y + 44), fill=hexc(LP.LAB["accent"]))
        d.text((38, y + 22), text, font=H1, fill=ink, anchor="lm")
        if sub:
            d.text((24 + d.textlength(text, font=H1) + 48, y + 24), sub, font=H2, fill=muted, anchor="lm")
        return y + 64

    y = 20
    d.text((24, y), "GENO LAB - UI art review sheet", font=font(44), fill=bone)
    d.text((24, y + 52), "Private (Geno). Composed from the decoded .gxtex files (what GX samples), "
           "at 2x = 1:1 with a 1280x960 window. Kit art alongside for comparison. Reference only.",
           font=H2, fill=muted)
    y += 100

    # ---- 1. icons: on glass over a busy backdrop; bone, ON (accent), OFF (off + slash), 1x
    icons = [n for n in ART if ART[n]["group"] == "icon" and not n.startswith("ico_lab_slash")]
    y = head(y, "1  CONTROL ICONS", "ico_lab_* 64x64 @2x I4 masks - tint: bone | ON accent | "
             "OFF disabled + slash | at 32 and 16 px (1x)")
    cols, cw, ch = 7, 244, 150
    nrow = (len(icons) + cols - 1) // cols
    area = backdrop(W - 48, nrow * ch + 16)
    glass = nine_slice(gx, W - 48, nrow * ch + 16)
    area.alpha_composite(glass)
    canvas.alpha_composite(area, (24, y))
    for i, n in enumerate(icons):
        x0, y0 = 24 + 20 + (i % cols) * cw, y + 18 + (i // cols) * ch
        m = gx[n]
        paste_mask(canvas, m, (x0, y0), LP.KIT["bone"])
        paste_mask(canvas, m, (x0 + 74, y0), LP.LAB["accent"])
        paste_mask(canvas, gx["ico_lab_slash_gap"], (x0 + 148, y0), (17, 17, 34, 255))
        paste_mask(canvas, m, (x0 + 148, y0), LP.LAB["off"])
        cut = gx["ico_lab_slash_gap"].getchannel("A")
        # the gap cuts the icon: re-draw the glass-coloured gap over it, then the slash
        canvas.paste(Image.new("RGBA", (64, 64), (26, 26, 44, 255)), (x0 + 148, y0), cut)
        paste_mask(canvas, gx["ico_lab_slash"], (x0 + 148, y0), LP.LAB["off"])
        paste_mask(canvas, m, (x0, y0 + 74), LP.KIT["bone"], size=(32, 32))
        paste_mask(canvas, m, (x0 + 40, y0 + 74), LP.KIT["bone"], size=(16, 16))
        d.text((x0 + 64, y0 + 96), n[8:], font=SM, fill=bone)
    y += nrow * ch + 36

    # kit icons for comparison, same treatment
    y = head(y, "   KIT ICONS FOR COMPARISON", "the kit's own masks, same draw")
    x = 44
    for n in ("ico_training", "ico_solo", "ico_host", "ico_join", "ico_copy", "ico_warning",
              "ico_lock", "ico_wait", "ico_keyboard", "ico_target", "ico_records"):
        k = kit_png("2x", n)
        if k is None:
            continue
        paste_mask(canvas, k, (x, y), LP.KIT["bone"], size=(64, 64))
        paste_mask(canvas, k, (x + 72, y + 16), LP.KIT["bone"], size=(32, 32))
        d.text((x, y + 72), n, font=SM, fill=muted)
        x += 148
    y += 110

    # ---- 2. LAB menu icon beside the kit's hub icons, on the SOLO face
    y = head(y, "2  LAB MENU ICON", "ico_lab 256x256 @2x I4 - SOLO > LAB, beside SOLO's other "
             "entries (face_hi on face, then selected: ink on gold)")
    face = hexc(LP.kit.SECTIONS["solo"]["face"])
    fhi = LP.kit.SECTIONS["solo"]["face_hi"]
    d.rectangle((24, y, W - 24, y + 200), fill=face)
    x = 48
    for n in ("ico_lab", "ico_training", "ico_regular", "ico_event", "ico_stadium"):
        m = gx.get(n) or kit_png("2x", n)
        if m is None:
            continue
        paste_mask(canvas, m, (x, y + 16), fhi, size=(128, 128))
        paste_mask(canvas, m, (x + 140, y + 16), fhi, size=(32, 32))
        paste_mask(canvas, m, (x + 140, y + 60), fhi, size=(16, 16))
        d.text((x, y + 160), n, font=H2, fill=bone)
        x += 250
    d.rectangle((x, y + 8, x + 300, y + 192), fill=hexc(LP.KIT["gold"]))
    paste_mask(canvas, gx["ico_lab"], (x + 16, y + 24), LP.KIT["ink"], size=(128, 128))
    d.text((x + 160, y + 88), "LAB", font=font(44), fill=ink, anchor="lm")
    y += 232

    # ---- 3. panels: the Lab glass panel over a backdrop, beside the kit frame
    y = head(y, "3  COMPACT IN-MATCH PANEL", "lab_frame_* 32x32 @2x RGB5A3 9-slice: glass #111122 "
             "@ 219/255, ink border, accent cap; beside the kit's frame_*")
    bd = backdrop(W - 48, 330)
    canvas.alpha_composite(bd, (24, y))
    for (px, py, pw, ph, title) in ((60, 24, 520, 280, "INFO  P1 KIRBY"), (620, 24, 300, 120, None),
                                   (620, 170, 160, 64, None), (820, 170, 64, 64, None)):
        pn = nine_slice(gx, pw, ph)
        canvas.alpha_composite(pn, (24 + px, y + py))
        if title:
            d.rectangle((24 + px + 16, y + py + 4, 24 + px + 16 + 250, y + py + 40),
                        fill=hexc(LP.LAB["accent"]))
            d.text((24 + px + 28, y + py + 22), title, font=font(26), fill=ink, anchor="lm")
            lines = [("action 44 AttackAirF  f12", LP.KIT["bone"]),
                     ("anim 12.00  x1.00", LP.KIT["muted"]),
                     ("hitlag 0  hitstun 0  kb 0.0", LP.KIT["muted"]),
                     ("intang 0  invinc 0  body normal", LP.MARK["iasa"]),
                     ("#0 b3 12.0% a361 kbg100 bkb10", LP.HIT["hit0"]),
                     ("#1 b4 10.0% a361 kbg100 bkb10", LP.HIT["hit1"])]
            for k, (t, c) in enumerate(lines):
                d.text((24 + px + 28, y + py + 58 + k * 34), t, font=font(26, "Bold"), fill=hexc(c))
    # kit frame for comparison (its own RGB5A3 pieces, 128 corners)
    kf = {k: kit_png("2x", "frame_" + k) for k in ("corner_tl", "corner_tr", "corner_bl",
                                                     "corner_br", "edge_h", "edge_v")}
    if all(v is not None for v in kf.values()):
        kx, ky, kw, kh = 24 + 960, y + 24, 740, 280
        c = 128
        fr = Image.new("RGBA", (kw, kh), (0, 0, 0, 0))
        top = kf["edge_h"].resize((kw - 2 * c, 32), Image.NEAREST)
        fr.alpha_composite(top, (c, 0)); fr.alpha_composite(top.transpose(Image.FLIP_TOP_BOTTOM), (c, kh - 32))
        lv = kf["edge_v"].resize((32, kh - 2 * c), Image.NEAREST)
        fr.alpha_composite(lv, (0, c)); fr.alpha_composite(lv.transpose(Image.FLIP_LEFT_RIGHT), (kw - 32, c))
        for k, xy in (("corner_tl", (0, 0)), ("corner_tr", (kw - c, 0)), ("corner_bl", (0, kh - c)),
                      ("corner_br", (kw - c, kh - c))):
            fr.alpha_composite(kf[k], xy)
        canvas.alpha_composite(fr, (kx, ky))
        d.text((kx + 150, ky + 130), "kit frame_* (opaque, menus)", font=H2, fill=ink)
    y += 350

    # ---- 4. timeline
    y = head(y, "4  MOVE TIMELINE", "track + lab_tl_cap_l/r, ticks (flat), hitbox windows "
             "(flat, hit colours), lab_mk_* markers, lab_tl_playhead")
    bd = backdrop(W - 48, 300)
    canvas.alpha_composite(bd, (24, y))
    pw, ph = W - 48 - 80, 260
    canvas.alpha_composite(nine_slice(gx, pw, ph), (64, y + 20))
    d.text((96, y + 50), "P1 AttackAirF  (PlyKirby5K_Share_ACTION_AttackAirF_figatree)  49 frames",
           font=font(26, "Bold"), fill=hexc(LP.kit.PORTS["p1"]))
    tx0, tx1, ty = 110, W - 150, y + 150          # track 16 @2x tall (8 @1x)
    frames = 49
    fx = lambda f: tx0 + (f - 1) * (tx1 - tx0) / frames  # noqa: E731
    trk = LP.LAB["track"]
    paste_mask(canvas, gx["lab_tl_cap_l"], (tx0 - 16, ty), trk, size=(16, 16))
    paste_mask(canvas, gx["lab_tl_cap_r"], (tx1, ty), trk, size=(16, 16))
    d.rectangle((tx0, ty, tx1 - 1, ty + 15), fill=hexc(trk))
    for f in range(5, frames + 1, 5):
        h = 8 if f % 10 == 0 else 4
        d.rectangle((fx(f), ty + 18, fx(f) + 1, ty + 18 + h), fill=hexc(LP.LAB["tick"]))
    for (a, b, hid) in ((6, 8, "hit0"), (6, 8, "hit1"), (9, 20, "hit2"), (21, 24, "hit3")):
        yy = ty + 2 if hid in ("hit0", "hit2", "hit3") else ty + 8
        hh = 12 if hid in ("hit2", "hit3") else 6
        d.rectangle((fx(a), yy, fx(b + 1) - 2, yy + hh - 1), fill=hexc(LP.HIT[hid]))
    for (f, k) in ((33, "iasa"), (4, "invinc"), (28, "invinc")):
        paste_mask(canvas, gx["lab_mk_" + k], (int(fx(f)) - 8, ty - 22), LP.MARK[k], size=(16, 16), shadow=2)
    for (f, k) in ((6, "gfx"), (6, "sfx"), (18, "vis"), (40, "sfx")):
        off = 10 if k == "sfx" and f == 6 else 0
        paste_mask(canvas, gx["lab_mk_" + k], (int(fx(f)) - 8 + off * 2, ty + 32), LP.MARK[k],
                   size=(16, 16), shadow=2)
    ph_x = int(fx(12))
    paste_mask(canvas, gx["lab_tl_playhead"], (ph_x - 16 + 2, ty - 26 + 2), LP.KIT["ink"])
    paste_mask(canvas, gx["lab_tl_playhead"], (ph_x - 16, ty - 26), LP.LAB["playhead"])
    d.text((110, ty + 64), "now 12.0  IASA f33  f6-8 #0 12% a361 g100 b10  f6-8 #1 10% a361  "
           "f9-20 #2 8% a50  f21-24 #3 6% a361", font=font(22, "Bold"), fill=muted)
    # legend
    lx = 110
    for k in ("iasa", "invinc", "gfx", "sfx", "vis", "hitbox"):
        c = LP.MARK.get(k, LP.HIT["hit0"])
        paste_mask(canvas, gx["lab_mk_" + k], (lx, y + 244), c, size=(32, 32), shadow=2)
        d.text((lx + 40, y + 260), {"invinc": "body state", "hitbox": "hit window"}.get(k, k.upper()),
               font=H2, fill=bone, anchor="lm")
        lx += 200
    y += 330

    # ---- 5. key chips
    y = head(y, "5  KEY-HINT CHIPS", "lab_key_l/r + lab_key_top_l/r (16x32 @2x I4), 3-sliced; "
             "label from the font atlas (caption, ink) - beside the kit's glyph_*")
    d.rectangle((24, y, W - 24, y + 120), fill=hexc(LP.LAB["glass"]))
    x = 48
    for lab, kw in (("N", {}), ("B", {}), ("P", {}), ("F3", {}), ("TAB", {}), ("CTRL", {}),
                    ("PGDN", {}), ("HOME", {}), ("1", {}), ("0", {}), ("N", {"held": True}),
                    ("B", {"pressed": True, "held": True})):
        chip = key_chip(gx, lab, **kw)
        canvas.alpha_composite(chip, (x, y + 20))
        c1 = chip.resize((chip.width // 2, 16), Image.BOX)
        canvas.alpha_composite(c1, (x, y + 70))
        x += chip.width + 22
    d.text((x + 10, y + 36), "step", font=H2, fill=bone)
    kx = x + 90
    for g in ("glyph_a", "glyph_b", "glyph_x", "glyph_start", "glyph_z"):
        k = kit_png("2x", g)
        if k is not None:
            paste_mask(canvas, k, (kx, y + 20), LP.KIT["bone"])
            kx += k.width + 16
    d.text((48, y + 96), "top: 2x (1:1 at 1280x960)   below: 1x.   held = accent body;  "
           "pressed = top face +1 px down", font=SM, fill=muted)
    y += 150

    # ---- 6. palette
    y = head(y, "6  LAB PALETTE", "lab_palette.py - checked: text on glass over white/grey/black, "
             "timeline colours on the track, CVD distances")
    groups = [("lab", {k: v for k, v in LP.LAB.items()}), ("hit", LP.HIT), ("marks", LP.MARK)]
    for gname, g in groups:
        d.text((24, y + 18), gname, font=H2, fill=muted)
        x = 110
        for k, v in g.items():
            d.rectangle((x, y, x + 110, y + 44), fill=hexc(v))
            d.text((x, y + 50), k, font=SM, fill=bone)
            d.text((x, y + 68), v + (" @219" if k == "glass" else ""), font=SM, fill=muted)
            x += 128
        y += 100
    # CVD strip for timeline colours
    for kind in LP.CVD:
        d.text((24, y + 10), kind, font=SM, fill=muted)
        x = 150
        for v in list(LP.HIT.values()) + list(LP.MARK.values()) + [LP.LAB["playhead"], LP.LAB["accent"]]:
            c = tuple(round(t) for t in LP.K.simulate(LP.rgb(v), kind))
            d.rectangle((x, y, x + 60, y + 30), fill=c + (255,))
            x += 66
        y += 38
    y += 20
    canvas = canvas.crop((0, 0, W, y))
    out = os.path.join(PREVIEW, "lab_sheet.png")
    canvas.convert("RGB").save(out, optimize=True)
    print("preview sheet: %s (%dx%d)" % (out, canvas.width, canvas.height))


if __name__ == "__main__":
    sys.exit(main())
