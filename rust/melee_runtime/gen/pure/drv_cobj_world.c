/* LINK: src/sysdolphin/baselib/wobj.c src/sysdolphin/baselib/jobj.c src/sysdolphin/baselib/mtx.c src/sysdolphin/baselib/spline.c src/sysdolphin/baselib/dobj.c pc/gameworld/mtx_pc.c src/melee/lb/lbtrigf.c libs/dolphin/src/dolphin/mtx/mtx44.c rust/melee_runtime/oracle/helper_jobj.c */
/* PLAIN: rust/melee_runtime/oracle/stubs_cobj.c */
/* C oracle for melee_runtime::tu::cobj_world: baselib/cobj.c included verbatim over the real wobj.c.
 * One line per case: op case hash (return value, outputs and the touched fields of the CObj / WObjs). */
#include "src/sysdolphin/baselib/cobj.c"
#include "common.h"
extern int printf(const char*, ...);
extern void* calloc(unsigned, unsigned);

void* orc_alloc(unsigned n) { return calloc(1, n); }
static int g_allocs;
static unsigned g_scale;
int View_ProjectionScaleBits(void) { return (int) g_scale; }
void diag_cobj_view(const float* m, const float* e, const float* u, const float* i) { (void) m; (void) e; (void) u; (void) i; }

static u64_ H;
static void hw(unsigned w) { H = fnv(&w, 4, H); }
static void hf(float f) { hw(*(unsigned*) &f); }
static void hwobj(HSD_WObj* w) { if (!w) { hw(0xDEAD); return; } hf(w->pos.x); hf(w->pos.y); hf(w->pos.z); hw(w->flags); }
static void hcobj(HSD_CObj* c)
{
    hw(c->flags); hf(c->viewport.xmin); hf(c->viewport.xmax); hf(c->viewport.ymin); hf(c->viewport.ymax);
    hw(c->scissor.left); hw(c->scissor.right); hw(c->scissor.top); hw(c->scissor.bottom);
    hf(c->projection_param.frustum.top); hf(c->projection_param.frustum.bottom);
    hf(c->projection_param.frustum.left); hf(c->projection_param.frustum.right);
    hw(c->projection_type); hf(c->view_authored_aspect); hw(c->view_auto);
    hwobj(c->eyepos); hwobj(c->interest);
    hf(c->u.up.x); hf(c->u.up.y); hf(c->u.up.z);
    { int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) hf(c->view_mtx[i][j]); }
    if (c->proj_mtx) { int i, j; hw(1); for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) hf((*c->proj_mtx)[i][j]); } else hw(0);
}
static void rwobj(HSD_WObj* w)
{
    w->pos.x = rndf(); w->pos.y = rndf(); w->pos.z = rndf();
    w->flags = (unsigned) rnd() & 0xFFFEu;
    w->aobj = 0;
}

