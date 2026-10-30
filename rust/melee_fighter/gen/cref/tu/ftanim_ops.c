/* TU: src/melee/ft/ftanim.c */
/* C oracle for src/tu/ftanim.rs (fighter animation: per-part anim walk, request / rate, skeleton setup): the whole TU
 * included verbatim; every callee out of it is a recording stub (gen/mtu/stubs.py) except HSD_ForeachAnim (variadic,
 * defined here); the world is three random fighters over an 8-node joint tree, filled field by field (gen/mtu/fill.py).
 * Each line: op case ret state-hash trace. */
#include TU_SRC
#include <sysdolphin/baselib/jobj.h>
#include "mtu_common.h"
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCount(int a) { (void) a; }
int ProfActive(void) { return 0; }

#define NG 3
#define NJ 8
#define NP 16
static HSD_GObj GO[NG];
static HSD_JObj JO[NJ];
static Fighter FT[NG];
static ftData FDATA[NG];
static FighterBone PARTS[NG][64];   /* walks past the filled NP bones read zero bones */
static FighterPartsTable PT;
static FighterPartsTable* PTP[Ft_Kind_Max];
static ftCommonData CD;
static UnkCostumeStruct CLB[256];
static u8 PADB[96][2048];
static HSD_Joint JNT[2];
static FigaTree FIGA[NG];
static struct ftData_x8 X8S[NG];
static u16* X8ROWS[NG][4];
static u16 X8VALS[NG][4][4];
static HSD_TObj TOBJ;
static HSD_AObj AOB;
static ftData_x1C* X1CP[NG][5];
static ftData_x1C X1CS[NG][5];
static u8 X1CL[NG][5][8];
static HSD_AnimJoint* ANIMB[512];
#define ANIMP (ANIMB + 256)   /* indexed by a signed byte */
static s8 FNODES[NG][40];
static FigaTrack FTRACKS[NG][64];

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % NG]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % NJ]; }
static Fighter* pick_Fighter(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FT[(r >> 8) % NG]; }
static Item* pick_Item(void) { return 0; }
static HSD_DObj* pick_HSD_DObj(void) { return 0; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cb4(HSD_GObj* g, int i, float f) { T(7006); T(PI(g)); T((u32) i); T(B(f)); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : (void*) cbev1); }

#include "ftanim_stubs.h"
#include "ftanim_state.h"

/* HSD_ForeachAnim(obj, type, mask, cb, argtype, value): variadic; the two uses are HSD_AObjSetFlags(AU, LOOP) and
 * HSD_AObjSetRate(AF, rate) */
void HSD_ForeachAnim(void* obj, HSD_Type type, HSD_TypeMask mask, void* cb, AObj_Arg_Type at, ...)
{
    __builtin_va_list ap;
    __builtin_va_start(ap, at);
    T(2100); T(PI(obj)); T((u32) type); T((u32) mask); T((u32) at);
    if (cb == (void*) HSD_AObjSetRate) {
        double d = __builtin_va_arg(ap, double);
        T(2101); T(B((float) d));
    } else {
        u32 v = __builtin_va_arg(ap, u32);
        T(2102); T(v);
    }
    __builtin_va_end(ap);
}

