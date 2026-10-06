#include "atlas_check.h"
#include "../platform/gw_ui_stack.h"

int main(void)
{
    AtStack s; AtTween t; int i;
    memset(&s, 0, sizeof s);
    CHECK(at_stack_top(&s) == -1 && at_stack_pop(&s) == -1 && at_stack_top_entry(&s) == NULL);
    CHECK(at_stack_push(&s, 3) == 1 && at_stack_top(&s) == 3);
    CHECK(at_stack_push(&s, 3) == 0);                                   /* pushing the screen already on top does nothing */
    at_stack_top_entry(&s)->focus.block = 1; at_stack_top_entry(&s)->focus.index = 2; at_stack_top_entry(&s)->scroll = 4;
    CHECK(at_stack_push(&s, 5) == 1 && at_stack_top(&s) == 5);
    CHECK(at_stack_pop(&s) == 5 && at_stack_top(&s) == 3);
    CHECK(at_stack_top_entry(&s)->focus.block == 1 && at_stack_top_entry(&s)->focus.index == 2 && at_stack_top_entry(&s)->scroll == 4);   /* focus is remembered */
    at_stack_push(&s, 6); at_stack_push(&s, 7);
    CHECK(at_stack_remove(&s, 6) == 1 && s.n == 2 && at_stack_top(&s) == 7);   /* removal from the middle keeps the order */
    CHECK(at_stack_remove(&s, 99) == 0);
    for (i = 10; i < 20; i++) at_stack_push(&s, i);
    CHECK(s.n == AT_STACK_MAX);                                         /* full: a push is refused, nothing is lost */
    CHECK(at_stack_push(&s, 50) == 0 && at_stack_top(&s) == 15);   /* 3 and 7, then 10 to 15, fill the eight places */

    at_tween_start(&t, 1000.0, 100.0, 0);
    CHECK_NEAR(at_tween_value(&t, 1000.0), 0.0);
    CHECK_NEAR(at_tween_value(&t, 1050.0), 0.875);                      /* ease-out cubic: 1 - (1 - 0.5)^3 */
    CHECK_NEAR(at_tween_value(&t, 1100.0), 1.0);
    CHECK_NEAR(at_tween_value(&t, 5000.0), 1.0);
    CHECK_NEAR(at_tween_value(&t, 900.0), 0.0);                         /* before the start: not started */
    at_tween_start(&t, 1000.0, 100.0, 1);                               /* Reduced motion: an immediate cut */
    CHECK_NEAR(at_tween_value(&t, 1000.0), 1.0);
    at_tween_start(&t, 1000.0, 0.0, 0);
    CHECK_NEAR(at_tween_value(&t, 1000.0), 1.0);
    ATLAS_DONE("atlas stack");
}
