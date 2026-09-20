#include "gmscene.h"

#include "gm_1A36.h"
#include "gm_unsplit.h"
#include "gmmain_lib.h"
#include "gmscdata.h"
#include <dolphin/os/OSThread.h>
#include <melee/db/db.h>
#include <melee/if/ifcoget.h>
#include <melee/lb/lb_013B.h>
#include <melee/lb/lb_0195.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/lb/lbcardgame.h>
#include <melee/lb/lbheap.h>
#include <sysdolphin/baselib/class.h>
#include <sysdolphin/baselib/controller.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/hsd_3924.h>
#include <sysdolphin/baselib/hsd_392C.h>
#include <sysdolphin/baselib/initialize.h>
#include <sysdolphin/baselib/leak.h>
#include <sysdolphin/baselib/perf.h>
#include <sysdolphin/baselib/sobjlib.h>
#include <melee/mn/mnmain.h>
#include "gmscenelaunch.h"
#if defined(TARGET_PC)
#include <melee/if/textdraw.h>
#include <melee/if/textlib.h>
#include <melee/if/types.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/gobjobject.h>
#include <sysdolphin/baselib/gobjplink.h>
#include <sysdolphin/baselib/memory.h>
#include <sysdolphin/baselib/tobj.h>
#include <dolphin/gx.h>
#include <dolphin/mtx.h>
#include <dolphin/os.h>
#endif

/* 479D30 */ static HSD_GObjLibInitDataType gobj_init_data;
/* 479D58 */ static struct gm_80479D58_t gm_80479D58;
/* 4D672C */ HSD_GObj* gm_804D672C;
/* 4D6728 */ UNK_T gm_804D6728;
/* 4D6724 */ void (*gm_804D6724)(void);
/* 4D6720 */ struct GameSceneInfo* gm_804D6720;

static u64 gm_803DA888[8] = {
    0, 0x82FFFA, 0, 0x8EFFFA, 0x800FFA, 0x808FFA, 0x800FFA, 0,
};

u64 gm_803DA8C8[2] = { -1, -1 };

bool gm_GetDbPauseFlag(int bit)
{
    return gm_80479D58.unk_10.x0 & (1ULL << bit);
}

int gm_801A4624(void)
{
    return gm_80479D58.unk_10.x0;
}

void gm_SetDbPauseFlag(int bit)
{
    gm_80479D58.unk_10.x0 |= 1ULL << bit;
}

void gm_ClearDbPauseFlag(int bit)
{
    gm_80479D58.unk_10.x0 &= ~(1ULL << bit);
}

bool gm_801A46B8(int bit)
{
    return gm_80479D58.unk_10.x2 & (1ULL << bit);
}

bool fn_801A46F4(void)
{
    int i;
    for (i = 0; i < PAD_MAX_CONTROLLERS; i++) {
        HSD_PadStatus* pad = &HSD_PadMasterStatus[(u8) i];
        if (pad->err == 0 && (pad->trigger & HSD_PAD_DPADUP) &&
            (pad->button & HSD_PAD_X))
        {
            return true;
        }
    }
    return false;
}

bool fn_801A47E4(void)
{
    int i;
    for (i = 0; i < PAD_MAX_CONTROLLERS; i++) {
        HSD_PadStatus* pad = &HSD_PadMasterStatus[(u8) i];
        if (pad->err == 0 && (pad->trigger & HSD_PAD_Z)) {
            return true;
        }
    }
    return false;
}

u64 gm_801A48A4(u8 arg0)
{
    int i;
    u64 result = 0;

    for (i = 0; i < ARRAY_SIZE(gm_803DA888); i++) {
        if (arg0 & 1) {
            result |= gm_803DA888[i];
        }
        arg0 >>= 1;
    }

    return result;
}

