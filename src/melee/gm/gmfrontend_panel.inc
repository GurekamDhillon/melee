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

/* Layout files may opt into different corners without re-exporting vertices.
 * Old files keep their exact outline: a solid panel with four square corners.
 * cuts uses FPANEL_CUT_*; hollow omits only the centre cell. */
static void fe_panel_read(FpQuad* q, const FjNode* n)
{
    const FjNode* p = fj_get(n, "panel");
    const FjNode* corner = fj_get(p, "corner");
    if (q->kind != FPK_PANEL || p == NULL) {
        return;
    }
    q->corner_w = fj_num(fj_idx(corner, 0), 4.0F);
    q->corner_h = fj_num(fj_idx(corner, 1), 4.0F);
    q->cuts = (u8) fe_panel_limit(fj_num(fj_get(p, "cuts"), 0), 15);
    q->hollow = fj_get(p, "hollow") != NULL && fj_get(p, "hollow")->type == FJ_TRUE;
}

/* Slice a logical quad in rest space, then map into its evaluated vertices.
 * This retains the player's single ID, tile draw order, material channels,
 * late/toast flag and joint hierarchy (including slider scale animations).
 * Bilinear mapping also preserves the authored trapezoids in online layouts.
 * Opposite slices never overlap, including translucent and chamfered panels. */
static void fe_panel_draw(const FpQuad* q, GXColor c)
{
    FePanelRect b, r[FPANEL_COUNT];
    int i, k, axis;
    fe_panel_rect(&b, 0, 0, q->v[1][0] - q->v[0][0], q->v[3][1] - q->v[0][1]);
    if (!fe_panel_grid(&b, q->corner_w, q->corner_h,
                       q->corner_w, q->corner_h, r)) {
        return;
    }
    for (i = q->hollow ? FPANEL_TOP : FPANEL_FILL; i < FPANEL_COUNT; i++) {
        float v[4][2];
        if (r[i].w <= 0 || r[i].h <= 0) {
            continue;
        }
        for (k = 0; k < 4; k++) {
            float u = (r[i].x + (k == 1 || k == 2 ? r[i].w : 0)) / b.w;
            float t = (r[i].y + (k >= 2 ? r[i].h : 0)) / b.h;
            for (axis = 0; axis < 2; axis++) {
                float top = q->w[0][axis] + u * (q->w[1][axis] - q->w[0][axis]);
                float bottom = q->w[3][axis] + u * (q->w[2][axis] - q->w[3][axis]);
                v[k][axis] = top + t * (bottom - top);
            }
        }
        if (i >= FPANEL_TL && (q->cuts & (1U << (i - FPANEL_TL)))) {
            int outer = i == FPANEL_TL ? 0 : i == FPANEL_TR ? 1 : i == FPANEL_BL ? 3 : 2;
            int next = (outer + 1) % 4;
            v[outer][0] = v[next][0];
            v[outer][1] = v[next][1];
        }
        fp_flat(v, c);
    }
}

/* The legacy SisLib path also has solid slider/toggle furniture. It has no
 * player model, but shares the same bounds/clamping as the kit panels. */
static void fe_panel_solid(float x, float y, float w, float h, GXColor colour)
{
    FePanelRect bounds, r[FPANEL_COUNT];
    int i;
    fe_panel_rect(&bounds, x, y, w, h);
    if (!fe_panel_grid(&bounds, 4, 4, 4, 4, r)) return;
    for (i = 0; i < FPANEL_COUNT; i++) {
        if (r[i].w > 0 && r[i].h > 0) {
            fe_solid(r[i].x, r[i].y, r[i].w, r[i].h, colour);
        }
    }
}

/* Single-image legacy panels retain their authored UVs at the old size. On
 * resize, only the centre and edge spans stretch; corners keep layout size.
 * This preserves panel_bg's tab and the row/button shadows without new art. */
