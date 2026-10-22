"""Oracle patch for src/melee/gm/gmregclear.c (applied to a copy; see gen/tu/run.py).

The TU views one static object (lbl_80472D28) through two structs, `lbl_80472D28_t` and
`fn_8017FA1C_arg`, whose fields are meant to alias by GameCube offset (4-byte pointers). On a
64-bit build the two layouts diverge, and `fn_8017F47C` also indexes the text-pointer array with an
`s32*` (`p[8]`) assuming 4-byte slots. The patch makes the two views ONE struct (every name from
both, in GC order, contiguous pointer runs kept contiguous) and moves the `p[8]` cache index to the
8 text slots, so the native C computes what the PPC build computes. The Rust port
(tu/gmregclear.rs `RegClear`) mirrors this struct field for field.
"""

UNIFIED = """struct lbl_80472D28_t {
    HSD_GObj* x0;
    HSD_JObj* x4;
    HSD_JObj* x8;
    HSD_JObj* xC;
    HSD_JObj* x10;
    HSD_JObj* x14;
    HSD_JObj* x18;
    HSD_JObj* x1C;
    HSD_JObj* x20;
    HSD_JObj* x24;
    HSD_JObj* x28;
    HSD_GObj* x2C;
    HSD_ImageDesc x30;
    HSD_Archive* x48;
    DynamicModelDesc x4C;
    void* x5C;
    void* x60;
    HSD_Text* x6C;
    HSD_Text* x70;
    HSD_Text* x74;
    HSD_Text* x78;
    HSD_Text* x7C;
    HSD_Text* x80;
    HSD_Text* x84;
    HSD_Text* x88;
    HSD_Text* x8C;
    HSD_Text* x90;
    HSD_Text* x94;
    HSD_Text* x98;
    HSD_Text* x9C;
    HSD_Text* xA0;
    s32 xA4[8];
    u16 xC0;
    u16 pad_C2;
    u32 xC4;
    u8 xC8;
    u8 pad_C9[3];
    s32 xCC;
    s32 xD0;
    s32 xD4;
    s32 xD8;
    s32 xDC;
    s32 xE0;
    s32 xE4;
    s32 xE8;
    s32 xEC;
    s32 xF0;
    s32 xF4;
    s32 xF8;
    s32 xFC;
    s32 x100;
    int x104;
    s16 x108;
    s16 x10A;
    f32 x10C;
    u32 x110;
    u8 x114;
    u8 x115;
    u8 x116;
    u8 x117;
    u8 x118;
    u8 x119;
    u8 x11A;
    u8 x11B;
    u8 x11C;
    u8 x11D;
    u8 x11E;
    u8 x11F;
};"""


def patch(t):
    a = t.index("struct lbl_80472D28_t {")
    b = t.index("};", a) + 2
    t = t[:a] + UNIFIED + t[b:]
    a = t.index("typedef struct fn_8017FA1C_arg {")
    end = "} fn_8017FA1C_arg;"
    b = t.index(end, a) + len(end)
    t = t[:a] + "typedef struct lbl_80472D28_t fn_8017FA1C_arg;" + t[b:]
    assert t.count("p[8]") == 2
    t = t.replace("p[8]", "((s32*) (arg0 + 8))[i]")
    return t
