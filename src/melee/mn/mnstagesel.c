#include "mnstagesel.h"

#include <placeholder.h>

#include "inlines.h"
#include "mnmain.h"
#include "mnstagesel.static.h"
#include <melee/gm/gm_unsplit.h>
#include <melee/gm/gmmain_lib.h>
#include <melee/lb/lb_00B0.h>
#include <melee/lb/lb_013B.h>
#include <melee/lb/lbarchive.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/lblanguage.h>
#include <melee/lb/types.h>
#include <sysdolphin/baselib/controller.h>
#include <sysdolphin/baselib/fog.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/gobjobject.h>
#include <sysdolphin/baselib/gobjplink.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/gobjuserdata.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/lobj.h>
#include <sysdolphin/baselib/memory.h>
#include <sysdolphin/baselib/random.h>
#if defined(TARGET_PC)
#include <sysdolphin/baselib/robj.h>
#endif

/// @todo .sdata2 order hack
#ifdef MUST_MATCH
static void order_sdata2(void)
{
    (void) S32_TO_F32;
}
#endif

#if defined(TARGET_PC)
/* ---- m-ex stage-select expansion ---------------------------------------
 *
 * Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/SSS Expansion/
 * - `MnSlMap References/` (retail's constant 29/30 and 0x1C stride become the
 * live icon count and 0x20), `mexMapData/Load mexMapData.asm` (the icon
 * models) and `SSSPages/LRCheck.asm` (L/R paging). Reimplemented in original
 * C; none of m-ex's sources are vendored. See _research/mex-stages.md
 * section 7.
 *
 * Verified off both discs rather than quoted:
 *   - `mexData.menu +0x08` is the icon table, stride 0x20. Rows 0..28
 * reproduce the retail rows below field for field; row 1 is w2 = 0x02010C00
 * (x8 = 2, x9 = 1, xA = 0x0C, byte 3 = 0) with the external id 11 in its own
 * word.
 *   - `metadata.sss_icon_count` is 67 on Akaneia, 163 on ACE.
 *   - MnSlMap.usd carries a `mexMapData` public symbol on both discs, and its
 *     positions model has exactly that many sibling joints (67 / 163) - one
 * per icon, which is what the icon loop walks.
 *   - Akaneia's row 61 is external 293 = Meta Crystal, and it has TWO "Random"
 *     icons (rows 29 and 60), so the Random entry is NOT at a fixed index any
 *     more and is found by its type byte (3) instead.
 *
 * Layout, per icon: x8 type (0 hidden, 1 locked/blank, 2 shown, 3 Random),
 * x9 preview id (0xFF = no preview), xA retail random-slot id, stkind the
 * EXTERNAL stage id. */

/// The MnSlMap archive's `mexMapData`: the one icon model every icon shares,
/// the positions model whose child chain gives each icon its place, the
/// stage-name matanim, and the paging block.
typedef struct MexMapPages {
    /* +00 */ HSD_JObj* jobj; /* filled in at init with the positions root */
    /* +04 */ int count;
    /* +08 */ int current;
    /* +0C */ int unk_C;
    /* +10 */ HSD_AnimJoint** anims;
} MexMapPages;

typedef struct MexMapData {
    /* +00 */ HSD_Joint* icon_joint;
    /* +04 */ HSD_AnimJoint* icon_animjoint;
    /* +08 */ HSD_MatAnimJoint* icon_matanim;
    /* +0C */ HSD_Joint* pos_joint;
    /* +10 */ HSD_AnimJoint* pos_animjoint;
    /* +14 */ HSD_MatAnimJoint* stagename_matanim;
    /* +18 */ MexMapPages* pages;
} MexMapData;

static MexMapData* mnStageSel_MexMap;

static void do_anim(HSD_JObj* jobj, int frame);

extern int Mex_SssIconCount(void);
extern void* Mex_SssTable(void);
extern int Mex_GrKindForExt(int ext);
extern const char* Mex_GrFile(int grkind);

/// m-ex's own row, exactly as it sits in MxDt.dat. Read by gwtool-compiled
/// code, so every field arrives byte-swapped for free; the copy into
/// ::stagelistinfo is field by field so the two layouts stay independent.
typedef struct MexSssIcon {
    /* +00 */ u32 runtime_joint;
    /* +04 */ s32 runtime_cooldown;
    /* +08 */ u8 type;
    /* +09 */ u8 preview_id;
    /* +0A */ u8 random_id;
    /* +0B */ u8 pad;
    /* +0C */ f32 cursor_w;
    /* +10 */ f32 cursor_h;
    /* +14 */ f32 cursor_scale_x;
    /* +18 */ f32 cursor_scale_y;
    /* +1C */ s32 stkind;
} MexSssIcon;

/// True when icon @p i may be rolled by the Random picker.
///
/// Retail's picker keys on `gm_80164330(xA)`, and xA is an index into
/// `lbl_803B7808[30]` AND into the save file's 29-bit random-stage mask -
/// vanilla-sized per-index structures with no room for an added stage. Every
/// m-ex-added row has xA = 0, so putting them through that path would make all
/// of them share retail slot 0's bit. Added stages are left out of Random
/// rather than handed someone else's bit, and blank slots are excluded
/// outright so Random can never roll an icon that names no stage.
static bool sss_random_ok(int i)
{
    if (mnStageSel_MexActive) {
        if (mnStageSel_803F06D0[i].x8 < 2 || mnStageSel_803F06D0[i].x8 == 3) {
            return false;
        }
        if (mnStageSel_803F06D0[i].stkind >= SSS_EXT_MEX_FIRST) {
            return false;
        }
        /* xA indexes `lbl_803B7808[30]`, and gm_80164330() truncates it to u8
         * before doing so, so a row carrying anything but a retail random id
         * reads past that table. m-ex's rows 0..28 reproduce retail's, but
         * nothing in mexData constrains a row naming a vanilla external id to
         * do the same - and ACE ships 163 rows where Akaneia ships 67. Refuse
         * rather than read off the end. */
        if (mnStageSel_803F06D0[i].xA >= SSS_RANDOM_ID_COUNT) {
            return false;
        }
    }
    return gm_80164330(mnStageSel_803F06D0[i].xA) != 0;
}

