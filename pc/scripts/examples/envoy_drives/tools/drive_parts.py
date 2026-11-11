#!/usr/bin/env python3
"""The Envoy drive part lists (Party pack, DRIVE-MODELS.md step 9): one definition per drive (46) and per Unique (16),
plus the mesh primitives that turn a part list into a model (prism, ring_xy, ball) and the glyph writer that turns the
same list into the sticker glyph. Both `make_drives.py` (the game's meshes) and the design sheet generator
(`menu/out_envoy/build_step9.py`) import this file, so the model and the glyph cannot drift.

Extracted verbatim from build_step9.py (2026-11-11); edit the part lists HERE from now on.
Every coordinate is original geometry written for this project. Nothing is traced or adapted from any game.
Python standard library only.
"""
import math

SUN, INK = "#FFD84A", "#2A1A6E"
HOLO = "#FF8FB8,#FFE066,#8CF0C8,#7FD4FF,#C9A0FF,#FFB27A"

# ---------------------------------------------------------------------------------------------- vectors, mesh
def sub(a, b): return (a[0]-b[0], a[1]-b[1], a[2]-b[2])
def add(a, b): return (a[0]+b[0], a[1]+b[1], a[2]+b[2])
def mul(a, s): return (a[0]*s, a[1]*s, a[2]*s)
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def dot(a, b): return a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
def norm(a):
    n = math.sqrt(dot(a, a)) or 1.0
    return (a[0]/n, a[1]/n, a[2]/n)
def mid(pts): return tuple(sum(p[k] for p in pts)/len(pts) for k in range(3))
def hexrgb(h): return tuple(int(h[i:i+2], 16) for i in (1, 3, 5))
def mix(a, b, t): return tuple(a[i] + (b[i]-a[i])*t for i in range(3))


class Mesh:
    """The generator's mesh builder (tri, poly, prism) with the two primitives this step adds: ring_xy and ball."""
    def __init__(self):
        self.tris = []   # (p0, p1, p2, t, rowname)

    def tri(self, pts, t, rw, centre):
        a, b, c = pts
        n = cross(sub(b, a), sub(c, a))
        if dot(n, sub(mid(pts), centre)) < 0:
            pts = (a, c, b)
        self.tris.append((pts[0], pts[1], pts[2], t, rw))

    def poly(self, pts, t, rw, centre):
        for i in range(1, len(pts)-1):
            self.tri((pts[0], pts[i], pts[i+1]), t, rw, centre)

    def prism(self, poly2d, z0, z1, front, rows, ts):
        cx = sum(p[0] for p in poly2d)/len(poly2d); cy = sum(p[1] for p in poly2d)/len(poly2d)
        bk = [(p[0], p[1], z0) for p in poly2d]
        ft = [(cx + (p[0]-cx)*front, cy + (p[1]-cy)*front, z1) for p in poly2d]
        ctr = (cx, cy, (z0+z1)/2)
        n = len(poly2d)
        for i in range(n):
            j = (i+1) % n
            self.tri((bk[i], bk[j], ft[j]), ts[1], rows[0], ctr)
            self.tri((bk[i], ft[j], ft[i]), ts[1], rows[0], ctr)
        self.poly(ft, ts[2], rows[1], ctr)
        self.poly(bk, ts[0], rows[0], ctr)

    def ring_xy(self, c, r, w, a0, a1, segs, zh, rw, t, tilt=0.0, z=0.0):
        """NEW. A ring (or an arc of one) facing the camera: flat front face, two slanted back faces (6 triangles a
        segment). `tilt` leans it back about its own X axis (a ring seen from above)."""
        ct, st = math.cos(math.radians(tilt)), math.sin(math.radians(tilt))
        def P(a, rr, zz):
            x, y = rr*math.cos(a), rr*math.sin(a)
            return (c[0] + x, c[1] + y*ct - zz*st, z + y*st + zz*ct)
        full = abs(a1 - a0) >= 359.9
        for i in range(segs):
            b0 = math.radians(a0 + (a1-a0)*i/segs); b1 = math.radians(a0 + (a1-a0)*(i+1)/segs)
            ctr = mid([P(b0, r, 0), P(b1, r, 0)])
            q = [(P(b0, r-w/2, zh), P(b1, r-w/2, zh), P(b1, r+w/2, zh), P(b0, r+w/2, zh), t),
                 (P(b0, r+w/2, zh), P(b1, r+w/2, zh), P(b1, r, -zh), P(b0, r, -zh), t*.55),
                 (P(b0, r, -zh), P(b1, r, -zh), P(b1, r-w/2, zh), P(b0, r-w/2, zh), t*.55)]
            for p0, p1, p2, p3, tt in q:
                self.tri((p0, p1, p2), tt, rw, ctr); self.tri((p0, p2, p3), tt, rw, ctr)
            if not full and i in (0, segs-1):
                b = b0 if i == 0 else b1
                self.tri((P(b, r-w/2, zh), P(b, r+w/2, zh), P(b, r, -zh)), t*.8, rw, ctr)

    def ball(self, c, r, rw, t):
        """NEW as a helper (the generator inlines it): an icosahedron."""
        p = (1 + math.sqrt(5))/2
        v = [norm(x) for x in [(-1, p, 0), (1, p, 0), (-1, -p, 0), (1, -p, 0), (0, -1, p), (0, 1, p), (0, -1, -p),
                               (0, 1, -p), (p, 0, -1), (p, 0, 1), (-p, 0, -1), (-p, 0, 1)]]
        f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6),
             (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11), (6, 2, 10),
             (8, 6, 7), (9, 8, 1)]
        for a, b, d in f:
            self.tri(tuple(add(c, mul(v[i], r)) for i in (a, b, d)), t, rw, c)

    def ring_y(self, radius, thick, segs, rw, t=.7, y=0.0, tiltz=0.0):
        """The generator's ring_overlay: a thin ring about the vertical axis (the tier ring)."""
        sec = [(0.0, thick), (thick*0.9, -thick*0.5), (-thick*0.9, -thick*0.5)]
        cz, sz = math.cos(tiltz), math.sin(tiltz)
        def P(a, s):
            r = radius + sec[s][0]
            x, yy, z = r*math.cos(a), sec[s][1], r*math.sin(a)
            return (x*cz - yy*sz, x*sz + yy*cz + y, z)
        for i in range(segs):
            a0, a1 = 2*math.pi*i/segs, 2*math.pi*(i+1)/segs
            ctr = mid([P(a0, 0), P(a0, 1), P(a0, 2), P(a1, 0), P(a1, 1), P(a1, 2)])
            for s in range(3):
                u = (s+1) % 3
                rr = rw if isinstance(rw, str) else rw[i % len(rw)]
                self.tri((P(a0, s), P(a1, s), P(a1, u)), t, rr, ctr); self.tri((P(a0, s), P(a1, u), P(a0, u)), t, rr, ctr)

    def scaled(self, s, off=(0, 0, 0)):
        m = Mesh()
        m.tris = [(add(mul(a, s), off), add(mul(b, s), off), add(mul(c, s), off), t, rw) for a, b, c, t, rw in self.tris]
        return m

    def rerow(self, fn):
        m = Mesh()
        m.tris = [(a, b, c, t, fn(rw)) for a, b, c, t, rw in self.tris]
        return m

    def extend(self, o):
        self.tris += o.tris
        return self


# ---------------------------------------------------------------------------------------------- atlas rows (proposed)
# name: (dark, main, glow). The six families take the Party pack colours so the orbiter matches its sticker.
ROWS = {}
def row(name, dark, main, glow=None):
    ROWS[name] = (dark, main, glow or main)

