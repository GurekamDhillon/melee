#!/usr/bin/env python3
"""Envoy models, chunk 1: item pickups, act-map nodes / room markers, the heart container and the pool frames.

The same method as the Party drive models (`../../envoy_drives/tools/drive_parts.py`, imported here so the mesh
primitives and the colour ramp cannot drift): every model is a short PART LIST in a -1..1 box (bevelled prisms `P`,
camera-facing rings and arcs `R`, icosahedron balls `B`), coloured from a ramp atlas (a dark body row and a bright
"hot" row per colour). One part list gives three outputs:

  models/em_<key>.gxmesh            GXMS v2, the drives' format and frame (parts box x 2.2, no recentring)
  models/envoy_models_atlas.gxtex   GXTX v1 RGBA8 colour ramps, + .glow.gxtex (the drives' atlas layout)
  models/envoy_models.txt           the manifest: one key per line, in model order
  <art>/em_<key>.png                a 96 x 96 sticker sprite (48 game pixels at 2x), ink edge, for gd_ui_tex_load

Every coordinate and colour is original, written for this project; nothing is traced or converted from any game,
model file or other project. Meshes, atlas and manifest need only the Python standard library and are byte-for-byte
deterministic. The PNG sprites and the contact sheet are rasterised here too (numpy z-buffer, Pillow to encode).

usage: python make_models1.py [--no-sprites] [--sheet OUT.png] [--check]
  --check   regenerate into memory and compare with the files on disk (exit 1 on any difference)
"""
import io
import math
import struct
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
sys.path.insert(0, str(HERE.parents[1] / "envoy_drives" / "tools"))
import drive_parts as DP  # noqa: E402
from drive_parts import P, R, B, bar, flame, kite, ngon, rect, rot, mov, scl, mir, arcpts  # noqa: E402

OUT_MODELS = HERE.parent / "models"
OUT_ART = REPO / "rust" / "mods" / "envoy_ui" / "art"
HALF = 2.2                     # the drives' frame: parts box x HALF world units
AW, AH, ARH = 128, 256, 4      # atlas: the Party atlas layout (64 rows of 4 texels; 33 used)
SPRITE = 96                    # sprite size (2x of 48 game pixels)
SPAN = 1.25                    # a sprite shows the parts box -SPAN..SPAN
FRAME_SPAN = SPAN * 4 / 3      # a pool frame's sprite shows more: draw it at 4/3 the item's size, same centre
INK = (42, 26, 110)            # the Party pack ink (#2A1A6E)

# ---------------------------------------------------------------------------------------------- palette
# name: (dark, main, hot). Two atlas rows each: <name> (dark -> main) and <name>_hot (main/hot mix -> hot).
PALETTE = [
    ("volt",    (150, 110, 10), (255, 240, 112), (255, 255, 224)),   # Spark Ring      0xFFF070
    ("moon",    (104, 112, 170), (232, 236, 255), (255, 255, 255)),  # Moon Drops      0xE8ECFF
    ("pink",    (150, 50, 124), (255, 154, 224), (255, 232, 248)),   # Twin Shot       0xFF9AE0
    ("green",   (18, 104, 52), (106, 224, 122), (226, 255, 206)),    # Bouncy          0x6AE07A
    ("sky",     (24, 84, 160), (128, 208, 255), (236, 250, 255)),    # Chain Spark     0x80D0FF
    ("ice",     (40, 112, 172), (168, 232, 255), (255, 255, 255)),   # Ice Glass       0xA8E8FF
    ("violet",  (48, 26, 122), (138, 90, 240), (232, 206, 255)),     # Gravity Well    0x8A5AF0, the secret room
    ("orange",  (150, 44, 8), (255, 138, 48), (255, 234, 124)),      # Big Bang        0xFF8A30, the campfire
    ("heart",   (140, 20, 52), (255, 90, 122), (255, 218, 228)),     # Spare Heart     0xFF5A7A
    ("crimson", (70, 6, 22), (208, 26, 42), (255, 132, 64)),         # the devil door  0xD01A2A
    ("cream",   (176, 146, 84), (255, 244, 184), (255, 255, 255)),   # the angel door  0xFFF4B8
    ("gold",    (150, 96, 10), (255, 216, 74), (255, 248, 196)),     # SUN #FFD84A: coins, crowns, hilts
    ("teal",    (10, 92, 92), (46, 196, 182), (206, 255, 246)),      # the shop
    ("stone",   (82, 76, 122), (174, 168, 204), (255, 255, 255)),    # pedestals, walls (the Utility row)
    ("steel",   (84, 96, 126), (204, 212, 228), (255, 255, 255)),    # blades
    ("wood",    (78, 42, 20), (170, 106, 58), (236, 190, 130)),      # logs, grips, posts
]
ROW_ORDER = []
for _n, _dk, _mn, _hot in PALETTE:
    DP.row(_n, _dk, _mn, _mn); DP.row(_n + "_hot", DP.mix(_mn, _hot, .6), _hot, _hot)
    ROW_ORDER += [_n, _n + "_hot"]
DP.row("ink", (20, 12, 51), INK)        # already in drive_parts; restated so this file reads alone
ROW_ORDER.append("ink")
assert len(ROW_ORDER) * ARH <= AH


