/* gw_ui_stack.h - the Atlas screen stack and the tween. Pure C. */
#ifndef GW_UI_STACK_H
#define GW_UI_STACK_H
#include "gw_ui_focus.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_STACK_MAX 8
typedef struct { int screen; AtFocusPos focus; int scroll; } AtStackEntry;
typedef struct { AtStackEntry e[AT_STACK_MAX]; int n; } AtStack;

int at_stack_push(AtStack *s, int screen);        /* 1 pushed; 0 when full or already on top */
int at_stack_pop(AtStack *s);                      /* the popped screen, -1 when empty */
int at_stack_remove(AtStack *s, int screen);       /* 1 removed (from anywhere in the stack) */
int at_stack_top(const AtStack *s);                /* the top screen, -1 when empty */
AtStackEntry *at_stack_top_entry(AtStack *s);      /* NULL when empty; the caller saves focus and scroll here */

typedef struct { double t0, dur; } AtTween;
void at_tween_start(AtTween *t, double now_ms, double dur_ms, int reduced);   /* reduced: a cut */
float at_tween_value(const AtTween *t, double now_ms);                        /* 0..1, ease-out cubic; 1 when dur <= 0 */

#ifdef __cplusplus
}
#endif
#endif
