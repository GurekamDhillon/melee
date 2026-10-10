/* gw_workbench.c - opt-in, offline workbench diagnostics.
 * Pure frame serialization, camera orbit math and file publication; no game pointers.
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

/* Headless workbench (MELEE_WB_HEADLESS=1, honoured only for the workbench scene).
 * DEFINITION: headless = Dawn null backend, window placed offscreen, no GPU device. A no-window
 * path is out of scope (Dawn still wants an HWND surface).
 * Design (2026-10-10): the game boots as normal but Aurora gets its Dawn NULL backend
 * (no GPU device; the SDL window is created OFFSCREEN (MELEE_WINDOW_X/Y=30000) because Dawn still wants an HWND surface; SDL_HideWindow made the null-backend present loop spin in win32u, found 2026-10-10). Every GX call still
 * lands in a real, device-less Aurora, so no texture/model/display-list shim needs a gate and
 * windowed behaviour is untouched. The scene gate is published long after Aurora starts, so
 * main() calls gw_Wb_PreScan(argc, argv) before aurora_initialize.
 * GPU assumptions found in main.c: aurora_initialize (backend, window), gw_window_drag_install,
 * gw_pcache_guard_run (cache file only), AuroraSetViewportPolicy; all are harmless on the Null
 * backend. wb.render and gw_Wb_CaptureScreenshot need a real frame and are refused. */
static int g_headless_requested;
void gw_Wb_PreScan(int argc, char **argv)
{
    const char *env = getenv("MELEE_WB_HEADLESS");
    const char *scene = getenv("MELEE_SCENE");
    int i, workbench = scene != NULL && strstr(scene, "mode=workbench") != NULL;
    for (i = 1; i < argc; ++i) if (argv != NULL && argv[i] != NULL && strcmp(argv[i], "--workbench") == 0) workbench = 1;
    g_headless_requested = env != NULL && strcmp(env, "1") == 0 && workbench;
    if (g_headless_requested) {
        /* No swapchain: every frame is an offscreen frame (turbo with presentation off). */
#ifdef _WIN32
        if (getenv("MELEE_TURBO") == NULL) _putenv("MELEE_TURBO=1");
        _putenv("MELEE_TURBO_RENDER=0");
        _putenv("MELEE_WINDOW_X=30000");
        _putenv("MELEE_WINDOW_Y=30000");
#else
        if (getenv("MELEE_TURBO") == NULL) setenv("MELEE_TURBO", "1", 1);
        setenv("MELEE_TURBO_RENDER", "0", 1);
        setenv("MELEE_WINDOW_X", "30000", 1);
        setenv("MELEE_WINDOW_Y", "30000", 1);
#endif
    }
}
int gw_Wb_HeadlessRequested(void) { return g_headless_requested; }
int gw_Wb_Headless(void)
{
    const char *env = getenv("MELEE_WB_HEADLESS");
    return gw_Wb_Enabled() && env != NULL && strcmp(env, "1") == 0 && g_headless_requested;
}
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

int gw_Wb_CameraOrbit(const GwWbFrame *f, int target_joint, float yaw, float pitch,
                     float distance, float eye[3], float interest[3])
{
    const float radians = 0.017453292519943295f;
    float horizontal;
    int i, target = -1;
    if (f == NULL || eye == NULL || interest == NULL || f->nbones < 1 || f->nbones > 160 ||
        target_joint < -1 || !isfinite(yaw) || !isfinite(pitch) || !isfinite(distance) || distance <= 0)
        return -1;
    if (target_joint == -1) {
        float lo[3] = {f->bones[0].x, f->bones[0].y, f->bones[0].z};
        float hi[3] = {lo[0], lo[1], lo[2]};
        for (i = 1; i < f->nbones; ++i) {
            const GwWbBone *b = &f->bones[i];
            if (b->x < lo[0]) lo[0] = b->x;
            if (b->y < lo[1]) lo[1] = b->y;
            if (b->z < lo[2]) lo[2] = b->z;
            if (b->x > hi[0]) hi[0] = b->x;
            if (b->y > hi[1]) hi[1] = b->y;
            if (b->z > hi[2]) hi[2] = b->z;
        }
        for (i = 0; i < 3; ++i) interest[i] = (lo[i] + hi[i]) * 0.5f;
    } else {
        for (i = 0; i < f->nbones; ++i) {
            if (f->bones[i].joint == target_joint) { target = i; break; }
        }
        if (target < 0) return -1;
        interest[0] = f->bones[target].x;
        interest[1] = f->bones[target].y;
        interest[2] = f->bones[target].z;
    }
    for (i = 0; i < 3; ++i) if (!isfinite(interest[i])) return -1;
    /* HSD's look-at uses Y-up: an exactly vertical eye makes its basis singular. */
    if (pitch > 89.9f) pitch = 89.9f;
    if (pitch < -89.9f) pitch = -89.9f;
    yaw *= radians;
    pitch *= radians;
    horizontal = distance * cosf(pitch);
    eye[0] = interest[0] + horizontal * sinf(yaw);
    eye[1] = interest[1] + distance * sinf(pitch);
    eye[2] = interest[2] + horizontal * cosf(yaw);
    return 0;
}

