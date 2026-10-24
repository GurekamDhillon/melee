import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/gr/grzakogenerator.c"
RUST_MOD = "melee_world::tu::grzakogenerator"
LOWMEM = True  # u32 views alias pointer slots (Item_ItemVars/Ground unions): pool objects below 4 GB, same addresses on both sides
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {}
RUST_GLOBAL_TYPES = {"lbl_8049F030": "melee_world::tu::grzakogenerator::lbl_8049F030_t"}


def F(x):
    return "f:%08x" % struct.unpack("<I", struct.pack("<f", x))[0]


CASES = {
    "grZakoGenerator_StageSlotRelease": dict(n=20),
    "grZakoGenerator_801CA394": dict(n=60, args=["p:grZakoGenerator_Spawn", None, "fn:zg_cb", None]),
    "grZakoGenerator_801CA43C": dict(n=400, args=["p:grZakoGenerator_Config", "p:HSD_JObj", None]),
    "grZakoGenerator_801CA67C": dict(n=4),
    "grZakoGenerator_801CA8B4": dict(n=800),
    "grZakoGenerator_801CAC14": dict(n=300, args=["p:HSD_GObj"]),
    "grZakoGenerator_801CACB8": dict(n=300, args=["p:HSD_GObj"]),
    "fn_801CADBC": dict(n=100, args=["p:HSD_GObj"]),
    "grZakoGenerator_801CADE0": dict(n=100),
    "grZakoGenerator_801CAE04": dict(n=40, args=["null"]),
    "grZakoGenerator_801CAEB0": dict(n=100),
    "grZakoGenerator_801CAEF0": dict(n=8),
    "grZakoGenerator_801CAF08": dict(n=200),
}
FMT_VARIADIC = ["OSReport"]
EXTERN_ARRAY_LEN = {}
OUT_F32S = {"lb_8000B1CC": {2: 3}, "Ground_801C2D24": {1: 3}}
_pos = [F(v) for v in (0.0, 50.0, -50.0, 150.0, -150.0, 250.0, -250.0, 90.0, -90.0)]
DOM = {
    "g:lbl_8049F030.x8": [0, 1, 40, 80, 79],
    "g:lbl_8049F030.xA_b0": [0, 1, 1],
    "ret:itGetKind": [159, 159, 43, 44, 45, 46, 47, 48, 208, 211, 212, 213, 217, 231, 232, 0, 5],
    "ret:Ground_801C2D24": [0, 1, 1, 1],
    "ret:Stage_GetBlastZoneRightOffset": [F(200.0), F(100.0)],
    "ret:Stage_GetBlastZoneLeftOffset": [F(-200.0), F(-100.0)],
    "ret:Stage_GetBlastZoneTopOffset": [F(200.0), F(100.0)],
    "ret:Stage_GetBlastZoneBottomOffset": [F(-200.0), F(-100.0)],
    "ret:Stage_GetCamBoundsRightOffset": [F(150.0), F(50.0), F(100.0)],
    "ret:Stage_GetCamBoundsLeftOffset": [F(-150.0), F(-50.0), F(-100.0)],
    "ret:Stage_GetCamBoundsTopOffset": [F(150.0), F(50.0), F(100.0)],
    "ret:Stage_GetCamBoundsBottomOffset": [F(-150.0), F(-50.0), F(-100.0)],
    "out:Ground_801C2D24#1[0]": _pos,
    "out:Ground_801C2D24#1[1]": _pos,
    "out:Ground_801C2D24#1[2]": _pos,
    "ret:HSD_Randf": [F(0.0), F(0.25), F(0.5), F(0.75), F(1.0)],
    "ret:it_8026C1E8": [0, 1],
    # entries of the spawn table
    ".x0": [-1, -1, 0x20, 0x21, 0x30, 0x4F, 0x6F, 0x70, 0xDC, 0xE0, 0xFB, 0xFC],
    ".x2": [-1, -1, 0, 0, 1, 2, 0x708],
    ".x8": [-1, 0, 1, 2, 77],
    ".kind": [43, 44, 45, 46, 211, 212, 213, 217, 0, 1, 99],
    ".respawn": [0, 1, 1, 2],
    ".idx": [-1, -1, 0, 1, 5, 40, 79, 80],
    ".count": [0, 1, 2, 3, 5, 8],
    "arg:grZakoGenerator_801CA394#1": [0, 1, 3, 4, 6, 8],
    "arg:grZakoGenerator_801CA43C#2": [F(0.0), F(10.0), F(-10.0), F(25.0), F(-40.0), F(100.0)],
    ".x10": [F(-1.0), F(-0.5), F(0.0), F(0.25), F(0.5), F(0.9), F(1.0)],
    "arg:grZakoGenerator_801CA8B4#0": [0, 1],
    "arg:grZakoGenerator_801CAEB0#0": [-1, 0, 5, 77],
    "arg:grZakoGenerator_801CAEB0#1": [-1, 0, 5, 77, 0x1234],
}
LINKS = {
    "g:lbl_8049F030.x4": "maybe:grZakoGenerator_Data",
    "g:lbl_8049F030.x0": "maybe:grZakoGenerator_SpawnDesc:80",
    "g:HSD_GObjPLinkHead": "ptrs:HSD_GObj:16",
    ".spawn_descs": "new:grZakoGenerator_Spawn:8",
    ".spawns": "new:grZakoGenerator_Spawn:8",
    ".callback": "fn:zg_cb",
    ".gen": "maybe:HSD_Generator",
    ".user_data": "new:Item",
    ".x4": "mayarg:0:HSD_GObj",
    ".next": "maybe:HSD_GObj",
    "ret:zg_cb": "maybe:HSD_Generator",
    "ret:Ground_801C58E0": "maybe:HSD_GObj",
    "ret:it_8027B5B0": "maybe:HSD_GObj",
    "ret:it_802EA9FC": "maybe:HSD_GObj",
    "ret:it_802DD7F0": "maybe:HSD_GObj",
    "ret:it_802E16F8": "maybe:HSD_GObj",
    "ret:it_802DC4BC": "maybe:HSD_GObj",
    "ret:GObj_Create": "maybe:HSD_GObj",
}
# the item/player-link pointers need an item behind them where the TU dereferences
LINK_FORCE = {
    "g:lbl_8049F030.x4": ["grZakoGenerator_801CAC14", "grZakoGenerator_801CACB8", "grZakoGenerator_801CAEB0"],
    "g:lbl_8049F030.x0": ["grZakoGenerator_801CACB8"],
}
PRUNE = {"Item": ["xDD4_itemVar"], "HSD_Generator": ["pos"]}
UNION_VIEW = {"Item_ItemVars": "zako"}
EXTRA_FNS = {
    "zg_cb": dict(
        c='void* zg_cb(void* a0, int a1) { tuo_call_begin("zg_cb"); tuo_arg_bytes(a0, 12); tuo_arg_i(a1); tuo_call_end(); return tuo_ret_ptr("zg_cb"); }',
        rust='unsafe extern "C" fn s_zg_cb(a0: *mut melee_types::Vec3, a1: i32) -> *mut melee_types::HSD_Generator { let v = vv(); v.call_begin("zg_cb"); v.arg_bytes(a0 as *const u8, 12); v.arg_i(a1 as i64); v.call_end(); v.ret_ptr("zg_cb") as _ }',
    )
}
EXTRA_GLOBALS = ["lbl_8049F030"]

