#include "gw_ui_focus.h"

#include <math.h>
#include <stddef.h>

static int cx_of(const AtFocusBlock *b, int i) { return b->col0 + i % b->cols; }
static int cy_of(const AtFocusBlock *b, int i) { return b->row0 + i / b->cols; }
static int present(const AtFocusBlock *b, int i) { return b->exists == NULL || b->exists[i]; }

AtFocusPos at_focus_first(const AtFocusBlock *bl, int nb)
{
    int b, i;
    AtFocusPos none = { -1, -1 };
    for (b = 0; b < nb; b++)
        for (i = 0; i < bl[b].n; i++)
            if (present(&bl[b], i)) { AtFocusPos p = { b, i }; return p; }
    return none;
}

AtFocusPos at_focus_move(const AtFocusBlock *bl, int nb, AtFocusPos cur, int dir, int wrap)
{
    int fx, fy, b, i, have = 0, pass;
    double best = 0.0;
    AtFocusPos out = cur;
    if (cur.block < 0 || cur.block >= nb || cur.index < 0 || cur.index >= bl[cur.block].n || !present(&bl[cur.block], cur.index))
        return at_focus_first(bl, nb);
    fx = cx_of(&bl[cur.block], cur.index);
    fy = cy_of(&bl[cur.block], cur.index);
    for (pass = 0; pass < 2 && !have; pass++) {
        if (pass == 1 && !wrap) break;
        for (b = 0; b < nb; b++) {
            for (i = 0; i < bl[b].n; i++) {
                int dx, dy, horizontal = (dir == AT_DIR_LEFT || dir == AT_DIR_RIGHT);
                double prim, perp, s;
                if (!present(&bl[b], i) || (b == cur.block && i == cur.index)) continue;
                dx = cx_of(&bl[b], i) - fx;
                dy = cy_of(&bl[b], i) - fy;
                perp = fabs((double) (horizontal ? dy : dx));
                if (pass == 0) {
                    prim = dir == AT_DIR_RIGHT ? dx : dir == AT_DIR_LEFT ? -dx : dir == AT_DIR_DOWN ? dy : -dy;
                    if (prim <= 0.01) continue;
                    s = prim + 2.0 * perp;
                } else {
                    int ax = cx_of(&bl[b], i), ay = cy_of(&bl[b], i);
                    prim = dir == AT_DIR_RIGHT ? ax : dir == AT_DIR_LEFT ? -ax : dir == AT_DIR_DOWN ? ay : -ay;
                    s = prim + 4.0 * perp;
                }
                if (!have || s < best) { best = s; out.block = b; out.index = i; have = 1; }
            }
        }
    }
    return out;
}

int at_repeat_step(AtRepeat *r, int held, double now)
{
    if (held == 0) { r->dir = 0; return 0; }
    if (held != r->dir) { r->dir = held; r->down_ms = now; r->last_ms = now; return held; }
    if (now - r->down_ms >= 300.0 && now - r->last_ms >= 80.0) { r->last_ms = now; return held; }
    return 0;
}
