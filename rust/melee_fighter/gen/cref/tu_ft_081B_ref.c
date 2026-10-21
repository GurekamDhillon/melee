/* C reference for src/tu/ft_081b.rs: the float-carrying functions of src/melee/ft/ft_081B.c
 * (ft_80081B38, ft_80081C88, ft_80081DD4/F2C/2084, ft_80082838, ft_80084CE4) cut verbatim into
 * tu_ft_081B_extract.inc by gen/cref/tu_run_s57.py; mpColl / mpCheck*Wall / ft_80081A00 are
 * logging stubs. tests/tu_ft_081b.rs repeats the fill order and compares every line. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ft_081B.h"
#include "ft/ft_0819.h"
#include <melee/mp/mpcoll.h>
#include <melee/mp/mplib.h>
#include <sysdolphin/baselib/gobj.h>

int printf(const char*, ...);
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj* heads[16];
HSD_GObj** HSD_GObjPLinkHead = heads;
ftCommonData* p_ftCommonData;
static Fighter fs[2];
static HSD_GObj go[2];
static FighterBone bones[8];
static u8 joints[8];
static ftData fd;
static ftData_x44_t x44;
static int g_ret, g_a00;
static int ji(void* p) { return p == NULL ? -1 : (int) ((u8*) p - joints); }
static void L(const char* n, CollData* c) { printf("%s %08X\n", n, B(c->ledge_snap_height)); }
void mpColl_80041EE4(CollData* c) { printf("1EE4 %d\n", c == &fs[0].coll_data); }
void mpColl_SetECBSource_JObj(CollData* cd, HSD_GObj* g, HSD_JObj* a, HSD_JObj* b, HSD_JObj* c,
                              HSD_JObj* d, HSD_JObj* e, HSD_JObj* f, HSD_JObj* h, float s)
{ printf("ECB %d %d %d %d %d %d %d %d %08X\n", g == &go[0], ji(a), ji(b), ji(c), ji(d), ji(e), ji(f), ji(h), B(s)); }
void mpColl_SetLedgeSnap(CollData* c, float x, float y, float h) { printf("SNAP %08X %08X %08X\n", B(x), B(y), B(h)); }
bool mpColl_800477E0(CollData* c) { L("77E0", c); return g_ret; }
bool mpColl_800471F8(CollData* c) { L("71F8", c); return g_ret; }
bool mpColl_800473CC(CollData* c) { L("73CC", c); return g_ret; }
bool mpColl_80048274(CollData* c) { L("8274", c); return g_ret; }
bool mpColl_80048160(CollData* c) { L("8160", c); return g_ret; }
bool mpColl_80048464(CollData* c) { L("8464", c); return g_ret; }
bool mpColl_80048388(CollData* c) { L("8388", c); return g_ret; }
bool mpColl_80048768(CollData* c) { L("8768", c); return g_ret; }
bool mpColl_80048578(CollData* c) { L("8578", c); return g_ret; }
bool ft_80081A00(Fighter_GObj* g) { printf("1A00\n"); return g_a00; }
bool mpCheckRightWall(float ax, float ay, float bx, float by, Vec3* a, int* b, u32* c, Vec3* d, int e, int f)
{ printf("RW %08X %08X %08X %08X %d\n", B(ax), B(ay), B(bx), B(by), e + f); return g_ret; }
bool mpCheckLeftWall(float ax, float ay, float bx, float by, Vec3* a, int* b, u32* c, Vec3* d, int e, int f)
{ printf("LW %08X %08X %08X %08X %d\n", B(ax), B(ay), B(bx), B(by), e + f); return g_ret; }

#include "tu_ft_081B_extract.inc"

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static ftCommonData cd;

int main(void)
{
    int op, k, i;
    p_ftCommonData = &cd;
    for (i = 0; i < 8; i++) bones[i].joint = (HSD_JObj*) &joints[i];
    fd.x44 = &x44;
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 1000; k++) {
            int r = 0;
            zero(fs, sizeof fs); zero(go, sizeof go);
            rs = 0x81B38u + (u32) op * 7919u + (u32) k * 104729u;
            for (i = 0; i < 2; i++) {
                Fighter* f = &fs[i];
                CollData* c = &f->coll_data;
                go[i].user_data = f;
                f->ft_data = &fd; f->parts = bones;
                f->ecb_lock = (int) rnd();
                f->allow_sdi = rnd() & 1; f->x2224_b2 = (rnd() % 3) == 0;
                f->x2064_ledgeCooldown = (rnd() % 3) == 0 ? 5 : 0;
                f->cur_pos.x = rf(); f->cur_pos.y = rf(); f->cur_pos.z = rf();
                f->x34_scale.y = rf(); f->co_attrs.weight = rf(); f->facing_dir = rf();
                c->cur_pos.x = rf(); c->cur_pos.y = rf(); c->cur_pos.z = rf();
                c->ecb_source.kind = (int) (rnd() % 3);
                c->ecb_source.x124 = rf(); c->ecb_source.x128 = rf(); c->ecb_source.x12C = rf();
                c->ledge_snap_x = rf(); c->ledge_snap_y = rf(); c->ledge_snap_height = rf();
                c->ecb.top.y = rf(); c->ecb.bottom.y = rf();
            }
            x44.unk0 = rnd() % 8; x44.unk2 = rnd() % 8; x44.unk4 = rnd() % 8;
            x44.unk6 = rnd() % 8; x44.unk8 = rnd() % 8; x44.unkA = rnd() % 8;
            x44.unkC = rf(); x44.ledge_snap_x = rf(); x44.ledge_snap_y = rf(); x44.ledge_snap_height = rf();
            cd.x1CC = rf();
            g_ret = rnd() & 1; g_a00 = (rnd() % 4) == 0;
            printf("OP %d %d\n", op, k);
            switch (op) {
            case 0: ft_80081B38(&go[0]); break;
            case 1: ft_80081C88(&go[0], rf()); break;
            case 2: r = ft_80081DD4(&go[0]); break;
            case 3: r = ft_80081F2C(&go[0]); break;
            case 4: r = ft_80082084(&go[0]); break;
            case 5: {
                ftCollisionBox a, b, *p;
                a.top = rf(); a.bottom = rf(); a.left.x = rf(); a.left.y = rf(); a.right.x = rf(); a.right.y = rf();
                zero(&b, sizeof b);
                p = ft_80082838(&b, &a, fs[0].facing_dir);
                r = p == &b;
                printf("BOX %08X %08X %08X %08X %08X %08X\n", B(p->top), B(p->bottom), B(p->left.x), B(p->left.y), B(p->right.x), B(p->right.y));
                break;
            }
            case 6: r = ft_80084CE4(&fs[0], &fs[1]); break;
            }
            {
                Fighter* f = &fs[0];
                CollData* c = &f->coll_data;
                printf("R %d %d %d | %08X %08X %08X | %08X %08X %08X | %08X %08X %08X\n", r, f->ecb_lock, c->x34_flags.b1234,
                       B(c->ecb_source.x124), B(c->ecb_source.x128), B(c->ecb_source.x12C),
                       B(c->ledge_snap_x), B(c->ledge_snap_y), B(c->ledge_snap_height),
                       B(c->x50), B(f->cur_pos.y), B(c->last_pos.x));
            }
        }
    }
    return 0;
}
