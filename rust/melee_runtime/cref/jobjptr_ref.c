/* C reference for melee_runtime::tu::jobj_ptr (the raw-pointer jobj.c entry points: matrix-dirty
 * propagation, flag setters, DObj list accessors, JObjSortAnim and the JObjUpdateFunc animation
 * sink, with the jobj.h inline setters). The functions are cut verbatim from jobj.c by
 * gen_jobjptr_ref.py (the header inlines come from the real jobj.h) and built with the shipping
 * contraction. splArcLengthPoint / HSD_RObjGetByType are stubs over fixed values; the matrix
 * conversion cases 0x36..0x39 are left to the hsd_mtx reference.
 * One random JObj tree, one op per case; each line is op, case, return, hash of the whole tree. */
#include "ft/types.h"
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/robj.h>
#include <sysdolphin/baselib/spline.h>
#include <sysdolphin/baselib/aobj.h>
#include <sysdolphin/baselib/fobj.h>
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/list.h>

#include <Runtime/platform.h>

int printf(const char*, ...);

void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ATTRIBUTE_NORETURN void HSD_Panic(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static HSD_SList* ufc_callbacks;
static void (*dptcl_callback)(int, int lo, int hi, HSD_JObj* jobj);
static void (*jsound_callback)(s32);
static void (*ptcltgt_callback)(HSD_JObj*, s32);

static HSD_JObj JO[6];
static HSD_DObj DO[4];
static HSD_AObj AO;
static HSD_FObj FO[4];
static HSD_RObj RO;
static HSD_Spline SPL;
static int ut[6]; /* what each JObj union holds: 0 empty, 1 DObj, 2 spline (set with the matching flags) */
static u32 cblog;
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 jid(HSD_JObj* j) { int i; for (i = 0; i < 6; i++) if (j == &JO[i]) return 1u + (u32) i; return j == NULL ? 0 : 99; }

void splArcLengthPoint(Vec3* out, HSD_Spline* spl, f32 s) { (void) spl; out->x = s * 2.0f; out->y = s + 1.0f; out->z = -s; cblog = cblog * 31u + 7u; }
HSD_RObj* HSD_RObjGetByType(HSD_RObj* robj, u32 type, u32 subtype) { (void) subtype; cblog = cblog * 31u + type; return robj; }

static void ufc1(HSD_JObj* j, u32 t, f32 v) { cblog = cblog * 31u + jid(j); cblog = cblog * 31u + t; cblog = cblog * 31u + B(v); }
static void ufc2(HSD_JObj* j, u32 t, f32 v) { cblog = cblog * 17u + jid(j) + 1u; cblog = cblog * 17u + t; cblog = cblog * 17u + B(v); }
static void dpc(int a, int lo, int hi, HSD_JObj* j) { cblog = cblog * 31u + (u32) a; cblog = cblog * 31u + (u32) lo; cblog = cblog * 31u + (u32) hi; cblog = cblog * 31u + jid(j); }
static void jsc(s32 v) { cblog = cblog * 31u + (u32) v + 3u; }
static void ptc(HSD_JObj* j, s32 v) { cblog = cblog * 31u + jid(j) + 5u; cblog = cblog * 31u + (u32) v; }

float fabsf_bitwise(float f) { union { float f; u32 u; } c; c.f = f; c.u &= 0x7FFFFFFFu; return c.f; }
#include "jobjptr_extract.inc"

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return 0.0005f;
    return (float) ((int) (rnd() % 2001) - 1000) / 37.0f;
}

static u32 state(void)
{
    u32 h = 0;
    int i, r, c;
#define H(v) (h = h * 31u + (u32) (v))
    for (i = 0; i < 6; i++) {
        HSD_JObj* j = &JO[i];
        H(j->flags);
        H(jid(j->parent)); H(jid(j->child)); H(jid(j->next));
        H(B(j->translate.x)); H(B(j->translate.y)); H(B(j->translate.z));
        H(B(j->rotate.x)); H(B(j->rotate.y)); H(B(j->rotate.z)); H(B(j->rotate.w));
        H(B(j->scale.x)); H(B(j->scale.y)); H(B(j->scale.z));
        for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) H(B(j->mtx[r][c]));
        H(ut[i]);
        if (ut[i] == 1) H((j->u.dobj == NULL) ? 0 : (u32) (j->u.dobj - DO) + 1u); else if (ut[i] == 2) H(j->u.spline == &SPL);
    }
    for (i = 0; i < 4; i++) { H(DO[i].next == NULL ? 0 : (u32) (DO[i].next - DO) + 1u); }
    for (i = 0; i < 4; i++) { H(FO[i].next == NULL ? 0 : (u32) (FO[i].next - FO) + 1u); H(FO[i].obj_type); }
    H(AO.fobj == NULL ? 0 : (u32) (AO.fobj - FO) + 1u);
    H(B(RO.u.ik_hint.rotate_x));
    H(cblog);
#undef H
    return h;
}

