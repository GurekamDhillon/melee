"""Scenarios for gmstaffroll.c."""
import random
import struct

import model
from scen import World, a, s, f32
from scen_scene import pad_set, rl


def fb(x):
    return struct.unpack("<I", struct.pack("<f", x))[0]


def wfl(w, off, x):
    w.w(off, 4, fb(x))


def mtx(w, off, rng, z=None, ident=True):
    """Write a sane 3x4 matrix at off: near-identity with some scale and translation; [2][3] = z."""
    for r in range(3):
        for c in range(4):
            v = 1.0 if (r == c and ident) else 0.0
            if r == c:
                v = rng.choice([1.0, 1.0, 0.5, 2.0, -1.0])
            elif c < 3:
                v = rng.choice([0.0, 0.0, 0.25, -0.25])
            else:
                v = rng.uniform(-3, 3)
            wfl(w, off + 4 * (4 * r + c), v)
    if z is not None:
        wfl(w, off + 4 * (4 * 2 + 3), z)


class Staff:
    """Arena objects shared by the staff-roll scenarios."""

    def __init__(self, w, rng):
        self.w = w
        self.rng = rng
        W = w
        self.jo = lambda o, p: W.foff("HSD_JObj", p)[0] + o

    def jobj(self, **kw):
        w = self.w
        j = w.alloc("HSD_JObj", 1, dump=False)
        for k, v in kw.items():
            if k in ("child", "next", "parent"):
                w.linkf(j, "HSD_JObj", k, v)
            elif k == "flags":
                w.wf(j, "HSD_JObj", "flags", v)
        return j

    def text(self, entity=None):
        w = self.w
        t = w.alloc("HSD_Text", 1, dump=False)
        w.linkf(t, "HSD_Text", "entity", entity)
        return t


STATIC_INTS = ["gm_804D6800", "gm_804D6810", "gm_804D6814", "gm_804D6818", "gm_804D681C", "gm_804D6820", "gm_804D6848",
               "gm_804D684C"]
POST_STATE = (["gdump " + n for n in STATIC_INTS] + ["gdump gm_804D6824", "gdump gm_804D6828", "gdump gm_804D6804",
              "gdump gm_803DD1E4", "pdump staffInfo", "pdump staffInfoSortBuf", "pdump gm_804D680C", "pdump gm_804D682C",
              "pdump gm_804D6830", "pdump gm_804D6834", "pdump gm_804D6838", "pdump gm_804D683C", "pdump gm_804D6840",
              "pdump gm_804D6844", "padump gm_80480D58"])


def base(w, rng, **kw):
    """Plain statics: counters at stage values, floats sane, pointers NULL."""
    ints = dict(gm_804D6800=0, gm_804D6810=rng.choice([0, 1, 2, 3]), gm_804D6814=rng.choice([0, 100, 0x1000, 0x1284, 0x1285, 0x12D5, 0x1311, 0x1400]),
                gm_804D6818=rng.choice([0, 0, 1]), gm_804D681C=rng.choice([1, 1, 2, 6, 0]), gm_804D6820=rng.choice([0, 0, 1, 0x3C]),
                gm_804D6848=rng.choice([0, 1]), gm_804D684C=rng.randrange(0, 44))
    ints.update(kw)
    for k, v in ints.items():
        w.gw(k, 0, 4, v)
    wfl_g(w, "gm_804D6824", rng.choice([0.0, 0.5, 1.0, 2.5]))
    wfl_g(w, "gm_804D6828", rng.choice([0.0, 0.01, 0.5, 1.0, 3.0]))
    wfl_g(w, "gm_804D6804", rng.uniform(-1, 1), 0)
    wfl_g(w, "gm_804D6804", rng.uniform(-1, 1), 4)
    for g in ("staffInfo", "staffInfoSortBuf", "gm_804D680C", "gm_804D682C", "gm_804D6830", "gm_804D6834", "gm_804D6838",
              "gm_804D683C", "gm_804D6840", "gm_804D6844"):
        w.gset(g, None)
    for i in range(6):
        w.ops.append(f"GP gm_80480D58 {8 * i:x} ffff")


