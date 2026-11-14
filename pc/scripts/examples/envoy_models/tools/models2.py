"""Envoy models, chunk 2: the part lists (the other 23 items, keyword glyphs, tag emblems, HUD bits, projectile
bodies, map furniture, world pieces). `make_models1.py` imports this file and owns the helpers, the palette and every
writer; this file is only shapes. Every coordinate is original, written for this project.
"""
import math

import drive_parts as DP
from drive_parts import P, R, B, bar, flame, kite, ngon, rect, rot, mov, scl, mir, arcpts


def star5(c, r, k=.44, a0=90, **kw):
    """A five-point star: a pentagon core and five triangle points."""
    inner = ngon(c[0], c[1], r*k, 5, a0 + 36)
    tips = ngon(c[0], c[1], r, 5, a0)
    out = [P(inner, **kw)]
    for i in range(5):
        out.append(P([inner[(i-1) % 5], tips[i], inner[i]], bev=.7, **kw))
    return out


def qmark(cx, cy, s, role="b", z=0.0):
    """A question mark: a hook, a stem and a dot."""
    return [R((cx, cy + .34*s), .27*s, .15*s, -80, 190, role, z=z), bar((cx + .03*s, cy + .1*s), (cx + .03*s, cy - .16*s), .15*s, role=role, z=(z - .15, z + .15)),
            B((cx + .03*s, cy - .44*s), .1*s, role, z=z)]


def cross_x(cx, cy, r, w, **kw):
    return [bar((cx - r, cy - r), (cx + r, cy + r), w, **kw), bar((cx - r, cy + r), (cx + r, cy - r), w, **kw)]


def leaf(L, w):
    """A feather vane pointing up from the origin."""
    return [(0, L), (w, .55*L), (.9*w, -.1*L), (0, -.42*L), (-.9*w, -.1*L), (-w, .55*L)]


def feather_barbs():
    """Three pairs of barbs up a quill on the Y axis, each pair a little narrower."""
    out = []
    for y0, w in ((-.52, .44), (-.14, .42), (.24, .34)):
        q = [(0, y0), (w, y0 + .16), (w*.82, y0 + .5), (0, y0 + .4)]
        out += [q, mir(q)]
    return out


def bricks(skip=()):
    rows = [(.56, ((-.5, .94), (.5, .94))), (0, ((-.76, .42), (0, .94), (.76, .42))), (-.56, ((-.5, .94), (.5, .94)))]
    out, i = [], 0
    for y, bs in rows:
        for x, w in bs:
            if i not in skip:
                out.append(P(rect(x, y, w, .46)))
            i += 1
    return out


def mine_body(c):
    x, y = c
    return [P(ngon(x, y, .5, 8, 22.5))] + [P(kite((x + .44*math.cos(math.radians(a)), y + .44*math.sin(math.radians(a))), a, .36, .11, .3), bev=.6) for a in range(0, 360, 45)]