FAMS = [
    # key, word, pool family, colour, dark, hot, s4 family key
    ("fire", "Fire", "RED", "#FF7A1F", (150, 44, 8), (255, 222, 96), "fire"),
    ("frost", "Frost", "PURPLE", "#62CCF5", (22, 92, 156), (232, 249, 255), "frost"),
    ("sky", "Sky", "YELLOW", "#86D63C", (38, 104, 22), (240, 255, 130), "wind"),
    ("tempo", "Tempo", "GREEN", "#C866F2", (96, 30, 150), (255, 226, 255), "tech"),
    ("guard", "Guard", "BLUE", "#C99455", (106, 62, 28), (255, 240, 200), "stone"),
    ("util", "Utility", "WHITE", "#AEA8CC", (82, 76, 122), (255, 255, 255), "util"),
]
FAMD = {f[0]: f for f in FAMS}
for k, _, _, col, dk, hot, _ in FAMS:
    row(k, dk, hexrgb(col), hexrgb(col)); row(k + "_hot", mix(hexrgb(col), hot, .6), hot, hot)
row("white", (190, 186, 214), (255, 255, 255)); row("sun", (214, 150, 20), hexrgb(SUN))
row("ink", (20, 12, 51), hexrgb(INK))
for i, c in enumerate(HOLO.split(",")):
    row("foil%d" % i, mix(hexrgb(c), (60, 40, 120), .35), hexrgb(c))


def ramp(rw, t):
    dark, main, glow = ROWS[rw]
    c = mix(dark, main, min(1.0, (t*1.15) ** 0.8))
    return tuple(min(255.0, c[i] + glow[i]*(t**2)*.18) for i in range(3))


# ---------------------------------------------------------------------------------------------- parts: one drive, two outputs
def P(pts, role="b", z=None, bev=.72, g=1, m=1):
    return dict(k="P", pts=list(pts), role=role, z=z, bev=bev, g=g, m=m)
def R(c, r, w, a0=0, a1=360, role="b", zh=.15, tilt=0, g=1, m=1, z=0.0):
    return dict(k="R", c=c, r=r, w=w, a0=a0, a1=a1, role=role, zh=zh, tilt=tilt, g=g, m=m, z=z)
def B(c, r, role="b", g=1, m=1, z=0.0):
    return dict(k="B", c=c, r=r, role=role, g=g, m=m, z=z)

def rot(pts, ang, c=(0, 0)):
    a = math.radians(ang); cs, sn = math.cos(a), math.sin(a)
    return [(c[0] + (x-c[0])*cs - (y-c[1])*sn, c[1] + (x-c[0])*sn + (y-c[1])*cs) for x, y in pts]
def mov(pts, dx, dy): return [(x+dx, y+dy) for x, y in pts]
def scl(pts, s, c=(0, 0)): return [(c[0] + (x-c[0])*s, c[1] + (y-c[1])*s) for x, y in pts]
def mir(pts): return [(-x, y) for x, y in pts]
def ngon(cx, cy, r, n, a0=90): return [(cx + r*math.cos(math.radians(a0 + 360*i/n)), cy + r*math.sin(math.radians(a0 + 360*i/n))) for i in range(n)]
def rect(cx, cy, w, h, ang=0): return rot([(cx-w/2, cy-h/2), (cx+w/2, cy-h/2), (cx+w/2, cy+h/2), (cx-w/2, cy+h/2)], ang, (cx, cy))
def bar(p0, p1, w, **kw):
    dx, dy = p1[0]-p0[0], p1[1]-p0[1]; L = math.hypot(dx, dy) or 1; nx, ny = -dy/L*w/2, dx/L*w/2
    return P([(p0[0]+nx, p0[1]+ny), (p1[0]+nx, p1[1]+ny), (p1[0]-nx, p1[1]-ny), (p0[0]-nx, p0[1]-ny)], **kw)
def flame(cx, cy, hw, hh, lean=0.0):
    u = [(lean, 1), (.6, .42), (1, -.12), (.94, -.5), (.54, -.84), (0, -1), (-.54, -.84), (-.94, -.5), (-1, -.12), (-.6, .42)]
    return [(cx + x*hw, cy + y*hh) for x, y in u]
def kite(root, ang, L, w, at=.45):
    a = math.radians(ang); d = (math.cos(a), math.sin(a)); p = (-d[1], d[0])
    return [root, (root[0]+d[0]*at*L+p[0]*w, root[1]+d[1]*at*L+p[1]*w), (root[0]+d[0]*L, root[1]+d[1]*L),
            (root[0]+d[0]*at*L-p[0]*w, root[1]+d[1]*at*L-p[1]*w)]
def arcpts(c, r, a0, a1, n): return [(c[0] + r*math.cos(math.radians(a0 + (a1-a0)*i/n)), c[1] + r*math.sin(math.radians(a0 + (a1-a0)*i/n))) for i in range(n+1)]
def frame(outer, s, c, **kw):
    """A hollow outline: quads between a polygon and its copy shrunk by `s` about `c`."""
    inner = scl(outer, s, c); n = len(outer)
    return [P([outer[i], outer[(i+1) % n], inner[(i+1) % n], inner[i]], bev=.8, **kw) for i in range(n)]

SH = [(-.8, .85), (.8, .85), (.8, .05), (.45, -.6), (0, -.98), (-.45, -.6), (-.8, .05)]   # Guard's plate: flat top, point down
HEXC = [(0, 1.02), (.36, .62), (.36, -.62), (0, -1.02), (-.36, -.62), (-.36, .62)]


def build(parts, fam, hot=None):
    """parts -> Mesh. Body parts take the family row, 'h' and 'i' parts the hot row ('i' sits proud of the face)."""
    m = Mesh(); hot = hot or fam + "_hot"
    for p in parts:
        if not p["m"]:
            continue
        role = p["role"]
        rw = fam if role == "b" else hot
        if p["k"] == "P":
            z = p["z"] or {"b": (-.2, .2), "h": (-.24, .26), "i": (.14, .33)}[role]
            m.prism(p["pts"], z[0], z[1], p["bev"], (rw, rw), (.3, .95, .7) if role == "b" else (.4, .95, .75))
        elif p["k"] == "R":
            span = abs(p["a1"] - p["a0"])
            segs = max(2, round(14*span/360))
            m.ring_xy(p["c"], p["r"], p["w"], p["a0"], p["a1"], segs, p["zh"], rw, .8, p["tilt"], p["z"] + (.2 if role == "i" else 0))
        else:
            m.ball((p["c"][0], p["c"][1], p["z"] + (.22 if role == "i" else 0)), p["r"], rw, .78)
    return m


U = 8.0   # model unit -> glyph unit; the glyph box is about 20 across, as the family glyphs are
def _d(pts): return "M" + "L".join(f"{x*U:.2f},{-y*U:.2f}" for x, y in pts) + "Z"
def part_d(p):
    if p["k"] == "P":
        return _d(p["pts"])
    if p["k"] == "B":
        return _d(ngon(p["c"][0], p["c"][1], p["r"], 20))
    c, r, w, a0, a1 = p["c"], p["r"], p["w"], p["a0"], p["a1"]
    ct = math.cos(math.radians(p["tilt"]))
    n = max(6, round(40*abs(a1-a0)/360))
    sq = lambda pts: [(x, c[1] + (y-c[1])*ct) for x, y in pts]
    if abs(a1-a0) >= 359.9:
        return _d(sq(arcpts(c, r+w/2, 0, 360, n)[:-1])) + _d(sq(arcpts(c, r-w/2, 360, 0, n)[:-1]))
    return _d(sq(arcpts(c, r+w/2, a0, a1, n) + arcpts(c, r-w/2, a1, a0, n)))


