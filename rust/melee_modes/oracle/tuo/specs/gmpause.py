TU = "src/melee/gm/gmpause.c"
RUST_MOD = "melee_modes::tu::gmpause"
RUST_TYPES = {"PauseData": "melee_modes::tu::gmpause::PauseData"}
CASES = {
    "fn_801A0E34": dict(n=300),
    "gm_801A0FEC": dict(n=300),
    "gm_801A10FC": dict(n=50),
    "fn_801A1134": dict(n=50),
}
DOM = {}
VARIADIC = {
    "lb_80011E24": dict(ret="i", args=[("p", "*mut melee_types::HSD_JObj"), ("p", "*mut *mut melee_types::HSD_JObj"), ("i", "i32"), ("i", "i32")]),
    "lbArchive_80016DBC": dict(ret="p", ret_rust="*mut melee_types::HSD_Archive", link="buf:64",
                               args=[("s", "*const i8"), ("p", "*mut core::ffi::c_void"), ("s", "*const i8"), ("i", "i32")]),
}
OUTS = {"lbArchive_80016DBC": [1], "lb_80011E24": [1]}
LINKS = {
    "out:lbArchive_80016DBC#1": "new:SceneDesc",
    "out:lb_80011E24#1": "new:HSD_JObj",
    ".models": "ptrs:DynamicModelDesc:1",
    "lbl_80479B10.background": "new:HSD_JObj",
    "lbl_80479B10.analog_stick": "new:HSD_JObj",
    "lbl_80479B10.lras": "new:HSD_JObj",
    "lbl_80479B10.z": "new:HSD_JObj",
    "lbl_80479B10.analog_stick_outline": "new:HSD_JObj",
}
# JObj flags without JOBJ_USE_QUATERNION (the inlined setters assert on it): all combinations of
# MTX_DIRTY 0x40, USER_DEF_MTX 0x800000, MTX_INDEP_SRT 0x2000000, HIDDEN 0x10
_f = [a | b | c | d for a in (0, 0x40) for b in (0, 0x800000) for c in (0, 0x2000000) for d in (0, 0x10)]
DOM = {
    ".flags": _f,
    "lbl_80479B10.slot": [99, 0, 1, 2, 3],
    "arg:gm_801A0FEC#0": [-1, 0, 1, 2, 3, 99, 255],
    "arg:gm_801A0FEC#1": [0, 1, 2, 3, 4, 5, 6, 7, 0xFF],
}
SKIP_LEAF_TYPES = ["HSD_IDKey"]   # u32 in melee_types, uintptr_t natively (group-0 mismatch)
SIZE_OK = ["HSD_JObj"]
