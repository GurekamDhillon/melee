/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerWait.c */
/* C oracle for src/tu/ftCo_HammerWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static u32 itid(void* p) { return p == NULL ? 0 : p == (void*) &IGO ? 1 : 99; }
static u32 isf(Fighter* f) { return f == &F; }
bool ft_8008A1FC(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(3); T(isf(f)); T(a); T(b); }
void ftColl_8007AFF8(Fighter_GObj* g) { T(4); T(g == &GO); }
void ft_800892A0(Fighter_GObj* g) { T(5); T(g == &GO); }
void ft_80089824(Fighter_GObj* g) { T(6); T(g == &GO); }
void it_8027B070(Item_GObj* i, Fighter_GObj* g) { T(7); T(itid(i)); T(g == &GO); }
bool ftCo_800C5A50(Fighter_GObj* g) { T(8); return SR(2); }
bool ftCo_800C60C8(Fighter_GObj* g) { T(9); return SR(3); }
bool ftCo_800C576C(Fighter_GObj* g) { T(10); return SR(4); }
bool ftCo_800C55CC(Fighter_GObj* g) { T(11); return SR(5); }
void ft_80084F3C(Fighter_GObj* g) { T(12); T(g == &GO); }
void ftCo_800C5D34(Fighter_GObj* g) { T(13); T(g == &GO); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(14); T(g == &GO); T(cb == ftCo_800C5D34); cb(g); }
void it_80293E34(Item_GObj* i) { T(15); T(itid(i)); }
void ftCommon_8007ECD4(Fighter* f, s32 a) { T(16); T(isf(f)); T(a); }
void Item_8026A8EC(Item_GObj* i) { T(17); T(itid(i)); }
void ft_80088110(Fighter* f) { T(18); T(isf(f)); }
enum_t ftCo_800C0694(Fighter* f) { T(19); T(isf(f)); return SR(6) ? 106 : (enum_t) (rnd() % 200); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(20); T(g == &GO); }
void ftCo_800C0200(Fighter* f, int a) { T(21); T(isf(f)); T(a); }
ItemKind itGetKind(Item_GObj* i) { T(22); T(itid(i)); return SR(7) ? It_Kind_Hammer : (ItemKind) (rnd() % 40); }
int it_80293DCC(Item_GObj* i) { T(23); T(itid(i)); return (int) (rnd() % 3); }
void pl_800403C0(int a, int b) { T(24); T(a); T(b); }
void ft_800880AC(Fighter* f) { T(25); T(isf(f)); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(26); T(isf(f)); T(a); T(b); return SR(8); }
void it_8026C16C(Item_GObj* i, bool h) { T(27); T(itid(i)); T(h); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.motion_id); HM(F.x2330.x); HM(F.x2330.y); HM(F.x2338.x); HM(F.x2338.y); HM(F.x2219_b4);
    return H;
}

#define NOPS 22
#define NCASE 1000

int main(void)
{
    int op, k;
    static const int ms[6] = { 306, 307, 310, 313, 314, ftCo_MS_LightGet };
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.motion_id = ms[rnd() % 6];
            F.x2330.x = rnd() % 3; F.x2330.y = rnd() % 3;
            F.x2338.x = rnd() % 2; F.x2338.y = rnd() % 3;
            F.cur_anim_frame = rg(); F.frame_speed_mul = rg();
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.ground_or_air = rnd() & 1;
            F.player_id = rnd() % 6; F.is_sub_fighter = rnd() & 1;
            F.x2219_b4 = rnd() & 1;
            CD.x6AC = (int) (rnd() % 1000) - 5; CD.x6B8 = (int) (rnd() % 1000) - 5;
            switch (op) {
            case 0: ret = ftCo_800C4E4C(&GO); break;
            case 1: ftCo_800C4E94(&F); break;
            case 2: ftCo_800C4ED8(&GO); break;
            case 3: ftCo_800C4F64(&GO); break;
            case 4: ftCo_HammerWait_Anim(&GO); break;
            case 5: ftCo_HammerWait_IASA(&GO); break;
            case 6: ftCo_HammerWait_Phys(&GO); break;
            case 7: ftCo_HammerWait_Coll(&GO); break;
            case 8: ftCo_800C511C(&GO); break;
            case 9: ret = ftCo_800C5240(&GO); break;
            case 10: ftCo_800C5284(&GO); break;
            case 11: ftCo_800C52F4(&GO); break;
            case 12: ret = ftCo_800C53E4(&F); break;
            case 13: ftCo_800C5408(&F); break;
            case 14: ftCo_800C544C(&F); break;
            case 15: ret = B(ftCo_800C548C(&F)); break;
            case 16: ret = ftCo_800C54C4(&F); break;
            case 17: ftCo_800C5500(&GO); break;
            case 18: ftCo_800C554C(&F); break;
            case 19: ftCo_800C555C(&GO); break;
            case 20: ftCo_800C5588(&F); break;
            case 21: ftCo_800C511C(&GO); ftCo_800C511C(&GO); ftCo_800C511C(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