/// True when icon @p i has a preview model to show.
/// m-ex's `MnSlMap References/Null Value
/// Adjustment/CheckForRandomAndNull.asm`: the "nothing hovered" sentinel, the
/// Random icon and a preview id of -1 all mean "no preview". Retail just asks
/// whether the index is a stage.
static bool sss_has_preview(int i)
{
    if (!mnStageSel_MexActive) {
        return i < NUM_STAGES;
    }
    return i >= 0 && i < SSS_ICON_COUNT && mnStageSel_803F06D0[i].x8 >= 2 &&
           mnStageSel_803F06D0[i].x8 != 3 && mnStageSel_803F06D0[i].x9 != 0xFF;
}

/// Read the icon table out of mexData, once per visit to the screen.
/// Leaves everything at its retail value and returns with
/// #mnStageSel_MexActive clear on a disc with no m-ex data, or when the m-ex
/// stage tables are off - an icon names an EXTERNAL stage id, and without
/// those tables the id resolves to nothing.
static void mnStageSel_MexSetup(HSD_Archive* archive)
{
    MexSssIcon* tbl;
    int n, i;

    mnStageSel_MexMap = NULL;
    mnStageSel_MexActive = false;
    mnStageSel_NumStages = NUM_STAGES_RETAIL;
    mnStageSel_IconCount = SSS_ICON_COUNT_RETAIL;

    /* The models are as necessary as the table: retail's screen draws its 30
     * icons from three hard-coded models at fixed joints, and there is no
     * geometry anywhere for a 31st. */
    mnStageSel_MexMap = HSD_ArchiveGetPublicAddress(archive, "mexMapData");
    if (mnStageSel_MexMap == NULL || mnStageSel_MexMap->icon_joint == NULL ||
        mnStageSel_MexMap->pos_joint == NULL)
    {
        mnStageSel_MexMap = NULL;
        return;
    }
    n = Mex_SssIconCount();
    tbl = (n > SSS_ICON_COUNT_RETAIL && n <= SSS_ICON_MAX) ? Mex_SssTable()
                                                           : NULL;
    if (tbl == NULL) {
        mnStageSel_MexMap = NULL;
        return;
    }

    for (i = 0; i < n; i++) {
        struct stagelistinfo* icon = &mnStageSel_803F06D0[i];
        icon->x0 = NULL;
        icon->x4 = 0;
        icon->x8 = tbl[i].type;
        icon->x9 = tbl[i].preview_id;
        icon->xA = tbl[i].random_id;
        icon->stkind = tbl[i].stkind;
        icon->xC = tbl[i].cursor_w;
        icon->x10 = tbl[i].cursor_h;
        icon->x14 = tbl[i].cursor_scale_x;
        icon->x18 = tbl[i].cursor_scale_y;
    }
    mnStageSel_IconCount = n;
    /* Retail keeps one trailing Random entry at NUM_STAGES and treats every
     * lower index as a stage. m-ex scatters its Random icons through the
     * table, so there is no such split: every icon is a real row and only the
     * type byte says what it is. The loops that ran to NUM_STAGES must
     * therefore run to the whole count. */
    mnStageSel_NumStages = n;
    mnStageSel_MexActive = true;
}

/// Mark each icon locked or unlocked, the way the retail loop does, but
/// without letting a blank slot become selectable: `gm_80164430(0)` answers
/// for external stage 0, which is not this icon.
static void mnStageSel_MexScanUnlocked(void)
{
    int i;
    for (i = 0; i < SSS_ICON_COUNT; i++) {
        struct stagelistinfo* icon = &mnStageSel_803F06D0[i];
        int grkind;
        if (icon->x8 == 0 || icon->x8 == 3) {
            continue; /* hidden slot, or the Random icon: keep what m-ex
                         authored */
        }
        grkind = icon->stkind > 0 ? Mex_GrKindForExt(icon->stkind) : -1;
        /* An added stage the disc does not actually ship has a table row but
         * no file; selecting it would walk off into a missing file at load. */
        if (grkind < 0 ||
            (icon->stkind >= SSS_EXT_MEX_FIRST && Mex_GrFile(grkind) == NULL))
        {
            icon->x8 = 1;
            continue;
        }
        icon->x8 = gm_80164430(icon->stkind) ? 2 : 1;
    }
}

/// Build the icons from mexMapData: one shared model per icon, placed on the
/// nth joint of the positions model. Replaces retail's three fixed models and
/// its hard-coded 11 pairs + 5 + 1 + 2 layout entirely.
static void mnStageSel_MexInitIcons(void)
{
    HSD_GObj* gobj;
    HSD_JObj* root;
    HSD_JObj* pos;
    int i;

    mnStageSel_MexScanUnlocked();

    gobj = GObj_Create(4, 5, 0x80);
    root = HSD_JObjLoadJoint(mnStageSel_MexMap->pos_joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x82);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 4);
    HSD_JObjAddAnimAll(root, mnStageSel_MexMap->pos_animjoint, NULL, NULL);
    HSD_JObjReqAnimAll(root, 0.0F);
    HSD_JObjAnimAll(root);
    if (mnStageSel_MexMap->pages != NULL) {
        mnStageSel_MexMap->pages->jobj = root;
        mnStageSel_MexMap->pages->current = 0;
    }

    pos = root->child;
    for (i = 0; i < SSS_ICON_COUNT && pos != NULL; i++, pos = pos->next) {
        struct stagelistinfo* icon = &mnStageSel_803F06D0[i];
        HSD_JObj* jobj;
        HSD_GObj* g = GObj_Create(4, 5, 0x80);

        /* The Random icon keeps retail's own model - it is the one icon whose
         * art is not a frame of the shared icon matanim. */
        jobj =
            HSD_JObjLoadJoint(icon->x8 == 3 ? mnStageSel_804D6C98->x10.joint
                                            : mnStageSel_MexMap->icon_joint);
        HSD_GObjObject_80390A70(g, HSD_GObj_JObjKind, jobj);
        GObj_SetupGXLink(g, HSD_GObj_JObjCallback, 4, 0x83);
        HSD_GObj_SetupProc(g, mn_8022EAE0, 3);
        HSD_JObjAddAnimAll(jobj, mnStageSel_MexMap->icon_animjoint,
                           mnStageSel_MexMap->icon_matanim, NULL);
        jobj = GET_JOBJ(g);

        /* Follow the position joint in translation, rotation AND scale. The
         * last one is why this is not lb_8000C2F8: the positions model scales
         * a page's icons, and the icons have to scale with it. */
        lb_8000C2F8(jobj, pos);
        {
            HSD_RObj* robj = HSD_RObjAlloc();
            HSD_RObjSetFlags(robj, 0x90000008);
            HSD_RObjSetConstraintObj(robj, pos);
            HSD_JObjPrependRObj(jobj, robj);
        }
        icon->x0 = jobj;

        if (icon->x8 == 0) {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN);
            continue;
        }
        /* One matanim frame per icon: 0 is unused, 1 is the locked plate and
         * icon i's art is frame i + 2. The Random icon's own model has its own
         * single frame, so it takes the locked frame's slot. */
        do_anim(jobj, icon->x8 == 1 ? 1 : i + 2);
    }
    if (i < SSS_ICON_COUNT) {
        /* Fewer position joints than icons: the remaining rows have no place
         * to be drawn, so hide them rather than leave them hit-testable at the
         * origin. */
        for (; i < SSS_ICON_COUNT; i++) {
            mnStageSel_803F06D0[i].x0 = NULL;
            mnStageSel_803F06D0[i].x8 = 0;
        }
    }
}