def glyph_inner(parts, face="#fff", edge=INK, ew=2.8):
    """The sticker glyph: every body part white with one merged ink edge, 'i' parts as ink marks on the white."""
    body = [part_d(p) for p in parts if p["g"] and p["role"] != "i"]
    inks = [part_d(p) for p in parts if p["g"] and p["role"] == "i"]
    a = 'stroke-linejoin="round" fill-rule="evenodd"'
    return ("".join(f'<path d="{d}" fill="{edge}" stroke="{edge}" stroke-width="{ew}" {a}/>' for d in body)
            + "".join(f'<path d="{d}" fill="{face}" {a}/>' for d in body)
            + "".join(f'<path d="{d}" fill="{edge}" {a}/>' for d in inks))

def D(key, name, fam, tier, rule, idea, form, anim, parts):
    return dict(key=key, name=name, fam=fam, tier=tier, rule=rule, idea=idea, form=form, anim=anim, parts=parts)

def _feathers():
    return [P(kite((0, -.9), a, L, .25)) for a, L in ((56, 1.5), (90, 1.86), (124, 1.5))] + [B((0, -.8), .2, "h")]
def _arrowarc(a0, a1):
    a = math.radians(a1); p = (.7*math.cos(a), .7*math.sin(a)); t = (-math.sin(a), math.cos(a)); rr = (math.cos(a), math.sin(a))
    return [R((0, 0), .7, .26, a0, a1), P([(p[0]+rr[0]*.3, p[1]+rr[1]*.3), (p[0]+t[0]*.46, p[1]+t[1]*.46), (p[0]-rr[0]*.3, p[1]-rr[1]*.3)], bev=.6)]
def _thorn(s): return [P([(s*.5, .66), (s*1.04, .52), (s*.5, .3)], bev=.5), P([(s*.5, .12), (s*1.0, -.1), (s*.4, -.26)], bev=.5)]
def _zig(pts, w, **kw): return [bar(pts[i], pts[i+1], w, **kw) for i in range(len(pts)-1)]

