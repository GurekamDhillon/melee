"""Scenario builders for the shard-497-504 state oracle. A scenario file is a list of
`begin <func> <seed>` ... `end` blocks (see rt.c for the ops). Each builder returns the text."""
import random

import model
from model import T


class World:
    """Bump allocator over the harness arena (ARENA_SIZE 512 KiB) with typed objects."""

    def __init__(self, tu, seed, func):
        self.tu = tu
        self.top = 0x1000
        self.ops = []
        self.objs = []  # (off, typename, count)
        self.seed = seed
        self.func = func
        self.args = []
        self.dumps = []

    def size(self, tname):
        return model.find_type(self.tu, tname).get_canonical().get_size()

    def alloc(self, tname, count=1, dump=True):
        sz = self.size(tname) * count
        off = (self.top + 15) & ~15
        self.top = off + sz
        assert self.top < 0x7F000, (tname, self.top)
        if dump:
            self.objs.append((off, tname, count))
        if self.has_bf(tname):
            self.ops.append(f"bf a:{off:x} {tname} {count:x} {self.seed * 0x10001 + off:x}")
        return off

    _bf = {}

    def has_bf(self, tname):
        if tname not in World._bf:
            import gen as G
            World._bf[tname] = bool(G.bf_leaves(G.leaves(model.find_type(self.tu, tname))))
        return World._bf[tname]

    def raw(self, nbytes, align=16):
        off = (self.top + align - 1) & ~(align - 1)
        self.top = off + nbytes
        assert self.top < 0x7F000
        return off

    def foff(self, tname, path):
        """(byte offset, size) of a possibly nested field `a.b[2].c` of a record type."""
        t = model.find_type(self.tu, tname).get_canonical()
        off = 0
        for part in path.split("."):
            idx = None
            if "[" in part:
                part, i = part[:-1].split("[")
                idx = int(i)
            f = next(c for c in t.get_fields() if c.spelling == part)
            off += t.get_offset(part) // 8
            t = f.type.get_canonical()
            if idx is not None:
                el = t.element_type.get_canonical()
                off += idx * el.get_size()
                t = el
        return off, t.get_size()

    def w(self, off, n, val):
        self.ops.append(f"W {off:x} {n:x} {val & ((1 << (8 * n)) - 1):x}")

    def wf(self, obj, tname, path, val):
        o, s = self.foff(tname, path)
        self.w(obj + o, s, val)

    def link(self, off, target):
        self.ops.append(f"P {off:x} {0xFFFF if target is None else target:x}")

    def linkf(self, obj, tname, path, target):
        o, s = self.foff(tname, path)
        self.link(obj + o, target)

    def fptr(self, off, name):
        self.ops.append(f"PF {off:x} {name}")

    def gset(self, g, target):
        self.ops.append(f"GSET {g} {0xFFFF if target is None else target:x}")

    def gw(self, g, off, n, val):
        self.ops.append(f"GW {g} {off:x} {n:x} {val & ((1 << (8 * n)) - 1):x}")

    def ret(self, callee, *vals):
        self.ops.append("RET " + callee + " " + " ".join(vals))

    def call(self, *args):
        self.ops.append("call " + " ".join(args))

    def text(self, post=()):
        L = [f"begin {self.func} {self.seed:x}"]
        L += self.ops
        for o, tn, c in self.objs:
            L.append(f"dump a:{o:x} {tn} {c:x}")
        L += list(post)
        L.append("end")
        return "\n".join(L)


def a(off):
    return f"a:{off:x}"


def s(v):
    return f"s:{v & 0xFFFFFFFFFFFFFFFF:x}"


def f32(x):
    import struct
    return f"s:{struct.unpack('<I', struct.pack('<f', x))[0]:x}"


