/* gw_script_meter.inc - the HUD meter of a Geno define (geno.json fighters[].meter, geno 11; docs/geno.md section 25.7).
 * Included by gw_script.c beside the draw list (gw_Script_DrawCount / gw_Script_DrawAt, queried by the overlay every rendered frame,
 * frozen logic included). For every fighter slot whose define declares a meter, four draw entries in the 640x480 virtual screen: a dark
 * back bar, the filled part (the combo ki value over the meter's max), a frame and the name. It only READS the combo block
 * (gw_GenoGame_ComboInfo: the same value the digest folds), writes nothing into the game, and exists only while a match is running, so
 * it cannot change the simulation, a savestate or a rollback. */
extern int gw_GenoGame_ComboInfo(int port, int what);
extern int gw_Geno_MeterInfo(int p, char *name, int cap, unsigned *rgba, float *x, float *y, float *w, float *h, int *max);

enum { GS_METER_MAX = 6 * 4 };
static GwScriptDraw gs_meter_draw[GS_METER_MAX];
static int gs_meter_n;

/* Fills gs_meter_draw and returns the entry count. */
static int gs_meter_build(void)
{
    int slot;
    gs_meter_n = 0;
    if (!gs.match_active) {
        return 0;
    }
    for (slot = 0; slot < 6 && gs_meter_n + 4 <= GS_METER_MAX; ++slot) {
        char name[16];
        unsigned rgba;
        float x, y, w, h, frac;
        int prof, ki, max;
        GwScriptDraw *d;
        if (gw_GenoGame_ComboInfo(slot, 3) < 0) {
            continue; /* no combo graph on this slot's main fighter */
        }
        prof = gw_GenoGame_ComboInfo(slot, 11);
        if (!gw_Geno_MeterInfo(prof, name, (int) sizeof name, &rgba, &x, &y, &w, &h, &max) || max <= 0) {
            continue;
        }
        ki = gw_GenoGame_ComboInfo(slot, 1);
        frac = (float) ki / (float) max;
        if (frac < 0.0f) frac = 0.0f;
        if (frac > 1.0f) frac = 1.0f;
        if (x < 0.0f) {
            x = 60.0f + (float) slot * 150.0f; /* default: under the port's damage plate */
        }
        d = &gs_meter_draw[gs_meter_n];
        memset(d, 0, 4 * sizeof *d);
        d[0].kind = GW_SDRAW_FILL; d[0].x = x; d[0].y = y; d[0].w = w; d[0].h = h; d[0].rgba = 0x000000B4u;
        d[1].kind = GW_SDRAW_FILL; d[1].x = x; d[1].y = y; d[1].w = w * frac; d[1].h = h; d[1].rgba = rgba;
        d[2].kind = GW_SDRAW_BOX;  d[2].x = x - 1.0f; d[2].y = y - 1.0f; d[2].w = w + 2.0f; d[2].h = h + 2.0f; d[2].rgba = frac >= 1.0f ? 0xFFFFFFFFu : 0xC8C8C8D2u;
        d[3].kind = GW_SDRAW_TEXT; d[3].x = x; d[3].y = y - 13.0f; d[3].size = 1.0f; d[3].rgba = 0xFFFFFFFFu;
        snprintf(d[3].text, sizeof d[3].text, "%s %d", name, (ki + 50) / 100);
        gs_meter_n += 4;
    }
    return gs_meter_n;
}
