"""Native layouts for motion-var unions whose C members alias pointers by GameCube offset.

ftMasterHand_MotionVars (src/melee/ft/kinds/ftMasterHand/types.h) is used by Master Hand and Crazy Hand. The C
relies on its members sharing 4-byte slots: `unk0.x34..x40 = 0` NULLs the laser items `fingerbeam/dmg0.x34..x40`,
`unk0.x28..x30` (sound handles) are `dmg0.x28..x30`, and Crazy Hand's `unk0.x28..x34 = 0` NULLs `grab.x28..x34`.
Natively `unk0.x4` is an 8-byte HSD_GObjEvent (every later unk0 field sits 8 bytes on) and the items are 8-byte
pointers, so a straight native layout puts the item slots over unrelated unk0 ints (Master Hand's damage state then
freed a garbage pointer). Here: `dmg0.x28..x30` sit on `unk0.x28..x30`'s native offsets (0x30), and the item
pointers of fingerbeam / dmg0 / grab move past the end of unk0 (0x88), where nothing else lives. The TUs NULL them
where the C's int stores did (melee_chara ftmasterhand*.rs / ftcrazyhand.rs). The union grows from 0x88 to 0xA8,
inside Fighter_MotionVars (0xAC). GC offsets (gc_decode / GcLayout) are unchanged.

  native_unions.py FILE     patch a generated gen.rs in place (gen_types.py applies the same patches)
"""
import sys

PATCHES = [
    # ftCommon itemthrow: x18/x1C are UNK_T (never used as pointers); the C's x20 is fp+2360, the slot ftCo_Guard's
    # guard.x20 (an int) writes before the item throw reads it (ftCo_8009515C). 8-byte pointers moved x20 to mv+0x28.
    ("""    pub x14: i32,
    pub x18: *mut c_void,
    pub x1C: *mut c_void,
    pub x20: i32,
}
impl ftCommon_MotionVars_itemthrow {""", """    pub x14: i32,
    /// native only (gen/native_unions.py): UNK_T in the C, kept 4 bytes so x20 is guard.x20's slot (fp+2360)
    pub x18: u32,
    pub x1C: u32,
    pub x20: i32,
}
impl ftCommon_MotionVars_itemthrow {"""),
    ("const _: () = assert!(core::mem::size_of::<ftCommon_MotionVars_itemthrow>() == 48);",
     "const _: () = assert!(core::mem::size_of::<ftCommon_MotionVars_itemthrow>() == 40);"),
    ("""        v.x18 = <*mut c_void as GcDecode>::gc_decode(d, at + 0x18);
        v.x1C = <*mut c_void as GcDecode>::gc_decode(d, at + 0x1C);
        v.x20 = <i32 as GcDecode>::gc_decode(d, at + 0x20);
        v
    }
}
impl GcPointee for ftCommon_MotionVars_itemthrow {""", """        v.x18 = <u32 as GcDecode>::gc_decode(d, at + 0x18);
        v.x1C = <u32 as GcDecode>::gc_decode(d, at + 0x1C);
        v.x20 = <i32 as GcDecode>::gc_decode(d, at + 0x20);
        v
    }
}
impl GcPointee for ftCommon_MotionVars_itemthrow {"""),
    ("""pub struct ftMasterHand_FingerBeamVars {
    pub pad_0: [i8; 52],""", """pub struct ftMasterHand_FingerBeamVars {
    /// native only (gen/native_unions.py): the item slots past ftMasterHand_Unk0Vars' 0x88 bytes
    pub pad_0: [i8; 0x88],"""),
    ("const _: () = assert!(core::mem::size_of::<ftMasterHand_FingerBeamVars>() == 88);",
     "const _: () = assert!(core::mem::size_of::<ftMasterHand_FingerBeamVars>() == 168);"),
    ("""pub struct ftMasterHand_GrabVars {
    pub pad_0: [i8; 40],""", """pub struct ftMasterHand_GrabVars {
    /// native only (gen/native_unions.py): the item slots past ftMasterHand_Unk0Vars' 0x88 bytes
    pub pad_0: [i8; 0x88],"""),
    ("const _: () = assert!(core::mem::size_of::<ftMasterHand_GrabVars>() == 72);",
     "const _: () = assert!(core::mem::size_of::<ftMasterHand_GrabVars>() == 168);"),
    ("""pub struct ftMasterHand_Damage_0 {
    pub pad_0: [i8; 40],
    pub x28: i32,
    pub x2C: i32,
    pub x30: i32,
    pub x34: *mut Item_GObj,""", """pub struct ftMasterHand_Damage_0 {
    /// native only (gen/native_unions.py): x28..x30 on ftMasterHand_Unk0Vars.x28..x30, the items past its 0x88 bytes
    pub pad_0: [i8; 0x30],
    pub x28: i32,
    pub x2C: i32,
    pub x30: i32,
    pub pad_3C: [i8; 0x4C],
    pub x34: *mut Item_GObj,"""),
    ("const _: () = assert!(core::mem::size_of::<ftMasterHand_Damage_0>() == 88);",
     "const _: () = assert!(core::mem::size_of::<ftMasterHand_Damage_0>() == 168);"),
    ("const _: () = assert!(core::mem::size_of::<ftMasterHand_MotionVars>() == 136);",
     "const _: () = assert!(core::mem::size_of::<ftMasterHand_MotionVars>() == 168);"),
]


