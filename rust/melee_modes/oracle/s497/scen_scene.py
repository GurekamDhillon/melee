"""Scenarios for gmscene.c (the scene loop and its helpers)."""
import random

import model
from scen import World, a, s, f32


def base(w, rng, db_level=None, ptrs=True):
    """Common global state: no stale pointers, a sane bus clock, bitfields set logically."""
    w.gw("__OSBusClock", 0, 4, 162000000)
    w.gw("gm_onep_interstage", 0, 4, rng.choice([0, 0, 0, 1]))
    w.gw("DbLevel", 0, 4, rng.choice([0, 1, 2, 3, 4, 5]) if db_level is None else db_level)
    for p in ("unk_10.pre_gobj_proc", "unk_10.db_input.check_pause", "unk_10.db_input.check_framestep"):
        o, _ = w.foff("gm_80479D58_t", p)
        w.gw("gm_80479D58", o, 8, 0)
    o, _ = w.foff("gm_80479D58_t", "unk_10.unk_34")
    w.gw("gm_80479D58", o, 4, rng.choice([0, 1]))
    w.ops.append(f"bf g:gm_80479D58 gm_80479D58_t 1 {w.seed * 77 + 5:x}")
    if ptrs:
        for g in ("gm_804D672C", "gm_804D6728", "gm_804D6724", "gm_804D6720", "mnLoadScreen_panel", "mnLoadScreen_text",
                  "mnOverlay_panel", "mnOverlay_caption", "mnOverlay_toast"):
            w.gset(g, None)
    seedp = w.raw(8)
    w.w(seedp, 4, rng.getrandbits(32))
    w.gset("HSD_RandSeedPtr", seedp)
    return seedp


def pad_set(w, i, _g="HSD_PadMasterStatus", **kw):
    sz = w.size("HSD_PadStatus")
    for k, v in kw.items():
        o, n = w.foff("HSD_PadStatus", k)
        w.gw(_g, i * sz + o, n, v)


def string(w, text):
    off = w.raw(len(text) + 1, 1)
    for i, ch in enumerate(text.encode()):
        w.w(off + i, 1, ch)
    w.w(off + len(text), 1, 0)
    return off


def rl(rng, vals, lo=1, hi=6, last=None):
    n = rng.randint(lo, hi)
    out = [rng.choice(vals) for _ in range(n)]
    if last is not None:
        out.append(last)
    return [s(v) for v in out]


POST_STATE = ["dump g:gm_80479D58 gm_80479D58_t 1", "dump g:gmMainLib_8046B0F0 gmMainLib_8046B0F0_t 1",
              "pdump gm_804D672C", "pdump gm_804D6728", "pdump gm_804D6724", "pdump gm_804D6720"]
POST_LS = ["gdump mnLoadScreen_holding", "gdump mnLoadScreen_frames", "gdump mnLoadScreen_settled",
           "gdump mnLoadScreen_created", "gdump mnLoadScreen_warm", "gdump mnLoadScreen_started",
           "pdump mnLoadScreen_panel", "pdump mnLoadScreen_text"]
POST_OV = ["pdump mnOverlay_panel", "pdump mnOverlay_caption", "pdump mnOverlay_toast"]