void gm_801A4970(struct gm_DbPauseInputHandlers* db_input)
{
    HSD_PadStatus* temp_r3;
    s8 var_r26;
    s8* temp_r4;
    u64 temp_ret;
    int i;
    PAD_STACK(8);

    var_r26 = 0;
    for (i = 0; i < PAD_MAX_CONTROLLERS; i++) {
        temp_r3 = &HSD_PadMasterStatus[(u8) i];
        if ((temp_r3->trigger & HSD_PAD_DPADRIGHT) &&
            (temp_r3->button & HSD_PAD_X))
        {
            lbHeap_80015DF8();
            OSReport("[hsdDumpClassStat] -- Report --\n");
            hsdDumpClassStat(NULL, 0, 1);
            OSReport("\n");
            OSReport("[HSD_ObjDumpStat] -- Report --\n");
            HSD_ObjDumpStat();
            OSReport("\n");
            db_PrintEntityCounts();
            db_PrintThreadInfo();
            HSD_Leak_80387DF8(0);
            if (gm_804D6728 != NULL) {
                gm_801653C8(gm_804D6728);
                gm_804D6728 = NULL;
            } else {
                gm_804D6728 = gm_80165388(0x19, 0x3F, 0, 0xFE);
                if (gm_804D6724 != NULL) {
                    gm_804D6724();
                }
            }
        }
    }

    if (db_input->check_pause != NULL && db_input->check_pause()) {
        if (gm_GetDbPauseFlag(0)) {
            gm_80479D58.unk_10.x0 &= ~1;
        } else {
            gm_80479D58.unk_10.x0 |= 1;
        }
    }
    if (gm_GetDbPauseFlag(0)) {
        if (db_input->check_framestep != NULL && db_input->check_framestep()) {
            gm_80479D58.unk_10.x2 |= 1;
        }
    }
}

void gm_SetDbPauseInputHandlers(Predicate check_db_pause,
                                Predicate check_db_framestep)
{
    gm_80479D58.unk_10.db_input.check_pause = check_db_pause;
    gm_80479D58.unk_10.db_input.check_framestep = check_db_framestep;
}

void gm_801A4B1C(void)
{
    gm_SetDbPauseInputHandlers(fn_801A46F4, fn_801A47E4);
}

void gm_SetPreGObjProcCallback(Event cb)
{
    gm_80479D58.unk_10.pre_gobj_proc = cb;
}

void gm_801A4B50(int arg0)
{
    gm_80479D58.unk_10.unk_34 = arg0;
}

void gm_801A4B60(void)
{
    gm_80479D58.unk_C = 1;
}

void gm_801A4B74(void)
{
    gm_80479D58.unk_C = 2;
}

void gm_801A4B88(struct GameSceneInfo* info)
{
    gm_804D6720 = info;
}

/// @brief returns a pointer to the current scenes enter data
void* gm_GetCurrentSceneEnterData(void)
{
    return gm_804D6720->enter_data;
}

/// @brief returns a pointer to the current scenes exit data
void* gm_GetCurrentSceneExitData(void)
{
    return gm_804D6720->exit_data;
}

u32 gm_801A4BA8(void)
{
    return gm_80479D58.unk_0;
}

u32 gm_801A4BB8(void)
{
    return gm_80479D58.unk_8;
}

HSD_GObj* gm_801A4BC8(void)
{
    return gm_804D672C;
}

void fn_801A4BD0(HSD_GObj* gobj) {}

void gm_801A4BD4(void)
{
    PAD_STACK(0x18);

    gm_SetDbPauseInputHandlers(fn_801A46F4, fn_801A47E4);
    gm_SetPreGObjProcCallback(NULL);
    gm_801A4B50(0);

    lb_80019880(OSSecondsToTicks(1.0F / GM_FPS));
    HSD_GObjSetInitDefaults(&gobj_init_data);
    gobj_init_data.gproc_pri_max = 0x18;
    HSD_SObjLib_804D7960 =
        HSD_GObj_803912A8(&gobj_init_data, &HSD_SObjLib_8040C3A4);
    HSD_SObjLib_803A44A4();
    gobj_init_data.unk_2 = &gm_80479D58.unk_10.unk_28;
    HSD_GObjInit(&gobj_init_data);
    hsd_80392474();
    un_802FF78C();
    gm_804D672C = GObj_Create(14, 0, 0);
    if (gm_804D672C != NULL) {
        HSD_GObj_SetupProc(gm_804D672C, fn_801A4BD0, 0);
    }
    gm_804D6728 = NULL;
    gm_804D6724 = NULL;
    gm_801A3E88();
    lbAudioAx_8002835C();
    lb_80014534();
}

