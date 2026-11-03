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
    return txt


if __name__ == "__main__":
    p = sys.argv[1]
    t = open(p, encoding="utf8", newline="").read()
    crlf = "\r\n" in t
    t = apply(t.replace("\r\n", "\n"))
    open(p, "w", encoding="utf8", newline="").write(t.replace("\n", "\r\n") if crlf else t)
