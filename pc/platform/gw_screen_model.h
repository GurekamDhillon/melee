/* gw_screen_model.h - the maths of gd.kit.model (a script model drawn at a rectangle of the 640x480
 * script canvas). Pure C, no GX, no Lua, no game memory: given a decoded mesh and a pose it returns
 * lit, screen-space triangles sorted far to near. Presentation only.
 *
 * The camera is fixed and orthographic: it looks down -Z with +Y up (the way the match camera does),
 * so the model's own axes are the screen's. The light is the world draw's key light (gs_model_draw in
 * gw_script.c): direction (0.25, 1.0, 0.55) normalised, colour (235,235,240), ambient (92,96,112), the
 * GX clamped-diffuse sum amb + col * max(0, N.L), per channel. It is fixed here too, so a model looks the
 * same under any match camera, stage lighting or fog.
 *
 * FIT. The model is centred on the middle of its bounding box and spun about Y (yaw), then tilted about
 * X (pitch). The horizontal half-extent is therefore rxz (the largest distance from the centre axis in
 * XZ) at every yaw, and the vertical half-extent is hy*|cos p| + rxz*|sin p| at every yaw: the scale is
 * chosen from those, once per asset and pitch, so a spinning model never changes size and never leaves
 * its rectangle.
 *
 * DEPTH. Triangles are painter-sorted by the centroid's depth. That is exact for a convex closed mesh and
 * for anything whose triangles do not interpenetrate; interpenetrating triangles can show a seam. */
#ifndef GW_SCREEN_MODEL_H
#define GW_SCREEN_MODEL_H
#include <math.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define GSM_MAX_TRIS 512      /* a screen model is for icons: a bigger mesh is refused, not thinned */
#define GSM_MAX_SIDE 4096.0f  /* script canvas units */

typedef struct {
    int ntri, nvert, has_normals;
    float (*pos)[3], (*nrm)[3], (*uv)[2];
    uint16_t *idx; /* 3 * ntri */
    float center[3];
    float rxz; /* largest distance from the centre axis in XZ */
    float hy;  /* half height */
    float radius;
} GsmMesh;

typedef struct {
    float x, y, w, h;    /* the rectangle, script canvas units */
    float yaw, pitch;    /* degrees */
    float spin;          /* degrees per second, at 60 clock units per second */
    float clock;         /* clock units (1/60 s); the caller's deterministic frame counter */
    float margin;        /* fraction of w and h left empty on each side, 0..0.45 */
    float dim;           /* 0..1 multiplies the lit colour (a locked or empty cell) */
    float alpha;         /* 0..1 multiplies the tint alpha */
    uint32_t tint;       /* 0xRRGGBBAA, multiplied in */
    int cull;            /* 1: drop back faces (exact for closed convex meshes) */
} GsmParams;

typedef struct {
    float x[3], y[3], u[3], v[3];
    uint32_t c[3]; /* 0xRRGGBBAA per corner */
    float z;       /* centroid depth, larger = nearer */
} GsmTri;

