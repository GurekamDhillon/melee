/* gw_nucleus_image.h - turning the pictures SSBM Nucleus serves (PNG portraits, stock icons and screenshots) into the small textures the browser draws.
 * Pure C over stb_image (PNG only: the API's WebP thumbnails are never used); no game types, no threads, no network. The engine's picture threads call ni_make off the game thread and
 * keep the result as a .gxtex blob; the game thread uploads it (gw_Kit_TexAddGxtex) a couple per frame. Header-only (static), like gw_nucleus_core.h.
 *
 * Every picture is fitted INSIDE a fixed canvas (aspect kept, centred, transparent around), so a texture is always the same small size whatever the
 * source: NI_THUMB 96x72 (a list row's picture box is 4:3), NI_SHOT 256x144 (a screenshot, 16:9), NI_ICON 32x32 (a stock icon). 96x72 RGB5A3 is 14 KB. */
#ifndef GW_NUCLEUS_IMAGE_H
#define GW_NUCLEUS_IMAGE_H

#include "gw_nucleus_install.h"

enum { NI_THUMB, NI_SHOT, NI_ICON, NI_KINDS };
static const int ni_canvas_w[NI_KINDS] = { 96, 256, 32 };
static const int ni_canvas_h[NI_KINDS] = { 72, 144, 32 };
#define NI_MAX_BYTES (6u * 1024u * 1024u)          /* a source picture larger than this is refused (the download limit too) */
#define NI_MAX_DIM 2048

/* the picture's bytes -> RGBA8 (malloc'd; free with ni_free_rgba). A PNG only. 0 ok, -1 with err. */
static int ni_decode(const uint8_t *d, size_t n, uint8_t **rgba, int *w, int *h, char *err, size_t ecap) {
    *rgba = NULL; *w = *h = 0;
    if (n < 16 || n > NI_MAX_BYTES) { snprintf(err, ecap, "not a picture"); return -1; }
    if (!memcmp(d, "\x89PNG", 4)) {
        int comp = 0;
        unsigned char *px = stbi_load_from_memory(d, (int) n, w, h, &comp, 4);
        if (!px) { snprintf(err, ecap, "PNG unreadable (%s)", stbi_failure_reason() ? stbi_failure_reason() : "?"); return -1; }
        *rgba = px;
        return 0;
    }
    snprintf(err, ecap, "not a PNG");
    return -1;
}
static void ni_free_rgba(uint8_t *px, const uint8_t *src) { (void) src; if (px) stbi_image_free(px); }

/* Fit sw x sh RGBA8 inside cw x ch: scaled by the largest factor that fits (a box average when shrinking, the nearest pixel when growing), centred; dst (cw*ch*4)
 * is cleared first, so the margin is transparent. Colour is averaged weighted by alpha, so a soft edge does not darken. */
static void ni_fit(const uint8_t *src, int sw, int sh, int cw, int ch, uint8_t *dst) {
    double sx = (double) cw / (double) sw, sy = (double) ch / (double) sh, k = sx < sy ? sx : sy;
    int dw = (int) (sw * k + 0.5), dh = (int) (sh * k + 0.5), ox, oy, x, y;
    memset(dst, 0, (size_t) cw * (size_t) ch * 4);
    if (dw < 1) dw = 1;
    if (dh < 1) dh = 1;
    if (dw > cw) dw = cw;
    if (dh > ch) dh = ch;
    ox = (cw - dw) / 2; oy = (ch - dh) / 2;
    for (y = 0; y < dh; ++y) {
        int y0 = (int) ((double) y * sh / dh), y1 = (int) (((double) (y + 1) * sh) / dh + 0.999999);
        if (y1 <= y0) y1 = y0 + 1;
        if (y1 > sh) y1 = sh;
        for (x = 0; x < dw; ++x) {
            int x0 = (int) ((double) x * sw / dw), x1 = (int) (((double) (x + 1) * sw) / dw + 0.999999), xx, yy;
            unsigned long long a = 0, r = 0, g = 0, b = 0, cnt = 0;
            uint8_t *o = dst + ((size_t) (oy + y) * (size_t) cw + (size_t) (ox + x)) * 4;
            if (x1 <= x0) x1 = x0 + 1;
            if (x1 > sw) x1 = sw;
            for (yy = y0; yy < y1; ++yy) for (xx = x0; xx < x1; ++xx) {
                const uint8_t *p = src + ((size_t) yy * (size_t) sw + (size_t) xx) * 4;
                a += p[3]; r += (unsigned long long) p[0] * p[3]; g += (unsigned long long) p[1] * p[3]; b += (unsigned long long) p[2] * p[3];
                ++cnt;
            }
            if (cnt && a) {
                o[0] = (uint8_t) (r / a); o[1] = (uint8_t) (g / a); o[2] = (uint8_t) (b / a);
                o[3] = (uint8_t) (a / cnt);
            }
        }
    }
}

/* picture bytes -> a .gxtex blob of the kind's canvas (*gx malloc'd). 0 ok, -1 with err. */
static int ni_make(const uint8_t *data, size_t n, int kind, uint8_t **gx, size_t *gl, char *err, size_t ecap) {
    uint8_t *px = NULL, *canvas;
    int w = 0, h = 0, rc;
    if (kind < 0 || kind >= NI_KINDS) { snprintf(err, ecap, "bad kind"); return -1; }
    if (ni_decode(data, n, &px, &w, &h, err, ecap) < 0) return -1;
    canvas = (uint8_t *) malloc((size_t) ni_canvas_w[kind] * (size_t) ni_canvas_h[kind] * 4);
    if (!canvas) { ni_free_rgba(px, data); snprintf(err, ecap, "out of memory"); return -1; }
    ni_fit(px, w, h, ni_canvas_w[kind], ni_canvas_h[kind], canvas);
    ni_free_rgba(px, data);
    rc = nc_rgba_to_gxtex(canvas, ni_canvas_w[kind], ni_canvas_h[kind], gx, gl);
    free(canvas);
    if (rc < 0) snprintf(err, ecap, "texture conversion failed");
    return rc;
}

#endif
