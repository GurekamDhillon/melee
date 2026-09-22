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
#include "gmfrontend.h"
#include "gmvs.h"
#include <sysdolphin/baselib/random.h>
#if defined(TARGET_PC)
#include <melee/if/textdraw.h>
#include <melee/if/textlib.h>
#include <melee/if/types.h>
#include <sysdolphin/baselib/gobjplink.h>
#include <dolphin/gx.h>
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
#if defined(TARGET_PC)
    {
        extern void ifMagnify_LogicDisarm(void);
        extern void Camera_ForgetGameCamera(void);
        ifMagnify_LogicDisarm(); /* the magnifier re-arms when a match creates it */
        /* ...and the game camera: the logic-side refresh after HSD_GObj_RunProcs
           (Camera_RefreshViewingMtx, determinism work) runs in EVERY scene and trusts
           game_camera.gobj, which vanilla never clears - after a match it points into the freed
           scene. User-found: idle on the title > attract demo > Start > the title screen wrote
           through the demo's camera and crashed. A match's camera setup assigns it again. */
        Camera_ForgetGameCamera();
    }
#endif

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
 * worker thread, and SKIPS that draw until it is ready, so the first frames of a cold scene are
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
 * compiling in the background, a GPU being shared - must still boot into its match. Measured on
 * a cold boot into Training on Final Destination: the hold lasts about 205 frames and 56
 * pipelines get compiled inside it.
 *
 * WHAT IT DRAWS, AND WHY IT IS THE DEBUG FONT RATHER THAN THE PNG ART. Everything here goes
 * through DevText (src/melee/if/textdraw.c), which is the game's own screen-space 2D: one text
 * box with its background and no text is a filled rectangle, which is the panel; a second with
 * its background and text is the caption on its plate. It is set up once per scene by
 * gm_801A4BD4 -> un_802FF78C and drawn by its own camera on gx_link 17, so there is nothing to
 * initialise and nothing to tear down but the boxes.
 *
 * The .gxtex art in _build/menutex (panel_bg, the 9-sliced frame, the cursor) is NOT drawn here
 * yet, and the reason is worth recording because it cost most of this session. The HSD_SObj path
 * from mnCharSel_MenuTexSetup works - the same binary, the same scene and the same .gxtex file
 * put its quad on screen when mncharsel creates it. The identical code called from HERE does
 * not: the callback runs, the sobj list walks correctly (11 sprites, right rectangles, right
 * scales, right GX formats, image pointers in MEM1), 803A4A68 emits the quads, and nothing
 * appears. Ruled out by experiment, not by argument: render priority (10 and 0xFF), the raw-GX
 * viewport/projection/position-matrix prologue copied verbatim from mncharsel, an HSD_CObj set
 * up exactly like DevText's plus an explicit GXLoadPosMtxImm, drawing inside DevText's own
 * camera pass on gx_link 17 instead of standing alone on the GXLinkMax list, the stage's fog,
 * one sprite instead of eleven, and 1x art instead of 2x. The remaining difference between the
 * two call sites is the call site itself. Captures: _build/runs/ldscr1..ldscr11.
 */

/* Native entry points. gwtool prefixes every game symbol with `gw_`, so the unprefixed name here
 * binds to the shim of the same name in pc/platform/gw_runtime.c. */
extern int Gfx_PipelinesUrgent(void);
extern int Gfx_PipelinesCreated(void);
extern int Gfx_LoadScreenEnabled(void);

/* Settled means nothing the frozen frame draws is still compiling (Gfx_PipelinesUrgent) - NOT
   "nothing queued at all": the pipeline seed's background warm-up keeps thousands queued for
   minutes, and waiting on that pinned every hold to its ceiling while the match ran underneath.
   After the frontend's loading screen the seed's core is already built, so the hold only has
   to cover the first frame's own stragglers. */
#define LOADSCREEN_MIN_FRAMES 20
#define LOADSCREEN_SETTLE_FRAMES 30
#define LOADSCREEN_WARM_MIN_FRAMES 2
#define LOADSCREEN_WARM_SETTLE_FRAMES 8
#define LOADSCREEN_CEILING_SECONDS 10
#define LOADSCREEN_DOT_FRAMES 15