static void gsm_free(GsmMesh *m)
{
    free(m->pos); free(m->nrm); free(m->uv); free(m->idx);
    memset(m, 0, sizeof *m);
}
/* Bounds, once per mesh. Returns 0 for a mesh with nothing to draw. */
static int gsm_bounds(GsmMesh *m)
{
    float lo[3], hi[3], r2 = 0, rxz2 = 0;
    int i, k;
    if (m->nvert < 1 || m->ntri < 1) return 0;
    for (k = 0; k < 3; ++k) lo[k] = hi[k] = m->pos[0][k];
    for (i = 1; i < m->nvert; ++i)
        for (k = 0; k < 3; ++k) {
            if (m->pos[i][k] < lo[k]) lo[k] = m->pos[i][k];
            if (m->pos[i][k] > hi[k]) hi[k] = m->pos[i][k];
        }
    for (k = 0; k < 3; ++k) m->center[k] = lo[k] * 0.5f + hi[k] * 0.5f;
    m->hy = (hi[1] - lo[1]) * 0.5f;
    for (i = 0; i < m->nvert; ++i) {
        float dx = m->pos[i][0] - m->center[0], dy = m->pos[i][1] - m->center[1], dz = m->pos[i][2] - m->center[2];
        if (dx * dx + dz * dz > rxz2) rxz2 = dx * dx + dz * dz;
        if (dx * dx + dy * dy + dz * dz > r2) r2 = dx * dx + dy * dy + dz * dz;
    }
    m->rxz = sqrtf(rxz2); m->radius = sqrtf(r2);
    return m->radius > 0.0f || m->hy > 0.0f;
}
/* The uniform scale (canvas units per model unit) that fits the mesh in the rectangle. */
static float gsm_fit(const GsmMesh *m, float w, float h, float margin, float pitch_deg)
{
    const float rad = 3.14159265f / 180.0f;
    float p = pitch_deg * rad, hh = m->rxz, vh = m->hy * fabsf(cosf(p)) + m->rxz * fabsf(sinf(p));
    float sx, sy;
    if (margin < 0) margin = 0;
    if (margin > 0.45f) margin = 0.45f;
    if (hh < 1e-6f) hh = 1e-6f;
    if (vh < 1e-6f) vh = 1e-6f;
    sx = w * 0.5f * (1.0f - 2.0f * margin) / hh;
    sy = h * 0.5f * (1.0f - 2.0f * margin) / vh;
    return sx < sy ? sx : sy;
}
/* Rotation = Rx(pitch) * Ry(yaw), row-major. */
static void gsm_rotation(float yaw_deg, float pitch_deg, float r[3][3])
{
    const float rad = 3.14159265f / 180.0f;
    float cy = cosf(yaw_deg * rad), sy = sinf(yaw_deg * rad), cp = cosf(pitch_deg * rad), sp = sinf(pitch_deg * rad);
    r[0][0] = cy;       r[0][1] = 0;   r[0][2] = sy;
    r[1][0] = sp * sy;  r[1][1] = cp;  r[1][2] = -sp * cy;
    r[2][0] = -cp * sy; r[2][1] = sp;  r[2][2] = cp * cy;
}
static uint32_t gsm_pack(float r, float g, float b, float a)
{
    int ri = (int)(r * 255.0f + 0.5f), gi = (int)(g * 255.0f + 0.5f), bi = (int)(b * 255.0f + 0.5f), ai = (int)(a * 255.0f + 0.5f);
    if (ri < 0) ri = 0; if (ri > 255) ri = 255;
    if (gi < 0) gi = 0; if (gi > 255) gi = 255;
    if (bi < 0) bi = 0; if (bi > 255) bi = 255;
    if (ai < 0) ai = 0; if (ai > 255) ai = 255;
    return ((uint32_t)ri << 24) | ((uint32_t)gi << 16) | ((uint32_t)bi << 8) | (uint32_t)ai;
}
static int gsm_cmp(const void *a, const void *b)
{
    float za = ((const GsmTri *)a)->z, zb = ((const GsmTri *)b)->z;
    return za < zb ? -1 : za > zb ? 1 : 0;
}
/* Fills out[] with up to cap triangles, far to near. Returns the count. */
static int gsm_build(const GsmMesh *m, const GsmParams *p, GsmTri *out, int cap)
{
    static const float L[3] = {0.25f / 1.1942f, 1.0f / 1.1942f, 0.55f / 1.1942f}; /* |(.25,1,.55)| = 1.1942 */
    static const float amb[3] = {92 / 255.0f, 96 / 255.0f, 112 / 255.0f}, lc[3] = {235 / 255.0f, 235 / 255.0f, 240 / 255.0f};
    float rot[3][3], s, cx, cy, yaw, tr, tg, tb, ta;
    int t, k, n = 0;
    if (!m || !m->ntri || cap < 1) return 0;
    yaw = p->yaw + p->spin * p->clock / 60.0f;
    gsm_rotation(fmodf(yaw, 360.0f), p->pitch, rot);
    s = gsm_fit(m, p->w, p->h, p->margin, p->pitch);
    cx = p->x + p->w * 0.5f; cy = p->y + p->h * 0.5f;
    tr = (float)((p->tint >> 24) & 255) / 255.0f * p->dim;
    tg = (float)((p->tint >> 16) & 255) / 255.0f * p->dim;
    tb = (float)((p->tint >> 8) & 255) / 255.0f * p->dim;
    ta = (float)(p->tint & 255) / 255.0f * p->alpha;
    for (t = 0; t < m->ntri && n < cap; ++t) {
        GsmTri *o = &out[n];
        float vx[3], vy[3], vz[3], ex, ey, ez, fx, fy, nz;
        for (k = 0; k < 3; ++k) {
            const int i = m->idx[t * 3 + k];
            const float *q = m->pos[i];
            float px = q[0] - m->center[0], py = q[1] - m->center[1], pz = q[2] - m->center[2];
            float lit = 1.0f, lr = 1, lg = 1, lb = 1;
            vx[k] = rot[0][0] * px + rot[0][1] * py + rot[0][2] * pz;
            vy[k] = rot[1][0] * px + rot[1][1] * py + rot[1][2] * pz;
            vz[k] = rot[2][0] * px + rot[2][1] * py + rot[2][2] * pz;
            if (m->has_normals) {
                const float *nn = m->nrm[i];
                float nx = rot[0][0] * nn[0] + rot[0][1] * nn[1] + rot[0][2] * nn[2];
                float ny = rot[1][0] * nn[0] + rot[1][1] * nn[1] + rot[1][2] * nn[2];
                float nzz = rot[2][0] * nn[0] + rot[2][1] * nn[1] + rot[2][2] * nn[2];
                float d = nx * L[0] + ny * L[1] + nzz * L[2];
                if (d < 0) d = 0;
                lr = amb[0] + lc[0] * d; lg = amb[1] + lc[1] * d; lb = amb[2] + lc[2] * d;
                if (lr > 1) lr = 1; if (lg > 1) lg = 1; if (lb > 1) lb = 1;
            }
            (void)lit;
            o->x[k] = cx + vx[k] * s; o->y[k] = cy - vy[k] * s;
            o->u[k] = m->uv[i][0]; o->v[k] = m->uv[i][1];
            o->c[k] = gsm_pack(lr * tr, lg * tg, lb * tb, ta);
        }
        ex = vx[1] - vx[0]; ey = vy[1] - vy[0]; ez = vz[1] - vz[0];
        fx = vx[2] - vx[0]; fy = vy[2] - vy[0];
        (void)ez;
        nz = ex * fy - ey * fx; /* the face normal's z in the eye: > 0 faces the viewer when wound CCW */
        if (p->cull && nz <= 0.0f) continue;
        o->z = (vz[0] + vz[1] + vz[2]) * (1.0f / 3.0f);
        ++n;
    }
    qsort(out, (size_t)n, sizeof out[0], gsm_cmp);
    return n;
}
#endif
