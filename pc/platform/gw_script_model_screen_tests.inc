/* gd.kit.model: the fit/layout/light maths (gw_screen_model.h), the kit triangles it appends, and the
 * refusals of the Lua call. Headless: no GX, no stage, no disc. */
static void t_sm_cube(GsmMesh *m)
{
    static const int f[6][4] = {{0, 1, 2, 3}, {4, 5, 6, 7}, {8, 9, 10, 11}, {12, 13, 14, 15}, {16, 17, 18, 19}, {20, 21, 22, 23}};
    static const float q[24][3] = {
        {-1,-1,1},{1,-1,1},{1,1,1},{-1,1,1},          /* +Z */
        {1,-1,-1},{-1,-1,-1},{-1,1,-1},{1,1,-1},      /* -Z */
        {1,-1,1},{1,-1,-1},{1,1,-1},{1,1,1},          /* +X */
        {-1,-1,-1},{-1,-1,1},{-1,1,1},{-1,1,-1},      /* -X */
        {-1,1,1},{1,1,1},{1,1,-1},{-1,1,-1},          /* +Y */
        {-1,-1,-1},{1,-1,-1},{1,-1,1},{-1,-1,1}};     /* -Y */
    int i, k;
    memset(m, 0, sizeof *m);
    m->nvert = 24; m->ntri = 12; m->has_normals = 1;
    m->pos = malloc(24 * sizeof *m->pos); m->nrm = malloc(24 * sizeof *m->nrm);
    m->uv = malloc(24 * sizeof *m->uv); m->idx = malloc(36 * sizeof *m->idx);
    for (i = 0; i < 24; ++i) {
        for (k = 0; k < 3; ++k) m->pos[i][k] = q[i][k];
        m->uv[i][0] = (float)((i % 4) & 1); m->uv[i][1] = (float)((i % 4) >> 1);
    }
    for (i = 0; i < 6; ++i) {
        /* the face normal from two edges: outward when the face is wound counter-clockwise */
        float e1[3], e2[3], n[3], len;
        for (k = 0; k < 3; ++k) { e1[k] = q[f[i][1]][k] - q[f[i][0]][k]; e2[k] = q[f[i][2]][k] - q[f[i][0]][k]; }
        n[0] = e1[1]*e2[2] - e1[2]*e2[1]; n[1] = e1[2]*e2[0] - e1[0]*e2[2]; n[2] = e1[0]*e2[1] - e1[1]*e2[0];
        len = sqrtf(n[0]*n[0] + n[1]*n[1] + n[2]*n[2]);
        for (k = 0; k < 4; ++k) { m->nrm[f[i][k]][0] = n[0]/len; m->nrm[f[i][k]][1] = n[1]/len; m->nrm[f[i][k]][2] = n[2]/len; }
        m->idx[i*6+0] = (uint16_t)f[i][0]; m->idx[i*6+1] = (uint16_t)f[i][1]; m->idx[i*6+2] = (uint16_t)f[i][2];
        m->idx[i*6+3] = (uint16_t)f[i][0]; m->idx[i*6+4] = (uint16_t)f[i][2]; m->idx[i*6+5] = (uint16_t)f[i][3];
    }
    gsm_bounds(m);
}
static void t_sm_be32(unsigned char *p, unsigned v) { p[0] = (unsigned char)(v >> 24); p[1] = (unsigned char)(v >> 16); p[2] = (unsigned char)(v >> 8); p[3] = (unsigned char)v; }
static void t_sm_bef(unsigned char *p, float f) { unsigned v; memcpy(&v, &f, 4); t_sm_be32(p, v); }

