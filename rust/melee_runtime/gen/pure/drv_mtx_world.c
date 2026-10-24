/* LINK: src/sysdolphin/baselib/mtx.c pc/gameworld/mtx_pc.c src/melee/lb/lbtrigf.c libs/dolphin/src/dolphin/mtx/mtx44.c */
/* PLAIN: rust/melee_runtime/oracle/stubs_cobj.c */
/* C oracle for melee_runtime::tu::mtx_world (HSD_MtxGetScale, HSD_MtxInverseConcat): baselib/mtx.c as built.
 * One line per case: op case hash (the outputs, with the destination aliased to a source in some cases). */
#include <sysdolphin/baselib/mtx.h>
#include "common.h"
extern int printf(const char*, ...);
void* orc_alloc(unsigned n) { extern void* calloc(unsigned, unsigned); return calloc(1, n); }

static void rm(Mtx m, int singular)
{
    int i, j;
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) m[i][j] = rndf();
    if (singular) for (j = 0; j < 4; j++) m[1][j] = m[0][j] * 2.0f;
}

int main(void)
{
    int op, k;
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 400; k++) {
            Mtx inv, src, dst; Vec3 sc; u64_ h = FNV0; int sing = (rnd() % 5 == 0);
            rm(inv, sing); rm(src, 0); rm(dst, 0);
            sc.x = 77; sc.y = 78; sc.z = 79;
            switch (op) {
            case 0: HSD_MtxGetScale(src, &sc); h = fnv(&sc, sizeof sc, h); break;
            case 1: HSD_MtxInverseConcat(inv, src, dst); h = fnv(dst, sizeof(Mtx), h); break;
            case 2: HSD_MtxInverseConcat(inv, src, src); h = fnv(src, sizeof(Mtx), h); break;
            case 3: HSD_MtxInverseConcat(inv, inv, inv); h = fnv(inv, sizeof(Mtx), h); break;
            }
            printf("op%d %d %016llx\n", op, k, h);
        }
    }
    return 0;
}
