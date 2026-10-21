/* Oracle scenarios for the logic half of cobj.c (build_cobj.sh). Compiled with the game flags;
 * includes cobj.c itself so the statics (upvec2roll, roll2upvec, makeProjectionMtx) are
 * reachable. Output goes through the native driver (drv_*).
 *
 * Rows (all hex words):
 *   S <state>                        a new camera
 *   O <op> <isnull> <scale> <args...> | <results...> | <FNV-1a of state>
 * state = flags, viewport x4, scissor x4, eye present, eye flags, eye pos x3, interest present,
 *         interest flags, interest pos x3, u x3, near, far, param x4, projection type,
 *         view_mtx x12, proj_mtx present, proj_mtx x12, view_authored_aspect, view_auto (56). */
#include "../../../src/sysdolphin/baselib/cobj.c"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_tag(const char* t);
void drv_u32(unsigned v);
void drv_eol(void);
void drv_hash_begin(void);
void drv_hash_end(void);
void* orc_alloc(unsigned n);
extern unsigned g_scale_bits;

int View_ProjectionScaleBits(void) { return (int) g_scale_bits; }
void diag_cobj_view(const float* m, const float* e, const float* u, const float* i)
{
    (void) m; (void) e; (void) u; (void) i;
}

static u32 fbits(f32 f) { return *(u32*) &f; }
/* NaN is written as 7fc00000: its sign is not stable across the x86 C and the Rust port's opt
 * levels (see group-runtime.md, round 7), and no ported function stores a NaN's bits. */
static void out_f(f32 f) { drv_u32(f != f ? 0x7FC00000u : fbits(f)); }
static void out_v(Vec3* v) { out_f(v->x); out_f(v->y); out_f(v->z); }

static void dump(HSD_CObj* c)
{
    int i, j;
    drv_u32(c->flags);
    out_f(c->viewport.xmin); out_f(c->viewport.xmax); out_f(c->viewport.ymin); out_f(c->viewport.ymax);
    drv_u32(c->scissor.left); drv_u32(c->scissor.right); drv_u32(c->scissor.top); drv_u32(c->scissor.bottom);
    drv_u32(c->eyepos != NULL);
    drv_u32(c->eyepos ? c->eyepos->flags : 0);
    if (c->eyepos) out_v(&c->eyepos->pos); else { drv_u32(0); drv_u32(0); drv_u32(0); }
    drv_u32(c->interest != NULL);
    drv_u32(c->interest ? c->interest->flags : 0);
    if (c->interest) out_v(&c->interest->pos); else { drv_u32(0); drv_u32(0); drv_u32(0); }
    out_v(&c->u.up);
    out_f(c->near); out_f(c->far);
    out_f(c->projection_param.frustum.top); out_f(c->projection_param.frustum.bottom);
    out_f(c->projection_param.frustum.left); out_f(c->projection_param.frustum.right);
    drv_u32(c->projection_type);
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) out_f(c->view_mtx[i][j]);
    drv_u32(c->proj_mtx != NULL);
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) out_f(c->proj_mtx ? (*c->proj_mtx)[i][j] : 0.0F);
    out_f(c->view_authored_aspect);
    drv_u32(c->view_auto);
}

static f32 rcoord(void)
{
    switch (drv_rand() % 6) {
    case 0: return 0.0F;
    case 1: return (f32) ((int) (drv_rand() % 5) - 2);
    case 2: return drv_randf(1.0F) * 1e-20F;
    default: return drv_randf(100.0F);
    }
}
static void rvec(Vec3* v)
{
    v->x = rcoord(); v->y = rcoord(); v->z = rcoord();
    if (drv_rand() % 10 == 0) { v->x = 0.0F; v->z = 0.0F; } /* eye along +-y: roll2upvec's other branch */
}

static HSD_WObj* new_wobj(void)
{
    HSD_WObj* w = orc_alloc(sizeof(HSD_WObj));
    w->flags = drv_rand() % 4;
    rvec(&w->pos);
    return w;
}

static HSD_CObj* new_cobj(void)
{
    HSD_CObj* c = orc_alloc(sizeof(HSD_CObj));
    int i, j;
    c->flags = (drv_rand() % 4) | (drv_rand() % 4 << 30);
    c->viewport.xmin = drv_randf(640.F); c->viewport.xmax = drv_randf(640.F);
    c->viewport.ymin = drv_randf(480.F); c->viewport.ymax = drv_randf(480.F);
    c->scissor.left = drv_rand(); c->scissor.right = drv_rand(); c->scissor.top = drv_rand(); c->scissor.bottom = drv_rand();
    c->eyepos = drv_rand() % 8 ? new_wobj() : NULL;
    c->interest = drv_rand() % 8 ? new_wobj() : NULL;
    if (c->eyepos && c->interest && drv_rand() % 4 == 0) {
        /* a (nearly) vertical eye: roll2upvec's |eye.y| ~ 1 branch */
        c->interest->pos = c->eyepos->pos;
        c->interest->pos.y += drv_randf(50.F);
        if (drv_rand() % 2) c->interest->pos.x += drv_randf(0.004F);
        if (drv_rand() % 2) c->interest->pos.z += drv_randf(0.004F);
    }
    rvec(&c->u.up);
    if (drv_rand() % 3 == 0) { f32 r = drv_randf(4.F); c->u.up.x = r; }
    c->near = drv_randf(10.F); c->far = drv_randf(10000.F);
    c->projection_param.frustum.top = drv_randf(90.F); c->projection_param.frustum.bottom = drv_randf(3.F);
    c->projection_param.frustum.left = drv_randf(100.F); c->projection_param.frustum.right = drv_randf(100.F);
    c->projection_type = drv_rand() % 5;
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) c->view_mtx[i][j] = drv_randf(2.F);
    if (drv_rand() % 3 == 0) {
        c->proj_mtx = orc_alloc(sizeof(Mtx));
        for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) (*c->proj_mtx)[i][j] = drv_randf(2.F);
    }
    c->view_authored_aspect = drv_randf(2.F);
    c->view_auto = drv_rand() % 2;
    return c;
}