#if defined(TARGET_PC)

/* ---- The loading screen ---------------------------------------------------------------------
 *
 * WHAT IS ACTUALLY WRONG. Booting into a match gives a black period in which the stage and the
 * fighters arrive part by part. It is NOT stutter - the frame loop stays at full speed the whole
 * time. Aurora creates a WebGPU render pipeline the first time a draw needs one, compiles it on a
 * worker thread, and SKIPS that draw until it is ready; so the first frames of a cold scene are
 * missing whatever has not finished compiling. gw_Gfx_PipelinesPending / PipelinesCreated report
 * that directly (pc/platform/gw_runtime.c).
 *
 * WHY THE HOLD IS HERE AND NOT IN A SCENE OF ITS OWN. The only thing that compiles a match's
 * pipelines is drawing that match, so a separate loading scene in front of it would warm its own
 * pipelines and none of the ones that matter. What this does instead is let the scene the game
 * already has - GS_VS, GS_SUDDEN_DEATH, GS_TRAINING - enter, render and queue its pipelines with
 * the match's own clock stopped, behind a panel. Nothing new is scheduled, nothing is preloaded,
 * and no permutation has to be guessed: the frames that warm the renderer are the real ones.
 * Concretely, the scene's on_frame is skipped, so the match state machine does not advance; the
 * rest of the loop - pads, gobj procs, HSD_StartRender - runs untouched.
 *
 * WHEN IT LETS GO. `pending == 0` means nothing on its own, because it is also what the counter
 * reads before the scene's first draw has asked for anything. So all three of these must hold:
 *
 *   - at least LOADSCREEN_MIN_FRAMES frames have been drawn, so "nothing in flight" is an answer
 *     about this scene rather than about the moment before it started;
 *   - nothing is in flight now;
 *   - createdPipelines has not moved for LOADSCREEN_SETTLE_FRAMES frames. Half a second, because
 *     a compile finishing un-skips a draw that can itself ask for the NEXT pipeline, so the count
 *     rises in chains; the wait has to outlast a link of that chain, and a queue-to-ready is tens
 *     of milliseconds. Shorter than this and the loader steps aside mid-chain, which is the whole
 *     bug wearing a panel.
 *
 * And a wall-clock ceiling on top of all of it, because a machine that never settles - a driver
 * compiling in the background, a GPU being shared - must still boot into its match.
 *
 * THE ART IS OPTIONAL. MELEE_MENUTEX_DIR names a directory of .gxtex files (pc/tools/png2gx.py);
 * with it unset every GxTex_Open fails, no sprite is placed, and the hold plus the caption still
 * do their job. Nothing on the disc is involved either way.
 *
 * WHY SObjs ON THE GXLinkMax LIST: see mnCharSel_MenuTexSetup in src/melee/mn/mncharsel.c, which
 * is the worked example this follows. One gobj carries all of them; HSD_SObjLib_803A49E0 walks
 * its list in ascending priority, so 0x10/0x20/0x30/0x40 is back-to-front. Render priority 10 on
 * the max list puts the panel above the match and its HUD (priorities 0..9) and below the
 * DevText camera (11), which is how the caption lands on top of the panel rather than under it.
 */

/* Native entry points. gwtool prefixes every game symbol with `gw_`, so the unprefixed name here
 * binds to the shim of the same name in pc/platform/gw_runtime.c. */
