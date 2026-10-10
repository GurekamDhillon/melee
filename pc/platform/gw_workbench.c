/* gw_workbench.c - opt-in, offline workbench diagnostics.
 * Pure frame serialization and file publication; no game pointers or simulation state.
 * gw_script.c supplies the session predicate (native tests supply its seam).
 * frame.json is replaced only after the complete temporary file is closed. */
#include "gw_workbench.h"
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#endif

extern int gw_Wb_NetplayActive(void);
static int g_enabled;

int gw_Wb_Enabled(void) { return g_enabled && !gw_Wb_NetplayActive(); }
void gw_Wb_SetEnabled(int on) { g_enabled = on != 0; }
const char *gw_Wb_Dir(void)
{
    const char *dir = getenv("MELEE_WB_DIR");
    return dir != NULL ? dir : "";
}

int gw_Wb_WritePayload(const char *name, const char *data, size_t n,
                      char *path_out, size_t cap)
{
    const char *dir = gw_Wb_Dir();
    char *temp;
    FILE *file;
    int len, failed;
    if (path_out == NULL || cap == 0) return -1;
    path_out[0] = '\0';
    if (!dir[0] || name == NULL || !name[0] || data == NULL ||
        strchr(name, '/') || strchr(name, '\\') || strchr(name, ':') ||
        strcmp(name, ".") == 0 || strcmp(name, "..") == 0) return -1;
    len = snprintf(path_out, cap, "%s/%s", dir, name);
    if (len < 0 || (size_t)len >= cap) { path_out[0] = '\0'; return -1; }
    temp = (char *)malloc((size_t)len + 5);
    if (temp == NULL) { path_out[0] = '\0'; return -1; }
    memcpy(temp, path_out, (size_t)len);
    memcpy(temp + len, ".tmp", 5);
    file = fopen(temp, "wb");
    if (file == NULL) { free(temp); path_out[0] = '\0'; return -1; }
    failed = fwrite(data, 1, n, file) != n;
    if (fclose(file) != 0) failed = 1;
    if (!failed) {
#ifdef _WIN32
        /* The Windows CRT's rename refuses an existing destination. This is its
         * replacing equivalent, so repeated frame_info calls never expose half a file. */
        failed = !MoveFileExA(temp, path_out, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH);
#else
        failed = rename(temp, path_out) != 0;
#endif
    }
    if (failed) { remove(temp); path_out[0] = '\0'; }
    free(temp);
    return failed ? -1 : 0;
}

typedef struct {
    char *out;
    size_t cap, used;
    int failed;
} WbJson;

static void wb_append(WbJson *j, const char *fmt, ...)
{
    va_list ap;
    int n;
    if (j->failed) return;
    va_start(ap, fmt);
    n = vsnprintf(j->out + j->used, j->cap - j->used, fmt, ap);
    va_end(ap);
    if (n < 0 || (size_t)n >= j->cap - j->used) { j->failed = 1; return; }
    j->used += (size_t)n;
}

static void wb_string(WbJson *j, const char *s)
{
    const unsigned char *p = (const unsigned char *)(s != NULL ? s : "");
    wb_append(j, "\"");
    for (; *p && !j->failed; ++p) {
        switch (*p) {
        case '"': wb_append(j, "\\\""); break;
        case '\\': wb_append(j, "\\\\"); break;
        case '\n': wb_append(j, "\\n"); break;
        case '\r': wb_append(j, "\\r"); break;
        case '\t': wb_append(j, "\\t"); break;
        default:
            if (*p < 0x20) wb_append(j, "\\u%04x", (unsigned)*p);
            else wb_append(j, "%c", *p);
        }
    }
    wb_append(j, "\"");
}

static double wb_number(float f) { return isfinite(f) ? (double)f : 0.0; }
static void wb_vec(WbJson *j, float x, float y, float z)
{
    wb_append(j, "{\"x\":%g,\"y\":%g,\"z\":%g}", wb_number(x), wb_number(y), wb_number(z));
}

int gw_Wb_FrameJson(const GwWbFrame *f, const char *row, char *out, size_t cap)
{
    WbJson j;
    int i;
    if (out == NULL || cap == 0) return -1;
    out[0] = '\0';
    if (f == NULL || f->nbones < 0 || f->nbones > 160 ||
        f->nhits < 0 || f->nhits > 8 || f->nhurts < 0 || f->nhurts > 24) return -1;
    j.out = out; j.cap = cap; j.used = 0; j.failed = 0;
    wb_append(&j, "{\"row\":");
    wb_string(&j, row);
    wb_append(&j, ",\"motion\":%d,\"frame\":%d,\"bones\":[", f->motion, f->frame);
    for (i = 0; i < f->nbones; ++i) {
        const GwWbBone *b = &f->bones[i];
        wb_append(&j, "%s{\"joint\":%d,\"parent\":%d,\"pos\":", i ? "," : "", b->joint, b->parent);
        wb_vec(&j, b->x, b->y, b->z);
        wb_append(&j, "}");
    }
    wb_append(&j, "],\"hitboxes\":[");
    for (i = 0; i < f->nhits; ++i) {
        const GwWbHit *h = &f->hits[i];
        wb_append(&j, "%s{\"slot\":%d,\"joint\":%d,\"active\":%s,\"offset\":",
                  i ? "," : "", h->slot, h->joint, h->active ? "true" : "false");
        wb_vec(&j, h->ox, h->oy, h->oz);
        wb_append(&j, ",\"radius\":%g,\"world\":", wb_number(h->radius));
        wb_vec(&j, h->wx, h->wy, h->wz);
        wb_append(&j, "}");
    }
    wb_append(&j, "],\"hurtboxes\":[");
    for (i = 0; i < f->nhurts; ++i) {
        const GwWbHurt *h = &f->hurts[i];
        wb_append(&j, "%s{\"joint\":%d,\"a\":", i ? "," : "", h->joint);
        wb_vec(&j, h->ax, h->ay, h->az);
        wb_append(&j, ",\"b\":");
        wb_vec(&j, h->bx, h->by, h->bz);
        wb_append(&j, ",\"radius\":%g}", wb_number(h->radius));
    }
    wb_append(&j, "],\"ecb\":{\"bottom\":{\"x\":%g,\"y\":%g},\"top\":{\"x\":%g,\"y\":%g}},"
              "\"floorY\":%g,\"lowestY\":%g}",
              wb_number(f->ecb_bx), wb_number(f->ecb_by), wb_number(f->ecb_tx), wb_number(f->ecb_ty),
              wb_number(f->floor_y), wb_number(f->lowest_y));
    if (j.failed) { out[0] = '\0'; return -1; }
    return (int)j.used;
}
