/* C reference for src/tu/ftcpuattack.rs: the whole game TU src/melee/ft/ftcpuattack.c is
 * #included (tu_run.py links with /FORCE:UNRESOLVED; the drivers reach only the functions
 * below). Drives ftCo_800B4AB0 / ftCo_800B52AC / ftCo_800B5AB0 (motion prediction + weighted
 * pick) and ftCo_800BB104 (3-frame hitbox extrapolation) on generated inputs; HSD_Randf and
 * the lbColl probes are deterministic stubs. tests/tu_ftcpuattack.rs repeats the generator. */
#include "melee/ft/ftcpuattack.c"

int printf(const char*, ...);
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 rs, rr;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
f32 HSD_Randf(void) { rr ^= rr << 13; rr ^= rr >> 17; rr ^= rr << 5; return (f32) (rr % 999u) / 1000.0f; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static float gr(void)
{
    u32 k = rnd() % 6;
    if (k == 0) return 0.0f;
    if (k == 1) return 0.000001f;
    if (k == 2) return -0.000005f;
    return rf() / 50.0f;
}
static float sc(void) { return rnd() % 3 == 0 ? 1.0f : (float) (rnd() % 200 + 1) / 100.0f; }
static float side(void) { return rnd() % 2 ? 1.0f : -1.0f; }
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 TRC;
bool lbColl_8000ACFC(void* a, HitCapsule* h) { (void) a; (void) h; return 0; }
bool lbColl_80006094(Vec3* a, Vec3* b, Vec3* c, Vec3* d, Vec3* e, Vec3* f, float s0, float s1)
{
    (void) a; (void) c; (void) d; (void) e; (void) f;
    TRC = TRC * 31u + B(b->x); TRC = TRC * 31u + B(b->y); TRC = TRC * 31u + B(b->z);
    TRC = TRC * 31u + B(s0); TRC = TRC * 31u + B(s1);
    return 0;
}

static Fighter F, T;
static Item IT;
static ItemAttr IA;
static ftCo_AttackEntry L[16];

static void mover(Fighter* f)
{
    f->cur_pos.x = rf(); f->cur_pos.y = rf();
    f->pos_delta.x = rf(); f->pos_delta.y = rf();
    f->co_attrs.terminal_velocity = rf(); f->co_attrs.gravity = gr();
    f->facing_dir = side(); f->x34_scale.y = sc();
    f->ground_or_air = rnd() % 2;
    if (rnd() % 2) {
        f->cpu.x55C = (float) (rnd() % 300); f->cpu.x560 = (float) (rnd() % 300); f->cpu.x568 = (float) (rnd() % 300);
    } else {
        f->cpu.x55C = rf(); f->cpu.x560 = rf(); f->cpu.x568 = rf();
    }
}

int main(void)
{
    int k, i, n;
    for (k = 0; k < 20000; k++) {
        int mode = k % 4, ret = 0;
        zero(&F, sizeof F); zero(&T, sizeof T); zero(&IT, sizeof IT); zero(&IA, sizeof IA); zero(L, sizeof L);
        rs = 0x1234567u + (u32) k * 2654435761u;
        if (rs == 0) rs = 1;
        rr = (u32) k * 7919u + 1u;
        mover(&F); mover(&T);
        F.cpu.level = rnd() % 10;
        F.cpu.xEC = rnd() % 3;
        for (i = 0; i < F.cpu.xEC; i++) F.cpu.xCC_array[i] = rnd() % 8 + 1;
        F.cpu.xC8 = rnd() % 8 == 0 ? rnd() % 3 : 0;
        for (i = 0; i < F.cpu.xC8; i++) F.cpu.xA8_array[i] = rnd() % 8 + 1;
        F.cpu.x80 = rnd() % 100;
        F.cpu.x570 = rf() / 100.0f;
        n = rnd() % 12 + 1;
        for (i = 0; i < n; i++) {
            L[i].cmd = rnd() % 8 + 1; L[i].x04 = rnd() % 40;
            if (rnd() % 2) {
                L[i].x08 = -(float) (rnd() % 600); L[i].x0C = (float) (rnd() % 600);
                L[i].x10 = -(float) (rnd() % 600); L[i].x14 = (float) (rnd() % 600);
            } else {
                L[i].x08 = rf(); L[i].x0C = rf(); L[i].x10 = rf(); L[i].x14 = rf();
            }
            L[i].weight = rnd() % 4 == 0 ? 0.0f : (float) (rnd() % 100) / 10.0f;
            L[i].x1C = rnd() % 4 == 0 ? 3 : 1; L[i].x20 = rnd() % 10;
        }
        TRC = 0;
        if (mode == 0) {
            ret = ftCo_800B4AB0(&F, &T, L);
        } else if (mode == 1) {
            float reach = rf();
            ret = ftCo_800B52AC(&F, &T, L, reach);
        } else if (mode == 2) {
            IT.pos.x = rf(); IT.pos.y = rf(); IT.x40_vel.x = rf(); IT.x40_vel.y = rf();
            IA.x10_fall_speed = gr(); IA.x14_fall_speed_max = rf();
            IT.xC1C.top = (float) (rnd() % 400); IT.xC1C.bottom = rf(); IT.xC1C.right = (float) (rnd() % 400); IT.xC1C.left = rf();
            IT.ground_or_air = rnd() % 2;
            IT.xCC_item_attr = &IA;
            ret = ftCo_800B5AB0(&F, &IT, L);
        } else {
            Vec3 p;
            for (i = 0; i < 4; i++) {
                HitCapsule* h = &T.x914[i];
                h->state = rnd() % 4; h->x43_b2 = rnd() % 4 == 0; h->element = rnd() % 10;
                h->x4C.x = rf(); h->x4C.y = rf(); h->x4C.z = rf();
                h->x58.x = rf(); h->x58.y = rf(); h->x58.z = rf(); h->scale = rf();
            }
            p.x = rf(); p.y = rf(); p.z = rf();
            ret = ftCo_800BB104(&F, &T, &p, rf());
        }
        printf("%d %08X %08X %08X %08X %08X\n", ret, B(F.cpu.x6C.x), B(F.cpu.x6C.y), B(F.cpu.x74.x),
               B(F.cpu.x74.y), TRC);
    }
    return 0;
}
