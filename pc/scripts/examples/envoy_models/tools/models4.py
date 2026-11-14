"""Envoy models, chunk 4: curse-pool marks and the two curse items, camp and road scenery, and the animated sprites
(a few frames each for pieces that should move in 2D). `make_models1.py` imports this file and owns the helpers, the
palette and every writer; this file is only shapes. Every coordinate is original, written for this project.
"""
import math

import drive_parts as DP
from drive_parts import P, R, B, bar, flame, kite, ngon, rect, rot, mov, scl, mir, arcpts
from models2 import star5, qmark, cross_x
from models3 import arrowhead


def models(h):
    G, M, FRONT, BACK = h.G, h.M, h.FRONT, h.BACK
    heart, spark4, crescent, arch, horn = h.heart, h.spark4, h.crescent, h.arch, h.horn
    C = "the drawback line on a curse-pool card (design 7, the `curse` pool): %s"
    skull = lambda cx, cy, s, **kw: [P(ngon(cx, cy + .14*s, .6*s, 8, 22.5), **kw), P(rect(cx, cy - .46*s, .56*s, .4*s), **kw)]
    eyes = lambda cx, cy, s, z=FRONT: [P(ngon(cx - .24*s, cy + .1*s, .16*s, 6), z=z), P(ngon(cx + .24*s, cy + .1*s, .16*s, 6), z=z), P([(cx, cy - .12*s), (cx + .08*s, cy - .28*s), (cx - .08*s, cy - .28*s)], z=z)]
    return [
        # ------------------------------------------------------------------ the curse pool
        M("curse_mark", "Curse: mark", "curse", "A purple skull under three thorns: this item is cursed.", "the corner of a curse-pool item's card and chip; the curse pool's pedestal",
          [G("magenta", skull(0, -.2, 1.2), [P(kite((x, .5), a, .56, .13, .3), "h", bev=.55) for x, a in ((-.4, 110), (0, 90), (.4, 70))]),
           G("ink", eyes(0, -.2, 1.2), P(rect(-.12, -.84, .06, .26), z=FRONT), P(rect(.12, -.84, .06, .26), z=FRONT))]),
        M("curse_shared", "Curse: shared", "curse", "Two arrows passing each other: foes get it too.", C % "the effect also works for foes (Haunted Shell)",
          [G("magenta", P(rect(-.2, .46, 1.3, .3)), P([(.34, .9), (1.06, .46), (.34, .02)], "h", bev=.66)),
           G("red", P(rect(.2, -.46, 1.3, .3)), P([(-.34, -.02), (-1.06, -.46), (-.34, -.9)], "h", bev=.66))]),
        M("curse_fragile", "Curse: fragile", "curse", "A heart split by a crack, a chip falling from it: you are launched further.", C % "you take more knockback (Glass Fist)",
          [G("heart", heart(0, -.1, 1.1)),
           G("ink", [bar(a, b, .09, z=FRONT) for a, b in (((.06, .5), (-.14, .14)), ((-.14, .14), (.14, -.12)), ((.14, -.12), (-.06, -.5)), ((-.06, -.5), (.04, -.86)))]),
           G("moon", P([(.74, -.5), (1.04, -.62), (.84, -.9)], "h", bev=.6))]),
        M("curse_weak", "Curse: weak", "curse", "A sword snapped in two, the point hanging off to one side.", C % "you deal less",
          [G("steel", P(rot([(-.13, -.3), (.13, -.3), (.13, .2), (-.02, .34), (-.13, .12)], 14), bev=.6), P(mov(rot([(-.13, -.3), (.1, -.14), (.13, .3), (0, .56), (-.13, .3)], -38), .46, .64), bev=.6)),
           G("gold", P(rot(rect(0, -.38, .7, .15), 14)), B(rot([(0, -.94)], 14)[0], .12, "h")),
           G("wood", P(rot(rect(0, -.66, .15, .44), 14)))]),
        M("curse_slow", "Curse: slow", "curse", "A snail: a spiral shell on a low body with two feelers.", C % "you move or act slower",
          [G("sand", R((.14, .06), .6, .22), R((.14, .06), .26, .16, 0, 250, "h")),
           G("green", P([(-1.06, -.84), (.86, -.84), (.74, -.56), (-.86, -.56)], bev=.8), B((-.86, -.36), .24), bar((-.94, -.2), (-1.04, .16), .08, role="h"), bar((-.74, -.2), (-.66, .16), .08, role="h"),
             B((-1.04, .2), .07, "h"), B((-.66, .2), .07, "h"))]),
        M("curse_heavy", "Curse: heavy", "curse", "An iron weight with a ring on top and a down arrow on its face.", C % "you fall faster or jump lower",
          [G("steel", P([(-.5, .3), (.5, .3), (.9, -.9), (-.9, -.9)], bev=.8), R((0, .5), .3, .13, 0, 180, "h")),
           G("ink", P(rect(0, -.2, .16, .44), z=FRONT), P([(-.26, -.38), (.26, -.38), (0, -.7)], z=FRONT, bev=.7))]),
        M("curse_costly", "Curse: costly", "curse", "A coin with a red arrow falling out of it.", C % "it costs coins, or shops cost more",
          [G("gold", P(ngon(-.1, .26, .72, 12, 15), bev=.84), P(rect(-.1, .26, .14, .5), "i")),
           G("red", P(rect(.62, -.34, .22, .5), z=FRONT), P([(.3, -.56), (.94, -.56), (.62, -1.0)], z=FRONT, bev=.66))]),
        M("curse_stock", "Curse: stock", "curse", "The heart container with a red minus on its shoulder: a stock slot gone.", C % "it takes a stock slot or a stock",
          [G("stone", R((-.46, .12), .4, .18, 0, 225), R((.34, .12), .4, .18, -45, 180), bar((-.75, -.15), (-.06, -.94), .18), bar((.63, -.15), (-.06, -.94), .18), B((-.06, -.92), .1), B((-.06, .12), .09)),
           G("red", P(rect(.72, .76, .6, .2), z=(.1, .4), bev=.8))]),
        M("haunted_shell", "Haunted Shell", "item", "A skull peering out of a spiral shell, a wisp trailing off it.", "pickup prompt, item strip, build page; the curse pool's item 33",
          [G("teal", R((.2, -.1), .7, .26, 20, 300), R((.2, -.1), .34, .16, 60, 300, "h")),
           G("moon", skull(-.4, -.3, .8, role="h", z=(.0, .3)), P([(.5, .6), (.96, .96), (.84, .5)], "h", bev=.7)),
           G("ink", eyes(-.4, -.3, .8, z=(.3, .42)))]),
        M("glass_fist", "Glass Fist", "item", "A fist of clear glass, four knuckles and a thumb, shards coming off it.", "pickup prompt, item strip, build page; the curse pool's item 34",
          [G("ice", P(rect(-.14, -.3, 1.2, .9), bev=.82), [P(rect(x, .32, .28, .44), "h", bev=.74) for x in (-.59, -.29, .01, .31)], P(rot(rect(.62, -.22, .3, .66), -18), bev=.76),
             P(rect(-.14, -.9, .8, .26), bev=.8), P([(.86, .5), (1.1, .9), (.72, .78)], "h", bev=.6), P([(.9, .1), (1.14, .26), (1.0, -.06)], "h", bev=.6)),
           G("ink", bar((-.5, -.1), (-.24, -.36), .06, z=FRONT), bar((-.24, -.36), (-.36, -.6), .06, z=FRONT))]),
        # ------------------------------------------------------------------ camp and road scenery
        M("camp_tent", "Camp: tent", "camp", "A teal tent with a cream stripe, its door flap open on the dark inside.", "the camp screen behind the party cards",
          [G("teal", P([(-1.06, -.84), (1.06, -.84), (0, .9)], bev=.8), B((0, .96), .09, "h")),
           G("cream", P([(-.56, -.02), (.56, -.02), (.4, .26), (-.4, .26)], "h", z=FRONT, bev=.86)),
           G("ink", P([(-.4, -.84), (.4, -.84), (0, -.1)], z=FRONT, bev=.8)),
           G("wood", P(rect(0, -.94, 2.2, .14)), P(rect(-1.04, -.8, .08, .26)), P(rect(1.04, -.8, .08, .26)))]),
        M("camp_lantern", "Camp: lantern", "camp", "A lantern with a lit glass, a cap and a carrying ring.", "the camp screen; a road stop at night",
          [G("steel", R((0, .72), .26, .09, 0, 180), P([(-.4, .44), (.4, .44), (.26, .66), (-.26, .66)], bev=.8), P(rect(0, -.84, .9, .2))),
           G("gold", P([(-.36, .4), (.36, .4), (.44, -.7), (-.44, -.7)], bev=.8), P(flame(0, -.2, .2, .4, 0), "i")),
           G("steel", P(rect(-.44, -.16, .07, 1.14), z=FRONT), P(rect(.44, -.16, .07, 1.14), z=FRONT))]),
        M("camp_board", "Camp: notice board", "camp", "A board on two posts with three notes pinned to it.", "the camp's run record (best loop, last run); the pause Run page header",
          [G("wood", P(rect(-.8, -.3, .14, 1.5)), P(rect(.8, -.3, .14, 1.5)), P(rect(0, .2, 1.9, 1.2), "h", bev=.9), P(rect(0, .9, 2.1, .14))),
           G("cream", P(rot(rect(-.5, .36, .5, .6), 6), "h", z=FRONT, bev=.9), P(rot(rect(.14, .08, .5, .5), -5), "h", z=FRONT, bev=.9), P(rot(rect(.6, .46, .36, .44), 8), "h", z=FRONT, bev=.9)),
           G("red", B((-.5, .62), .06, z=.4), B((.14, .3), .06, z=.4), B((.6, .64), .06, z=.4))]),
        M("camp_seat", "Camp: log seat", "camp", "A log on its side with a mushroom growing by it.", "the camp screen, beside the fire",
          [G("wood", P(rect(.06, -.5, 1.8, .5), bev=.86), P(ngon(-.84, -.5, .27, 8, 22.5), "h", z=(-.26, .3)), R((-.84, -.5), .13, .05, role="h", z=.3, zh=.03)),
           G("red", P(arcpts((.74, -.06), .24, 0, 180, 5))), G("cream", P(rect(.74, -.16, .1, .2), "h"))]),
        M("camp_pot", "Camp: cooking pot", "camp", "A pot hung from a tripod over a small flame, steam curling up.", "the campfire room; the camp screen",
          [G("wood", bar((-.76, -.94), (0, .9), .08), bar((.76, -.94), (0, .9), .08), bar((0, .86), (0, .1), .05)),
           G("steel", P([(-.5, .1), (.5, .1), (.4, -.4), (-.4, -.4)], bev=.8), P(rect(0, .14, 1.14, .1), "h")),
           G("orange", P(flame(0, -.7, .2, .26, 0)), P(flame(0, -.76, .09, .14, 0), "i")),
           G("moon", R((-.2, .44), .14, .05, 200, 380, "h"), R((.2, .56), .14, .05, 20, 200, "h"))]),
        M("road_sign", "Road: signpost", "camp", "A post with two arrow boards pointing opposite ways.", "the road screen at a fork; the act map's foot",
          [G("wood", P(rect(0, -.2, .16, 1.7)), P(rect(0, -.98, .6, .14))),
           G("cream", P([(-.04, .9), (.7, .9), (1.0, .68), (.7, .46), (-.04, .46)], "h", bev=.84), P([(.04, .3), (-.7, .3), (-1.0, .08), (-.7, -.14), (.04, -.14)], bev=.84)),
           G("ink", P(rect(.36, .68, .44, .07), z=FRONT), P(rect(-.36, .08, .44, .07), z=FRONT))]),
        M("road_stone", "Road: milestone", "camp", "A low round-topped stone with one pip cut in it, grass at its foot.", "the road screen: one per fight cleared",
          [G("stone", P(arch(0, -.9, 1.1, 1.5)), P(ngon(0, -.1, .15, 4, 90), "h", z=FRONT)),
           G("green", P([(-.9, -.9), (-.72, -.46), (-.56, -.9)], bev=.7), P([(-.6, -.9), (-.46, -.6), (-.34, -.9)], "h", bev=.7), P([(.5, -.9), (.7, -.5), (.86, -.9)], bev=.7))]),
        M("road_pine", "Road: pine", "camp", "A pine of three stacked tiers on a short trunk.", "the road and camp backdrops",
          [G("wood", P(rect(0, -.86, .2, .4))),
           G("green", P([(-.8, -.66), (.8, -.66), (0, .1)], bev=.8), P([(-.62, -.2), (.62, -.2), (0, .56)], bev=.8), P([(-.44, .26), (.44, .26), (0, 1.04)], "h", bev=.8))]),
        M("road_bush", "Road: bush", "camp", "A bush of three round lobes with red berries.", "the road and camp backdrops",
          [G("green", P(ngon(-.5, -.5, .46, 8, 22.5)), P(ngon(.5, -.5, .46, 8, 22.5)), P(ngon(0, -.2, .56, 8, 22.5), "h", z=(-.1, .3))),
           G("red", B((-.3, -.36), .08, z=.34), B((.2, -.1), .08, z=.4), B((.5, -.5), .08, z=.3))]),
        M("road_rock", "Road: rock", "camp", "A boulder with a smaller stone leaning on it.", "the road and camp backdrops; a secret wall's rubble",
          [G("stone", P([(-.8, -.9), (.5, -.9), (.74, -.3), (.3, .3), (-.4, .36), (-.9, -.2)], bev=.72), P([(.34, -.9), (1.04, -.9), (.96, -.5), (.6, -.36)], "h", bev=.7))]),
        M("road_fence", "Road: fence", "camp", "Two posts and two rails.", "the road screen's edge; the camp's edge",
          [G("wood", P(rect(-.7, -.3, .16, 1.3)), P(rect(.7, -.3, .16, 1.3)), P(rect(0, .0, 2.1, .16), "h", z=FRONT), P(rect(0, -.5, 2.1, .16), "h", z=FRONT))]),
        M("road_gate", "Road: finish gate", "camp", "Two poles with a string of red and gold pennants between them.", "the end of the road (the boss ahead); the loop-cleared banner",
          [G("wood", P(rect(-.96, -.1, .13, 1.9)), P(rect(.96, -.1, .13, 1.9)), bar((-.96, .8), (.96, .8), .05)),
           G("red", [P([(x - .17, .78), (x + .17, .78), (x, .36)], bev=.7) for x in (-.62, 0, .62)]),
           G("gold", [P([(x - .17, .78), (x + .17, .78), (x, .36)], "h", bev=.7) for x in (-.31, .31)], B((-.96, .9), .1, "h"), B((.96, .9), .1, "h"))]),
    ]