def glow_k(name, t):
    """The glow atlas, as make_drives.party_atlas: hot rows glow along the ramp, bodies only on the bevel rim."""
    return (t ** 1.2) * .8 if name.endswith("_hot") else (t ** 3) * .35


# ---------------------------------------------------------------------------------------------- shape helpers
def G(pal, *parts):
    """One colour group of a model: body parts take row `pal`, 'h' and 'i' parts `pal`_hot."""
    flat = []
    for p in parts:
        flat += p if isinstance(p, list) else [p]
    return (pal, flat)

FRONT = (.14, .34)             # a mark standing proud of a face (any role)
BACK = (-.42, -.22)            # the pool frames sit behind whatever they frame

def heart(cx, cy, s, **kw):
    """Two 8-sided lobes and a triangle (the Cinder Heart construction)."""
    return [P(ngon(cx - .36*s, cy + .22*s, .42*s, 8, 22.5), **kw), P(ngon(cx + .36*s, cy + .22*s, .42*s, 8, 22.5), **kw),
            P([(cx - .76*s, cy + .08*s), (cx + .76*s, cy + .08*s), (cx, cy - .74*s)], bev=.8, **kw)]

def spark4(c, r, w, **kw):
    """A four-point glint: two thin diamonds crossed."""
    x, y = c
    return [P([(x, y + r), (x + w, y), (x, y - r), (x - w, y)], bev=.5, **kw), P([(x + r, y), (x, y + w), (x - r, y), (x, y - w)], bev=.5, **kw)]

def crescent(c, r, k, d, ang, n=7, **kw):
    """A crescent: the part of the disc (c, r) outside the disc of radius k*r set `d` to its right; `ang` turns it.
    Built as n convex quads from the outer arc to the inner one (the two end ones are triangles)."""
    r2 = k*r
    xi = (d*d + r*r - r2*r2) / (2*d); yi = math.sqrt(r*r - xi*xi)
    a = math.degrees(math.atan2(yi, xi)); b = math.degrees(math.atan2(yi, xi - d))
    outer = arcpts((0, 0), r, a, 360 - a, n); inner = arcpts((d, 0), r2, b, 360 - b, n)
    out = []
    for i in range(n):
        q = [outer[i], outer[i+1], inner[i+1], inner[i]]
        if i == 0: q = q[:3]
        if i == n-1: q = [q[0], q[1], q[3]]
        out.append(P(mov(rot(q, ang), c[0], c[1]), bev=.86, **kw))
    return out

def arch(cx, y0, w, h, n=6):
    """A door: a rectangle with a round top, base at y0, `h` tall in all."""
    return [(cx - w/2, y0), (cx + w/2, y0)] + arcpts((cx, y0 + h - w/2), w/2, 0, 180, n)

def horn(side, base, **kw):
    """A curved horn from `base` up and out to `side` (+1 right, -1 left): a root quad and a tip triangle."""
    x, y = base
    f = lambda pts: [(x + side*px, y + py) for px, py in pts]
    return [P(f([(-.14, -.1), (.1, -.2), (.5, .16), (.32, .38)]), bev=.8, **kw), P(f([(.5, .16), (.32, .38), (.5, .78)]), bev=.7, **kw)]

def sword(ang, zo):
    """One sword through the origin, point up, turned `ang`; `zo` lifts it so two can cross."""
    z = (-.2 + zo, .2 + zo)
    t = lambda pts: rot(mov(pts, 0, -.15), ang)
    blade = P(t([(-.11, -.2), (.11, -.2), (.11, .9), (0, 1.14), (-.11, .9)]), z=z, bev=.55)
    guard = P(t(rect(0, -.26, .6, .14)), z=(z[0] - .03, z[1] + .03))
    grip = P(t(rect(0, -.52, .14, .4)), z=z)
    px, py = t([(0, -.76)])[0]
    return [G("steel", blade), G("gold", guard, B((px, py), .11, "h", z=zo)), G("wood", grip)]

OCT = [(-.84, 1.18), (.84, 1.18), (1.18, .84), (1.18, -.84), (.84, -1.18), (-.84, -1.18), (-1.18, -.84), (-1.18, .84)]

def frame_ring(keep=lambda i: True, sub=1, s=.84, **kw):
    """The pool frame: a cut-corner square outline (outer 2.36 across), each side in `sub` quads; `keep` drops some."""
    outer = []
    for i in range(8):
        a, b = OCT[i], OCT[(i+1) % 8]
        k = sub if i % 2 == 0 else 1
        outer += [(a[0] + (b[0]-a[0])*j/k, a[1] + (b[1]-a[1])*j/k) for j in range(k)]
    inner = scl(outer, s); n = len(outer)
    return [P([outer[i], outer[(i+1) % n], inner[(i+1) % n], inner[i]], bev=.8, z=BACK, **kw) for i in range(n) if keep(i)]


# ---------------------------------------------------------------------------------------------- the models
def M(key, name, kind, concept, use, groups):
    flat = []
    for g in groups:
        flat += g if isinstance(g, list) else [g]
    return dict(key=key, name=name, kind=kind, concept=concept, use=use, groups=flat)

