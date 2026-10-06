#include "atlas_check.h"
#include "../platform/gw_ui_input.h"

static AtEvent ev[8];

static void pad(void)
{
    AtPad p; int n;
    memset(&p, 0, sizeof p);
    n = at_pad_events(&p, AT_PAD_A, 0, 0, 0.0, ev, 8);
    CHECK(n == 1 && ev[0].type == AT_EV_ACCEPT);
    n = at_pad_events(&p, AT_PAD_A, 0, 0, 16.0, ev, 8); CHECK(n == 0);              /* held: no repeat for buttons */
    n = at_pad_events(&p, 0, 0, 0, 32.0, ev, 8); CHECK(n == 0);
    n = at_pad_events(&p, AT_PAD_B | AT_PAD_X, 0, 0, 48.0, ev, 8);
    CHECK(n == 2 && ev[0].type == AT_EV_BACK && ev[1].type == AT_EV_ALT && ev[1].a == 'X');
    n = at_pad_events(&p, 0, 0, 0, 64.0, ev, 8);
    n = at_pad_events(&p, AT_PAD_L, 0, 0, 80.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_PAGE && ev[0].a == -1);
    n = at_pad_events(&p, AT_PAD_R | AT_PAD_L, 0, 0, 96.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_PAGE && ev[0].a == 1);
    n = at_pad_events(&p, 0, 0, 0, 112.0, ev, 8);
    n = at_pad_events(&p, AT_PAD_START, 0, 0, 128.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_START);
    n = at_pad_events(&p, 0, 0, 0, 144.0, ev, 8);
    n = at_pad_events(&p, 0, 0, 100, 160.0, ev, 8);                                /* stick up past 60 */
    CHECK(n == 1 && ev[0].type == AT_EV_MOVE && ev[0].a == AT_DIR_UP);
    n = at_pad_events(&p, 0, 0, 100, 200.0, ev, 8); CHECK(n == 0);
    n = at_pad_events(&p, 0, 0, 100, 460.0, ev, 8); CHECK(n == 1 && ev[0].a == AT_DIR_UP);   /* repeat after 300 ms */
    n = at_pad_events(&p, 0, 59, 0, 480.0, ev, 8); CHECK(n == 0);                  /* under the threshold: nothing */
    n = at_pad_events(&p, AT_PAD_LEFT | AT_PAD_UP, 0, 0, 500.0, ev, 8);
    CHECK(n == 1 && ev[0].a == AT_DIR_UP);                                         /* up beats left */
    n = at_pad_events(&p, AT_PAD_RIGHT, 0, 0, 520.0, ev, 8); CHECK(n == 1 && ev[0].a == AT_DIR_RIGHT);
    n = at_pad_events(&p, 0, -80, 0, 540.0, ev, 8); CHECK(n == 1 && ev[0].a == AT_DIR_LEFT);
}

static void keys(void)
{
    AtKeys k; int n;
    memset(&k, 0, sizeof k);
    n = at_key_events(&k, AT_KEY_ENTER, 0.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_ACCEPT);
    n = at_key_events(&k, AT_KEY_ENTER, 10.0, ev, 8); CHECK(n == 0);
    n = at_key_events(&k, AT_KEY_ESC, 20.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_BACK);
    n = at_key_events(&k, 0, 30.0, ev, 8);
    n = at_key_events(&k, AT_KEY_TAB, 40.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_PAGE && ev[0].a == 1);
    n = at_key_events(&k, 0, 50.0, ev, 8);
    n = at_key_events(&k, AT_KEY_TAB | AT_KEY_SHIFT, 60.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_PAGE && ev[0].a == -1);
    n = at_key_events(&k, 0, 70.0, ev, 8);
    n = at_key_events(&k, AT_KEY_DOWN, 80.0, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_MOVE && ev[0].a == AT_DIR_DOWN);
    n = at_key_events(&k, AT_KEY_DOWN, 400.0, ev, 8); CHECK(n == 1 && ev[0].a == AT_DIR_DOWN);   /* repeats like the pad */
}