def anims(h):
    """key -> (frames, fn(i) -> groups): the animated sprites. Frame 0 is the model's own look; the mesh is not animated."""
    G, FRONT = h.G, h.FRONT
    heart, spark4 = h.heart, h.spark4

    def campfire(i, room=False):
        lean = (-.2, -.06, .1, -.08)[i]; tall = (1.0, 1.08, .97, 1.05)[i]
        logs = G("wood", bar((-.86, -.94), (.7, -.5), .27, z=(-.26, .08)), bar((.86, -.94), (-.7, -.5), .27, z=(-.1, .26)))
        fl = G("orange", P(flame(0, .14 + .84*(tall - 1), .56, .84*tall, lean)), P(flame(.04, -.12, .25, .44*(2 - tall), -lean*.5), "i"),
               P(flame(-.5, -.28, .19, .34*(2 - tall), .15), "h"), B((.62, .5 + .14*i), .07, "h"), B((.4 - .06*i, .8 + .08*i), .05, "h"))
        if room:
            return [G("stone", [P(ngon(x, -.94, .16, 6), z=(-.06, .34)) for x in (-.9, -.54, .54, .9)]), logs, fl]
        return [logs, fl]

    def refill(i):
        up = .1*i
        return [G("heart", heart(0, .26 + up, .8)),
                G("moon", bar((-.3, -.6 + up*.5), (0, -.36 + up*.5), .12, role="h"), bar((0, -.36 + up*.5), (.3, -.6 + up*.5), .12, role="h"),
                  bar((-.22, -.94 + up), (0, -.76 + up), .1, role="h"), bar((0, -.76 + up), (.22, -.94 + up), .1, role="h"),
                  spark4((.78, .8), .1 + .06*i, .04 + .02*i, role="h"))]

    def cursor(i):
        dy = (0, -.08, -.14, -.08)[i]
        return [G("gold", P([(-.8, .84 + dy), (.8, .84 + dy), (0, -.96 + dy)], bev=.62), P([(-.36, .58 + dy), (.36, .58 + dy), (0, -.26 + dy)], "i", bev=.6), P(rect(0, 1.0 + dy, 1.7, .16), "h"))]

    def a_button(i):
        top = ((-.1, .16), (-.2, .0))[i]
        zf = (top[1] - .02, top[1] + .18)
        return [G("green", P(ngon(0, 0, .98, 12, 15), z=(-.24, .0), bev=.86), P(ngon(0, 0, .8 if i == 0 else .76, 12, 15), z=top, bev=.8)),
                G("moon", bar((-.3, -.42), (0, .46), .16, role="h", z=zf), bar((.3, -.42), (0, .46), .16, role="h", z=zf), bar((-.2, -.14), (.2, -.14), .13, role="h", z=zf))]

    def burn(i):
        lean = (-.25, -.08, .12, -.1)[i]; tall = (1.0, 1.06, .96, 1.04)[i]
        return [G("orange", P(flame(0, -.04, .64, 1.0*tall if tall <= 1.02 else 1.02, lean)), P(flame(.04, -.4, .28, .48*(2 - tall), -lean*.4), "i"))]

    def coin(i):
        wd = (1.0, .7, .3, .08, .3, .7)[i]
        pts = [(x*wd, y) for x, y in ngon(0, 0, .9, 12, 15)]
        return [G("gold", P(pts, bev=.84), P(rect(0, 0, max(.03, .16*wd), .62), "i"))]

    def spark(i):
        k = (1.0, .8, .56, .8)[i]
        return [G("volt", P([(0, 1.1*k), (.24*k, 0), (0, -1.1*k), (-.24*k, 0)], "h", bev=.5), P([(.8*k, 0), (0, .2*k), (-.8*k, 0), (0, -.2*k)], bev=.5))]

    return {"node_campfire": (4, campfire), "w_campfire": (4, lambda i: campfire(i, True)), "fx_refill": (4, refill), "map_cursor": (4, cursor),
            "hud_a_button": (2, a_button), "st_burn": (4, burn), "hud_coin": (6, coin), "fx_spark": (4, spark)}
