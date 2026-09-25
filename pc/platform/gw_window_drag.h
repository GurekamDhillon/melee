/* Pure screen-coordinate math for a nonmodal Windows caption drag. */
#ifndef GW_WINDOW_DRAG_H
#define GW_WINDOW_DRAG_H
#include <limits.h>
#include <stdint.h>

typedef struct GwWindowDrag {
    int active, moved;
    int cursor_x, cursor_y, window_x, window_y;
} GwWindowDrag;

static inline void gw_window_drag_begin(GwWindowDrag *drag, int cursor_x, int cursor_y,
                                        int window_x, int window_y) {
    drag->active = 1; drag->moved = 0;
    drag->cursor_x = cursor_x; drag->cursor_y = cursor_y;
    drag->window_x = window_x; drag->window_y = window_y;
}

static inline void gw_window_drag_cancel(GwWindowDrag *drag) { drag->active = 0; }

static inline int gw_window_drag_step(GwWindowDrag *drag, int cursor_x, int cursor_y,
                                      int threshold_x, int threshold_y, int *x, int *y) {
    int64_t dx, dy, left, top;
    if (!drag->active || !x || !y) return 0;
    dx = (int64_t)cursor_x - drag->cursor_x;
    dy = (int64_t)cursor_y - drag->cursor_y;
    if (!drag->moved && dx < threshold_x && dx > -threshold_x &&
        dy < threshold_y && dy > -threshold_y) return 0;
    drag->moved = 1;
    left = (int64_t)drag->window_x + dx;
    top = (int64_t)drag->window_y + dy;
    *x = left < INT_MIN ? INT_MIN : left > INT_MAX ? INT_MAX : (int)left;
    *y = top < INT_MIN ? INT_MIN : top > INT_MAX ? INT_MAX : (int)top;
    return 1;
}
#endif
