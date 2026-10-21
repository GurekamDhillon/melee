/* C reference for src/tu/ft_0899.rs: the whole of src/melee/ft/ft_0899.c (+ MSL atan2f from
 * lbtrigf.c and lbvector.c, the game's own), included verbatim; callees outside are logging
 * stubs that draw their results from the shared xorshift stream at call time, so the Rust test
 * (tests/tu_ft_0899.rs) matches only if it makes the same calls in the same order.
 * Build: python rust/melee_fighter/gen/cref/tu_run.py tu_ft_0899 */
#include "ft/types.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>
#include "lb/lb_020A.h"
#include "mp/mplib.h"

int printf(const char*, ...);
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* s, ...) { (void) s; }
ftCommonData* p_ftCommonData;
int db_804D4AF8;

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void rv(Vec3* v) { v->x = rf(); v->y = rf(); v->z = rf(); }

int mpLib_8004DD90_Floor(int line, Vec3* pos, float* y, u32* fl, Vec3* n)
{
    printf("FLOOR %d %08X %08X\n", line, B(pos->x), B(pos->y));
    if (rnd() % 3 == 0) return -1;
    *y = rf(); *fl = rnd(); rv(n);
    return 0;
}
void mpFloorGetLeft(int line, Vec3* v) { printf("LEFT %d\n", line); rv(v); }
void mpFloorGetRight(int line, Vec3* v) { printf("RIGHT %d\n", line); rv(v); }
void mpLineGetV1Pos(int line, Vec3* v) { printf("V1 %d\n", line); rv(v); if (rnd() % 2) { v->x /= 64; v->y /= 64; } }
void mpLineGetV0Pos(int line, Vec3* v) { printf("V0 %d\n", line); rv(v); if (rnd() % 2) { v->x /= 64; v->y /= 64; } }
s32 mpLineGetNext(s32 line) { printf("NEXT %d\n", line); return rnd() % 3 == 0 ? -1 : line + 1; }
s32 mpLineGetPrev(s32 line) { printf("PREV %d\n", line); return rnd() % 3 == 0 ? -1 : line - 1; }
enum_t mpLineGetKind(int line) { printf("KIND %d\n", line); return rnd() % 4; }
Vec3* mpLineGetNormal(int line, Vec3* v) { printf("NORM %d\n", line); rv(v); return v; }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { (void) off; *out = j->translate; }
void lbBgFlash_80021410(IKState* ik)
{
    printf("IK %08X %08X %08X %08X %08X %08X\n", B(ik->pos3.x), B(ik->pos3.y), B(ik->pos3.z),
           B(ik->pos4.x), B(ik->pos4.y), B(ik->pos4.z));
    ik->jobj0->rotate.x = rf(); ik->jobj1->rotate.y = rf();
}
void lbBgFlash_80020E38(HSD_JObj* j, Vec3* v, f32 a, f32 b, f32 c)
{
    printf("E38 %08X %08X %08X %08X %08X %08X\n", B(v->x), B(v->y), B(v->z), B(a), B(b), B(c));
    j->rotate.z = rf();
}
void ftPartSetRotX(Fighter* fp, int part, f32 x) { (void) fp; printf("ROTX %d %08X\n", part, B(x)); }

#include "lb/lbtrigf.c"
#include "lb/lbvector.c"
#include "ft/ft_0899.c"

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static Fighter fp;
static ftCommonData cd;
static ftData fd;
static ftData_x58_t x58;
static FighterBone parts[6];
static HSD_JObj joints[6], root;

int main(void)
{
    int k, i;
    HSD_GObj g;
    zero(&g, sizeof g);
    p_ftCommonData = &cd;
    for (k = 0; k < 3000; k++) {
        zero(&fp, sizeof fp);
        zero(&root, sizeof root);
        zero(joints, sizeof joints);
        g.user_data = &fp;
        g.hsd_obj = &root;
        fp.gobj = &g;
        fp.ft_data = &fd;
        fd.x58 = &x58;
        fp.parts = parts;
        rs = 0x0899u + (u32) k * 104729u;
        x58.x0 = 0; x58.x8 = 1; x58.x10 = 2; x58.x1 = 3; x58.x9 = 4; x58.x11 = 5;
        x58.x4 = rf(); x58.xC = rf(); x58.x18 = rf();
        for (i = 0; i < 6; i++) {
            parts[i].joint = &joints[i];
            rv(&joints[i].translate);
            joints[i].rotate.x = rf(); joints[i].rotate.y = rf(); joints[i].rotate.z = rf(); joints[i].rotate.w = rf();
        }
        rv(&root.translate);
        fp.x34_scale.y = rf();
        fp.facing_dir = rnd() % 2 ? 1.0f : -1.0f;
        fp.coll_data.floor.index = (int) (rnd() % 50);
        fp.coll_data.floor.normal.x = rf(); fp.coll_data.floor.normal.y = rf();
        cd.x804 = rf();
        fp.x2219_b5 = rnd() % 5 == 0;
        fp.ground_or_air = rnd() % 5 == 0 ? GA_Air : GA_Ground;
        db_804D4AF8 = (int) (rnd() % 2);
        if (rnd() % 8 == 0) {
            u32 fl = rnd() % 8;
            ft_8008A1B8(&g, fl);
        } else {
            fp.x221C_u16_y = rnd() % 8;
            ft_80089B08(&g);
        }
        printf("END %d", (int) fp.x221C_u16_y);
        for (i = 0; i < 6; i++)
            printf(" %08X %08X %08X", B(joints[i].rotate.x), B(joints[i].rotate.y), B(joints[i].rotate.z));
        printf("\n");
    }
    return 0;
}