import re

# the native pads are not the GC pads: their gc_decode reads are dropped (pads hold nothing)
PAD_DECODES = ["ftMasterHand_FingerBeamVars", "ftMasterHand_GrabVars", "ftMasterHand_Damage_0"]


# Motion-var records whose C `UNK_T` (void*) fields no code uses as pointers: natively 4 bytes, so every later field keeps the
# GameCube offset its union siblings alias (Yoshi's guard.x20/x24 are ftCommon guard.x20/x24; a state's ints read what another
# state stored at the same fp offset). (struct, [fields])
UNK_U32 = [
    ("ftCommon_MotionVars_capturekirby", ["x1C", "x20", "x24", "x28"]),
    ("ftCommon_MotionVars_capturekoopa", ["x4"]),
    ("ftCommon_MotionVars_common", ["x10"]),
    ("ftCommon_MotionVars_damage", ["xC", "x10"]),
    ("ftCommon_MotionVars_fighterthrow", ["x0"]),
    ("ftCommon_MotionVars_guard", ["x28"]),
    ("ftCommon_MotionVars_hammerlanding", ["x0"]),
    ("ftCommon_MotionVars_thrown", ["x8"]),
    ("ftCommon_MotionVars_unk_800D3680", ["x74"]),
    ("ftYoshi_GuardVars", ["x1C"]),
]


def native_size(txt, name):
    """repr(C) size of a generated struct of scalars / pointers / arrays / known records (8-byte pointers)."""
    prim = {"u8": 1, "i8": 1, "u16": 2, "i16": 2, "u32": 4, "i32": 4, "f32": 4, "u64": 8, "i64": 8, "f64": 8, "usize": 8, "isize": 8, "bool": 1}

    def lay(ty):
        ty = ty.strip()
        if ty in prim:
            return prim[ty], prim[ty]
        if ty.startswith(("*", "Option<", "unsafe extern", "extern")):
            return 8, 8
        am = re.fullmatch(r"\[(.+);\s*([0-9a-fA-Fx]+)\]", ty)
        if am:
            sz, al = lay(am.group(1))
            return sz * int(am.group(2), 0), al
        m = re.search(r"#\[repr\(C(?:, align\((\d+)\))?\)\]\s*(?:#\[derive[^\]]*\]\s*)?pub (struct|union) %s \{(.*?)\n\}" % re.escape(ty), txt, re.S)
        if not m:
            raise SystemExit(f"native_unions: no layout for {ty}")
        al = int(m.group(1) or 1)
        off = 0
        for ft in re.findall(r"^\s*pub (?:r#)?\w+:\s*(.+?),\s*$", m.group(3), re.M):
            fs, fa = lay(ft)
            al = max(al, fa)
            off = max(off, fs) if m.group(2) == "union" else (off + fa - 1) // fa * fa + fs
        return (off + al - 1) // al * al, al

    return lay(name)[0]


def apply_unk_u32(txt):
    for name, fields in UNK_U32:
        m = re.search(r"pub struct %s \{.*?\n\}" % name, txt, re.S)
        if not m:
            raise SystemExit(f"native_unions: struct {name} not found")
        body = m.group(0)
        for f in fields:
            old = f"    pub {f}: *mut c_void,\n"
            if body.count(old) != 1:
                raise SystemExit(f"native_unions: {name}.{f} is not one `*mut c_void`")
            body = body.replace(old, f"    /// native only (gen/native_unions.py UNK_U32): the C's 4-byte UNK_T\n    pub {f}: u32,\n")
        txt = txt[:m.start()] + body + txt[m.end():]
        for f in fields:
            pat = re.compile(r"(impl GcDecode for %s \{.*?v\.%s = <)\*mut c_void( as GcDecode>)" % (name, f), re.S)
            txt, n = pat.subn(lambda mm: mm.group(1) + "u32" + mm.group(2), txt, count=1)
            if n != 1:
                raise SystemExit(f"native_unions: decode of {name}.{f} not found")
        pat = re.compile(r"(const _: \(\) = assert!\(core::mem::size_of::<%s>\(\) == )\d+(\);)" % name)
        txt, n = pat.subn(lambda mm: mm.group(1) + str(native_size(txt, name)) + mm.group(2), txt, count=1)
        if n != 1:
            raise SystemExit(f"native_unions: size assert of {name} not found")
    return txt


def apply(txt):
    for name in PAD_DECODES:
        pat = re.compile(r"(impl GcDecode for %s \{.*?)\n\s*v\.pad_0 = <\[i8; \d+\] as GcDecode>::gc_decode\(d, at \+ 0x0\);" % name, re.S)
        txt, n = pat.subn(lambda m: m.group(1), txt, count=1)
        if n != 1:
            raise SystemExit(f"native_unions: pad decode of {name} not found")
    for old, new in PATCHES:
        if txt.count(old) != 1:
            raise SystemExit(f"native_unions: patch target not found once: {old.splitlines()[0]!r}")
        txt = txt.replace(old, new)
    return apply_unk_u32(txt)


if __name__ == "__main__":
    p = sys.argv[1]
    t = open(p, encoding="utf8", newline="").read()
    crlf = "\r\n" in t
    t = apply(t.replace("\r\n", "\n"))
    open(p, "w", encoding="utf8", newline="").write(t.replace("\n", "\r\n") if crlf else t)