static int test_script_model_screen(void)
{
    GsmMesh m;
    GsmParams p = {10, 20, 100, 60, 0, 0, 0, 0, 0.1f, 1, 1, 0xFFFFFFFFu, 0};
    static GsmTri a[GSM_MAX_TRIS], b[GSM_MAX_TRIS];
    char out[512];
    int rc = 1, n, i, k, yaw, pitches[3] = {0, 30, -45}, pi;
    t_sm_cube(&m);
    /* bounds: a 2-unit cube has rxz = sqrt 2, half height 1, radius sqrt 3 */
    if (fabsf(m.rxz - 1.41421356f) > 1e-4f || fabsf(m.hy - 1) > 1e-6f || fabsf(m.radius - 1.7320508f) > 1e-4f) {
        gw_test_fail("screen model bounds rxz=%f hy=%f r=%f", m.rxz, m.hy, m.radius); goto done;
    }
    /* fit: 100 x 60 with a 0.1 margin leaves 80 x 48; width allows 40/1.414 = 28.28, height 24/1 = 24; 60 x 100: 48 wide gives 24/1.414 = 16.97 */
    {
        float s = gsm_fit(&m, 100, 60, 0.1f, 0);
        if (fabsf(s - 24.0f) > 1e-3f) { gw_test_fail("screen model fit s=%f, want 24 (height-limited)", s); goto done; }
        s = gsm_fit(&m, 60, 100, 0.1f, 0);
        if (fabsf(s - 16.9706f) > 1e-2f) { gw_test_fail("screen model fit s=%f, want 16.97 (width-limited)", s); goto done; }
    }
    /* never outside the rectangle, at every yaw and for three pitches; sorted far to near */
    for (pi = 0; pi < 3; ++pi)
        for (yaw = 0; yaw < 360; yaw += 7) {
            p.yaw = (float)yaw; p.pitch = (float)pitches[pi];
            n = gsm_build(&m, &p, a, GSM_MAX_TRIS);
            if (n != 12) { gw_test_fail("screen model built %d triangles, want 12", n); goto done; }
            for (i = 0; i < n; ++i) {
                if (i && a[i].z < a[i - 1].z) { gw_test_fail("screen model not sorted far to near"); goto done; }
                for (k = 0; k < 3; ++k)
                    if (a[i].x[k] < p.x - 1e-3f || a[i].x[k] > p.x + p.w + 1e-3f ||
                        a[i].y[k] < p.y - 1e-3f || a[i].y[k] > p.y + p.h + 1e-3f) {
                        gw_test_fail("screen model leaves its rectangle at yaw %d pitch %d: %f,%f", yaw, pitches[pi], a[i].x[k], a[i].y[k]);
                        goto done;
                    }
            }
        }
    /* a spin is the clock: yaw + spin * clock / 60 is the same pose as that yaw, and it is repeatable */
    p.yaw = 0; p.pitch = 20; p.spin = 90; p.clock = 40; /* 60 degrees */
    n = gsm_build(&m, &p, a, GSM_MAX_TRIS);
    p.spin = 0; p.yaw = 60;
    if (gsm_build(&m, &p, b, GSM_MAX_TRIS) != n || memcmp(a, b, (size_t)n * sizeof a[0])) {
        gw_test_fail("screen model spin is not the clock times the rate"); goto done;
    }
    p.spin = 90; p.yaw = 0;
    gsm_build(&m, &p, b, GSM_MAX_TRIS);
    if (memcmp(a, b, (size_t)n * sizeof a[0])) { gw_test_fail("screen model is not repeatable"); goto done; }
    /* back-face culling: a cube facing the camera shows one face (2 triangles), and two when turned */
    p.spin = 0; p.pitch = 0; p.cull = 1; p.yaw = 0;
    if ((n = gsm_build(&m, &p, a, GSM_MAX_TRIS)) != 2) { gw_test_fail("cull at yaw 0 kept %d triangles, want 2", n); goto done; }
    p.yaw = 30;
    if ((n = gsm_build(&m, &p, a, GSM_MAX_TRIS)) != 4) { gw_test_fail("cull at yaw 30 kept %d triangles, want 4", n); goto done; }
    p.cull = 0; p.yaw = 0;
    /* light: the face toward +Z at yaw 0 has N.L = 0.55/1.1942 (the last, nearest triangle); a face
     * toward the light saturates; tint multiplies, dim scales RGB, alpha scales A */
    n = gsm_build(&m, &p, a, GSM_MAX_TRIS);
    {
        float nl = 0.55f / 1.1942f; int want_r = (int)((92 + 235 * nl) + 0.5f), got = (int)(a[n - 1].c[0] >> 24), top = -1;
        for (i = 0; i < n; ++i) if (top < 0 && (a[i].c[0] >> 24) == 255) top = i;
        if (abs(got - want_r) > 1) { gw_test_fail("screen model +Z face red %d, want %d", got, want_r); goto done; }
        if (top < 0) { gw_test_fail("screen model top face does not saturate"); goto done; }
    }
    p.tint = 0x80FF4020u; p.dim = 0.5f; p.alpha = 0.5f;
    n = gsm_build(&m, &p, a, GSM_MAX_TRIS);
    for (i = 0; i < n; ++i) {
        unsigned c = a[i].c[0];
        if ((c & 255) != 0x10 /* 0x20 * .5 */ || ((c >> 24) & 255) > 0x40 || ((c >> 16) & 255) > 0x80) {
            gw_test_fail("screen model tint/dim/alpha: %08x", c); goto done;
        }
    }
    /* a mesh with no normals draws unlit: the tint alone */
    free(m.nrm); m.nrm = NULL; m.has_normals = 0; p.tint = 0xFFFFFFFFu; p.dim = 1; p.alpha = 1;
    n = gsm_build(&m, &p, a, GSM_MAX_TRIS);
    for (i = 0; i < n; ++i) if (a[i].c[0] != 0xFFFFFFFFu) { gw_test_fail("unlit model colour %08x", a[i].c[0]); goto done; }
    if (!(gsm_fit(&m, 0.5f, 0.5f, 0.45f, 90) > 0)) { gw_test_fail("fit of a tiny rectangle"); goto done; }
    gsm_free(&m);

    /* the Lua call: a bad argument is refused before any drawing (a handle is not needed to reach them) */
    {
        static const char *bad[][2] = {
            {"gd.kit.model(1, 0, 0, 0, 10)", "w and h"}, {"gd.kit.model(1, 0, 0, 10, -1)", "w and h"},
            {"gd.kit.model(1, 0, 0, 10, 1e9)", "w and h"}, {"gd.kit.model(1, 0/0, 0, 10, 10)", "finite"},
            {"gd.kit.model(1, 0, 0, 10, 10, 5)", "table expected"}, {"gd.kit.model(1, 0, 0, 10, 10, {yaw=400})", "yaw must be"},
            {"gd.kit.model(1, 0, 0, 10, 10, {pitch='a'})", "pitch must be a number"}, {"gd.kit.model(1, 0, 0, 10, 10, {spin=1e6})", "spin must be"},
            {"gd.kit.model(1, 0, 0, 10, 10, {margin=0.5})", "margin must be"}, {"gd.kit.model(1, 0, 0, 10, 10, {dim=2})", "dim must be"},
            {"gd.kit.model(1, 0, 0, 10, 10, {alpha=-1})", "alpha must be"}, {"gd.kit.model(1, 0, 0, 10, 10, {t=0/0})", "t must be"},
            {"gd.kit.model(1, 0, 0, 10, 10, {})", "stale or released model handle"}, {"gd.kit.model()", "number expected"},
        };
        for (i = 0; i < (int)(sizeof bad / sizeof bad[0]); ++i) {
            char line[256];
            snprintf(line, sizeof line, "print(pcall(function() return %s end))", bad[i][0]);
            out[0] = 0;
            t_exec(line, out, sizeof out);
            if (!strstr(out, "false") || !strstr(out, bad[i][1])) {
                gw_test_fail("gd.kit.model refusal %d (%s): got \"%s\", want \"%s\"", i, bad[i][0], out, bad[i][1]); goto done;
            }
        }
    }

    /* an injected asset: the triangles reach the kit's list with their clip, vertex colours and texture */
    {
        unsigned char *mesh = calloc(1, 36 + 4 * 32 + 6 * 2), *img = calloc(1, 64);
        static const float vv[4][8] = {{-2,-1,0, 0,0, 0,0,1}, {2,-1,0, 1,0, 0,0,1}, {2,1,0, 1,1, 0,0,1}, {-2,1,0, 0,1, 0,0,1}};
        static const unsigned short ii[6] = {0, 1, 2, 0, 2, 3};
        GsStageModel saved = gs_stage_models[0];
        const char *why = "";
        const GwKitQuad *q;
        int added, before;
        memcpy(mesh, "GXMS", 4); t_sm_be32(mesh + 4, 2); t_sm_be32(mesh + 8, 4); t_sm_be32(mesh + 12, 6);
        t_sm_bef(mesh + 16, 4); t_sm_bef(mesh + 20, 2); t_sm_bef(mesh + 24, 0.1f);
        t_sm_be32(mesh + 28, 36); t_sm_be32(mesh + 32, 36 + 4 * 32);
        for (i = 0; i < 4; ++i) for (k = 0; k < 8; ++k) t_sm_bef(mesh + 36 + i * 32 + k * 4, vv[i][k]);
        for (i = 0; i < 6; ++i) { mesh[36 + 4 * 32 + i * 2] = (unsigned char)(ii[i] >> 8); mesh[36 + 4 * 32 + i * 2 + 1] = (unsigned char)ii[i]; }
        for (i = 0; i < 32; ++i) img[i] = 255;                      /* AR pairs */
        for (i = 32; i < 64; ++i) img[i] = (unsigned char)(i * 3);  /* GB pairs */
        memset(&gs_stage_models[0], 0, sizeof gs_stage_models[0]);
        gs_stage_models[0].mesh = mesh; gs_stage_models[0].bytes = 36 + 4 * 32 + 12; gs_stage_models[0].stride = 32;
        gs_stage_models[0].has_normals = 1; gs_stage_models[0].image = img; gs_stage_models[0].image_bytes = 64;
        gs_stage_models[0].tex_w = gs_stage_models[0].tex_h = 4; gs_stage_models[0].token = 90917;
        snprintf(gs_stage_models[0].path, sizeof gs_stage_models[0].path, "screen-model-test");
        gw_Kit_BeginFrame();
        before = gw_Kit_QuadCount();
        p = (GsmParams){10, 20, 100, 60, 0, 0, 0, 0, 0.1f, 1, 1, 0xFFFFFFFFu, 0};
        added = gs_sm_emit(0, &p, 1, &why);
        q = gw_Kit_QuadAt(before);
        if (added != 2 || gw_Kit_QuadCount() != before + 2 || !q || q->tex < 0 || !(q->flags & GW_KITQ_TRI) ||
            !(q->flags & GW_KITQ_VCOL) || !(q->flags & GW_KITQ_CLIP) || q->clip[0] != 10 || q->clip[1] != 20 ||
            q->clip[2] != 110 || q->clip[3] != 80) {
            gw_test_fail("screen model emit: added=%d tex=%d flags=%d (%s)", added, q ? q->tex : -9, q ? q->flags : -9, why);
        } else {
            added = gs_sm_emit(0, &p, 0, &why); /* clip=false */
            q = gw_Kit_QuadAt(before + 2);
            if (added != 2 || !q || (q->flags & GW_KITQ_CLIP)) gw_test_fail("screen model clip=false still clipped");
            else rc = 0;
        }
        /* too many triangles is a refusal, not a thinned draw */
        gsm_free(&gs_sm_cache[0].mesh); gs_sm_cache[0].token = 0;
        t_sm_be32(mesh + 12, (GSM_MAX_TRIS + 1) * 3);
        if (gs_sm_emit(0, &p, 1, &why) != -1 || !strstr(why, "too many")) { gw_test_fail("oversize screen model was not refused"); rc = 1; }
        gsm_free(&gs_sm_cache[0].mesh); gs_sm_cache[0].token = 0; gs_sm_cache[0].tex_token = 0;
        gs_stage_models[0] = saved;
        free(mesh); free(img);
        gw_Kit_BeginFrame();
    }
    return rc;
done:
    gsm_free(&m);
    return rc;
}