extern int Gfx_PipelinesPending(void);
extern int Gfx_PipelinesCreated(void);
extern int Gfx_LoadScreenEnabled(void);
extern int GxTex_Open(const char* name);
extern int GxTex_Width(int handle);
extern int GxTex_Height(int handle);
extern int GxTex_Format(int handle);
extern int GxTex_ImageSize(int handle);
extern int GxTex_TlutSize(int handle);
extern int GxTex_TlutFormat(int handle);
extern int GxTex_TlutEntries(int handle);
extern void GxTex_CopyImage(int handle, void* dst);
extern void GxTex_CopyTlut(int handle, void* dst);
extern void GxTex_Close(int handle);

#define LOADSCREEN_MIN_FRAMES 20
#define LOADSCREEN_SETTLE_FRAMES 30
#define LOADSCREEN_CEILING_SECONDS 10
#define LOADSCREEN_DOT_FRAMES 15

enum {
    LS_PANEL,
    LS_CORNER_TL,
    LS_CORNER_TR,
    LS_CORNER_BL,
    LS_CORNER_BR,
    LS_EDGE_H,
    LS_EDGE_V,
    LS_CURSOR,
    LS_TEX_MAX,
};

typedef struct mnLoadScreenTex {
    HSD_ImageDesc image;
    HSD_Tlut tlut;
    HSD_SObjDesc desc;
    int loaded;
} mnLoadScreenTex;

static mnLoadScreenTex mnLoadScreen_tex[LS_TEX_MAX];
static HSD_GObj* mnLoadScreen_gobj;
static DevText* mnLoadScreen_text;
static char mnLoadScreen_textbuf[2 * 16];
static int mnLoadScreen_holding;
static int mnLoadScreen_frames;
static int mnLoadScreen_settled;
static int mnLoadScreen_created;
static OSTime mnLoadScreen_started;

static void mnLoadScreen_Draw(HSD_GObj* gobj, int pass)
{
    Mtx44 proj;
    Mtx view;

    /* 640x480 in screen pixels with Y growing downwards, which is the space HSD_SObj's quad is
       written in: it emits GXPosition2f32(x, -y). */
    GXSetViewport(0.0F, 0.0F, 640.0F, 480.0F, 0.0F, 1.0F);
    GXSetScissor(0, 0, 640, 480);
    MTXOrtho((MtxPtr) proj, 0.0F, -480.0F, 0.0F, 640.0F, 0.0F, 2.0F);
    GXSetProjection(proj, GX_ORTHOGRAPHIC);
    view[0][0] = 1.0F; view[0][1] = 0.0F; view[0][2] = 0.0F; view[0][3] = 0.0F;
    view[1][0] = 0.0F; view[1][1] = 1.0F; view[1][2] = 0.0F; view[1][3] = 0.0F;
    view[2][0] = 0.0F; view[2][1] = 0.0F; view[2][2] = 1.0F; view[2][3] = -1.0F;
    GXLoadPosMtxImm(view, GX_PNMTX0);
    GXSetCurrentMtx(GX_PNMTX0);

    /* The sprite draw sets its own TEV stages, vertex descriptor and blend mode but inherits the
       texgen and the colour channel. A TEV stage naming GX_COLOR0A0 with no channel enabled is
       undefined rather than merely unused, so the channel is a constant white here. */
    GXSetNumChans(1);
    GXSetChanCtrl(GX_COLOR0A0, GX_DISABLE, GX_SRC_REG, GX_SRC_REG, GX_LIGHT_NULL,
                  GX_DF_NONE, GX_AF_NONE);
    {
        GXColor white = { 255, 255, 255, 255 };
        GXSetChanMatColor(GX_COLOR0A0, white);
    }
    GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY,
                      GX_DISABLE, GX_PTIDENTITY);

    HSD_SObjLib_803A49E0(gobj, pass);
}

/* The bytes are copied into the game heap because that is what an HSD_ImageDesc points at and
   what GXInitTexObj records; the loader's own copy is released straight away. */