/* The panel has to cover the whole visible frame, which is a little LARGER than 640x480: the
   DevText camera's ortho box runs -20..660 by -20..500, so a box placed at 0,0 at exactly
   640x480 leaves a strip of the match showing down each edge. 72x36 cells at the 10x16 default
   scale is 720x576, started off-screen at -24,-24. */
#define LOADSCREEN_PANEL_COLS 72
#define LOADSCREEN_PANEL_ROWS 36
#define LOADSCREEN_PANEL_X (-24)
#define LOADSCREEN_PANEL_Y (-24)
#define LOADSCREEN_CAPTION_COLS 16

static DevText* mnLoadScreen_panel;
static DevText* mnLoadScreen_text;
static char mnLoadScreen_panelbuf[2 * LOADSCREEN_PANEL_COLS * LOADSCREEN_PANEL_ROWS];
static char mnLoadScreen_textbuf[2 * LOADSCREEN_CAPTION_COLS];
static int mnLoadScreen_holding;
static int mnLoadScreen_frames;
static int mnLoadScreen_settled;
static int mnLoadScreen_created;
static bool mnLoadScreen_warm; ///< the frontend's loading screen ran first
static OSTime mnLoadScreen_started;

static void mnLoadScreen_Release(char* why)
{
    if (!mnLoadScreen_holding) {
        return;
    }
    mnLoadScreen_holding = 0;
    /* DevText_Unlink, not DevText_Remove. The loading screen's boxes are the first two on the
       draw list, and DevText_Remove(&handle) on the head box never updates devtext_drawlist: it
       kept naming the freed box, whose next is the free pool, and every box behind it - the F9
       panel, the toast, the run label - vanished the moment the match started. The pool itself
       is re-initialised per scene by gm_801A4BD4 -> DevText_Setup, so nothing accumulates. */
    if (mnLoadScreen_text != NULL) {
        DevText_HideText(mnLoadScreen_text);
        DevText_HideBackground(mnLoadScreen_text);
        DevText_Unlink(mnLoadScreen_text);
        mnLoadScreen_text = NULL;
    }
    if (mnLoadScreen_panel != NULL) {
        DevText_HideText(mnLoadScreen_panel);
        DevText_HideBackground(mnLoadScreen_panel);
        DevText_Unlink(mnLoadScreen_panel);
        mnLoadScreen_panel = NULL;
    }
    /* the match clock, which the freeze keeps where it started */
    OSReport("loadscreen: released (%s) after %d frames, %d pipelines created, clock %u.%02u\n",
             why, mnLoadScreen_frames, Gfx_PipelinesCreated(), gm_8016AEEC(),
             (u32) gm_8016AF0C());
}

static void mnLoadScreen_Begin(GameSceneInfo* info)
{
    HSD_GObj* text_gobj;

    mnLoadScreen_holding = 0;
    mnLoadScreen_panel = NULL;
    mnLoadScreen_text = NULL;
    if (info == NULL || !Gfx_LoadScreenEnabled()) {
        return;
    }
    /* After the frontend's loading screen (after the SSS) the renderer is warm; the hold still
       runs, briefly, so the match's first frame is complete before its clock starts. */
    mnLoadScreen_warm = gmFrontend_TakeWarmed();
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

    text_gobj = DevText_GetGObj();
    if (text_gobj != NULL) {
        /* Created panel first, caption second: DevText_AddToList appends, and the list is drawn
           in order, so the panel is behind. */
        mnLoadScreen_panel =
            DevText_Create(0x4A, LOADSCREEN_PANEL_X, LOADSCREEN_PANEL_Y, LOADSCREEN_PANEL_COLS,
                           LOADSCREEN_PANEL_ROWS,
                           mnLoadScreen_panelbuf);
        if (mnLoadScreen_panel != NULL) {
            GXColor panel = { 10, 14, 24, 255 }; /* the menu art's ink */
            DevText_Show(text_gobj, mnLoadScreen_panel);
            DevText_HideCursor(mnLoadScreen_panel);
            DevText_HideText(mnLoadScreen_panel);
            DevText_SetBGColor(mnLoadScreen_panel, panel);
        }
        /* After the frontend's loading screen (it said what is loading) the hold is a few
           frames of plain ink - no second, older-looking caption flashing up. */
        mnLoadScreen_text = mnLoadScreen_warm ? NULL
                                              : DevText_Create(0x4B, 250, 230,
                                                               LOADSCREEN_CAPTION_COLS, 1,
                                                               mnLoadScreen_textbuf);
        if (mnLoadScreen_text != NULL) {
            GXColor plate = { 20, 38, 92, 255 }; /* its dark cobalt */
            DevText_Show(text_gobj, mnLoadScreen_text);
            DevText_HideCursor(mnLoadScreen_text);
            DevText_SetBGColor(mnLoadScreen_text, plate);
            DevText_SetScale(mnLoadScreen_text, 10.0F, 16.0F);
        }
    }
    OSReport("loadscreen: holding scene %d, %d pipelines created so far\n",
             (int) info->scene_kind, mnLoadScreen_created);
}