/// L/R paging (m-ex `SSSPages/LRCheck.asm`). 67 icons do not fit one screen;
/// the positions model carries one animation per page and moves the whole
/// grid. Returns true when a page change was consumed this frame.
static bool mnStageSel_MexChangePage(void)
{
    MexMapPages* pages;
    HSD_JObj* jobj;
    int page;

    if (!mnStageSel_MexActive || mnStageSel_MexMap == NULL) {
        return false;
    }
    pages = mnStageSel_MexMap->pages;
    if (pages == NULL || pages->count <= 1 || pages->anims == NULL) {
        return false;
    }
    jobj = pages->jobj;
    if (jobj == NULL) {
        return false;
    }
    /* 0x40 = L, 0x20 = R. */
    if (!(mnStageSel_804D6CA0 & 0x60)) {
        return false;
    }
    page = pages->current;
    if (mnStageSel_804D6CA0 & 0x20) {
        page = (page + 1 < pages->count) ? page + 1 : 0;
    } else {
        page = (page > 0) ? page - 1 : pages->count - 1;
    }
    pages->current = page;

    lbAudioAx_80024030(2);
    HSD_JObjAddAnimAll(jobj, pages->anims[page], NULL, NULL);
    HSD_JObjReqAnimAll(jobj, 0.0F);
    HSD_JObjAnimAll(jobj);
    /* Drop the hover: the icon under the cursor is about to be a different
     * one. */
    mnStageSel_804D6CAE = SSS_ICON_COUNT;
    mnStageSel_804D6CA4 = 15;
    return true;
}
#endif

#if defined(TARGET_PC)
#define SSS_RANDOM_OK(i) sss_random_ok(i)
#else
#define SSS_RANDOM_OK(i) ((u8) gm_80164330(mnStageSel_803F06D0[i].xA))
#endif

/// Random stage selection
/// Returns an internal stage ID - 2 (since first 2 internal stage IDs are
/// invalid)
int mnStageSel_802599EC(void)
{
    int var_r0;
    int iter;
    bool var_r29 = true;
    int i;

    for (i = 0; i < NUM_STAGES; i++) {
        if (mnStageSel_803F06D0[i].x4 >= 0 && SSS_RANDOM_OK(i)) {
            break;
        }
    }
    if (i == NUM_STAGES) {
        for (i = 0; i < NUM_STAGES; i++) {
            mnStageSel_803F06D0[i].x4 = 0;
        }
    }
#if defined(TARGET_PC)
    /* Retail spins here until some row is both off cooldown and eligible. With
     * an expanded table it is no longer self-evident that any row ever is (a
     * build could ship nothing but added stages), and an unbounded spin inside
     * a menu is a hang, not a bug report. */
    for (iter = 0; var_r29 && iter < MAX_ITER; iter++) {
#else
    while (var_r29) {
#endif
        for (i = 0; i < NUM_STAGES; i++) {
            if (mnStageSel_803F06D0[i].x4 > 0) {
                mnStageSel_803F06D0[i].x4--;
            }
        }
        for (i = 0; i < NUM_STAGES; i++) {
            if (mnStageSel_803F06D0[i].x4 == 0 && SSS_RANDOM_OK(i)) {
                var_r29 = false;
            }
        }
    }
    for (iter = 0; iter < MAX_ITER; iter++) {
        int tmp = HSD_Randi(NUM_STAGES);
        i = tmp;
        if (mnStageSel_803F06D0[i].x4 == 0) {
            if (SSS_RANDOM_OK(i)) {
                break;
            }
        }
    }
    if (iter >= MAX_ITER) {
        i = 0;
    }
    mnStageSel_803F06D0[i].x4 = -1;
    /* Retail's icons 0..21 are drawn in pairs sharing one model, so picking
     * one puts its partner on a short cooldown. m-ex's grid has no pairs. */
#if defined(TARGET_PC)
    if (!mnStageSel_MexActive && i < 22) {
#else
    if (i < 22) {
#endif
        if (i & 1) {
            var_r0 = i - 1;
        } else {
            var_r0 = i + 1;
        }
        if (mnStageSel_803F06D0[var_r0].x4 >= 0) {
            mnStageSel_803F06D0[var_r0].x4 = 3;
        }
    }
    return i;
}

void mnStageSel_80259C28(void)
{
    HSD_JObj* jobj;
    HSD_GObj* gobj;
    u64 _[2];

    if (mnStageSel_804D6CA4 != 0) {
        return;
    }
#if defined(TARGET_PC)
    /* Retail's Random icon is the single trailing entry at NUM_STAGES, so the
     * retail switch can name it as a case label. m-ex puts Random icons
     * anywhere in the table (Akaneia has two, at 29 and 60) and marks them
     * with type 3, so the test has to be on the data. */
    {
        bool hovered =
            mnStageSel_804D6CAE >= 0 && mnStageSel_804D6CAE < SSS_ICON_COUNT;
        int type = hovered ? mnStageSel_803F06D0[mnStageSel_804D6CAE].x8 : 0;
        bool random =
            !hovered || type == 3 ||
            (!mnStageSel_MexActive && mnStageSel_804D6CAE == NUM_STAGES);

        if (!(mnStageSel_804D6CA0 & (hovered ? 0x1100 : 0x1000))) {
            return;
        }
        if (random) {
            mnStageSel_804D6CAE = mnStageSel_802599EC();
        } else if (type < 2) {
            lbAudioAx_80024030(3);
            return;
        }
    }
#else
    switch (mnStageSel_804D6CAE) {
    case SSS_ICON_COUNT:
        if (!(mnStageSel_804D6CA0 & 0x1000)) {
            return;
        }
        break;
    case NUM_STAGES:
        if (!(mnStageSel_804D6CA0 & 0x1100)) {
            return;
        }
        break;
    default:
        if (!(mnStageSel_804D6CA0 & 0x1100)) {
            return;
        }
        if (mnStageSel_804D6CAE < SSS_ICON_COUNT &&
            mnStageSel_803F06D0[mnStageSel_804D6CAE].x8 >= 2)
        {
            goto skip_randomize;
        }
        lbAudioAx_80024030(3);
        return;
    }
    mnStageSel_804D6CAE = mnStageSel_802599EC();
skip_randomize:
#endif

    gobj = GObj_Create(HSD_GOBJ_CLASS_FIGHTER, 5, 0x80);
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->xB0);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x87);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 0);
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->xB4,
                       mnStageSel_804D6C98->xB8, mnStageSel_804D6C98->xBC);
    HSD_JObjReqAnimAll(gobj->hsd_obj, 0.0F);
    HSD_JObjAnimAll(gobj->hsd_obj);
    mnStageSel_804D6CAF = 1;
    mnStageSel_804D6CA4 = 0x1E;
    sfxForward();
}