#define NOPS 12
int main(void)
{
    int op, k, i;
    static const int utypes[] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 20, 23, 29, 30, 33, 39, 0x28, 0x29, 0x2A, 0x32, 0x33, 0x34, 0x35, 99 };
    HSD_SList n1, n2;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            HSD_JObj* j;
            u32 fl;
            zero(JO, sizeof JO); zero(DO, sizeof DO); zero(&AO, sizeof AO); zero(FO, sizeof FO); zero(&RO, sizeof RO);
            zero(&SPL, sizeof SPL);
            rs = 0x7B0u + (u32) op * 7919u + (u32) k * 104729u;
            cblog = 0;
            /* a random forest: parent index < child index, children chained through next */
            for (i = 0; i < 6; i++) {
                int pi;
                JO[i].flags = rnd() & (0x40u | 0x800000u | 0x1000000u | 0x2000000u | 0x1000u | 0x10u | 0x8u | 0x20u | 0x4000u | 0x200000u | 0x400000u | 0x80u | 0x100u);
                JO[i].translate.x = rf(); JO[i].translate.y = rf(); JO[i].translate.z = rf();
                JO[i].rotate.x = rf(); JO[i].rotate.y = rf(); JO[i].rotate.z = rf(); JO[i].rotate.w = rf();
                JO[i].scale.x = rf(); JO[i].scale.y = rf(); JO[i].scale.z = rf();
                { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) JO[i].mtx[r][c] = rf(); }
                if (i > 0) {
                    HSD_JObj* last;
                    pi = (int) (rnd() % (u32) i);
                    JO[i].parent = &JO[pi];
                    if (JO[pi].child == NULL) JO[pi].child = &JO[i];
                    else { last = JO[pi].child; while (last->next) last = last->next; last->next = &JO[i]; }
                }
            }
            for (i = 0; i < 4; i++) { DO[i].next = (i < 3 && (rnd() & 1)) ? &DO[i + 1] : NULL; }
            for (i = 0; i < 6; i++) {
                u32 m = rnd() % 3;
                ut[i] = (int) m;
                if (m == 0) { JO[i].u.dobj = NULL; JO[i].flags &= ~(0x20u | 0x4000u); }
                else if (m == 1) { JO[i].u.dobj = &DO[rnd() % 4]; JO[i].flags &= ~(0x20u | 0x4000u); }
                else { JO[i].u.spline = &SPL; JO[i].flags |= 0x4000u; }
                JO[i].robj = (rnd() & 1) ? &RO : NULL;
                JO[i].aobj = (rnd() & 1) ? &AO : NULL;
            }
            { int hi = (int) (rnd() % 6); AO.hsd_obj = (HSD_Obj*) &JO[hi]; JO[hi].u.spline = &SPL; ut[hi] = 2; JO[hi].flags |= 0x4000u; }
            for (i = 0; i < 4; i++) { FO[i].obj_type = (rnd() % 3 == 0) ? 12 : (u8) (rnd() % 12); FO[i].next = (i < 3) ? &FO[i + 1] : NULL; }
            AO.fobj = &FO[0];
            for (i = 0; i < 6; i++) JO[i].flags &= ~0x20000u; /* JOBJ_USE_QUATERNION asserts */
            n1.next = &n2; n1.data = (void*) ufc1; n2.next = NULL; n2.data = (void*) ufc2;
            ufc_callbacks = (rnd() & 1) ? &n1 : NULL;
            dptcl_callback = (rnd() & 1) ? dpc : NULL;
            jsound_callback = (rnd() & 1) ? jsc : NULL;
            ptcltgt_callback = (rnd() & 1) ? ptc : NULL;
            j = &JO[rnd() % 6];
            fl = rnd();
            switch (rnd() % 4) { case 0: fl &= 0x8u; break; case 1: fl &= 0x10u | 0x8u; break; case 2: fl &= 0x1000u | 0x40u | 0x10u; break; default: break; }
            fl &= ~0x20000u;
            switch (op) {
            case 0: HSD_JObjSetMtxDirtySub(j); break;
            case 1: HSD_JObjCheckDepend(rnd() % 8 == 0 ? NULL : j); break;
            case 2: HSD_JObjSetFlags(j, fl); break;
            case 3: HSD_JObjClearFlags(j, fl); break;
            case 4: HSD_JObjSetFlagsAll(j, fl); break;
            case 5: HSD_JObjClearFlagsAll(j, fl); break;
            case 6: ret = jid((HSD_JObj*) 0) + (u32) (HSD_JObjGetDObj(j) == NULL ? 0 : (HSD_JObjGetDObj(j) - DO) + 1); break;
            case 7: { HSD_JObj* a = rnd() % 8 == 0 ? NULL : j; HSD_DObj* b = (rnd() % 8 == 0) ? NULL : &DO[rnd() % 4]; HSD_JObjAddDObj(a, b); } break;
            case 8: {
                HSD_ObjData d;
                int t = utypes[rnd() % (sizeof utypes / sizeof utypes[0])];
                zero(&d, sizeof d);
                switch (rnd() % 3) {
                case 0: d.fv = rf(); break;
                case 1: d.iv = (int) rnd(); break;
                default: d.p.x = rf(); d.p.y = rf(); d.p.z = rf(); break;
                }
                if (t == 4 && j->aobj == NULL) j->aobj = &AO;
                JObjUpdateFunc(j, t, &d);
                ret = B(d.fv);
            } break;
            case 9: JObjSortAnim(rnd() % 8 == 0 ? NULL : &AO); break;
            case 10: { Vec3 v; v.x = rf(); v.y = rf(); v.z = rf(); HSD_JObjSetTranslate(j, &v); } break;
            case 11: HSD_JObjSetRotationZ(j, rf()); break;
            }
            printf("%d %d %08X %08X\n", op, k, ret, state());
        }
    }
    return 0;
}
