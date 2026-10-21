/* TU: src/melee/ft/kinds/ftCommon/ftCo_BarrelWait.c */
/* C oracle for src/tu/ftCo_BarrelWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static CmSubject CAM;
static u8 ARG2[64];
#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
VSTUB(ftCommon_8007DB58, 1)
VSTUB(ftCo_8009750C, 2)
VSTUB(ftCo_800DD168, 3)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
VSTUB(ftAnim_8006EBA4, 5)
void ftCo_8009EAF8(HSD_GObj* g) {}
VSTUB(ftCommon_8007E2FC, 6)
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(7); T(f == &F); T(v); }
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(8); T(f == &F); T(v); }
bool Ground_801C4DD0(void) { T(9); return SR(9); }
bool Ground_801C4DA0(Vec3* p, f32* o) { T(10); T(p == &F.cur_pos); *o = rg(); return SR(10); }
void mpColl_80043680(CollData* c, Vec3* v) { T(11); T(c == &F.coll_data); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void ftCamera_80076064(Fighter* f) { T(12); T(f == &F); }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a)
{ T(13); T(a == (void*) ARG2); h->unk_count = rnd() % 50; h->element = rnd() % 12; h->damage = rw(); }
VSTUB(ftColl_800788D4, 14)
void Fighter_UnkTakeDamage_8006CC30(Fighter* f, float d) { T(15); T(f == &F); T(B(d)); }
void ftCo_8008DCE0(Fighter_GObj* g, int a, float d) { T(16); T(g == &GO); T(a); T(B(d)); }
float ftColl_80079EA8(Fighter* f, HitCapsule* h, u32 n) { T(17); T(f == &F); T(n); return rw(); }

static u32 cbid(HSD_GObjEvent e)
{
    if (e == NULL) return 0;
    if (e == ftCo_8009EAF8) return 1;
    if (e == ftCo_8009EC44) return 2;
    return 99;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(cbid(F.death2_cb)); HM(cbid(F.accessory1_cb));
    HM(F.invisible); HM(F.x1988); HM(F.bury_timer_2);
    HM(B(CAM.target_ext.h.x)); HM(B(CAM.target_ext.h.y));
    HM(B(CAM.target_ext.v.x)); HM(B(CAM.target_ext.v.y)); HM(B(CAM.target_ext.v.z));
    HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z));
    HM(B(F.dmg.kb_applied)); HM(F.dmg.x1848_kb_angle); HM(B(F.dmg.facing_dir_1));
    HM(F.dmg.x184c_damaged_hurtbox); HM(B(F.dmg.x1854_collpos.x)); HM(B(F.dmg.x1854_collpos.y));
    HM(B(F.dmg.x1854_collpos.z)); HM(F.dmg.x1860_element);
    return H;
}

#define NOPS 7
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            Vec3 pos;
            float ang;
            tu_case(op, k);
            zero(&CAM, sizeof CAM);
            F.x890_cameraBox = &CAM;
            CD.x4DC.x = rg(); CD.x4DC.y = rw(); CD.x4E4.x = rg(); CD.x4E4.y = rw(); CD.x4E4.z = rg();
            CD.bury_timer_unk2 = rnd();
            F.facing_dir = rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rg(); F.cur_pos.z = rg();
            F.input.pressed_buttons = rnd() & 0x3FF;
            F.dmg.x184c_damaged_hurtbox = rnd();
            pos.x = rw(); pos.y = rg(); pos.z = rw();
            ang = rg() * 400.0f;
            switch (op) {
            case 0: ftCo_8009EB18(&GO); break;
            case 1: ftCo_BarrelWait_Anim(&GO); break;
            case 2: ftCo_BarrelWait_IASA(&GO); break;
            case 3: ftCo_BarrelWait_Phys(&GO); break;
            case 4: ftCo_BarrelWait_Coll(&GO); break;
            case 5: ftCo_8009EC44(&GO); break;
            case 6: ftCo_8009EC70(&GO, &pos, (UNK_T) ARG2, ang); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