void fn_80259D84(HSD_GObj* gobj)
{
    struct StageSelUserData* temp_r31 = HSD_GObjGetUserData(gobj);
    HSD_JObj* jobj = GET_JOBJ(gobj);

    switch (temp_r31->x2) {
    case 0:
        if (mnStageSel_804D6CAF == 0) {
            mnStageSel_80259C28();
        }
        if (++temp_r31->x4 >= 9U) {
            HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim,
                            AOBJ_ARG_AOV, 0, 0);
            temp_r31->x2 = 1;
        }
        break;
    case 1:
        if (mnStageSel_804D6CAF == 0) {
            mnStageSel_80259C28();
        }
        if (temp_r31->x0 != mnStageSel_804D6CAE) {
            if (temp_r31->x0 < SSS_ICON_COUNT &&
                mnStageSel_803F06D0[temp_r31->x0].x8 >= 2)
            {
                HSD_JObjReqAnimAllByFlags(jobj, 1, 10.0F);
            }
            sfxMove();
            temp_r31->x4 = 0;
            temp_r31->x2 = 2;
            mnStageSel_80259ED8(mnStageSel_804D6CAE);
        }
        break;
    case 2:
        if (++temp_r31->x4 > 0xAU) {
            HSD_GObjFree(gobj);
            temp_r31->x2++;
        }
        break;
    }
}

static void do_anim(HSD_JObj* jobj, int frame)
{
    HSD_JObjReqAnimAll(jobj, 0.0F);
    HSD_JObjReqAnimAllByFlags(jobj, 0x10, frame);
    HSD_JObjAnimAll(jobj);
    HSD_ForeachAnim(jobj, JOBJ_TYPE, TOBJ_MASK, HSD_AObjStopAnim, AOBJ_ARG_AOV,
                    0, 0);
}

void mnStageSel_80259ED8(int id)
{
    HSD_GObj* gobj;
    HSD_JObj* jobj;
    struct StageSelUserData* temp_r3_2;
    u8 _[4];

    gobj = GObj_Create(HSD_GOBJ_CLASS_FIGHTER, 5, 0x80);
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x30.joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x84);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 1);
#if defined(TARGET_PC)
    /* m-ex's stage-name strip is its own matanim with one frame per ICON, in
     * place of retail's 30-name strip indexed by preview id. */
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x30.animjoint,
                       mnStageSel_MexMap != NULL
                           ? mnStageSel_MexMap->stagename_matanim
                           : mnStageSel_804D6C98->x30.matanim_joint,
                       mnStageSel_804D6C98->x30.shapeanim_joint);
#else
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x30.animjoint,
                       mnStageSel_804D6C98->x30.matanim_joint,
                       mnStageSel_804D6C98->x30.shapeanim_joint);
#endif
    jobj = GET_JOBJ(gobj);
    temp_r3_2 = HSD_MemAlloc(sizeof(struct StageSelUserData));
    GObj_InitUserData(gobj, 4, HSD_Free, temp_r3_2);
    HSD_GObj_SetupProc(gobj, fn_80259D84, 1);
    temp_r3_2->x0 = id;
    temp_r3_2->x4 = 0;
    temp_r3_2->x2 = 0;
    if (id < SSS_ICON_COUNT && mnStageSel_803F06D0[id].x8 >= 2) {
#if defined(TARGET_PC)
        do_anim(jobj, mnStageSel_MexMap != NULL
                          ? (f32) id
                          : 20.0F * mnStageSel_803F06D0[id].x9);
#else
        do_anim(jobj, 20.0F * mnStageSel_803F06D0[id].x9);
#endif
    }
}

void fn_8025A090(HSD_GObj* gobj)
{
    u32 var_r3;
    struct {
        u32 x0;
        u32 x4;
    }* temp_r30;
    HSD_JObj* jobj;

    jobj = GET_JOBJ(gobj);
    temp_r30 = HSD_GObjGetUserData(gobj);
    var_r3 = mnStageSel_804D6CAE;
    if (mnStageSel_803F06D0[mnStageSel_804D6CAE].x8 < 2) {
        var_r3 = SSS_ICON_COUNT;
    }
    if (temp_r30->x0 != var_r3) {
        temp_r30->x0 = var_r3;
        temp_r30->x4 = 0;
#if defined(TARGET_PC)
        if (sss_has_preview((s32) var_r3)) {
#else
        if ((s32) var_r3 < NUM_STAGES) {
#endif
            HSD_JObjReqAnimAll(jobj, 50.0F * mnStageSel_803F06D0[var_r3].x9);
            HSD_JObjAnimAll(jobj);
            HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim,
                            AOBJ_ARG_AOV, 0, 0);
            HSD_JObjSetTranslateX(jobj, 0.0F);
        } else {
            HSD_JObjSetTranslateX(jobj, 100.0F);
        }
    }
    if (temp_r30->x4 < 0x5A) {
        temp_r30->x4++;
        if (temp_r30->x4 == 0x14) {
            HSD_JObjReqAnimAll(jobj,
                               50.0F * mnStageSel_803F06D0[temp_r30->x0].x9);
        }
        if (temp_r30->x4 == 0x45) {
            HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim,
                            AOBJ_ARG_AOV, 0, 0);
        }
    } else if (mnStageSel_804D6CAF) {
        mnStageSel_804D6CAF = 2;
    }
}

