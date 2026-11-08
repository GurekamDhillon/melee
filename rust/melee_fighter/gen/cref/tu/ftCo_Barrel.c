/* TU: src/melee/ft/kinds/ftCommon/ftCo_Barrel.c */
/* C oracle for src/tu/ftCo_Barrel.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static HSD_JObj JO;
static u8 ARG3[64];
#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
static u32 itid(void* p) { return p == NULL ? 0 : p == (void*) &IGO ? 1 : 99; }
VSTUB(ftCommon_8007DB58, 1)
VSTUB(ftCo_8009750C, 2)
VSTUB(ftCo_800DD168, 3)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
VSTUB(ftAnim_8006EBA4, 5)
int it_802960B8(Item_GObj* i) { T(6); T(itid(i)); return (int) (rnd() % 4); }
int it_802960CC(Item_GObj* i) { T(7); T(itid(i)); return (int) (rnd() % 4); }
void it_80295F38(Item_GObj* i) { T(8); T(itid(i)); }
void it_8026C334(Item_GObj* i, Vec3* p) { T(9); T(itid(i)); T(p == &F.cur_pos); }
HSD_GObj* itGetOwner(Item_GObj* i) { T(10); T(itid(i)); return SR(10) ? &GO2 : NULL; }
void ftCommon_8007D5D4(Fighter* f) { T(11); T(f == &F); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(12); T(g == &GO); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(13); T(f == &F); T(v); }
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(14); T(f == &F); T(v); }
void ftColl_800787B4(Item_GObj* i, Fighter_GObj* g, int v) { T(15); T(itid(i)); T(g == &GO); T(v); }
bool ftLib_IsFighter(HSD_GObj* g) { T(16); T(g == &GO2); return SR(16); }
void ftColl_8007B8CC(Fighter* f, Fighter_GObj* o) { T(17); T(f == &F); T(o == &GO2); }
void ftLib_GetCameraBonePos(HSD_GObj* g, Vec3* v) { T(18); T(g == &GO); v->x = rg(); v->y = rw(); v->z = rg(); }
void mpColl_80043680(CollData* c, Vec3* v) { T(19); T(c == &F.coll_data); T(v == NULL); }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a)
{ T(20); T(a == (void*) ARG3); h->unk_count = rnd() % 50; h->element = rnd() % 12; }
float ftColl_80079EA8(Fighter* f, HitCapsule* h, u32 n) { T(21); T(f == &F); T(n); return rw(); }
void ftCo_8008DCE0(Fighter_GObj* g, int a, float d) { T(22); T(g == &GO); T(a); T(B(d)); }

static u32 cbid(HSD_GObjEvent e)
{
    if (e == NULL) return 0;
    if (e == ftCo_Barrel_Death2_Cb) return 1;
    if (e == ftCo_Barrel_Accessory1_Cb) return 2;
    if (e == ftCo_Barrel_Take_Dmg_Cb) return 3;
    if (e == fn_800C9290) return 4;
    return 99;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.mv.co.barrel.x0); HM(F.mv.co.barrel.x4); HM(itid(F.mv.co.barrel.x8));
    HM(cbid(F.death2_cb)); HM(cbid(F.accessory1_cb)); HM(cbid(F.take_dmg_cb)); HM(cbid(F.x21EC));
    HM(F.x221D_b5); HM(F.invisible); HM(F.x1988);
    HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z));
    HM(B(F.dmg.kb_applied)); HM(F.dmg.x1848_kb_angle); HM(B(F.dmg.facing_dir_1));
    HM(F.dmg.x184c_damaged_hurtbox); HM(B(F.dmg.x1854_collpos.x)); HM(B(F.dmg.x1854_collpos.y));
    HM(B(F.dmg.x1854_collpos.z)); HM(F.dmg.x1860_element);
    return H;
}

#define NOPS 10
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    GO.hsd_obj = &JO;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            Vec3 a1;
            float ang;
            tu_case(op, k);
            JO.translate.x = rw(); JO.translate.y = rg(); JO.translate.z = rg();
            F.facing_dir = rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rg(); F.cur_pos.z = rg();
            F.mv.co.barrel.x0 = (int) (rnd() % 3);
            F.mv.co.barrel.x4 = (int) (rnd() % 3);
            F.mv.co.barrel.x8 = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.input.pressed_buttons = rnd() & 0x3FF;
            F.dmg.x184c_damaged_hurtbox = rnd();
            a1.x = rw(); a1.y = rg(); a1.z = rg();
            ang = rg() * 400.0f;
            switch (op) {
            case 0: ftCo_Barrel_Death2_Cb(&GO); break;
            case 1: ftCo_Barrel_Enter(&GO, (Item_GObj*) &IGO); break;
            case 2: ftCo_Barrel_Take_Dmg_Cb(&GO); break;
            case 3: ftCo_Barrel_Anim(&GO); break;
            case 4: ftCo_Barrel_IASA(&GO); break;
            case 5: ftCo_Barrel_Phys(&GO); break;
            case 6: ftCo_Barrel_Coll(&GO); break;
            case 7: ftCo_Barrel_Accessory1_Cb(&GO); break;
            case 8: fn_800C9290(&GO); break;
            case 9: ftCo_800C92E4(&GO, &a1, NULL, (lbColl_80008D30_arg1*) ARG3, ang); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