float gw_Wb_CameraFloorFov(float floor_y, const float eye[3],
                         const float interest[3], float fov)
{
    const float degrees = 57.29577951308232f;
    float x = eye[0] - interest[0], y = eye[1] - interest[1];
    float z = eye[2] - interest[2], height = interest[1] - floor_y;
    float horizontal = sqrtf(x * x + z * z);
    float distance = sqrtf(horizontal * horizontal + y * y);
    float depth, required;
    if (!isfinite(height) || fabsf(height) > 60.0f ||
        !isfinite(distance) || distance <= 0.0f) return fov;
    /* Project the point directly below the target into HSD's Y-up camera.
     * Pitch changes both its depth and vertical offset; using height alone
     * would needlessly widen top views. Keep 10% room beyond the floor. */
    depth = distance + height * y / distance;
    if (depth <= 0.0f) return fov; /* behind the eye: no perspective lens can fit it */
    required = 2.0f * degrees * atan2f(1.1f * fabsf(height) * horizontal / distance, depth);
    if (required > 178.0f) required = 178.0f;
    return required > fov ? required : fov;
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
static void wb_mtx(WbJson *j, const float m[12])
{
    int i;
    wb_append(j, ",\"mtx\":[");
    for (i = 0; i < 12; ++i) wb_append(j, "%s%.7g", i ? "," : "", wb_number(m[i]));
    wb_append(j, "]");
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
    wb_append(&j, "{\"schema\":2,\"row\":");
    wb_string(&j, row);
    wb_append(&j, ",\"motion\":%d,\"frame\":%d,\"bones\":[", f->motion, f->frame);
    for (i = 0; i < f->nbones; ++i) {
        const GwWbBone *b = &f->bones[i];
        wb_append(&j, "%s{\"joint\":%d,\"parent\":%d,\"pos\":", i ? "," : "", b->joint, b->parent);
        wb_vec(&j, b->x, b->y, b->z);
        wb_mtx(&j, b->m);
        wb_append(&j, "}");
    }
    wb_append(&j, "],\"root\":{\"joint\":%d,\"pos\":", f->root_joint);
    wb_vec(&j, f->root_m[3], f->root_m[7], f->root_m[11]);
    wb_append(&j, ",\"facing\":%d", f->facing < 0 ? -1 : 1);
    wb_mtx(&j, f->root_m);
    wb_append(&j, "},\"hitboxes\":[");
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
    wb_append(&j, "],\"ecb\":{\"bottom\":{\"x\":%g,\"y\":%g},\"top\":{\"x\":%g,\"y\":%g},"
              "\"left\":{\"x\":%g,\"y\":%g},\"right\":{\"x\":%g,\"y\":%g}},"
              "\"floorY\":%g,\"lowestY\":%g}",
              wb_number(f->ecb_bx), wb_number(f->ecb_by), wb_number(f->ecb_tx), wb_number(f->ecb_ty),
              wb_number(f->ecb_lx), wb_number(f->ecb_ly), wb_number(f->ecb_rx), wb_number(f->ecb_ry),
              wb_number(f->floor_y), wb_number(f->lowest_y));
    if (j.failed) { out[0] = '\0'; return -1; }
    return (int)j.used;
}
