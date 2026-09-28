/* Included by gmfrontend.c after the kit, TARGET_PC only.
 * Geometry is in 640x480 layout units, not texture pixels. The kit applies its
 * shear after slicing; the legacy art adapter uses the same grid unsheared. */

enum {
    FPANEL_FILL, FPANEL_TOP, FPANEL_BOTTOM, FPANEL_LEFT, FPANEL_RIGHT,
    FPANEL_TL, FPANEL_TR, FPANEL_BL, FPANEL_BR, FPANEL_COUNT
};
enum {
    FPANEL_CUT_TL = 1, FPANEL_CUT_TR = 2,
    FPANEL_CUT_BL = 4, FPANEL_CUT_BR = 8
};

typedef struct FePanelRect {
    float x, y, w, h;
} FePanelRect;

typedef struct FePanel {
    FePanelRect bounds;
    float corner_w, corner_h;
    unsigned cuts; ///< chamfer the selected corner across its corner cell
    const char* fill;
    const char* edge;
    const char* corner; ///< NULL token omits that part, e.g. a hollow frame
    int section; ///< kit_colour's section for @face, @bg, etc.
    u8 role; ///< FPR_CHROME, FPR_DECOR, etc.; retained on every slice
} FePanel;

static float fe_panel_limit(float value, float max)
{
    /* Also reject NaN: never submit an inverted/non-finite slice to GX. */
    return !(value > 0) ? 0 : value > max ? max : value;
}

static void fe_panel_rect(FePanelRect* r, float x, float y, float w, float h)
{
    r->x = x; r->y = y; r->w = w; r->h = h;
}

/* Corners keep their size as the panel grows. Tiny panels shrink them to half
 * the extent, so opposite corners meet instead of crossing. Separate edge
 * thickness is for fe_draw_frame's original 64px corners / 16px edge art;
 * solid kit panels use ex=cx, ey=cy for a gap-free, non-overlapping nine-slice. */
static bool fe_panel_grid(const FePanelRect* b, float cx, float cy, float ex, float ey,
                          FePanelRect r[FPANEL_COUNT])
{
    float x = b->x, y = b->y, w = b->w, h = b->h;
    if (!(w > 0 && h > 0) || !(x - x == 0 && y - y == 0 &&
                              w - w == 0 && h - h == 0 &&
                              (x + w) - (x + w) == 0 && (y + h) - (y + h) == 0)) {
        return false;
    }
    cx = fe_panel_limit(cx, w * 0.5F);
    cy = fe_panel_limit(cy, h * 0.5F);
    ex = fe_panel_limit(ex, cx);
    ey = fe_panel_limit(ey, cy);
    fe_panel_rect(&r[FPANEL_FILL], x + cx, y + cy, w - 2 * cx, h - 2 * cy);
    fe_panel_rect(&r[FPANEL_TOP], x + cx, y, w - 2 * cx, ey);
    fe_panel_rect(&r[FPANEL_BOTTOM], x + cx, y + h - ey, w - 2 * cx, ey);
    fe_panel_rect(&r[FPANEL_LEFT], x, y + cy, ex, h - 2 * cy);
    fe_panel_rect(&r[FPANEL_RIGHT], x + w - ex, y + cy, ex, h - 2 * cy);
    fe_panel_rect(&r[FPANEL_TL], x, y, cx, cy);
    fe_panel_rect(&r[FPANEL_TR], x + w - cx, y, cx, cy);
    fe_panel_rect(&r[FPANEL_BL], x, y + h - cy, cx, cy);
    fe_panel_rect(&r[FPANEL_BR], x + w - cx, y + h - cy, cx, cy);
    return true;
}

/* Add chrome to the existing player; no extra renderer, heap, or texture asset.
 * The id prefix is kept on every slice so header/footer joint assignment works.
 * Return the first quad, or -1 without adding a partial panel. Call during screen
 * construction, before fp_reset_channels. Tokens resolve at construction time.
 * NULL is transparent; unknown tokens use the kit's usual white fallback. */
