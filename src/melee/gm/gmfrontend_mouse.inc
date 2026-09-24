/* gmfrontend_mouse.inc - included by gmfrontend.c (TARGET_PC only), after the kit.
 *
 * ---- THE MOUSE IN THE MENUS ---------------------------------------------------------------------
 *
 * Every frontend screen takes the mouse next to the controllers: pointing moves the screen's own
 * cursor (the same move a d-pad press makes, with its sound and motion), a left click selects
 * what is under the pointer, a right click is B, the wheel scrolls lists (or steps a value / turns
 * a page where that is what the screen scrolls), and the footer's button hints are buttons. The
 * pointer comes from the host (gw_console.cpp) in the same 640x480 space the screens are laid out
 * in; hit tests use the player's evaluated quads (fp.q[].w: sheared, scrolled and lifted exactly as
 * drawn) or, for the room and select screens' rects, the pointer unsheared into layout space.
 *
 * Mouse input is local UI input only: it turns into what a local pad press would have done on this
 * screen and never reaches the pads (HSD_PadCopyStatus) or the netplay input stream. Pads behave
 * exactly as before; the mouse only adds. A pointer resting still does nothing - hovering acts
 * only when it moves, so a controller moving the cursor is never pulled back. */

extern int Mouse_X(void);
extern int Mouse_Y(void);
extern int Mouse_Buttons(void);
extern int Mouse_MoveSeq(void);
extern int Mouse_Wheel(void);
extern void Mouse_UiFrame(void);

#define FMS_MAX_HINTS 8

static struct {
    float x, y;    ///< 640x480 (screen space, as drawn)
    float ux, uy;  ///< unsheared into layout space
    bool in;       ///< over the picture
    bool moved;    ///< moved since the last frame
    bool click;    ///< left button went down this frame
    bool rclick;   ///< right button went down
    bool mclick;   ///< middle button went down
    int wheel;     ///< notches this frame (+ up)
    int btn, seq;
    /* the footer's hints (fm_chrome_with), layout space; valid while their glyph quad is */
    int nhint;
    int hint_q[FMS_MAX_HINTS];
    const char* hint_glyph[FMS_MAX_HINTS];
    float hint_x0[FMS_MAX_HINTS], hint_x1[FMS_MAX_HINTS], hint_y0, hint_y1;
} fms;

/* Once per frontend frame, first thing: read the host's pointer and tell it a screen is up (the
 * cursor shows only while one is, or a script's menu is). */
static void fms_poll(void)
{
    int b = Mouse_Buttons(), seq = Mouse_MoveSeq();
    Mouse_UiFrame();
    fms.x = (float) Mouse_X();
    fms.y = (float) Mouse_Y();
    fms.in = fms.x > -999.0F;
    fms.ux = fms.x - (fp.shear_y0 - fms.y) * fp.shear_s;
    fms.uy = fms.y;
    fms.moved = fms.in && seq != fms.seq;
    fms.seq = seq;
    fms.click = fms.in && (b & 1) != 0 && (fms.btn & 1) == 0;
    fms.rclick = fms.in && (b & 2) != 0 && (fms.btn & 2) == 0;
    fms.mclick = fms.in && (b & 4) != 0 && (fms.btn & 4) == 0;
    fms.btn = b;
    fms.wheel = Mouse_Wheel();
    if (!fms.in) {
        fms.wheel = 0;
    }
}

/* Nothing this frame (a screen that is not taking input yet eats the click, so it does not land
   on the next screen). */
static void fms_clear(void)
{
    fms.moved = fms.click = fms.rclick = fms.mclick = false;
    fms.wheel = 0;
}

/* The pointer inside a convex quad given TL TR BR BL (either winding). */
static bool fms_in_verts(const float v[4][2])
{
    int i, pos = 0, neg = 0;
    for (i = 0; i < 4; i++) {
        const float* a = v[i];
        const float* b = v[(i + 1) & 3];
        float c = (b[0] - a[0]) * (fms.y - a[1]) - (b[1] - a[1]) * (fms.x - a[0]);
        if (c > 0.0F) pos++;
        if (c < 0.0F) neg++;
    }
    return fms.in && (pos == 0 || neg == 0);
}