static void mnLoadScreen_Load(int slot, const char* name)
{
    mnLoadScreenTex* t = &mnLoadScreen_tex[slot];
    void* bits;
    void* lut;
    int handle;
    int tlut_fmt;

    t->loaded = 0;
    t->image.image_ptr = NULL;
    t->tlut.lut = NULL;
    t->desc.tlut = NULL;

    handle = GxTex_Open(name);
    if (handle < 0) {
        return;
    }
    bits = HSD_MemAlloc(GxTex_ImageSize(handle));
    if (bits == NULL) {
        GxTex_Close(handle);
        return;
    }
    GxTex_CopyImage(handle, bits);

    t->image.image_ptr = bits;
    t->image.width = (u16) GxTex_Width(handle);
    t->image.height = (u16) GxTex_Height(handle);
    t->image.format = (GXTexFmt) GxTex_Format(handle);
    t->image.mipmap = 0;
    t->image.minLOD = 0.0F;
    t->image.maxLOD = 0.0F;
    t->desc.image = &t->image;

    tlut_fmt = GxTex_TlutFormat(handle);
    if (tlut_fmt >= 0) {
        lut = HSD_MemAlloc(GxTex_TlutSize(handle));
        if (lut == NULL) {
            HSD_Free(bits);
            t->image.image_ptr = NULL;
            GxTex_Close(handle);
            return;
        }
        GxTex_CopyTlut(handle, lut);
        t->tlut.lut = lut;
        t->tlut.fmt = (GXTlutFmt) tlut_fmt;
        t->tlut.n_entries = (u16) GxTex_TlutEntries(handle);
        /* GX_TLUT0, because the sprite draw hardcodes GXLoadTlut(..., GX_TLUT0) for a CI
           texture. A different name here would load the palette into a bank nothing samples. */
        t->tlut.tlut_name = GX_TLUT0;
        t->desc.tlut = &t->tlut;
    }
    GxTex_Close(handle);
    t->loaded = 1;
}

/* w and h are the rectangle wanted on the 640x480 screen; the scale comes from the texture's own
   size, so the same layout holds whether the art was converted at 1x or at 2x. */
static void mnLoadScreen_Place(int slot, f32 x, f32 y, f32 w, f32 h, u8 priority)
{
    mnLoadScreenTex* t = &mnLoadScreen_tex[slot];
    HSD_SObj* sobj;

    if (!t->loaded || mnLoadScreen_gobj == NULL) {
        return;
    }
    sobj = HSD_SObjLib_803A477C(mnLoadScreen_gobj, &t->desc, GX_CLAMP, GX_CLAMP, priority, 0);
    if (sobj == NULL) {
        return;
    }
    sobj->x10 = x;
    sobj->x14 = y;
    sobj->x1C = w / (f32) t->image.width;
    sobj->x20 = h / (f32) t->image.height;
}

static void mnLoadScreen_Unload(void)
{
    int i;

    for (i = 0; i < LS_TEX_MAX; i++) {
        mnLoadScreenTex* t = &mnLoadScreen_tex[i];
        if (t->tlut.lut != NULL) {
            HSD_Free(t->tlut.lut);
        }
        if (t->image.image_ptr != NULL) {
            HSD_Free(t->image.image_ptr);
        }
        t->tlut.lut = NULL;
        t->image.image_ptr = NULL;
        t->desc.tlut = NULL;
        t->loaded = 0;
    }
}

static void mnLoadScreen_Release(char* why)
{
    if (!mnLoadScreen_holding) {
        return;
    }
    mnLoadScreen_holding = 0;
    if (mnLoadScreen_text != NULL) {
        DevText_Remove(&mnLoadScreen_text);
        mnLoadScreen_text = NULL;
    }
    if (mnLoadScreen_gobj != NULL) {
        HSD_GObjFree(mnLoadScreen_gobj);
        mnLoadScreen_gobj = NULL;
    }
    mnLoadScreen_Unload();
    OSReport("loadscreen: released (%s) after %d frames, %d pipelines created\n", why,
             mnLoadScreen_frames, Gfx_PipelinesCreated());
}