def wfl_g(w, g, x, off=0):
    w.gw(g, off, 4, fb(x))


def setup_tables(w, rng, st, n, entry_idx=None, mtx_z=None):
    """staffInfo / staffInfoSortBuf with n sorted entries, every entry's windows set."""
    info = w.raw(198 * 24)
    sort = w.raw(198 * 72)
    w.gset("staffInfo", info)
    w.gset("staffInfoSortBuf", sort)
    ta = st.text(entity=w.alloc("HSD_GObj", 1, dump=False))
    tb = st.text(entity=w.alloc("HSD_GObj", 1, dump=False))
    w.ops.append(f"PW {info:x} 18 c6 {ta:x}")
    w.ops.append(f"PW {info + 8:x} 18 c6 {tb:x}")
    w.gw("gm_804D6800", 0, 4, n)
    idxs = []
    for i in range(n):
        idx = rng.choice([0, 1, 2, 6, 7, 8, 0x30, 0x4B, 0xAC, 0xAF, 0xB6, 0xB7, 0xBF, 0xC0, 0xC4, 0xC5]) if entry_idx is None else entry_idx[i]
        idxs.append(idx)
        eo = sort + 72 * i
        w.w(eo, 4, idx)
        j = st.jobj(flags=0)
        mtx(w, j + w.foff("HSD_JObj", "mtx")[0], rng)
        w.link(eo + 8, j)
        mtx(w, eo + 16, rng, z=(rng.choice([-5.0, -2.0, -1.0, -0.5, 0.0, -9.0]) if mtx_z is None else mtx_z))
        w.w(info + 24 * idx + 16, 4, rng.choice([-1, 0, 0, 1, 2]))
    return info, sort, ta, tb, idxs


def pool(w, rng, st):
    """Four universal JObjs (what lb_80011E24 hands out): u.ptcl -> particle chain, child -> a JObj."""
    # chain M -x4-> M ... -x8-> L -xC-> W; W (written with the colour) is never walked through x4
    W = [w.raw(32) for _ in range(3)]
    L = [w.raw(32) for _ in range(2)]
    M = [w.raw(32) for _ in range(3)]
    for nd in W:
        for k in range(4):
            w.link(nd + 8 * k, None if k == 1 else rng.choice(W))  # x4 holds a 32-bit colour: keep its upper half clear
    for nd in L:
        for k in range(4):
            w.link(nd + 8 * k, rng.choice(L))
        w.link(nd + 24, rng.choice(W))
    for nd in M:
        w.link(nd, rng.choice(M))
        w.link(nd + 8, rng.choice(M))
        w.link(nd + 16, rng.choice(L))
        w.link(nd + 24, rng.choice(L))
    nodes = W + L + M
    objs = []
    for _ in range(4):
        c = st.jobj(flags=0)
        j = st.jobj(flags=rng.choice([0, 0x40, 0x40, 0x2000000]), child=c)
        w.link(j + w.foff("HSD_JObj", "u")[0], rng.choice(M))
        mtx(w, j + w.foff("HSD_JObj", "mtx")[0], rng)
        objs.append(j)
    w.ret("lb_80011E24.out", *[a(j) for j in objs])
    return nodes, objs


