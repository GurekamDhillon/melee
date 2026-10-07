/* gw_ui_item.h - Atlas: the value rule of a list row (the legacy fe_change rule, pure) and the text a row shows.
 * NO libc include and no other header: game-side files include this one, and the PowerPC syntax check runs with -nostdinc. The value
 * kinds are plain ints equal to AT_VAL_* (gw_ui_screen.c asserts the two stay in step). */
#ifndef GW_UI_ITEM_H
#define GW_UI_ITEM_H
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_IVK_NONE, AT_IVK_TOGGLE, AT_IVK_CHOICE, AT_IVK_SLIDER, AT_IVK_TEXT, AT_IVK_COUNTER };

/* The legacy fe_change rule. dir -1 or +1. Returns 1 when the value changed (the caller then calls set), 0 when not (a slider at an end, a readout,
 * an inverted range, an unknown kind), and writes the new value to *v only when it returns 1. Toggle flips; slider adds dir * (step > 0 ? step : 1)
 * and CLAMPS; choice WRAPS over min..max. */
int at_item_apply(int vkind, int vmin, int vmax, int vstep, int cur, int dir, int *v);

/* Which text a row shows for a value: toggle ON/OFF; a choice's option (value - vmin) of `opts` (n_opts entries of 24 chars), or `text` when it has
 * no options; a slider's `text` (the game's own format, "45%") or its number; else `text`. Always terminated, cut to cap; cap <= 0 writes nothing. */
void at_item_text(int vkind, int value, int vmin, const char (*opts)[24], int n_opts, const char *text, char *out, int cap);

#ifdef __cplusplus
}
#endif
#endif