/* True while the scene is being held, which is the caller's cue to skip the scene's own frame.
   The caption's ellipsis is animated from here - one line of scene state rather than four
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
    mnLoadScreen_created = created;
    if (Gfx_PipelinesUrgent() != 0) {
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

    if (mnLoadScreen_frames >=
            (mnLoadScreen_warm ? LOADSCREEN_WARM_MIN_FRAMES : LOADSCREEN_MIN_FRAMES) &&
        mnLoadScreen_settled >=
            (mnLoadScreen_warm ? LOADSCREEN_WARM_SETTLE_FRAMES : LOADSCREEN_SETTLE_FRAMES))
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

/* ---- the F9 panel, the toast and the run label, in the game's own 2D ----------------------
 *
 * These were Dear ImGui windows, composited by Aurora over the finished frame. They are drawn
 * here instead, with DevText - the same screen-space text the loading screen above uses, and the
 * same thing the retail debug builds used. The port's only C++ TU keeps the parts the game
 * cannot see (the F9 key, the environment, the DVD counters, the mirrored pad state) and
 * formats the lines; everything visible is now the engine's.
 *
 * The boundary is pull-only and scalar-only, on purpose. A game TU's memory accesses are
 * byte-swapped by gwtool - including to its own stack - so a host function writing through a
 * pointer this file passes it would land byte-reversed. Strings are bytes, so a const char*
 * coming back the other way is safe, and that is all that crosses.
 *
 * Lifetime is the scene's. DevText's pool is re-initialised per scene by gm_801A4BD4, so the
 * boxes are created in mnOverlay_Begin next to the loading screen's and are never freed here;
 * they go away with the pool. Creating them AFTER the loading screen's panel matters, because
 * DevText_AddToList appends and the list draws in order: the caption has to sit on top of the
 * loader, not behind it.
 */

extern char* Overlay_GetRunLabel(void);
extern char* Overlay_GetToast(void);
extern int Overlay_GetPanelOpen(void);
extern int Overlay_GetPanelLineCount(void);
extern char* Overlay_GetPanelLine(int i);
extern int Overlay_GetPanelLineDim(int i);

/* 8x12 rather than DevText's 10x16 default: the panel is 44 columns of diagnostics and at the
   default scale it would be 440 points wide on a 640-point screen. */
#define OVERLAY_CELL_W 8.0F
#define OVERLAY_CELL_H 12.0F
#define OVERLAY_PANEL_COLS 44
#define OVERLAY_PANEL_ROWS 20
#define OVERLAY_CAPTION_COLS 48

/* DevText reads the colour index out of the TOP TWO BITS of each cell's attribute byte
   (textdraw.c: `(buf[index + 1] & 0xC0) >> 6`) while DevText_StoreColorIndex writes whatever
   it is given, so a palette slot has to be pre-shifted to select it when printing. Slot 0 is
   left white; slot 3 is re-coloured grey for the lines the ImGui panel drew with TextDisabled. */