def build_staff(stem, cfg, oc, tu):
    out = []
    rng = random.Random(0x57AF)
    seed = [0x100]

    def blk(fn):
        seed[0] += 1
        w = World(tu, seed[0], fn)
        base(w, rng)
        return w, Staff(w, rng)

    # tables (static data)
    out.append("begin __tables 1\ndump g:gm_803DBFD8 StaffEntryData c6\ndump g:gm_803DBFD8_extra StaffRollExtraData 1\n"
               "dump g:gm_803DD0C8 HSD_WObjDesc 1\ndump g:gm_803DD0DC HSD_WObjDesc 1\ndump g:gm_803DD0F0 HSD_CameraDescPerspective 1\n"
               "gdump gm_804D42B8\ngdump gm_804D42BC\ngdump gm_804D42C0\ngdump gm_804D42C4\ngdump gm_804D42C8\ngdump gm_804D42CC\n"
               "gdump gm_803DD1C8\ngdump gm_803DD1E4\ngdump gm_804DAAEC\nend")
    # ---- unlock predicates
    for fn, stubs in (("gm_801AA644", ["gmMainLib_8015EDD4"]), ("gm_801AA664", ["gm_IsCKindUnlocked"]),
                      ("gm_801AA688", ["gm_IsCKindUnlocked"]), ("gm_801AA6D8", ["Toy_803048C0"]),
                      ("gm_801AA6FC", ["Toy_803048C0"]), ("gm_801AA774", ["Toy_803048C0"])):
        for rep in range(10):
            w, st = blk(fn)
            for sname in stubs:
                w.ret(sname, *rl(rng, [0, 1, 2], 1, 3))
            w.call(s(rng.choice([0, 1, 15, 269, -1, 0x7FFF, 0x8000, 272])))
            out.append(w.text())
    for v in (0, 0x134C, 0x134D, 0x134E, 0x7FFFFFFF, -5):
        w, st = blk("gm_Scene_StaffRoll_OnFrame")
        w.gw("gm_804D6814", 0, 4, v)
        w.call()
        out.append(w.text(post=POST_STATE))
    # ---- simple procs over gobj->hsd_obj
    for fn, ty in (("fn_801AA7F8", "HSD_CObj"), ("fn_801AAABC", "HSD_LObj"), ("fn_801AAB18", "HSD_JObj"), ("fn_801AC67C", "HSD_Fog")):
        for rep in range(8):
            w, st = blk(fn)
            g = w.alloc("HSD_GObj", 1, dump=False)
            o = w.alloc(ty, 1, dump=False)
            w.linkf(g, "HSD_GObj", "hsd_obj", o)
            w.call(a(g))
            out.append(w.text(post=POST_STATE[:3]))
    out.append(build_render(tu, rng, blk))
    out.append(build_sortbuild(tu, rng, blk))
    out.append(build_driver(tu, rng, blk))
    out.append(build_enter(tu, rng, blk))
    w, st = blk("gm_Scene_StaffRoll_OnExit")
    w.call("n")
    out.append(w.text())
    return "\n".join(out) + "\n"


def build_render(tu, rng, blk):
    out = []
    # fn_801AA854: cursor camera render callback
    for rep in range(30):
        w, st = blk("fn_801AA854")
        n = rng.choice([0, 1, 2, 4])
        info, sort, ta, tb, idxs = setup_tables(w, rng, st, n, entry_idx=[rng.randrange(0, 198) for _ in range(n)])
        cobj = w.alloc("HSD_CObj", 1, dump=False)
        mtx(w, cobj + w.foff("HSD_CObj", "view_mtx")[0], rng)
        w.gset("gm_804D6830", cobj)
        fog = w.alloc("HSD_Fog", 1, dump=False)
        w.gset("gm_804D6838", fog)
        gobj = w.alloc("HSD_GObj", 1, dump=False)
        w.linkf(gobj, "HSD_GObj", "hsd_obj", cobj)
        chain = [w.alloc("HSD_GObj", 1, dump=False) for _ in range(rng.choice([0, 1, 1]))]  # the C asserts gobj == head[9]: one node at most
        for i, c in enumerate(chain):
            w.linkf(c, "HSD_GObj", "next_gx", chain[i + 1] if i + 1 < len(chain) else None)
        heads = w.raw(8 * 16)
        w.link(heads + 8 * 9, chain[0] if chain else None)
        w.gset("HSD_GObjGXLinkHead", heads)
        w.ret("HSD_CObjSetCurrent", *rl(rng, [0, 1, 1], 1, 2))
        w.call(a(gobj), s(rng.randrange(0, 10)))
        out.append(w.text(post=POST_STATE + [f"hashn {info:x} {198 * 24:x}", f"hashn {sort:x} {198 * 72:x}",
                                             f"hashn {cobj:x} {w.size('HSD_CObj'):x}"]))
    # fn_801AAA28
    for rep in range(14):
        w, st = blk("fn_801AAA28")
        cobj = w.alloc("HSD_CObj", 1, dump=False)
        gobj = w.alloc("HSD_GObj", 1, dump=False)
        w.linkf(gobj, "HSD_GObj", "hsd_obj", cobj)
        for i in range(6):
            if rng.random() < 0.5:
                w.ops.append(f"GP gm_80480D58 {8 * i:x} {st.text(entity=w.alloc('HSD_GObj', 1, dump=False)):x}")
        if rng.random() < 0.5:
            w.gset("gm_804D680C", st.text(entity=w.alloc("HSD_GObj", 1, dump=False)))
        w.ret("HSD_CObjSetCurrent", *rl(rng, [0, 1, 1], 1, 2))
        w.call(a(gobj), s(rng.randrange(0, 5)))
        out.append(w.text(post=POST_STATE))
    return "\n".join(out)