static void world(void)
{
    int i, b;
    zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT); zero(FDATA, sizeof FDATA);
    zero(&CD, sizeof CD); zero(JNT, sizeof JNT); JNT[0].child = &JNT[1];
    for (i = 0; i < NG; i++) fill_Fighter(&FT[i]);
    /* the x594 union: only the flag bits are filled; x594_bits / x597_bits do not overlap them in either bit order, so they get their own
       values (the generated fill leaves them to whatever the flag fill's layout puts there) */
    for (i = 0; i < NG; i++) { FT[i].x594_bits = fv(); FT[i].x597_bits = fv(); }
    for (i = 0; i < NJ; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) {
        GO[i].user_data = &FT[i];
        GO[i].hsd_obj = &JO[i];
        GO[i].classifier = HSD_GOBJ_CLASS_FIGHTER;
        FT[i].gobj = &GO[i];
        FT[i].ft_data = &FDATA[i];
        FT[i].grab_cb = cbev1; FT[i].grabbed_cb = cbev3;
        FT[i].kind = (FighterKind) ((u32) FT[i].kind % 27u); FT[i].player_id %= 4; FT[i].x618_player_id %= 4;
        for (b = 0; b < 64; b++) { if (b < NP) fill_FighterBone(&PARTS[i][b]); else { zero(&PARTS[i][b], sizeof PARTS[i][b]); PARTS[i][b].flags_b1 = 1; } PARTS[i][b].joint = &JO[(u32) (i * 5 + b * 3) % NJ]; }
        PARTS[i][0].flags_b1 = 1; PARTS[i][0].flags_b0 = 0; PARTS[i][0].flags_b5 = 0;   /* ftAnim_8006F3DC falls off its end (undefined) without one live part */
        FT[i].parts = PARTS[i];
        FT[i].x108_costume_joint = &JNT[0];
        { int n_ = (int) (rnd() % 5), q_; for (q_ = 0; q_ < n_; q_++) FNODES[i][q_] = (s8) (1 + rnd() % 4); for (; q_ < 40; q_++) FNODES[i][q_] = -1;
          zero(&FTRACKS[i], sizeof FTRACKS[i]); zero(&FIGA[i], sizeof FIGA[i]); FIGA[i].nodes = FNODES[i]; FIGA[i].tracks = FTRACKS[i]; FIGA[i].frames = rg();
          FT[i].x590 = &FIGA[i]; }
    }
    for (i = 0; i < NG; i++) {
        { int pk = i * 24; FDATA[i].x0 = (void*) PADB[pk++]; FDATA[i].ext_attr = (void*) PADB[pk++]; FDATA[i].x8 = (void*) PADB[pk++]; FDATA[i].xC = (void*) PADB[pk++]; FDATA[i].x10 = (void*) PADB[pk++]; FDATA[i].x14 = (void*) PADB[pk++]; FDATA[i].x18 = (void*) PADB[pk++]; FDATA[i].x1C = (void*) PADB[pk++]; FDATA[i].x20 = (void*) PADB[pk++]; FDATA[i].x24 = (void*) PADB[pk++]; FDATA[i].x28 = (void*) PADB[pk++]; FDATA[i].x2C = (void*) PADB[pk++]; FDATA[i].x30 = (void*) PADB[pk++]; FDATA[i].x34 = (void*) PADB[pk++]; FDATA[i].x38 = (void*) PADB[pk++]; FDATA[i].x3C = (void*) PADB[pk++]; FDATA[i].x40 = (void*) PADB[pk++]; FDATA[i].x44 = (void*) PADB[pk++]; FDATA[i].x48_items = (void*) PADB[pk++]; FDATA[i].x4C_sfx = (void*) PADB[pk++]; FDATA[i].x50 = (void*) PADB[pk++]; FDATA[i].x54 = (void*) PADB[pk++]; FDATA[i].x58 = (void*) PADB[pk++]; FDATA[i].x5C = (void*) PADB[pk++]; }
    }
    for (i = 0; i < NG; i++) {
        int r_, c_;
        zero(&X8S[i], sizeof X8S[i]);
        X8S[i].x8.x8 = rnd() % 4;
        for (r_ = 0; r_ < 4; r_++) { for (c_ = 0; c_ < 4; c_++) X8VALS[i][r_][c_] = (u16) (rnd() % 3); X8ROWS[i][r_] = (r_ == 0 || (rnd() % 4)) ? X8VALS[i][r_] : 0; }
        X8S[i].x8.xC = X8ROWS[i];
        FDATA[i].x8 = &X8S[i];
        for (r_ = 0; r_ < 5; r_++) {
            int e_;
            zero(&X1CS[i][r_], sizeof X1CS[i][r_]);
            X1CS[i][r_].x2 = rnd() % 6;
            for (e_ = 0; e_ < 8; e_++) X1CL[i][r_][e_] = (u8) (rnd() % 16);
            X1CS[i][r_].x4 = X1CL[i][r_];
            X1CS[i][r_].x8 = ANIMP;
            X1CP[i][r_] = &X1CS[i][r_];
        }
        FDATA[i].x1C = (void*) X1CP[i];
    }
    for (i = 0; i < 512; i++) ANIMB[i] = (HSD_AnimJoint*) PADB[10 + i % 8];
    zero(&TOBJ, sizeof TOBJ); TOBJ.aobj = (HSD_AObj*) PADB[5];
    for (i = 0; i < NG; i++) {
        int q_;
        FT[i].tobj_list.n_costume_tobjs %= ARRAY_SIZE(FT[i].tobj_list.costume_tobjs) + 1;
        for (q_ = 0; q_ < (int) ARRAY_SIZE(FT[i].tobj_list.costume_tobjs); q_++) FT[i].tobj_list.costume_tobjs[q_] = &TOBJ;
    }
    for (i = 0; i < NJ; i++) {
        JO[i].parent = i ? &JO[(i - 1) / 2] : 0;
        JO[i].child = 2 * i + 1 < NJ ? &JO[2 * i + 1] : 0;
        JO[i].next = (i & 1) && i + 1 < NJ ? &JO[i + 1] : 0;
    }
    zero(&AOB, sizeof AOB); AOB.curr_frame = rg();
    for (i = 0; i < NJ; i++) JO[i].aobj = &AOB;
    zero(&PT, sizeof PT);
    PT.parts_num = (u8) (6 + rnd() % 11);
    for (i = 0; i < Ft_Kind_Max; i++) PTP[i] = &PT;
    ftPartsTable = PTP;
    for (i = 0; i < Ft_Kind_Max; i++) CostumeListsForeachCharacter[i].costume_list = CLB;
    for (i = 0; i < Ft_Kind_Max; i++) { ftData_UnkCallbackPairs0[i].x0 = (rnd() & 1) ? cbev1 : 0; ftData_UnkCallbackPairs0[i].x4 = (rnd() & 1) ? cb4 : 0; }
    p_ftCommonData = &CD;
}
static void hworld(void)
{
    int i, b;
    for (i = 0; i < NG; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) for (b = 0; b < NP; b++) hash_FighterBone(&PARTS[i][b]);
}

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT); preg(FDATA, sizeof FDATA); preg(PADB, sizeof PADB);
    freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cb4);
    for (op = 0; op < 12; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            world();
            ASRT_ARMED = 1;
            if (_setjmp(ASRT_JB, 0) != 0) { printf("%d %d ASSERT%c", op, k, 10); fflush(0); continue; }
            {
                Fighter_GObj* g = &GO[rnd() % NG];
                Fighter* fp = &FT[(u32) (g - GO)];
                switch (op) {
                case 0: ftAnim_8006E054(fp, pick_HSD_JObj(), pick_HSD_JObj(), pick_HSD_JObj()); break;
                case 1: ftAnim_8006E7B8(fp, (Fighter_Part) (rnd() % NP)); break;
                case 2: ftAnim_8006E9B4(g); break;
                case 3: { float a = rw(), r = rg(), bl = rw(); ftAnim_8006EBE8(g, a, r, bl); break; }
                case 4: ftAnim_8006FE48(g); break;
                case 5: ftAnim_80070308(g); break;
                case 6: ftAnim_80070654(g); break;
                case 7: ftAnim_80070758(pick_HSD_JObj()); break;
                case 8: ftAnim_800707B0(g); break;
                case 9: ftAnim_80070904(fp, (Fighter_Part) (rnd() % NP), (HSD_AnimJoint*) PADB[0]); break;
                case 10: { Fighter_Part pt_ = (Fighter_Part) (rnd() % NP); ftAnim_80070A10(fp, pt_, &FIGA[rnd() % NG]); break; }
                case 11: ftAnim_80070E74(g); break;
                }
            }
            hworld();
            ASRT_ARMED = 0;
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
