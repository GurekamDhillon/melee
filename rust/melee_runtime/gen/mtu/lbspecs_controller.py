"""controller.c spec (see lbspecs.py)."""
from lbspecs import add

OWN = ["HSD_PadLibData", "HSD_PadMasterStatus", "HSD_PadCopyStatus", "HSD_PadGameStatus"]
POOLS = [("HSD_PadData", "PQ", 16), ("HSD_PadRumbleListData", "RL", 2)]

_PAD_C = ("{ int q; for (q = 0; q < 4; q++) { PADStatus* s = &a0[q]; s->button = (u16) fv(); s->stickX = (s8) fv(); s->stickY = (s8) fv(); "
          "s->substickX = (s8) fv(); s->substickY = (s8) fv(); s->triggerLeft = (u8) fv(); s->triggerRight = (u8) fv(); s->analogA = (u8) fv(); "
          "s->analogB = (u8) fv(); { u32 e = fv() % 9; s->err = e < 5 ? 0 : e == 5 ? -1 : e == 6 ? -3 : e == 7 ? -2 : 1; } s->extButton = 0; } }")
_PAD_R = ("{ for q in 0..4usize { let s = &mut *(a0.add(q)); s.button = fv() as u16; s.stickX = fv() as i8; s.stickY = fv() as i8; "
          "s.substickX = fv() as i8; s.substickY = fv() as i8; s.triggerLeft = fv() as u8; s.triggerRight = fv() as u8; s.analogA = fv() as u8; "
          "s.analogB = fv() as u8; { let e = fv() % 9; s.err = if e < 5 { 0 } else if e == 5 { -1 } else if e == 6 { -3 } else if e == 7 { -2 } else { 1 }; } s.extButton = 0; } }")
_LAB_C = _PAD_C.replace("&a0[q]", "&((PADStatus*) a0)[q]")
_LAB_R = _PAD_R.replace("a0.add(q)", "(a0 as *mut PADStatus).add(q)")

_SETUP_C = ("{ HSD_PadLibData.qnum = (u8) (1 + fv() % 4); HSD_PadLibData.qread = (u8) (fv() % HSD_PadLibData.qnum); "
            "HSD_PadLibData.qwrite = (u8) (fv() % HSD_PadLibData.qnum); HSD_PadLibData.qcount = (u8) (fv() % (HSD_PadLibData.qnum + 1)); "
            "HSD_PadLibData.qtype = (u8) (fv() % 4); HSD_PadLibData.queue = PQ; "
            "HSD_PadLibData.clamp_stickType = (u8) ((fv() & 3) == 0 ? 1 : 0); HSD_PadLibData.adc_type = (s8) ((fv() & 3) == 0 ? 1 : 0); "
            "HSD_PadLibData.adc_th = (s8) (fv() % 50); HSD_PadLibData.adc_angle = rw(); HSD_PadLibData.cross_dir = (u8) (fv() % 5); "
            "HSD_PadLibData.clamp_stickMax = (s8) (30 + fv() % 98); HSD_PadLibData.clamp_stickMin = (s8) (fv() % 30); "
            "HSD_PadLibData.clamp_stickShift = (u8) (fv() % 3); HSD_PadLibData.repeat_start = (s32) (1 + fv() % 40); HSD_PadLibData.repeat_interval = (s32) (1 + fv() % 20); "
            "HSD_PadLibData.scale_stick = (s8) (fv() % 128); HSD_PadLibData.reset_switch_status = (u8) (fv() & 1); HSD_PadLibData.reset_switch = (u8) (fv() & 1); }")
_SETUP_R = ("{ m::HSD_PadLibData.qnum = (1 + fv() % 4) as u8; m::HSD_PadLibData.qread = (fv() % m::HSD_PadLibData.qnum as u32) as u8; "
            "m::HSD_PadLibData.qwrite = (fv() % m::HSD_PadLibData.qnum as u32) as u8; m::HSD_PadLibData.qcount = (fv() % (m::HSD_PadLibData.qnum as u32 + 1)) as u8; "
            "m::HSD_PadLibData.qtype = (fv() % 4) as u8; m::HSD_PadLibData.queue = (&raw mut PQ) as *mut HSD_PadData; "
            "m::HSD_PadLibData.clamp_stickType = (if fv() & 3 == 0 { 1 } else { 0 }) as u8; m::HSD_PadLibData.adc_type = (if fv() & 3 == 0 { 1 } else { 0 }) as i8; "
            "m::HSD_PadLibData.adc_th = (fv() % 50) as i8; m::HSD_PadLibData.adc_angle = rw(); m::HSD_PadLibData.cross_dir = (fv() % 5) as u8; "
            "m::HSD_PadLibData.clamp_stickMax = (30 + fv() % 98) as i8; m::HSD_PadLibData.clamp_stickMin = (fv() % 30) as i8; "
            "m::HSD_PadLibData.clamp_stickShift = (fv() % 3) as u8; m::HSD_PadLibData.repeat_start = (1 + fv() % 40) as i32; m::HSD_PadLibData.repeat_interval = (1 + fv() % 20) as i32; "
            "m::HSD_PadLibData.scale_stick = (fv() % 128) as i8; m::HSD_PadLibData.reset_switch_status = (fv() & 1) as u8; m::HSD_PadLibData.reset_switch = (fv() & 1) as u8; }")

add("controller", "src/sysdolphin/baselib/controller.c", dict(pub_only=True, pools=POOLS, ov={"HSD_PadRenewRawStatus": {0: ("(fv() & 1)", "fv() & 1 != 0")}}, setup=(_SETUP_C, _SETUP_R), ncases=400),
    pub_only=True, skip_stubs=["memcpy", "memset", "__frsqrte", "fabs"], types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=OWN, own_hash=OWN,
    skip_data=["default_status_data", "default_libinfo_data", "pad_bit"],
    effects={"PADRead": (_PAD_C, _PAD_R), "LabPad_Replay": (_LAB_C, _LAB_R)},
    ret={"RB_PadGoverned": ("(int) (fv() % 3 == 0)", "(fv() % 3 == 0) as i32"), "LabPad_Replay": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "Replay_ControlsRaw": ("(int) (fv() & 1)", "(fv() & 1) as i32"), "Mex_Enabled": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "atan2f": ("rw()", "rw()")})