static void mnLoadScreen_Begin(GameSceneInfo* info)
{
    HSD_GObj* text_gobj;

    mnLoadScreen_holding = 0;
    mnLoadScreen_gobj = NULL;
    mnLoadScreen_text = NULL;
    if (info == NULL || !Gfx_LoadScreenEnabled()) {
        return;
    }
    switch (info->scene_kind) {
    case GS_VS:
    case GS_SUDDEN_DEATH:
    case GS_TRAINING:
        break;
    default:
        return;
    }

    mnLoadScreen_frames = 0;
    mnLoadScreen_settled = 0;
    mnLoadScreen_created = Gfx_PipelinesCreated();
    mnLoadScreen_started = OSGetTime();
    mnLoadScreen_holding = 1;

    mnLoadScreen_gobj = GObj_Create(0xE, 0xF, 0);
    if (mnLoadScreen_gobj != NULL) {
        HSD_GObjObject_80390A70(mnLoadScreen_gobj, HSD_SObjLib_804D7960, NULL);
        GObj_SetupGXLinkMax(mnLoadScreen_gobj, mnLoadScreen_Draw, 10);

        mnLoadScreen_Load(LS_PANEL, "panel_bg");
        mnLoadScreen_Load(LS_CORNER_TL, "frame_corner_tl");
        mnLoadScreen_Load(LS_CORNER_TR, "frame_corner_tr");
        mnLoadScreen_Load(LS_CORNER_BL, "frame_corner_bl");
        mnLoadScreen_Load(LS_CORNER_BR, "frame_corner_br");
        mnLoadScreen_Load(LS_EDGE_H, "frame_edge_h");
        mnLoadScreen_Load(LS_EDGE_V, "frame_edge_v");
        mnLoadScreen_Load(LS_CURSOR, "cursor_hand");

        /* Back to front. The panel is drawn twice from one copy of its texture: once stretched
           over the whole screen, because whatever is behind the loader has to stop being the
           thing the player is looking at, and once at its authored 512x256 as the caption plate
           the text sits on. */
        mnLoadScreen_Place(LS_PANEL, 0.0F, 0.0F, 640.0F, 480.0F, 0x10);
        mnLoadScreen_Place(LS_EDGE_H, 64.0F, 0.0F, 512.0F, 16.0F, 0x20);
        mnLoadScreen_Place(LS_EDGE_H, 64.0F, 464.0F, 512.0F, 16.0F, 0x20);
        mnLoadScreen_Place(LS_EDGE_V, 0.0F, 64.0F, 16.0F, 352.0F, 0x20);
        mnLoadScreen_Place(LS_EDGE_V, 624.0F, 64.0F, 16.0F, 352.0F, 0x20);
        mnLoadScreen_Place(LS_CORNER_TL, 0.0F, 0.0F, 64.0F, 64.0F, 0x20);
        mnLoadScreen_Place(LS_CORNER_TR, 576.0F, 0.0F, 64.0F, 64.0F, 0x20);
        mnLoadScreen_Place(LS_CORNER_BL, 0.0F, 416.0F, 64.0F, 64.0F, 0x20);
        mnLoadScreen_Place(LS_CORNER_BR, 576.0F, 416.0F, 64.0F, 64.0F, 0x20);
        mnLoadScreen_Place(LS_PANEL, 64.0F, 112.0F, 512.0F, 256.0F, 0x30);
        mnLoadScreen_Place(LS_CURSOR, 400.0F, 220.0F, 32.0F, 32.0F, 0x40);
    }

    /* The game's own text system, set up once per scene by gm_801A4BD4 -> un_802FF78C, so there
       is nothing to initialise here. Its camera draws on gx_link 17 after the panel. */
    text_gobj = DevText_GetGObj();
    if (text_gobj != NULL) {
        mnLoadScreen_text =
            DevText_Create(0x4C, 250, 230, 16, 1, mnLoadScreen_textbuf);
        if (mnLoadScreen_text != NULL) {
            DevText_Show(text_gobj, mnLoadScreen_text);
            DevText_HideCursor(mnLoadScreen_text);
            DevText_HideBackground(mnLoadScreen_text);
            DevText_SetScale(mnLoadScreen_text, 10.0F, 16.0F);
        }
    }
    OSReport("loadscreen: holding scene %d, %d pipelines created so far\n",
             (int) info->scene_kind, mnLoadScreen_created);
}

