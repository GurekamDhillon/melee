"""Oracle patch for src/melee/gm/gmresultplayer.c (applied to a copy; see gen/tu/run.py).

`MatchPlayerData.x3_u` is a byte of bitfields (`x3_b0 : 6`, `x3_b6`, `x3_b7`). The game (PPC) allocates
bitfields from the most significant bit, the native C from the least; the oracle state is the
port's bytes (PPC order), so the copy reads the costume (`x3_b0`, the top six bits) from the byte.
"""


def patch(t):
    a = ".x3_u.x0.x3_b0"
    assert t.count(a) >= 1, a
    # every use is a call argument: `X.x3_u.x3 >> 2` needs no extra parentheses there
    t = t.replace(a, ".x3_u.x3 >> 2")
    assert ".x3_u.x0." not in t, "another x3 bitfield use: extend the patch"
    return t