void fn_8025A310(HSD_GObj* gobj)
{
    Vec3 sp1C;
    Vec3 sp10;
    f32 temp_f1;
    f32 temp_f2;
    int i;
    HSD_JObj* jobj;
    u32 unused;

    jobj = gobj->hsd_obj;
    if (mnStageSel_804D6CAF != 0) {
        HSD_JObjSetFlags(jobj, JOBJ_HIDDEN);
        return;
    }
    HSD_JObjGetTranslation(jobj, &sp1C);
    sp1C.x = 0.03f * mnStageSel_804D6CAC + sp1C.x;
    if (-27.0F > sp1C.x) {
        sp1C.x = -27.0F;
    }
    if (27.0F < sp1C.x) {
        sp1C.x = 27.0F;
    }
    sp1C.y = 0.03f * mnStageSel_804D6CAD + sp1C.y;
    if (-19.0F > sp1C.y) {
        sp1C.y = -19.0F;
    }
    if (19.0F < sp1C.y) {
        sp1C.y = 19.0F;
    }

    HSD_JObjSetTranslate(jobj, &sp1C);
    lb_8000B1CC(jobj, NULL, &sp1C);
    for (i = 0; i < SSS_ICON_COUNT; i++) {
        if (mnStageSel_803F06D0[i].x8 != 0) {
            lb_8000B1CC(mnStageSel_803F06D0[i].x0, NULL, &sp10);
            temp_f2 = sp10.x;
            temp_f1 = mnStageSel_803F06D0[i].xC;
            if (temp_f2 - temp_f1 < sp1C.x && temp_f2 + temp_f1 > sp1C.x) {
                if (sp10.y - mnStageSel_803F06D0[i].x10 < sp1C.y &&
                    sp10.y + mnStageSel_803F06D0[i].x10 > sp1C.y)
                {
                    mnStageSel_804D6CAE = i;
                    return;
                }
            }
        }
    }
}

void fn_8025A560(HSD_GObj* gobj)
{
    struct StageSelUserData {
        int x0;
    }* temp_r30;
    Vec3 sp10;
    HSD_JObj* jobj = GET_JOBJ(gobj);
    temp_r30 = HSD_GObjGetUserData(gobj);

    /* Retail hides the cursor on the odd half of an empty icon pair. m-ex's
     * grid has no pairs; a hidden icon is simply never hovered. */
#if defined(TARGET_PC)
    if (!mnStageSel_MexActive && mnStageSel_804D6CAE < 0x16 &&
        (mnStageSel_804D6CAE & 1) &&
        mnStageSel_803F06D0[mnStageSel_804D6CAE].x8 == 0)
    {
#else
    if (mnStageSel_804D6CAE < 0x16 && (mnStageSel_804D6CAE & 1) &&
        mnStageSel_803F06D0[mnStageSel_804D6CAE].x8 == 0)
    {
#endif
        HSD_JObjSetTranslateX(jobj, 100.0F);
    } else if (mnStageSel_804D6CAE < SSS_ICON_COUNT) {
        lb_8000B1CC(mnStageSel_803F06D0[mnStageSel_804D6CAE].x0, NULL, &sp10);
        HSD_JObjSetTranslateX(jobj, sp10.x);
        HSD_JObjSetTranslateY(jobj, sp10.y);
        HSD_JObjSetScaleX(jobj, mnStageSel_803F06D0[mnStageSel_804D6CAE].x14);
        HSD_JObjSetScaleY(jobj, mnStageSel_803F06D0[mnStageSel_804D6CAE].x18);
    } else {
        HSD_JObjSetTranslateX(jobj, 100.0F);
    }
    if (mnStageSel_804D6CAF != 0) {
        HSD_JObjReqAnimAll(jobj, 0.0F);
        HSD_JObjAnimAll(jobj);
        return;
    }
    if (++temp_r30->x0 >= 10) {
        temp_r30->x0 = 0;
        HSD_JObjReqAnimAll(jobj, 0.0F);
        HSD_JObjAnimAll(jobj);
    }
}

void fn_8025A91C(HSD_GObj* gobj)
{
    HSD_JObj* jobj = GET_JOBJ(gobj);
    if (++mnStageSel_804D6CA8 >= 0xFA) {
        mnStageSel_804D6CA8 = 0;
        HSD_JObjReqAnimAll(jobj, 0.0F);
        HSD_JObjAnimAll(jobj);
    }
}

void fn_8025A974(HSD_GObj* gobj, int unused)
{
    HSD_FogSet(gobj->hsd_obj);
}

static const Vec3 mnStageSel_803B8550 = { 0, -13, 0 };

static inline void make_stage_icon(HSD_JObj** out)
{
    HSD_GObj* gobj;
    HSD_JObj* jobj;
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x40.joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x83);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 3);
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x40.animjoint,
                       mnStageSel_804D6C98->x40.matanim_joint,
                       mnStageSel_804D6C98->x40.shapeanim_joint);
    *out = GET_JOBJ(gobj);
}

static inline void attach_menu_model(HSD_GObj* gobj)
{
    HSD_JObj* jobj;
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->xA0.joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x80);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 0);
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->xA0.animjoint,
                       mnStageSel_804D6C98->xA0.matanim_joint,
                       mnStageSel_804D6C98->xA0.shapeanim_joint);
}

static inline void make_bg_model(HSD_JObj** out)
{
    HSD_GObj* gobj;
    HSD_JObj* jobj;
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x50.joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x82);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 0);
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x50.animjoint,
                       mnStageSel_804D6C98->x50.matanim_joint,
                       mnStageSel_804D6C98->x50.shapeanim_joint);
    *out = gobj->hsd_obj;
}

static inline void make_icon_root(HSD_JObj** icons)
{
    HSD_GObj* gobj;
    HSD_JObj* jobj;
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x90.joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
    GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x82);
    HSD_GObj_SetupProc(gobj, mn_8022EAE0, 4);
    HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x90.animjoint,
                       mnStageSel_804D6C98->x90.matanim_joint,
                       mnStageSel_804D6C98->x90.shapeanim_joint);
    icons[0] = GET_JOBJ(gobj)->child;
    HSD_JObjReqAnimAll(icons[0], 0.0F);
    HSD_JObjAnimAll(icons[0]);
}

