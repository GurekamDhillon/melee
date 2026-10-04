/* Host-only test of the shared kit/overlay canvas, no game or renderer. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "../platform/gw_view_math.h"
#include "view_aspect_cases.h"

static void check(float ww, float wh, int wide, float expected)
{
    float aspect = gw_view_aspect(ww, wh, wide);
    float width = 480.0f * aspect, left = gw_view_left(width);
    float scale, ox, oy, x, y;
    assert(fabsf(width - expected) < 0.001f);
    assert(fabsf(left + width * 0.5f - 320.0f) < 0.001f);
    gw_view_map(ww, wh, width, &scale, &ox, &oy);
    x = ((320.0f - left) * scale + ox - ox) / scale + left;
    y = (240.0f * scale + oy - oy) / scale;
    assert(fabsf(x - 320.0f) < 0.001f && fabsf(y - 240.0f) < 0.001f);
    x = ((91.5f - left) * scale + ox - ox) / scale + left;
    assert(fabsf(x - 91.5f) < 0.001f);
    if (wide && ww/wh >= GW_VIEW_MIN_ASPECT && ww/wh <= GW_VIEW_MAX_ASPECT) assert(fabsf(ww / width - wh / 480.0f) < 0.001f);
}

int main(void)
{
    assert(gw_view_default_wide(800, 600) == 0);
    assert(gw_view_default_wide(1280, 720) == 1);
    assert(gw_view_default_wide(2520, 1080) == 1);
    assert(gw_view_default_wide(600, 800) == 0);
    assert(gw_view_default_wide(0, 0) == -1);
    assert(gw_view_aspect_cases() == 0);
    check(1280, 800, 1, 768);
    check(3840, 1080, 1, 1706.666667f);
    check(5000, 1080, 1, 1706.666667f);
    check(600, 800, 1, 640);
    check(800, 600, 1, 640);
    check(1280, 720, 1, 853.333333f);
    check(2520, 1080, 1, 1120);
    check(2520, 1080, 0, 640);
    check(0, 0, 1, 640);
    puts("view canvas aspect/centering/mouse round-trip passed");
    return 0;
}