static int fe_panel_add(const char* id, const FePanel* p)
{
    static const char* const suffix[FPANEL_COUNT] = {
        "fill", "top", "bottom", "left", "right", "tl", "tr", "bl", "br"
    };
    FePanelRect r[FPANEL_COUNT];
    int i, first = fp.nq, count = 0;
    int section = p->section >= 0 && p->section < KIT_SECTIONS ? p->section : KS_VERSUS;
    if (!fe_panel_grid(&p->bounds, p->corner_w, p->corner_h,
                       p->corner_w, p->corner_h, r)) {
        OSReport("frontend: panel %s skipped (invalid bounds)\n", id);
        return -1;
    }
    for (i = 0; i < FPANEL_COUNT; i++) {
        const char* token = i == FPANEL_FILL ? p->fill : i < FPANEL_TL ? p->edge : p->corner;
        if (token != NULL && r[i].w > 0 && r[i].h > 0) {
            count++;
        }
    }
    if (count == 0 || fp.nq + count > FP_MAX_QUADS) {
        OSReport("frontend: panel %s skipped (%d slices, %d free)\n", id, count,
                 FP_MAX_QUADS - fp.nq);
        return -1;
    }
    for (i = 0; i < FPANEL_COUNT; i++) {
        const char* token = i == FPANEL_FILL ? p->fill : i < FPANEL_TL ? p->edge : p->corner;
        FpQuad* q;
        char name[FP_ID];
        int qi, k;
        if (token == NULL || r[i].w <= 0 || r[i].h <= 0) {
            continue;
        }
        /* Leave room for '_' and the longest suffix including its terminator. */
        fp_copy(name, id, FP_ID - 7);
        {
            /* no strcat: the game TUs link only the libc the shims provide (gw_strcat is not one) */
            char tail[16];
            sprintf(tail, "_%s", suffix[i]);
            fp_copy(name + strlen(name), tail, FP_ID - (int) strlen(name));
        }
        qi = fp_add_quad(name, FPK_FLAT, p->role, -1);
        q = &fp.q[qi];
        q->has_colour = true;
        if (!kit_colour(token, section, q->rgb)) {
            q->rgb[0] = q->rgb[1] = q->rgb[2] = 1.0F;
        }
        q->v[0][0] = q->v[3][0] = r[i].x;
        q->v[1][0] = q->v[2][0] = r[i].x + r[i].w;
        q->v[0][1] = q->v[1][1] = r[i].y;
        q->v[2][1] = q->v[3][1] = r[i].y + r[i].h;
        if (i >= FPANEL_TL && (p->cuts & (1U << (i - FPANEL_TL)))) {
            /* Duplicate a neighbouring vertex: one triangle becomes degenerate,
             * leaving exactly the inner half of the corner, with no overdraw. */
            int outer = i == FPANEL_TL ? 0 : i == FPANEL_TR ? 1 : i == FPANEL_BL ? 3 : 2;
            int next = (outer + 1) % 4;
            q->v[outer][0] = q->v[next][0];
            q->v[outer][1] = q->v[next][1];
        }
        for (k = 0; k < 4; k++) {
            q->v[k][0] += (fp.shear_y0 - q->v[k][1]) * fp.shear_s;
        }
    }
    OSReport("frontend: parametric panel %s: %d slices, cuts=0x%x\n", id, count, p->cuts);
    return first;
}

/* Compatibility skin for the original art pack: preserve its gate, draw order,
 * UV flips and white tint. The fill remains the independently placed panel_bg;
 * callers can now place/resize this frame without scaling its corner artwork. */
static void fe_panel_art_frame(const FePanelRect* bounds, float corner, float edge)
{
    static const int tex[FPANEL_COUNT] = {
        FT_PANEL, FT_EDGE_H, FT_EDGE_H, FT_EDGE_V, FT_EDGE_V,
        FT_FRAME_TL, FT_FRAME_TR, FT_FRAME_BL, FT_FRAME_BR
    };
    FePanelRect r[FPANEL_COUNT];
    GXColor white = fe_rgba(255, 255, 255, 255);
    int i;
    if (!fe_tex[FT_EDGE_H].ok || !fe_tex[FT_FRAME_TL].ok ||
        !fe_panel_grid(bounds, corner, corner, edge, edge, r)) {
        return;
    }
    for (i = FPANEL_TOP; i < FPANEL_COUNT; i++) {
        if (r[i].w > 0 && r[i].h > 0) {
            fe_tex_quad_uv(&fe_tex[tex[i]], r[i].x, r[i].y, r[i].w, r[i].h,
                           i == FPANEL_RIGHT ? 1 : 0, i == FPANEL_BOTTOM ? 1 : 0,
                           i == FPANEL_RIGHT ? 0 : 1, i == FPANEL_BOTTOM ? 0 : 1, white);
        }
    }
}