def build_sortbuild(tu, rng, blk):
    """fn_801AAB74: build/destroy the text windows of the credit entries and depth-sort the visible ones."""
    out = []
    for rep in range(36):
        w, st = blk("fn_801AAB74")
        n = rng.choice([3, 8, 30, 100, 198, 198])
        info = w.raw(198 * 24)
        sort = w.raw(198 * 72)
        w.gset("staffInfo", info)
        w.gset("staffInfoSortBuf", sort)
        # zero the table, then set a few windows / hit counts
        for idx in rng.sample(range(n), min(n, 10)):
            w.w(info + 24 * idx, 8, 0)
        cobj = w.alloc("HSD_CObj", 1, dump=False)
        mtx(w, cobj + w.foff("HSD_CObj", "view_mtx")[0], rng)
        w.gset("gm_804D6830", cobj)
        root = w.alloc("HSD_JObj", 1, dump=False)
        w.linkf(root, "HSD_JObj", "child", None)
        prev = None
        leaves = []
        heads = []
        for i in range(n):
            leaf = st.jobj(flags=rng.choice([0, 0, 0, 0x10, 0x10, 0x40, 0x10 | 0x40]))
            mtx(w, leaf + w.foff("HSD_JObj", "mtx")[0], rng, z=rng.choice([-9.0, -4.0, -1.0, 0.0, 2.0, -6.5]))
            mid = st.jobj(child=leaf)
            top = st.jobj(child=mid)
            leaves.append(leaf)
            heads.append(top)
        for i, h in enumerate(heads):
            w.linkf(h, "HSD_JObj", "next", heads[i + 1] if i + 1 < n else None)
        w.linkf(root, "HSD_JObj", "child", heads[0])
        g = w.alloc("HSD_GObj", 1, dump=False)
        w.linkf(g, "HSD_GObj", "hsd_obj", root)
        texts = [st.text() for _ in range(24)]
        w.ret("HSD_SisLib_803A5ACC", *[a(t) for t in texts])
        for name in ("lbLang_IsSavedLanguageJP", "lbLang_IsSettingJP", "gmMainLib_8015EDD4", "gm_IsCKindUnlocked", "Toy_803048C0"):
            w.ret(name, *rl(rng, [0, 1], 1, 5))
        w.call(a(g))
        post = POST_STATE + [f"hashn {info:x} {198 * 24:x}", f"hashn {sort:x} {198 * 72:x}"]
        out.append(w.text(post=post))
    return "\n".join(out)