_DART = [(-.26, -.62), (.26, -.62), (.26, .34), (0, .92), (-.26, .34)]
_C45 = [(math.cos(math.radians(a)), math.sin(math.radians(a)), a) for a in range(0, 360, 45)]

MODELS = [
    # ------------------------------------------------------------------ item pickups
    M("spark_ring", "Spark Ring", "item", "A charged hoop throwing eight sparks: the ring of sparks a nair hit pops.",
      "pickup prompt, item strip chip, new-item flash, build page; item id 1",
      [G("volt", R((0, 0), .5, .2), [P(kite((.64*c, .64*s), a, .44 if i % 2 == 0 else .27, .13, .38), "h", bev=.5) for i, (c, s, a) in enumerate(_C45)])]),
    M("moon_drops", "Moon Drops", "item", "A crescent moon with a little moon riding its orbit: two moons that circle you.",
      "pickup prompt, item strip, build page; item id 15 (also the orbital's own body if the sim's moons get a model)",
      [G("moon", crescent((-.14, .12), .78, .8, .4, -28), B((.72, -.62), .25, "h")),
       G("violet", R((0, .02), 1.0, .08, 222, 300, "h")),
       G("gold", spark4((.66, .74), .22, .07, role="h"), spark4((-.86, -.66), .14, .05, role="h"))]),
    M("twin_shot", "Twin Shot", "item", "Two darts flying side by side, the second a paler copy: every shot gets a twin.",
      "pickup prompt, item strip, build page; item id 39; the synergy banner for doubled shots",
      [G("pink",
         P(rot(mov(_DART, -.36, .2), -42)),
         P(rot(mov(rect(0, -.3, .52, .13), -.36, .2), -42), "i"),
         P(rot(mov(kite((0, -.62), 270, .5, .2), -.36, .2), -42), "h", bev=.5),
         P(rot(mov(scl(_DART, .74), .42, -.3), -42), "h"),
         P(rot(mov(scl(kite((0, -.62), 270, .5, .2), .74), .42, -.3), -42), "h", bev=.5))]),
    M("bouncy", "Bouncy", "item", "A ball skipping along the floor, each hop higher than the last: shots that bounce.",
      "pickup prompt, item strip, build page; item id 42",
      [G("stone", P(rect(0, -.88, 1.84, .22))),
       G("green", B((.6, -.02), .42), R((-.74, -.77), .24, .11, 8, 172, "h"), R((-.12, -.77), .36, .11, 6, 174, "h"), R((.78, -.77), .52, .11, 138, 176, "h"))]),
    M("chain_spark", "Chain Spark", "item", "Two links joined by a lightning bolt in place of the middle link: a hit that arcs on to the next foe.",
      "pickup prompt, item strip, build page; item id 38; the synergy banner for arcs",
      [G("sky", R((-.64, -.4), .36, .17), R((.64, .4), .36, .17)),
       G("volt", [bar(a, b, .17, role="h", z=(.02, .34), bev=.6) for a, b in (((-.64, -.4), (-.02, .26)), ((-.02, .26), (.02, -.26)), ((.02, -.26), (.64, .4)))],
         spark4((-.9, .7), .2, .07, role="h"), spark4((.92, -.66), .16, .06, role="h"))]),
    M("ice_glass", "Ice Glass", "item", "A tall pane of ice with three splinters flying off it: shots that shatter in three.",
      "pickup prompt, item strip, build page; item id 37 (the splinter is the PJ_SHARD shape)",
      [G("ice", P(mov([(-.22, 1.0), (.3, .62), (.34, -.5), (.02, -1.0), (-.5, -.5), (-.6, .42)], -.22, 0), bev=.7),
         bar((-.64, .26), (-.3, .6), .09, role="i"), bar((-.56, -.12), (-.1, .36), .07, role="i"),
         P([(.42, .56), (.98, .8), (.62, .3)], "h", bev=.6), P([(.5, .06), (1.04, .0), (.66, -.26)], "h", bev=.6), P([(.4, -.44), (.9, -.8), (.5, -.82)], "h", bev=.6))]),
    M("gravity_well", "Gravity Well", "item", "Three rings narrowing down a funnel to a dark core, two motes falling in: bursts that pull foes in.",
      "pickup prompt, item strip, build page; item id 40",
      [G("violet", R((0, .56), .92, .17, tilt=40), R((0, .02), .6, .16, tilt=40), R((0, -.42), .32, .15, tilt=40, role="h")),
       G("ink", B((0, -.86), .22)),
       G("pink", B((.98, .06), .1, "h"), B((-.84, -.3), .08, "h"))]),
    M("big_bang", "Big Bang", "item", "A white-hot core, eight fat rays and a broken shock ring: bursts twice as big.",
      "pickup prompt, item strip, build page; item id 41; the devil-deal card",
      [G("orange", B((0, 0), .4, "h", z=.1), [P(kite((.26*c, .26*s), a, .62 if i % 2 == 0 else .42, .2, .42), bev=.62) for i, (c, s, a) in enumerate(_C45)],
         [R((0, 0), 1.04, .11, a, a + 46, "h") for a in (22, 112, 202, 292)])]),
    M("spare_heart", "Spare Heart", "item", "A full heart with a plus on its shoulder: one more stock slot, filled.",
      "pickup prompt, item strip, build page; item id 43; the shop's stock-slot row",
      [G("heart", heart(-.06, -.12, 1.08), bar((-.74, .12), (-.5, .38), .1, role="i")),
       G("moon", P(rect(.72, .72, .56, .19), "h", z=(.1, .4), bev=.8), P(rect(.72, .72, .19, .56), "h", z=(.1, .4), bev=.8))]),
    # ------------------------------------------------------------------ act-map nodes / room markers
    M("node_fight", "Fight", "node", "Two crossed swords: a plain fight (a duel or a crowd).",
      "act map fight nodes (rows 0, 2, 4), the road screen, the pause Run page",
      sword(40, -.06) + sword(-40, .1)),
    M("node_elite", "Elite", "node", "A pale skull with red horns: a fight that hits back.",
      "act map elite nodes, the elite's card header (\"ELITE CARRIES\")",
      [G("moon", P(ngon(0, .06, .62, 8, 22.5)), P(rect(0, -.56, .6, .42))),
       G("ink", P(ngon(-.25, .02, .17, 6), z=FRONT), P(ngon(.25, .02, .17, 6), z=FRONT), P([(0, -.2), (.09, -.38), (-.09, -.38)], z=FRONT),
         P(rect(-.1, -.64, .05, .24), z=FRONT), P(rect(.1, -.64, .05, .24), z=FRONT)),
       G("crimson", horn(-1, (-.5, .36)), horn(1, (.5, .36)))]),
    M("node_boss", "Boss", "node", "A three-point crown set with red gems: the act's boss.",
      "act map boss node (row 6), the boss card header, the boss-drop banner",
      [G("gold", P(rect(0, -.5, 1.56, .4)), P(rect(0, -.82, 1.7, .17), "h"),
         P([(-.78, -.32), (-.9, .5), (-.3, -.32)], bev=.7), P([(-.4, -.32), (0, .84), (.4, -.32)], bev=.7), P([(.78, -.32), (.9, .5), (.3, -.32)], bev=.7),
         B((-.9, .58), .13, "h"), B((0, .94), .15, "h"), B((.9, .58), .13, "h")),
       G("crimson", P(ngon(0, -.5, .19, 4, 0), "h", z=FRONT, bev=.5), P(ngon(-.5, -.5, .12, 4, 0), "h", z=FRONT, bev=.5), P(ngon(.5, -.5, .12, 4, 0), "h", z=FRONT, bev=.5))]),
    M("node_item", "Item room", "node", "A gem floating over a stone pedestal: a free item.",
      "act map item-room nodes (row 1, 3), the item-room banner, the pedestal's stone in the world",
      [G("stone", P(rect(0, -.94, 1.3, .2)), P([(-.3, -.84), (.3, -.84), (.42, -.42), (-.42, -.42)]), P(rect(0, -.34, 1.1, .18), "h")),
       G("gold", P([(0, .98), (.44, .46), (0, -.06), (-.44, .46)], bev=.55), P([(0, .7), (.2, .46), (0, .22), (-.2, .46)], "i", bev=.6),
         spark4((.76, .82), .17, .06, role="h"), spark4((-.74, .22), .13, .05, role="h"))]),
    M("node_shop", "Shop", "node", "A striped awning over a counter with a coin on it: spend coins.",
      "act map shop node, the shop screen header",
      [G("teal", [P([(x - .19, .8), (x + .19, .8), (x + .19, .38), (x, .22), (x - .19, .38)], bev=.8) for x in (-.76, 0, .76)], P(rect(0, .88, 1.96, .14), "h")),
       G("cream", [P([(x - .19, .8), (x + .19, .8), (x + .19, .38), (x, .22), (x - .19, .38)], bev=.8) for x in (-.38, .38)]),
       G("wood", P(rect(-.84, -.16, .13, .86)), P(rect(.84, -.16, .13, .86)), P(rect(0, -.76, 1.9, .38))),
       G("gold", P(ngon(0, -.26, .31, 10), z=(-.12, .12)), P(rect(0, -.26, .09, .3), "i", z=(.08, .2)))]),
    M("node_campfire", "Campfire", "node", "A flame on two crossed logs: refill your stocks.",
      "act map campfire nodes (the row before the boss), the campfire banner and room",
      [G("wood", bar((-.86, -.94), (.7, -.5), .27, z=(-.26, .08)), bar((.86, -.94), (-.7, -.5), .27, z=(-.1, .26))),
       G("orange", P(flame(0, .14, .56, .84, -.2)), P(flame(.04, -.12, .25, .44, 0), "i"), P(flame(-.5, -.28, .19, .34, .15), "h"),
         B((.62, .66), .07, "h"), B((.4, 1.0), .05, "h"))]),
    M("node_devil", "Devil deal", "node", "A red arched door with horns and two burning eyes in its dark: a deal paid in stock slots.",
      "act map door node (row 7) when the red door opens, the devil-deal banner, the door's chance tag",
      [G("crimson", P(arch(0, -.98, 1.24, 1.72)), horn(-1, (-.48, .3), role="h"), horn(1, (.48, .3), role="h")),
       G("ink", P(arch(0, -.98, .82, 1.4), z=(.1, .3), bev=.86)),
       G("orange", P([(-.3, .0), (-.06, -.16), (-.3, -.16)], "h", z=(.26, .4), bev=.6), P([(.3, .0), (.06, -.16), (.3, -.16)], "h", z=(.26, .4), bev=.6))]),
    M("node_angel", "Angel room", "node", "A pale glowing door with two wings and a halo over it: one free, strong item.",
      "act map door node (row 7) when the white door opens, the angel-room banner",
      [G("cream", P(arch(0, -.98, 1.0, 1.5)), P(arch(0, -.98, .6, 1.16), "i", bev=.86)),
       G("moon", [P(kite((s*.42, -.3), 90 - s*a, L, .16), "h", bev=.6) for s in (1, -1) for a, L in ((74, .7), (50, .84), (24, .7))]),
       G("gold", R((0, .9), .38, .1, tilt=44, role="h", zh=.05))]),
    M("node_secret", "Secret room", "node", "A brick wall split by a glowing crack: hit it three times.",
      "act map marker on the fight node that holds the cracked wall, the secret-room banner, the wall itself",
      [G("stone", P(rect(-.5, .56, .94, .46)), P(rect(.5, .56, .94, .46)), P(rect(-.76, 0, .42, .46)), P(rect(0, 0, .94, .46)), P(rect(.76, 0, .42, .46)),
         P(rect(-.5, -.56, .94, .46)), P(rect(.5, -.56, .94, .46))),
       G("violet", [bar(a, b, .15, z=(.0, .36), bev=.6) for a, b in (((.16, .84), (-.14, .44)), ((-.14, .44), (.18, .1)), ((.18, .1), (-.12, -.32)), ((-.12, -.32), (.1, -.84)))],
         bar((.18, .1), (.5, .24), .1, z=(.0, .34), bev=.6), bar((-.12, -.32), (-.44, -.42), .1, z=(.0, .34), bev=.6))]),
    # ------------------------------------------------------------------ stocks
    M("heart_container", "Heart container", "hud", "An empty heart outline: a stock slot with nothing in it.",
      "the stock row (limit above stocks held), the devil deal's slot price, the shop's stock-slot row",
      [G("stone", R((-.4, .2), .4, .18, 0, 225), R((.4, .2), .4, .18, -45, 180), bar((-.69, -.07), (0, -.86), .18), bar((.69, -.07), (0, -.86), .18),
         B((0, -.84), .1), B((0, .2), .09))]),
    M("heart_full", "Heart", "hud", "A plain full heart with a glint: one stock held.",
      "the stock row, the campfire's refill tick, co-op per-player stocks",
      [G("heart", heart(0, -.1, 1.12), bar((-.72, .16), (-.46, .44), .1, role="i"))]),
    # ------------------------------------------------------------------ pool frames (drawn behind an item, same frame)
    M("frame_item", "Frame: item room", "frame", "A plain cut-corner frame in stone: an item-room find.",
      "behind an item on its card, pickup prompt and build page (pool `item`)",
      [G("stone", frame_ring())]),
    M("frame_shop", "Frame: shop", "frame", "The frame in teal with a coin set in its foot: bought.",
      "behind a shop item (pool `shop`); the price tag's anchor",
      [G("teal", frame_ring()), G("gold", P(ngon(0, -1.1, .24, 10), z=(-.36, -.08)), P(rect(0, -1.1, .07, .22), "i", z=(-.1, .0)))]),
    M("frame_boss", "Frame: boss", "frame", "The frame in gold with three crown points on top: a boss drop.",
      "behind a boss-pool item; the boss-drop pedestals",
      [G("gold", frame_ring(), P([(-.62, 1.14), (-.5, 1.42), (-.22, 1.14)], "h", z=BACK, bev=.7), P([(-.24, 1.14), (0, 1.5), (.24, 1.14)], "h", z=BACK, bev=.7),
         P([(.62, 1.14), (.5, 1.42), (.22, 1.14)], "h", z=BACK, bev=.7))]),
    M("frame_devil", "Frame: devil", "frame", "The frame in dark red with two horns and a spade point below: a deal.",
      "behind a devil-pool item; the deal pedestal and its slot price",
      [G("crimson", frame_ring(), P([(-1.14, .8), (-1.46, 1.44), (-.76, 1.14)], "h", z=BACK, bev=.7), P([(1.14, .8), (1.46, 1.44), (.76, 1.14)], "h", z=BACK, bev=.7),
         P([(-.24, -1.14), (.24, -1.14), (0, -1.56)], "h", z=BACK, bev=.7))]),
    M("frame_angel", "Frame: angel", "frame", "The frame in cream with a halo over it and a feather at each shoulder: a gift.",
      "behind an angel-pool item; the angel room's pedestal",
      [G("cream", frame_ring(), [P(kite((s*1.1, .62), 90 - s*a, L, .12), "h", z=BACK, bev=.6) for s in (1, -1) for a, L in ((62, .5), (30, .44))]),
       G("gold", R((0, 1.24), .4, .1, tilt=46, role="h", z=-.3, zh=.05))]),
    M("frame_secret", "Frame: secret", "frame", "The frame in violet, broken into dashes: found behind a wall.",
      "behind a secret-pool item; the secret room's pedestal",
      [G("violet", frame_ring(keep=lambda i: i % 6 not in (1, 3), sub=5))]),
]
KEYS = [m["key"] for m in MODELS]
BY_KEY = {m["key"]: m for m in MODELS}
def span_of(model): return FRAME_SPAN if model["kind"] == "frame" else SPAN
assert len(set(KEYS)) == len(KEYS)


