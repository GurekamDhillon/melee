/* Restricted JSON schema for runtime model collision. No executable Lua parsing.
 * Kept independent of Windows/GX so exporters and headless validation can test it. */
#ifndef GW_MODEL_FORMAT_H
#define GW_MODEL_FORMAT_H
#include <math.h>
#include <stdlib.h>
#include <string.h>
#define GM_COLLISION_LINES 32
typedef struct { float x0, y0, x1, y1; int kind, flags; } GmLine;
typedef struct { char atlas[49]; int count; GmLine line[GM_COLLISION_LINES]; } GmCollision;

static int gm_model_path(const char *s)
{
    int n = 0, component = 0;
    for (; *s; ++s, ++n) {
        if (n >= 180) return 0;
        if (*s == '/') { if (!component) return 0; component = 0; }
        else if ((*s >= 'a' && *s <= 'z') || (*s >= 'A' && *s <= 'Z') ||
                 (*s >= '0' && *s <= '9') || *s == '_' || *s == '-') ++component;
        else return 0;
    }
    return component != 0;
}
static void gm_ws(const char **p) { while (**p == ' ' || **p == '\t' || **p == '\r' || **p == '\n') ++*p; }
static int gm_take(const char **p, char c)
{
    gm_ws(p); if (**p != c) return 0; ++*p; return 1;
}
static int gm_string(const char **p, char *out, int cap)
{
    int n = 0;
    if (!gm_take(p, '"')) return 0;
    while (**p && **p != '"') {
        unsigned char c = (unsigned char)*(*p)++;
        if (c < 32 || c == '\\' || n == cap - 1) return 0;
        out[n++] = (char)c;
    }
    out[n] = 0;
    return gm_take(p, '"');
}
static int gm_number(const char **p, float *out)
{
    const char *start; char *end; double d;
    gm_ws(p); start = *p;
    if (**p == '-') ++*p;
    if (**p == '0') ++*p;
    else { if (**p < '1' || **p > '9') return 0; while (**p >= '0' && **p <= '9') ++*p; }
    if (**p == '.') { ++*p; if (**p < '0' || **p > '9') return 0; while (**p >= '0' && **p <= '9') ++*p; }
    if (**p == 'e' || **p == 'E') {
        ++*p; if (**p == '+' || **p == '-') ++*p;
        if (**p < '0' || **p > '9') return 0;
        while (**p >= '0' && **p <= '9') ++*p;
    }
    d = strtod(start, &end);
    if (end != *p || !isfinite(d) || d < -100000 || d > 100000) return 0;
    *out = (float)d; return 1;
}
static int gm_line_valid(const GmLine *l)
{
    if (l->kind != 1 && l->flags) return 0;
    return (l->kind == 1 && l->x0 < l->x1) ||
           (l->kind == 2 && l->x0 > l->x1) ||
           (l->kind == 3 && l->y0 > l->y1) ||
           (l->kind == 4 && l->y0 < l->y1);
}
static int gm_small_integer(const char **p, int max, int *out)
{
    gm_ws(p);
    if (**p < '0' || **p > '0' + max) return 0;
    *out = *(*p)++ - '0';
    return 1; /* the next punctuation check rejects decimals and extra digits */
}
static int gm_collision_parse(const char *p, GmCollision *out)
{
    int seen = 0;
    memset(out, 0, sizeof *out);
    if (!gm_take(&p, '{')) return 0;
    do {
        char key[16]; int v;
        if (!gm_string(&p, key, sizeof key) || !gm_take(&p, ':')) return 0;
        if (!strcmp(key, "version")) {
            if ((seen & 1) || !gm_small_integer(&p, 1, &v) || v != 1) return 0;
            seen |= 1;
        } else if (!strcmp(key, "atlas")) {
            if ((seen & 2) || !gm_string(&p, out->atlas, sizeof out->atlas) ||
                !gm_model_path(out->atlas) || strchr(out->atlas, '/')) return 0;
            seen |= 2;
        } else if (!strcmp(key, "lines")) {
            if ((seen & 4) || !gm_take(&p, '[')) return 0;
            seen |= 4; gm_ws(&p);
            if (*p != ']') do {
                GmLine *l; char kind[16];
                if (out->count == GM_COLLISION_LINES || !gm_take(&p, '[')) return 0;
                l = &out->line[out->count++];
                if (!gm_string(&p, kind, sizeof kind)) return 0;
                l->kind = !strcmp(kind, "floor") ? 1 : !strcmp(kind, "ceiling") ? 2 :
                          !strcmp(kind, "right_wall") ? 3 : !strcmp(kind, "left_wall") ? 4 : 0;
                if (!gm_take(&p, ',') || !gm_number(&p, &l->x0) ||
                    !gm_take(&p, ',') || !gm_number(&p, &l->y0) ||
                    !gm_take(&p, ',') || !gm_number(&p, &l->x1) ||
                    !gm_take(&p, ',') || !gm_number(&p, &l->y1) ||
                    !gm_take(&p, ',') || !gm_small_integer(&p, 3, &v) || !gm_take(&p, ']')) return 0;
                l->flags = v;
                if (!gm_line_valid(l)) return 0;
            } while (gm_take(&p, ','));
            if (!gm_take(&p, ']')) return 0;
        } else return 0;
    } while (gm_take(&p, ','));
    if (!gm_take(&p, '}')) return 0;
    gm_ws(&p);
    return *p == 0 && (seen & 5) == 5;
}
#endif
