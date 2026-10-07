/* gmfrontend_atlas_toy.inc - Atlas step 10: the retail Trophy Gallery (GS_TOY_GALLERY), Lottery (GS_TOY_LOTTERY) and Collection (GS_TOY_COLLECTION)
 * re-hosted in Atlas chrome. The retail scene runs UNCHANGED: its own pad polling, logic, 3D, popups and exit. After retail's own on_frame this wrapper
 * submits one framed screen per frame (pc/platform/gw_ui_frame.h: opaque plates around a window onto the retail picture, with the trail, an
 * explainer, a counter and key hints on the plates) through the host shims (pc/platform/gw_script_ui_toy.inc). Atlas reads no pad, injects nothing,
 * keeps no focus and records no hit rectangle; the mouse is not supported on a framed screen in version 1.
 *
 * Every scene stays RETAIL until the owner says go for it: MELEE_ATLAS_SCENES=<kind>:overlay (11 Gallery, 12 Lottery, 13 Collection) turns one on for
 * development. MELEE_ATLAS=0 leaves all of it retail.
 *
 * THE WINDOW IS UNMEASURED. Where retail's camera frames its model on the 640x480 canvas is not in the source (it is camera data on the disc), so the
 * three FAT_*_WIN rectangles below are zero and a scene with no window is NOT drawn and hides NOTHING (the plan's "the code refuses to draw rather than
 * guess"). MELEE_ATLAS_FRAME_WIN=x,y,w,h tunes it and MELEE_ATLAS_FRAME_OUTLINE=1 draws its edge in ember; the four numbers that fit are then written
 * here, one rectangle per scene. Until a person has looked, a screen is `hold`, not `go`.
 *
 * TEXT. The trophy names and descriptions are retail glyph streams (SdToy.usd, SdToyExp.usd), not Unicode. Step 8's decoder (at_sis_decode) is not in
 * this tree, so Ui_ToyTextOk is 0: the retail text objects stay visible (the mask leaves toy.text alone) and the explainer uses authored fallbacks
 * ("TROPHY 7", the counter). */
#include <sysdolphin/baselib/cobj.h>

_Static_assert(GS_TOY_GALLERY == 11 && GS_TOY_LOTTERY == 12 && GS_TOY_COLLECTION == 13, "gw_ui_policy.c scene kinds 11, 12 and 13 must be these");

extern int Ui_FrameBegin(const char* id, const char* title, int x, int y, int w, int h);
extern void Ui_ToyChrome(int index, int count, const char* name, const char* desc, const char* series);
extern void Ui_LotChrome(int coins, int bet);
extern void Ui_CollChrome(int total);
extern int Ui_NetplayActive(void);
extern int Ui_ToyTextOk(int kind);
extern int Ui_ToyProbe(void);
extern int Ui_ScenePolicy(int kind);
extern int Toy_PcReadback(int what);          /* src/melee/ty/toy.c */
extern int tyFigupon_PcReadback(int what);    /* src/melee/ty/tyfigupon.c */

/* x, y, w, h in the 640x480 retail canvas. Zero w means unmeasured (see above). */
static const int FAT_GALLERY_WIN[4] = { 0, 0, 0, 0 };
static const int FAT_LOTTERY_WIN[4] = { 0, 0, 0, 0 };
static const int FAT_COLLECTION_WIN[4] = { 0, 0, 0, 0 };

static void (*fat_inner)(void);
static u8 fat_kind;
static int fat_online_noted, fat_probed;

static const int* fat_window(u8 kind)
{
    switch (kind) {
    case GS_TOY_GALLERY:
        return FAT_GALLERY_WIN;
    case GS_TOY_LOTTERY:
        return FAT_LOTTERY_WIN;
    case GS_TOY_COLLECTION:
        return FAT_COLLECTION_WIN;
    }
    return NULL;
}

/* gm_801A4014 asks before the scene's on_enter: is there a window to frame? With none, the policy mask stays empty (nothing is hidden for chrome that
 * will not draw). The host adds MELEE_ATLAS_FRAME_WIN. */
int Fad_ToyWindowKnown(u8 kind)
{
    const int* w = fat_window(kind);
    return w != NULL && w[2] > 0 && w[3] > 0;
}

/* how many GObjs retail has at a GX link (the list the renderer walks: tail first, prev_gx back) */
static int fat_count_gxlink(int link)
{
    HSD_GObj* g;
    int n = 0;
    if (HSD_GObj_804D7820 == NULL) {
        return -1;
    }
    for (g = HSD_GObj_804D7820[link]; g != NULL && n < 512; g = g->prev_gx) {
        n++;
    }
    return n;
}

/* MELEE_ATLAS_TOYPROBE=1: numbers about a bespoke scene, never text. Once per scene entry, from the wrapper (after retail's first frame). */
static void fat_probe(u8 kind)
{
    HSD_CObj* cobj = HSD_CObjGetCurrent();
    int link;
    OSReport("frontend: toyprobe kind %d: policy %d window known %d trophies %d selected %d id %d state %d coins %d bet %d\n", (int) kind,
             Ui_ScenePolicy(kind), Fad_ToyWindowKnown(kind), Toy_PcReadback(0), Toy_PcReadback(1), Toy_PcReadback(2), Toy_PcReadback(3),
             tyFigupon_PcReadback(0), tyFigupon_PcReadback(1));
    if (cobj != NULL) {
        HSD_RectF32 r;
        HSD_CObjGetViewportf(cobj, &r);
        OSReport("frontend: toyprobe kind %d: viewport %d %d %d %d (x min, x max, y min, y max)\n", (int) kind, (int) r.xmin, (int) r.xmax,
                 (int) r.ymin, (int) r.ymax);
    } else {
        OSReport("frontend: toyprobe kind %d: no current camera at this point\n", (int) kind);
    }
    OSReport("frontend: toyprobe kind %d: gx links", (int) kind);
    for (link = 0x30; link < 0x40; link++) {
        OSReport(" %02X:%d", link, fat_count_gxlink(link));
    }
    OSReport("\n");
}