/* True while the scene is being held, which is the caller's cue to skip the scene's own frame.
   The caption's ellipsis is animated from here - one line of scene-proc state rather than four
   textures - so it keeps moving while the renderer works. */
static bool mnLoadScreen_Frame(void)
{
    int created;
    int dots;

    if (!mnLoadScreen_holding) {
        return false;
    }
    mnLoadScreen_frames++;

    created = Gfx_PipelinesCreated();
    if (created != mnLoadScreen_created) {
        mnLoadScreen_created = created;
        mnLoadScreen_settled = 0;
    } else if (Gfx_PipelinesPending() != 0) {
        mnLoadScreen_settled = 0;
    } else {
        mnLoadScreen_settled++;
    }

    if (mnLoadScreen_text != NULL) {
        DevText_Erase(mnLoadScreen_text);
        DevText_SetCursorXY(mnLoadScreen_text, 0, 0);
        DevText_Print(mnLoadScreen_text, "NOW LOADING");
        dots = (mnLoadScreen_frames / LOADSCREEN_DOT_FRAMES) % 3;
        DevText_Print(mnLoadScreen_text, ".");
        if (dots >= 1) {
            DevText_Print(mnLoadScreen_text, ".");
        }
        if (dots >= 2) {
            DevText_Print(mnLoadScreen_text, ".");
        }
    }

    if (mnLoadScreen_frames >= LOADSCREEN_MIN_FRAMES &&
        mnLoadScreen_settled >= LOADSCREEN_SETTLE_FRAMES)
    {
        mnLoadScreen_Release("warm");
        return false;
    }
    if (OSGetTime() - mnLoadScreen_started >
        (OSTime) OSSecondsToTicks(LOADSCREEN_CEILING_SECONDS))
    {
        mnLoadScreen_Release("ceiling");
        return false;
    }
    return true;
}

#endif /* TARGET_PC */

GameScene* gm_FindGameSceneHandler(u8 kind)
{
    GameScene* cur;
    for (cur = gm_GetAllGameScenes(); cur->kind != GS_COUNT; cur++) {
        if (cur->kind == kind) {
            return cur;
        }
    }
    return NULL;
}

static inline u64 maybe_gm_801A48A4(u8 i)
{
    u64 temp_ret = gm_801A48A4(i);
    if (gm_80479D58.unk_10.unk_38_0) {
        return temp_ret;
    } else {
        return -1ULL;
    }
}

