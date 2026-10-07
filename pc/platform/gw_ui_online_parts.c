#include "gw_ui_online_parts.h"
#include "gw_ui_tokens.h"
#include <ctype.h>
#include <stdio.h>
#include <string.h>

/* bars b holds up to this many ms (4: 50, 3: 90, 2: 140; 1 has no upper line). The lines are a proposal, unmeasured against play. */
static int limit_ms(int bars) { return bars == 4 ? 50 : bars == 3 ? 90 : bars == 2 ? 140 : 1000000; }

int at_link_bars(int ms, int prev)
{
    int b = prev;
    if (ms < 0) return 0;
    if (b < 1 || b > 4) b = ms <= 50 ? 4 : ms <= 90 ? 3 : ms <= 140 ? 2 : 1;   /* the first reading has no memory */
    while (b > 1 && ms > limit_ms(b) + 8) b--;
    while (b < 4 && ms <= limit_ms(b + 1) - 8) b++;
    return b;
}

AtLinkInfo at_link_info(int bars)
{
    AtLinkInfo i;
    if (bars < 0 || bars > 4) bars = 0;
    i.bars = bars;
    switch (bars) {
    case 4: case 3: i.word = "GOOD"; i.rgba = AT_C_JADE; break;
    case 2: i.word = "FAIR"; i.rgba = AT_C_SUN; break;
    case 1: i.word = "POOR"; i.rgba = AT_C_ROSE; break;
    default: i.word = "NO LINK"; i.rgba = AT_C_DIM; break;
    }
    return i;
}

float at_part_linkmeter(const AtSink *s, const AtTextOps *o, float x, float base, int bars, int ms, int with_word)
{
    AtLinkInfo li = at_link_info(bars);
    float bx = x, w;
    int i;
    for (i = 0; i < 4; i++) {
        float h = 5.0f + 3.0f * (float) i;
        at_poly_rect(s, bx, base - h, 3.0f, h, i < li.bars ? li.rgba : AT_C_LINE);
        bx += 5.0f;
    }
    w = bx - x - 2.0f;                                                   /* 4 bars of 3 px with 2 px gaps: 18 px */
    if (with_word) {
        at_text(s, o, AT_R_CAP12, li.word, x + w + 6.0f, base, li.bars ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
        w += 6.0f + o->width(o->user, AT_R_CAP12, li.word);
        if (ms >= 0 && li.bars > 0) {
            char num[16];
            snprintf(num, sizeof num, "%d ms", ms);
            at_text(s, o, AT_R_NUM12, num, x + w + 8.0f, base, AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
            w += 8.0f + o->width(o->user, AT_R_NUM12, num);
        }
    }
    return w;
}

/* ---- the room-code field ------------------------------------------------------------------------- */

AtRect at_code_slot_rect(AtRect f, int i)
{
    float gap = 12.0f, sw = (f.w - 3.0f * gap) / 4.0f, sh = f.h - 2.0f * AT_CODE_BAND;
    AtRect r;
    if (sw > 64.0f) sw = 64.0f;
    if (sw < 8.0f) sw = 8.0f;
    if (sh > 84.0f) sh = 84.0f;
    r.w = sw;
    r.h = sh;
    r.x = f.x + (f.w - (4.0f * sw + 3.0f * gap)) * 0.5f + (float) i * (sw + gap);
    r.y = f.y + (f.h - sh) * 0.5f;
    return r;
}

int at_code_hit(AtRect f, int active, float px, float py, int *arg)
{
    int i;
    for (i = 0; i < 4; i++) {
        AtRect r = at_code_slot_rect(f, i);
        if (px < r.x || px >= r.x + r.w) continue;
        if (py >= r.y && py < r.y + r.h) { *arg = i; return 1; }
        if (i == active && py >= r.y - AT_CODE_BAND && py < r.y) return 2;
        if (i == active && py >= r.y + r.h && py < r.y + r.h + AT_CODE_BAND) return 3;
    }
    return 0;
}

/* one registration bracket: two 2 px strips meeting in the corner (x, y); sx and sy are +1 or -1, the way it points inward */
static void bracket(const AtSink *s, float x, float y, float sx, float sy, unsigned c)
{
    at_poly_rect(s, sx > 0.0f ? x : x - 10.0f, sy > 0.0f ? y : y - 2.0f, 10.0f, 2.0f, c);
    at_poly_rect(s, sx > 0.0f ? x : x - 2.0f, sy > 0.0f ? y : y - 10.0f, 2.0f, 10.0f, c);
}

static void chevron(const AtSink *s, float cx, float y, int up, unsigned c)
{
    float x[4] = { cx - 7.0f, cx + 7.0f, cx, cx };
    float ya = up ? y + 8.0f : y, yb = up ? y : y + 8.0f;
    float yy[4];
    yy[0] = ya; yy[1] = ya; yy[2] = yb; yy[3] = yb;
    s->poly(s->user, x, yy, c);
}

void at_part_code(const AtSink *s, const AtTextOps *o, AtRect f, const AtCodeView *v)
{
    int i;
    for (i = 0; i < 4; i++) {
        AtRect r = at_code_slot_rect(f, i), pr;
        int active = i == v->slot;
        float y = active ? r.y - 2.0f : r.y, cx = r.x + r.w * 0.5f;
        unsigned face = active ? AT_C_LIFT : AT_C_PLATE2;
        unsigned edge = active ? AT_C_EMBER : v->invalid ? AT_C_ROSE : AT_C_EDGE2;
        pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
        at_plate(s, pr, face, edge, 3.0f, (float) AT_PX_CH_XS);
        if (v->c[i] != '\0') {
            char one[2];
            one[0] = (char) toupper((unsigned char) v->c[i]);
            one[1] = '\0';
            at_text(s, o, AT_R_DISPLAY, one, cx, y + r.h * 0.5f + 22.0f, AT_C_IVORY, AT_ALIGN_CENTER, r.w - 8.0f);
        } else {
            at_text(s, o, AT_R_NUM16, "-", cx, y + r.h * 0.5f + 6.0f, AT_C_DIM, AT_ALIGN_CENTER, 0.0f);
        }
        if (active) {
            bracket(s, r.x - 3.0f, y - 3.0f, 1.0f, 1.0f, AT_C_EMBER);
            bracket(s, r.x + r.w + 3.0f, y - 3.0f, -1.0f, 1.0f, AT_C_EMBER);
            bracket(s, r.x - 3.0f, y + r.h + 3.0f, 1.0f, -1.0f, AT_C_EMBER);
            bracket(s, r.x + r.w + 3.0f, y + r.h + 3.0f, -1.0f, -1.0f, AT_C_EMBER);
            chevron(s, cx, r.y - 14.0f, 1, AT_C_MUTED);
            chevron(s, cx, r.y + r.h + 6.0f, 0, AT_C_MUTED);
        }
    }
}
