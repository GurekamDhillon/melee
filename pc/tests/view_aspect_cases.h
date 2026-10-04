/* Shared by the standalone canvas check and registered native test suite. */
#ifndef GW_VIEW_ASPECT_CASES_H
#define GW_VIEW_ASPECT_CASES_H
#include "../platform/gw_view_math.h"
#include "../../src/melee/if/ifhud_math.h"
static inline int gw_view_aspect_cases(void)
{
    const int cases[][8] = {
        /* window W,H, wide, rect X,Y,W,H, aspect numerator (denominator below) */
        {800,600,1,0,0,800,600,4},
        {1280,800,1,0,0,1280,800,16},
        {1280,720,1,0,0,1280,720,16},
        {2520,1080,1,0,0,2520,1080,21},
        {3840,1080,1,0,0,3840,1080,32},
        {5000,1080,1,580,0,3840,1080,32},
        {600,800,1,0,175,600,450,4},
        {1379,767,1,0,0,1379,767,1379},
        {1280,720,0,160,0,960,720,4},
        {801,900,1,0,149,801,601,4},
    };
    const int den[] = {3,10,9,9,9,9,3,767,3,3};
    int i;
    for (i = 0; i < 10; ++i) {
        int x, y, w, h;
        float aspect = gw_view_aspect((float)cases[i][0], (float)cases[i][1], cases[i][2]);
        float error = aspect - (float)cases[i][7] / den[i];
        float scale = gw_view_projection_scale(aspect, cases[i][2]);
        float projection = (73.0f / 60.0f) * scale;
        float expected = cases[i][2] ? aspect : 73.0f / 60.0f;
        if (error < -0.000002f || error > 0.000002f) return i + 1;
        error = projection - expected;
        if (error < -0.000002f || error > 0.000002f) return i + 11;
        gw_view_rectangle(cases[i][0], cases[i][1], aspect, &x, &y, &w, &h);
        if (x != cases[i][3] || y != cases[i][4] || w != cases[i][5] || h != cases[i][6]) return i + 21;
        /* Rounding can leave one extra pixel on the right/bottom, never stretch. */
        if (cases[i][0]-w-x < x || cases[i][0]-w-x > x+1 ||
            cases[i][1]-h-y < y || cases[i][1]-h-y > y+1) return i + 31;
    }
    {
        float delta = gw_view_projection_scale(16.0f/9.0f, 1) - 320.0f/219.0f;
        if (delta < -0.0000002f || delta > 0.0000002f) return 41;
    }
    for (i = -1; i <= 47; ++i) {
        int expected = i == 2 || i == 3 || i == 4 || i == 44 || i == 46;
        if (gw_view_scene_wide(i) != expected) return 42;
    }
    /* Six-player row stays inside, centred on and independent of the view
     * aspect. Cover symmetric, translated and degenerate authored bounds. */
    for (i = 0; i < 6; ++i) {
        float x = ifHUD_FitCentre(-162.5f + 65.0f*i, -162.5f,162.5f,-150,150);
        float error = x - (-150.0f + 60.0f*i);
        if (error < -0.00002f || error > 0.00002f) return 46;
    }
    if (ifHUD_FitCentre(7,5,5,-150,150) != 7 ||
        ifHUD_FitCentre(-162.5f,-162.5f,162.5f,-70,230) != -70 ||
        ifHUD_FitCentre(162.5f,-162.5f,162.5f,-70,230) != 230) return 47;
    /* Dense 0.001-aspect sweep of the shared projection scale and fitted
     * rectangle. This checks CPU maths, not GPU depth/interpolation. */
    for (i = 0; i <= 2223; ++i) {
        float requested = GW_VIEW_MIN_ASPECT + (float)i * 0.001f;
        float aspect = gw_view_aspect(requested * 640.0f, 640.0f, 1);
        float scale = gw_view_projection_scale(aspect, 1);
        float projection = (73.0f / 60.0f) * scale;
        float error = projection - aspect;
        int x, y, w, h;
        if (error < -0.000002f || error > 0.000002f) return 43;
        if (!(projection >= GW_VIEW_MIN_ASPECT - 0.000002f &&
              projection <= GW_VIEW_MAX_ASPECT + 0.000002f)) return 44;
        gw_view_rectangle((int)(requested * 640.0f + 0.5f), 640, aspect, &x, &y, &w, &h);
        if (w <= 0 || h <= 0 || x < 0 || y < 0) return 45;
    }
    return 0;
}
#endif