int main(void)
{
    int k, op;
    for (op = 0; op < 23; op++) {
        for (k = 0; k < 300; k++) {
            HSD_CObj c; HSD_WObj we, wi; Vec v; Mtx pm; Scissor sc; HSD_RectF32 vp; float t = -7, b = -9; int ret = 0;
            c = *(HSD_CObj*) calloc(1, sizeof c);
            c.flags = (unsigned) rnd();
            c.viewport.xmin = rndf(); c.viewport.xmax = rndf(); c.viewport.ymin = rndf(); c.viewport.ymax = rndf();
            c.scissor.left = (unsigned short) rnd(); c.scissor.right = (unsigned short) rnd();
            c.scissor.top = (unsigned short) rnd(); c.scissor.bottom = (unsigned short) rnd();
            c.projection_type = (unsigned char) (rnd() % 4);
            c.projection_param.frustum.top = rndf(); c.projection_param.frustum.bottom = rndf();
            c.projection_param.frustum.left = rndf(); c.projection_param.frustum.right = rndf();
            c.view_authored_aspect = rndf();
            c.view_auto = (unsigned char) (rnd() % 2);
            g_scale = 0x3F000000u + (unsigned) (rnd() % 0x01000000u);
            rwobj(&we); rwobj(&wi);
            if (rnd() % 5 == 0) { wi.pos = we.pos; }
            if (rnd() % 7 == 0) { wi.pos.x = we.pos.x + 1e-39f; wi.pos.y = we.pos.y; wi.pos.z = we.pos.z; }
            c.eyepos = rnd() % 8 == 0 ? 0 : &we;
            c.interest = rnd() % 8 == 0 ? 0 : &wi;
            v.x = rndf(); v.y = rndf(); v.z = rndf();
            {
                int i, j;
                c.u.up.x = rndf(); c.u.up.y = rndf(); c.u.up.z = rndf();
                for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) c.view_mtx[i][j] = rndf();
                for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) pm[i][j] = rndf();
                c.proj_mtx = rnd() % 3 == 0 ? 0 : &pm;
                if (op == 18 || op == 19) { c.eyepos = &we; c.interest = &wi; }
            }
            H = FNV0;
            switch (op) {
            case 0: ret = (int) HSD_CObjGetFlags(&c); break;
            case 1: hf(HSD_CObjGetFov(rnd() % 6 == 0 ? 0 : &c)); break;
            case 2: hf(HSD_CObjGetAuthoredAspect(rnd() % 6 == 0 ? 0 : &c)); break;
            case 3: HSD_CObjSetAspect(rnd() % 6 == 0 ? 0 : &c, rndf()); break;
            case 4: {
                int m = (int) (rnd() % 4);
                HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c;
                HSD_CObjGetPerspective(cp, m & 1 ? &t : 0, m & 2 ? &b : 0);
                hf(t); hf(b);
                break;
            }
            case 5: sc.left = 1; sc.right = 2; sc.top = 3; sc.bottom = 4; HSD_CObjGetScissor(rnd() % 6 == 0 ? 0 : &c, &sc);
                hw(sc.left); hw(sc.right); hw(sc.top); hw(sc.bottom); break;
            case 6: vp.xmin = 1; vp.xmax = 2; vp.ymin = 3; vp.ymax = 4; HSD_CObjGetViewportf(rnd() % 6 == 0 ? 0 : &c, &vp);
                hf(vp.xmin); hf(vp.xmax); hf(vp.ymin); hf(vp.ymax); break;
            case 7: vp.xmin = rndf(); vp.xmax = rndf(); vp.ymin = rndf(); vp.ymax = rndf();
                HSD_CObjSetViewportf(rnd() % 6 == 0 ? 0 : &c, &vp); break;
            case 8: HSD_CObjSetProjectionType(rnd() % 6 == 0 ? 0 : &c, (unsigned) rnd()); break;
            case 9: if (c.interest) { HSD_CObjGetInterest(&c, &v); hf(v.x); hf(v.y); hf(v.z); } break;
            case 10: if (c.interest) HSD_CObjSetInterest(&c, &v); break;
            case 11: if (c.eyepos) { HSD_CObjGetEyePosition(&c, &v); hf(v.x); hf(v.y); hf(v.z); } break;
            case 12: if (c.eyepos) HSD_CObjSetEyePosition(&c, &v); break;
            case 13: ret = HSD_CObjGetEyeVector(&c, &v); hf(v.x); hf(v.y); hf(v.z); break;
            case 14: ret = HSD_CObjGetEyeVector(&c, 0); break;
            case 15: { HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c; Vec* vp = rnd() % 6 == 0 ? 0 : &v; ret = HSD_CObjGetUpVector(cp, vp); hf(v.x); hf(v.y); hf(v.z); break; }
            case 16: { HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c; Vec* vp = rnd() % 6 == 0 ? 0 : &v; HSD_CObjSetUpVector(cp, vp); break; }
            case 17: { HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c; float roll = rndf(); HSD_CObjSetRoll(cp, roll); break; }
            case 18: { float* m = (float*) HSD_CObjGetViewingMtxPtr(&c); ret = m == &c.view_mtx[0][0] ? 1 : 2; break; }
            case 19: { float* m = (float*) HSD_CObjGetInvViewingMtxPtrDirect(&c); ret = m == (float*) &pm ? 1 : m == (float*) c.proj_mtx ? 2 : 3; break; }
            case 20: { HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c; unsigned short l = (unsigned short) rnd(), r_ = (unsigned short) rnd(), t_ = (unsigned short) rnd(), b_ = (unsigned short) rnd(); HSD_CObjSetScissorx4(cp, l, r_, t_, b_); break; }
            case 21: { HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c; float l = rndf(), r_ = rndf(), t_ = rndf(), b_ = rndf(); HSD_CObjSetViewportfx4(cp, l, r_, t_, b_); break; }
            case 22: {
                int m = (int) (rnd() % 16); float o[4] = { -1, -2, -3, -4 };
                HSD_CObj* cp = rnd() % 6 == 0 ? 0 : &c;
                HSD_CObjGetOrtho(cp, m & 1 ? &o[0] : 0, m & 2 ? &o[1] : 0, m & 4 ? &o[2] : 0, m & 8 ? &o[3] : 0);
                hf(o[0]); hf(o[1]); hf(o[2]); hf(o[3]);
                break;
            }
            }
            hw((unsigned) ret);
            hcobj(&c);
            printf("op%d %d %016llx\n", op, k, H);
        }
    }
    return 0;
}
