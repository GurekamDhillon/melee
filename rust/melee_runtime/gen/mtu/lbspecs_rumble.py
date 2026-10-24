"""rumble.c spec (see lbspecs.py)."""
import os
from lbspecs import add

OWN = ["HSD_Rumble_804C22E0", "HSD_Rumble_80406DE0"]
POOLS = [("HSD_PadRumbleListData", "RL", 8), ("HSD_Rumble", "CM", 40)]
CH = ("(u8) (fv() % 4)", "(fv() % 4) as u8")

_SETUP_C = ("{ int q, ch, cnt[4]; HSD_PadRumbleListData* fr = 0; HSD_PadRumbleListData* tail[4]; "
            "for (q = 0; q < 40; q++) { CM[q].command.op = (q >= 39) ? 0 : (q % 4 == 1) ? 4 : (q % 4 == 3) ? 5 : 1 + fv() % 3; CM[q].command.frame = 1 + fv() % 6; } "
            "for (ch = 0; ch < 4; ch++) { cnt[ch] = 0; tail[ch] = 0; HSD_Rumble_804C22E0[ch].listdatap = 0; } "
            "for (q = 0; q < 8; q++) { u32 f; RL[q].next = 0; RL[q].listp = &CM[fv() % 30]; RL[q].headp = RL[q].listp; RL[q].stack = &CM[fv() % 30]; "
            "RL[q].pause = (u8) ((fv() & 3) == 0); RL[q].id = fv() % 6; RL[q].pri = (u8) (fv() % 5); RL[q].loop_count = (u16) (fv() % 5); RL[q].wait = (u16) (fv() % 3); "
            "f = fv() % 5; RL[q].frame = f == 0 ? -1 : f == 1 ? -2 : (s32) (1 + fv() % 6); "
            "ch = (int) (fv() % 5); if (ch == 4) { RL[q].next = fr; fr = &RL[q]; } else { if (tail[ch]) tail[ch]->next = &RL[q]; else HSD_Rumble_804C22E0[ch].listdatap = &RL[q]; tail[ch] = &RL[q]; cnt[ch]++; } } "
            "for (ch = 0; ch < 4; ch++) HSD_Rumble_804C22E0[ch].nb_list = (u16) cnt[ch]; "
            "HSD_PadLibData.rumble_info.listdatap = fr; HSD_PadLibData.rumble_info.max_list = (u16) (1 + fv() % 8); HSD_PadLibData.rumble_info.unk2 = (u8) ((fv() & 3) == 0); }")
_SETUP_R = ("{ let mut cnt = [0u32; 4]; let mut fr: *mut HSD_PadRumbleListData = null_mut(); let mut tail: [*mut HSD_PadRumbleListData; 4] = [null_mut(); 4]; "
            "m::ext::use_own_libdata = true; "
            "for q in 0..40usize { CM[q].command.set_op(if q >= 39 { 0 } else if q % 4 == 1 { 4 } else if q % 4 == 3 { 5 } else { 1 + fv() % 3 } as _); CM[q].command.set_frame((1 + fv() % 6) as _); } "
            "for ch in 0..4usize { cnt[ch] = 0; tail[ch] = null_mut(); m::HSD_Rumble_804C22E0[ch].listdatap = null_mut(); } "
            "for q in 0..8usize { RL[q].next = null_mut(); RL[q].listp = &raw mut CM[(fv() % 30) as usize]; RL[q].headp = RL[q].listp; RL[q].stack = &raw mut CM[(fv() % 30) as usize]; "
            "RL[q].pause = (fv() & 3 == 0) as u8; RL[q].id = fv() % 6; RL[q].pri = (fv() % 5) as u8; RL[q].loop_count = (fv() % 5) as u16; RL[q].wait = (fv() % 3) as u16; "
            "let f = fv() % 5; RL[q].frame = if f == 0 { -1 } else if f == 1 { -2 } else { (1 + fv() % 6) as i32 }; "
            "let ch = (fv() % 5) as usize; if ch == 4 { RL[q].next = fr; fr = &raw mut RL[q]; } else { if !tail[ch].is_null() { (*tail[ch]).next = &raw mut RL[q]; } else { m::HSD_Rumble_804C22E0[ch].listdatap = &raw mut RL[q]; } tail[ch] = &raw mut RL[q]; cnt[ch] += 1; } } "
            "for ch in 0..4usize { m::HSD_Rumble_804C22E0[ch].nb_list = cnt[ch] as u16; } "
            "m::ext::HSD_PadLibData.rumble_info.listdatap = fr; m::ext::HSD_PadLibData.rumble_info.max_list = (1 + fv() % 8) as u16; m::ext::HSD_PadLibData.rumble_info.unk2 = (fv() & 3 == 0) as u8; }")

# MTU_RUMBLE_DBG=1 compiles a spin guard into HSD_PadRumbleInterpret1 (debugging an infinite-loop state)
_DBG = []
if os.environ.get("MTU_RUMBLE_DBG"):
    _DBG = [("    if (a->pause == 1) {\n", "    int sp = 0;\n    if (a->pause == 1) {\n"),
            ("    while (a->wait == 0) {\n", "    while (a->wait == 0) {\n"
             "        { extern int printf(const char*, ...); extern void exit(int); if (++sp > 3000000) { int z; "
             "printf(\"SPIN lp=%ld st=%ld lc=%d fr=%d ops:\", (long)(a->listp - a->headp), (long)(a->stack - a->headp), a->loop_count, a->frame); "
             "for (z = 0; z < 14; z++) printf(\" %d:%d\", a->headp[z].command.op, a->headp[z].command.frame); printf(\"\\n\"); exit(3); } }\n")]

add("rumble", "src/sysdolphin/baselib/rumble.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, skip=["HSD_PadRumbleFree", "func_80378430_inline"],
    ov={"HSD_PadRumbleOn": {0: CH}, "HSD_PadRumbleOffN": {0: CH}, "HSD_PadRumbleRemove": {0: CH}, "HSD_PadRumbleRemoveId": {0: CH, 1: ("(int) (fv() % 6)", "(fv() % 6) as i32")},
        "HSD_PadRumblePause": {0: CH, 1: ("(int) (fv() % 3)", "(fv() % 3) as i32")},
        "HSD_PadRumbleAdd": {0: CH, 1: ("(int) (fv() % 6)", "(fv() % 6) as i32"), 3: ("(int) (fv() % 5)", "(fv() % 5) as i32"),
                             4: ("(void*) &CM[fv() % 30]", "(&raw mut CM[(fv() % 30) as usize]) as *mut c_void")},
        "HSD_PadRumbleInit": {0: ("(u16) (fv() % 9)", "(fv() % 9) as u16"), 1: ("(void*) RL", "(&raw mut RL) as *mut c_void")}}),
    pub_only=True, skip_stubs=["exit"] if _DBG else [], patch=_DBG, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=OWN, own_hash=OWN)
