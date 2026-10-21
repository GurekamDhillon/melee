/* C reference for the deterministic parts of src/tu/ft_0d27.rs, ft_0cd1.rs and ft_0cdd.rs
 * (group 3, game_tus shard 73-80): ftCo_800D27C4, fn_800D2818, fn_800D2A3C (fused),
 * fn_800D2B30, fn_800D2BF8 (ft_0D27.c), ftCo_800CD278 (ft_0CD1.c) and ftCo_800CDE94
 * (ft_0CDD.c), cut verbatim into tu_ft_0d27_extract.inc by tu_ft_0d27_run.py. Calls out of
 * the cut are recording stubs; tests/tu_ft_0d27.rs records the same. Each line:
 * op, case, return bits, field hash, call-trace hash. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftcommon.h"
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/aobj.h>
#include <dolphin/mtx.h>

#include <Runtime/platform.h>

ftCommonData* p_ftCommonData;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj GO;
static u32 g_aflags;

void ftLib_SetScale(HSD_GObj* g, float s) { T(1); T(g == &GO); T(B(s)); }
void HSD_JObjAnim(HSD_JObj* j) { (void) j; T(2); }
u32 HSD_AObjGetFlags(HSD_AObj* a) { (void) a; T(3); return g_aflags; }
void ft_80084FA8(HSD_GObj* g) { T(4); T(g == &GO); }
void ft_80085030(HSD_GObj* g, float fr, float dir) { T(5); T(g == &GO); T(B(fr)); T(B(dir)); }
void HSD_JObjSetupMatrixSub(HSD_JObj* j) { (void) j; T(6); }
void C_MTXMultVec(Mtx44 m, Vec* s, Vec* d) { (void) m; T(7); T(B(s->x)); T(B(s->y)); T(B(s->z)); *d = *s; }
void PSMTXMultVec(Mtx44 m, Vec* s, Vec* d) { (void) m; T(7); T(B(s->x)); T(B(s->y)); T(B(s->z)); *d = *s; }
void ftCommon_8007E2FC(HSD_GObj* g) { T(8); T(g == &GO); }

#include "tu_ft_0d27_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 12) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: c.u = 0x7FC00000u; return c.f;
    default: c.u = (rnd() & 0x807FFFFFu) | ((0x70u + rnd() % 0x20u) << 23); return c.f;
    }
}
static u32 hv(const void* p, unsigned n) { const unsigned char* b = p; u32 h = 7; while (n--) h = h * 131u + *b++; return h; }

static Fighter F;
static ftCommonData CD;
static ftData FD;
static struct ftData_x8 FD8;
static FighterBone PARTS[4];
static HSD_JObj JO;
static HSD_AObj AO;

int main(void)
{
    int op, k;
    rs = 0x2545F491u;
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 300; k++) {
            u32 ret = 0, hsh = 0;
            Vec3 in, out;
            zero(&F, sizeof F); zero(&JO, sizeof JO);
            TR = 0;
            GO.user_data = &F;
            F.gobj = &GO;
            p_ftCommonData = &CD;
            F.ft_data = &FD; FD.x8 = &FD8; FD8.x10 = rnd() % 4;
            F.parts = PARTS; PARTS[FD8.x10].joint = &JO;
            F.x2184 = &JO; JO.aobj = &AO;
            switch (rnd() % 4) {
            case 0: F.motion_id = 325; break;
            case 1: F.motion_id = 314 + rnd() % 8; break;
            case 2: F.motion_id = 313 + (rnd() & 1) * 9; break;
            default: F.motion_id = rnd() % 400; break;
            }
            F.x2070.x2071_b0_3 = rnd() & 15;
            F.x2070.x2071_b4 = rnd() & 1;
            g_aflags = rnd() & 0x40000000u ? 0x40000000u : rnd() & 0x3FFFFFFFu;
            JO.scale.x = rg();
            F.mv.co.walk.middle_anim_frame = rg();
            F.mv.co.walk.fast_anim_frame = rg();
            CD.x420 = rg(); F.co_attrs.ground_friction = rg(); F.facing_dir = rg();
            F.x34_scale.y = rnd() % 3 ? rg() : 1.0f;
            in.x = rg(); in.y = rg(); in.z = rg();
            {
                float* v = (float*) &F.mv.co.common;
                int i;
                for (i = 0; i < (int) (sizeof F.mv.co.common / 4); i++) v[i] = rg();
            }
            F.xE4_ground_accel_1 = rg(); F.xE8_ground_accel_2 = rg(); F.gr_vel = rg();
            F.xF0_ground_kb_vel = rg(); F.xF4_ground_attacker_shield_kb_vel = rg();
            F.x74_self_accel.x = rg(); F.self_vel.y = rg(); F.x8c_kb_vel.z = rg();
            F.x98_atk_shield_kb.x = rg(); F.xD4_unk_vel.y = rg();
            if (op == 3) F.mv.co.swing.x4 = rnd() % 6;
            switch (op) {
            case 0: ret = ftCo_800D27C4(&F); break;
            case 1: ret = fn_800D2818(&F); break;
            case 2: ret = fn_800D2A3C(&GO); break;
            case 3: ftCo_800CD278(&GO); break;
            case 4: ftCo_800CDE94(&F, &in, &out); ret = B(out.x) ^ B(out.y) * 3u ^ B(out.z) * 7u; break;
            case 5: fn_800D2B30(&GO); hsh = hv(&F.mv.co.common, sizeof F.mv.co.common); break;
            case 6: fn_800D2BF8(&GO);
                hsh = B(F.xE4_ground_accel_1) ^ B(F.xE8_ground_accel_2) * 3u ^ B(F.gr_vel) * 5u
                    ^ B(F.xF0_ground_kb_vel) * 7u ^ B(F.xF4_ground_attacker_shield_kb_vel) * 11u
                    ^ hv(&F.x74_self_accel, 12) ^ hv(&F.self_vel, 12) * 13u ^ hv(&F.x8c_kb_vel, 12) * 17u
                    ^ hv(&F.x98_atk_shield_kb, 12) * 19u ^ hv(&F.xD4_unk_vel, 12) * 23u;
                break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, hsh, TR);
        }
    }
    return 0;
}