def mesh_of(model):
    m = DP.Mesh()
    for pal, parts in model["groups"]:
        m.extend(DP.build(parts, pal))
    return m


# ---------------------------------------------------------------------------------------------- checks
def convex(pts):
    """Whether the fan from pts[0] (how Mesh.poly triangulates) covers the polygon: every fan triangle turns the same
    way. True for a convex polygon and for the mildly leaning flame, whose tip is pts[0]."""
    s = 0
    for i in range(1, len(pts)-1):
        a, b, c = pts[0], pts[i], pts[i+1]
        cr = (b[0]-a[0])*(c[1]-a[1]) - (b[1]-a[1])*(c[0]-a[0])
        if abs(cr) < 0.03:      # a sliver (the leaning flame's shoulder, as the Ember drive has)
            continue
        if s and (cr > 0) != (s > 0):
            return False
        s = cr
    return True


def self_test(meshes):
    bad = []
    for model in MODELS:
        k = model["key"]; m = meshes[k]
        for pal, parts in model["groups"]:
            for p in parts:
                if p["k"] == "P" and not convex(p["pts"]):
                    bad.append("%s: a prism polygon does not fan from its first point: %s" % (k, p["pts"]))
        if len(m.tris) < 16:
            bad.append("%s: only %d triangles" % (k, len(m.tris)))
        for p0, p1, p2, t, rw in m.tris:
            if rw not in ROW_ORDER:
                bad.append("%s: row %s is not in this atlas" % (k, rw)); break
            if not all(abs(c) <= 1.9 for p in (p0, p1, p2) for c in p):
                bad.append("%s: outside the 1.9 box (the drives' bound)" % k); break
            if not 0 <= t <= 1.0001:
                bad.append("%s: ramp %.2f" % (k, t)); break
        lim = span_of(model) - .1      # the sprite keeps a margin for the ink edge
        far = max(max(abs(p[0]), abs(p[1]*math.cos(PITCH) - p[2]*math.sin(PITCH))) for t3 in m.tris for p in t3[:3])
        if far > lim:
            bad.append("%s: reaches %.2f, the sprite shows %.2f" % (k, far, lim))
        vol = sum(DP.dot(p0, DP.cross(p1, p2)) for p0, p1, p2, _, _ in m.tris) / 6
        if vol <= 0:
            bad.append("%s: inward winding (volume %.3f)" % (k, vol))
    if bad:
        raise SystemExit("self-test FAILED:\n  " + "\n  ".join(bad))