def build_scene(stem, cfg, oc, tu):
    out = []
    rng = random.Random(0x5CE4)
    seed = [0x100]

    def blk(fn):
        seed[0] += 1
        w = World(tu, seed[0], fn)
        base(w, rng)
        return w

    # ---- flag accessors
    for fn in ("gm_GetDbPauseFlag", "gm_ClearDbPauseFlag", "gm_SetDbPauseFlag", "gm_801A46B8"):
        for bit in [0, 1, 2, 3, 7, 8, 31, 32, 63, 64, 65, -1, 5, 0, 4, 6]:
            w = blk(fn)
            w.call(s(bit))
            w.text()
            out.append(w.text(post=POST_STATE))
    for fn in ("gm_801A4624", "ProfBenchFrame", "gm_801A4BA8", "gm_801A4BB8", "gm_801A4BC8", "gm_801A4B60", "gm_801A4B74", "gm_801A4B1C"):
        for rep in range(3):
            w = blk(fn)
            if fn == "gm_801A4BC8":
                w.gset("gm_804D672C", w.alloc("HSD_GObj", 1, dump=False))
            w.call()
            out.append(w.text(post=POST_STATE))
    # ---- pad predicates
    for fn in ("fn_801A46F4", "fn_801A47E4"):
        for rep in range(24):
            w = blk(fn)
            for i in range(4):
                pad_set(w, i, err=rng.choice([0, 0, 0, 1, -1 & 0xFF]),
                        trigger=rng.choice([0, 8, 16, 0x18, 2, 0x400, 0x18, 0xFFFFFFFF, 0x1000]),
                        button=rng.choice([0, 0x400, 0x400, 0x10, 0xFFFFFFFF, 0x300]))
            w.call()
            out.append(w.text())
    # ---- gm_801A48A4
    for v in [0, 1, 2, 3, 4, 5, 6, 7, 8, 0x0F, 0x55, 0xAA, 0x80, 0xFF, 0x7F, 0xF0, 0x81, 0x24, 0x49, 0x92]:
        w = blk("gm_801A48A4")
        w.call(s(v))
        out.append(w.text())
    # ---- handlers
    for rep in range(6):
        w = blk("gm_SetDbPauseInputHandlers")
        pick = lambda: rng.choice(["n", "f:tcb_pred_a", "f:tcb_pred_b", "f:fn_801A46F4", "f:fn_801A47E4"])
        w.call(pick(), pick())
        out.append(w.text(post=POST_STATE))
    for rep in range(4):
        w = blk("gm_SetPreGObjProcCallback")
        w.call(rng.choice(["n", "f:tcb_pre"]))
        out.append(w.text(post=POST_STATE))
    for v in (0, 1, -1, 5):
        w = blk("gm_801A4B50")
        w.call(s(v))
        out.append(w.text(post=POST_STATE))
    # ---- scene info
    for kind in (2, 3, 4, 8, 0xFF, None, None):
        w = blk("gm_801A4B88")
        info = None if kind is None else w.alloc("GameSceneInfo")
        if info is not None:
            w.wf(info, "GameSceneInfo", "scene_kind", kind)
        w.call("n" if info is None else a(info))
        out.append(w.text(post=POST_STATE))
    for fn in ("gm_GetCurrentSceneEnterData", "gm_GetCurrentSceneExitData"):
        for rep in range(3):
            w = blk(fn)
            info = w.alloc("GameSceneInfo")
            w.gset("gm_804D6720", info)
            w.call()
            out.append(w.text(post=POST_STATE))
    for rep in range(2):
        w = blk("fn_801A4BD0")
        w.call(a(w.alloc("HSD_GObj", 1, dump=False)))
        out.append(w.text())
    # ---- gm_801A4970 (debug dump + pause toggles)
    o_db, _ = (lambda ww: ww.foff("gm_80479D58_t", "unk_10.db_input"))(World(tu, 0, "x"))
    o_cp, _ = World(tu, 0, "x").foff("gm_80479D58_t", "unk_10.db_input.check_pause")
    o_cf, _ = World(tu, 0, "x").foff("gm_80479D58_t", "unk_10.db_input.check_framestep")
    o_x0, _ = World(tu, 0, "x").foff("gm_80479D58_t", "unk_10.x0")
    for rep in range(40):
        w = blk("gm_801A4970")
        for i in range(4):
            pad_set(w, i, trigger=rng.choice([0, 2, 2, 8]), button=rng.choice([0, 0x400, 0x400, 0x10]))
        if rng.random() < 0.7:
            w.ops.append(f"GPF gm_80479D58 {o_cp:x} tcb_pred_a")
        if rng.random() < 0.7:
            w.ops.append(f"GPF gm_80479D58 {o_cf:x} tcb_pred_b")
        w.gw("gm_80479D58", o_x0, 1, rng.choice([0, 1, 0xFE, 0xFF, 4]))
        if rng.random() < 0.5:
            w.gset("gm_804D6728", w.alloc("HSD_GObj", 1, dump=False))
        if rng.random() < 0.5:
            w.ops.append("GPF gm_80479D58 0 tcb_hook" if False else "")
        if rng.random() < 0.5:
            w.ops.append("GPF gm_804D6724 0 tcb_hook")
        w.ret("tcb_pred_a", *rl(rng, [0, 1, 2], 1, 3))
        w.ret("tcb_pred_b", *rl(rng, [0, 1, 2], 1, 3))
        w.ret("gm_80165388", a(w.alloc("HSD_GObj", 1, dump=False)), "n")
        w.call(f"g:gm_80479D58+{o_db:x}")
        out.append(w.text(post=POST_STATE))
    # ---- gm_801A4BD4 (scene setup)
    for rep in range(10):
        w = blk("gm_801A4BD4")
        g = w.alloc("HSD_GObj", 1, dump=False)
        w.ret("GObj_Create", rng.choice([a(g), "n"]))
        w.ret("HSD_GObj_803912A8", s(rng.randrange(256)))
        w.call()
        out.append(w.text(post=POST_STATE + ["dump g:gobj_init_data HSD_GObjLibInitDataType 1", "gdump HSD_SObjLib_804D7960"]))
    # ---- gm_FindGameSceneHandler
    for rep in range(14):
        w = blk("gm_FindGameSceneHandler")
        n = rng.randint(1, 6)
        tab = w.alloc("GameScene", n + 1)
        sz = w.size("GameScene")
        kinds = [rng.randrange(0, 45) for _ in range(n)] + [45]
        for i, k in enumerate(kinds):
            w.wf(tab + i * sz, "GameScene", "kind", k)
        w.ret("gm_GetAllGameScenes", a(tab))
        want = rng.choice(kinds[:-1] + [44, 1, 200])
        w.call(s(want))
        out.append(w.text())
    # ---- loading screen
    for rep in range(30):
        w = blk("mnLoadScreen_Begin")
        info = None if rep % 7 == 0 else w.alloc("GameSceneInfo")
        if info is not None:
            w.wf(info, "GameSceneInfo", "scene_kind", rng.choice([2, 3, 4, 2, 4, 5, 0]))
        w.ret("Gfx_LoadScreenEnabled", *rl(rng, [0, 1], 1, 2))
        w.ret("Script_LaunchPending", *rl(rng, [0, 1], 1, 3))
        w.ret("gmFrontend_TakeWarmed", *rl(rng, [0, 1], 1, 2))
        w.ret("Gfx_PipelinesCreated", s(rng.randrange(0, 200)))
        w.ret("OSGetTime", s(rng.randrange(0, 1 << 40)))
        w.call("n" if info is None else a(info))
        out.append(w.text(post=POST_LS))
    for rep in range(40):
        w = blk("mnLoadScreen_Frame")
        warm = rng.choice([0, 1])
        w.gw("mnLoadScreen_holding", 0, 4, rng.choice([0, 1, 1, 1]))
        w.gw("mnLoadScreen_frames", 0, 4, rng.choice([0, 1, 5, 19, 20, 21, 100]))
        w.gw("mnLoadScreen_settled", 0, 4, rng.choice([0, 7, 8, 29, 30, 31]))
        w.gw("mnLoadScreen_warm", 0, 4, warm)
        t0 = rng.randrange(0, 1 << 40)
        w.gw("mnLoadScreen_started", 0, 8, t0)
        w.gw("mnLoadScreen_created", 0, 4, rng.randrange(0, 100))
        t = None if rng.random() < 0.5 else w.alloc("DevText")
        w.gset("mnLoadScreen_text", t)
        w.gset("mnLoadScreen_panel", None if rng.random() < 0.5 else w.alloc("DevText"))
        w.ret("Gfx_PipelinesCreated", s(rng.randrange(0, 200)))
        w.ret("Gfx_PipelinesUrgent", *rl(rng, [0, 0, 1], 1, 2))
        w.ret("Script_LaunchPending", *rl(rng, [0, 0, 1], 1, 4))
        w.ret("Gfx_SeedCorePending", *rl(rng, [0, 0, 1], 1, 3))
        w.ret("Gfx_SeedCoreCount", s(rng.randrange(0, 50)))
        w.ret("OSGetTime", s(t0 + rng.choice([0, 1000, 405000000, 405000001, 500000000, 10**9])), s(rng.randrange(0, 1 << 41)))
        w.ret("gm_8016AEEC", s(rng.randrange(0, 600)))
        w.ret("gm_8016AF0C", s(rng.randrange(0, 100)))
        w.call()
        out.append(w.text(post=POST_LS))
    for rep in range(12):
        w = blk("mnLoadScreen_Release")
        t0 = rng.randrange(0, 1 << 40)
        w.gw("mnLoadScreen_holding", 0, 4, rng.choice([0, 1, 1]))
        w.gw("mnLoadScreen_frames", 0, 4, rng.randrange(0, 300))
        w.gw("mnLoadScreen_started", 0, 8, t0)
        w.gset("mnLoadScreen_text", None if rng.random() < 0.5 else w.alloc("DevText"))
        w.gset("mnLoadScreen_panel", None if rng.random() < 0.5 else w.alloc("DevText"))
        w.ret("OSGetTime", s(t0 + rng.randrange(0, 10**9)))
        w.ret("gm_8016AEEC", s(rng.randrange(0, 600)))
        w.ret("gm_8016AF0C", s(rng.randrange(0, 100)))
        why = string(w, rng.choice(["warm", "ceiling", "scene ended"]))
        w.call(a(why))
        out.append(w.text(post=POST_LS))
    # ---- overlay
    for rep in range(12):
        w = blk("mnOverlay_Make")
        t = w.alloc("DevText")
        gobj = w.alloc("HSD_GObj", 1, dump=False)
        buf = w.raw(64)
        w.ret("DevText_Create", rng.choice([a(t), "n"]))
        w.call(a(gobj), s(rng.choice([0x4C, 0x4D, 0x4E])), s(rng.randrange(0, 200)), s(rng.randrange(0, 100)),
               s(rng.randrange(1, 60)), s(rng.randrange(1, 30)), a(buf), s(rng.getrandbits(32)))
        out.append(w.text())
    for rep in range(10):
        w = blk("mnOverlay_Begin")
        g = w.alloc("HSD_GObj", 1, dump=False)
        ts = [w.alloc("DevText", 1, dump=False) for _ in range(3)]
        w.ret("DevText_GetGObj", rng.choice([a(g), "n"]))
        w.ret("DevText_Create", *[rng.choice([a(t), "n"]) for t in ts])
        w.call()
        out.append(w.text(post=POST_OV))
    for rep in range(30):
        w = blk("mnOverlay_Line")
        t = None if rep % 6 == 0 else w.alloc("DevText")
        text = rng.choice([None, "", "x", "RUN 12", "a" * 47, "b" * 48, "c" * 60])
        sp = None if text is None else string(w, text)
        w.call("n" if t is None else a(t), "n" if sp is None else a(sp))
        out.append(w.text())
    for rep in range(30):
        w = blk("mnOverlay_Reshow")
        g = w.alloc("HSD_GObj", 1, dump=False)
        for gname in ("mnOverlay_panel", "mnOverlay_caption", "mnOverlay_toast"):
            w.gset(gname, None if rng.random() < 0.3 else w.alloc("DevText", 1, dump=False))
        w.ret("Overlay_GetReshowCount", *rl(rng, [0, 1, 2, 3], 1, 3))
        w.ret("DevText_GetGObj", rng.choice([a(g), "n"]))
        w.ret("DevText_IsListed", *rl(rng, [0, 1], 1, 4))
        w.call()
        w.call()
        out.append(w.text(post=POST_OV))
    for rep in range(30):
        w = blk("mnOverlay_Frame")
        g = w.alloc("HSD_GObj", 1, dump=False)
        for gname in ("mnOverlay_panel", "mnOverlay_caption", "mnOverlay_toast"):
            w.gset(gname, None if rng.random() < 0.25 else w.alloc("DevText"))
        lines = [string(w, "line%d" % i) for i in range(4)]
        w.ret("Overlay_GetReshowCount", *rl(rng, [0, 0, 1], 1, 2))
        w.ret("DevText_GetGObj", rng.choice([a(g), "n"]))
        w.ret("DevText_IsListed", *rl(rng, [0, 1], 1, 4))
        w.ret("Overlay_GetPanelOpen", *rl(rng, [0, 1], 1, 3))
        w.ret("Overlay_GetRunLabel", rng.choice([a(lines[0]), "n"]))
        w.ret("Overlay_GetToast", rng.choice([a(lines[1]), "n"]))
        w.ret("Overlay_GetPanelLineCount", s(rng.choice([0, 1, 3, 4, 20, 25])))
        w.ret("Overlay_GetPanelLine", *[rng.choice([a(x), "n"]) for x in lines])
        w.ret("Overlay_GetPanelLineDim", *rl(rng, [0, 1], 1, 4))
        w.call()
        out.append(w.text(post=POST_OV))
    # ---- the scene loop
    nf = [
        "Det_Enabled", "RB_Enabled", "Netplay_Enabled", "Netplay_PeerLeft", "Replay_Enabled", "Snap_Curated",
        "Snap_Resimulating", "Snap_CuratedNoRender", "Turbo_FrameHidden", "Script_LaunchPending", "Gfx_LoadScreenEnabled",
        "Gfx_PipelinesUrgent", "Gfx_SeedCorePending", "lb_80019A30", "gmFrontend_TakeWarmed", "Overlay_GetPanelOpen",
        "DevText_IsListed",
    ]
    cnt = ["RB_Iterations", "SyncTest_Iterations", "Script_Iterations", "RenderTest_Iterations"]
    for rep in range(60):
        fn = "gm_801A4D34" if rep % 10 != 9 else "gm_OnePInterstage"
        w = blk(fn)
        info = None if rep % 11 == 5 else w.alloc("GameSceneInfo")
        if info is not None:
            w.wf(info, "GameSceneInfo", "scene_kind", rng.choice([2, 2, 3, 4, 8, 1]))
        g = w.alloc("HSD_GObj", 1, dump=False)
        texts = [w.alloc("DevText", 1, dump=False) for _ in range(3)]
        lines = [string(w, "label"), string(w, "toast")]
        bias = rng.choice([0.2, 0.5, 0.8])
        for name in nf:
            w.ret(name, *[s(1 if rng.random() < bias else 0) for _ in range(rng.randint(1, 5))])
        for name in cnt:
            w.ret(name, *rl(rng, [0, 1, 1, 2, 3], 1, 5))
        w.ret("lb_80019894", *rl(rng, [0, 1, 2, 3], 1, 4, last=rng.choice([1, 2])))
        w.ret("HSD_PadGetResetSwitch", *([s(0)] * rng.randint(0, 4) + [s(1)]))
        w.ret("tcb_frame_end", *[s(rng.choice([0, 0, 1])) for _ in range(rng.randint(2, 4))], s(1))
        w.ret("tcb_frame2_end", *[s(rng.choice([0, 0, 1])) for _ in range(rng.randint(2, 4))], s(1))
        w.ret("tcb_pred_a", *rl(rng, [0, 1], 1, 3))
        w.ret("tcb_pred_b", *rl(rng, [0, 1], 1, 3))
        w.ret("DevText_GetGObj", rng.choice([a(g), "n"]))
        w.ret("DevText_Create", *[rng.choice([a(t), "n"]) for t in texts])
        w.ret("Overlay_GetRunLabel", a(lines[0]))
        w.ret("Overlay_GetToast", rng.choice([a(lines[1]), "n"]))
        w.ret("Overlay_GetPanelLineCount", s(0))
        w.ret("Overlay_GetReshowCount", s(0))
        w.ret("OSGetTime", s(1000), s(2000), s(3000 + rng.choice([0, 500000000])), s(4000))
        w.ret("Replay_ResyncSeed", s(rng.choice([0, 0x1234])))
        w.ret("Snap_SuppressDraws", *rl(rng, [0, 1], 1, 2))
        w.ret("gm_ChangeGameModeAfterCurrentScene", s(0))
        for pad in range(4):
            pad_set(w, pad, trigger=rng.choice([0, 2, 8, 16]), button=rng.choice([0, 0x400]))
        w.gw("gmMainLib_8046B0F0", 0, 4, rng.choice([0, 1]))
        o_ld, _ = w.foff("gm_80479D58_t", "unk_10.pre_gobj_proc")
        if rng.random() < 0.5:
            w.ops.append(f"GPF gm_80479D58 {o_ld:x} tcb_pre")
        # state the loop reads: frame counters and the pause bytes
        for fld, nbytes in (("unk_0", 4), ("unk_4", 4), ("unk_8", 4)):
            o, _ = w.foff("gm_80479D58_t", fld)
            w.gw("gm_80479D58", o, nbytes, rng.choice([0, 1, 5, 0xFFFFFFFD, 0xFFFFFFFE, 0xFFFFFFFF]))
        if fn == "gm_801A4D34":
            w.call(rng.choice(["f:tcb_frame", "f:tcb_frame", "f:tcb_frame2", "n"]), "n" if info is None else a(info))
        else:
            w.call()
        out.append(w.text(post=POST_STATE + POST_LS + POST_OV + ["gdump gm_onep_interstage"]))
    return "\n".join(out) + "\n"