static inline HSD_JObj* get_jobj(HSD_GObj* gobj)
{
    HSD_JObj* jobj = GET_JOBJ(gobj);
    return jobj;
}

void mnStageSel_Scene_OnEnter(void* arg0)
{
    HSD_JObj* spDC[0x13];
    u8 _[0xDC - 0xD8];
    Vec3 spCC;

    int i;
    struct {
        HSD_CObjDesc* unk0;
        HSD_LightDesc* unk4;
        HSD_LightDesc* unk8;
        HSD_FogDesc* unkC;
        struct mnStageSel_804D6C98_t x10;
    }* temp_r3;

    PAD_STACK(0xDC - 0x50);

    sss_data = (SSSData*) arg0;

    if (sss_data->force_stage_id < 0) {
        if (lbLang_IsSavedLanguageUS() != 0) {
            mnStageSel_804D6C94 = lbArchive_LoadArchive("MnSlMap.usd");
        } else {
            mnStageSel_804D6C94 = lbArchive_LoadArchive("MnSlMap.dat");
        }
        temp_r3 = HSD_ArchiveGetPublicAddress(mnStageSel_804D6C94,
                                              "MnSelectStageDataTable");
        MenMain_cam = temp_r3->unk0;
        mnStageSel_804D6C98 = &temp_r3->x10;
#if defined(TARGET_PC)
        mnStageSel_MexSetup(mnStageSel_804D6C94);
#endif
        mnStageSel_804D6CAF = 0;
        mnStageSel_804D6CA0 = 0;
        mnStageSel_804D6CAC = 0;
        mnStageSel_804D6CAD = 0;
        mnStageSel_804D6CAE = SSS_ICON_COUNT;
        mnStageSel_804D50A0 = sss_data->unk_stage - 1;
        mnStageSel_804D6CA4 = 0x14;

        {
            HSD_GObj* gobj = mnStageSel_804D6C9C = GObj_Create(2, 3, 0x80);
            HSD_CObj* cobj = HSD_CObjLoadDesc(MenMain_cam);
            HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj);
            GObj_SetupGXLinkMax(gobj, HSD_GObj_803910D8, 0);
            gobj->gxlink_prios = 0x11;
            HSD_GObj_SetupProc(gobj, mn_8022BA1C, 5);
        }

        {
            HSD_GObj* gobj;
            HSD_LObj* lobj1;
            HSD_LObj* lobj2;
            gobj = GObj_Create(3, 4, 0x80);
            lobj1 = HSD_LObjLoadDesc(temp_r3->unk4);
            lobj2 = HSD_LObjLoadDesc(temp_r3->unk8);
            HSD_LObjSetNext(lobj1, lobj2);
            HSD_GObjObject_80390A70(gobj, (u8) HSD_GObj_LightKind, lobj1);
            GObj_SetupGXLink(gobj, HSD_GObj_LObjCallback, 0, 0x80);
        }

        {
            HSD_GObj* gobj = GObj_Create(0xE, 0xF, 0);
            HSD_Fog* fog = HSD_FogLoadDesc(temp_r3->unkC);
            HSD_GObjObject_80390A70(gobj, HSD_GObj_FogKind, fog);
            GObj_SetupGXLink(gobj, fn_8025A974, 0, 0x80);
        }

        {
            HSD_JObj* jobj2;
            HSD_GObj* gobj;
            gobj = GObj_Create(4, 5, 0x80);
            attach_menu_model(gobj);
            {
                HSD_GObj* g = gobj;
                jobj2 = GET_JOBJ(g);
                HSD_GObj_SetupProc(g, fn_8025A91C, 0);
                HSD_JObjReqAnimAll(jobj2, 0.0F);
                HSD_JObjAnimAll(jobj2);
            }
        }

        {
            HSD_JObj* temp_r22_4;
            make_bg_model(&temp_r22_4);
            HSD_JObjReqAnimAll(temp_r22_4, 0.0F);
            HSD_JObjAnimAll(temp_r22_4);
        }

#if defined(TARGET_PC)
        if (mnStageSel_MexActive) {
            mnStageSel_MexInitIcons();
        } else
#endif
        {
            make_icon_root(spDC);

            for (i = 0; i < 0x12; i++) {
                spDC[i + 1] = spDC[i]->next;
                HSD_JObjReqAnimAll(spDC[i + 1], 0.0F);
                HSD_JObjAnimAll(spDC[i + 1]);
            }

            for (i = 0; i < NUM_STAGES; i++) {
                mnStageSel_803F06D0[i].x8 =
                    gm_80164430(mnStageSel_803F06D0[i].stkind) ? 2 : 1;
            }

            for (i = 0; i <= 0xA; i++) {
                HSD_JObj* temp_r22_6;
                HSD_JObj* jobj;
                make_stage_icon(&temp_r22_6);
                jobj = temp_r22_6;
                lb_8000C1C0(jobj, spDC[i]);
                mnStageSel_803F06D0[i * 2].x0 = temp_r22_6->child->next;
                switch (mnStageSel_803F06D0[i * 2].x8) {
                case 0:
                    HSD_JObjSetFlags(mnStageSel_803F06D0[i * 2].x0,
                                     JOBJ_HIDDEN);
                    break;
                case 1:
                    HSD_JObjReqAnimAllByFlags(mnStageSel_803F06D0[i * 2].x0,
                                              0x10, 1.0F);
                    break;
                default:
                    HSD_JObjReqAnimAllByFlags(
                        mnStageSel_803F06D0[i * 2].x0, 0x10,
                        mnStageSel_803F06D0[i * 2].x9 / 2 + 2);
                    break;
                }
                mnStageSel_803F06D0[i * 2 + 1].x0 = temp_r22_6->child;
                switch (mnStageSel_803F06D0[i * 2 + 1].x8) {
                case 0:
                    HSD_JObjSetFlags(mnStageSel_803F06D0[i * 2 + 1].x0,
                                     JOBJ_HIDDEN);
                    break;
                case 1:
                    HSD_JObjReqAnimAllByFlags(
                        mnStageSel_803F06D0[i * 2 + 1].x0, 0x10, 1.0F);
                    break;
                default:
                    HSD_JObjReqAnimAllByFlags(
                        mnStageSel_803F06D0[i * 2 + 1].x0, 0x10,
                        mnStageSel_803F06D0[i * 2 + 1].x9 / 2 + 2);
                    break;
                }
                HSD_JObjAnimAll(jobj);
                HSD_ForeachAnim(jobj, JOBJ_TYPE, TOBJ_MASK, HSD_AObjStopAnim,
                                AOBJ_ARG_AOV, 0, 0);
            }

            for (i = 0xB; i <= 0xF; i++) {
                HSD_JObj* jobj;
                HSD_GObj* gobj = GObj_Create(4, 5, 0x80);
                s32 temp_r22_7;
                HSD_JObj* temp_r23_3;
                jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x20.joint);
                HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
                GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x83);
                HSD_GObj_SetupProc(gobj, mn_8022EAE0, 3);
                HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x20.animjoint,
                                   mnStageSel_804D6C98->x20.matanim_joint,
                                   mnStageSel_804D6C98->x20.shapeanim_joint);
                temp_r23_3 = gobj->hsd_obj;
                lb_8000C1C0(temp_r23_3, spDC[i]);
                mnStageSel_803F06D0[i + 13].x0 = temp_r23_3;
                switch (mnStageSel_803F06D0[i + 13].x8) {
                case 1:
                    mnStageSel_803F06D0[i + 13].x8 = 0;
                    /* fallthrough */
                case 0:
                    HSD_JObjSetFlagsAll(temp_r23_3, JOBJ_HIDDEN);
                    break;
                default:
                    temp_r22_7 = mnStageSel_803F06D0[i + 13].x9 - 0x16;
                    do_anim(temp_r23_3, temp_r22_7);
                    break;
                }
            }

            {
                HSD_JObj* jobj;
                HSD_GObj* gobj = GObj_Create(4, 5, 0x80);
                HSD_JObj* temp_r22_8;
                jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x10.joint);
                HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
                GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x83);
                HSD_GObj_SetupProc(gobj, mn_8022EAE0, 3);
                HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x10.animjoint,
                                   mnStageSel_804D6C98->x10.matanim_joint,
                                   mnStageSel_804D6C98->x10.shapeanim_joint);
                temp_r22_8 = gobj->hsd_obj;
                lb_8000C1C0(temp_r22_8, spDC[0x10]);
                do_anim(temp_r22_8, 2);
                mnStageSel_803F06D0[NUM_STAGES].x0 = temp_r22_8;
            }

            for (i = 0x11; i <= 0x12; i++) {
                HSD_JObj* jobj;
                HSD_AnimJoint* animjoint;
                HSD_GObj* gobj = GObj_Create(4, 5, 0x80);
                s32 temp_r22_9;
                HSD_JObj* temp_r23_6;
                jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x0.joint);
                HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
                GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x83);
                HSD_GObj_SetupProc(gobj, mn_8022EAE0, 3);
                animjoint = mnStageSel_804D6C98->x0.animjoint;
                HSD_JObjAddAnimAll(jobj, animjoint,
                                   mnStageSel_804D6C98->x0.matanim_joint,
                                   mnStageSel_804D6C98->x0.shapeanim_joint);

                temp_r23_6 = gobj->hsd_obj;
                lb_8000C1C0(temp_r23_6, spDC[i]);
                mnStageSel_803F06D0[i + 5].x0 = temp_r23_6;
                switch (mnStageSel_803F06D0[i + 5].x8) {
                case 1:
                    mnStageSel_803F06D0[i + 5].x8 = 0;
                    /* fallthrough */
                case 0:
                    HSD_JObjSetFlagsAll(temp_r23_6, JOBJ_HIDDEN);
                    break;
                default:
                    temp_r22_9 = mnStageSel_803F06D0[i + 5].x9 - 0x14;
                    do_anim(temp_r23_6, temp_r22_9);
                    break;
                }
            }
        }

        {
            HSD_JObj* jobj;
            HSD_GObj* gobj;
            gobj = GObj_Create(4, 5, 0x80);
            jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x80.joint);
            HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
            GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x86);
            HSD_GObj_SetupProc(gobj, mn_8022EAE0, 2);
            HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x80.animjoint,
                               mnStageSel_804D6C98->x80.matanim_joint,
                               mnStageSel_804D6C98->x80.shapeanim_joint);

            {
                HSD_JObj* jobj2;
                HSD_GObj* g;
                jobj = get_jobj(gobj);
                spCC = mnStageSel_803B8550;
                g = gobj;
                jobj2 = jobj;
                HSD_GObj_SetupProc(g, fn_8025A310, 2);
                do_anim(jobj2, mnStageSel_804D50A0 + 1);
                HSD_JObjSetTranslate(jobj2, &spCC);
            }
        }

        {
            HSD_JObj* jobj;
            HSD_GObj* gobj;
            gobj = GObj_Create(HSD_GOBJ_CLASS_FIGHTER, 5, 0x80);
            jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x30.joint);
            HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
            GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x84);
            HSD_GObj_SetupProc(gobj, mn_8022EAE0, 1);
            HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x30.animjoint,
                               mnStageSel_804D6C98->x30.matanim_joint,
                               mnStageSel_804D6C98->x30.shapeanim_joint);
            {
                struct StageSelUserData* userdata;
                HSD_JObj* jobj = GET_JOBJ(gobj);
                userdata = HSD_MemAlloc(sizeof(struct StageSelUserData));
                GObj_InitUserData(gobj, 4, HSD_Free, userdata);
                HSD_GObj_SetupProc(gobj, fn_80259D84, 1);
                userdata->x0 = SSS_ICON_COUNT;
                userdata->x4 = 0;
                userdata->x2 = 0;
            }
        }

        {
            HSD_JObj* jobj;
            HSD_GObj* gobj = GObj_Create(4, 5, 0x80);
            jobj = HSD_JObjLoadJoint(mnStageSel_804D6C98->x60.joint);
            HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj);
            GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x81);
            HSD_GObj_SetupProc(gobj, mn_8022EAE0, 1);
            HSD_JObjAddAnimAll(jobj, mnStageSel_804D6C98->x60.animjoint,
                               mnStageSel_804D6C98->x60.matanim_joint,
                               mnStageSel_804D6C98->x60.shapeanim_joint);

            {
                HSD_GObj* g;
                HSD_JObj* jobj;
                struct foo {
                    int x0, x4;
                }* temp_r3_14;
                g = gobj;
                jobj = GET_JOBJ(g);
                temp_r3_14 = HSD_MemAlloc(sizeof(*temp_r3_14));
                GObj_InitUserData(g, 4, HSD_Free, temp_r3_14);
                HSD_GObj_SetupProc(g, fn_8025A090, 1);
                HSD_JObjReqAnimAll(jobj, 0.0F);
                HSD_JObjAnimAll(jobj);
                HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK,
                                HSD_AObjStopAnim, AOBJ_ARG_AOV, 0, 0);
                HSD_JObjSetTranslateX(jobj, 100.0F);
                temp_r3_14->x0 = SSS_ICON_COUNT;
                temp_r3_14->x4 = 0;
            }
        }

        {
            HSD_JObj* jobj;
            HSD_GObj* gobj = GObj_Create(4, 5, 0x80);
            HSD_JObj* temp_r3_15 =
                HSD_JObjLoadJoint(mnStageSel_804D6C98->x70.joint);
            s32* temp_r3_16;
            HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, temp_r3_15);
            GObj_SetupGXLink(gobj, HSD_GObj_JObjCallback, 4, 0x85);
            HSD_GObj_SetupProc(gobj, mn_8022EAE0, 1);
            HSD_JObjAddAnimAll(temp_r3_15, mnStageSel_804D6C98->x70.animjoint,
                               mnStageSel_804D6C98->x70.matanim_joint,
                               mnStageSel_804D6C98->x70.shapeanim_joint);
            jobj = gobj->hsd_obj;
            temp_r3_16 = HSD_MemAlloc(sizeof(*temp_r3_16));
            GObj_InitUserData(gobj, 4, HSD_Free, temp_r3_16);
            *temp_r3_16 = 0;
            HSD_GObj_SetupProc(gobj, fn_8025A560, 1);
            HSD_JObjSetTranslateX(jobj, 100.0F);
        }

        lbAudioAx_80023F28(gmMainLib_8015ECB0());
    }
}