#define OVERLAY_COLOR_NORMAL 0x00
#define OVERLAY_COLOR_DIM (3 << 6)

static DevText* mnOverlay_panel;
static DevText* mnOverlay_caption;
static DevText* mnOverlay_toast;
static char mnOverlay_panelbuf[2 * OVERLAY_PANEL_COLS * OVERLAY_PANEL_ROWS];
static char mnOverlay_captionbuf[2 * OVERLAY_CAPTION_COLS];
static char mnOverlay_toastbuf[2 * OVERLAY_CAPTION_COLS];

static DevText* mnOverlay_Make(HSD_GObj* gobj, char id, int x, int y, int w, int h, char* buf,
                               GXColor bg)
{
    GXColor dim = { 150, 150, 160, 255 };
    DevText* t = DevText_Create(id, x, y, w, h, buf);
    if (t == NULL) {
        return NULL;
    }
    DevText_Show(gobj, t);
    DevText_HideCursor(t);
    DevText_SetScale(t, OVERLAY_CELL_W, OVERLAY_CELL_H);
    DevText_SetBGColor(t, bg);
    /* Recolour palette slot 3 while the index is still unshifted - SetTextColor indexes
       text_colors[current_color], so it must not be called once the index carries the shift. */
    DevText_StoreColorIndex(t, 3);
    DevText_SetTextColor(t, dim);
    DevText_StoreColorIndex(t, OVERLAY_COLOR_NORMAL);
    DevText_HideText(t);
    return t;
}

static void mnOverlay_Begin(void)
{
    GXColor panel = { 10, 10, 16, 220 };
    GXColor plate = { 20, 24, 40, 180 };
    HSD_GObj* gobj;

    mnOverlay_panel = NULL;
    mnOverlay_caption = NULL;
    mnOverlay_toast = NULL;

    gobj = DevText_GetGObj();
    if (gobj == NULL) {
        return;
    }
    mnOverlay_caption = mnOverlay_Make(gobj, 0x4C, 8, 6, OVERLAY_CAPTION_COLS, 1,
                                       mnOverlay_captionbuf, plate);
    mnOverlay_toast = mnOverlay_Make(gobj, 0x4D, 160, 28, OVERLAY_CAPTION_COLS, 1,
                                     mnOverlay_toastbuf, plate);
    mnOverlay_panel = mnOverlay_Make(gobj, 0x4E, 16, 28, OVERLAY_PANEL_COLS,
                                     OVERLAY_PANEL_ROWS, mnOverlay_panelbuf, panel);
}

/* A one-line box, sized to the text so its plate is not a full-width bar behind six
   characters. DevText fixes w at create time, but w is only ever used as a stride and a
   bound, and the buffer is allocated for the widest case, so narrowing it is safe. */
static void mnOverlay_Line(DevText* t, char* str)
{
    int n;

    if (t == NULL) {
        return;
    }
    if (str == NULL) {
        DevText_HideText(t);
        DevText_HideBackground(t);
        return;
    }
    n = 0;
    while (str[n] != '\0' && n < OVERLAY_CAPTION_COLS) {
        n++;
    }
    t->w = n > 0 ? n : 1;
    DevText_Erase(t);
    DevText_SetCursorXY(t, 0, 0);
    DevText_Print(t, str);
    DevText_ShowText(t);
    DevText_ShowBackground(t);
}

/* F10: put every overlay box back on screen. A box that fell out of DevText's draw list is
   re-listed; one that is still listed is left alone (listing it twice would loop the list). */
static void mnOverlay_Reshow(void)
{
    extern int Overlay_GetReshowCount(void);
    static int seen;
    HSD_GObj* gobj;
    int n = Overlay_GetReshowCount();
    if (n == seen) {
        return;
    }
    seen = n;
    gobj = DevText_GetGObj();
    if (gobj == NULL) {
        return;
    }
    if (mnOverlay_caption != NULL && !DevText_IsListed(mnOverlay_caption)) {
        DevText_Show(gobj, mnOverlay_caption);
    }
    if (mnOverlay_toast != NULL && !DevText_IsListed(mnOverlay_toast)) {
        DevText_Show(gobj, mnOverlay_toast);
    }
    if (mnOverlay_panel != NULL && !DevText_IsListed(mnOverlay_panel)) {
        DevText_Show(gobj, mnOverlay_panel);
    }
    OSReport("overlay: F10 - boxes re-shown\n");
}

