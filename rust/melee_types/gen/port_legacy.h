/* Records the Rust port still relies on after upstream dropped them (doldecomp #3583 ff.). Declared here, in the
 * decomp's own words at the commit before the drop, so gen_types.py keeps generating their native type, GameCube
 * layout and decoder like any other record (the layouts are still cross-checked against the ppc32 dump).
 *
 * sislib: upstream now reads the SIS font archive and the text state through `u8*` (HSD_Text.sis_buffer, x60,
 * state_stack) and types alloc_data as SisBuffer; the port's sislib.rs / melee::retail::sis and the engine's DAT
 * decode (arrays.tsv: SIS.kerning / SIS.textures run to the next object) still use the typed views below. */
#ifndef MELEE_RUST_PORT_LEGACY_H
#define MELEE_RUST_PORT_LEGACY_H
/* A TU oracle that only needs the item views defines PORT_LEGACY_ITEMS_ONLY before including this header. */
#ifndef PORT_LEGACY_ITEMS_ONLY
#include <sysdolphin/baselib/sislib.h>
#include <sysdolphin/baselib/sislib_font.h>

typedef struct TextKerning {
    /*0x00*/ u8 left;
    /*0x01*/ u8 right;
} TextKerning;

typedef struct SIS {
    /*0x00*/ TextKerning* kerning;
    /*0x04*/ TextGlyphTexture* textures;
} SIS;

typedef struct sisLib_803A7664_t {
    void* x0;
    void* x4;
    u32 x8; ///< alloc size
    u32 xC;
} sisLib_803A7664_t;


/* Item attribute views (it/itCommonItems.h, it/kinds/types.h, it/kinds/itkinko.h before doldecomp c05757540 / #3583):
 * upstream now types x4_specialAttributes as a union of per-item records; the port's item ports still read these
 * generic views over the same bytes. itUnkAttributes itself survives upstream as a 1-float record (It_Kind_Unk1), so
 * the port's 5-float view keeps the old layout under the _legacy name. */
#endif /* PORT_LEGACY_ITEMS_ONLY */

#include <melee/it/types.h>
#include <melee/it/kinds/types.h>
typedef struct itUnkAttributes_legacy {
    float x0_float;
    float x4_float;
    float x8;
    float xC;
    float x10;
} itUnkAttributes_legacy;

typedef struct itPokemonAttributes {
    f32 x0;
    s32 timer;
    s32 max;
    s32 xC;
    s32 x10;
    s32 x14;
    s32 x18;
} itPokemonAttributes;

typedef struct itRshellAttributes {
    u8 _pad[0x14];
    Vec3 x14;
} itRshellAttributes;

typedef struct KinokoAnim {
    HSD_AnimJoint* joint;
} KinokoAnim;

struct itSword_UnkArticle1 {
    /*  +0 */ UNK_T x0;
    /*  +4 */ UNK_T x4;
    /*  +8 */ UNK_T x8;
    /*  +C */ float xC;
    /* +10 */ float x10;
    /* +14 */ float x14;
    /* +18 */ int x18;
    /* +1C */ itSword_UnkBytes x1C;
};

/* ty/types.h before #3553: the trophy list's view of the SIS font (`font + 0x4B8` row strings, the digits). */
struct SisFontData {
    u8 pad[0x4B8];
    u8* rows[12];
    u8* digits;
};


#ifndef PORT_LEGACY_ITEMS_ONLY
/* mn/ records before #3583 / #3562: mndiagram.c kept the menu archive pointers and the assert strings in its asset and
 * anim-table records (the port's mndiagram.rs mirrors that layout); mndatadel.static.h's WarnCmnData and
 * mnstagesel.static.h's model table are read by mndatadel.rs / mnstagesel.rs. */
#include <melee/lb/types.h>
#include <melee/mn/types.h>
#include <melee/sc/types.h>
typedef struct mnDiagram_Assets_legacy {
    /* 0x00 */ u8 sorted_fighters[0x19];
    /* 0x19 */ u8 pad_19[3];
    /* 0x1C */ u8 sorted_names[0x78];
    /* 0x94 */ void* SubB1[4];
    /* 0xA4 */ void* NmB[4];
    /* 0xB4 */ void* FaceB[4];
    /* 0xC4 */ void* CursorB1[4];
    /* 0xD4 */ void* ConB1[4];
    /* 0xE4 */ void* ConB2[4];
    /* 0xF4 */ void* ConB3[4];
    /* 0x104 */ void* CursorB3[5];
} mnDiagram_Assets_legacy;

typedef struct mnDiagram_AnimTable_legacy {
    /* 0x00 */ Vec3 points[3];
    /* 0x24 */ u8 default_fighter_order[0x1C];
    /* 0x40 */ AnimLoopSettings intro_anim;
    /* 0x4C */ AnimLoopSettings exit_anim;
    /* 0x58 */ AnimLoopSettings arrow_anim;
    /* 0x64 */ AnimLoopSettings cursor_anim;
    /* 0x70 */ char user_data_error[0x18];
    /* 0x88 */ char file_name[0xC];
    /* 0x94 */ char user_data_name[0x14];
} mnDiagram_AnimTable_legacy;

struct WarnCmnData {
    /* +00 */ u8 x0;
    /* +01 */ u8 visible;
    /* +02 */ u8 cursor_idx;
};

struct mnStageSel_804D6C98_t {
    StaticModelDesc x0;
    StaticModelDesc x10;
    StaticModelDesc x20;
    StaticModelDesc x30;
    StaticModelDesc x40;
    StaticModelDesc x50;
    StaticModelDesc x60;
    StaticModelDesc x70;
    StaticModelDesc x80;
    StaticModelDesc x90;
    StaticModelDesc xA0;
    HSD_Joint* xB0;
    HSD_AnimJoint* xB4;
    HSD_MatAnimJoint* xB8;
    HSD_ShapeAnimJoint* xBC;
};


/* mnstagesel.c's stage-icon row, in its TARGET_PC layout (the record's closing brace sits inside the #if branch, so the
 * generator cannot extract it verbatim). */
struct StageListInfo {
    HSD_JObj* x0;
    int x4;
    u8 x8, x9, xA;
    s32 stkind;
    f32 xC, x10, x14, x18;
};

#endif /* PORT_LEGACY_ITEMS_ONLY */
#endif
