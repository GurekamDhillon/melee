/* TU: src/melee/ft/fighter.c */
/* C oracle for src/tu/fighter.rs (the per-frame fighter functions): the whole TU included verbatim; every
 * callee out of it is a recording stub (gen/mtu/stubs.py); the state is a random fighter world filled field by
 * field (gen/mtu/fill.py). Each line: op case ret state-hash trace. */
#include TU_SRC
/* the game links lbtrigf.c's atan2f (not the CRT's); the port calls melee_runtime::lbtrigf */
#include <melee/lb/lbtrigf.c>
#include "mtu_common.h"
/* PC profiler diagnostics: not game logic, not ported, not traced */
int ProfActive(void) { return 0; }
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCount(int a) { (void) a; }

#define NG 3
#define NJ 8
static HSD_GObj GO[NG];
static HSD_JObj JO[NJ];
static Fighter FT[NG];
static ftData FDATA[NG];
static ftCommonData CD;
static u8 PADB[80][2048];
static UnkCostumeStruct CLB[256];

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

#include "fighter_stubs.h"
#include "fighter_state.h"

static void world(void)
{
    int i;
    zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT); zero(FDATA, sizeof FDATA);
    for (i = 0; i < NG; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) {
        GO[i].user_data = &FT[i];
        GO[i].hsd_obj = &JO[i];
        GO[i].classifier = HSD_GOBJ_CLASS_FIGHTER;
        FT[i].gobj = &GO[i];
        FT[i].ft_data = &FDATA[i];
        FT[i].grab_cb = cbev1; FT[i].grabbed_cb = cbev3;
        FT[i].kind = (FighterKind) ((u32) FT[i].kind % 27u); FT[i].x200C &= 3; FT[i].x2010 &= 3; FT[i].player_id = FT[i].player_id % 4; FT[i].x618_player_id = FT[i].x618_player_id % 4; FT[i].x61A_controller_index = FT[i].x61A_controller_index % 4;
        { int pk = i * 24; FDATA[i].x0 = (void*) PADB[pk++]; FDATA[i].ext_attr = (void*) PADB[pk++]; FDATA[i].x8 = (void*) PADB[pk++]; FDATA[i].xC = (void*) PADB[pk++]; FDATA[i].x10 = (void*) PADB[pk++]; FDATA[i].x14 = (void*) PADB[pk++]; FDATA[i].x18 = (void*) PADB[pk++]; FDATA[i].x1C = (void*) PADB[pk++]; FDATA[i].x20 = (void*) PADB[pk++]; FDATA[i].x24 = (void*) PADB[pk++]; FDATA[i].x28 = (void*) PADB[pk++]; FDATA[i].x2C = (void*) PADB[pk++]; FDATA[i].x30 = (void*) PADB[pk++]; FDATA[i].x34 = (void*) PADB[pk++]; FDATA[i].x38 = (void*) PADB[pk++]; FDATA[i].x3C = (void*) PADB[pk++]; FDATA[i].x40 = (void*) PADB[pk++]; FDATA[i].x44 = (void*) PADB[pk++]; FDATA[i].x48_items = (void*) PADB[pk++]; FDATA[i].x4C_sfx = (void*) PADB[pk++]; FDATA[i].x50 = (void*) PADB[pk++]; FDATA[i].x54 = (void*) PADB[pk++]; FDATA[i].x58 = (void*) PADB[pk++]; FDATA[i].x5C = (void*) PADB[pk++]; }
    }
    for (i = 0; i < NJ; i++) {
        JO[i].parent = i ? &JO[(i - 1) / 2] : 0;
        JO[i].child = 2 * i + 1 < NJ ? &JO[2 * i + 1] : 0;
        JO[i].next = (i & 1) && i + 1 < NJ ? &JO[i + 1] : 0;
    }
    for (i = 0; i < Ft_Kind_Max; i++) CostumeListsForeachCharacter[i].costume_list = CLB;
    p_ftCommonData = &CD;
}
static void hworld(void)
{
    int i;
    for (i = 0; i < NG; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) hash_HSD_GObj(&GO[i]);
}

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT); preg(FDATA, sizeof FDATA);
    freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3);
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            world();
            ASRT_ARMED = 1;
            if (_setjmp(ASRT_JB, 0) != 0) { printf("%d %d ASSERT%c", op, k, 10); fflush(0); continue; }
            switch (op) {
            case 0: Fighter_8006C5F4(&GO[rnd() % NG]); break;
            case 1: Fighter_CallAcessoryCallbacks_8006C624(&GO[rnd() % NG]); break;
            case 2: Fighter_8006C80C(&GO[rnd() % NG]); break;
            case 3: Fighter_UnkProcessGrab_8006CA5C(&GO[rnd() % NG]); break;
            case 4: Fighter_8006CB94(&GO[rnd() % NG]); break;
            case 5: Fighter_8006D9AC(&GO[rnd() % NG]); break;
            case 6: Fighter_UnkCallCameraCallback_8006D9EC(&GO[rnd() % NG]); break;
            case 7: Fighter_8006DA4C(&GO[rnd() % NG]); break;
            case 8: Fighter_8006A1BC(&GO[rnd() % NG]); break;
            case 9: Fighter_8006A360(&GO[rnd() % NG]); break;
            case 10: Fighter_8006ABA0(&GO[rnd() % NG]); break;
            case 11: Fighter_UnkIncrementCounters_8006ABEC(&GO[rnd() % NG]); break;
            case 12: Fighter_Spaghetti_8006AD10(&GO[rnd() % NG]); break;
            case 13: Fighter_procUpdate(&GO[rnd() % NG]); break;
            case 14: Fighter_UnkApplyTransformation_8006C0F0(&GO[rnd() % NG]); break;
            case 15: Fighter_procMap(&GO[rnd() % NG]); break;
            case 16: Fighter_ProcessHit_8006D1EC(&GO[rnd() % NG]); break;
            case 17: Fighter_UnkProcessDeath_80068354(&GO[rnd() % NG]); break;
            case 18: Fighter_UnkUpdateCostumeJoint_800686E4(&GO[rnd() % NG]); break;
            case 19: Fighter_ResetInputData_80068854(&GO[rnd() % NG]); break;
            }
            hworld();
            ASRT_ARMED = 0;
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
