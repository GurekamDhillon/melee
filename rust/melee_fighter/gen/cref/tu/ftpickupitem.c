/* TU: src/melee/ft/kinds/ftCommon/ftpickupitem.c */
/* C oracle for src/tu/ftpickupitem.rs: the whole TU included verbatim (TARGET_PC); every callee
 * out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static HSD_GObj IG[3];
static HSD_GObj* PL[16];
HSD_GObj** HSD_GObjPLinkHead = PL;
struct Fighter_804D6524_t* Fighter_804D6524;
static struct Fighter_804D6524_t D6524;
Fighter_ItemEvent ftData_OnItemPickupExt[Ft_Kind_Max];
static ftData FD;
static ftData_x8 FX8;
static FtSFX SFX;
static const int KINDS[13] = { It_Kind_Heart, It_Kind_Tomato, It_Kind_Foods, It_Kind_Lucky_Egg,
                               It_Kind_WhispyHealApple, It_Kind_WStar, It_Kind_Hammer, It_Kind_RabbitC,
                               It_Kind_MetalB, It_Kind_Spycloak, It_Kind_Coin, 0, 50 };
static u32 iid(HSD_GObj* g)
{
    int i;
    for (i = 0; i < 3; i++) if (g == &IG[i]) return 1 + i;
    if (g == &GO) return 10;
    return g == NULL ? 0 : 99;
}

bool Item_IsGrabbable(Item_GObj* i) { T(1); T(iid(i)); return (rnd() % 4) != 0; }
s32 it_8026B47C(Item_GObj* i) { T(2); T(iid(i)); return (s32) (rnd() % 3); }
bool it_8026B4F0(Item_GObj* i) { T(3); T(iid(i)); return rnd() & 1; }
bool itIsHeavy(Item_GObj* i) { T(4); T(iid(i)); return (rnd() % 3) == 0; }
void it_8026B344(Item_GObj* i, Vec3* p) { T(5); T(iid(i)); p->x = rg() * 10.0f / 3.0f; p->y = rg() * 10.0f / 7.0f; p->z = rg(); }
float itGetGrabRangeX(Item_GObj* i) { T(6); T(iid(i)); return rg() * 5.0f; }
float itGetGrabRangeY(Item_GObj* i) { T(7); T(iid(i)); return rg() * 5.0f; }
bool ftCo_IsCpuControlled(Fighter* f) { T(8); T(f == &F); return rnd() & 1; }
float Geno_ItemPressDist(HSD_GObj* i, Fighter* f)
{
    T(9); T(iid(i)); T(f == &F);
    switch (rnd() % 4) {
    case 0: return rg() * 100.0f;
    case 1: return rg() * 100000.0f;
    default: return -1.0f;
    }
}
enum_t it_8026B30C(Item_GObj* i) { T(10); T(iid(i)); return (rnd() % 3) ? 5 : (enum_t) (rnd() % 7); }
void Geno_ItemFamilyEvent(HSD_GObj* i, int a, int p) { T(11); T(iid(i)); T(a); T(p); }
ItemKind itGetKind(Item_GObj* i) { T(12); T(iid(i)); return KINDS[rnd() % 13]; }
void Fighter_8006CF5C(Fighter* f, s32 v) { T(13); T(f == &F); T(v); }
void Item_8026A8EC(Item_GObj* i) { T(14); T(iid(i)); }
void ftCo_800C4724(Fighter_GObj* g) { T(15); T(iid(g)); }
void ftCo_800C52F4(Fighter_GObj* g) { T(16); T(iid(g)); }
void ftCommon_8007FA58(Fighter_GObj* g, Item_GObj* i) { T(17); T(iid(g)); T(iid(i)); }
void ftLib_800871A8(Fighter_GObj* g, Item_GObj* i) { T(18); T(iid(g)); T(iid(i)); }
void ftCo_800C88D4(Fighter_GObj* g, int v, bool b) { T(19); T(iid(g)); T(v); T(b); }
float ftCo_CalcYScaledKnockback(float a, float b, float c) { T(20); T(B(a)); T(B(b)); T(B(c)); return rw(); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(21); T(iid(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(22); T(iid(g)); if (rnd() % 3 == 0) F.throw_flags_b3 = 1; }
void ftCo_800974C4(Fighter_GObj* g) { T(23); }
void pl_8003E17C(int s, int sub, Item_GObj* i) { T(24); T(s); T(sub); T(iid(i)); }
static void pickup_ext(HSD_GObj* g, bool a) { T(25); T(iid(g)); T(a); }
void Mex_OnItemPickupDispatch(int kind, void* g, void* i, void* v) { T(26); T(kind); T(iid(g)); T(iid(i)); T(v == NULL); }
void pl_8003E854(int s, int sub, Item_GObj* i) { T(27); T(s); T(sub); T(iid(i)); }
void Item_8026AB54(HSD_GObj* i, HSD_GObj* o, Fighter_Part p) { T(28); T(iid(i)); T(iid(o)); T(p); }
void ft_800881D8(Fighter* f, int id, u8 v, u8 p) { T(29); T(f == &F); T(id); T(v); T(p); }
void Geno_ItemPressCollect(HSD_GObj* i, int p) { T(30); T(iid(i)); T(p); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(31); T(iid(g)); return SR(31); }
void ftDk_MS_341_800DF980(HSD_GObj* g) { T(32); T(iid(g)); }
void ftCo_80096D9C(Fighter_GObj* g) { T(33); T(iid(g)); }
void ft_8008A2BC(Fighter_GObj* g) { T(34); T(iid(g)); }
void ft_80084F3C(Fighter_GObj* g) { T(35); T(iid(g)); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(36); T(iid(g)); T(cb == ftpickupitem_80094D90); }
void Item_8026ABD8(Item_GObj* i, Vec3* p, float f) { T(37); T(iid(i)); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(f)); }
void ftCo_800C5284(Fighter_GObj* g) { T(38); T(iid(g)); }
void ftCo_8009750C(Fighter_GObj* g) { T(39); T(iid(g)); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(40); T(iid(g)); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(iid(F.item_gobj)); HM(iid(F.x1978));
    HM(F.throw_flags_b0); HM(F.throw_flags_b1); HM(F.throw_flags_b2); HM(F.throw_flags_b3);
    HM(F.throw_flags_b4); HM(F.throw_flags_b5); HM(F.throw_flags_b6); HM(F.throw_flags_b7);
    HM(F.mv.co.itemget.x0);
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == ftCo_800974C4 ? 1 : F.take_dmg_cb == ftpickupitem_80094DF8 ? 2 : 3);
    return H;
}

static float r10(void) { return rg() * 10.0f / 3.0f; }
static void q(Quaternion* v) { v->x = r10(); v->y = r10(); v->z = r10(); v->w = r10(); }

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(IG, sizeof IG); zero(PL, sizeof PL); zero(&D6524, sizeof D6524);
            zero(ftData_OnItemPickupExt, sizeof ftData_OnItemPickupExt);
            zero(&FD, sizeof FD); zero(&FX8, sizeof FX8); zero(&SFX, sizeof SFX);
            Fighter_804D6524 = &D6524;
            FD.x8 = &FX8; FD.x4C_sfx = &SFX;
            F.ft_data = &FD;
            ftData_OnItemPickupExt[0] = pickup_ext;
            ftData_OnItemPickupExt[2] = pickup_ext;
            {
                int n = rnd() % 4;
                for (i = 0; i < n; i++) IG[i].next = (i + 1 < n) ? &IG[i + 1] : NULL;
                PL[HSD_GOBJ_PLINK_ITEM] = n ? &IG[0] : NULL;
            }
            D6524.x8 = rw();
            FX8.x10 = rnd() % 8; FX8.x11 = rnd() % 8;
            SFX.x2C = rnd() % 100000;
            CD.x7CC = rnd() % 100;
            F.kind = rnd() % 4;
            F.player_id = rnd() % 4;
            F.is_sub_fighter = rnd() & 1;
            F.item_gobj = (rnd() & 1) ? &IG[1] : NULL;
            F.x1978 = (rnd() & 1) ? &IG[2] : NULL;
            F.ground_or_air = rnd() & 1;
            F.facing_dir = rg();
            F.cur_pos.x = r10(); F.cur_pos.y = r10();
            q(&F.x294_itPickup.gr_light_offset); q(&F.x294_itPickup.gr_heavy_offset); q(&F.x294_itPickup.air_light_offset);
            F.x2222_b4 = rnd() & 1; F.x2222_b0 = rnd() & 1;
            F.cpu.kind = (rnd() & 1) ? 28 : 3;
            F.motion_id = (rnd() & 1) ? ftCo_MS_LightGet : ftCo_MS_HeavyGet;
            F.x34_scale.y = rw();
            F.mv.co.itemget.x0 = rnd() & 1;
            F.throw_flags = rnd() & 0xFF;
            switch (op) {
            case 0: ret = ftpickupitem_80094150(&GO, &IG[0]); break;
            case 1: ret = iid(ftpickupitem_800942A0(&GO, rnd() % 4)); break;
            case 2: ret = ftpickupitem_8009447C(&GO, &IG[0]); break;
            case 3: { FtMotionId m = (rnd() & 1) ? ftCo_MS_LightGet : ftCo_MS_HeavyGet; ftpickupitem_80094694(&GO, m, rnd() & 1); break; }
            case 4: ret = ftpickupitem_80094790(&GO); break;
            case 5: ftpickupitem_80094818(&GO, rnd() & 1); break;
            case 6: ftpickupitem_800948A8(&GO, &IG[0]); break;
            case 7: ftpickupitem_Anim(&GO); break;
            case 8: ftpickupitem_IASA(&GO); break;
            case 9: ftpickupitem_Phys(&GO); break;
            case 10: ftpickupitem_Coll(&GO); break;
            case 11: ftpickupitem_80094B6C(&GO, &IG[0]); break;
            case 12: ftpickupitem_80094D90(&GO); break;
            case 13: ftpickupitem_80094DF8(&GO); break;
            case 14: geno_consumable_collect(&IG[0], rnd() % 4); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
