/* TU: src/melee/ft/ft_0C31.c */
/* C oracle for src/tu/ft_0c31.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (player.c accessors read the same globals the Rust test mirrors into
 * PLAYER_SLOTS). Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

extern int fflush(void*);
extern char* getenv(const char*);
extern int atoi(const char*);
static int DBGOP = -1, DBGK = -1;
static HSD_JObj J, A;
static u32 EP(const void* p)
{
    unsigned long long a = (unsigned long long) p;
    if (p == 0) return 0;
    if (a >= (unsigned long long) &F && a < (unsigned long long) &F + sizeof F) return 0x1000u + (u32) (a - (unsigned long long) &F);
    if (a >= (unsigned long long) &F2 && a < (unsigned long long) &F2 + sizeof F2) return 0x2000u + (u32) (a - (unsigned long long) &F2);
    if (p == &J) return 0x3000;
    if (p == &A) return 0x4000;
    if (p == &GO) return 0x5000;
    if (p == &GO2) return 0x6000;
    return 0xFFFF;
}
static int g_u4c, g_bit4;
static HSD_GObj* g_ent[2];
static HSD_Joint JOINT;
HSD_Joint* Fighter_804D6514 = &JOINT;
static u32 CB1, CB2;

int Player_GetUnk4C(int slot) { (void) slot; return g_u4c; }
HSD_GObj* Player_GetEntityAtIndex(int slot, int i) { (void) slot; return g_ent[i]; }
int Player_GetFlagsBit4(int slot) { (void) slot; return g_bit4; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(EP(g)); T((u32) m); T((u32) fl); T(B(a)); T(B(b)); T(B(c)); T(EP(o)); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(5); T(EP(j)); }
void ftCommon_SetAccessory(Fighter* fp, HSD_Joint* j) { T(6); T(EP(fp)); T(j == &JOINT); fp->x20A0_accessory = &A; }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 id, HSD_JObj* j, ...)
{
    __builtin_va_list ap; void* v;
    __builtin_va_start(ap, j); v = __builtin_va_arg(ap, void*); __builtin_va_end(ap);
    T(7); T(EP(g)); T(EP(q)); T(kind); T(id); T(EP(j)); T(EP(v));
}
bool lbAudioAx_80024304(int id) { T(8); T((u32) id); return 0; }
static u32 cbid(void (*cb)(Fighter_GObj*)) { return cb == fn_800C63BC ? 1 : cb == fn_800C63E0 ? 2 : 3; }
void ft_80083E64(Fighter_GObj* g, ftCollisionBox* b, void (*cb)(Fighter_GObj*))
{ T(9); T(EP(g)); T(EP(b)); T(B(b->top)); T(B(b->bottom)); T(B(b->right.x)); T(B(b->right.y)); T(B(b->left.x)); T(B(b->left.y)); T(cbid(cb)); }
void ft_800846B0(Fighter_GObj* g, ftCollisionBox* b, void (*cb)(Fighter_GObj*))
{ T(10); T(EP(g)); T(EP(b)); T(B(b->top)); T(B(b->bottom)); T(B(b->right.x)); T(B(b->right.y)); T(B(b->left.x)); T(B(b->left.y)); T(cbid(cb)); }
void ftColl_8007B760(Fighter_GObj* g, int n) { T(11); T(EP(g)); T((u32) n); }
bool ftCo_800BFFD0(Fighter* fp, enum_t k, bool b) { T(12); T(EP(fp)); T((u32) k); T((u32) b); return 0; }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(13); T(EP(g)); }
void ft_8008A2BC(Fighter_GObj* g) { T(14); T(EP(g)); }
void ft_80084CB0(Fighter* fp, ftCollisionBox* box)
{
    box->top = fp->coll_data.ecb.top.y;
    box->bottom = fp->coll_data.ecb.bottom.y;
    box->right.x = fp->coll_data.ecb.right.x;
    box->right.y = fp->coll_data.ecb.right.y;
    box->left.x = fp->coll_data.ecb.left.x;
    box->left.y = fp->coll_data.ecb.left.y;
}
void ftCommon_8007D92C(HSD_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (fp->ground_or_air == GA_Air) {
        ftCo_Fall_Enter(gobj);
    } else {
        ft_8008A2BC(gobj);
    }
}
float ftCommon_800804EC(Fighter* fp) { return fp->x40 * fp->x34_scale.x; }
void ftCommon_8007D7FC(Fighter* fp) { T(15); T(EP(fp)); }
void ftCommon_8007D5D4(Fighter* fp) { T(16); T(EP(fp)); }

static void HX(u32 v) { if (dbgT) { printf("H %u", v); printf("%c", 10); } HM(v); }
static void hv(Vec3* v) { HX(B(v->x)); HX(B(v->y)); HX(B(v->z)); }
static void hj(HSD_JObj* j) { hv(&j->scale); hv(&j->translate); HX(B(j->rotate.x)); HX(B(j->rotate.y)); HX(B(j->rotate.z)); HX(B(j->rotate.w)); HX(j->flags); }

int main(void)
{
    int op, k;
    static const u32 FL[6] = { 0, 1u << 6, 1u << 25, (1u << 6) | (1u << 25), (1u << 23) | (1u << 6), 1u << 23 };
    tu_init();
    { char* e = getenv("TUDBG"); if (e) { DBGOP = atoi(e); while (*e && *e != ':') e++; if (*e) DBGK = atoi(e + 1); } }
    for (op = 0; op < 16; op++) {
        for (k = 0; k < 1500; k++) {
            tu_case(op, k); dbgT = (op == DBGOP && k == DBGK);
            zero(&J, sizeof J); zero(&A, sizeof A); H = 0x811C9DC5u;
            J.flags = FL[rnd() % 6]; A.flags = FL[rnd() % 6];
            J.scale.x = rw(); J.scale.y = rw(); J.scale.z = rw();
            A.translate.x = rw(); A.rotate.y = rw();
            CD.x6BC = (s32) (rnd() % 9) - 1; CD.x6C0 = (s32) (rnd() % 9); CD.x6C4 = rw(); CD.x6C8 = (s32) (rnd() % 50);
            g_u4c = (int) (rnd() % 5); g_bit4 = (int) (rnd() & 1);
            F.player_id = (u8) (rnd() % 3);
            F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rw();
            F.facing_dir = rg(); F.x34_scale.y = rg(); F.x34_scale.x = rg(); F.x40 = rw();
            F.co_attrs.trophy_scale = rw();
            F.ground_or_air = (GroundOrAir) (rnd() & 1);
            F.is_sub_fighter = rnd() & 1;
            F.x20A0_accessory = (rnd() % 5) ? &A : NULL;
            if (!F.x20A0_accessory) F.x20A0_accessory = &A;
            F.coll_data.ecb.top.y = rw(); F.coll_data.ecb.bottom.y = rw();
            F.coll_data.ecb.right.x = rw(); F.coll_data.ecb.right.y = rw();
            F.coll_data.ecb.left.x = rw(); F.coll_data.ecb.left.y = rw();
            F.mv.co.entry.timer = (s32) (rnd() % 7) - 1;
            F.mv.co.entry.x4 = rw(); F.mv.co.entry.x8.x = rw(); F.mv.co.entry.x8.y = rw(); F.mv.co.entry.x8.z = rw();
            F.mv.co.entry.x14.x = rw(); F.mv.co.entry.x14.y = rw(); F.mv.co.entry.x14.z = rw();
            F.mv.co.entry.x20 = rw(); F.mv.co.entry.x24 = rw(); F.mv.co.entry.x28 = rw();
            F.mv.co.entry.x2C.top = rw(); F.mv.co.entry.x2C.bottom = rw();
            F.mv.co.entry.x2C.right.x = rw(); F.mv.co.entry.x2C.right.y = rw();
            F.mv.co.entry.x2C.left.x = rw(); F.mv.co.entry.x2C.left.y = rw();
            F2.cur_pos.y = rw(); F2.mv.co.entry.x28 = rw(); F2.x221F_b3 = rnd() & 1;
            g_ent[0] = &GO2; g_ent[1] = (rnd() & 1) ? &GO2 : NULL;
            GO.hsd_obj = &J;
            {
                u32 ret = 0;
                switch (op) {
                case 0: ftCo_800C61B0(&GO); break;
                case 1: ftCo_Entry_Anim(&GO); break;
                case 2: ftCo_800C6408(&GO); break;
                case 3: ftCo_EntryStart_Anim(&GO); break;
                case 4: ftCo_EntryStart_Phys(&GO); break;
                case 5: ftCo_EntryStart_Coll(&GO); break;
                case 6: fn_800C69F4(&GO); break;
                case 7: { HSD_JObj* jj = (rnd() & 3) ? &J : NULL; (HSD_JObjSetMtxDirty)(jj); } break;
                case 8: ftCo_800C6B6C(&GO); break;
                case 9: ftCo_EntryEnd_Anim(&GO); break;
                case 10: ftCo_EntryEnd_Phys(&GO); break;
                case 11: ftCo_EntryEnd_Coll(&GO); break;
                case 12: fn_800C6F34(&GO); break;
                case 13: ftCo_Entry_IASA(&GO); ftCo_Entry_Phys(&GO); ftCo_Entry_Coll(&GO); break;
                case 14: ftCo_EntryStart_IASA(&GO); break;
                case 15: ftCo_EntryEnd_IASA(&GO); break;
                }
                hv(&F.cur_pos); hv(&F2.cur_pos);
                HX(F.invisible); HX(F.x2219_b1); HX(F.x221E_b1); HX(F.x221E_b2); HX(F.x221F_b1);
                HX(F.accessory1_cb == fn_800C69F4 ? 1 : F.accessory1_cb == fn_800C6F34 ? 2 : F.accessory1_cb ? 3 : 0);
                HX((u32) F.mv.co.entry.timer); HX(B(F.mv.co.entry.x4)); hv(&F.mv.co.entry.x8); hv(&F.mv.co.entry.x14);
                HX(B(F.mv.co.entry.x20)); HX(B(F.mv.co.entry.x24)); HX(B(F.mv.co.entry.x28));
                HX(B(F.mv.co.entry.x2C.top)); HX(B(F.mv.co.entry.x2C.bottom)); HX(B(F.mv.co.entry.x2C.right.x));
                HX(B(F.mv.co.entry.x2C.right.y)); HX(B(F.mv.co.entry.x2C.left.x)); HX(B(F.mv.co.entry.x2C.left.y));
                HX(B(F.mv.co.common.x4.x));
                HX(EP(F.x20A0_accessory));
                hj(&J); hj(&A);
                printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
                fflush(0);
            }
        }
    }
    return 0;
}