def build_vsmode(stem, cfg, oc, tu):
    """The 13 handlers/hooks of a VS-variant mode TU. They call into other TUs with
    (scene, &gmMainLib->modes.table[mode], ...): the trace pins the table index, the extra
    arguments and which callback each Enter* passes."""
    out = []
    funcs = oc["funcs"]
    states = oc["globals"][-1][0]
    nstates = oc.get("nstates", 9)
    rng = random.Random(0x497 + len(stem))
    for k, fn in enumerate(funcs):
        for rep in range(3):
            w = World(tu, 0x100 * (k + 1) + rep, fn)
            gmm = w.alloc("gmm_x0", 1, dump=False)
            w.gset("gmMainLib_804D3EE0", gmm)
            scene = w.alloc("GameModeState", 1)
            if fn in ("fn_801B8E50", "fn_8801B8C5C", "fn_801B8C5C"):
                data = w.alloc("PlayerInitData")
                w.call(a(data), "n" if rep == 0 else a(w.alloc("PlayerInitData", 1, dump=False)))
            elif fn in ("fn_801BA1B4", "fn_801BA5B4"):
                data = w.alloc("StartMeleeData")
                w.call(a(data), "n" if rep == 0 else a(w.alloc("StartMeleeData", 1, dump=False)))
            elif fn.endswith("_OnInit") or fn.endswith("_OnLoad"):
                w.call()
            else:
                w.call(a(scene))
            out.append(w.text())
    # the state table itself: every entry's ids, flags, handler pointers and scene data
    out.append(f"begin __table_{states} 1\ndump g:{states} GameModeState {nstates:x}\nend")
    return "\n".join(out) + "\n"


def _vs_world(tu, seed, fn, mode_idx=None):
    w = World(tu, seed, fn)
    gmm = w.alloc("gmm_x0", 1, dump=False)
    w.gset("gmMainLib_804D3EE0", gmm)
    o, _ = w.foff("gmm_x0", "modes.table[0]")
    w.ops.append(f"bf a:{gmm + o:x} VsModeData f {seed * 0x10001 + gmm:x}")
    return w, gmm


def _table_off(w, gmm, idx):
    """Arena offset of gmMainLib->modes.table[idx]."""
    o, _ = w.foff("gmm_x0", "modes.table[%d]" % idx)
    return gmm + o