/* A player quad as drawn this frame (hidden ones never hit). */
static bool fms_in_quad(int q)
{
    return q >= 0 && q < fp.nq && fp.q[q].wa > 0.05F && fms_in_verts(fp.q[q].w);
}

/* A layout-space rect (x0, y0, x1, y1), before the kit's shear. */
static bool fms_in_rect(const float* r)
{
    return fms.in && fms.ux >= r[0] && fms.ux < r[2] && fms.uy >= r[1] && fms.uy < r[3];
}

/* fm_chrome_with: a hint drawn at [x0, x1) of the footer. */
static void fms_hint_add(int glyph_q, const char* glyph, float x0, float x1, float y0, float y1)
{
    if (fms.nhint >= FMS_MAX_HINTS) {
        return;
    }
    fms.hint_q[fms.nhint] = glyph_q;
    fms.hint_glyph[fms.nhint] = glyph;
    fms.hint_x0[fms.nhint] = x0;
    fms.hint_x1[fms.nhint] = x1;
    fms.hint_y0 = y0;
    fms.hint_y1 = y1;
    fms.nhint++;
}

/* The hint glyph clicked this frame, or NULL. */
static const char* fms_hint_clicked(void)
{
    int i;
    if (!fms.click) {
        return NULL;
    }
    for (i = 0; i < fms.nhint; i++) {
        int q = fms.hint_q[i];
        float r[4];
        if (q < 0 || q >= fp.nq || strncmp(fp.q[q].id, "hint_", 5) != 0) {
            continue; /* the model was rebuilt without this footer */
        }
        r[0] = fms.hint_x0[i];
        r[1] = fms.hint_y0;
        r[2] = fms.hint_x1[i];
        r[3] = fms.hint_y1;
        if (fms_in_rect(r)) {
            return fms.hint_glyph[i];
        }
    }
    return NULL;
}

static bool fms_is(const char* glyph, const char* name)
{
    return glyph != NULL && strcmp(glyph, name) == 0;
}

/* The pointer over the footer strip (a click there is a hint's, or nothing). */
static bool fms_over_footer(void)
{
    return fms.nhint > 0 && fms.in && fms.uy >= fms.hint_y0 && fms.uy < fms.hint_y1;
}

/* What the mouse adds to a menu screen's mn_80229624 input: a right click is B, a clicked hint
 * is its button. Pointing and clicking on the screen's own items is each screen's (it knows its
 * items). */
static u32 fms_menu_bits(void)
{
    const char* g = fms_hint_clicked();
    u32 in = 0;
    if (fms.rclick) in |= MenuInput_Back;
    if (fms_is(g, "glyph_a")) in |= MenuInput_Confirm | MenuInput_AButton;
    if (fms_is(g, "glyph_b")) in |= MenuInput_Back;
    if (fms_is(g, "glyph_start")) in |= MenuInput_Confirm | MenuInput_StartButton;
    if (fms_is(g, "glyph_x")) in |= MenuInput_XButton;
    if (fms_is(g, "glyph_y")) in |= MenuInput_YButton;
    if (fms_is(g, "glyph_l")) in |= MenuInput_LTrigger;
    if (fms_is(g, "glyph_r")) in |= MenuInput_RTrigger;
    return in;
}

/* The same as pad buttons (the select screens read buttons per port). */
static u32 fms_pad_bits(void)
{
    const char* g = fms_hint_clicked();
    u32 b = 0;
    if (fms.rclick) b |= PAD_BUTTON_B;
    if (fms_is(g, "glyph_a")) b |= PAD_BUTTON_A;
    if (fms_is(g, "glyph_b")) b |= PAD_BUTTON_B;
    if (fms_is(g, "glyph_start")) b |= PAD_BUTTON_START;
    if (fms_is(g, "glyph_x")) b |= PAD_BUTTON_X;
    if (fms_is(g, "glyph_y")) b |= PAD_BUTTON_Y;
    if (fms_is(g, "glyph_l")) b |= PAD_TRIGGER_L;
    if (fms_is(g, "glyph_r")) b |= PAD_TRIGGER_R;
    return b;
}