def models(h):
    G, M, FRONT, BACK = h.G, h.M, h.FRONT, h.BACK
    heart, spark4, crescent, arch, horn, frame_ring = h.heart, h.spark4, h.crescent, h.arch, h.horn, h.frame_ring
    C45 = [(math.cos(math.radians(a)), math.sin(math.radians(a)), a) for a in range(0, 360, 45)]
    C60 = [(math.cos(math.radians(a)), math.sin(math.radians(a)), a) for a in range(0, 360, 60)]
    A = "pickup prompt, item strip, build page; item id %d"
    pedestal = [P(rect(0, -.94, 1.5, .22)), P([(-.36, -.83), (.36, -.83), (.5, -.2), (-.5, -.2)]), P(rect(0, -.1, 1.3, .2), "h")]
    wing_r = [P([(.08, .0), (.3, .95), (.6, .42)], bev=.85), P([(.08, .0), (.6, .42), (1.08, .6)], bev=.85), P([(.08, .0), (1.08, .6), (.74, .0)], bev=.85),
              P([(.08, .0), (.74, .0), (1.02, -.3)], bev=.85), P([(.08, .0), (1.02, -.3), (.36, -.4)], bev=.85)]
    wing_l = [P(mir(p["pts"]), bev=.85) for p in wing_r]
    return [
        # ------------------------------------------------------------------ the other 23 items
        M("glide_wake", "Glide Wake", "item", "Three flames rising along a streak on the floor: the burning line a wavedash leaves.", A % 2,
          [G("orange", P([(-1.08, -.66), (1.06, -.66), (1.06, -.36)], "h", bev=.8),
             P(flame(-.52, -.2, .2, .3, -.3)), P(flame(-.02, -.02, .27, .46, -.3)), P(flame(.6, .2, .36, .66, -.3)), P(flame(.63, .0, .16, .32, 0), "i"))]),
        M("shine_bell", "Shine Bell", "item", "The shine's hexagon ringing like a bell, sound arcs to both sides.", A % 3,
          [G("sky", P(ngon(0, 0, .58, 6, 0)), P(ngon(0, 0, .3, 6, 0), "i"), R((0, 0), .84, .11, -34, 34, "h"), R((0, 0), .84, .11, 146, 214, "h"),
             R((0, 0), 1.06, .09, -22, 22, "h"), R((0, 0), 1.06, .09, 158, 202, "h"))]),
        M("hop_shot", "Hop Shot", "item", "A star spat upward off two hop marks: a short hop fires a star.", A % 4,
          [G("gold", star5((0, .24), .78), P(ngon(0, .24, .16, 5, 90), "i"),
             bar((-.34, -.8), (0, -.58), .11, role="h"), bar((0, -.58), (.34, -.8), .11, role="h"), bar((-.24, -1.04), (0, -.88), .09, role="h"), bar((0, -.88), (.24, -1.04), .09, role="h"))]),
        M("ledge_spikes", "Ledge Spikes", "item", "Three ice spikes stabbing up from a ledge.", A % 5,
          [G("stone", P(rect(.12, -.86, 1.8, .36)), P(rect(-.86, -.62, .34, .16), "h")),
           G("ice", P([(-.62, -.68), (-.42, .42), (-.2, -.68)], bev=.6), P([(-.16, -.68), (.14, 1.08), (.44, -.68)], bev=.6), P([(.5, -.68), (.74, .2), (.96, -.68)], bev=.6),
             bar((.06, -.2), (.14, .5), .06, role="i"))]),
        M("whiff_echo", "Whiff Echo", "item", "A pale wisp with a swing arc beside it: the whiff swings again.", A % 6,
          [G("teal", P(ngon(-.1, .34, .56, 8, 22.5), "h"), P([(-.62, .14), (.4, .1), (-.2, -.6)], "h", bev=.8), P([(-.5, -.2), (-.02, -.36), (-.74, -1.0)], "h", bev=.75),
             R((-.1, .1), 1.1, .12, -36, 42), R((-.1, .1), .88, .08, -22, 28)),
           G("ink", P(ngon(-.3, .36, .1, 6), z=FRONT), P(ngon(.08, .36, .1, 6), z=FRONT))]),
        M("roll_mines", "Roll Mines", "item", "A spiked mine with a red light and two blink arcs over it.", A % 7,
          [G("steel", mine_body((0, -.2))),
           G("red", B((0, -.2), .27, z=.26), R((0, -.2), .98, .1, 60, 120), R((0, -.2), 1.2, .09, 70, 110))]),
        M("smash_quake", "Smash Quake", "item", "Rock teeth thrown up along the floor, tallest at the middle, leaning outward: shockwaves from a smash.", A % 12,
          [G("stone", P(rect(0, -.9, 2.16, .24))),
           G("sand", [P([(s*(x - .15), -.79), (s*(x + .15), -.79), (s*(x + ln), top)], bev=.65) for s in (1, -1) for x, ln, top in ((.24, .12, .62), (.6, .2, .1), (.9, .16, -.3))],
             bar((0, -.6), (0, .2), .08, role="h"), spark4((0, .56), .26, .08, role="h"))]),
        M("coin_magnet", "Coin Magnet", "item", "A horseshoe magnet pulling a coin up between its tips.", A % 18,
          [G("red", R((0, -.42), .5, .28, 180, 360), P(rect(-.5, -.14, .28, .58)), P(rect(.5, -.14, .28, .58))),
           G("steel", P(rect(-.5, .3, .28, .3), "h"), P(rect(.5, .3, .28, .3), "h")),
           G("gold", P(ngon(0, .78, .29, 10)), P(rect(0, .78, .08, .28), "i"), bar((-.26, .3), (-.2, .46), .06, role="h"), bar((.26, .3), (.2, .46), .06, role="h"))]),
        M("homing_ink", "Homing Ink", "item", "An ink stroke curling round to an arrowhead that points at a dot: shots that curve to foes.", A % 35,
          [G("magenta", R((0, 0), .74, .22, 20, 270), P([(0, -.42), (.5, -.74), (0, -1.08)], bev=.6), B((.06, .04), .22, "h"))]),
        M("fire_lace", "Fire Lace", "item", "Two loops of fire laced through each other, embers rising off them.", A % 36,
          [G("orange", R((-.36, -.2), .44, .17), R((.36, -.2), .44, .17, z=.12), P(ngon(0, -.2, .15, 4, 0), "h", z=(.2, .42)),
             P(flame(-.5, .62, .15, .26, .2), "h"), P(flame(.1, .82, .13, .24, -.2), "h"), P(flame(.62, .6, .12, .2, 0), "h"))]),
        M("laser_prism", "Laser Prism", "item", "A glass prism: one beam in, three beams fanning out.", A % 8,
          [G("ice", P([(-.5, -.6), (.5, -.6), (0, .64)], bev=.62)),
           G("heart", bar((-1.1, -.12), (-.3, -.12), .13, role="h"), bar((.26, -.06), (1.06, .5), .12), bar((.3, -.14), (1.12, -.14), .12), bar((.26, -.22), (1.06, -.78), .12))]),
        M("crouch_thorns", "Crouch Thorns", "item", "A low mound bristling with thorns: crouch under a hit and they strike back.", A % 21,
          [G("green", P(arcpts((0, -.78), .72, 0, 180, 7)), P(rect(0, -.86, 1.7, .18)),
             [P(kite((.6*math.cos(math.radians(a)), -.78 + .6*math.sin(math.radians(a))), a, .62 if a == 90 else .5, .13, .3), "h", bev=.55) for a in (22, 56, 90, 124, 158)])]),
        M("perfect_guard", "Perfect Guard", "item", "A gold shield with the powershield flash on its face.", A % 26,
          [G("gold", P(scl(DP.SH, 1.08)), spark4((0, .04), .58, .15, role="h", z=FRONT)),
           G("moon", B((0, .04), .13, "h", z=.34))]),
        M("brimstone_jab", "Brimstone Jab", "item", "A horned ball of fire at the head of a wide red beam.", A % 27,
          [G("crimson", P(rect(.34, -.06, 1.5, .5)), P(rect(.34, -.06, 1.5, .18), "h", z=FRONT), B((-.5, -.06), .5, "h", z=.1),
             P([(-.84, .28), (-.92, .84), (-.5, .42)], bev=.7), P([(-.16, .28), (-.08, .84), (-.5, .42)], bev=.7))]),
        M("demon_wings", "Demon Wings", "item", "A pair of dark bat wings with scalloped edges: one more air jump.", A % 28,
          [G("dusk", wing_r, wing_l, B((0, .0), .2, "h", z=.1)),
           G("crimson", P([(.3, .95), (.2, 1.12), (.42, 1.04)], "h", bev=.7), P([(-.3, .95), (-.2, 1.12), (-.42, 1.04)], "h", bev=.7))]),
        M("crimson_count", "Crimson Count", "item", "A red diamond sigil with an eye, three tally marks under it: one sigil a KO.", A % 29,
          [G("crimson", DP.frame(ngon(0, .22, .86, 4, 90), .66, (0, .22)), P(ngon(0, .22, .26, 4, 90), "h"),
             P(rect(-.3, -.94, .13, .34), "h"), P(rect(0, -.94, .13, .34), "h"), P(rect(.3, -.94, .13, .34), "h"))]),
        M("bloodlust", "Bloodlust", "item", "A drop of blood with three tendrils whipping out of it.", A % 30,
          [G("crimson", P(flame(0, -.06, .4, .56, 0)), P(flame(.02, -.22, .16, .24, 0), "i"),
             R((.58, .46), .44, .13, -80, 150), R((-.6, .34), .42, .13, 30, 250), R((.2, -.66), .32, .12, 170, 400))]),
        M("halo", "Halo", "item", "A broad gold ring seen from below, light falling from it.", A % 31,
          [G("cream", R((0, .34), .74, .2, tilt=48, zh=.07), P(kite((0, -.1), 270, .9, .14, .3), "h", bev=.5),
             P(kite((-.36, -.14), 250, .6, .1, .3), "h", bev=.5), P(kite((.36, -.14), 290, .6, .1, .3), "h", bev=.5)),
           G("gold", spark4((.84, .9), .18, .06, role="h"))]),
        M("guardian_feather", "Guardian Feather", "item", "One large white feather held upright against a hit.", A % 32,
          [G("moon", [P(rot(q, -24), bev=.8) for q in feather_barbs()], P(rot([(-.25, .6), (.25, .6), (0, 1.08)], -24), bev=.75),
             bar(rot([(0, -1.04)], -24)[0], rot([(0, .9)], -24)[0], .09, role="h", z=(.1, .32))),
           G("gold", spark4((.8, -.66), .2, .07, role="h"))]),
        M("dodge_orb", "Dodge Orb", "item", "An orb shooting down and forward, three streaks behind it: it leads the airdodge.", A % 44,
          [G("ice", B((.46, -.42), .46), bar((-.92, .98), (.0, .14), .13, role="h"), bar((-.98, .5), (-.3, -.1), .1, role="h"), bar((-.44, 1.0), (.3, .34), .1, role="h"))]),
        M("nair_ring", "Nair Ring", "item", "Six orbs on a ring around a spinning mark: orbs that circle a nair.", A % 45,
          [G("orange", R((0, 0), .78, .07), [B((.78*c, .78*s), .21, "h", z=.1) for c, s, a in C60],
             [P(kite((0, 0), a, .44, .12, .5), bev=.55) for a in (45, 135, 225, 315)])]),
        M("dash_orb", "Dash Orb", "item", "An orb running ahead of three speed lines: it leads the dash.", A % 46,
          [G("green", B((.56, 0), .46), bar((-1.08, .34), (.06, .34), .12, role="h"), bar((-.8, 0), (-.04, 0), .12, role="h"), bar((-1.08, -.34), (.06, -.34), .12, role="h"))]),
        M("shield_orbs", "Shield Orbs", "item", "A shield bubble with three small orbs standing guard around it.", A % 47,
          [G("lilac", B((0, -.04), .7), bar((-.4, .18), (-.16, .42), .1, role="i", z=(.5, .7)), B((0, .92), .18, "h"), B((-.86, -.52), .18, "h"), B((.86, -.52), .18, "h"))]),
        # ------------------------------------------------------------------ keyword glyphs (design 11.9)
        M("kw_timing", "Keyword: timing", "kw", "A clock face with two hands: ON / WHEN / WHILE / AFTER.", "before a timing keyword on an item card; the glossary's Timing header",
          [G("gold", R((0, 0), .82, .2), bar((0, 0), (0, .52), .13, role="h"), bar((0, 0), (.36, -.2), .13, role="h"), B((0, 0), .12, "h", z=.1))]),
        M("kw_condition", "Keyword: condition", "kw", "One path forking in two: IF.", "before a condition keyword; the glossary's Conditions header",
          [G("lilac", bar((0, -.98), (0, -.1), .2), bar((0, -.14), (-.56, .52), .2), bar((0, -.14), (.56, .52), .2),
             P([(-.84, .44), (-.5, .98), (-.28, .52)], "h", bev=.6), P([(.84, .44), (.5, .98), (.28, .52)], "h", bev=.6))]),
        M("kw_move", "Keyword: move", "kw", "A slash with an impact star at its end: NAIR, JAB, SMASH and the other attacks.", "before a move keyword; the glossary's Moves header",
          [G("red", R((.3, -.3), 1.0, .2, 100, 190), R((.3, -.3), .72, .1, 112, 178, "h"), spark4((.42, .62), .42, .12, role="h"))]),
        M("kw_tech", "Keyword: tech", "kw", "Two chevrons skimming a ground line: WAVEDASH, SHORT HOP, L-CANCEL and the other tech.", "before a tech keyword; the glossary's Tech header",
          [G("green", bar((-.7, .62), (-.1, .0), .2), bar((-.1, .0), (-.7, -.5), .2), bar((.0, .62), (.6, .0), .2, role="h"), bar((.6, .0), (.0, -.5), .2, role="h"),
             P(rect(0, -.86, 1.9, .16)))]),
        M("kw_outcome", "Keyword: outcome", "kw", "A jagged impact burst: HIT, WHIFF, SHIELD, KO, TAKE HIT.", "before an outcome keyword; the glossary's Outcomes header",
          [G("orange", [P(kite((0, 0), a, 1.0 if i % 2 == 0 else .62, .26, .34), bev=.6) for i, (c, s, a) in enumerate(C45)], P(ngon(0, 0, .3, 8, 22.5), "i"))]),
        M("kw_effect", "Keyword: effect", "kw", "An orb inside a ring: PROJECTILE, BURST, TRAIL, AURA, ORB, BEAM.", "before an effect keyword; the glossary's Effects header",
          [G("sky", B((0, 0), .4, "h"), R((0, 0), .86, .17, 20, 160), R((0, 0), .86, .17, 200, 340))]),
        M("kw_status", "Keyword: status", "kw", "A drop with a mark in it: BURN, CHILL, MARK.", "before a status keyword; the glossary's Statuses header",
          [G("violet", P(flame(0, -.06, .66, .98, 0)), P(ngon(0, -.3, .24, 6), "i"))]),
        M("kw_stack", "Keyword: stacking", "kw", "Three slabs stacked, the top one bright: STACK, PER STACK.", "before a stacking keyword; the stack count on a chip",
          [G("stone", P(rect(0, -.66, 1.7, .4)), P(rect(0, -.06, 1.36, .4)), P(rect(0, .54, 1.0, .4), "h"))]),
        M("st_burn", "Status: Burn", "kw", "A single flame with a hot core.", "the BURN keyword, the Burning status tag over a foe",
          [G("orange", P(flame(0, -.04, .64, 1.0, -.25)), P(flame(.04, -.4, .28, .48, 0), "i"))]),
        M("st_chill", "Status: Chill", "kw", "A six-armed ice crystal.", "the CHILL keyword, the Chilled status tag over a foe",
          [G("ice", [P(rot([(0, 1.02), (.16, .8), (.16, -.8), (0, -1.02), (-.16, -.8), (-.16, .8)], a), bev=.6) for a in (0, 60, 120)], P(ngon(0, 0, .32, 6, 0), "h", z=(-.1, .34)))]),
        M("st_mark", "Status: Mark", "kw", "A crosshair: a ring, four ticks and a centre dot.", "the MARK keyword, the Marked status tag over a foe",
          [G("heart", R((0, 0), .66, .17), [bar((.5*c, .5*s), (1.04*c, 1.04*s), .16, role="h") for c, s, a in C45[::2]], B((0, 0), .16, "h"))]),
        # ------------------------------------------------------------------ tag emblems (synergy banners)
        M("tag_projectile", "Tag: projectile", "tag", "An arrow in flight.", "synergy banners and modifier cards for TG_PROJECTILE",
          [G("pink", bar((-1.0, -.5), (.34, .17), .2), P([(.14, .62), (1.02, .51), (.5, -.2)], bev=.6),
             P([(-1.04, -.2), (-.68, -.34), (-.86, -.66)], "h", bev=.6), P([(-.72, -.76), (-.6, -.4), (-.92, -.58)], "h", bev=.6))]),
        M("tag_burst", "Tag: burst", "tag", "A ring thrown outward in four pieces from a core.", "synergy banners and modifier cards for TG_BURST",
          [G("orange", B((0, 0), .3, "h"), [R((0, 0), .74, .22, a + 12, a + 78) for a in (0, 90, 180, 270)], [P(kite((.92*c, .92*s), a, .2, .09, .4), "h", bev=.5) for c, s, a in C45[1::2]])]),
        M("tag_trail", "Tag: trail", "tag", "A line of marks growing toward the front.", "synergy banners and modifier cards for TG_TRAIL",
          [G("green", P(ngon(-.86, -.5, .14, 6)), P(ngon(-.44, -.22, .2, 6)), P(ngon(.06, .08, .27, 6)), P(ngon(.66, .42, .36, 6), "h"))]),
        M("tag_orbital", "Tag: orbital", "tag", "A body with a moon on a ring around it.", "synergy banners and modifier cards for TG_ORBITAL",
          [G("moon", B((0, 0), .46), R((0, 0), .96, .08, tilt=56, zh=.04, role="h"), B((.82, -.3), .18, "h", z=.5))]),
        M("tag_spark", "Tag: spark", "tag", "A lightning bolt.", "synergy banners and modifier cards for TG_SPARK",
          [G("volt", P([(.42, 1.04), (-.7, -.16), (.12, -.16)], bev=.6), P([(-.42, -1.04), (.7, .16), (-.12, .16)], bev=.6))]),
        M("tag_launch", "Tag: launch", "tag", "A thick arrow going up off a line.", "synergy banners and modifier cards for TG_LAUNCH",
          [G("red", P(rect(0, -.2, .4, .9)), P([(-.62, .2), (.62, .2), (0, 1.02)], bev=.65), P(rect(0, -.9, 1.5, .16), "h"))]),
        # ------------------------------------------------------------------ HUD bits
        M("hud_a_button", "A button", "hud", "A round green button with an A on it.", "the pickup prompt over a pedestal in range (V_PED_NEAR); the map's \"A takes it\"",
          [G("green", P(ngon(0, 0, .98, 12, 15), z=(-.24, .0), bev=.86), P(ngon(0, 0, .8, 12, 15), z=(-.1, .16), bev=.8)),
           G("moon", bar((-.3, -.42), (0, .46), .16, role="h", z=FRONT), bar((.3, -.42), (0, .46), .16, role="h", z=FRONT), bar((-.2, -.14), (.2, -.14), .13, role="h", z=FRONT))]),
        M("hud_coin", "Coin", "hud", "A gold coin with a slot mark.", "the coin count, shop prices, the cash-out line",
          [G("gold", P(ngon(0, 0, .9, 12, 15), bev=.84), R((0, 0), .62, .08, role="h", z=.2, zh=.04), P(rect(0, 0, .16, .62), "i"))]),
        M("hud_pip_on", "Quality pip, lit", "hud", "A small gold diamond.", "an item's quality (0 to 4 pips) on its card and in the shop",
          [G("gold", P(ngon(0, 0, .8, 4, 90), bev=.6), P(ngon(0, 0, .3, 4, 90), "i"))]),
        M("hud_pip_off", "Quality pip, empty", "hud", "The same diamond as an empty stone outline.", "the unlit pips of an item's quality",
          [G("stone", DP.frame(ngon(0, 0, .8, 4, 90), .6, (0, 0)))]),
        M("hud_slot_lost", "Stock slot lost", "hud", "The heart container struck through with a red cross: a slot a deal takes.", "the devil deal's price tag (one per slot), the stock row as a slot leaves",
          [G("stone", R((-.4, .2), .4, .18, 0, 225), R((.4, .2), .4, .18, -45, 180), bar((-.69, -.07), (0, -.86), .18), bar((.69, -.07), (0, -.86), .18), B((0, -.84), .1), B((0, .2), .09)),
           G("red", cross_x(0, -.06, .72, .2, z=(.16, .44)))]),
        M("frame_curse", "Frame: curse", "frame", "The frame in purple with a thorn at each corner pointing in: it comes with a drawback.", "behind a curse-pool item (design 7, the `curse` pool) when that pool is built",
          [G("magenta", frame_ring(), [P([(sx*1.0, sy*.62), (sx*.62, sy*1.0), (sx*.5, sy*.5)], z=BACK, bev=.6) for sx in (1, -1) for sy in (1, -1)])]),
        # ------------------------------------------------------------------ projectile bodies (envoy_data::view PJ_*)
        M("pj_star", "Shot: star", "pj", "A plain five-point star.", "PJ_STAR in the world (Hop Shot); a size that reads at 6 world units",
          [G("gold", star5((0, -.04), 1.04), P(ngon(0, -.04, .2, 5, 90), "i"))]),
        M("pj_shard", "Shot: ice shard", "pj", "A long ice splinter.", "PJ_SHARD in the world (Ice Glass); turn it to its velocity",
          [G("ice", P([(1.2, 0), (-.3, .44), (-1.16, .0), (-.3, -.44)], bev=.55), bar((-.56, .0), (.7, .0), .09, role="i"))], spin=38),
        M("pj_feather", "Shot: feather", "pj", "A dark feather with a red quill.", "PJ_FEATHER in the world (Demon Wings)",
          [G("dusk", [P(rot(q, -90), bev=.8) for q in feather_barbs()], P(rot([(-.25, .6), (.25, .6), (0, 1.1)], -90), bev=.75)),
           G("crimson", bar((-1.1, 0), (.92, 0), .1, role="h", z=(.1, .32)))], spin=38),
        M("pj_coin", "Shot: coin", "pj", "A coin with a round hole, for turning in the air.", "PJ_COIN in the world (Coin Magnet): spin it by yaw",
          [G("gold", R((0, 0), .66, .5), R((0, 0), .98, .1, role="h"))]),
        M("pj_mine", "Shot: mine", "pj", "The spiked mine with its red light.", "the mine on the floor (Roll Mines); pulse the light with the blink",
          [G("steel", mine_body((0, 0))),
           G("red", B((0, 0), .27, z=.26))]),
        M("pj_moon", "Shot: moon", "pj", "A round moon with two craters.", "the Moon Drops orbital's body",
          [G("moon", B((0, 0), .98, "h")),
           G("stone", P(ngon(-.3, .26, .22, 8, 22.5), z=(.62, .86), bev=.7), P(ngon(.34, -.2, .15, 8, 22.5), z=(.66, .86), bev=.7), P(ngon(-.2, -.46, .1, 8, 22.5), z=(.62, .84), bev=.7))]),
        M("pj_bolt", "Shot: prism bolt", "pj", "A thin bright bolt, pointed at both ends.", "PJ_PRISM in the world (Laser Prism's three bolts)",
          [G("heart", P([(1.2, 0), (.4, .36), (-1.2, 0), (.4, -.36)], bev=.6), P([(.9, 0), (.4, .13), (-.6, 0), (.4, -.13)], "i", bev=.6))], spin=38),
        # ------------------------------------------------------------------ map furniture
        M("map_dot", "Map: path dot", "map", "A small stone bead.", "the dotted links between act-map nodes",
          [G("stone", P(ngon(0, 0, .5, 8, 22.5), bev=.6))]),
        M("map_cursor", "Map: cursor", "map", "A gold arrowhead pointing down at the chosen node, with a bright rim.", "P1's node cursor on the road screen, the pause Run page and the in-match room choice",
          [G("gold", P([(-.8, .84), (.8, .84), (0, -.96)], bev=.62), P([(-.36, .58), (.36, .58), (0, -.26)], "i", bev=.6), P(rect(0, 1.0, 1.7, .16), "h"))]),
        M("map_flag", "Map: wish flag", "map", "A pennant on a pole: where P2 would like to go.", "P2's wish flag on the act map",
          [G("wood", P(rect(-.5, -.1, .13, 2.0)), B((-.5, .98), .12, "h")),
           G("teal", P([(-.43, .86), (.9, .46), (-.43, .06)], bev=.7), P([(-.43, .64), (.34, .46), (-.43, .28)], "i", bev=.6)),
           G("stone", P(rect(-.5, -1.04, .8, .16)))]),
        M("map_tick", "Map: visited", "map", "A green tick.", "a cleared node on the act map",
          [G("green", bar((-.84, -.06), (-.3, -.66), .3), bar((-.3, -.66), (.86, .68), .3))]),
        M("map_door_unknown", "Map: unknown door", "map", "A stone arch with a question mark in it: the door after the boss, before it is known.", "the door node (row 7) until the devil or angel door opens; its chance tag",
          [G("stone", P(arch(0, -1.0, 1.3, 2.0))),
           G("ink", P(arch(0, -1.0, .9, 1.68), z=(.1, .3), bev=.86)),
           G("violet", qmark(0, -.14, 1.3, role="h", z=.3))]),
        # ------------------------------------------------------------------ world pieces
        M("w_pedestal", "World: pedestal", "world", "The stone pedestal alone: a foot, a tapered column and a bright top slab.", "under every floating item (item room, boss drop, devil, angel, secret): replaces the FX_BEAM stone in items_ui::pedestals",
          [G("stone", pedestal, P(ngon(0, -.5, .16, 4, 90), "h", z=FRONT))]),
        M("w_pedestal_locked", "World: locked pedestal", "world", "The pedestal crossed out in red: a deal you cannot afford.", "a devil deal that would take the last stock slot (drawn crossed)",
          [G("stone", pedestal),
           G("red", cross_x(0, .3, .62, .2, z=(.0, .3)))]),
        M("w_wall_0", "World: wall, whole", "world", "The brick wall with a hairline crack.", "the secret wall before any hit",
          [G("stone", bricks()),
           G("violet", bar((.1, .3), (-.06, .0), .07, z=(.0, .34), bev=.6), bar((-.06, .0), (.08, -.26), .07, z=(.0, .34), bev=.6))]),
        M("w_wall_1", "World: wall, cracked", "world", "The wall with the crack run top to bottom.", "the secret wall after one hit",
          [G("stone", bricks()),
           G("violet", [bar(a, b, .13, z=(.0, .36), bev=.6) for a, b in (((.16, .84), (-.14, .44)), ((-.14, .44), (.18, .1)), ((.18, .1), (-.12, -.32)), ((-.12, -.32), (.1, -.84)))])]),
        M("w_wall_2", "World: wall, breaking", "world", "The wall with a brick knocked out and light pouring through the gap.", "the secret wall after two hits (the third breaks it)",
          [G("stone", bricks(skip=(3,)), P(rot(rect(.2, -.98, .5, .26), 14), "h")),
           G("violet", P(rect(0, 0, .86, .4), "h", z=(-.14, .06), bev=.8),
             [bar(a, b, .14, z=(.0, .36), bev=.6) for a, b in (((.16, .84), (-.1, .26)), ((.1, -.26), (-.12, -.5)), ((-.12, -.5), (.1, -.84)), ((-.1, .26), (-.62, .5)), ((.1, -.26), (.66, -.44)))])]),
        M("w_door_devil", "World: devil door", "world", "The red door at full height on its step, horned, eyes lit.", "the door standing in the held match when the devil deal opens",
          [G("crimson", P(arch(0, -.86, 1.2, 1.66)), P(rect(0, -.96, 1.7, .2)), horn(-1, (-.46, .3), role="h"), horn(1, (.46, .3), role="h")),
           G("ink", P(arch(0, -.86, .8, 1.36), z=(.1, .3), bev=.86)),
           G("orange", P([(-.3, .14), (-.06, -.02), (-.3, -.02)], "h", z=(.26, .4), bev=.6), P([(.3, .14), (.06, -.02), (.3, -.02)], "h", z=(.26, .4), bev=.6),
             P(rect(0, -.6, .5, .08), "h", z=(.26, .38)))]),
        M("w_door_angel", "World: angel door", "world", "The pale door at full height on its step, a halo over it, light in the doorway.", "the door standing in the held match when the angel room opens",
          [G("cream", P(arch(0, -.86, 1.06, 1.62)), P(rect(0, -.96, 1.7, .2))),
           G("sky", P(arch(0, -.86, .7, 1.36), z=(.1, .3), bev=.8), P(arch(0, -.86, .36, 1.0), "h", z=(.2, .36), bev=.8)),
           G("moon", [P(kite((sd*.44, -.2), 90 - sd*a, L, .15), "h", bev=.6) for sd in (1, -1) for a, L in ((70, .66), (46, .78), (22, .64))]),
           G("gold", R((0, .94), .42, .11, tilt=44, role="h", zh=.05), spark4((0, -.1), .3, .09, role="h", z=(.34, .44)))]),
    ]
