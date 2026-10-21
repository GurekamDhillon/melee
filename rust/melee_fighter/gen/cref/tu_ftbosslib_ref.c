/* C reference for src/tu/ftbosslib.rs: the whole of src/melee/ft/ftbosslib.c and
 * src/melee/lb/lbvector.c are #included verbatim (shipping contraction via run.py). Callees
 * outside the tested function are left undefined-but-unreferenced or stubbed below.
 * Prints "BE40 tx ty tz cx cy cz a3 a4 -> vx vy out" as hex bit patterns. */
#include "ft/ftbosslib.c"
#include "lb/lbvector.c"

static u32 s = 0x1234567u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 200001) - 100000)) / 37.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    int i;
    G.user_data = &F;
    for (i = 0; i < 4000; i++) {
        Vec3 t;
        float out = 12345.0f, a3, a4;
        t.x = rf(); t.y = rf(); t.z = rf();
        F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
        a3 = (rnd() % 4 == 0) ? 0.0f : (float) (rnd() % 3000) / 3.0f;
        a4 = rf() / 1000.0f;
        printf("BE40 %08x %08x %08x %08x %08x %08x %08x %08x", B(t.x), B(t.y), B(t.z),
               B(F.cur_pos.x), B(F.cur_pos.y), B(F.cur_pos.z), B(a3), B(a4));
        ftBossLib_8015BE40(&G, &t, &out, a3, a4);
        printf(" %08x %08x %08x\n", B(F.self_vel.x), B(F.self_vel.y), B(out));
    }
    return 0;
}
