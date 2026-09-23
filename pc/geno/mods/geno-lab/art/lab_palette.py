"""The Geno Lab's accent palette, in the kit's colour-module style (menu/pipeline/kit.py).

    python lab_palette.py         -> prints the checks; lab_art.py imports it

The Lab borrows the kit wholesale: ink structure, bone text, gold = the thing you are on
(here: the timeline playhead), muted / disabled for secondary and off. It adds only what an
in-match overlay needs and the menus never did:

  * a translucent GLASS body, so panels read over gameplay without hiding it;
  * one Lab ACCENT (cyan), which none of the five section faces, the four ports or the
    hitbox-id colours use, so "this is Lab chrome" never reads as "this is a player";
  * hitbox-id colours for timeline windows and hitbox labels (ids 0-3 + thrown);
  * event-marker colours for the timeline, each paired with its own marker SHAPE
    (lab_mk_*), so colour is never the only cue - the kit's rule for ports and sections.

Every colour is checked here with the root pipeline's colour maths (colour.py: WCAG
contrast, Machado 2009 CVD simulation, CIEDE2000). Nothing is modified in the root tree.
"""
import itertools
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def _find_up(rel):
    d = HERE
    while True:
        if os.path.exists(os.path.join(d, rel)):
            return d
        up = os.path.dirname(d)
        if up == d:
            return None
        d = up


# The root pipeline is shared code, imported by path and never edited from here.
MENU = os.environ.get("GD_MENU_ROOT") or os.path.join(_find_up("menu/pipeline/kit.py") or "", "menu")
sys.path.insert(0, os.path.join(MENU, "pipeline"))
import colour as K                                        # noqa: E402
import kit                                                # noqa: E402

KIT = kit.PALETTE

# ----------------------------------------------------------------- palette
# glass is the one translucent colour. Its RGB is exact in RGB5A3's translucent (RGB444)
# path - every channel a multiple of 17 - and its alpha is exactly 3-bit alpha level 6
# (219), so the 9-slice pieces carry it with zero quantisation error.
GLASS_ALPHA = 219
LAB = dict(
    glass="#111122",        # panel body, drawn at GLASS_ALPHA (86%) over gameplay
    track="#232b40",        # timeline track, opaque, inside a glass panel
    tick="#7d88a6",         # timeline ruler ticks (= kit disabled)
    accent="#38c9d9",       # Lab chrome: title slab, panel corner, toggle ON tint
    accent_dk="#16707c",    # accent pressed / accent-on-bone
    off=KIT["disabled"],    # toggle OFF tint (+ the lab_slash overlay: never colour alone)
    running=KIT["ok"],      # status: running
    paused=KIT["gold"],     # status: paused
    warn=KIT["danger"],     # refused writes, netplay read-only, step-back failure
    playhead=KIT["gold"],   # the frame shown: gold is "what you are on", everywhere
)

# Timeline windows and hitbox labels, by hitbox id (0-3 = fp->x914[], 4 = thrown).
# No yellow: yellow is the gold playhead's neighbour.
HIT = {
    "hit0": "#e5483b",      # = P1 / kit danger red
    "hit1": "#f5902e",
    "hit2": "#e24fb7",
    "hit3": "#8e72ff",
    "thrown": KIT["muted"],
}

# Event markers above / below the track. Each has a shape (lab_mk_<name>).
MARK = {
    "iasa": "#27b88a",      # flag     (= kit ok) - interruptible from here
    "invinc": KIT["bone"],  # shield   body / hurtbox state changes
    "gfx": "#4d8dff",       # spark
    "sfx": "#e8a6ff",       # speaker (light: stays apart from gfx blue under protanopia)
    "vis": "#f7df5e",       # eye      model visibility
}
LANES = {
    "above": ["iasa", "invinc"],
    "below": ["gfx", "sfx", "vis"],
}

MIN_TEXT = 4.5          # WCAG AA (kit.MIN_TEXT)
MIN_GRAPHIC = 3.0       # WCAG non-text (kit.MIN_GRAPHIC)
MIN_HIT_DE = 15         # hitbox windows sit next to each other on one bar
MIN_MARK_DE = 12        # markers also differ by shape
CVD = ["normal", "deuteranopia", "protanopia", "tritanopia"]


def rgb(h):
    return K.hexrgb(h)


def over(fg_hex, alpha, bg):
    """fg at alpha over an opaque bg (sRGB-space blend, what GX's blend unit does)."""
    f = rgb(fg_hex)
    a = alpha / 255
    return tuple(round(f[i] * a + bg[i] * (1 - a)) for i in range(3))


# Worst cases for a translucent panel: gameplay that is pure white, pure black, and a mid
# grey. Text contrast is taken against the worst of the three.
BACKDROPS = {"white": (255, 255, 255), "grey": (128, 128, 128), "black": (0, 0, 0)}


def glass_on(bg):
    return over(LAB["glass"], GLASS_ALPHA, bg)


