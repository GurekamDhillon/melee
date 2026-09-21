#include "gmfrontend.h"

#if defined(TARGET_PC)

#include "gm_1A3F.h"
#include "gmscene.h"
#include <dolphin/gx.h>
#include <dolphin/os.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/mn/inlines.h>
#include <melee/mn/mnmain.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/hsd_3915.h>
#include <sysdolphin/baselib/sislib.h>

/* ---- The port's frontend: screens placed anywhere in Melee's menu flow ------------------------
 *
 * HOW MELEE'S FLOW WORKS. A game mode ("major scene": the main menu, VS mode, training...) is a
 * table of states ("minor scenes": CSS, SSS, the match, results), each naming a scene kind whose
 * GameScene supplies on_enter / on_frame / on_exit. The top-level loop (gm_801A4510) runs one mode
 * until it asks for another, then runs that one.
 *
 * HOW A SCREEN IS PLACED. A frontend screen is its own mode, GM_FRONTEND, with one state and one
 * scene, GS_FRONTEND - built exactly like a native screen, so it gets the engine's own per-scene
 * setup, input, audio and transitions. gmFrontend_Route runs on every mode change: when a rule
 * says "between FROM and TO, show SCREEN", the loop enters GM_FRONTEND instead of TO. Confirming
 * continues to TO, exactly as the game intended; backing out returns to FROM. Replacing a native
 * screen is the same rule with a different continuation.
 *
 * DRAWING. Melee's own systems: the text canvas (HSD_SisLib - the menus' font, with its own
 * orthographic 640x480 camera) and a GX link callback on that camera for the panels, so the
 * screen renders through aurora at the window's resolution like everything else. */

/* One screen's content. */
typedef struct FrontendScreen {
    const char* title;
    const char* subtitle;
    const char* note;
    const char* items[4];
    int n_items;
    int continue_item; /* confirming this item continues to the rule's TO mode */
} FrontendScreen;

static const FrontendScreen fe_screen_vs_setup = {
    "VS. MELEE",
    "MATCH SETUP",
    "A custom screen, placed between VS Mode and Character Select.",
    { "Continue to Character Select", "Back to Main Menu" },
    2,
    0,
};

typedef struct FrontendRule {
    u8 from;
    u8 to;
    const FrontendScreen* screen;
} FrontendRule;

/* WHERE SCREENS GO. One line per placement: Title > VS. Mode > Melee enters GM_VS from GM_MENU,
 * and GM_VS's first state is the CSS - so this screen sits between "Melee" and the CSS. */
static const FrontendRule fe_rules[] = {
    { GM_MENU, GM_VS, &fe_screen_vs_setup },
};

static struct {
    const FrontendScreen* screen;
    u8 continue_to;
    u8 back_to;
    int canvas;
    HSD_Text* texts[16];
    int n_texts;
    int cursor;
    int frames;
    int result; /* 0 still here, 1 continue, 2 back */
} fe;

u8 gmFrontend_Route(u8 from, u8 to)
{
    static int enabled = -1;
    int i;
    if (enabled < 0) {
        extern int PcFrontendEnabled(void);
        enabled = PcFrontendEnabled();
    }
    if (!enabled || from == GM_FRONTEND) {
        return to; /* leaving a screen goes exactly where it chose */
    }
    for (i = 0; i < (int) (sizeof fe_rules / sizeof fe_rules[0]); i++) {
        if (fe_rules[i].from == from && fe_rules[i].to == to) {
            fe.screen = fe_rules[i].screen;
            fe.continue_to = to;
            fe.back_to = from;
            OSReport("frontend: mode %d -> %d, showing \"%s\" first\n", from, to,
                     fe.screen->title);
            return GM_FRONTEND;
        }
    }
    return to;
}

/* ---- drawing ------------------------------------------------------------------------------- */

#define FE_GX_LINK 14
#define FE_W 640.0F
#define FE_H 480.0F
#define FE_ROW_X 64.0F
#define FE_ROW_Y 150.0F
#define FE_ROW_W 512.0F
#define FE_ROW_H 46.0F
#define FE_ROW_STEP 58.0F

static GXColor fe_rgba(u8 r, u8 g, u8 b, u8 a)
{
    GXColor c;
    c.r = r;
    c.g = g;
    c.b = b;
    c.a = a;
    return c;
}

/* A rectangle in 640x480 screen space, y down, with a vertical gradient. The canvas camera is
 * ortho with y running 0..-480, hence the negation. */
static void fe_rect(float x, float y, float w, float h, GXColor top, GXColor bottom)
{
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    GXPosition2f32(x, -y);
    GXColor4u8(top.r, top.g, top.b, top.a);
    GXPosition2f32(x + w, -y);
    GXColor4u8(top.r, top.g, top.b, top.a);
    GXPosition2f32(x + w, -(y + h));
    GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
    GXPosition2f32(x, -(y + h));
    GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
}