static inline HSD_PadStatus* get_pad(u8 i)
{
    return &HSD_PadCopyStatus[i];
}

/// OnFrame
void mnStageSel_Scene_OnFrame(void)
{
    if (sss_data->force_stage_id >= 0) {
        mnStageSel_804D6CAF = 2;
        sss_data->vs.start.rules.stkind = sss_data->force_stage_id;
        gm_801A4B60();
        return;
    }
    if (sss_data->no_lras == 0 && mn_8022F218()) {
        sfxBack();
        lb_800145F4();
        HSD_GObjFree(mnStageSel_804D6C9C);
        mn_8022F268();
        gm_ChangeGameModeAfterCurrentScene(GM_MENU);
        gm_801A4B60();
        return;
    }
    if (mnStageSel_804D50A0 < 0) {
        mnStageSel_804D6CA0 = 0;
        mnStageSel_804D6CA0 |= HSD_PadCopyStatus[0].trigger;
        mnStageSel_804D6CA0 |= HSD_PadCopyStatus[1].trigger;
        mnStageSel_804D6CA0 |= HSD_PadCopyStatus[2].trigger;
        mnStageSel_804D6CA0 |= HSD_PadCopyStatus[3].trigger;
        {
            int i;
            for (i = 0; i < 4; i++) {
                mnStageSel_804D6CAC = get_pad(i)->stickX;
                mnStageSel_804D6CAD = get_pad(i)->stickY;
                if (get_pad(i)->stickX < -0x1E || get_pad(i)->stickX > +0x1E ||
                    get_pad(i)->stickY < -0x1E || get_pad(i)->stickY > +0x1E)
                {
                    break;
                }
            }
        }
    } else {
        mnStageSel_804D6CA0 = get_pad(mnStageSel_804D50A0)->trigger;
        mnStageSel_804D6CAC = get_pad(mnStageSel_804D50A0)->stickX;
        mnStageSel_804D6CAD = get_pad(mnStageSel_804D50A0)->stickY;
    }
    if (mnStageSel_804D6CAC < -0x1E) {
        mnStageSel_804D6CAC += 0x1E;
    } else if (mnStageSel_804D6CAC > 0x1E) {
        mnStageSel_804D6CAC -= 0x1E;
    } else {
        mnStageSel_804D6CAC = 0;
    }
    if (mnStageSel_804D6CAD < -0x1E) {
        mnStageSel_804D6CAD += 0x1E;
    } else if (mnStageSel_804D6CAD > 0x1E) {
        mnStageSel_804D6CAD -= 0x1E;
    } else {
        mnStageSel_804D6CAD = 0;
    }
    if (mnStageSel_804D6CA4 != 0) {
        mnStageSel_804D6CA4 -= 1;
        return;
    }
#if defined(TARGET_PC)
    if (sss_data->x1 == 0 && mnStageSel_804D6CAF == 0 &&
        mnStageSel_MexChangePage())
    {
        return;
    }
#endif
    if (sss_data->x1 == 0 && (mnStageSel_804D6CA0 & 0x200) &&
        mnStageSel_804D6CAF == 0)
    {
        sfxBack();
        gm_801A4B60();
    }
    if (mnStageSel_804D6CAF == 2) {
        sss_data->vs.start.rules.stkind =
            mnStageSel_803F06D0[mnStageSel_804D6CAE].stkind;
        gm_801A4B60();
    }
}

