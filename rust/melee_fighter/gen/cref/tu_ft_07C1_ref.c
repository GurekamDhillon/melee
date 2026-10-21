/* C reference for src/tu/ft_07C1.rs: src/melee/ft/ft_07C1.c cut verbatim into
 * tu_ft_07C1_extract.inc by gen/cref/tu_run_s57.py; the lbColl / lb / ftColl / item callees
 * are logging stubs over 4 fighters. tests/tu_ft_07C1.rs repeats the fill order and compares
 * the log, the bitfields and pointer fields (as indices) and a hash of each native Fighter. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ft_07C1.h"
#include <melee/it/types.h>
#include <melee/it/it_26B1.h>
#include <melee/lb/lbcollision.h>
#include <melee/lb/lb_00B0.h>
#include <sysdolphin/baselib/gobj.h>

int printf(const char*, ...);
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj* heads[16];
HSD_GObj** HSD_GObjPLinkHead = heads;
ftCommonData* p_ftCommonData;
#define NF 4
static Fighter fs[NF];
static HSD_GObj go[NF];
static FighterBone bones[8];
static u8 joints[8];
static ftData fd;
static struct ftData_x34 x34;
static HSD_GObj item_go;
static int g_kind, g_b868[NF], g_acfc, g_7afc;
static float g_pos[3];
static int fi(void* p) { return p == NULL ? -1 : (u8*) p >= (u8*) fs && (u8*) p < (u8*) (fs + NF) ? (int) (((u8*) p - (u8*) fs) / (int) sizeof(Fighter)) : 99; }
static int gi(void* p) { return p == NULL ? -1 : (HSD_GObj*) p >= go && (HSD_GObj*) p < go + NF ? (int) ((HSD_GObj*) p - go) : 99; }
static int ji(void* p) { return p == NULL ? -1 : (int) ((u8*) p - joints); }
ItemKind itGetKind(Item_GObj* g) { printf("KIND %d\n", g == &item_go); return g_kind; }
void ftCo_800C555C(Fighter_GObj* g) { printf("C555C %d\n", gi(g)); }
void lbColl_80008440(HitCapsule* h) { printf("8440 %d\n", fi(h)); h->x4C.x = 1.5f; }
void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* b)
{ printf("B1CC %d %08X %08X %08X\n", ji(j), B(a->x), B(a->y), B(a->z)); b->x = g_pos[0]; b->y = g_pos[1]; b->z = g_pos[2]; }
bool lbColl_8000ACFC(void* v, HitCapsule* h) { printf("ACFC %d %d\n", fi(v), fi(h)); return g_acfc; }
bool lbColl_80008688(HitCapsule* h, int t, void* v) { printf("8688 %d %d %d\n", fi(h), t, fi(v)); return 0; }
s32 ftColl_8007B868(Fighter_GObj* g) { printf("B868 %d\n", gi(g)); return g_b868[gi(g)]; }
bool lbColl_80007AFC(HitCapsule* a, HitCapsule* b, float x, float y)
{ printf("7AFC %d %d %08X %08X\n", fi(a), fi(b), B(x), B(y)); return g_7afc; }
void lbColl_800089B8(HitCapsule* h, UNK_T v) { printf("89B8 %d %d\n", fi(h), fi(v)); }

#include "tu_ft_07C1_extract.inc"

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
static u32 fnv(const void* p, unsigned n)
{ const unsigned char* b = p; u32 h = 2166136261u; while (n--) { h ^= *b++; h *= 16777619u; } return h; }
static ftCommonData cd;

int main(void)
{
    int op, k, i;
    p_ftCommonData = &cd;
    for (i = 0; i < 8; i++) bones[i].joint = (HSD_JObj*) &joints[i];
    fd.x34 = &x34;
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 1500; k++) {
            int n, sel;
            zero(fs, sizeof fs); zero(go, sizeof go); zero(&cd, sizeof cd);
            rs = 0x7C114u + (u32) op * 7919u + (u32) k * 104729u;
            n = 1 + (int) (rnd() % NF);
            for (i = 0; i < NF; i++) {
                Fighter* f = &fs[i];
                go[i].user_data = f;
                go[i].next = i + 1 < n ? &go[i + 1] : NULL;
                f->ft_data = &fd; f->parts = bones;
                f->item_gobj = (rnd() & 1) ? &item_go : NULL;
                f->x2219_b4 = rnd() & 1; f->x2219_b1 = (rnd() % 4) == 0;
                f->x2227_b2 = (rnd() % 4) != 0; f->is_sub_fighter = (rnd() % 6) == 0;
                f->x221C_b6 = (rnd() % 4) == 0;
                f->victim_gobj = (rnd() % 4) == 0 ? &go[rnd() % NF] : NULL;
                f->dmg.x1868_source = (rnd() % 3) == 0 ? &go[rnd() % NF] : NULL;
                f->x1064_thrownHitbox.state = rnd() % 5;
                f->x1064_thrownHitbox.x4 = rnd();
                f->x1064_thrownHitbox.jobj = (HSD_JObj*) &joints[rnd() % 8];
                f->x1064_thrownHitbox.b_offset.x = rf(); f->x1064_thrownHitbox.b_offset.y = rf();
                f->x1064_thrownHitbox.x4C.x = rf(); f->x1064_thrownHitbox.x4C.y = rf();
                f->x1064_thrownHitbox.x43_b1 = rnd() & 1; f->x1064_thrownHitbox.x43_b2 = rnd() & 1;
                f->ground_or_air = rnd() % 2;
                f->co_attrs.weight = rf();
                f->pos_delta.x = rf(); f->pos_delta.y = rf();
                f->x98_atk_shield_kb.x = rf(); f->x98_atk_shield_kb.y = rf();
                f->x34_scale.y = rf();
                g_b868[i] = (rnd() % 5) == 0 ? 2 : 0;
            }
            x34.x0 = rnd() % 8; x34.scale = rf();
            cd.hit_weight_mul = rf();
            g_kind = (rnd() & 1) ? It_Kind_Hammer : (int) (rnd() % 40);
            g_acfc = (rnd() % 4) == 0; g_7afc = rnd() & 1;
            g_pos[0] = rf(); g_pos[1] = rf(); g_pos[2] = rf();
            heads[HSD_GOBJ_PLINK_FIGHTER] = &go[0];
            sel = (int) (rnd() % n);
            printf("OP %d %d %d %d\n", op, k, n, sel);
            switch (op) {
            case 0: ft_8007C114(&go[sel]); break;
            case 1: ft_8007C17C(&go[sel]); break;
            case 2: ft_8007C224(&go[sel]); break;
            case 3: if (n > 1) ft_8007C2E0(&fs[0], &fs[0].x1064_thrownHitbox, &fs[1], &fs[1].x1064_thrownHitbox); break;
            case 4: ft_8007C4BC(&go[sel]); break;
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &fs[i];
                printf("F %d %d%d%d%d%d %d%d %d %d %d %d\n", i, f->x2219_b4, f->x2219_b1, f->x2227_b2,
                       f->is_sub_fighter, f->x221C_b6, f->x1064_thrownHitbox.x43_b1, f->x1064_thrownHitbox.x43_b2,
                       f->item_gobj != NULL, gi(f->victim_gobj), gi(f->dmg.x1868_source), ji(f->x1064_thrownHitbox.jobj));
                f->x2219_b4 = 0; f->x2219_b1 = 0; f->x2227_b2 = 0; f->is_sub_fighter = 0; f->x221C_b6 = 0;
                f->x1064_thrownHitbox.x43_b1 = 0; f->x1064_thrownHitbox.x43_b2 = 0;
                f->item_gobj = NULL; f->victim_gobj = NULL; f->dmg.x1868_source = NULL;
                f->x1064_thrownHitbox.jobj = NULL; f->ft_data = NULL; f->parts = NULL;
                printf("H %08X\n", fnv(f, sizeof *f));
            }
        }
    }
    return 0;
}
