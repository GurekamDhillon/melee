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