DRIVES = [
    # ------------------------------------------------------------------ Fire (pool RED)
    D("ember", "Ember", "fire", 0, "Every 40% dealt: a fire burst, 6% and Burning.",
      "One flame with a hot core: the plain burst, and the family's base form.",
      "A 10-sided teardrop prism, tip leaning left, 1.2 wide by 2.0 tall, bevel 0.72; a second small teardrop proud of the face as the core.",
      "Faces the camera, rocks 12 degrees; the tip flickers (scale Y 1.0 to 1.06, 0.5 s).",
      [P(flame(0, 0, .62, .98, -.3)), P(flame(.04, -.36, .26, .46, 0), "i")]),
    D("bolt", "Bolt", "fire", 0, "Every 30% dealt: lightning strikes the nearest foe, 6%.",
      "A tall zigzag blade, sharp at both ends: it jumps to a foe.",
      "Two long triangle prisms overlapped into a zigzag, 1.3 wide by 2.0 tall, bevel 0.55.",
      "Faces the camera; snaps 8 degrees left and right every 20 frames, never eases.",
      [P([(.38, 1.0), (-.68, -.2), (.14, -.2)], bev=.6), P([(-.38, -1.0), (.68, .2), (-.14, .2)], bev=.6)]),
    D("nova", "Nova", "fire", 1, "Every 12 seconds: a blast around you, 10%. It launches.",
      "A flame between two blast arcs: the blast goes out around you.",
      "A narrow teardrop prism (0.8 by 2.0) and two ring arcs of 90 degrees, radius 0.82, left and right of it.",
      "The arcs breathe outward (radius 0.82 to 0.95) on the drive's 12-second clock and snap back on the fire.",
      [P(flame(0, 0, .38, .98, 0)), R((0, -.1), .84, .24, -44, 44), R((0, -.1), .84, .24, 136, 224)]),
    D("wildfire", "Wildfire", "fire", 1, "A cash-out on a Burning foe: 12% to every other foe.",
      "One tall flame with a small one thrown off to each side: the fire jumps to the others.",
      "A teardrop prism 0.8 by 1.9 and two small teardrops (0.44 by 0.76) standing apart from it at its foot, leaning out 20 degrees.",
      "Rocks 12 degrees; the small flames hop 0.1 out and back, out of step with each other.",
      [P(flame(0, .05, .4, .95, -.2)), P(rot(flame(-.74, -.6, .22, .38, 0), 20, (-.74, -.98)), "h"), P(rot(flame(.74, -.6, .22, .38, 0), -20, (.74, -.98)), "h")]),
    D("spire", "Spire", "fire", 1, "Every 10 seconds: a spire of light under the nearest foe.",
      "A needle on a foot plate: it rises from the floor.",
      "One tall triangle prism (0.56 wide at the foot, 1.75 tall) on a flat slab 1.3 wide; the needle uses the hot row.",
      "Faces the camera, no rock; the needle stretches 10% on the drive's 10-second clock.",
      [P([(0, 1.02), (.28, -.62), (-.28, -.62)], "h", bev=.5), P(rect(0, -.8, 1.3, .32))]),
    D("kindle", "Kindle", "fire", 0, "A smash attack: the foe is Burning.",
      "A struck match: a small flame on a stick. One hard hit lights it.",
      "A thin upright bar (0.3 by 1.1) with a teardrop prism (0.84 by 1.1) as its head.",
      "Rocks 12 degrees; the head flares (scale 1.2, 6 frames) on a smash that lands.",
      [P(rect(0, -.48, .3, 1.0)), P(flame(0, .42, .42, .58, .25))]),
    D("cinder_heart", "Cinder Heart", "fire", 1, "A Burning foe KO'd: Heat fills 2% faster, this run.",
      "A heart-shaped coal with a flame at the cleft: it keeps what it has burned.",
      "Two 8-sided discs and a triangle prism make the coal; a small teardrop stands in the cleft (hot row).",
      "Beats (scale 1.0 to 1.08, twice a second) once it holds a pip; faster at 20 pips.",
      [P(ngon(-.36, -.08, .42, 8, 22.5)), P(ngon(.36, -.08, .42, 8, 22.5)), P([(-.76, -.22), (.76, -.22), (0, -1.0)], bev=.8), P(flame(0, .56, .25, .46, 0), "h")]),
    D("solar_flare", "Solar Flare", "fire", 2, "Every 8 s while Blazing: fire falls on every foe, 8%.",
      "A rising half sun with five flame rays: fire from the sky.",
      "A half disc (radius 0.55, flat side down) and five triangle prisms fanned over it, the middle one longest.",
      "The rays lengthen one after another, left to right, on the 8-second clock; still when the wearer is not Blazing.",
      [P(arcpts((0, -.72), .56, 0, 180, 8))] + [P([(-.72*0 + .5*math.cos(math.radians(a+13)), -.72 + .5*math.sin(math.radians(a+13))),
                                                  (L*math.cos(math.radians(a)), -.72 + L*math.sin(math.radians(a))),
                                                  (.5*math.cos(math.radians(a-13)), -.72 + .5*math.sin(math.radians(a-13)))], "h", bev=.5)
                                               for a, L in ((14, 1.12), (52, 1.42), (90, 1.72), (128, 1.42), (166, 1.12))]),
    # ------------------------------------------------------------------ Frost (pool PURPLE)
    D("frost", "Frost", "frost", 0, "Every 40% dealt: 3% and the foe is Chilled.",
      "A six-armed ice crystal: the family's base form.",
      "Three long hexagonal bars (0.34 by 2.0, pointed ends) crossed at 60 degrees, and a small hexagon boss.",
      "Turns slowly in its own plane (one turn in 6 s), always facing the camera.",
      [P(rot([(0, 1), (.17, .8), (.17, -.8), (0, -1), (-.17, -.8), (-.17, .8)], a), bev=.6) for a in (0, 60, 120)] + [P(ngon(0, 0, .34, 6, 0), "h")]),
    D("shatter", "Shatter", "frost", 0, "Every 20% dealt to Chilled foes: ice shards, 7%, ending the Chill.",
      "A hexagon broken in two, the halves slipped apart.",
      "One hexagon prism (radius 0.9) cut by a slanted plane into two five-sided prisms, moved 0.36 apart; one loose shard.",
      "The halves drift 0.1 apart and back; they clap shut and fly open on the fire.",
      [P(mov([(0, .9), (-.78, .45), (-.78, -.45), (-.2, -.78), (.2, .78)], -.2, .1)), P(mov([(.2, .78), (-.2, -.78), (0, -.9), (.78, -.45), (.78, .45)], .2, -.1)),
       P([(.5, 1.0), (.84, .9), (.62, .7)], "h", g=0)]),
    D("hex", "Hex", "frost", 0, "Every 50% dealt: 3% and the foe is Marked.",
      "A hexagon cut through by a cross into four pieces: the mark it leaves.",
      "One flat-topped hexagon plate (radius 0.94, depth 0.3) cut by two planes at 45 degrees into four wedges 0.2 apart.",
      "Faces the camera, rocks 12 degrees; the four wedges close up for 10 frames when a foe is Marked.",
      [P(q, z=(-.15, .15), bev=.8) for q in ([(0, .15), (.55, .7), (.47, .814), (-.47, .814), (-.55, .7)], [(.15, 0), (.655, .505), (.94, 0), (.655, -.505)],
                                              [(0, -.15), (.55, -.7), (.47, -.814), (-.47, -.814), (-.55, -.7)], [(-.15, 0), (-.655, .505), (-.94, 0), (-.655, -.505)])]),
    D("static", "Static", "frost", 0, "Every 20% with specials: 3% and Shocked.",
      "An upright crystal with two barbs and a zigzag belt: charge held in ice.",
      "A tall six-sided column (0.72 by 2.0, pointed ends), a triangle barb on each side, a three-bar zigzag raised on the face.",
      "The barbs twitch in and out 0.06, irregularly; the zigzag blinks while a foe is Shocked.",
      [P(HEXC), P([(-.3, .6), (-1.0, .22), (-.3, .02)], bev=.5), P([(.3, -.6), (1.0, -.22), (.3, -.02)], bev=.5)]
      + _zig([(-.22, .36), (.16, .12), (-.16, -.12), (.22, -.36)], .17, role="i")),
    D("affliction", "Affliction", "frost", 1, "Each word on the foe: Heat fills x1.25.",
      "Four hexagons in a honeycomb: one cell for each word a foe can carry.",
      "Four small hexagon prisms (radius 0.34) in a diamond of honeycomb cells, 0.17 apart.",
      "Faces the camera; a cell lights (hot row) for each word on the foe being hit.",
      [P(ngon(x, y, .345, 6, 90), bev=.74) for x, y in ((0, .66), (-.38, 0), (.38, 0), (0, -.66))]),
    D("contagion", "Contagion", "frost", 1, "A word you apply: also lands on foes nearby.",
      "One crystal budding three small ones: the word spreads outward.",
      "A hexagon prism (radius 0.48) with three small hexagons (radius 0.25) on short necks at 90, 210 and 330 degrees.",
      "Turns in its own plane; the buds push out 0.15 and return when a word spreads.",
      [bar((0, -.08), (.8*math.cos(math.radians(a)), -.08 + .8*math.sin(math.radians(a))), .17) for a in (90, 210, 330)]
      + [P(ngon(0, -.08, .48, 6, 0), "h")] + [P(ngon(.8*math.cos(math.radians(a)), -.08 + .8*math.sin(math.radians(a)), .25, 6, 0)) for a in (90, 210, 330)]),
    D("steam", "Steam", "frost", 2, "A Burning and Chilled foe: damage to it fills Heat x2.5.",
      "Half a crystal as a cup, three plumes rising from it: ice meeting fire.",
      "The lower half of a hexagon prism (1.6 wide) and three two-bar plumes above it, the middle one tallest (hot row).",
      "The plumes rise and fade in turn (a 3-step loop, 0.9 s); only while a foe is both Burning and Chilled.",
      [P([(-.82, -.12), (.82, -.12), (.82, -.5), (0, -.98), (-.82, -.5)])]
      + sum([_zig([(x, .06), (x+.16, .36+h*.4), (x-.04, .66+h)], .21, role="h") for x, h in ((-.52, 0), (0, .3), (.52, 0))], [])),
    # ------------------------------------------------------------------ Sky (pool YELLOW)
    D("feather_storm", "Feather Storm", "sky", 0, "Every 30% of aerial damage: three feathers, 3% each.",
      "A fan of three feathers: the three it throws. The family's base form.",
      "Three kite prisms from one root at 56, 90 and 124 degrees, the middle one 1.86 long; a small ball at the root.",
      "The fan opens and closes 8 degrees; it snaps shut on the fire and the feathers leave.",
      _feathers()),
    D("gatekeeper", "Gatekeeper", "sky", 1, "Damage to an offstage foe fills Heat x2.",
      "An open arch on two feet: the gate at the edge of the stage.",
      "A half ring (radius 0.7, width 0.34) on two upright bars with wide feet.",
      "Faces the camera, no rock; the arch glows while a foe is offstage.",
      [R((0, .0), .7, .34, 0, 180), P(rect(-.7, -.42, .34, .86)), P(rect(.7, -.42, .34, .86)), P(rect(-.7, -.86, .6, .2)), P(rect(.7, -.86, .6, .2))]),
    D("comet", "Comet", "sky", 0, "Every 9 seconds: a shockwave around you, 7%. It pops foes up.",
      "A ball thrown up off a wide arc: the wave that pops foes up.",
      "An icosahedron (radius 0.4) above a 140-degree ring arc (radius 0.86) that opens upward.",
      "The ball bobs 0.15 above the arc on the 9-second clock and jumps on the fire.",
      [B((0, .42), .4, "h"), R((0, .42), .9, .27, 200, 340)]),
    D("updraft", "Updraft", "sky", 0, "Every 35% of aerial damage: an extra air jump.",
      "Two wide carets, one over the other: a second lift.",
      "Two wide V-shaped prisms pointing up (1.8 wide, arms 0.3 thick), stacked 0.75 apart.",
      "The lower caret rises into the upper one and resets (a 1 s loop); both flash when the jump is granted.",
      sum([[bar((-.9, -.28+o), (.06, .38+o), .3), bar((.9, -.28+o), (-.06, .38+o), .3)] for o in (.42, -.4)], [])),
    D("skyhook", "Skyhook", "sky", 1, "Aerial damage: fills Heat x1.5.",
      "A hook with a small wing on its shank: it holds you in the air.",
      "An upright bar, a half ring at its foot turning up into a barb, and one kite prism as a wing on the shank.",
      "Swings 10 degrees from its top like a hanging hook.",
      [P(rect(.36, .28, .28, 1.3)), R((-.08, -.34), .44, .28, 180, 360), P([(-.74, -.3), (-.3, -.3), (-.52, .3)], bev=.5), P([(.4, .62), (1.04, .9), (.84, .42), (.42, .3)], "h")]),
    D("meteor", "Meteor", "sky", 2, "A foe KO'd off the bottom: Heat x1.5, before it pays.",
      "A dart diving point down with three tails: the KO off the bottom.",
      "A triangle prism pointing down (1.0 wide) with two kite prisms and one bar streaming up behind it.",
      "Points down always; shakes 0.04 while a foe is below the stage.",
      [P([(-.52, -.06), (.52, -.06), (0, -1.0)], "h", bev=.55), P(kite((-.3, -.1), 112, 1.12, .2, .3)), P(kite((.3, -.1), 68, 1.12, .2, .3)), P(rect(0, .4, .22, .9))]),
    D("thunderhead", "Thunderhead", "sky", 1, "Every 20% dealt offstage: a thunderbolt, 7%.",
      "A flat-bottomed cloud with a small bolt under it.",
      "Three 8-sided discs on a flat bar make the cloud (1.8 wide); two small triangle prisms make the bolt (hot row).",
      "Drifts 0.08 left and right; the bolt shows only when the charge is over half.",
      [P(ngon(-.52, .3, .36, 8, 22.5)), P(ngon(0, .52, .46, 8, 22.5)), P(ngon(.52, .3, .36, 8, 22.5)), P(rect(0, .16, 1.5, .42)),
       P([(.16, -.2), (-.3, -.66), (.1, -.66)], "h", bev=.6), P([(-.14, -1.02), (.3, -.5), (-.08, -.5)], "h", bev=.6)]),
    # ------------------------------------------------------------------ Tempo (pool GREEN)
    D("rush", "Rush", "tempo", 0, "Every 40% dealt: Haste for 5 seconds.",
      "A wheel with three speed lines behind it.",
      "A ring (radius 0.5, width 0.28) at the right and three flat bars trailing left, the middle one longest.",
      "The ring turns in its plane; the bars stretch while Haste is on.",
      [R((.4, 0), .5, .28), P(rect(-.62, .42, .7, .22)), P(rect(-.72, 0, .62, .22)), P(rect(-.62, -.42, .7, .22))]),
    D("tailwind", "Tailwind", "tempo", 1, "In Haste, Heat fills x1.75.",
      "A hoop with an arrow flying through it: the push from behind.",
      "A ring (radius 0.62) with a bar through its centre ending in a triangle arrow head at the right.",
      "The arrow slides 0.2 through the hoop and back (0.8 s); faster in Haste.",
      [R((-.12, 0), .62, .24), P(rect(-.3, 0, 1.4, .25)), P([(.42, .44), (1.02, 0), (.42, -.44)], "h", bev=.6)]),
    D("clean_landing", "Clean Landing", "tempo", 0, "An L-cancelled aerial that dealt damage: +10 Heat.",
      "A ring with a tick in it: the landing was clean.",
      "A thin ring (radius 0.86, width 0.2) with a two-bar tick inside it in the hot row, the long arm reaching the rim.",
      "Faces the camera; the tick pops (scale 1.3, 6 frames) on each L-cancel that counts.",
      [R((0, 0), .86, .2), bar((-.46, .06), (-.14, -.3), .27, role="h"), bar((-.2, -.32), (.56, .5), .27, role="h")]),
    D("perfect_form", "Perfect Form", "tempo", 1, "While your last three aerial landings were L-cancelled: Heat fills x1.75.",
      "A bullseye of three bands: three clean landings in a row. The family's base form.",
      "A ring (radius 0.84), a 10-sided disc inside it (radius 0.42) and a small ball at the centre.",
      "A band lights for each L-cancel in the streak; all three pulse while the bonus is on.",
      [R((0, 0), .84, .25), P(ngon(0, 0, .43, 10, 0)), B((0, 0), .17, "i")]),
    D("slipstream", "Slipstream", "tempo", 0, "Tag in: Haste for 6 seconds.",
      "Two arrows chasing each other round a circle: the swap.",
      "Two ring arcs of 140 degrees (radius 0.7), each ending in a triangle arrow head.",
      "Turns half a turn on each tag and stops.",
      _arrowarc(20, 150) + _arrowarc(200, 330)),
    D("slick", "Slick", "tempo", 0, "After a wavedash: drives charge x1.5 for 1 second.",
      "A ring lying flat with skid marks under it: the slide along the floor.",
      "A ring (radius 0.86) tilted back 62 degrees so it reads as a flat oval, and two bars under it.",
      "Slides 0.2 sideways and back with each wavedash; the bars fade over the one-second window.",
      [R((0, .22), .86, .3, tilt=60), P(rect(-.2, -.56, 1.3, .22)), P(rect(.2, -.9, 1.0, .2))]),
    D("jet_trail", "Jet Trail", "tempo", 1, "Every 30% dealt in Haste: a wind ring, 5%. Haste renews.",
      "A small ring with two wakes spreading behind it.",
      "A ring (radius 0.34) at the right and two ring arcs (radius 0.8 and 1.3, 80 and 56 degrees) behind it.",
      "The wakes travel outward and fade (a 0.6 s loop) while Haste is on.",
      [R((.6, 0), .34, .22), R((.62, 0), .86, .24, 138, 222), R((.62, 0), 1.36, .24, 152, 208)]),
    D("quicksilver", "Quicksilver", "tempo", 2, "In Haste: all your moves are 15% faster.",
      "A stopwatch: the clock itself runs fast.",
      "A ring (radius 0.7), a small block on top as the crown, one raised bar as the hand, a ball at the pivot.",
      "The hand sweeps a turn every 2 s, every 1 s in Haste.",
      [R((0, -.16), .72, .25), P(rect(0, .8, .36, .3)), P(rect(.5, .56, .2, .26, -45)), bar((0, -.16), (.3, .26), .2), B((0, -.16), .17, "h")]),
    # ------------------------------------------------------------------ Guard (pool BLUE)
    D("riposte", "Riposte", "guard", 0, "A tech: +15 Heat.",
      "A plate with a blade crossed behind it: the answer to a hit.",
      "The family plate (flat top, point down) at 0.8 size in front of a long diagonal bar with a triangle tip.",
      "Rocks 12 degrees; the blade jabs 0.2 along its length on a tech.",
      [bar((-.86, -.8), (.7, .76), .24, z=(-.34, -.1)), P([(.56, .86), (1.0, 1.0), (.84, .56)], "h", z=(-.34, -.1), bev=.5), P(mov(scl(SH, .78), 0, -.06))]),
    D("insulate", "Insulate", "guard", 1, "First 20% taken per fight, and after each cash-out: no Heat lost.",
      "A plate inside a second shell: two walls.",
      "The family plate as a hollow frame (seven quads) with a smaller solid plate inside it (hot row).",
      "The outer shell lifts 0.1 off the inner while the allowance is unspent and drops when it is used up.",
      frame(SH, .74, (0, .04)) + [P(scl(SH, .5, (0, .04)), "h")]),
    D("bulwark", "Bulwark", "guard", 0, "Every 15 seconds: an orb absorbs the next hit.",
      "A dome on a slab: the cover that takes one hit.",
      "A half disc (radius 0.86, flat side down) on a flat slab 1.9 wide.",
      "The dome swells 8% on the 15-second clock; it flattens for 10 frames when the orb is spent.",
      [P(arcpts((0, -.36), .86, 0, 180, 9), bev=.8), P(rect(0, -.56, 1.92, .34))]),
    D("thorns", "Thorns", "guard", 0, "Every 25% taken: the attacker takes 6% and is Marked.",
      "A plate with barbs on both sides: it hurts to hit.",
      "The family plate at 0.7 size with two triangle barbs on each side (hot row).",
      "Rocks 12 degrees; the barbs stab out 0.15 when the wearer is hit.",
      [P(mov(scl(SH, .7), 0, .02))] + [dict(p, role="h") for p in _thorn(1) + _thorn(-1)]),
    D("leech", "Leech", "guard", 0, "Every 60% dealt: heal 6%.",
      "A heavy plus: it gives health back.",
      "Two thick bars (0.56 by 1.9) crossed as a plus, with a small square boss raised at the crossing.",
      "Faces the camera; the plus pulses once on each heal.",
      [P(rect(0, 0, .56, 1.9), bev=.8), P(rect(0, 0, 1.9, .56), bev=.8), P(rect(0, 0, .3, .3), "i")]),
    D("breaker", "Breaker", "guard", 1, "A shield break: +45 Heat. Shield damage counts at half.",
      "A wedge driven into a slab that has split: it breaks cover.",
      "A triangle prism pointing down (1.0 wide, flat top) over two slab halves tilted into a shallow V.",
      "The wedge taps down 0.1 each time shield damage counts; the halves fly apart on a break.",
      [P([(-.52, .95), (.52, .95), (0, -.3)], "h", bev=.6), P([(-1, -.3), (-.1, -.6), (-.1, -.96), (-1, -.66)]), P([(1, -.3), (.1, -.6), (.1, -.96), (1, -.66)])]),
    D("last_stand", "Last Stand", "guard", 1, "At 100% or more yourself: your drive fires deal x1.25.",
      "A tower with a notched top: the place you hold.",
      "An upright block (1.1 by 1.3) on a wider foot, three small blocks along its top, a slot raised on its face.",
      "Still below 100%; from 100% it shakes 0.03 and the slot glows.",
      [P(rect(0, -.14, 1.1, 1.22)), P(rect(0, -.84, 1.5, .26)), P(rect(-.41, .74, .28, .42)), P(rect(0, .74, .28, .42)), P(rect(.41, .74, .28, .42)), P(rect(0, -.2, .2, .56), "i")]),
    D("colossus", "Colossus", "guard", 2, "While Hot or hotter: you are giant and take 20% less.",
      "Three slabs, each wider than the one below: it grows.",
      "Three stacked slabs 0.7, 1.24 and 1.9 wide, 0.12 apart, the top one deepest.",
      "The stack swells 15% while the wearer is Hot or hotter.",
      [P(rect(0, -.74, .7, .42)), P(rect(0, -.16, 1.24, .5)), P(rect(0, .54, 1.9, .66), "h")]),
    # ------------------------------------------------------------------ Utility (pool WHITE)
    D("echo", "Echo", "util", 1, "The drive to its right fires again at half strength.",
      "Three blocks, each smaller, stepping to the right: the repeat, and which way it looks.",
      "Three square prisms 0.86, 0.5 and 0.28 across in a row, depth equal to width.",
      "The blocks pulse left to right (a 3-step loop) when the neighbour fires.",
      [P(rect(-.52, 0, .86, .86), z=(-.4, .4), bev=.8), P(rect(.29, 0, .5, .5), z=(-.25, .25), bev=.8), P(rect(.82, 0, .28, .28), "h", z=(-.14, .14), bev=.8)]),
    D("metronome", "Metronome", "util", 0, "A move not among your last three deals damage: +6 Heat.",
      "A metronome with its arm swung over: change the beat.",
      "A tapered block (1.1 wide at the foot, 0.44 at the top) and a thin bar with a small weight leaning out of it.",
      "The arm swings side to side once a second; it stops upright when the next move would not count.",
      [P([(-.56, -.95), (.56, -.95), (.22, .56), (-.22, .56)]), bar((0, -.5), (.56, .96), .2, role="h", z=(.0, .3)), P(rect(.38, .5, .36, .24, 69), "h", z=(.0, .34)),
       bar((0, -.5), (.2, 0), .15, role="i", m=0)]),
    D("relay", "Relay", "util", 1, "Tag in: all of this fighter's drives fire at once.",
      "Two linked rings: the hand-off. The family's base form.",
      "Two rings (radius 0.5, width 0.24) set diagonally and interlocked, one 0.12 in front of the other.",
      "The links swing against each other 10 degrees; they pull tight on a tag.",
      [R((-.32, .26), .52, .25, z=-.08), R((.32, -.26), .52, .25, role="h", z=.08)]),
    D("surge", "Surge", "util", 1, "Stage shift: every drive is fully charged.",
      "Two slabs sliding past each other: the stage shifts.",
      "Two parallelogram prisms (1.5 by 0.62, sheared 0.3), the upper pushed right and the lower pushed left, 0.2 apart.",
      "The slabs slide 0.3 past each other and back on a stage shift.",
      [P([(-.62, .16), (.7, .16), (1.0, .8), (-.32, .8)]), P([(-1.0, -.8), (.32, -.8), (.62, -.16), (-.7, -.16)], "h")]),
    D("crescendo", "Crescendo", "util", 2, "Deal 50% and take none: Heat x1.25.",
      "Four pillars rising left to right: it builds while you are not hit.",
      "Four upright bars 0.34 wide, 0.5, 0.95, 1.4 and 1.9 tall, on one base line, 0.14 apart; the tallest uses the hot row.",
      "The pillars light left to right with the counter and all drop when the wearer is hit.",
      [P(rect(x, -.95 + h/2, .34, h), "h" if i == 3 else "b") for i, (x, h) in enumerate(((-.72, .5), (-.24, .95), (.24, 1.4), (.72, 1.9)))]),
    D("snowball", "Snowball", "util", 1, "A cash-out while Blazing: +3 starting Heat, this run.",
      "A small ball and a big one: it grows as it rolls.",
      "Two icosahedra, radius 0.27 and 0.66, on one floor line.",
      "The big ball grows with its pips (radius 0.5 at none to 0.72 at twenty) and turns slowly.",
      [B((-.68, -.6), .27), B((.26, -.2), .68, "h")]),
    D("magpie", "Magpie", "util", 0, "A cash-out: half again in coins. Overflow Heat pays too.",
      "A tied pouch with a coin mark: more coins.",
      "A squat eight-sided pouch body (1.7 wide), a wide flared neck above the tie (1.4 wide), a raised band at the tie and a raised disc on the body.",
      "The pouch hops 0.1 on each cash-out.",
      [P([(-.28, .22), (.28, .22), (.86, -.3), (.84, -.7), (.5, -.95), (-.5, -.95), (-.84, -.7), (-.86, -.3)]), P([(-.7, .86), (.7, .86), (.22, .22), (-.22, .22)]),
       P(rect(0, .24, .7, .15), "i"), P(ngon(0, -.4, .26, 10), "i")]),
    D("mirror", "Mirror", "util", 2, "Always: copies the drive to its left.",
      "A diamond in two halves, the left solid and the right an empty outline: the copy of what is beside it.",
      "A solid triangle prism at the left and the same triangle as a hollow frame at the right, 0.4 apart.",
      "In orbit it takes the left drive's mesh in the hot row; this form is for the floor, menus and an empty copy.",
      [P([(-.2, 1.0), (-.2, -1.0), (-.98, 0)], bev=.74)] + frame([(.2, 1.0), (.98, 0), (.2, -1.0)], .42, (.46, 0), role="h")),
]
DK = {d["key"]: d for d in DRIVES}
TIERS = ["Common", "Rare", "Epic"]
BASE = {"fire": "ember", "frost": "frost", "sky": "feather_storm", "tempo": "perfect_form", "guard": None, "util": "relay"}
GUARD_BASE = [P(SH), P(scl(SH, .42, (0, .05)), "i")]