void mnStageSel_Scene_OnExit(UNUSED void* exit_data)
{
    if (mnStageSel_804D6C94 != NULL) {
        lbArchive_80016EFC(mnStageSel_804D6C94);
        mnStageSel_804D6C94 = NULL;
    }
#if defined(TARGET_PC)
    /* mexMapData lives INSIDE the archive just freed, and OnEnter only
     * rebuilds it on the interactive path (force_stage_id < 0). Drop it here
     * or a forced-stage visit would leave a pointer into freed memory behind.
     */
    mnStageSel_MexMap = NULL;
    mnStageSel_MexActive = false;
    mnStageSel_NumStages = NUM_STAGES_RETAIL;
    mnStageSel_IconCount = SSS_ICON_COUNT_RETAIL;
#endif
    {
        SSSData* sss = sss_data;
        sss->start_game = mnStageSel_804D6CAF == 2 ? true : false;
        if (sss->start_game) {
            PreloadedGameModeState* cache = lbDvd_GetPreloadCacheScene();
            cache->game_cache.stkind = sss->vs.start.rules.stkind;
            lbDvd_80018254();
        }
    }
}

int mnSelStageRandom(void)
{
    return mnStageSel_803F06D0[mnStageSel_802599EC()].stkind;
}

int mnStageSel_8025BC08(int idx)
{
    return mnStageSel_803F06D0[idx].stkind;
}
