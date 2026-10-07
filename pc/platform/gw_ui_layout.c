#include "gw_ui_layout.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

static const AtRoleInfo roles[AT_R_COUNT] = {
    { "a_cap12", 12, 0, 0, 1, -1 },        { "a_cap14", 14, 0, 0, 1, AT_R_CAP12 },
    { "a_cap16", 16, 0, 0, 1, AT_R_CAP14 }, { "a_cap20", 20, 0, 0, 1, AT_R_CAP16 },
    { "a_title", 28, 0, 0, 0, AT_R_CAP20 }, { "a_hero", 44, 0, 1, 0, AT_R_TITLE },
    { "a_display", 64, 0, 1, 0, AT_R_HERO },
    { "a_body12", 12, 1, 0, 0, -1 },       { "a_body14", 14, 1, 0, 0, AT_R_BODY12 },
    { "a_row16", 16, 1, 0, 0, AT_R_BODY14 },
    { "a_num12", 12, 2, 0, 0, -1 },        { "a_num14", 14, 2, 0, 0, AT_R_NUM12 },
    { "a_num16", 16, 2, 0, 0, AT_R_NUM14 },
};

const AtRoleInfo *at_role(int role) { return &roles[role >= 0 && role < AT_R_COUNT ? role : AT_R_BODY14]; }
int at_role_size(int role) { return at_role(role)->size; }
int at_role_by_name(const char *name)
{
    int i;
    for (i = 0; name != NULL && i < AT_R_COUNT; i++) if (strcmp(roles[i].name, name) == 0) return i;
    return -1;
}

static int prev_char(const char *s, int n) /* byte length of s[0..n) minus its last UTF-8 character */
{
    n--;
    while (n > 0 && ((unsigned char) s[n] & 0xC0) == 0x80) n--;
    return n;
}

/* Truncate with an ellipsis inside ONE role (the role is not changed). */
static void truncate_in(const AtTextOps *ops, int role, const char *s, float max_w, char *out, int cap)
{
    int n;
    snprintf(out, (size_t) cap, "%s", s);
    n = (int) strlen(out);
    while (n > 1 && ops->width(ops->user, role, out) > max_w) {
        n = prev_char(s, n);
        if (n < 1) n = 1;
        if (n + 4 > cap) n = cap - 4;
        memcpy(out, s, (size_t) n);
        memcpy(out + n, "\xE2\x80\xA6", 4); /* the ellipsis and the terminating NUL */
    }
}

int at_fit(const AtTextOps *ops, int role, const char *s, float max_w, char *out, int cap)
{
    snprintf(out, (size_t) cap, "%s", s);
    if (max_w <= 0.0f) return role;
    while (ops->width(ops->user, role, out) > max_w && at_role(role)->smaller >= 0) role = at_role(role)->smaller;
    truncate_in(ops, role, s, max_w, out, cap);
    return role;
}

static int push_line(char lines[][96], int *n, int max_lines, const char *text, int *clamped)
{
    if (*n >= max_lines) { *clamped = 1; return 0; }
    snprintf(lines[*n], 96, "%s", text);
    (*n)++;
    return 1;
}

int at_wrap(const AtTextOps *ops, int role, const char *s, float max_w, int max_lines, char lines[][96], int *clamped)
{
    char cur[96] = "", word[96], cand[200], fit[96];
    int n = 0, cl = 0;
    const char *p = s;
    while (*p != '\0' && !cl) {
        int k = 0;
        if (*p == '\n') {
            if (!push_line(lines, &n, max_lines, cur, &cl)) break;
            cur[0] = '\0';
            p++;
            continue;
        }
        if (*p == ' ') { p++; continue; }
        while (p[k] != '\0' && p[k] != ' ' && p[k] != '\n' && k < 95) { word[k] = p[k]; k++; }
        word[k] = '\0';
        p += k;
        snprintf(cand, sizeof cand, "%s%s%s", cur, cur[0] ? " " : "", word);
        if (ops->width(ops->user, role, cand) <= max_w || max_w <= 0.0f) {
            snprintf(cur, sizeof cur, "%s", cand);
        } else {
            if (cur[0] != '\0' && !push_line(lines, &n, max_lines, cur, &cl)) break;
            truncate_in(ops, role, word, max_w, fit, sizeof fit);   /* an overlong single word is cut, in the same role */
            snprintf(cur, sizeof cur, "%s", fit);
        }
    }
    if (!cl && (cur[0] != '\0' || n == 0)) push_line(lines, &n, max_lines, cur, &cl);
    if (cl && n > 0) {                                          /* the last allowed line says it was cut */
        char tmp[96];
        snprintf(tmp, sizeof tmp, "%.90s\xE2\x80\xA6", lines[n - 1]);
        truncate_in(ops, role, tmp, max_w, lines[n - 1], 96);
    }
    if (clamped) *clamped = cl;
    return n;
}