def build_driver(tu, rng, blk):
    """fn_801AB200: the cursor / selection / animation driver."""
    out = []
    for rep in range(160):
        w, st = blk("fn_801AB200")
        root = st.jobj(flags=0)
        nodes, objs = pool(w, rng, st)
        n = rng.choice([0, 1, 2, 3, 5, 8])
        info, sort, ta, tb, idxs = setup_tables(w, rng, st, n)
        cobj = w.alloc("HSD_CObj", 1, dump=False)
        w.gset("gm_804D6830", cobj)
        g = w.alloc("HSD_GObj", 1, dump=False)
        w.linkf(g, "HSD_GObj", "hsd_obj", root)
        texts = [st.text() for _ in range(10)]
        w.ret("HSD_SisLib_803A5ACC", *[a(t) for t in texts])
        w.ret("HSD_SisLib_803A6754", a(st.text()))
        w.ret("HSD_SisLib_803A6B98", s(7))
        w.ret("gm_801BF010", *rl(rng, [0, 1, 2, 3], 1, 4))
        w.ret("HSD_CObjGetFov", *[f32(rng.choice([30.0, 50.0, 60.0, 90.0])) for _ in range(2)])
        w.ret("lbLang_IsSavedLanguageJP", *rl(rng, [0, 1], 1, 4))
        w.ret("lbLang_IsSavedLanguageUS", *rl(rng, [0, 1], 1, 6))
        w.ret("gm_IsCKindUnlocked", *rl(rng, [0, 1], 1, 3))
        w.ret("hsd_8039F05C", a(w.alloc("HSD_Generator", 1, dump=False)))
        w.ret("psAddGeneratorAppSRT_begin", a(w.alloc("HSD_psAppSRT", 1, dump=False)))
        w.ret("HSD_GObjObject_80390A3C", a(w.alloc("HSD_GObj", 1, dump=False)))  # the C asserts non-null
        # the pads
        for p in range(4):
            pad_set(w, p, _g="HSD_PadCopyStatus", stickX=rng.choice([0, 3, 5, -5, 40, -40, 80, -80, 127, -128]),
                    stickY=rng.choice([0, 3, 5, -5, 40, -40, 80, -80, 127, -128]),
                    trigger=rng.choice([0, 0, 0x100, 0x200, 0x1000, 0x10, 0x100 | 0x200, 0x300]))
        # state
        w.gw("gm_804D6804", 0, 4, fb(rng.uniform(-1, 1)))
        for i in range(6):
            if rng.random() < 0.4:
                w.ops.append(f"GP gm_80480D58 {8 * i:x} {rng.choice(texts):x}")
        if rng.random() < 0.4:
            w.gset("gm_804D680C", rng.choice(texts))
        w.gset("gm_804D682C", objs[0])
        w.call(a(g))
        post = POST_STATE + [f"hashn {info:x} {198 * 24:x}", f"hashn {sort:x} {198 * 72:x}"]
        post += [f"hashn {nd:x} 32" for nd in nodes[:3]]
        post += [f"hashn {o:x} {w.size('HSD_JObj'):x}" for o in objs]
        post += [f"dump a:{t:x} HSD_Text 1" for t in texts[:6]] + [f"dump a:{ta:x} HSD_Text 1", f"dump a:{tb:x} HSD_Text 1"]
        out.append(w.text(post=post))
    return "\n".join(out)