#define NOPS 53
void orc_cobj_run(int nscen, int nops)
{
    int s, k, i;
    for (s = 0; s < nscen; s++) {
        HSD_CObj* c = new_cobj();
        drv_tag("S"); dump(c); drv_eol();
        for (k = 0; k < nops; k++) {
            int op = drv_rand() % NOPS;
            int both = c->eyepos && c->interest;
            int isnull = drv_rand() % 25 == 0;
            HSD_CObj* cn;
            Vec3 v, r;
            f32 a0 = drv_randf(100.F), a1 = drv_randf(100.F), a2 = drv_randf(100.F), a3 = drv_randf(100.F);
            u32 w0 = drv_rword();
            g_scale_bits = fbits(drv_rand() % 3 ? 1.0F : drv_randf(2.F));
            /* ops that dereference the camera or its WObjs unconditionally in the C */
            if ((op >= 5 && op <= 8 && !(op <= 6 ? c->interest : c->eyepos)) ||
                ((op == 10 || (op >= 15 && op <= 17)) && !both) ||
                (op == 18 && !(c->flags & 0x80000000) && !c->proj_mtx))
            {
                op = 0;
            }
            if (op == 0 || op == 1 || op == 2 || (op >= 5 && op <= 8) || (op >= 15 && op <= 18) || op >= 50)
                isnull = 0;
            cn = isnull ? NULL : c;
            rvec(&v);
            if (op == 52) {
                /* roll2upvec on a nearly vertical eye (eyepos 0, interest v): its fused y*y + z*z */
                if (!both) op = 0;
                v.x = drv_randf(0.01F); v.y = drv_rand() % 2 ? 1.0F + drv_randf(0.01F) : -1.0F; v.z = drv_randf(0.01F);
            }
            if (op == 12 && drv_rand() % 4 == 0) { v.x = v.y = v.z = 0.0F; }
            else if (op == 12 && both && drv_rand() % 3 == 0) {
                /* up along the eye: upvec2roll's 1 - |dot| < FLT_MIN branch */
                f32 k = drv_rand() % 2 ? 1.0F : -1.0F;
                VECSubtract(&c->interest->pos, &c->eyepos->pos, &v);
                if (!vec_normalize_check(&v, &v)) { v.x *= k; v.y *= k; v.z *= k; }
            }
            drv_tag("O"); drv_u32(op); drv_u32(isnull); drv_u32(g_scale_bits);
            out_f(a0); out_f(a1); out_f(a2); out_f(a3); drv_u32(w0); out_v(&v);
            drv_tag("|");
            switch (op) {
            case 0: HSD_CObjSetMtxDirty(c); break;
            case 1: drv_u32(HSD_CObjMtxIsDirty(c)); break;
            case 2: drv_u32(HSD_CObjGetFlags(c)); break;
            case 3: HSD_CObjSetFlags(cn, w0); break;
            case 4: HSD_CObjClearFlags(cn, w0); break;
            case 5: HSD_CObjGetInterest(c, &r); out_v(&r); break;
            case 6: HSD_CObjSetInterest(c, &v); break;
            case 7: HSD_CObjGetEyePosition(c, &r); out_v(&r); break;
            case 8: HSD_CObjSetEyePosition(c, &v); break;
            case 9: drv_u32(HSD_CObjGetEyeVector(cn, &r)); out_v(&r); break;
            case 10: out_f(HSD_CObjGetEyeDistance(cn)); break;
            case 11: drv_u32(HSD_CObjGetUpVector(cn, &r)); out_v(&r); break;
            case 12: HSD_CObjSetUpVector(cn, &v); break;
            case 13: drv_u32(HSD_CObjGetLeftVector(cn, &r)); out_v(&r); break;
            case 14: HSD_CObjSetRoll(cn, (w0 & 3) ? a0 * 0.05F : (c ? c->u.roll : 0.0F)); break;
            case 15: HSD_CObjSetupViewingMtx(c); break;
            case 16: { Mtx m; int x, y; HSD_CObjGetViewingMtx(c, m); for (x = 0; x < 3; x++) for (y = 0; y < 4; y++) out_f(m[x][y]); break; }
            case 17: { MtxPtr m = HSD_CObjGetInvViewingMtxPtr(c); drv_u32(m != NULL); break; }
            case 18: { MtxPtr m = HSD_CObjGetInvViewingMtxPtrDirect(c); drv_u32(m != NULL); break; }
            case 19: out_f(HSD_CObjGetFov(cn)); break;
            case 20: HSD_CObjSetFov(cn, a0); break;
            case 21: out_f(HSD_CObjGetAspect(cn)); break;
            case 22: HSD_CObjSetAspect(cn, a0 * 0.02F); break;
            case 23: out_f(HSD_CObjGetAuthoredAspect(cn)); break;
            case 24: out_f(HSD_CObjGetTop(cn)); break;
            case 25: HSD_CObjSetTop(cn, a0); break;
            case 26: out_f(HSD_CObjGetBottom(cn)); break;
            case 27: HSD_CObjSetBottom(cn, a0); break;
            case 28: out_f(HSD_CObjGetLeft(cn)); break;
            case 29: HSD_CObjSetLeft(cn, a0); break;
            case 30: out_f(HSD_CObjGetRight(cn)); break;
            case 31: HSD_CObjSetRight(cn, a0); break;
            case 32: out_f(HSD_CObjGetNear(cn)); break;
            case 33: HSD_CObjSetNear(cn, a0); break;
            case 34: out_f(HSD_CObjGetFar(cn)); break;
            case 35: HSD_CObjSetFar(cn, a0); break;
            case 36: { Scissor sc; sc.left = sc.right = sc.top = sc.bottom = 7; HSD_CObjGetScissor(cn, &sc);
                       drv_u32(sc.left); drv_u32(sc.right); drv_u32(sc.top); drv_u32(sc.bottom); break; }
            case 37: { Scissor sc; sc.left = w0; sc.right = w0 >> 16; sc.top = w0 >> 3; sc.bottom = w0 >> 9; HSD_CObjSetScissor(cn, &sc); break; }
            case 38: HSD_CObjSetScissorx4(cn, w0, w0 >> 16, w0 >> 3, w0 >> 9); break;
            case 39: { HSD_RectF32 vp; vp.xmin = vp.xmax = vp.ymin = vp.ymax = 3.0F; HSD_CObjGetViewportf(cn, &vp);
                       out_f(vp.xmin); out_f(vp.xmax); out_f(vp.ymin); out_f(vp.ymax); break; }
            case 40: { HSD_RectS16 vp; vp.xmin = w0; vp.xmax = w0 >> 16; vp.ymin = w0 >> 3; vp.ymax = w0 >> 9; HSD_CObjSetViewport(cn, &vp); break; }
            case 41: { HSD_RectF32 vp; vp.xmin = a0; vp.xmax = a1; vp.ymin = a2; vp.ymax = a3; HSD_CObjSetViewportf(cn, &vp); break; }
            case 42: HSD_CObjSetViewportfx4(cn, a0, a1, a2, a3); break;
            case 43: drv_u32(HSD_CObjGetProjectionType(cn)); break;
            case 44: HSD_CObjSetProjectionType(cn, w0 % 5 + (w0 & 0x300)); break;
            case 45: HSD_CObjSetPerspective(cn, a0, a1 * 0.02F); break;
            case 46: HSD_CObjSetFrustum(cn, a0, a1, a2, a3); break;
            case 47: HSD_CObjSetOrtho(cn, a0, a1, a2, a3); break;
            case 48: { f32 t = 5.0F, b = 6.0F; HSD_CObjGetPerspective(cn, &t, &b); out_f(t); out_f(b); break; }
            case 49: { f32 t = 5.0F, b = 6.0F, l = 7.0F, rr = 8.0F; HSD_CObjGetOrtho(cn, &t, &b, &l, &rr);
                       out_f(t); out_f(b); out_f(l); out_f(rr); break; }
            case 50: { Mtx44 m; int x, y; for (x = 0; x < 4; x++) for (y = 0; y < 4; y++) m[x][y] = 9.0F;
                       drv_u32(makeProjectionMtx(c, m)); for (x = 0; x < 4; x++) for (y = 0; y < 4; y++) out_f(m[x][y]); break; }
            case 52: {
                int res;
                c->eyepos->pos.x = c->eyepos->pos.y = c->eyepos->pos.z = 0.0F;
                c->interest->pos = v;
                r.x = r.y = r.z = 5.0F;
                res = roll2upvec(c, &r, a0 * 0.05F);
                drv_u32(res); out_v(&r);
                break;
            }
            case 51: /* the WObjs move (as an animation would) */
                if (c->eyepos && (w0 & 1)) { c->eyepos->flags |= 2; c->eyepos->pos.x = v.y; c->eyepos->pos.y = v.z; c->eyepos->pos.z = v.x; }
                if (c->interest && (w0 & 2)) { c->interest->flags |= 2; c->interest->pos = v; }
                break;
            }
            drv_tag("|"); drv_hash_begin(); dump(c); drv_hash_end(); drv_eol();
        }
    }
    (void) i;
}