TYPES = ["grZakoGenerator_Config", "grZakoGenerator_Data", "grZakoGenerator_Spawn"]
# HSD_MemAlloc hands out typed objects so the bytes the port writes (pointers included) compare by value:
# 56 = sizeof(grZakoGenerator_Config), 1944 = sizeof(grZakoGenerator_Data), n * 24 = n spawns
CUSTOM_STUBS = {
    "HSD_MemAlloc": dict(
        c=("extern unsigned long long tuo_draw(void); extern void* tuo_new(const char*, const char*, int); extern void* tuo_newbuf(const char*, unsigned long long); "
           'void* HSD_MemAlloc(int n) { tuo_call_begin("HSD_MemAlloc"); tuo_arg_i(n); tuo_call_end(); '
           'if (n == 1944) return tuo_new(0, "grZakoGenerator_Data", 0); '
           'if (n == 56 || (n > 0 && n % 24 == 0)) { if ((tuo_draw() & 7) == 0) return 0; '
           'return n == 56 ? tuo_new(0, "grZakoGenerator_Config", 0) : tuo_new(0, "grZakoGenerator_Spawn", n / 24); } '
           'return tuo_newbuf(0, n > 0 ? n : 1); }'),
        rust=('unsafe extern "C" fn s_HSD_MemAlloc(a0: i32) -> *mut core::ffi::c_void { let v = vv(); v.call_begin("HSD_MemAlloc"); v.arg_i(a0 as i64); v.call_end(); '
              'if a0 == 1944 { return v.new_typed(None, "grZakoGenerator_Data", 0) as _; } '
              'if a0 == 56 || (a0 > 0 && a0 % 24 == 0) { if v.draw() & 7 == 0 { return core::ptr::null_mut(); } '
              'return if a0 == 56 { v.new_typed(None, "grZakoGenerator_Config", 0) as _ } else { v.new_typed(None, "grZakoGenerator_Spawn", (a0 / 24) as usize) as _ }; } '
              'v.new_buf(None, if a0 > 0 { a0 as usize } else { 1 }) as _ }'),
        ext='    hook!(H_HSD_MemAlloc, HSD_MemAlloc, (a0: i32) -> *mut core::ffi::c_void);',
    ),
}