# ---------------------------------------------------------------------------------------------- file writers
def atlas_bytes():
    base = bytearray(AW*AH*4); glow = bytearray(AW*AH*4)
    for y in range(AH):
        ri = y // ARH
        name = ROW_ORDER[ri] if ri < len(ROW_ORDER) else None
        for x in range(AW):
            o = (y*AW + x)*4
            base[o+3] = glow[o+3] = 255
            if name is None:
                continue
            t = x / (AW-1)
            c = DP.ramp(name, t); g = DP.ROWS[name][2]; k = glow_k(name, t)
            base[o:o+3] = bytes(int(v) for v in c)
            glow[o:o+3] = bytes(int(v*k) for v in g)
    return bytes(base), bytes(glow)


def gxtex_bytes(rgba, w, h):
    """GXTX v1, RGBA8, one level (make_drives.write_gxtex_wh): 4x4 tiles, 16 AR pairs then 16 GB pairs."""
    image = bytearray()
    for ty in range(0, h, 4):
        for tx in range(0, w, 4):
            ar, gb = bytearray(), bytearray()
            for y in range(4):
                for x in range(4):
                    o = ((ty+y)*w + tx + x)*4
                    ar += bytes((rgba[o+3], rgba[o])); gb += bytes((rgba[o+1], rgba[o+2]))
            image += ar + gb
    head = struct.pack(">4sIIIIIIIIII", b"GXTX", 1, 6, w, h, 0xFFFFFFFF, 0, len(image), 0, 64, 64 + len(image))
    return head + bytes(64 - len(head)) + bytes(image)


