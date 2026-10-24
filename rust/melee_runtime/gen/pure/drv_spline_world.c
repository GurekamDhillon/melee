/* LINK: src/sysdolphin/baselib/spline.c rust/melee_runtime/oracle/wrap.c */
/* PLAIN: rust/melee_runtime/oracle/stubs_hsd.c */
/* C oracle for melee_runtime::tu::spline_world (splGetSplinePoint / splArcLengthPoint on a raw HSD_Spline):
 * baselib/spline.c as built, random splines of every type. One line per case: op case hash. */
#include <sysdolphin/baselib/spline.h>
#include "common.h"
extern int printf(const char*, ...);

int main(void)
{
    int op, k, i, j;
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 600; k++) {
            Vec3 cv[24], out; float lens[8]; float poly[8][5]; HSD_Spline s; float u; u64_ h = FNV0;
            for (i = 0; i < 24; i++) { cv[i].x = rndf(); cv[i].y = rndf(); cv[i].z = rndf(); }
            for (i = 0; i < 8; i++) { lens[i] = (float) (rnd() % 1000) / 100.0f; for (j = 0; j < 5; j++) poly[i][j] = (float) (rnd() % 10000) / 8192.0f; }
            s.type = (u8) (rnd() % 5); s.numcv = (s16) (4 + rnd() % 4); s.tension = rndf();
            s.cv = cv; s.totalLength = (float) (1 + rnd() % 10); s.segLength = lens; s.segPoly = poly;
            switch (rnd() % 6) { case 0: u = 0.0f; break; case 1: u = 1.0f; break; case 2: u = -0.25f; break; case 3: u = 1.25f; break;
                                 default: u = (float) (rnd() % 1025) / 1024.0f; }
            out.x = 99.0f; out.y = 98.0f; out.z = 97.0f;
            if (op == 0) splGetSplinePoint(&out, &s, u); else splArcLengthPoint(&out, &s, u * s.totalLength);
            h = fnv(&out, sizeof out, h);
            printf("op%d %d %016llx\n", op, k, h);
        }
    }
    return 0;
}
