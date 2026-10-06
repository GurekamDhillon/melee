#include "gw_ui_stack.h"

#include <stddef.h>

int at_stack_push(AtStack *s, int screen)
{
    if (s->n >= AT_STACK_MAX || (s->n > 0 && s->e[s->n - 1].screen == screen)) return 0;
    s->e[s->n].screen = screen;
    s->e[s->n].focus.block = s->e[s->n].focus.index = -1;
    s->e[s->n].scroll = 0;
    s->n++;
    return 1;
}

int at_stack_pop(AtStack *s) { return s->n > 0 ? s->e[--s->n].screen : -1; }

int at_stack_remove(AtStack *s, int screen)
{
    int i, j;
    for (i = 0; i < s->n; i++) {
        if (s->e[i].screen != screen) continue;
        for (j = i; j + 1 < s->n; j++) s->e[j] = s->e[j + 1];
        s->n--;
        return 1;
    }
    return 0;
}

int at_stack_top(const AtStack *s) { return s->n > 0 ? s->e[s->n - 1].screen : -1; }
AtStackEntry *at_stack_top_entry(AtStack *s) { return s->n > 0 ? &s->e[s->n - 1] : NULL; }

void at_tween_start(AtTween *t, double now, double dur, int reduced)
{
    t->t0 = now;
    t->dur = reduced ? 0.0 : dur;
}

float at_tween_value(const AtTween *t, double now)
{
    double p;
    if (t->dur <= 0.0) return 1.0f;
    p = (now - t->t0) / t->dur;
    if (p <= 0.0) return 0.0f;
    if (p >= 1.0) return 1.0f;
    return (float) (1.0 - (1.0 - p) * (1.0 - p) * (1.0 - p));
}
