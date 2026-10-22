/* TU: src/melee/gr/grdynamicattr.c */
/* C oracle for src/tu/grdynamicattr.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static Vec3 V;
static int LEAK;
#include "grdynamicattr_gen.h"

static void hash_pool(void)
{
    int i;
    for (i = 0; i < 4; i++) hash_grDynamicAttr_UnkStruct(&grDynamicAttr_8049EFA0[i]);
    HM(PI(grDynamicAttr_804D6960));
    HM(PI(grDynamicAttr_804D6964));
}

static int in_free(int s)
{
    grDynamicAttr_UnkStruct* c;
    for (c = grDynamicAttr_804D6964; c; c = c->next) if (c == &grDynamicAttr_8049EFA0[s]) return 1;
    return 0;
}

/* a random arrangement of the 4 slots: active list in a random order, the rest free, junk fields */
static void arrange(void)
{
    int i, j, n = (int) (rnd() % 5);
    int order[4];
    grDynamicAttr_UnkStruct* last;
    for (i = 0; i < 4; i++) order[i] = i;
    for (i = 3; i > 0; i--) { j = (int) (rnd() % (u32) (i + 1)); { int t = order[i]; order[i] = order[j]; order[j] = t; } }
    for (i = 0; i < 4; i++) fill_grDynamicAttr_UnkStruct(&grDynamicAttr_8049EFA0[i]);
    grDynamicAttr_804D6960 = 0;
    grDynamicAttr_804D6964 = 0;
    last = 0;
    for (i = 0; i < n; i++) {
        grDynamicAttr_UnkStruct* s = &grDynamicAttr_8049EFA0[order[i]];
        s->next = 0;
        if (last) last->next = s; else grDynamicAttr_804D6960 = s;
        last = s;
        s->unk1C = (int) (fv() % 4) - 1;
    }
    last = 0;
    LEAK = -1;
    if (n < 4 && (rnd() & 1)) { LEAK = order[3]; }
    for (i = n; i < 4; i++) {
        if (order[i] == LEAK) continue;
        grDynamicAttr_UnkStruct* s = &grDynamicAttr_8049EFA0[order[i]];
        s->next = 0;
        if (last) last->next = s; else grDynamicAttr_804D6964 = s;
        last = s;
    }
}

int main(void)
{
    int op, k;
    preg(&V, sizeof V);
    preg(grDynamicAttr_8049EFA0, sizeof grDynamicAttr_8049EFA0);
    freg_all();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            arrange();
            V.x = rw(); V.y = rw(); V.z = rw();
            switch (op) {
            case 0: grDynamicAttr_801CA0B4(); break;
            case 1: {
                int id = (int) fv(); if (id == -1) id = 0;
                { int a0 = (int) fv(); float f = rw(); int a3 = (int) fv();
                ret = PI(grDynamicAttr_801CA0F8(a0, &V, (enum_t) id, f, a3)); }
                break;
            }
            case 2: {
                /* an active slot, a free slot, a foreign pointer's neighbour (slot not in either list), or NULL */
                grDynamicAttr_UnkStruct* a = 0;
                u32 w = fv() % 6;
                if (w < 4) { int s = (int) w; if (!in_free(s)) a = &grDynamicAttr_8049EFA0[s]; }
                grDynamicAttr_801CA1C0(a);
                break;
            }
            case 3: grDynamicAttr_801CA224(); break;
            case 4: {
                int j;
                for (j = 0; j < 4; j++) grDynamicAttr_8049EFA0[j].unk18 = rw();
                ret = (u32) grDynamicAttr_801CA284(&V, (int) fv());
                break;
            }
            case 5: {
                /* a short sequence: init, then allocate / free / tick / query */
                int j;
                grDynamicAttr_801CA0B4();
                for (j = 0; j < 12; j++) {
                    u32 w = fv() % 4;
                    if (w == 0) {
                        int id = (int) fv(); if (id == -1) id = 1;
                        int a0 = (int) fv(); float f = rw(); int a3 = (int) fv() % 5;
                        HM(PI(grDynamicAttr_801CA0F8(a0, &V, (enum_t) id, f, a3)));
                    } else if (w == 1) { int s = (int) (fv() % 4); if (!in_free(s)) grDynamicAttr_801CA1C0(&grDynamicAttr_8049EFA0[s]); }
                    else if (w == 2) grDynamicAttr_801CA224();
                    else { V.x = rw(); V.y = rw(); HM((u32) grDynamicAttr_801CA284(&V, (int) fv())); }
                }
                break;
            }
            }
            hash_pool();
            HM(B(V.x)); HM(B(V.y)); HM(B(V.z));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
