/* TU: src/melee/if/ifstatus.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GXColor GCOL[3];
static HSD_GObj* GOP[1];
static u32 CHR[1];
static void cbA(s32 a) { T(7001); T((u32) a); }
static void cbB(s32 a) { T(7002); T((u32) a); }
static void* cbsel(void) { u32 q = rnd() % 3; return q == 0 ? (void*) 0 : q == 1 ? (void*) cbA : (void*) cbB; }
static HSD_MatAnimJoint* MAJP[2];
static DynamicModelDesc* NMOD[2];
static HSD_Archive* ARCP;
static Vec3 V3A;
static GXColor rcol(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }

#include "ifstatus_gen.h"



static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
int k;
    for (k = 0; k < 6; k++) P_HSD_GObj[k].hsd_obj = &P_HSD_JObj[k];
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    freg((const void*) cbA);
    freg((const void*) cbB);
    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\n", 99, 0, 0u, H, TR);
    for (op = 0; op < 33; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: {    ret = PI((const void*) ifStatus_GetHUDInfo()); } break;
            case 1: { __typeof__(UnkX *) a0; __typeof__(s32) a1; __typeof__(s32) a2;  a0 = (UnkX *) ((UnkX*) &ifStatus_HudInfo.players[rnd() % 6u]); a1 = (s32) (fv()); a2 = (s32) (fv()); ifStatus_PercentOnDeathAnimationThink(a0, a1, a2); } break;
            case 2: { __typeof__(float) a0; __typeof__(float) a1;  a0 = (float) (rg()); a1 = (float) (rg()); ret = B(foo(a0, a1)); } break;
            case 3: {    ret = B(offset_rand()); } break;
            case 4: { __typeof__(HSD_JObj *) a0;  a0 = (HSD_JObj *) (pick_HSD_JObj(0)); jobj_flagCheckSetMtxDirtySub(a0); } break;
            case 5: { __typeof__(IfDamageState *) a0; __typeof__(s32) a1;  a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (s32) (fv()); ifStatus_802F4B84(a0, a1); } break;
            case 6: { __typeof__(IfDamageState *) a0; __typeof__(HSD_MatAnimJoint ***) a1;  a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (HSD_MatAnimJoint ***) ((HSD_MatAnimJoint***) &ifStatus_HudInfo.damage_num_matanims); ifStatus_InitDamageDigits(a0, a1); } break;
            case 7: { __typeof__(IfDamageState *) a0; __typeof__(s32) a1;  a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (s32) (rnd() % 4u); ret = B(ifStatus_GetStoredTranslationX(a0, a1)); } break;
            case 8: { __typeof__(HSD_GObj **) a0;  a0 = (HSD_GObj **) (&GOP[0]); ifStatus_CreateMarkGObj(a0); } break;
            case 9: { __typeof__(s32) a0; __typeof__(CharacterKind *) a1;  a0 = (s32) (fv()); a1 = (CharacterKind *) ((CharacterKind*) &CHR[0]); ifStatus_GetPlayerCharacter(a0, a1); } break;
            case 10: { __typeof__(IfDamageState *) a0; __typeof__(HSD_JObj *) a1; __typeof__(GXColor *) a2; __typeof__(GXColor *) a3; __typeof__(GXColor *) a4;  a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (HSD_JObj *) (pick_HSD_JObj(0)); a2 = (GXColor *) (&GCOL[0]); a3 = (GXColor *) (&GCOL[1]); a4 = (GXColor *) (&GCOL[2]); ifStatus_UpdateDamageDisplay(a0, a1, a2, a3, a4); } break;
            case 11: { __typeof__(HSD_GObj *) a0; ifStatus_HudInfo.players[rnd() % 6u].HUD_parent_entity = &P_HSD_GObj[0]; a0 = (HSD_GObj *) (&P_HSD_GObj[0]); ifStatus_802F4EDC(a0); } break;
            case 12: { __typeof__(HSD_GObj *) a0; ifStatus_HudInfo.players[rnd() % 6u].HUD_parent_entity = &P_HSD_GObj[0]; { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; } a0 = (HSD_GObj *) (&P_HSD_GObj[0]); ifStatus_802F5B48(a0); } break;
            case 13: { __typeof__(HSD_GObj *) a0; __typeof__(s32) a1; ifStatus_HudInfo.players[rnd() % 6u].HUD_parent_entity = &P_HSD_GObj[0]; a0 = (HSD_GObj *) (&P_HSD_GObj[0]); a1 = (s32) (fv()); ifStatus_802F5DE0(a0, a1); } break;
            case 14: { __typeof__(HSD_GObj *) a0; __typeof__(s32) a1; ifStatus_HudInfo.players[rnd() % 6u].next = &P_HSD_GObj[0]; a0 = (HSD_GObj *) (&P_HSD_GObj[0]); a1 = (s32) (fv()); ifStatus_802F5E50(a0, a1); } break;
            case 15: { __typeof__(IfDamageState *) a0; __typeof__(s32) a1; { int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; } { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; } a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (s32) (rnd() % 6u); ret = PI((const void*) ifStatus_802F5EC0(a0, a1)); } break;
            case 16: { __typeof__(HSD_JObj *) a0; __typeof__(s32) a1; if (rnd() & 1) P_HSD_JObj[rnd() % 10u].next = 0; a0 = (HSD_JObj *) (pick_HSD_JObj(1)); a1 = (s32) ((-2 + (int) (rnd() % 8u))); ret = PI((const void*) ifStatus_802F6194(a0, a1)); } break;
            case 17: { __typeof__(IfDamageState *) a0; __typeof__(s32) a1;  a0 = (IfDamageState *) (&ifStatus_HudInfo.players[rnd() % 6u]); a1 = (s32) (rnd() % 6u); ret = PI((const void*) ifStatus_802F61FC(a0, a1)); } break;
            case 18: { __typeof__(s32) a0; { int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; } { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; } a0 = (s32) (rnd() % 6u); ifStatus_802F6508(a0); } break;
            case 19: { __typeof__(int) a0; { int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; } { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; } a0 = (int) ((0 + (int) (rnd() % 8u))); ifStatus_802F665C(a0); } break;
            case 20: {    ifStatus_802F66A4(); } break;
            case 21: { __typeof__(u8) a0;  a0 = (u8) (rnd() % 6u); ifStatus_802F6788(a0); } break;
            case 22: {    ifStatus_802F6804(); } break;
            case 23: {    ifStatus_802F6898(); } break;
            case 24: {    ifStatus_802F68F0(); } break;
            case 25: { __typeof__(s32) a0;  a0 = (s32) (rnd() % 6u); ifStatus_802F6948(a0); } break;
            case 26: { __typeof__(s32) a0; __typeof__(void (*)(s32)) a1;  a0 = (s32) (rnd() % 6u); a1 = (void (*)(s32)) ((void (*)(s32)) cbsel()); ifStatus_TriggerStockLoss(a0, a1); } break;
            case 27: { __typeof__(s32) a0; __typeof__(s32) a1;  a0 = (s32) (rnd() % 6u); a1 = (s32) (fv()); ifStatus_802F69C0(a0, a1); } break;
            case 28: { __typeof__(s32) a0;  a0 = (s32) (rnd() % 6u); ifStatus_802F6AF8(a0); } break;
            case 29: { __typeof__(s32) a0;  a0 = (s32) (rnd() % 6u); ifStatus_802F6C04(a0); } break;
            case 30: { __typeof__(s32) a0;  a0 = (s32) (rnd() % 6u); ifStatus_802F6D10(a0); } break;
            case 31: { __typeof__(int) a0; { int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; } { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; } a0 = (int) (rnd() % 6u); ifStatus_802F6E1C(a0); } break;
            case 32: { __typeof__(s32) a0;  a0 = (s32) (rnd() % 6u); ifStatus_802F6E3C(a0); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