LANG = {
    "fire": ("Tall, round at the foot, one sharp point up.", "Taller than wide and uneven top to bottom. Nothing else in the set tapers upward."),
    "frost": ("Six-sided crystals and their clusters.", "Straight edges at 60 degrees and points all round. The only family built on the hexagon."),
    "sky": ("Wide, swept and open: fans, arcs, carets, wings.", "Wider than tall, with daylight through or under it. The only family that spreads sideways."),
    "tempo": ("A ring or a dial, with one thing that moves.", "A round form with a hole reads at any size, and no other family is round."),
    "guard": ("Heavy plates and blocks with a flat top.", "The most filled silhouette in the set: square shoulders, no hole, no point up."),
    "util": ("Plain solids in sets of two or more.", "Counted pieces with ink between them. No other family is made of separate equal parts."),
}

# ---------------------------------------------------------------------------------------------- the 16 Uniques
def UQ(key, name, tag, c1, c2, fx, mat, says, form, parts):
    return dict(key=key, name=name, rule=tag, c1=c1, c2=c2, fx=fx, mat=mat, says=says, form=form, parts=parts)

_NUG = [(-.7, .2), (-.3, .78), (.42, .7), (.86, .16), (.62, -.62), (-.1, -.86), (-.74, -.5)]
UNIQUES = [
    UQ("puppeteer", "Puppeteer", "Every 25% dealt: fires the drive to its left.", "#7A5433", "#EDE3CB", "pulse",
       "Threads that glow in a pulse travelling to the left.", "I pull that one.",
       "A tilted cross of two bars with three thin threads of different lengths hanging from it, a small ball on each.",
       [bar((-.95, .5), (.95, .8), .26), bar((-.3, .98), (.2, .34), .24)] + sum([[bar((x, y0), (x, y1), .13, role="h"), B((x, y1-.1), .17, "h")] for x, y0, y1 in ((-.72, .5, -.5), (0, .62, -.84), (.72, .72, -.18))], [])),
    UQ("ashbank", "Ashbank", "Heat lost to hits: banked, and returned on your next KO.", "#4A4854", "#E0492C", "glow",
       "An inner glow that rises with the banked Heat; ash drifts in.", "How much is saved.",
       "A lidded urn with two ear handles: a wide-shouldered body on a small foot, a flat lid and a knob; a slot raised on the belly.",
       [P([(-.5, .36), (.5, .36), (.8, .0), (.66, -.56), (.34, -.8), (-.34, -.8), (-.66, -.56), (-.8, .0)]), P(rect(0, -.88, .8, .2)), P(rect(0, .5, 1.24, .22)), B((0, .8), .2),
        R((-.8, .02), .22, .13, 70, 290, role="h"), R((.8, .02), .22, .13, -110, 110, role="h"),
        P(rect(0, -.2, .7, .2), "i")]),
    UQ("changing_of_the_guard", "Changing of the Guard", "Tag in: the new fighter lands as a blast, 10%.", "#B99A46", "#7A2329", "sheen",
       "A brass sheen that flashes edge to edge when the tag is ready.", "Tag now.",
       "Two long poles crossed, each with an axe blade and a spike at its head.",
       sum([[bar((s*.72, -.98), (-s*.5, .78), .2), P([(-s*.5, .78), (-s*.62, 1.06), (-s*.36, .96)], "h", bev=.5),
             P([(-s*.4, .7), (-s*1.0, .78), (-s*.92, .22), (-s*.26, .42)], "h", bev=.7)] for s in (1, -1)], [])),
    UQ("turnabout", "Turnabout", "A cash-out: your row reverses. The new first drive fills.", "#8D96A2", "#E8D9A9", "motes",
       "Sand that flows upward after each flip.", "Which way the row reads.",
       "An hourglass: two triangles tip to tip between two end plates, with a pivot ball on each side.",
       [P([(-.6, .74), (.6, .74), (0, -.06)], "h", bev=.7), P([(-.6, -.74), (.6, -.74), (0, .06)], "h", bev=.7), P(rect(0, .86, 1.5, .22)), P(rect(0, -.86, 1.5, .22)),
        B((-.86, 0), .17), B((.86, 0), .17)]),
    UQ("mockingbird", "Mockingbird", "Always: becomes the last drive a foe fired at you.", "#8B8B95", "#D9D9E2", "dissolve",
       "A dissolve edge that eats the grey and shows the copied form.", "Empty or loaded, and with what.",
       "A hollow bird: an eight-sided body with an open centre, a round head, a beak and a long tail.",
       frame(ngon(-.05, -.12, .62, 8, 22.5), .5, (-.05, -.12)) + [B((.56, .5), .3), P([(.8, .62), (1.06, .44), (.78, .34)], "h", bev=.5),
                                                                  P([(-.5, .1), (-1.04, .62), (-.94, -.02), (-.56, -.3)], "h")]),
    UQ("hundred_year_seed", "Hundred-Year Seed", "From loop 3: fires +10%, doubling every loop.", "#6C4D2C", "#86A552", "rings",
       "Growth rings lit from inside, one more band each loop.", "How many loops it has grown.",
       "A pointed seed lying at an angle with a short stem and two leaves; from loop 3 the stem lengthens a step each loop.",
       [P(mov(rot([(0, .8), (.46, .36), (.5, -.2), (0, -.8), (-.5, -.2), (-.46, .36)], -28), -.1, -.28)),
        bar((.22, .3), (.3, .74), .16, role="h"), P(kite((.3, .7), 28, .7, .2), "h"), P(kite((.3, .7), 140, .62, .2), "h")]),
    UQ("lifeline_coin", "Lifeline Coin", "Your last stock lost: pay 150 coins and keep it.", "#B38E3B", "#2B2630", "foil",
       "Iridescent foil that goes dull when the purse is under the price.", "You can afford one.",
       "A thick coin with a round hole, hung on a cord that rises to a knot.",
       [R((0, -.3), .5, .42), bar((-.12, .1), (0, .94), .13, role="h"), bar((.12, .1), (0, .94), .13, role="h"), B((0, .94), .15, "h")]),
    UQ("kiln", "Kiln", "A KO: keeps all your Heat. Heat drains 3 a second.", "#34313C", "#FFF0CC", "haze",
       "A heat-haze halo that bends the air around it, always sinking.", "Heat is draining.",
       "A brick furnace: a block on a slab with a chimney at one side and an arched mouth raised on its face.",
       [P(rect(-.08, -.2, 1.5, 1.2)), P(rect(0, -.88, 1.9, .22)), P(rect(.44, .64, .4, .62)), P(rect(.44, .98, .56, .14)),
        P(arcpts((-.12, -.34), .4, 0, 180, 6) + [(-.52, -.74), (.28, -.74)], "i")]),
    UQ("absolution", "Absolution", "Always: your first keystone has no price.", "#707D8D", "#F3F0F8", "sheen",
       "A slow pearl sweep across the break while the price is off.", "The price is lifted now.",
       "A shackle with a gap in its ring and a short broken link thrown off above it.",
       [R((-.1, -.26), .62, .3, 128, 412), R((.6, .7), .26, .17, -50, 190, role="h"), P(rect(-.1, -.88, .5, .2))]),
    UQ("clockwork_heel", "Clockwork Heel", "An L-cancel: fills the drive to its right.", "#566170", "#DC9C8B", "pattern",
       "An animated gear pattern that ticks, and stops during the 3-second gate.", "Ready or winding.",
       "A spur: a half band open to the right, a short neck, a toothed wheel at the left, and a two-lobed key on top.",
       [R((.4, -.2), .5, .26, 90, 270), bar((-.1, -.2), (-.46, -.2), .22), P(ngon(-.66, -.2, .26, 8), "h")]
       + [P([(-.66 + .2*math.cos(math.radians(a+24)), -.2 + .2*math.sin(math.radians(a+24))), (-.66 + .5*math.cos(math.radians(a)), -.2 + .5*math.sin(math.radians(a))),
             (-.66 + .2*math.cos(math.radians(a-24)), -.2 + .2*math.sin(math.radians(a-24)))], "h", bev=.5) for a in (30, 90, 150, 210, 270, 330)]
       + [bar((.4, .3), (.4, .62), .16, role="h"), B((.22, .8), .2, "h"), B((.58, .8), .2, "h")]),
    UQ("hair_trigger", "Hair Trigger", "Your active: costs 40 Heat, not pips.", "#405A7B", "#F4F6FF", "sparks",
       "Sparks along the edge, denser when Heat can pay for more uses.", "Uses in hand.",
       "A cocked hammer: a pivot disc, a leaning neck, a jaw block holding a wedge of flint, a thumb spur behind.",
       [P(ngon(-.36, -.62, .34, 8)), bar((-.36, -.62), (.1, .4), .3), P(rect(.26, .56, .72, .3, -18)), P([(.5, .74), (1.04, .42), (.56, .32)], "h", bev=.5), bar((.0, .42), (-.52, .9), .2)]),
    UQ("shared_flame", "Shared Flame", "With a partner: one Heat meter, filled by both.", "#AEB8C2", "#F2F6FA", "flow",
       "A liquid-metal flow from one ring to the other, toward whoever gained last.", "Who is feeding the meter.",
       "Two rings one above the other, joined at the touch into a figure of eight, with a drop between them.",
       [R((0, .5), .44, .22), R((0, -.5), .44, .22), P(flame(0, 0, .15, .26, 0), "h", z=(.1, .34))]),
    UQ("pied_piper", "Pied Piper", "A popped enemy: fights for you for 10 seconds.", "#3B3780", "#EDE3CB", "trail",
       "A soft note-shaped trail behind each ally.", "Which enemies are yours.",
       "A long reed pipe set diagonally with a flared end, three holes raised on it and a pennant hanging from it.",
       [bar((-.92, -.62), (.7, .62), .26), P([(.62, .78), (1.02, .9), (.9, .44), (.78, .5)]), P([(-.3, -.2), (-.3, -.98), (.2, -.62)], "h", bev=.6)]
       + [B((-.5 + i*.36, -.3 + i*.275), .085, "i") for i in range(3)]),
    UQ("fools_gold", "Fool's Gold", "Every 10 coins held: your fires deal 1% more.", "#A39F58", "#E6E2A4", "glitter",
       "A glitter that scales with the purse and goes flat after a shop.", "How rich, so how strong.",
       "A rough lopsided nugget of seven sides with a facet line, and one small chunk beside it.",
       [P(mov(_NUG, -.1, .06), bev=.6), P([(.6, -.62), (.98, -.5), (.9, -.9), (.62, -.96)], "h", bev=.6), bar((-.5, .5), (.1, .0), .1, role="i"), bar((.1, .0), (.7, .2), .1, role="i"),
        bar((.1, .0), (-.1, -.7), .1, role="i")]),
    UQ("albatross", "Albatross", "Offstage: damage refills your jumps. No Heat is lost.", "#7C8895", "#F2F2EC", "streak",
       "A wind-streak trail that appears only offstage.", "You are in its element.",
       "One long wing: a bent spar from lower left to upper right with four feathers hanging off its trailing edge.",
       [bar((-1.0, -.5), (-.1, .34), .26), bar((-.14, .3), (1.02, .7), .24)]
       + [P(kite((x, y), a, L, .17, .35), "h") for x, y, a, L in ((-.56, -.12, -62, .7), (-.1, .26, -70, .86), (.36, .44, -74, .8), (.8, .58, -78, .62))]),
    UQ("ouroboros", "Ouroboros", "A chain's last link: also charges your first drive.", "#6FA596", "#1D2A28", "scales",
       "A scale pattern that runs once round the ring for each returned charge.", "The loop closed.",
       "A ring that thins to a tail, with a wedge head closing on the tail and an eye raised on the head.",
       [R((0, -.06), .68, .3, 80, 330), R((0, -.06), .68, .2, 330, 392), R((0, -.06), .68, .12, 32, 60),
        P(mov([(-.34, .26), (.3, .32), (.42, .0), (-.3, -.22)], .3, .6), "h"), B((.38, .66), .09, "i")]),
]
UK = {u["key"]: u for u in UNIQUES}
for u in UNIQUES:
    c1, c2 = hexrgb(u["c1"]), hexrgb(u["c2"])
    row("u_" + u["key"], mix(c1, (14, 9, 40), .72), mix(c1, (14, 9, 40), .12), c1)
    row("u_" + u["key"] + "_hot", mix(c2, (14, 9, 40), .3), c2, c2)