def build_enter(tu, rng, blk):
    out = []
    for rep in range(10):
        w, st = blk("gm_Scene_StaffRoll_OnEnter")
        # the scene description lbArchive hands out
        cam_desc = w.raw(64)
        cam_anim_arr = w.raw(16)
        cam_anim = w.raw(64)
        w.link(cam_anim_arr, cam_anim)
        cameras = w.alloc("SceneCameraDesc", 1, dump=False)
        w.linkf(cameras, "SceneCameraDesc", "desc", cam_desc)
        w.linkf(cameras, "SceneCameraDesc", "anims", cam_anim_arr)
        fog_desc = w.raw(64)
        fog_anim_arr = w.raw(16)
        fog_anim = w.alloc("HSD_CameraAnim", 1, dump=False)
        w.linkf(fog_anim, "HSD_CameraAnim", "aobjdesc", w.raw(32))
        w.link(fog_anim_arr, fog_anim)
        fogs = w.alloc("SceneFogDesc", 1, dump=False)
        w.linkf(fogs, "SceneFogDesc", "desc", fog_desc)
        w.linkf(fogs, "SceneFogDesc", "anims", fog_anim_arr)
        lights = w.raw(16)
        models = w.raw(16)
        for m in range(2):
            md = w.alloc("DynamicModelDesc", 1, dump=False)
            w.linkf(md, "DynamicModelDesc", "joint", w.raw(64))
            w.link(models + 8 * m, md)
        scene = w.alloc("SceneDesc", 1, dump=False)
        w.linkf(scene, "SceneDesc", "models", models)
        w.linkf(scene, "SceneDesc", "cameras", cameras)
        w.linkf(scene, "SceneDesc", "lights", lights)
        w.linkf(scene, "SceneDesc", "fogs", fogs)
        mdarr = w.raw(8 * 10)
        for m in range(10):
            md = w.alloc("DynamicModelDesc", 1, dump=False)
            w.linkf(md, "DynamicModelDesc", "joint", w.raw(64))
            w.link(mdarr + 8 * m, md)
        w.ret("lbArchive_80016DBC.out", a(scene), a(mdarr))
        w.ret("lbArchive_80016DBC", a(w.raw(64)))
        w.ret("HSD_MemAlloc", a(w.raw(198 * 24)), a(w.raw(198 * 72)))
        # the objects the loaders return
        cobj1, cobj2 = w.alloc("HSD_CObj", 1, dump=False), w.alloc("HSD_CObj", 1, dump=False)
        w.ret("HSD_CObjLoadDesc", a(cobj1), a(cobj2))
        w.ret("lb_80011AC4", a(w.alloc("HSD_LObj", 1, dump=False)))
        w.ret("HSD_FogLoadDesc", a(w.alloc("HSD_Fog", 1, dump=False)))
        gobjs = [w.alloc("HSD_GObj", 1, dump=False) for _ in range(8)]
        w.ret("GObj_Create", *[a(x) for x in gobjs])
        joints = []
        # model1 (US texture swap), model2 + 9 temps (each with children to reparent), final model
        m1 = st.jobj(flags=0)
        joints.append(m1)
        m2 = st.jobj(flags=0)
        joints.append(m2)
        for _ in range(9):
            c1 = st.jobj(flags=0)
            c2 = st.jobj(flags=0)
            t = st.jobj(flags=0, child=c1)
            w.linkf(c1, "HSD_JObj", "parent", t)
            w.linkf(c1, "HSD_JObj", "next", c2)
            w.linkf(c2, "HSD_JObj", "parent", t)
            w.linkf(c2, "HSD_JObj", "next", None)
            joints.append(t)
        fin = st.jobj(flags=0)
        joints.append(fin)
        w.ret("HSD_JObjLoadJoint", *[a(j) for j in joints])
        # lb_80011E24 outs: two JObjs with a dobj -> mobj -> tobj chain, then two plain ones
        outs = []
        for _ in range(2):
            tobj = w.alloc("HSD_TObj", 1, dump=False)
            mobj = w.alloc("HSD_MObj", 1, dump=False)
            dobj = w.alloc("HSD_DObj", 1, dump=False)
            w.linkf(mobj, "HSD_MObj", "tobj", tobj)
            w.linkf(dobj, "HSD_DObj", "mobj", mobj)
            j = st.jobj(flags=0)
            w.link(j + w.foff("HSD_JObj", "u")[0], dobj)
            outs.append(j)
        outs += [st.jobj(flags=0), st.jobj(flags=0)]
        w.ret("lb_80011E24.out", *[a(j) for j in outs])
        w.ret("lbLang_IsSettingUS", *rl(rng, [0, 1], 1, 2))
        w.ret("HSD_Randi", *rl(rng, [0, 1, 2, 3, 4], 1, 3))
        w.ret("gm_801BEFB0", s(rng.randrange(0, 26)))
        w.ret("lbAudioAx_8002305C", s(rng.randrange(0, 100)))
        w.call("n")
        post = POST_STATE + [f"hashn {outs[0]:x} {w.size('HSD_JObj'):x}", f"hashn {outs[1]:x} {w.size('HSD_JObj'):x}",
                             f"hashn {cobj1:x} 16", f"padump gm_80480D58", f"gdump HSD_GObj_CameraKind"]
        for g_ in gobjs[:6]:
            post.append(f"hashn {g_:x} {w.size('HSD_GObj'):x}")
        for tn in ("gm_804D6840",):
            pass
        out.append(w.text(post=post))
    return "\n".join(out)
