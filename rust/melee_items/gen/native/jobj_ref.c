/* Reference for tests/native_jobj.rs: the C bodies of src/sysdolphin/baselib/jobj.c (SetMtxDirtySub, flags, AddChild)
 * and the jobj.h WithMtxDirty setters, verbatim, over a reduced HSD_JObj (only the fields they touch).
 * Build: clang -O2 -Xclang -disable-llvm-passes -ffp-contract=on -fno-fast-math jobj_ref.c -o jobj_ref.exe */
#include <stdio.h>
#include <stdlib.h>
typedef unsigned u32;
#define JOBJ_CLASSICAL_SCALE 8
#define JOBJ_MTX_DIRTY 64
#define JOBJ_PTCL 32
#define JOBJ_SPLINE 16384
#define JOBJ_INSTANCE 4096
#define JOBJ_USER_DEF_MTX 8388608
#define JOBJ_MTX_INDEP_PARENT 16777216
#define JOBJ_MTX_INDEP_SRT 33554432
#define JOBJ_USE_QUATERNION 131072
#define JOBJ_ROOT_MASK ((1 << 28) | (1 << 30) | (1 << 29))
typedef struct J { u32 flags; struct J *next, *parent, *child; float ry, sx, sy, sz, tx, ty, tz; } HSD_JObj;
#define HSD_ASSERT(l, c) do { if (!(c)) { printf("ASSERT %d\n", l); exit(3); } } while (0)
static int HSD_JObjMtxIsDirty(HSD_JObj* j) { return !(j->flags & JOBJ_USER_DEF_MTX) && (j->flags & JOBJ_MTX_DIRTY); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* jobj)
{
    jobj->flags |= 0x40;
    if (!(jobj->flags & JOBJ_INSTANCE)) {
        HSD_JObj* child = jobj->child;
        while (child != NULL) {
            if (!(child->flags & JOBJ_MTX_INDEP_PARENT)) {
                if (!HSD_JObjMtxIsDirty(child)) HSD_JObjSetMtxDirtySub(child);
            }
            child = child->next;
        }
    }
}
#define HSD_JObjSetMtxDirty(jobj) { if (jobj != NULL && !HSD_JObjMtxIsDirty(jobj)) { HSD_JObjSetMtxDirtySub(jobj); } }
void HSD_JObjSetFlags(HSD_JObj* jobj, u32 flags)
{
    if (jobj != NULL) {
        if ((jobj->flags ^ flags) & JOBJ_CLASSICAL_SCALE) {
            if (jobj != NULL && !HSD_JObjMtxIsDirty(jobj)) HSD_JObjSetMtxDirtySub(jobj);
        }
        jobj->flags |= flags;
    }
}
void HSD_JObjSetFlagsAll(HSD_JObj* jobj, u32 flags)
{
    if (jobj != NULL) {
        HSD_JObjSetFlags(jobj, flags);
        if (!(jobj->flags & JOBJ_INSTANCE)) { HSD_JObj* i; for (i = jobj->child; i != NULL; i = i->next) HSD_JObjSetFlagsAll(i, flags); }
    }
}
void HSD_JObjClearFlags(HSD_JObj* jobj, u32 arg1)
{
    if (jobj != NULL) {
        if ((jobj->flags ^ arg1) & JOBJ_CLASSICAL_SCALE) {
            if (jobj != NULL && !HSD_JObjMtxIsDirty(jobj)) HSD_JObjSetMtxDirtySub(jobj);
        }
        jobj->flags &= ~arg1;
    }
}
void HSD_JObjClearFlagsAll(HSD_JObj* jobj, u32 flags)
{
    if (jobj != NULL) {
        HSD_JObjClearFlags(jobj, flags);
        if (!(jobj->flags & JOBJ_INSTANCE)) { HSD_JObj* i; for (i = jobj->child; i != NULL; i = i->next) HSD_JObjClearFlagsAll(i, flags); }
    }
}
static void UpdateParentTrspBits(HSD_JObj* jobj, HSD_JObj* child)
{
    u32 flags = (child->flags | (child->flags << 10)) & JOBJ_ROOT_MASK;
    while (jobj != NULL) {
        if (!(flags & ~jobj->flags)) break;
        jobj->flags |= flags;
        jobj = jobj->parent;
    }
}
void HSD_JObjAddChild(HSD_JObj* jobj, HSD_JObj* child)
{
    HSD_JObj* last;
    if (jobj == NULL || child == NULL) return;
    if (jobj->child == NULL) {
        jobj->child = child;
    } else {
        last = jobj->child;
        while (last->next != NULL) last = last->next;
        last->next = child;
    }
    child->parent = jobj;
    UpdateParentTrspBits(jobj, child);
}
#define SETTER(NAME, FIELD, LINE, EXTRA) \
    void NAME(HSD_JObj* jobj, float v) { HSD_ASSERT(LINE, jobj); EXTRA jobj->FIELD = v; if (!(jobj->flags & JOBJ_MTX_INDEP_SRT)) { HSD_JObjSetMtxDirty(jobj); } }