static void fe_draw(HSD_GObj* gobj, int pass)
{
    int i;
    float pulse;
    (void) gobj;
    if (pass != 0 || fe.screen == NULL) {
        return;
    }
    hsd_80391A04(1.0F, 1.0F, 1);

    /* backdrop, header band and its accent rule, footer band */
    fe_rect(0, 0, FE_W, FE_H, fe_rgba(20, 26, 46, 255), fe_rgba(5, 6, 12, 255));
    fe_rect(0, 0, FE_W, 92, fe_rgba(34, 44, 80, 255), fe_rgba(20, 26, 46, 255));
    fe_rect(0, 92, FE_W, 2, fe_rgba(96, 150, 255, 255), fe_rgba(96, 150, 255, 255));
    fe_rect(0, 94, FE_W, 10, fe_rgba(96, 150, 255, 60), fe_rgba(96, 150, 255, 0));
    fe_rect(0, 436, FE_W, 44, fe_rgba(0, 0, 0, 120), fe_rgba(0, 0, 0, 200));

    /* rows: a quiet panel each; the selected one lit, with a soft glow that breathes */
    pulse = (float) (fe.frames % 60) / 60.0F;
    pulse = pulse < 0.5F ? pulse * 2.0F : (1.0F - pulse) * 2.0F;
    for (i = 0; i < fe.screen->n_items; i++) {
        float y = FE_ROW_Y + FE_ROW_STEP * i;
        if (i == fe.cursor) {
            u8 glow = (u8) (40 + 50 * pulse);
            fe_rect(FE_ROW_X - 6, y - 6, FE_ROW_W + 12, FE_ROW_H + 12,
                    fe_rgba(96, 150, 255, glow), fe_rgba(96, 150, 255, glow));
            fe_rect(FE_ROW_X, y, FE_ROW_W, FE_ROW_H, fe_rgba(84, 132, 240, 245),
                    fe_rgba(52, 88, 190, 245));
            fe_rect(FE_ROW_X, y, 6, FE_ROW_H, fe_rgba(255, 255, 255, 255),
                    fe_rgba(200, 220, 255, 255));
        } else {
            fe_rect(FE_ROW_X, y, FE_ROW_W, FE_ROW_H, fe_rgba(255, 255, 255, 22),
                    fe_rgba(255, 255, 255, 10));
        }
    }
}

static HSD_Text* fe_text(float x, float y, float scale, GXColor color, const char* s)
{
    HSD_Text* t;
    if (fe.n_texts >= (int) (sizeof fe.texts / sizeof fe.texts[0])) {
        return NULL;
    }
    t = HSD_SisLib_803A6754(0, fe.canvas);
    if (t == NULL) {
        return NULL;
    }
    t->pos_x = x;
    t->pos_y = y;
    t->pos_z = 0.0F;
    t->font_size.x = scale;
    t->font_size.y = scale;
    t->default_kerning = 1;
    HSD_SisLib_803A74F0(t, HSD_SisLib_803A6B98(t, 0.0F, 0.0F, s), &color);
    fe.texts[fe.n_texts++] = t;
    return t;
}

/* ---- the scene ----------------------------------------------------------------------------- */

void gm_Scene_Frontend_OnEnter(void* enter_data)
{
    const FrontendScreen* s = fe.screen;
    HSD_GObj* gobj;
    int i;
    (void) enter_data;

    fe.cursor = 0;
    fe.frames = 0;
    fe.result = 0;
    fe.n_texts = 0;
    if (s == NULL) {
        return;
    }

    /* The text canvas makes its own 640x480 orthographic camera; the panels draw on its link,
       below the text. */
    fe.canvas = HSD_SisLib_803A611C(0, NULL, 0x13, 0x14, 0, FE_GX_LINK, 10, 0);
    gobj = GObj_Create(0xE, 0xF, 0);
    if (gobj != NULL) {
        GObj_SetupGXLink(gobj, fe_draw, FE_GX_LINK, 0);
    }

    fe_text(64, 26, 1.1F, fe_rgba(255, 255, 255, 255), s->title);
    fe_text(66, 64, 0.55F, fe_rgba(150, 185, 255, 255), s->subtitle);
    fe_text(64, 112, 0.5F, fe_rgba(170, 176, 196, 255), s->note);
    for (i = 0; i < s->n_items; i++) {
        fe_text(FE_ROW_X + 24, FE_ROW_Y + FE_ROW_STEP * i + 11, 0.7F,
                fe_rgba(245, 247, 255, 255), s->items[i]);
    }
    fe_text(64, 448, 0.55F, fe_rgba(200, 206, 224, 255), "A  Select        B  Back");
}

void gm_Scene_Frontend_OnFrame(void)
{
    u32 in;
    fe.frames++;
    if (fe.screen == NULL) {
        gm_ChangeGameModeAfterCurrentScene(fe.continue_to);
        gm_801A4B60();
        return;
    }
    if (fe.result != 0) {
        gm_ChangeGameModeAfterCurrentScene(fe.result == 1 ? fe.continue_to : fe.back_to);
        gm_801A4B60();
        return;
    }
    if (fe.frames < 4) {
        return; /* let the button that brought us here go */
    }
    in = mn_80229624(4);
    if (in & MenuInput_Up) {
        fe.cursor = (fe.cursor + fe.screen->n_items - 1) % fe.screen->n_items;
        sfxMove();
    } else if (in & MenuInput_Down) {
        fe.cursor = (fe.cursor + 1) % fe.screen->n_items;
        sfxMove();
    } else if (in & MenuInput_Confirm) {
        if (fe.cursor == fe.screen->continue_item) {
            sfxForward();
            fe.result = 1;
        } else {
            sfxBack();
            fe.result = 2;
        }
    } else if (in & MenuInput_Back) {
        sfxBack();
        fe.result = 2;
    }
}

void gm_Scene_Frontend_OnExit(void* exit_data)
{
    int i;
    (void) exit_data;
    for (i = 0; i < fe.n_texts; i++) {
        HSD_SisLib_803A5CC4(fe.texts[i]);
    }
    fe.n_texts = 0;
}

static u8 fe_enter_data[4];

GameModeState gm_Mode_Frontend_States[] = {
    {
        0,
        lbDvdPreload_2,
        0,
        NULL,
        NULL,
        {
            GS_FRONTEND,
            fe_enter_data,
            NULL,
        },
    },
    { GM_GAMEMODESTATE_TERMINATE },
};

#endif
