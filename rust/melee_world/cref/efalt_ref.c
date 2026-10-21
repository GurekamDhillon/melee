/* C reference for rust/melee_world/src/tu/efalt.rs (tests/tu_efalt.rs): efAlt_Spawn 0x49D's
 * `HSD_JObjGetTranslationZ(jobj) - 6.0f` and the M_PI_2 narrowing.
 * Build: bash rust/melee_world/cref/gen_efalt_ref.sh (contraction on, gwtool lowering). */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
typedef float f32;
#define M_PI 3.14159265358979323846
#define M_PI_2 (M_PI / 2)
static uint32_t st = 0x2545F491u;
static uint32_t rnd(void) { st ^= st << 13; st ^= st >> 17; st ^= st << 5; return st; }
static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float bf(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }
__attribute__((noinline)) static f32 offz(f32 z) { return z - 6.0f; }
__attribute__((noinline)) static f32 rot(f32 v) { f32 r; if (v < 0.0f) { r = -M_PI_2; } else { r = M_PI_2; } return r; }
int main(void)
{
    int i;
    for (i = 0; i < 2000; i++) {
        uint32_t u = rnd();
        f32 z = (i & 1) ? bf((u & 0x807FFFFFu) | ((120u + u % 16u) << 23)) : bf(u);
        if (z != z) { z = 1.0f; }
        printf("%08x %08x %08x\n", fb(z), fb(offz(z)), fb(rot(z)));
    }
    return 0;
}