static void mnOverlay_Frame(void)
{
    int i;
    int rows;

    mnOverlay_Reshow();

    /* The run label names a test run for whoever watches it; it belongs with the rest of the debug
       view, so it shows only while the F9 panel is open - never over normal play. */
    mnOverlay_Line(mnOverlay_caption, Overlay_GetPanelOpen() ? Overlay_GetRunLabel() : NULL);
    mnOverlay_Line(mnOverlay_toast, Overlay_GetToast());

    if (mnOverlay_panel == NULL) {
        return;
    }
    if (!Overlay_GetPanelOpen()) {
        DevText_HideText(mnOverlay_panel);
        DevText_HideBackground(mnOverlay_panel);
        return;
    }
    DevText_Erase(mnOverlay_panel);
    rows = Overlay_GetPanelLineCount();
    if (rows > OVERLAY_PANEL_ROWS) {
        rows = OVERLAY_PANEL_ROWS;
    }
    for (i = 0; i < rows; i++) {
        char* line = Overlay_GetPanelLine(i);
        if (line == NULL) {
            continue;
        }
        DevText_SetCursorXY(mnOverlay_panel, 0, i);
        DevText_StoreColorIndex(mnOverlay_panel, Overlay_GetPanelLineDim(i)
                                                     ? OVERLAY_COLOR_DIM
                                                     : OVERLAY_COLOR_NORMAL);
        DevText_Print(mnOverlay_panel, line);
    }
    DevText_StoreColorIndex(mnOverlay_panel, OVERLAY_COLOR_NORMAL);
    DevText_ShowText(mnOverlay_panel);
    DevText_ShowBackground(mnOverlay_panel);
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
    mnOverlay_Begin();
    if (info != NULL) {
        extern void RB_SceneBegin(int scene_kind);
        RB_SceneBegin((int) info->scene_kind); /* a rollback session governs VS matches only */
    }
    {
        /* the Lua scripting engine (pc/platform/gw_script.c): on_scene / on_match_end */
        extern void Script_SceneBegin(int scene_kind);
        Script_SceneBegin(info != NULL ? (int) info->scene_kind : -1);
    }
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
        /* Outside everything below: the caption, the toast and the panel have to stay live
           while the loading screen holds and while the debug pause has the scene stopped. */
        mnOverlay_Frame();
        {
            /* scripting: console + socket commands, on_tick, on_draw (gw_script.c) */
            extern void Script_Tick(void);
            Script_Tick();
        }
#endif
        hsd_80392E80();
        gmMainLib_8046B0F0.xC = false;

        while ((pad_queue_count = lb_80019894()) == 0) {
            lb_800195D0();
        }
        lb_800195D0();
#if defined(TARGET_PC)
        {
            /* MELEE_DETERMINISTIC (pc/platform/gw_replay.c): one logic frame per render, as on
               the console at full speed. Catching up with several logic frames per render made
               render-coupled state go stale for game logic - the magnifier's off-screen flag is
               set in its render callback (ifMagnify_802FBBDC) and read every logic frame by the
               1%-per-interval off-screen damage (Fighter_8006A1BC), so under load that 1% landed a
               frame early or late and two runs of one replay parted ways. The rest of the queue
               stays queued and runs next iteration: the game slows under load instead of
               skipping renders. */
            extern int Det_Enabled(void);
            if (pad_queue_count > 1 && Det_Enabled()) {
                pad_queue_count = 1;
            }
        }
        {
            /* MELEE_SYNCTEST (pc/platform/gw_snap.c): k+1 logic iterations this tick - roll back
               k frames, resimulate them, then run the new frame */
            extern int SyncTest_Iterations(int count);
            /* MELEE_RB_FAKE (pc/platform/gw_rollback.c): the rollback session decides instead -
               k resimulated iterations plus the new one, or none while it stalls */
            extern int RB_Enabled(void);
            extern int RB_Iterations(int count);
            if (RB_Enabled()) {
                pad_queue_count = RB_Iterations(pad_queue_count);
            } else {
                pad_queue_count = SyncTest_Iterations(pad_queue_count);
            }
        }
        {
            /* scripting: pause / frame advance (never during a rollback session) */
            extern int Script_Iterations(int count);
            pad_queue_count = Script_Iterations(pad_queue_count);
        }
#endif

        if (HSD_PadGetResetSwitch()) {
            gmMainLib_8046B0F0.resetting = true;
            break;
        }

        for (i = 0; i < pad_queue_count; i++) {
#if defined(TARGET_PC)
            bool held = false; /* the loading screen is holding this frame */
            {
                /* the logic-frame boundary: SyncTest saves, loads or compares here */
                extern void SyncTest_IterStart(void);
                extern int RB_Enabled(void);
                extern void RB_IterStart(void);
                if (RB_Enabled()) {
                    RB_IterStart();
                } else {
                    SyncTest_IterStart();
                }
            }
            {
                /* scripting, at the logic-frame boundary: pending savestate/loadstate and
                   on_frame_pre (not on a resimulated frame) */
                extern void Script_FramePre(void);
                Script_FramePre();
            }
#endif
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
                    /* While the loading screen holds, the scene renders (that is what warms the
                       renderer) but does not advance: no on_frame here, and no GObj procs
                       below - those run the fighters, the stage and the timer, and running them
                       let a match play out behind the loading screen. */
                    held = mnLoadScreen_Frame();
                    if (!held)
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
#if defined(TARGET_PC)
            if (!held)
#endif
            {
#if defined(TARGET_PC)
                {
                    /* MELEE_SLP playback: count the frame (Slippi numbers them from -123), force
                       the console's seed under MELEE_SLP_RESYNC, and report the first frame the
                       port's RNG leaves the console's. */
                    extern int Replay_Enabled(void);
                    extern int Replay_Tick(void);
                    extern u32 Replay_ResyncSeed(void);
                    extern void Replay_CheckSeed(u32 port_seed);
                    extern void Replay_NoteSeed(u32 arrived_seed);
                    if (Replay_Enabled()) {
                        u32 s;
                        Replay_Tick();
                        Replay_NoteSeed(*HSD_RandSeedPtr); /* the seed the port ARRIVED with */
                        s = Replay_ResyncSeed();
                        if (s != 0) {
                            *HSD_RandSeedPtr = s;
                        }
                        Replay_CheckSeed(*HSD_RandSeedPtr);
                        {
                            extern int Snap_Curated(void);
                            extern void Snap_CuratedMix(const u32* w, int n);
                            if (Snap_Curated()) {
                                u32 sw = *HSD_RandSeedPtr;
                                Snap_CuratedMix(&sw, 1);
                            }
                        }
                    }
                }
#endif
                if (temp_r25->unk_10.pre_gobj_proc != NULL) {
                    temp_r25->unk_10.pre_gobj_proc();
                }
                HSD_GObj_RunProcs();
#if defined(TARGET_PC)
                {
                    /* logic-side off-screen flag for the next logic frame (ifmagnify.c) */
                    extern void ifMagnify_UpdateLogicOffscreen(void);
                    extern void Camera_RefreshViewingMtx(void);
                    /* refill the pools the render pass drains, so IT never allocates from the
                     * shared heap - a resimulated frame does not render, and an allocation there
                     * would shift every later address (objalloc.c) */
                    extern void HSD_ObjAllocTopUp(void);
                    extern void Netplay_Background(void);
                    Netplay_Background(); /* an online lobby stays connected on the CSS too */
                    ifMagnify_UpdateLogicOffscreen();
                    Camera_RefreshViewingMtx();
                    HSD_ObjAllocTopUp();
                }
                {
                    /* scripting: on_frame, input-script tasks, match start (gw_script.c) */
                    extern void Script_FramePost(void);
                    Script_FramePost();
                }
#endif
            }
            if (temp_r25->unk_0 != -2) {
                temp_r25->unk_0++;
            }
#if defined(TARGET_PC)
            if (!held && gm_80479D58.unk_10.unk_38_0 && lb_80019A30(0)) {
#else
            if (gm_80479D58.unk_10.unk_38_0 && lb_80019A30(0)) {
#endif
                if (temp_r25->unk_8 != -2) {
                    temp_r25->unk_8++;
                }
            }
#if defined(TARGET_PC)
            {
                /* SyncTest: a resimulated frame must ALSO render, minus the present. The render
                 * pass is part of a frame - it moves object pools, fills matrix caches, clears
                 * dirty flags - and the first pass did it, so a resimulation that skips it lands
                 * in a different state (the whole "render-owned" chase in gw_snap.c). Same calls
                 * as the real render below, without HSD_VICopyXFBAsync. Its draw commands are
                 * discarded with the frame by aurora, so the picture is unaffected. */
                extern int Snap_Resimulating(void);
                extern int Snap_CuratedNoRender(void);
                extern void SyncTest_PreRender(void);
                if (Snap_Resimulating() && !Snap_CuratedNoRender()) {
                    extern void Snap_Time(int what, int begin);
                    Snap_Time(2, 0); /* a resimulated iteration's logic, end of IterStart -> here */
                    SyncTest_PreRender(); /* open the between-frames window here too */
                    extern void Gx_SuppressDraws(int on);
                    extern int Snap_SuppressDraws(void);
                    Snap_Time(0, 1);
                    Snap_Time(3, 1);
                    lb_800195D0();
                    GXInvalidateVtxCache();
                    GXInvalidateTexAll();
                    Snap_Time(3, 0);
                    Snap_Time(4, 1);
                    HSD_StartRender(HSD_RP_SCREEN);
                    Snap_Time(4, 0);
                    Snap_Time(5, 1);
                    Gx_SuppressDraws(Snap_SuppressDraws());
                    HSD_GObj_80390FC0();
                    Gx_SuppressDraws(0);
                    Snap_Time(5, 0);
                    Snap_Time(6, 1);
                    HSD_Init_803755A8();
                    Snap_Time(6, 0);
                    Snap_Time(0, 0);
                }
            }
#endif
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

#if defined(TARGET_PC)
        {
            extern void SyncTest_PreRender(void); /* gw_snap.c: measure render-owned state */
            extern void Snap_Time(int what, int begin);
            SyncTest_PreRender();
            Snap_Time(1, 1);
        }
#endif
        lb_800195D0();
        GXInvalidateVtxCache();
        GXInvalidateTexAll();
#if defined(TARGET_PC)
        {
            /* render-section marker for the determinism audit: an RNG draw made while it is set is
               render code consuming simulation state (gw_replay.c, gw_Replay_RandTrace) */
            extern void Det_SetInRender(int on);
            Det_SetInRender(1);
#endif
        HSD_StartRender(HSD_RP_SCREEN);
        HSD_GObj_80390FC0();
        HSD_Init_803755A8();
        HSD_PerfSetDrawTime();
        HSD_VICopyXFBAsync(HSD_RP_SCREEN);
#if defined(TARGET_PC)
            Det_SetInRender(0);
        }
#endif
        if (temp_r25->unk_4 != -2U) {
            temp_r25->unk_4++;
        }
#if defined(TARGET_PC)
        {
            extern void SyncTest_PostRender(void);
            extern void Snap_Time(int what, int begin);
            extern void RB_TickEnd(void);
            Snap_Time(1, 0);
            SyncTest_PostRender();
            RB_TickEnd();
        }
#endif
        db_TakeScreenshotIfPending();
        HSD_PerfSetTotalTime();
        HSD_PerfInitStat();
    }
#if defined(TARGET_PC)
    mnLoadScreen_Release("scene ended");
#endif
    HSD_VIWaitXFBFlush();
}
