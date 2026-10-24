/* C oracle for melee_runtime::tu::mtx44::raw (MTXFrustum, MTXPerspective, MTXOrtho): libs/dolphin/src/dolphin/mtx/mtx44.c
 * included verbatim (it writes four rows through an `Mtx` parameter, so the driver passes a Mtx44). One line per case: op case hash. */
#include "libs/dolphin/src/dolphin/mtx/mtx44.c"
#include "common.h"
extern int printf(const char*, ...);

int main(void)
{
    int k;
    for (k = 0; k < 400; k++) {
        f32 m[4][4]; f32 a = rndf(), b = rndf(), c = rndf(), d = rndf(), e = rndf(), f = rndf();
        u64_ h; int i, j;
        for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) m[i][j] = rndf();
        MTXFrustum(m, a, b, c, d, e, f);
        h = fnv(m, sizeof m, FNV0); printf("frustum %d %016llx\n", k, h);
        for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) m[i][j] = rndf();
        a = (k & 1) ? (f32) (rnd() % 17900) / 100.0f + 0.01f : rndf();
        MTXPerspective(m, a, b, c, d);
        h = fnv(m, sizeof m, FNV0); printf("persp %d %016llx\n", k, h);
        for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) m[i][j] = rndf();
        MTXOrtho(m, b, c, d, e, f, a);
        h = fnv(m, sizeof m, FNV0); printf("ortho %d %016llx\n", k, h);
    }
    return 0;
}