SETTER(RotY, ry, 660, HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION));)
SETTER(ScaleX, sx, 776, )
SETTER(ScaleY, sy, 791, )
SETTER(ScaleZ, sz, 806, )
SETTER(TransX, tx, 932, )
SETTER(TransY, ty, 947, )
SETTER(TransZ, tz, 962, )

#define N 12
static HSD_JObj n[N];
static u32 st = 12345;
static u32 rnd(void) { st = st * 214013u + 2531011u; return st >> 8; }
static u32 rflags(void)
{
    static const u32 bits[] = { 8, 64, 4096, 8388608, 16777216, 33554432, 1u << 28, 1u << 29, 1u << 30, 0x20000, 1, 2, 4 };
    u32 f = 0; int i;
    for (i = 0; i < 13; i++) if (rnd() & 1) f |= bits[i];
    return f;
}
static unsigned long long h;
static void hm(u32 x) { h = (h ^ x) * 1099511628211ull; }
static union { float f; u32 u; } cv;
static void hf(float f) { cv.f = f; hm(cv.u); }
static void hash_all(void)
{
    int i;
    for (i = 0; i < N; i++) { hm(n[i].flags); hf(n[i].ry); hf(n[i].sx); hf(n[i].sy); hf(n[i].sz); hf(n[i].tx); hf(n[i].ty); hf(n[i].tz);
        hm(n[i].parent ? (u32) (n[i].parent - n) : 99); hm(n[i].child ? (u32) (n[i].child - n) : 99); hm(n[i].next ? (u32) (n[i].next - n) : 99); }
}
int main(void)
{
    int round, i;
    for (round = 0; round < 300; round++) {
        st = 1000 + round; h = 14695981039346656037ull;
        for (i = 0; i < N; i++) { n[i] = (HSD_JObj) { 0 }; n[i].flags = rflags() & ~(u32) JOBJ_INSTANCE; }
        for (i = 1; i < N; i++) { int p = rnd() % i; HSD_JObjAddChild(&n[p], &n[i]); }
        for (i = 0; i < N; i++) if (rnd() % 4 == 0) n[i].flags |= JOBJ_INSTANCE;
        hash_all();
        for (i = 0; i < 40; i++) {
            u32 op = rnd() % 11, k = rnd() % N, f = rflags(); float v = (float) (int) (rnd() % 2000 - 1000) / 8.0f;
            switch (op) {
            case 0: HSD_JObjSetMtxDirtySub(&n[k]); break;
            case 1: HSD_JObjSetFlags(&n[k], f); break;
            case 2: HSD_JObjSetFlagsAll(&n[k], f); break;
            case 3: HSD_JObjClearFlags(&n[k], f); break;
            case 4: HSD_JObjClearFlagsAll(&n[k], f); break;
            case 5: n[k].flags &= ~(u32) JOBJ_USE_QUATERNION; RotY(&n[k], v); break;
            case 6: ScaleX(&n[k], v); break;
            case 7: ScaleY(&n[k], v); break;
            case 8: ScaleZ(&n[k], v); break;
            case 9: TransX(&n[k], v); TransY(&n[k], v + 1); break;
            default: TransZ(&n[k], v); break;
            }
            hash_all();
        }
        printf("%llx\n", h);
    }
    return 0;
}
