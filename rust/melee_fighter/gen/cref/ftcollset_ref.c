/* C reference for src/tu/ftcoll.rs (the ftcoll.c capsule setters ftColl_8007ABD0 .. ftColl_HurtboxInit)
 * and melee_runtime's script_hit.rs (the hit-rule metadata table). The functions are cut verbatim from
 * ftcoll.c / ft_0DF0.c / ft_0881.c / ftchangeparam.c / pc/gameworld/script_hit_rules.inc into
 * ftcollset_extract.inc by run.py and built with the shipping contraction. DbLevel is DebugRom so the
 * staling multiplier is skipped (stale.rs has its own reference); ftCo_800BFFD0 is a logging stub.
 * One fighter with random fields, one op per case; each line is op, case, return bits, a hash of
 * every field the ops write (pointers by index, bitfields read and cleared first), the call log. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include <melee/lb/types.h>
#include <melee/lb/inlines.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/debug.h>
#include <melee/db/db.h>

#include <Runtime/platform.h>

int printf(const char*, ...);

DbLKind DbLevel;
ftCommonData* p_ftCommonData;
struct Fighter_804D6524_t* Fighter_804D6524;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
int Geno_DefineIsNone(int kind) { (void) kind; return 0; }
int Geno_DefineTemplateSkeleton(int kind) { (void) kind; return 0; }
void* memset(void*, int, unsigned long);

static u32 bff_log;
bool ftCo_800BFFD0(Fighter* fp, int a, int b)
{
    (void) fp;
    bff_log = bff_log * 31u + (u32) a;
    bff_log = bff_log * 31u + (u32) b;
    return 0;
}
f32 ft_80089118(StaleMoveTable* table, int move_id, int arg2) { (void) table; (void) move_id; (void) arg2; return 1.0f; }
StaleMoveTable* Player_GetStaleMoveTableIndexPtr(int slot) { (void) slot; return NULL; }
void OSReport(char* fmt, ...) { (void) fmt; }
ATTRIBUTE_NORETURN void HSD_Panic(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
void lbColl_80008428(HitCapsule* c) { c->state = HitCapsule_Disabled; }
void lbColl_80008434(HitCapsule* c) { c->state = HitCapsule_Enabled; }
void lbColl_80008440(HitCapsule* hit)
{
    int i;
    for (i = 0; i < 12; i++) { hit->victims_1[i].victim = NULL; }
    hit->x44 = 0;
    for (i = 0; i < 12; i++) { hit->victims_2[i].victim = NULL; }
    hit->x45 = 0;
}

#include "ftcollset_extract.inc"

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 77.0f;
}

static Fighter F;
static HSD_GObj GO;
static FighterBone BONES[8];
static HSD_JObj JO[8];
static HitCapsule HB;
static ShieldDesc SD;
static AbsorbDesc AD;
static ReflectDesc RD;
static ftHurtboxInit HI;
static struct Fighter_804D6524_t D6524;

static u32 jid(HSD_JObj* j) { int i; for (i = 0; i < 8; i++) if (j == &JO[i]) return 1u + (u32) i; return j == NULL ? 0 : 99; }
static u32 vid(void* v) { return v == NULL ? 0 : 1; }

static u32 state(void)
{
    u32 h = 0;
    int i, j;
#define H(v) (h = h * 31u + (u32) (v))
#define HV3(v) (H(B((v).x)), H(B((v).y)), H(B((v).z)))
    for (i = 0; i < 4; i++) {
        H(F.x914[i].state);
        for (j = 0; j < 12; j++) { H(vid(F.x914[i].victims_1[j].victim)); H(vid(F.x914[i].victims_2[j].victim)); }
        H(F.x914[i].x44); H(F.x914[i].x45);
    }
    H(F.hurt_capsules_len);
    for (i = 0; i < 15; i++) {
        FighterHurtCapsule* hc = &F.hurt_capsules[i];
        H(hc->capsule.state); H(hc->capsule.skip_update_pos); H(hc->capsule.bone_idx);
        H(jid(hc->capsule.bone)); HV3(hc->capsule.a_offset); HV3(hc->capsule.b_offset); H(B(hc->capsule.scale));
        H(hc->height); H(hc->is_grabbable);
    }
    H(F.x221A_b5); H(F.x221A_b6); H(F.x2219_b3);
    H(F.x221B_b0); H(F.x221B_b1); H(F.x221B_b2); H(F.x221B_b3); H(F.x221B_b4);
    H((u32) (uintptr_t) F.shield_hit_cb); H(jid(F.shield_hit.bone)); H(B(F.shield_hit.size)); HV3(F.shield_hit.offset);
    H(F.x1988); H(F.x198C); H(F.x1990);
    H(F.x2218_b4); H(F.x2218_b5); H(F.x2218_b6); H(F.x2218_b7); H(F.reflecting);
    H((u32) (uintptr_t) F.reflect_hit_cb); H(jid(F.reflect_hit.bone)); H(B(F.reflect_hit.size)); HV3(F.reflect_hit.offset);
    H(F.ReflectAttr.x1A30_maxDamage); H(B(F.ReflectAttr.x1A34_damageMul)); H(B(F.ReflectAttr.x1A38_speedMul));
    H(jid(F.absorb_hit.bone)); H(B(F.absorb_hit.size)); HV3(F.absorb_hit.offset);
    H(bff_log);
    H(HB.unk_count); H(B(HB.damage));
    {
        HRMeta* m = script_hit_metadata(&HB, 0);
        H(script_hit_meta_count); H(m != NULL);
        if (m) { H(B(m->original_damage)); H(m->original_count); H(B(m->damage_scale)); }
    }
#undef HV3
#undef H
    return h;
}

#define NOPS 11
int main(void)
{
    int op, k, i;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            HSD_GObjEvent cb;
            zero(&F, sizeof F); zero(&GO, sizeof GO); zero(&HB, sizeof HB);
            zero(script_hit_meta, sizeof script_hit_meta); script_hit_meta_count = 0;
            rs = 0x51F3u + (u32) op * 7919u + (u32) k * 104729u;
            bff_log = 0;
            DbLevel = DbLKind_DebugRom;
            GO.user_data = &F;
            F.gobj = &GO;
            for (i = 0; i < 8; i++) { BONES[i].joint = &JO[i]; BONES[i].x4_jobj2 = &JO[(i + 1) & 7]; }
            F.parts = BONES;
            F.kind = (u8) (rnd() % 40);
            F.x34_scale.y = (rnd() % 3 == 0) ? 1.0f : rf();
            if (F.x34_scale.y == 0.0f) F.x34_scale.y = 1.0f;
            F.x2068_attackID = (int) (rnd() % 20);
            F.x206C_attack_instance = (u16) rnd();
            F.smash_attrs.state = (rnd() % 3 == 0) ? SmashState_Release : 0;
            F.smash_attrs.x2120_damageMul = rf();
            F.smash_attrs.x2118_frames = rf();
            F.smash_attrs.x211C_holdFrame = rf();
            D6524.x4 = rf();
            Fighter_804D6524 = &D6524;
            for (i = 0; i < 4; i++) {
                int j;
                F.x914[i].state = (HitCapsuleState) (rnd() % 2);
                for (j = 0; j < 12; j++) {
                    F.x914[i].victims_1[j].victim = (rnd() & 1) ? (void*) &JO[j & 7] : NULL;
                    F.x914[i].victims_2[j].victim = (rnd() & 1) ? (void*) &JO[j & 7] : NULL;
                }
                F.x914[i].x44 = (u8) rnd(); F.x914[i].x45 = (u8) rnd();
            }
            F.hurt_capsules_len = (u8) (rnd() % 16);
            for (i = 0; i < 15; i++) {
                FighterHurtCapsule* hc = &F.hurt_capsules[i];
                hc->capsule.state = (HurtCapsuleState) (rnd() % 3);
                hc->capsule.skip_update_pos = rnd() & 1;
                hc->capsule.bone_idx = (int) (rnd() % 8);
                hc->capsule.scale = rf();
            }
            F.x221A_b5 = rnd() & 1; F.x2219_b3 = rnd() & 1; F.x221A_b6 = rnd() & 1;
            F.x1988 = (int) (rnd() % 4); F.x198C = (int) rnd(); F.x1990 = (int) (rnd() % 50);
            SD.bone = (int) (rnd() % 8); SD.pos.x = rf(); SD.pos.y = rf(); SD.pos.z = rf(); SD.radius = rf();
            AD.x0_bone_id = (int) (rnd() % 8); AD.x4_offset.x = rf(); AD.x4_offset.y = rf(); AD.x4_offset.z = rf(); AD.x10_size = rf();
            RD.x0_bone_id = rnd() % 8; RD.x4_max_damage = (int) (rnd() % 100); RD.x8_offset.x = rf(); RD.x8_offset.y = rf();
            RD.x8_offset.z = rf(); RD.x14_size = rf(); RD.x18_damage_mul = rf(); RD.x1C_speed_mul = rf(); RD.x20_behavior = (u8) (rnd() % 4);
            HI.bone_idx = rnd() % 8; HI.height = (HurtHeight) (rnd() % 3); HI.is_grabbable = rnd() & 1;
            HI.a_offset.x = rf(); HI.a_offset.y = rf(); HI.a_offset.z = rf();
            HI.b_offset.x = rf(); HI.b_offset.y = rf(); HI.b_offset.z = rf(); HI.scale = rf();
            cb = (rnd() & 1) ? (HSD_GObjEvent) (uintptr_t) (0x1000u + rnd() % 64) : NULL;
            HB.unk_count = rnd(); HB.damage = rf(); HB.element = rnd() % 15;
            if (rnd() & 1) {
                HRMeta* m = script_hit_metadata(&HB, 1);
                m->damage_scale = (rnd() & 1) ? 1.0f : rf();
                m->original_damage = rf();
            }
            switch (op) {
            case 0: ftColl_8007ABD0(&HB, rnd() % 200, &GO); break;
            case 1: ftColl_8007AFF8(&GO); break;
            case 2: ftColl_8007B064(&GO, (int) (rnd() % 4)); break;
            case 3: ftColl_8007B0C0(&GO, (HurtCapsuleState) (rnd() % 3)); break;
            case 4: {
                /* a bone that has a capsule most of the time */
                int bone = (rnd() % 4 == 0) ? 9 : F.hurt_capsules[rnd() % (F.hurt_capsules_len ? F.hurt_capsules_len : 1)].capsule.bone_idx;
                if (F.hurt_capsules_len == 0 || bone == 9) { F.hurt_capsules_len = 1; F.hurt_capsules[0].capsule.bone_idx = 3; bone = 3; }
                ftColl_8007B128(&GO, bone, (HurtCapsuleState) (rnd() % 3));
            } break;
            case 5: ftColl_8007B1B8(&GO, &SD, cb); break;
            case 6: ftColl_8007B62C(&GO, (int) (rnd() % 4)); break;
            case 7: ftColl_8007B760(&GO, (int) (rnd() % 60)); break;
            case 8: ftColl_CreateAbsorbHit(&GO, &AD); break;
            case 9: ftColl_CreateReflectHit(&GO, &RD, cb); break;
            case 10: ftColl_HurtboxInit(&F, &F.hurt_capsules[rnd() % 15], &HI); break;
            }
            {
                u32 b[12]; int n = 0;
                b[n++] = F.x221A_b5; b[n++] = F.x221A_b6; b[n++] = F.x2219_b3;
                b[n++] = F.x221B_b0; b[n++] = F.x221B_b1; b[n++] = F.x221B_b2; b[n++] = F.x221B_b3; b[n++] = F.x221B_b4;
                b[n++] = F.x2218_b4; b[n++] = F.x2218_b5; b[n++] = F.x2218_b6; b[n++] = F.x2218_b7;
                ret = 0; for (i = 0; i < n; i++) ret = ret * 31u + b[i];
                ret = ret * 31u + F.reflecting;
            }
            printf("%d %d %08X %08X\n", op, k, ret, state());
        }
    }
    return 0;
}
