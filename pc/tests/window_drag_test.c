/* Pure nonmodal title-drag state tests; the live HWND path is checked in-game. */
#include "../platform/gw_window_drag.h"
#include <assert.h>
#include <limits.h>
#include <stdio.h>

int main(void) {
    GwWindowDrag drag = {0};
    int x = 0, y = 0;
    gw_window_drag_begin(&drag, -100, 80, -1200, -200);
    assert(gw_window_drag_step(&drag, -98, 82, 4, 4, &x, &y) == 0);
    assert(gw_window_drag_step(&drag, -90, 100, 4, 4, &x, &y) == 1);
    assert(x == -1190 && y == -180);
    assert(gw_window_drag_step(&drag, -85, 95, 4, 4, &x, &y) == 1);
    assert(x == -1185 && y == -185);
    gw_window_drag_cancel(&drag);
    assert(gw_window_drag_step(&drag, -80, 90, 4, 4, &x, &y) == 0);
    gw_window_drag_begin(&drag, INT_MAX - 2, INT_MIN + 2, INT_MAX - 1, INT_MIN + 1);
    assert(gw_window_drag_step(&drag, INT_MAX, INT_MIN, 1, 1, &x, &y) == 1);
    assert(x == INT_MAX && y == INT_MIN);
    puts("window drag coordinates passed");
    return 0;
}
