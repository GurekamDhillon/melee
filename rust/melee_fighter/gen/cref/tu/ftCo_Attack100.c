/* TU: src/melee/ft/kinds/ftCommon/ftCo_Attack100.c */
/* C oracle for src/tu/ftCo_Attack100.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

HSD_GObjEvent ftData_SpecialN[Ft_Kind_Max], ftData_SpecialLw[Ft_Kind_Max], ftData_SpecialHi[Ft_Kind_Max],
    ftData_SpecialAirHi[Ft_Kind_Max];
static void vanilla_fn(HSD_GObj* g) { T(99); }
static void cb_fn(HSD_GObj* g) { T(98); T(g == &GO); }
int ScriptGame_CapsActionAllowed(Fighter* f, int fl) { T(1); T(f == &F); T(fl); return chk(1) | chk(7); }
int Geno_SpecialEnter(Fighter_GObj* g, int w) { T(2); T(g == &GO); T(w); return chk(2); }
#define MEX(name, id) void name(int k, void* g, void* v) { T(id); T(k); T(g == &GO); T(v == NULL ? 0 : v == (void*) vanilla_fn ? 1 : 9); }
MEX(Mex_SpecialNDispatch, 3)
MEX(Mex_SpecialLwDispatch, 4)
MEX(Mex_SpecialHiDispatch, 5)
MEX(Mex_SpecialHiAirDispatch, 6)
STUB_V(ftGw_Attack100Start_Enter, 8)
STUB_CHK(ftpickupitem_80094790, 9)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(10); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 11)
STUB_CHK(ftAnim_IsFramesRemaining, 12)
STUB_V(ft_800892A0, 13)
STUB_V(ft_80089824, 14)
STUB_V(ft_8008A2BC, 15)
STUB_V(ft_80084FA8, 16)
STUB_V(ft_80084104, 17)
float Geno_ItemPressDist(HSD_GObj* i, Fighter* f) { T(18); T(af_item(i)); T(f == &F); return SR(5) ? (SR(21) ? 0.0f : 3.5f) : -1.0f; }
void Geno_ItemPressCollect(HSD_GObj* i, int p) { T(19); T(af_item(i)); T(p); }
void ftpickupitem_80094818(HSD_GObj* g, bool b) { T(20); T(g == &GO); T(b); }
ItemKind itGetKind(Item_GObj* i) { T(21); T(af_item(i)); return SR(6) ? It_Kind_WStar : (ItemKind) (SRV[7] % 50); }
void pl_8003E854(int a, int b, Item_GObj* i) { T(22); T(a); T(b); T(af_item(i)); }
void Item_8026AB54(HSD_GObj* i, HSD_GObj* o, Fighter_Part p) { T(23); T(af_item(i)); T(o == &GO); T(p); }
bool ftpickupitem_8009447C(HSD_GObj* g, Item_GObj* i) { T(24); T(g == &GO); T(af_item(i)); return chk(24); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 k, u32 id, HSD_JObj* j, ...) { T(25); T(g == &GO); T(q == (void*) &F.x60C); T(k); T(id); T(af_jobj(j)); }
Item_GObj* ftpickupitem_800942A0(Fighter_GObj* g, u32 fl) { T(26); T(g == &GO); T(fl); return SR(8) ? NULL : SR(24) ? (Item_GObj*) &IGO : (Item_GObj*) &IGO2; }
int ftKb_MexKirbyLike(int k) { T(27); T(k); return chk(27); }
int ftKb_SpecialN_800F1CD8(HSD_GObj* g) { T(28); T(g == &GO); return SRV[9] % 7; }

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 26; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int ms;
            tu_case(op, k);
            af_fill();
            ftData_SpecialN[F.kind] = (SRV[1] & 1) ? vanilla_fn : NULL;
            ftData_SpecialLw[F.kind] = (SRV[1] & 2) ? vanilla_fn : NULL;
            ftData_SpecialHi[F.kind] = (SRV[1] & 4) ? vanilla_fn : NULL;
            ftData_SpecialAirHi[F.kind] = (SRV[1] & 8) ? vanilla_fn : NULL;
            ms = rnd() % 400;
            switch (op) {
            case 0: ret = ftCo_800D6824(&GO); break;
            case 1: ret = ftCo_800D688C(&F); break;
            case 2: ret = ftCo_800D68C0(&GO); break;
            case 3: ret = ftCo_800D6928(&F); break;
            case 4: ret = ftCo_Attack100_CheckInput(&GO); break;
            case 5: ret = ftCo_800D69C4(&GO); break;
            case 6: ret = ftCo_Attack_800D6A50(&GO); break;
            case 7: fn_800D6AC4(&GO); break;
            case 8: ftCo_800D6B00(&GO, ms); break;
            case 9: fn_800D6B8C(&GO); break;
            case 10: ftCo_Attack100Start_Anim(&GO); break;
            case 11: ftCo_800D6C60(&GO, cb_fn); break;
            case 12: ftCo_Attack100Loop_Anim(&GO); break;
            case 13: ftCo_Attack100End_Anim(&GO); break;
            case 14: ftCo_Attack100Loop_IASA(&GO); break;
            case 15: ftCo_Attack100Start_Phys(&GO); break;
            case 16: ftCo_Attack100Loop_Phys(&GO); break;
            case 17: ftCo_Attack100End_Phys(&GO); break;
            case 18: ftCo_Attack100Start_Coll(&GO); break;
            case 19: ftCo_Attack100Loop_Coll(&GO); break;
            case 20: ftCo_Attack100End_Coll(&GO); break;
            case 21: fn_800D6F58(&GO, (ms & 1) ? (Item_GObj*) &IGO : (Item_GObj*) &IGO2); break;
            case 22: ret = ftCo_800D705C(&GO); break;
            case 23: ret = ftCo_800D7100(&GO); break;
            case 24: ftCo_800D71D8(&GO); break;
            case 25: ret = ftCo_800D7268(&F); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