static void fe_panel_art_image(int tex, const FePanelRect* bounds,
                               float authored_w, float authored_h,
                               float cx, float cy, GXColor fallback)
{
    FePanelRect source, dst[FPANEL_COUNT], uv[FPANEL_COUNT];
    int i;
    fe_panel_rect(&source, 0, 0, authored_w, authored_h);
    if (!fe_panel_grid(bounds, cx, cy, cx, cy, dst) ||
        !fe_panel_grid(&source, cx, cy, cx, cy, uv)) {
        return;
    }
    for (i = 0; i < FPANEL_COUNT; i++) {
        if (dst[i].w <= 0 || dst[i].h <= 0) continue;
        if (fe_tex[tex].ok) {
            fe_tex_quad_uv(&fe_tex[tex], dst[i].x, dst[i].y, dst[i].w, dst[i].h,
                           uv[i].x / authored_w, uv[i].y / authored_h,
                           (uv[i].x + uv[i].w) / authored_w,
                           (uv[i].y + uv[i].h) / authored_h,
                           fe_rgba(255, 255, 255, 255));
        } else {
            fe_solid(dst[i].x, dst[i].y, dst[i].w, dst[i].h, fallback);
        }
    }
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
        if (strcmp(id, "desc_strip") == 0) {
            q->canvas_y0 = p->bounds.y;
            q->canvas_y1 = p->bounds.y + p->bounds.h;
            q->canvas_x0 = p->bounds.x;
            q->canvas_x1 = p->bounds.x + p->bounds.w;
            q->canvas_edge = i == FPANEL_LEFT || i == FPANEL_TL || i == FPANEL_BL ? 2 :
                             i == FPANEL_RIGHT || i == FPANEL_TR || i == FPANEL_BR ? 3 : 1;
        }
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

/* Authored frame colours are never multiplied by the kit palette. Hollow is
 * the legacy screen surround; filled panels use frame_fill in the centre cross
 * only, excluding all four corner cells, including transparent chamfers. */
static void fe_panel_art_frame(const FePanelRect* bounds, float corner, float edge,
                               unsigned cuts, bool filled)
{
    static const int tex[FPANEL_COUNT] = {
        FT_PANEL, FT_EDGE_H, FT_EDGE_H, FT_EDGE_V, FT_EDGE_V,
        FT_FRAME_TL, FT_FRAME_TR, FT_FRAME_BL, FT_FRAME_BR
    };
    FePanelRect r[FPANEL_COUNT];
    GXColor white = fe_rgba(255, 255, 255, 255);
    int i;
    if (!fe_panel_grid(bounds, corner, corner, edge, edge, r)) {
        return;
    }
    /* A missing cut-corner asset must not silently turn into a square corner.
     * Texture loading reports missing art once, outside the draw callback. */
    for (i = filled ? FPANEL_FILL : FPANEL_TOP; i < FPANEL_COUNT; i++) {
        int t = i == FPANEL_FILL ? FT_FRAME_FILL : tex[i];
        if (i >= FPANEL_TL && (cuts & (1U << (i - FPANEL_TL)))) {
            t = FT_FRAME_CUT_TL + i - FPANEL_TL;
        }
        if (!fe_tex[t].ok) return;
    }
    if (filled) {
        FePanelRect fill[5];
        /* Edges are thinner than corner cells in this skin. Fill the four
         * inner edge gaps as well as the middle, never beneath an art slice. */
        fill[0] = r[FPANEL_FILL];
        fe_panel_rect(&fill[1], r[FPANEL_TOP].x, r[FPANEL_TOP].y + r[FPANEL_TOP].h,
                       r[FPANEL_TOP].w, r[FPANEL_TL].h - r[FPANEL_TOP].h);
        fe_panel_rect(&fill[2], r[FPANEL_BOTTOM].x, r[FPANEL_BL].y,
                       r[FPANEL_BOTTOM].w, r[FPANEL_BL].h - r[FPANEL_BOTTOM].h);
        fe_panel_rect(&fill[3], r[FPANEL_LEFT].x + r[FPANEL_LEFT].w, r[FPANEL_LEFT].y,
                       r[FPANEL_TL].w - r[FPANEL_LEFT].w, r[FPANEL_LEFT].h);
        fe_panel_rect(&fill[4], r[FPANEL_TR].x, r[FPANEL_RIGHT].y,
                       r[FPANEL_TR].w - r[FPANEL_RIGHT].w, r[FPANEL_RIGHT].h);
        for (i = 0; i < 5; i++) {
            if (fill[i].w > 0 && fill[i].h > 0) {
                fe_tex_quad(&fe_tex[FT_FRAME_FILL], fill[i].x, fill[i].y,
                            fill[i].w, fill[i].h, white);
            }
        }
    }
    for (i = FPANEL_TOP; i < FPANEL_COUNT; i++) {
        int t = tex[i];
        if (i >= FPANEL_TL && (cuts & (1U << (i - FPANEL_TL)))) {
            t = FT_FRAME_CUT_TL + i - FPANEL_TL;
        }
        if (r[i].w > 0 && r[i].h > 0) {
            fe_tex_quad_uv(&fe_tex[t], r[i].x, r[i].y, r[i].w, r[i].h,
                           i == FPANEL_RIGHT ? 1 : 0, i == FPANEL_BOTTOM ? 1 : 0,
                           i == FPANEL_RIGHT ? 0 : 1, i == FPANEL_BOTTOM ? 0 : 1, white);
        }
    }
}