def gxmesh_bytes(mesh):
    """GXMS v2 (make_drives.write_mesh + party_vertices): flat triangles, position, uv into the atlas, normal."""
    verts = []
    for p0, p1, p2, t, rw in mesh.tris:
        p0, p1, p2 = DP.mul(p0, HALF), DP.mul(p1, HALF), DP.mul(p2, HALF)
        cr = DP.cross(DP.sub(p1, p0), DP.sub(p2, p0))
        n = DP.norm(cr) if DP.dot(cr, cr) > 1e-12 else (0, 1, 0)
        u = (3 + t*(AW-6)) / AW
        v = (ROW_ORDER.index(rw)*ARH + ARH/2) / AH
        for p in (p0, p1, p2):
            verts.append((p[0], p[1], p[2], u, v, n[0], n[1], n[2]))
    ext = [max(c) - min(c) or 0.1 for c in ([v[k] for v in verts] for k in range(3))]
    voff = 36
    blob = struct.pack(">4sIIIfffII", b"GXMS", 2, len(verts), len(verts), ext[0], ext[1], ext[2], voff, voff + len(verts)*32)
    return blob + b"".join(struct.pack(">8f", *v) for v in verts) + b"".join(struct.pack(">H", i) for i in range(len(verts)))


# ---------------------------------------------------------------------------------------------- sprites (numpy + Pillow)
LIGHT = (0.35, 0.6, 0.72)      # drive_draw's light; shade = base x (0.42 + 0.7 diffuse) + glow x 0.8
PITCH = 0.35                   # drive_draw's sprite pitch


