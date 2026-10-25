"""video.c spec (see lbspecs.py): the XFB state machine over HSD_VIData."""
from lbspecs import add

POOLS = [("HSD_VIStatus", "VIS", 2), ("GXRenderModeObj", "RMO", 2)]
OWN = ["HSD_VIData", "garbage"]
# the XFB state machine only ends its waits when the retrace changes a status
EFFECT_C = "HSD_VIData.xfb[fv() % 3].status = (fv() & 1) ? HSD_VI_XFB_FREE : HSD_VI_XFB_DRAWING;"
EFFECT_R = "m::HSD_VIData.xfb[(fv() % 3) as usize].status = if fv() & 1 != 0 { HSD_VI_XFB_FREE } else { HSD_VI_XFB_DRAWING };"
C_DECL = "static unsigned char XB[1 << 20];"
RS_EXTRA = "static mut XB: [u8; 1 << 20] = [0; 1 << 20];"
SETUP_C = r'''{ int q; for (q = 0; q < 2; q++) if (fv() & 3) { VIS[q].rmode.fbWidth %= 200; VIS[q].rmode.efbHeight %= 100; RMO[q].fbWidth %= 200; RMO[q].efbHeight %= 100; } for (q = 0; q < 3; q++) { HSD_VIData.xfb[q].status = (HSD_VIXFBDrawDispStatus) (fv() % 9); HSD_VIData.xfb[q].buffer = (fv() & 3) ? (void*) (BUF + 8 * q) : 0; }
 HSD_VIData.efb.status = (fv() % 3); HSD_VIData.nb_xfb = (fv() & 3) ? 1 + fv() % 3 : 0; HSD_VIData.perf.frame_period = fv() % 5; HSD_VIData.drawdone.waiting = fv() & 1; }'''
SETUP_R = r'''{ for q in 0..2usize { if fv() & 3 != 0 { VIS[q].rmode.fbWidth %= 200; VIS[q].rmode.efbHeight %= 100; RMO[q].fbWidth %= 200; RMO[q].efbHeight %= 100; } }
 for q in 0..3usize { m::HSD_VIData.xfb[q].status = (fv() % 9) as _; m::HSD_VIData.xfb[q].buffer = if fv() & 3 != 0 { (&raw mut BUF as *mut u8).add(8 * q) as *mut c_void } else { null_mut() }; }
 m::HSD_VIData.efb.status = (fv() % 3) as _; m::HSD_VIData.nb_xfb = if fv() & 3 != 0 { (1 + fv() % 3) as i32 } else { 0 }; m::HSD_VIData.perf.frame_period = (fv() % 5) as i32; m::HSD_VIData.drawdone.waiting = (fv() & 1) as i32; }'''
IDX = ("(s32) (fv() % 3)", "(fv() % 3) as i32")
add("video", "src/sysdolphin/baselib/video.c", dict(
    pub_only=True, pools=POOLS, ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA, pregs=("preg(XB, sizeof XB);", "preg(&raw const XB, 1 << 20);"), setup=(SETUP_C, SETUP_R),
    stmt={n: ("ret = FI((const void*) {call});", "ret = fi({call});") for n in ("HSD_VISetUserPreRetraceCallback", "HSD_VISetUserPostRetraceCallback", "HSD_VISetUserGXDrawDoneCallback")},
    ov={"HSD_VICopyEFB2XFBPtr": {1: ("(void*) (XB + 4096)", "(&raw mut XB as *mut u8).add(4096) as *mut c_void"), 2: ("(HSD_RenderPass) (fv() % 5)", "(fv() % 5) as _")},
        "HSD_VICopyXFBAsync": {0: ("(HSD_RenderPass) (fv() % 5)", "(fv() % 5) as _")},
        "HSD_VISetXFBWaitDone": {0: IDX}, "HSD_VIDrawDoneXFB": {0: IDX}, "HSD_VIGetXFBPtr": {0: IDX},
        "HSD_VIInit": {1: ("(void*) BUF", "(&raw mut BUF) as *mut c_void"),
                       2: ("(fv() & 1) ? (void*) (BUF + 8) : 0", "if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(8) as *mut c_void } else { null_mut() }"),
                       3: ("(fv() & 1) ? (void*) (BUF + 16) : 0", "if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(16) as *mut c_void } else { null_mut() }")},
        "HSD_VISearchXFBByStatus": {0: ("(HSD_VIXFBDrawDispStatus) (fv() % 9)", "(fv() % 9) as _")},
        "HSD_VISetBlack": {0: ("(fv() & 1)", "(fv() & 1) as i32")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_hash=OWN, skip_stubs=["memcpy", "memset"],
    own_preg=[],
    sub=[(r"\(void\*\) \(\(u32\) buffer \+ offset\)", "(void*) ((unsigned long long) buffer + offset)")],
    effects={"VIWaitForRetrace": (EFFECT_C, EFFECT_R)},
    ret={"VIGetTvFormat": ("(fv() & 1)", "fv() & 1")})
