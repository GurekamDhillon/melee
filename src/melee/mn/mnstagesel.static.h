#ifndef __GALE01_2599EC
#define __GALE01_2599EC

#include <Runtime/platform.h>

#include <melee/mn/forward.h>

#include <melee/sc/types.h>

/// Number of actual (selectable) stages on the retail stage-select screen.
#define NUM_STAGES_RETAIL 29
/// Retail stage-select icons: ::NUM_STAGES_RETAIL stages plus the trailing
/// "Random" entry at index ::NUM_STAGES_RETAIL.
#define SSS_ICON_COUNT_RETAIL (NUM_STAGES_RETAIL + 1)

#if defined(TARGET_PC)
/// Rows #mnStageSel_803F06D0 is compiled for. Akaneia ships 67 icons and ACE
/// 163; `Mex_SssIconCount()` rejects anything past GW_MEX_SSS_MAX, which is
/// this same number, so disc data can never overrun the table.
#define SSS_ICON_MAX 256
/// Live icon counts, read from `mexData.metadata.sss_icon_count`. Ported from
/// m-ex (https://github.com/akaneia/m-ex): asm/m-ex/SSS Expansion/, whose
/// MnSlMap References/ patches replace retail's constant 29/30 with that count
/// at ~40 call sites. These hold the retail values until mnStageSel_MexSetup()
/// finds an m-ex table, so a vanilla disc behaves exactly as it did.
static int mnStageSel_NumStages = NUM_STAGES_RETAIL;
static int mnStageSel_IconCount = SSS_ICON_COUNT_RETAIL;
/// Non-zero once the table came from mexData. Guards the places where retail's
/// fixed 30-icon screen geometry is assumed.
static bool mnStageSel_MexActive;
#define NUM_STAGES mnStageSel_NumStages
#define SSS_ICON_COUNT mnStageSel_IconCount
/// First m-ex-ADDED external stage id (external 288+k -> internal 71+k, the
/// same on Akaneia and ACE). Retail ids are below it, and they are the only
/// ones with a bit in the save file's random-stage mask.
#define SSS_EXT_MEX_FIRST 288
/// Rows in `lbl_803B7808[]` (gm/gm_1601.static.h), which an icon's `xA` is an
/// index into. Retail never produces an out-of-range one; a disc's own table
/// can, so #sss_random_ok checks it.
#define SSS_RANDOM_ID_COUNT 30
#else
#define NUM_STAGES NUM_STAGES_RETAIL
#define SSS_ICON_COUNT SSS_ICON_COUNT_RETAIL
#endif
/// Iteration cap for the random-stage picker in mnStageSel_802599EC.
#define MAX_ITER 100000

/// One stage-select icon: model joint, random-picker cooldown, type/preview/
/// random-slot bytes, the external stage id, then the cursor's half-width and
/// half-height and the cursor's x/y scale.
///
/// Under TARGET_PC `stkind` is an `s32`: m-ex's external stage space is 313
/// ids on Akaneia and 372 on ACE, and neither fits the retail `u8`. m-ex made
/// the same change - in its own table the byte is always zero and the id lives
/// in a word of its own at +0x1C. The field ORDER is left alone so the retail
/// rows below read identically in both builds.
struct stagelistinfo {
    HSD_JObj* x0;
    int x4;
#if defined(TARGET_PC)
    u8 x8, x9, xA;
    s32 stkind;
    f32 xC, x10, x14, x18;
} mnStageSel_803F06D0[SSS_ICON_MAX] = {
#else
    u8 x8, x9, xA, stkind;
    f32 xC, x10, x14, x18;
} mnStageSel_803F06D0[30] = {
#endif
    { 0, 0, 0x2, 0x00, 0x00, 0x04, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x01, 0x0C, 0x0B, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x02, 0x01, 0x05, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x03, 0x0D, 0x0C, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x04, 0x02, 0x0D, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x05, 0x0E, 0x0E, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x08, 0x04, 0x08, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x09, 0x10, 0x10, 2.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0A, 0x05, 0x02, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0B, 0x11, 0x11, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0C, 0x06, 0x07, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0D, 0x12, 0x16, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x06, 0x03, 0x06, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x07, 0x0F, 0x0F, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x12, 0x09, 0x09, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x13, 0x15, 0x12, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x10, 0x08, 0x0A, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x11, 0x14, 0x18, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0E, 0x07, 0x03, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x0F, 0x13, 0x17, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x14, 0x0B, 0x13, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x15, 0x16, 0x14, 3.1F, 2.7F, 1.0F, 1.0F },
    { 0, 0, 0x2, 0x16, 0x0A, 0x19, 3.1F, 2.9F, 1.0F, 1.1F },
    { 0, 0, 0x2, 0x17, 0x17, 0x1B, 3.1F, 2.9F, 1.0F, 1.1F },
    { 0, 0, 0x2, 0x18, 0x18, 0x1F, 2.9F, 2.1F, 0.8F, 0.8F },
    { 0, 0, 0x2, 0x19, 0x19, 0x20, 2.9F, 2.1F, 0.8F, 0.8F },
    { 0, 0, 0x2, 0x1A, 0x1A, 0x1C, 2.9F, 2.1F, 0.8F, 0.8F },
    { 0, 0, 0x2, 0x1B, 0x1B, 0x1D, 2.9F, 2.1F, 0.8F, 0.8F },
    { 0, 0, 0x2, 0x1C, 0x1C, 0x1E, 2.9F, 2.1F, 0.8F, 0.8F },
    { 0, 0, 0x2, 0x1D, 0x00, 0x00, 3.6F, 2.7F, 1.2F, 1.0F },
};
static s8 mnStageSel_804D50A0 = -1;

static SSSData* sss_data;
static HSD_Archive* mnStageSel_804D6C94;
static struct mnStageSel_804D6C98_t {
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
}* mnStageSel_804D6C98;
static HSD_GObj* mnStageSel_804D6C9C;
static u32 mnStageSel_804D6CA0;
static u32 mnStageSel_804D6CA4;
static s32 mnStageSel_804D6CA8;
static s8 mnStageSel_804D6CAC;
static s8 mnStageSel_804D6CAD;
#if defined(TARGET_PC)
/* The hovered icon, with #SSS_ICON_COUNT as the "nothing hovered" sentinel. A
 * u8 still holds ACE's 163, but nothing in the data caps the count at 255, and
 * an index that wraps silently hovers a different stage. */
static s32 mnStageSel_804D6CAE;
#else
static u8 mnStageSel_804D6CAE;
#endif
static u8 mnStageSel_804D6CAF;

#ifndef M2CTX
#if defined(TARGET_PC)
ASSERT_SIZE(mnStageSel_803F06D0[0], 0x20);
#else
ASSERT_SIZE(mnStageSel_803F06D0[0], 0x1C);
#endif
#endif

struct StageSelUserData {
    u16 x0;
    u16 x2;
    int x4;
};

#endif
