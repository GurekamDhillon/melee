/* TU: src/melee/ft/kinds/ftCommon/ftCo_ThrownKirby.c */
/* C oracle for src/tu/ftCo_ThrownKirby.rs: the whole TU (TARGET_PC) included verbatim; every
 * callee out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static HSD_JObj JO, JO2, JB[4];
static FighterBone BONES[4];
static int GENO;

static u32 PI(void* p)
{
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &GO2) return 2;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F2) return 4;
    if (p == (void*) &JO) return 5;
    if (p == (void*) &JO2) return 6;
    if (p == (void*) &JB[0]) return 7;
    if (p == (void*) &JB[1]) return 8;
    if (p == (void*) &JB[2]) return 9;
    if (p == (void*) &JB[3]) return 10;
    if (p == (void*) &F.x914[0]) return 11;
    if (p == (void*) &F.mv.co.thrownkirby.coll_box) return 12;
    if (p == (void*) &F.self_vel) return 13;
    return 99;
}

int Geno_DefineKirbyCopy(int kind) { T(1); T(kind); return GENO; }
int ftKb_MexKirbyLike(int kind) { T(2); T(kind); return SR(2); }
bool ft_800824A0(Fighter_GObj* g, ftCollisionBox* b) { T(3); T(PI(g)); T(PI(b)); return SR(3); }
void Fighter_UpdateModelScale(Fighter_GObj* g) { T(4); T(PI(g)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(5); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(6); T(PI(f)); T((u32) (s32) v); }
void ftColl_8007B62C(Fighter_GObj* g, enum_t v) { T(7); T(PI(g)); T(v); }
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(8); T(PI(f)); T(v); }
void ftCommon_InitGrab(Fighter* f, bool a, float v) { T(9); T(PI(f)); T(a); T(B(v)); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(10); T(PI(g)); }
void ftCommon_8007D5D4(Fighter* f) { T(11); T(PI(f)); }
void ftCommon_SetAccessory(Fighter* f, HSD_Joint* j) { T(12); T(PI(f)); T(j == (HSD_Joint*) &IGO); }
HSD_Joint* ftKb_SpecialN_800F5898(Fighter_GObj* g) { T(13); T(PI(g)); return (HSD_Joint*) &IGO; }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(14); T(PI(j)); }
void lb_8000C2F8(HSD_JObj* a, HSD_JObj* b) { T(15); T(PI(a)); T(PI(b)); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(16); T(PI(f)); T(p); return (Fighter_Part) (rnd() % 4); }
void ftColl_8007ABD0(HitCapsule* h, u32 dmg, Fighter_GObj* g) { T(17); T(PI(h)); T(dmg); T(PI(g)); }
void ftColl_8007AFF8(Fighter_GObj* g) { T(18); T(PI(g)); }
void ftKb_SpecialN_800F5820(Fighter_GObj* g, ftCollisionBox* b, float s) { T(19); T(PI(g)); T(PI(b)); T(B(s)); }
float ftKb_SpecialN_800F58AC(Fighter_GObj* g, Vec3* v, float f) { T(20); T(PI(g)); T(PI(v)); T(B(f)); return rw(); }
float ftKb_SpecialN_800F58D8(Fighter_GObj* g, Vec3* v, float f) { T(21); T(PI(g)); T(PI(v)); T(B(f)); return rw(); }
float ftKb_SpecialN_800F5A88(Fighter_GObj* g) { T(22); T(PI(g)); return rw(); }
float ftKb_SpecialN_800F5AB0(Fighter_GObj* g) { T(23); T(PI(g)); return rw(); }
float ftKb_SpecialN_800F5AC0(void) { T(24); return rg(); }
float ftKb_SpecialN_800F5AD8(void) { T(25); return rg(); }
void ftKb_SpecialN_800F5874(Vec2* v) { T(26); v->x = rw(); v->y = rw(); }
void ftKb_SpecialN_800F190C(Fighter_GObj* g, FighterKind k) { T(27); T(PI(g)); T(k); }
void ftKb_SpecialN_800EEEC4(Fighter_GObj* g, FighterKind k) { T(28); T(PI(g)); T(k); }
float ftKb_SpecialN_800F5A98(void) { T(29); return rw(); }
float ftKb_SpecialN_800F5A70(void) { T(30); return rg(); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(31); T(PI(f)); T(B(v)); return SR(31); }
void lbVector_Mirror(Vec3* a, Vec3* b) { T(32); T(PI(a)); T(B(b->x)); T(B(b->y)); T(B(b->z)); a->x = rw(); a->y = rw(); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(33); T(PI(g)); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir)); HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z)); HM(B(F.grab_timer));
    HM(PI(F.mv.co.thrownkirby.thrower_gobj)); HM(B(F.mv.co.thrownkirby.x4)); HM(B(F.mv.co.thrownkirby.x8));
    HM(B(F.mv.co.thrownkirby.xC)); HM(B(F.mv.co.thrownkirby.x10)); HM(F.mv.co.thrownkirby.x14);
    HM(F.mv.co.thrownkirby.x18_b0); HM(F.mv.co.thrownkirby.x18_b1);
    HM(B(F.mv.co.thrownkirby.scale.x)); HM(B(F.mv.co.thrownkirby.scale.y)); HM(B(F.mv.co.thrownkirby.scale.z));
    HM(F.take_dmg_cb != NULL); HM(F.take_dmg_2_cb != NULL); HM(F.accessory4_cb != NULL); HM(F.invisible);
    HM(F.u.kb.hat.kind); HM(F2.u.kb.hat.kind); HM(F2.mv.co.thrownkirby.x18_b1);
    HM(B(JO.scale.x)); HM(B(JO.scale.y)); HM(B(JO.scale.z));
    HM(B(JO2.scale.x)); HM(B(JO2.scale.y)); HM(B(JO2.scale.z));
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&JO, sizeof JO); zero(&JO2, sizeof JO2);
            for (i = 0; i < 4; i++) BONES[i].joint = &JB[i];
            GO.hsd_obj = &JO;
            F.parts = BONES;
            F.x20A0_accessory = &JO2;
            JO.flags = rnd() & 0x02000000; JO2.flags = rnd() & 0x02000000;
            if (rnd() & 1) JO.flags |= 0x40; if (rnd() & 1) JO2.flags |= 0x40;
            JO.scale.x = rw(); JO.scale.y = rw(); JO.scale.z = rw();
            GENO = (rnd() & 3) ? -2 : (int) (rnd() % 40) - 1;
            F.kind = (rnd() & 1) ? Ft_Kind_Kirby : (FighterKind) (rnd() % 60);
            F2.kind = (FighterKind) (rnd() % 60);
            F.u.kb.hat.kind = (FighterKind) (rnd() % 34);
            F2.u.kb.hat.kind = (FighterKind) (rnd() % 34);
            F.u.kb.hat.x8_b0 = rnd() & 1;
            F.facing_dir = rg(); F2.facing_dir = rg();
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
            F.grab_timer = rw();
            F.co_attrs.xDC = rw(); F.co_attrs.kirby_b_star_damage = (rnd() & 1) ? (float) (rnd() % 30) : rw();
            F.mv.co.thrownkirby.x4 = rw(); F.mv.co.thrownkirby.xC = rg();
            F.mv.co.thrownkirby.x10 = (rnd() & 7) ? rw() : 0.0f;
            F.mv.co.thrownkirby.x18_b0 = rnd() & 1; F.mv.co.thrownkirby.x18_b1 = rnd() & 1;
            F.mv.co.thrownkirby.scale.x = rw(); F.mv.co.thrownkirby.scale.y = rw(); F.mv.co.thrownkirby.scale.z = rw();
            F.mv.ca.specialhi.vel.x = rw();
            F.mv.ca.specialhi.vel.y = (rnd() & 1) ? rw() : 0.0f;
            F.coll_data.env_flags = rnd() & (0x3F | 0xFC0 | 0x6000);
            F.coll_data.floor.normal.x = rg(); F.coll_data.floor.normal.y = rg(); F.coll_data.floor.normal.z = rg();
            F.coll_data.right_facing_wall.normal.x = rg(); F.coll_data.right_facing_wall.normal.y = rg();
            F.coll_data.left_facing_wall.normal.x = rg(); F.coll_data.left_facing_wall.normal.z = rg();
            F.coll_data.ceiling.normal.y = rg(); F.coll_data.ceiling.normal.z = rg();
            switch (op) {
            case 0: ret = ftCo_800BD9E0(&GO, &GO2); break;
            case 1: ftCo_800BDA50(&GO); break;
            case 2: { Vec3 n; n.x = n.y = n.z = 7.0f; ftCo_800BDA74(&GO, &n); ret = B(n.x) ^ (B(n.y) * 3u) ^ (B(n.z) * 7u); } break;
            case 3: ftCo_800BDB58(&GO, &GO2); break;
            case 4: ftCo_ThrownKirbyStar_Phys(&GO); break;
            case 5: ftCo_ThrownKirbyStar_Coll(&GO); break;
            case 6: ftCo_800BE000(&GO, &GO2); break;
            case 7: ftCo_ThrownCopyStar_Phys(&GO); break;
            case 8: ftCo_ThrownCopyStar_Coll(&GO); break;
            case 9: ftCo_800BE494(&GO); break;
            case 10: ftCo_ThrownKirby_Phys(&GO); break;
            case 11: ftCo_800BE6AC(&GO); break;
            case 12: ftCo_800BE7C0(&GO);
                ftCo_ThrownKirbyStar_Anim(&GO); ftCo_ThrownKirbyStar_IASA(&GO); ftCo_ThrownCopyStar_Anim(&GO);
                ftCo_ThrownCopyStar_IASA(&GO); ftCo_ThrownKirby_Anim(&GO); ftCo_ThrownKirby_IASA(&GO);
                ftCo_ThrownKirby_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
