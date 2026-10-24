/* C oracle for melee_runtime::tu::mtx_pc (PSVECAdd, PSVECCrossProduct, PSMTXIdentity, PSMTXConcat,
 * MTXLightPerspective): pc/gameworld/mtx_pc.c included verbatim. One line per case: op case hash. */
#include "pc/gameworld/mtx_pc.c"
#include "common.h"
extern int printf(const char*, ...);

static void rm(Mtx m) { int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) m[i][j] = rndf(); }
static void rv(Vec* v) { v->x = rndf(); v->y = rndf(); v->z = rndf(); }

int main(void)
{
    int k;
    for (k = 0; k < 400; k++) {
        Vec a, b, c; Mtx m1, m2, m3; u64_ h;
        rv(&a); rv(&b); rv(&c);
        PSVECAdd(&a, &b, &c);
        h = fnv(&c, sizeof c, FNV0);
        printf("add %d %016llx\n", k, h);
        rv(&a); rv(&b); rv(&c);
        PSVECCrossProduct(&a, &b, &c);
        h = fnv(&c, sizeof c, FNV0);
        printf("cross %d %016llx\n", k, h);
        /* aliased destination */
        rv(&a); rv(&b);
        PSVECCrossProduct(&a, &b, &a);
        h = fnv(&a, sizeof a, FNV0);
        printf("crossa %d %016llx\n", k, h);
        rm(m1);
        PSMTXIdentity(m1);
        h = fnv(m1, sizeof(Mtx), FNV0);
        printf("ident %d %016llx\n", k, h);
        rm(m1); rm(m2); rm(m3);
        PSMTXConcat(m1, m2, m3);
        h = fnv(m3, sizeof(Mtx), FNV0);
        printf("concat %d %016llx\n", k, h);
        rm(m1); rm(m2);
        PSMTXConcat(m1, m2, m1);
        h = fnv(m1, sizeof(Mtx), FNV0);
        printf("concat_a %d %016llx\n", k, h);
        {
            float fov = rndf() * 0.1f, asp = rndf(), ss = rndf(), st = rndf(), ts = rndf(), tt = rndf();
            rm(m1);
            MTXLightPerspective(m1, fov, asp, ss, st, ts, tt);
            h = fnv(m1, sizeof(Mtx), FNV0);
            printf("lightp %d %016llx\n", k, h);
        }
    }
    return 0;
}