def umesh(u): return build(u["parts"], "u_" + u["key"])
def dmesh(d): return build(d["parts"], d["fam"])


# ---------------------------------------------------------------------------------------------- model treatments
def tier_mesh(mesh, tier):
    """Tier is an added ring, never a new body (as the generator's rarity overlays). The ring lies flat under the
    form's foot, so the outline above it stays clean: white for Rare, foil with a spark for Epic."""
    m = Mesh().extend(mesh)
    if tier == 1:
        m.ring_y(.84, .085, 14, "white", .95, y=-1.14)
    if tier == 2:
        m.ring_y(.84, .1, 14, ["foil%d" % i for i in range(6)], .95, y=-1.14)
        s = Mesh(); s.prism([(0, .3), (.08, .08), (.3, 0), (.08, -.08), (0, -.3), (-.08, -.08), (-.3, 0), (-.08, .08)], -.06, .06, .8, ("sun", "sun"), (.6, 1, 1))
        m.extend(s.scaled(1, (.98, -.92, .3)))
    return m

def asc_mesh(d, tier=0):
    """Ascended: seams turn gold, and a gold star in a small gold ring floats over the form's head."""
    m = tier_mesh(build(d["parts"], d["fam"], hot="sun"), tier)
    m.ring_y(.52, .075, 12, "sun", 1.0, y=1.34)
    st = [(0, 1), (.3, .32), (1.02, .3), (.46, -.16), (.64, -.88), (0, -.48), (-.64, -.88), (-.46, -.16), (-1.02, .3), (-.3, .32)]
    s = Mesh()
    for i in range(10):
        for zz in (.1, -.1):
            s.tri(((0, 0, zz), (st[i][0]*.38, st[i][1]*.38, 0), (st[(i+1) % 10][0]*.38, st[(i+1) % 10][1]*.38, 0)), 1.0, "sun", (0, 0, 0))
    return m.extend(s.scaled(1, (0, 1.46, 0)))