static AtHits hits_fixture(void)
{
    AtHits h; memset(&h, 0, sizeof h);
    h.h[0].r = (AtRect){ 100, 100, 50, 50 }; h.h[0].kind = AT_HIT_CELL; h.h[0].a = 0; h.h[0].b = 0;
    h.h[1].r = (AtRect){ 200, 100, 50, 50 }; h.h[1].kind = AT_HIT_CELL; h.h[1].a = 0; h.h[1].b = 1;
    h.h[2].r = (AtRect){ 40, 440, 90, 20 };  h.h[2].kind = AT_HIT_KEY;  h.h[2].a = 'X';
    h.h[3].r = (AtRect){ 700, 440, 90, 20 }; h.h[3].kind = AT_HIT_KEY;  h.h[3].a = 'B';   /* a wide-window position */
    h.n = 4;
    return h;
}

static void mouse(void)
{
    AtHits h = hits_fixture();
    AtMouse m; int n;
    memset(&m, 0, sizeof m);
    n = at_mouse_events(&m, 120, 120, 0, 0, &h, ev, 8); CHECK(n == 0);              /* the first sample never focuses */
    n = at_mouse_events(&m, 120, 120, 0, 0, &h, ev, 8); CHECK(n == 0);              /* a pointer resting still does nothing */
    n = at_mouse_events(&m, 220, 120, 0, 0, &h, ev, 8);
    CHECK(n == 1 && ev[0].type == AT_EV_FOCUS && ev[0].a == 0 && ev[0].b == 1);     /* moving onto a cell focuses it */
    n = at_mouse_events(&m, 221, 120, 0, 0, &h, ev, 8); CHECK(n == 1 && ev[0].b == 1);
    n = at_mouse_events(&m, 221, 120, 1, 0, &h, ev, 8);
    CHECK(n == 2 && ev[0].type == AT_EV_FOCUS && ev[1].type == AT_EV_ACCEPT);       /* left click: focus then accept */
    n = at_mouse_events(&m, 221, 120, 1, 0, &h, ev, 8); CHECK(n == 0);              /* held: once */
    n = at_mouse_events(&m, 221, 120, 3, 0, &h, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_BACK);   /* right button down: back */
    n = at_mouse_events(&m, 221, 300, 0, 0, &h, ev, 8); CHECK(n == 0);              /* moving over nothing: no event */
    n = at_mouse_events(&m, 60, 450, 1, 0, &h, ev, 8);
    CHECK(n == 1 && ev[0].type == AT_EV_ALT && ev[0].a == 'X');                     /* a key hint is a button */
    n = at_mouse_events(&m, 60, 450, 0, 0, &h, ev, 8);
    n = at_mouse_events(&m, 740, 450, 1, 0, &h, ev, 8);
    CHECK(n == 1 && ev[0].type == AT_EV_BACK);                                      /* hit rectangles work beyond x = 640 */
    n = at_mouse_events(&m, 740, 450, 0, 0, &h, ev, 8);
    n = at_mouse_events(&m, 740, 450, 0, 3, &h, ev, 8); CHECK(n == 1 && ev[0].type == AT_EV_SCROLL && ev[0].a == -1);
    n = at_mouse_events(&m, 740, 450, 0, -2, &h, ev, 8); CHECK(n == 1 && ev[0].a == 1);
    n = at_mouse_events(&m, -1000, -1000, 1, 0, &h, ev, 8); CHECK(n == 0);         /* off the picture: nothing, ever */
    n = at_mouse_events(&m, -1000, -1000, 0, 5, &h, ev, 8); CHECK(n == 0);
    n = at_mouse_events(&m, 120, 120, 1, 0, &h, ev, 8);                              /* re-entering with the button held is not a click */
    CHECK(n == 0);
    CHECK(at_hit_test(&h, 120, 120) == 0 && at_hit_test(&h, 5, 5) == -1);
}

int main(void)
{
    pad(); keys(); mouse();
    ATLAS_DONE("atlas input");
}