def raster(mesh, size, ss=4, span=SPAN, yaw=0.0, edge=0.03):
    """The model as an RGBA image: z-buffered flat triangles shaded as drive_draw shades them, then an ink edge
    (`edge` of the image width) around the silhouette, as a Party sticker has."""
    import numpy as np
    from PIL import Image, ImageFilter
    S = size*ss
    col = np.zeros((S, S, 3), np.float32); zb = np.full((S, S), -1e9, np.float32); hit = np.zeros((S, S), bool)
    cp, sp, cy, sy = math.cos(PITCH), math.sin(PITCH), math.cos(yaw), math.sin(yaw)
    def xf(p):
        x, y, z = p[0]*cy + p[2]*sy, p[1], -p[0]*sy + p[2]*cy
        return (x, y*cp - z*sp, y*sp + z*cp)
    k = S / (2*span)
    for p0, p1, p2, t, rw in mesh.tris:
        a, b, c = xf(p0), xf(p1), xf(p2)
        n = DP.cross(DP.sub(b, a), DP.sub(c, a))
        if DP.dot(n, n) < 1e-14:
            continue
        n = DP.norm(n)
        if n[2] <= 0:
            continue          # back face
        dif = max(0.0, DP.dot(n, LIGHT))
        base = DP.ramp(rw, t); g = DP.ROWS[rw][2]; gk = glow_k(rw, t)
        rgb = [min(255.0, base[i]*(0.42 + 0.7*dif) + g[i]*gk*0.8) for i in range(3)]
        sx = [S/2 + q[0]*k for q in (a, b, c)]; syy = [S/2 - q[1]*k for q in (a, b, c)]; zz = [q[2] for q in (a, b, c)]
        x0, x1 = max(0, int(math.floor(min(sx)))), min(S-1, int(math.ceil(max(sx))))
        y0, y1 = max(0, int(math.floor(min(syy)))), min(S-1, int(math.ceil(max(syy))))
        if x1 < x0 or y1 < y0:
            continue
        area = (sx[1]-sx[0])*(syy[2]-syy[0]) - (sx[2]-sx[0])*(syy[1]-syy[0])
        if abs(area) < 1e-9:
            continue
        X, Y = np.meshgrid(np.arange(x0, x1+1, dtype=np.float32) + .5, np.arange(y0, y1+1, dtype=np.float32) + .5)
        e = lambda i, j: ((sx[j]-sx[i])*(Y-syy[i]) - (syy[j]-syy[i])*(X-sx[i])) / area
        w0, w1, w2 = e(1, 2), e(2, 0), e(0, 1)
        z = w0*zz[0] + w1*zz[1] + w2*zz[2]
        sub_z = zb[y0:y1+1, x0:x1+1]
        m = (w0 >= 0) & (w1 >= 0) & (w2 >= 0) & (z > sub_z)
        sub_z[m] = z[m]
        col[y0:y1+1, x0:x1+1][m] = rgb
        hit[y0:y1+1, x0:x1+1][m] = True
    alpha = Image.fromarray((hit*255).astype(np.uint8), "L")
    body = Image.fromarray(col.clip(0, 255).astype(np.uint8), "RGB").convert("RGBA"); body.putalpha(alpha)
    r = max(1, round(S*edge))
    grown = alpha.filter(ImageFilter.MaxFilter(2*r + 1)).filter(ImageFilter.GaussianBlur(ss*.35))
    out = Image.new("RGBA", (S, S), INK + (0,)); out.putalpha(grown)
    out.alpha_composite(body)
    return out.resize((size, size), Image.LANCZOS)


def png_bytes(im):
    b = io.BytesIO()
    im.save(b, "PNG", optimize=False, compress_level=9)
    return b.getvalue()