/* ---- the three screens ---- */

static void fat_gallery_submit(void)
{
    int state = Toy_PcReadback(3);
    const int* w = FAT_GALLERY_WIN;
    if (!Ui_FrameBegin("toy.gallery", "GALLERY", w[0], w[1], w[2], w[3])) {
        return;
    }
    Ui_Trail("MAIN MENU");
    Ui_Trail("COLLECTION");
    /* the numbers are retail's; the strings are empty (see TEXT above), so the explainer says TROPHY n and the counter n / total */
    Ui_ToyChrome(Toy_PcReadback(1), Toy_PcReadback(0), "", "", "");
    /* the hints are exactly what retail's pad does (toy.c _Toy_80309404): on the shelf A, X or Y examine and B leaves; while examining B goes back
     * to the shelf. Between the two (the camera moving) retail reads nothing. */
    if (state == 0) {
        Ui_Key('A', "Examine");
        Ui_Key('B', "Back");
    } else if (state == 2) {
        Ui_Key('B', "Back");
    }
    Ui_Commit(-1, -1);
}

static void fat_lottery_submit(void)
{
    int coins = tyFigupon_PcReadback(0), busy = tyFigupon_PcReadback(2);
    const int* w = FAT_LOTTERY_WIN;
    if (!Ui_FrameBegin("toy.lottery", "LOTTERY", w[0], w[1], w[2], w[3])) {
        return;
    }
    Ui_Trail("MAIN MENU");
    Ui_Trail("COLLECTION");
    Ui_LotChrome(coins, tyFigupon_PcReadback(1));
    /* retail's pad (tyfigupon.c _tyFigupon_80316C24): up and down (or Y and X, the stick, the d-pad) set the bet, Z the largest bet, A (or L, R,
     * START) draws, B first lowers the bet to 1 and then leaves. While a draw runs retail reads nothing, and the popup is retail's own. */
    if (busy == 0) {
        if (coins > 0) {
            Ui_Key('A', "Draw");
            Ui_Key('Z', "Most coins");
        }
        Ui_Key('B', "Back");
    }
    Ui_Commit(-1, -1);
}

static void fat_collection_submit(void)
{
    const int* w = FAT_COLLECTION_WIN;
    if (!Ui_FrameBegin("toy.collection", "COLLECTION", w[0], w[1], w[2], w[3])) {
        return;
    }
    Ui_Trail("MAIN MENU");
    Ui_CollChrome(Toy_PcReadback(0));
    /* retail's pad (tydisplay.c _tyDisplay_8031A4EC): the stick looks around, Y, L or d-pad up zoom in, X, R or d-pad down zoom out, A resets the
     * view, B or START leaves. The room has no cursor and no selected trophy. */
    Ui_Key('Y', "Zoom in");
    Ui_Key('X', "Zoom out");
    Ui_Key('A', "Reset view");
    Ui_Key('B', "Back");
    Ui_Commit(-1, -1);
}

static void fat_submit(u8 kind)
{
    if (kind == GS_TOY_GALLERY) {
        fat_gallery_submit();
    } else if (kind == GS_TOY_LOTTERY) {
        fat_lottery_submit();
    } else if (kind == GS_TOY_COLLECTION) {
        fat_collection_submit();
    }
}

/* every frame of the scene: retail first (its own pad polling, logic, 3D and exit), then the chrome from readbacks */
static void fat_overlay_frame(void)
{
    if (fat_inner != NULL) {
        fat_inner();
    }
    if (Toy_PcReadback(4) != 0) {
        return; /* retail's exit is under way: it frees the state the readbacks read */
    }
    if (Ui_ToyProbe() && !fat_probed) {
        fat_probed = 1;
        fat_probe(fat_kind);
    }
    if (Ui_ScenePolicy(fat_kind) != 1 /* AT_POLICY_OVERLAY */) {
        return; /* the wrapper may be here for the probe alone */
    }
    if (Ui_NetplayActive()) { /* the mask is 0 online: chrome over unhidden retail 2D would collide */
        if (!fat_online_noted) {
            OSReport("frontend: framed scene %d not drawn: a session is on\n", (int) fat_kind);
            fat_online_noted = 1;
        }
        return;
    }
    fat_submit(fat_kind);
}

void (*Fad_OverlayWrap(u8 kind, void (*inner)(void)))(void)
{
    if (kind != GS_TOY_GALLERY && kind != GS_TOY_LOTTERY && kind != GS_TOY_COLLECTION) {
        return inner;
    }
    fat_inner = inner;
    fat_kind = kind;
    fat_online_noted = 0;
    fat_probed = 0;
    return fat_overlay_frame;
}