def build_stamina(stem, cfg, oc, tu):
    out = []
    rng = random.Random(0x5A31)
    ST = 9  # GmVsMode_Stamina
    # ---- CSS enter / ExitCss / EnterSss: css data seeded from the mode's VS data
    for rep in range(6):
        w, gmm = _vs_world(tu, 0x1000 + rep, "gm_801B91C8")
        state = w.alloc("GameModeState")
        css = w.alloc("CSSData")
        w.ret("gm_GetGameModeStateEnterData", a(css))
        w.call(a(state))
        w.ops.append("gdump gm_804975F8")
        out.append(w.text())
    for fn in ("gm_801B922C", "gm_801B9254"):
        for rep in range(2):
            w, gmm = _vs_world(tu, 0x2000 + rep, fn)
            state = w.alloc("GameModeState")
            w.call(a(state))
            out.append(w.text())
    # ---- SSS exit: start_game 0 / 1 / other
    for rep, sg in enumerate([0, 1, 2, 0, 1, 255]):
        w, gmm = _vs_world(tu, 0x3000 + rep, "gm_801B927C")
        state = w.alloc("GameModeState")
        sss = w.alloc("SSSData")
        w.wf(sss, "SSSData", "start_game", sg)
        w.ret("gm_GetGameModeStateExitData", a(sss))
        w.call(a(state))
        # the VsModeData the handler overwrites lives in the arena: dump the whole mode table entry
        w.dumps = []
        o = _table_off(w, gmm, ST)
        out.append(w.text(post=[f"dump a:{o:x} VsModeData 1"]))
    # ---- VS enter
    for rep in range(6):
        w, gmm = _vs_world(tu, 0x4000 + rep, "gm_801B931C")
        state = w.alloc("GameModeState")
        start = w.alloc("StartMeleeData")
        w.ret("gm_GetGameModeStateEnterData", a(start))
        w.call(a(state))
        o = _table_off(w, gmm, ST)
        out.append(w.text(post=[f"dump a:{o:x} VsModeData 1"]))
    for rep in range(2):
        w, gmm = _vs_world(tu, 0x5000 + rep, "gm_801B9560")
        state = w.alloc("GameModeState")
        w.call(a(state))
        out.append(w.text())
    for fn in ("gm_Mode_StaminaVs_OnInit", "gm_Mode_StaminaVs_OnLoad"):
        w, gmm = _vs_world(tu, 0x6000, fn)
        w.call()
        out.append(w.text())
    # ---- gm_801B9600: players/teams remaining
    for rep in range(40):
        w, gmm = _vs_world(tu, 0x7000 + rep, "gm_801B9600")
        o = _table_off(w, gmm, ST)
        rules, _ = w.foff("VsModeData", "start.rules.is_teams")
        w.w(o + rules, 1, rng.choice([0, 1, 1, 2]))
        for i in range(4):
            w.gw("gm_804975F8", 4 + 4 * i, 4, rng.choice([0, 0, 1, 1, 2]))
        w.ret("Player_GetPlayerSlotType", *[s(rng.choice([0, 1, 2, 3, 3, 4])) for _ in range(rng.choice([4, 8, 12]))])
        w.ret("Player_GetStocks", *[s(rng.choice([0, 0, 1, 3])) for _ in range(4)])
        w.ret("Player_GetTeam", *[s(rng.choice([0, 1, 2])) for _ in range(rng.choice([4, 7, 11]))])
        w.call()
        w.ops.append("gdump gm_804975F8")
        out.append(w.text())
    # ---- fn_801B96E8: the elimination proc
    for rep in range(40):
        w, gmm = _vs_world(tu, 0x8000 + rep, "fn_801B96E8")
        o = _table_off(w, gmm, ST)
        rules, _ = w.foff("VsModeData", "start.rules.is_teams")
        w.w(o + rules, 1, rng.choice([0, 1]))
        w.gw("gm_804975F8", 0, 2, rng.choice([0, 1, 50, 99, 100, 101, 0xFFFF]))
        for i in range(4):
            w.gw("gm_804975F8", 4 + 4 * i, 4, rng.choice([0, 1]))
            w.gw("gm_804975F8", 20 + 2 * i, 2, rng.choice([0, 0, 1, 50, 99, 100, 101]))
        w.ret("Player_GetPlayerSlotType", *[s(rng.choice([0, 1, 3])) for _ in range(rng.choice([4, 8]))])
        w.ret("Player_GetStocks", *[s(rng.choice([0, 1])) for _ in range(4)])
        w.ret("Player_GetTeam", *[s(rng.choice([0, 1, 2])) for _ in range(5)])
        g = w.alloc("HSD_GObj", 1, dump=False)
        w.call(a(g))
        w.ops.append("gdump gm_804975F8")
        out.append(w.text())
    # ---- gm_801B97C4(slot, cond)
    for rep in range(40):
        w, gmm = _vs_world(tu, 0x9000 + rep, "gm_801B97C4")
        for i in range(4):
            w.gw("gm_804975F8", 4 + 4 * i, 4, rng.choice([0, 1]))
            w.gw("gm_804975F8", 20 + 2 * i, 2, rng.choice([0, 1, 100]))
        w.call(s(rng.choice([0, 1, 2, 3, 4, 5, 6])), s(rng.choice([0, 0, 1, 2])))
        w.ops.append("gdump gm_804975F8")
        out.append(w.text())
    # ---- fn_801B9850: match start
    for rep in range(12):
        w, gmm = _vs_world(tu, 0xA000 + rep, "fn_801B9850")
        w.ret("Player_GetPlayerSlotType", *[s(rng.choice([0, 1, 3])) for _ in range(4)])
        g = w.alloc("HSD_GObj", 1, dump=False)
        w.ret("GObj_Create", a(g))
        w.call()
        w.ops.append("gdump gm_804975F8")
        out.append(w.text())
    out.append(f"begin __table_gm_Mode_StaminaVs_States 1\ndump g:gm_Mode_StaminaVs_States GameModeState 4\nend")
    return "\n".join(out) + "\n"


def build_scdata(stem, cfg, oc, tu):
    out = []
    for fn, ret in (("gm_GetAllGameScenes", 0), ("gm_GetAllGameModes", 0), ("gm_GetNumGameModes", 0), ("gm_GetNumGameScenes", 0)):
        w = World(tu, 0x100, fn)
        w.call()
        out.append(w.text())
    out.append("begin __tables 1\ndump g:scenes GameScene 2e\ndump g:modes GameMode 30\nend")
    return "\n".join(out) + "\n"


from scen_scene import build_scene  # noqa: E402,F401
from scen_staff import build_staff  # noqa: E402,F401