def contact_sheet(meshes, path):
    """Each model four ways: 3/4 view at 176 px, the shipped sprite at 96 px, and the sprite at game size (48 px) on
    the dark camp colour and on the cream card colour."""
    from PIL import Image, ImageDraw, ImageFont
    cols, cw, ch = 4, 440, 236
    rows = (len(MODELS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols*cw + 16, rows*ch + 64), (36, 26, 94, 255))
    d = ImageDraw.Draw(sheet)
    try:
        f1 = ImageFont.truetype("arialbd.ttf", 17); f2 = ImageFont.truetype("arial.ttf", 12); f0 = ImageFont.truetype("arialbd.ttf", 22)
    except OSError:
        f1 = f2 = f0 = ImageFont.load_default()
    d.text((16, 14), "Envoy models, chunk 1  (%d models; make_models1.py)   left: 3/4 view   middle: the shipped sprite   right: at game size on dark and on a card" % len(MODELS), font=f2, fill=(220, 214, 255))
    d.text((16, 30), "ENVOY MODELS 1", font=f0, fill=(255, 216, 74))
    for i, model in enumerate(MODELS):
        x, y = 8 + (i % cols)*cw, 64 + (i // cols)*ch
        mesh = meshes[model["key"]]
        d.rounded_rectangle([x + 4, y + 4, x + cw - 6, y + ch - 6], 12, fill=(48, 36, 120, 255))
        sp = span_of(model)
        sheet.alpha_composite(raster(mesh, 176, span=sp*1.08, yaw=-0.5, edge=0.0), (x + 10, y + 34))
        spr = raster(mesh, SPRITE, span=sp)
        sheet.alpha_composite(spr, (x + 196, y + 46))
        small = spr.resize((48, 48), Image.LANCZOS)
        sheet.alpha_composite(small, (x + 312, y + 46))
        d.rounded_rectangle([x + 370, y + 40, x + 428, y + 98], 8, fill=(251, 246, 221, 255))
        sheet.alpha_composite(small, (x + 375, y + 45))
        tiny = spr.resize((24, 24), Image.LANCZOS)
        sheet.alpha_composite(tiny, (x + 324, y + 112))
        sheet.alpha_composite(tiny, (x + 387, y + 112))
        d.text((x + 16, y + 10), model["name"], font=f1, fill=(255, 255, 255))
        d.text((x + 196, y + 14), "em_%s  %s  %d tris" % (model["key"], model["kind"], len(mesh.tris)), font=f2, fill=(190, 182, 240))
        words, line, ty = model["concept"].split(), "", y + 150
        for w in words:
            if d.textlength(line + " " + w, font=f2) > 228:
                d.text((x + 196, ty), line, font=f2, fill=(220, 214, 255)); ty += 15; line = w
            else:
                line = (line + " " + w).strip()
        d.text((x + 196, ty), line, font=f2, fill=(220, 214, 255))
    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(path, "PNG")
    print("contact sheet: %s (%d x %d)" % (path, sheet.width, sheet.height))


# ---------------------------------------------------------------------------------------------- main
def outputs(sprites=True):
    """path -> bytes for every generated file."""
    meshes = {m["key"]: mesh_of(m) for m in MODELS}
    self_test(meshes)
    out = {}
    base, glow = atlas_bytes()
    out[OUT_MODELS / "envoy_models_atlas.gxtex"] = gxtex_bytes(base, AW, AH)
    out[OUT_MODELS / "envoy_models_atlas.glow.gxtex"] = gxtex_bytes(glow, AW, AH)
    for k in KEYS:
        out[OUT_MODELS / ("em_%s.gxmesh" % k)] = gxmesh_bytes(meshes[k])
        out[OUT_MODELS / ("em_%s.coll.json" % k)] = b'{"version":1,"atlas":"envoy_models_atlas","alpha":0,"lines":[]}\n'
        out[OUT_MODELS / ("em_%s.material.json" % k)] = b'{"builtin": "lit", "emission": 1.0}\n'
    out[OUT_MODELS / "envoy_models.txt"] = "".join(k + "\n" for k in KEYS).encode()
    if sprites:
        for k in KEYS:
            out[OUT_ART / ("em_%s.png" % k)] = png_bytes(raster(meshes[k], SPRITE, span=span_of(BY_KEY[k])))
    return meshes, out


def main():
    args = sys.argv[1:]
    meshes, out = outputs(sprites="--no-sprites" not in args)
    if "--check" in args:
        # git may check the text files out with CRLF; the binaries must match to the byte
        text = lambda p: p.suffix in (".json", ".txt")
        same = lambda p, b: (p.read_bytes().replace(b"\r\n", b"\n") if text(p) else p.read_bytes()) == b
        diff = [str(p) for p, b in out.items() if not p.exists() or not same(p, b)]
        if diff:
            raise SystemExit("out of date:\n  " + "\n  ".join(diff))
        print("check ok: %d files match" % len(out))
        return
    OUT_MODELS.mkdir(parents=True, exist_ok=True)
    for p, b in out.items():
        p.write_bytes(b)
    for m in MODELS:
        print("em_%-16s %-5s %4d triangles" % (m["key"], m["kind"], len(meshes[m["key"]].tris)))
    print("%d models, %d triangles, atlas %dx%d (%d rows), %d files; self-test ok"
          % (len(MODELS), sum(len(m.tris) for m in meshes.values()), AW, AH, len(ROW_ORDER), len(out)))
    if "--sheet" in args:
        contact_sheet(meshes, Path(args[args.index("--sheet") + 1]))


if __name__ == "__main__":
    main()