void gm_801A4D34(void (*on_frame)(void), UNUSED GameSceneInfo* info)
{
    int pad_queue_count;
    int i;
    struct gm_80479D58_t* temp_r25;

    PAD_STACK(28);

    temp_r25 = &gm_80479D58;
    gm_801677C0(&temp_r25->unk_10);
    gm_80479D58.unk_0 = 0;
    gm_80479D58.unk_4 = 0;
    gm_80479D58.unk_8 = 0;
    gm_80479D58.unk_C = 0;
    HSD_PadFlushQueue(HSD_PAD_FLUSH_QUEUE_LEAVE1);
    lbCardGame_InitScene();
#if defined(TARGET_PC)
    mnLoadScreen_Begin(info);
#endif

    while (temp_r25->unk_C == 0) {
#if defined(TARGET_PC)
        /* Which menu, and what is hovered on it. MenuFlow is the menu tree's own state, so
         * this covers every screen that runs through mnmain - the main menu, the VS and 1P
         * submenus, the rules screens. Reported edge-triggered, so it is a handful of lines
         * per run rather than one a frame. The CSS and SSS keep their cursors elsewhere and
         * are NOT covered yet (see _research/scene-launch.md). */
        SceneReport_Menu(mn_804A04F0.cur_menu, mn_804A04F0.hovered_selection,
                         mn_804A04F0.confirmed_selection);
#endif
        hsd_80392E80();
        gmMainLib_8046B0F0.xC = false;

        while ((pad_queue_count = lb_80019894()) == 0) {
            lb_800195D0();
        }
        lb_800195D0();

        if (HSD_PadGetResetSwitch()) {
            gmMainLib_8046B0F0.resetting = true;
            break;
        }

        for (i = 0; i < pad_queue_count; i++) {
            HSD_PerfSetStartTime();
            lb_800198E0();
            if (DbLevel >= DbLKind_DebugRom) {
                gm_801A4970(&temp_r25->unk_10.db_input);
            }
            if (gm_801A46B8(0) || !gm_GetDbPauseFlag(0)) {
                temp_r25->unk_10.unk_38_0 = true;
            } else {
                temp_r25->unk_10.unk_38_0 = false;
            }
            if (gm_80479D58.unk_10.unk_38_0) {
                lb_80019900();
                if (lb_80019A30(0)) {
                    gm_EvaluateAllControllerInputs();
                }
                if (lb_80019A30(0) && on_frame != NULL) {
#if defined(TARGET_PC)
                    /* While the loading screen holds, the scene renders but its clock does not
                       advance: everything else in this loop still runs. */
                    if (!mnLoadScreen_Frame())
#endif
                    {
                        on_frame();
                    }
                }
            }
            if (gm_80479D58.unk_10.x0 != gm_80479D58.unk_10.x1 ||
                temp_r25->unk_10.x2 != temp_r25->unk_10.x3)
            {
                temp_r25->unk_10.unk_20 =
                    maybe_gm_801A48A4(temp_r25->unk_10.x0);
                temp_r25->unk_10.x1 = temp_r25->unk_10.x0;
                temp_r25->unk_10.x3 = temp_r25->unk_10.x2;
                temp_r25->unk_10.x2 = 0;
            }
            temp_r25->unk_10.unk_28 = temp_r25->unk_10.unk_20;
            if (!lb_80019A30(0)) {
                temp_r25->unk_10.unk_28 |=
                    gm_803DA8C8[temp_r25->unk_10.unk_34];
            }
            if (!lb_80019A30(1)) {
                temp_r25->unk_10.unk_28 |=
                    ~gm_803DA8C8[temp_r25->unk_10.unk_34];
            }
            if (DbLevel >= DbLKind_DebugRom) {
                db_CheckScreenshot();
            }
            lbAudioAx_80027DF8();
            if (temp_r25->unk_10.pre_gobj_proc != NULL) {
                temp_r25->unk_10.pre_gobj_proc();
            }
            HSD_GObj_RunProcs();
            if (temp_r25->unk_0 != -2) {
                temp_r25->unk_0++;
            }
            if (gm_80479D58.unk_10.unk_38_0 && lb_80019A30(0)) {
                if (temp_r25->unk_8 != -2) {
                    temp_r25->unk_8++;
                }
            }
            HSD_PerfSetCPUTime();
            if (DbLevel >= DbLKind_DebugRom) {
                OSCheckActiveThreads();
            }
            gmMainLib_8046B0F0.xC = false;
            if (temp_r25->unk_C != 0) {
                break;
            }
        }
        if (temp_r25->unk_C == 2) {
            break;
        }

        lb_800195D0();
        GXInvalidateVtxCache();
        GXInvalidateTexAll();
        HSD_StartRender(HSD_RP_SCREEN);
        HSD_GObj_80390FC0();
        HSD_Init_803755A8();
        HSD_PerfSetDrawTime();
        HSD_VICopyXFBAsync(HSD_RP_SCREEN);
        if (temp_r25->unk_4 != -2U) {
            temp_r25->unk_4++;
        }
        db_TakeScreenshotIfPending();
        HSD_PerfSetTotalTime();
        HSD_PerfInitStat();
    }
#if defined(TARGET_PC)
    mnLoadScreen_Release("scene ended");
#endif
    HSD_VIWaitXFBFlush();
}