def check():
    errs, rep = [], {}

    # text and icons on glass, over each backdrop
    rep["on_glass"] = {}
    for name, h in [("bone", KIT["bone"]), ("muted", KIT["muted"]), ("accent", LAB["accent"]),
                    ("gold", KIT["gold"]), ("ok", KIT["ok"]), ("danger", KIT["danger"]),
                    ("disabled", KIT["disabled"])]:
        worst = min(K.contrast(rgb(h), glass_on(bg)) for bg in BACKDROPS.values())
        rep["on_glass"][name] = round(worst, 2)
        need = MIN_GRAPHIC if name in ("disabled", "danger") else MIN_TEXT
        if worst < need:
            errs.append("%s on glass only %.2f:1 (need %.1f)" % (name, worst, need))

    # ink text on the accent slab and on the bone keycap
    rep["ink_on_accent"] = round(K.contrast(rgb(KIT["ink"]), rgb(LAB["accent"])), 2)
    rep["ink_on_bone"] = round(K.contrast(rgb(KIT["ink"]), rgb(KIT["bone"])), 2)
    if rep["ink_on_accent"] < MIN_TEXT:
        errs.append("ink on accent %.2f:1" % rep["ink_on_accent"])

    # every timeline colour must stand off the track
    rep["on_track"] = {}
    for n, h in list(HIT.items()) + list(MARK.items()) + [("playhead", LAB["playhead"]),
                                                          ("tick", LAB["tick"])]:
        c = K.contrast(rgb(h), rgb(LAB["track"]))
        rep["on_track"][n] = round(c, 2)
        if c < MIN_GRAPHIC:
            errs.append("%s on track only %.2f:1" % (n, c))

    def closest(d, kind):
        rows = sorted((round(K.de2000(K.simulate(rgb(d[a]), kind), K.simulate(rgb(d[b]), kind)), 1),
                       a, b) for a, b in itertools.combinations(d, 2))
        return rows[0]

    rep["hit_closest"] = {k: closest(HIT, k) for k in CVD}
    if rep["hit_closest"]["normal"][0] < MIN_HIT_DE:
        errs.append("hit colours %s/%s only dE %.1f" % rep["hit_closest"]["normal"][1:] +
                    (rep["hit_closest"]["normal"][0],))
    marks = dict(MARK, playhead=LAB["playhead"])
    rep["mark_closest"] = {k: closest(marks, k) for k in CVD}
    if rep["mark_closest"]["normal"][0] < MIN_MARK_DE:
        errs.append("marker colours %s/%s only dE %.1f" % (rep["mark_closest"]["normal"][1],
                    rep["mark_closest"]["normal"][2], rep["mark_closest"]["normal"][0]))

    # the accent must not read as a port or a section face
    others = dict(kit.PORTS)
    others.update({"face_" + s: v["face"] for s, v in kit.SECTIONS.items()})
    others.update(HIT)
    rep["accent_vs"] = sorted((round(K.de2000(rgb(LAB["accent"]), rgb(v)), 1), n)
                              for n, v in others.items())[:3]
    if rep["accent_vs"][0][0] < 15:
        errs.append("accent too close to %s (dE %.1f)" % (rep["accent_vs"][0][1],
                                                          rep["accent_vs"][0][0]))
    return errs, rep


def u32(h, a=255):
    """0xRRGGBBAA, the form lab.lua and gd.fill / gd.text take."""
    r, g, b = rgb(h)
    return "0x%02X%02X%02X%02X" % (r, g, b, a)


def palette_dict(rep):
    return dict(
        units="sRGB hex; u32 = 0xRRGGBBAA as gd.fill/gd.text take it",
        kit_tokens_used=["ink", "bone", "muted", "disabled", "gold", "ok", "danger"],
        lab={k: dict(hex=v, u32=u32(v, GLASS_ALPHA if k == "glass" else 255))
             for k, v in LAB.items()},
        glass_alpha=GLASS_ALPHA,
        hit={k: dict(hex=v, u32=u32(v)) for k, v in HIT.items()},
        marks={k: dict(hex=v, u32=u32(v), texture="lab_mk_" + k) for k, v in MARK.items()},
        lanes=LANES,
        rules=[
            "Gold is still selection: in the Lab that is the playhead (the frame shown) and "
            "the focused fighter's highlight. The accent is Lab chrome only.",
            "Toggle icons: ON = accent (or bone), OFF = off + the lab_slash overlay; colour is "
            "never the only cue.",
            "Timeline markers pair a colour with a shape (lab_mk_*); hitbox windows are "
            "flat quads in hit colours, one bar per id row or stacked.",
            "Panels: glass at glass_alpha, ink border, accent corner (lab_frame_*).",
        ],
        checks=rep,
    )


def main():
    errs, rep = check()
    for k, v in rep.items():
        print("%-14s %s" % (k, v))
    if errs:
        print("\nPALETTE CHECKS FAILED:")
        for e in errs:
            print("  - " + e)
        return 1
    print("\nlab palette checks ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