void at_layout(float canvas_w, int preset, AtLayout *o)
{
    static const float base[3] = { 160.0f, 196.0f, 256.0f };
    float w = canvas_w < 640.0f ? 640.0f : canvas_w, cw, cx, L, R, px, ew = 0.0f;
    memset(o, 0, sizeof *o);
    o->wide = w >= 760.0f;
    cw = w < 1140.0f ? w : 1140.0f;
    cx = (w - cw) * 0.5f;
    L = cx + 32.0f;
    R = cx + cw - 32.0f;
    o->canvas = (AtRect){ 0.0f, 0.0f, w, 480.0f };
    o->content_x = cx;
    o->content_w = cw;
    o->header = (AtRect){ L, 22.0f, R - L, 30.0f };
    o->trail = (AtRect){ L, 22.0f, R - L - (o->wide ? 0.0f : 124.0f), 30.0f };
    o->rule = (AtRect){ L, 56.0f, R - L, 1.0f };
    o->body = (AtRect){ L, 66.0f, R - L, 362.0f };
    o->keys = (AtRect){ L, 434.0f, R - L, 26.0f };
    if (o->wide) o->rail = (AtRect){ L, 66.0f, 104.0f, 362.0f };
    else o->chapter = (AtRect){ R - 112.0f, 26.0f, 112.0f, 20.0f };
    px = L + (o->wide ? 116.0f : 0.0f);
    if (preset >= AT_PRESET_NARROW && preset <= AT_PRESET_WIDE) {
        ew = base[preset];
        if (o->wide) ew += (float) floor((cw - 640.0f) * 0.35f + 0.5f);
    }
    o->explainer = (AtRect){ ew > 0.0f ? R - ew : R, 66.0f, ew, 362.0f };
    o->primary = (AtRect){ px, 66.0f, (ew > 0.0f ? R - ew - 12.0f : R) - px, 362.0f };
}

int at_list_scroll(int focus, int scroll, int visible, int n)
{
    if (visible < 1) visible = 1;
    if (focus < scroll) scroll = focus;
    if (focus >= scroll + visible) scroll = focus - visible + 1;
    if (scroll > n - visible) scroll = n - visible;
    return scroll < 0 ? 0 : scroll;
}

void at_layout_split(const AtLayout *L, int has_tabs, int band, AtSplit *o)
{
    const float gap = 12.0f;
    float tabs_h = has_tabs ? 30.0f : 0.0f, band_h = band == 2 ? 40.0f : (band == 1 ? 56.0f : 0.0f), top = L->primary.y, bottom = L->primary.y + L->primary.h;
    o->tabs = (AtRect){ L->primary.x, top, L->primary.w, tabs_h };
    o->band = (AtRect){ L->primary.x, bottom - band_h, L->primary.w, band_h };
    o->grid = (AtRect){ L->primary.x, top + tabs_h, L->primary.w, bottom - band_h - (band_h > 0.0f ? gap : 0.0f) - (top + tabs_h) };
    if (o->grid.h < 0.0f) o->grid.h = 0.0f;
}

int at_grid_cols(float width, float min_cell, float max_cell, float gap)
{
    int n = (int) floor((double) ((width + gap) / (min_cell + gap)));
    if (n < 1) n = 1;
    while (n < 512) {                                                  /* a cell wider than max: one more column, while the next still fits min */
        float cell = (width - (float) (n - 1) * gap) / (float) n, next = (width - (float) n * gap) / (float) (n + 1);
        if (cell <= max_cell || next < min_cell) break;
        n++;
    }
    return n;
}
