/* gw_ui_item.c - the legacy fe_change rule and a row's text; pure C (see gw_ui_item.h). */
#include <stdio.h>
#include <string.h>
#include "gw_ui_item.h"

int at_item_apply(int vkind, int vmin, int vmax, int vstep, int cur, int dir, int *v)
{
    int n, nv;
    if (v == NULL) return 0;
    if (dir != -1 && dir != 1) dir = dir < 0 ? -1 : 1;
    switch (vkind) {
    case AT_IVK_TOGGLE:
        *v = !cur;
        return 1;
    case AT_IVK_CHOICE:
        n = vmax - vmin + 1;
        if (n <= 1) return 0;                                       /* an inverted range, or one option: nothing to change to */
        nv = vmin + ((cur - vmin + dir) % n + n) % n;               /* wraps */
        if (nv == cur) return 0;
        *v = nv;
        return 1;
    case AT_IVK_SLIDER:
        if (vmax <= vmin) return 0;                                 /* no range */
        nv = cur + dir * (vstep > 0 ? vstep : 1);
        nv = nv < vmin ? vmin : nv > vmax ? vmax : nv;              /* clamps */
        if (nv == cur) return 0;                                    /* held at an end: the caller bumps */
        *v = nv;
        return 1;
    default:
        return 0;
    }
}

static void put(char *out, int cap, const char *s)
{
    if (cap <= 0) return;
    snprintf(out, (size_t) cap, "%s", s != NULL ? s : "");
}

void at_item_text(int vkind, int value, int vmin, const char (*opts)[24], int n_opts, const char *text, char *out, int cap)
{
    char num[16];
    int i;
    if (cap <= 0 || out == NULL) return;
    switch (vkind) {
    case AT_IVK_TOGGLE:
        put(out, cap, value ? "ON" : "OFF");
        return;
    case AT_IVK_CHOICE:
        if (n_opts <= 0 || opts == NULL) { put(out, cap, text); return; }
        i = value - vmin;
        put(out, cap, (i >= 0 && i < n_opts) ? opts[i] : "");       /* out of range: empty, never a read past the options */
        return;
    case AT_IVK_SLIDER:
        if (text != NULL && text[0] != '\0') { put(out, cap, text); return; }
        snprintf(num, sizeof num, "%d", value);
        put(out, cap, num);
        return;
    case AT_IVK_TEXT:
    case AT_IVK_COUNTER:
        put(out, cap, text);
        return;
    default:
        out[0] = '\0';
        return;
    }
}
