/* LINK: src/sysdolphin/baselib/wobj.c src/sysdolphin/baselib/cobj.c src/sysdolphin/baselib/jobj.c src/sysdolphin/baselib/mtx.c src/sysdolphin/baselib/spline.c src/sysdolphin/baselib/dobj.c pc/gameworld/mtx_pc.c src/melee/lb/lbtrigf.c libs/dolphin/src/dolphin/mtx/mtx44.c rust/melee_runtime/oracle/helper_jobj.c */
/* PLAIN: rust/melee_runtime/oracle/stubs_cobj.c */
/* C oracle for melee_runtime::tu::jobj_world: baselib/jobj.c (linked as built) driven over random JObj
 * forests. One line per case: op case hash (flags and the tree links as node indices). */
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/dobj.h>
#include "common.h"
extern int printf(const char*, ...);
extern void* calloc(unsigned, unsigned);
extern void RecalcParentTrspBits(HSD_JObj*);
extern HSD_JObj* HSD_JObjGetPrev(HSD_JObj*);
void* orc_alloc(unsigned n) { return calloc(1, n); }
int View_ProjectionScaleBits(void) { return 0; }
void diag_cobj_view(const float* m, const float* e, const float* u, const float* i) { (void) m; (void) e; (void) u; (void) i; }

#define NN 8
static HSD_JObj N[NN];
static HSD_DObj D[NN];
static u64_ H;
static void hw(unsigned w) { H = fnv(&w, 4, H); }
static int idx(HSD_JObj* p) { int i; if (!p) return -1; for (i = 0; i < NN; i++) if (&N[i] == p) return i; return -2; }
static int didx(HSD_DObj* p) { int i; if (!p) return -1; for (i = 0; i < NN; i++) if (&D[i] == p) return i; return -2; }
static void hall(void)
{
    int i;
    for (i = 0; i < NN; i++) {
        hw(N[i].flags); hw((unsigned) idx(N[i].parent)); hw((unsigned) idx(N[i].child)); hw((unsigned) idx(N[i].next));
    }
}

int main(void)
{
    int k, op;
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 400; k++) {
            int i, last[NN];
            unsigned fl;
            int a, b, ret = 0;
            H = FNV0;
            for (i = 0; i < NN; i++) { HSD_JObj z; char* p = (char*) &z; unsigned j; for (j = 0; j < sizeof z; j++) p[j] = 0; N[i] = z; last[i] = -1; }
            /* forest over nodes 0..NN-2 (node NN-1 stays a detached singleton) */
            for (i = 0; i < NN - 1; i++) {
                int par = (i > 0 && rnd() % 4 != 0) ? (int) (rnd() % (unsigned) i) : -1;
                N[i].flags = (unsigned) rnd() & ~(1u << 12) & ~(1u << 5) & ~(1u << 14);
                if (rnd() % 5 == 0) N[i].flags |= 1u << 12;
                if (op >= 8) N[i].flags &= ~(1u << 12);
                if (par >= 0) {
                    N[i].parent = &N[par];
                    if (last[par] < 0) N[par].child = &N[i]; else N[last[par]].next = &N[i];
                    last[par] = i;
                }
            }
            N[NN - 1].flags = (unsigned) rnd() & ~(1u << 12) & ~(1u << 5) & ~(1u << 14);
            for (i = 0; i < NN; i++) {
                D[i].next = 0;
                N[i].u.dobj = rnd() % 3 == 0 ? 0 : &D[rnd() % NN];
                if (rnd() % 6 == 0) N[i].flags |= (rnd() % 2) ? (1u << 5) : (1u << 14);
            }
            fl = (unsigned) rnd();
            a = (int) (rnd() % (NN - 1));
            switch (op) {
            case 0: ret = (int) HSD_JObjGetFlags(rnd() % 8 == 0 ? 0 : &N[a]); break;
            case 1: HSD_JObjSetFlags(rnd() % 8 == 0 ? 0 : &N[a], fl); break;
            case 2: HSD_JObjClearFlags(rnd() % 8 == 0 ? 0 : &N[a], fl); break;
            case 3: HSD_JObjSetFlagsAll(rnd() % 8 == 0 ? 0 : &N[a], fl); break;
            case 4: HSD_JObjClearFlagsAll(rnd() % 8 == 0 ? 0 : &N[a], fl); break;
            case 5: HSD_JObjSetMtxDirtySub(&N[a]); break;
            case 6: ret = didx(HSD_JObjGetDObj(rnd() % 8 == 0 ? 0 : &N[a])); break;
            case 7: ret = idx(HSD_JObjGetPrev(rnd() % 8 == 0 ? 0 : &N[a])); break;
            case 8: HSD_JObjAddChild(&N[a], &N[NN - 1]); break;
            case 9: b = (int) (rnd() % (unsigned) (a + 1)) - 1; ret = idx(HSD_JObjReparent(&N[a], b < 0 ? 0 : &N[b])); break;
            case 10: HSD_JObjAddNext(&N[a], &N[NN - 1]); break;
            case 11: RecalcParentTrspBits(&N[a]); break;
            case 12: HSD_JObjAddChild(&N[NN - 1], &N[0]); break;
            }
            hw((unsigned) ret);
            hall();
            printf("op%d %d %016llx\n", op, k, H);
        }
    }
    return 0;
}
