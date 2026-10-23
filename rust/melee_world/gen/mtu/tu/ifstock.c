/* TU: src/melee/if/ifstock.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
static GXColor GCOL[2];
static Vec3 V3A;
static HSD_Archive* ARCP;
static DynamicModelDesc* PMDL[2];
static signed char SCA[140], SCB[140], SCC[140], SCD[140];
static u8 HEAP[4096] __attribute__((aligned(64)));
static int HOFF;
static int NULLJ;
static int STK[8], COIN[8];
static HSD_ImageDesc* PIDT[16];
static u8 FDAT[1024];
static u8 FBUF[64];
static u8* FPTR;
static int MAXI;
static char OUTB[4];
static void* halloc(int n) { int sz; void* r; if (n < 0) return 0; sz = (n + 7) & ~7; if (HOFF + sz > 4096) return 0; r = &HEAP[HOFF]; HOFF += sz; return r; }
static const char* const NM[8] = { "Captain Falcon", "mr game & watch", "Ness", "  --  ", "7up", "a", "", "Dr. Mario 64" };

#include "ifstock_gen.h"



static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
int k;
    if (HDBG) printf("PAD %02X %02X %02X %02X rs=%08X\n", (u8)ifStock_804A1378.pad1F0[0], (u8)ifStock_804A1378.pad1F0[1], (u8)ifStock_804A1378.pad1F0[2], (u8)ifStock_804A1378.pad1F0[3], rs);
    for (k = 0; k < 6; k++) P_HSD_GObj[k].user_data = &ifStock_804A1378.x204[rnd() % 6u];
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    for (op = 0; op < 52; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 800, 400, 400, 400, 400, 400, 300, 400, 300, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 800, 600, 1500, 400, 400, 800, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(int) a0; __typeof__(int) a1;  a0 = (int) (rnd() % 6u); a1 = (int) (rnd() % 6u); ret = (u32) ifStock_802F7EFC(a0, a1); } break;
            case 1: { __typeof__(u8) a0;  a0 = (u8) (rnd() % 6u); ret = B(ifStock_802F8298_tobj_frame(a0)); } break;
            case 2: { __typeof__(struct IfStockUserData *) a0;  a0 = (struct IfStockUserData *) ((struct IfStockUserData*) &ifStock_804A1378.x204[rnd() % 6u]); ret = PI((const void*) ifStock_802F8298_update_stocks(a0)); } break;
            case 3: { __typeof__(u8 *) a0;  a0 = (u8 *) (&ifStock_804A1378.x204[rnd() % 6u].anim[rnd() % 7u]); ifStock_802F8298_step_anim(a0); } break;
            case 4: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ifStock_802F8298(a0); } break;
            case 5: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ret = PI((const void*) ifStock_802F89F8_get_data(a0)); } break;
            case 6: { __typeof__(int) a0;  a0 = (int) (((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))); ret = (u32) ifStock_802F89F8_count(a0); } break;
            case 7: { __typeof__(int) a0;  a0 = (int) ((-2 + (int) (rnd() % 15u))); ret = (u32) ifStock_802F89F8_divisor(a0); } break;
            case 8: { __typeof__(int) a0; __typeof__(int) a1;  a0 = (int) (((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))); a1 = (int) ((-1 + (int) (rnd() % 9u))); ret = (u32) ifStock_802F89F8_digit(a0, a1); } break;
            case 9: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ifStock_802F89F8(a0); } break;
            case 10: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802F8DB0(a0); } break;
            case 11: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802F8E08(a0); } break;
            case 12: { __typeof__(HSD_GObj *) a0; { int q; for (q = 0; q < 16; q++) if (!ifStock_804A1A8C[q]) ifStock_804A1A8C[q] = &P_HSD_GObj[0]; } a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802F916C(a0); } break;
            case 13: { __typeof__(HSD_GObj *) a0; __typeof__(struct IfStockUserData *) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (struct IfStockUserData *) ((struct IfStockUserData*) &ifStock_804A1378.x204[rnd() % 6u]); fn_802F9410_inline(a0, a1); } break;
            case 14: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802F9410(a0); } break;
            case 15: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802F94E0(a0, a1); } break;
            case 16: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802F9548(a0, a1); } break;
            case 17: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802F9598(a0, a1); } break;
            case 18: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802F95E8(a0, a1); } break;
            case 19: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802F9680(a0, a1); } break;
            case 20: { __typeof__(HSD_JObj *) a0;  a0 = (HSD_JObj *) (pick_HSD_JObj(1)); ifStock_MexAttach(a0); } break;
            case 21: { __typeof__(int) a0; __typeof__(int) a1;  a0 = (int) (fv()); a1 = (int) (fv()); ret = B(ifStock_MexFrame(a0, a1)); } break;
            case 22: { __typeof__(int) a0; __typeof__(int) a1; __typeof__(float) a2; __typeof__(float) a3;  a0 = (int) (fv()); a1 = (int) (fv()); a2 = (float) (rg()); a3 = (float) (rg()); ret = PI((const void*) ifStock_802F96D0(a0, a1, a2, a3)); } break;
            case 23: { __typeof__(VsSceneState *) a0; __typeof__(int) a1;  a0 = (VsSceneState *) (&P_VsSceneController[0].state); a1 = (int) (rnd() % 6u); ret = PI((const void*) ifStock_802F98E8_get_match_info(a0, a1)); } break;
            case 24: { __typeof__(u8) a0; __typeof__(u8) a1; NULLJ = 1; if ((rnd() & 7) == 0) ifStock_804A1378.x0 = 0; { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } } a0 = (u8) (rnd() % 6u); a1 = (u8) ((0 + (int) (rnd() % 4u))); ifStock_802F98E8(a0, a1); } break;
            case 25: { __typeof__(int) a0;  a0 = (int) ((0 + (int) (rnd() % 130u))); ret = PI((const void*) ifStock_802F9F48_inline(a0)); } break;
            case 26: { __typeof__(int) a0;  a0 = (int) ((0 + (int) (rnd() % 130u))); ret = PI((const void*) ifStock_802F9F48(a0)); } break;
            case 27: { __typeof__(int) a0;  a0 = (int) ((0 + (int) (rnd() % 4u))); ret = PI((const void*) ifStock_802FA118(a0)); } break;
            case 28: { __typeof__(int) a0;  a0 = (int) ((-500 + (int) (rnd() % 12001u))); ifStock_802FA2D0(a0); } break;
            case 29: { __typeof__(int) a0;  a0 = (int) ((0 + (int) (rnd() % 2u))); ifStock_802FA5BC(a0); } break;
            case 30: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802FA6C4(a0); } break;
            case 31: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802FA8C0(a0); } break;
            case 32: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802FAC34(a0); } break;
            case 33: { __typeof__(int) a0;  a0 = (int) ((0 + (int) (rnd() % 2u))); ifStock_802FA5BC_noinline(a0); } break;
            case 34: { __typeof__(HSD_GObj **) a0; __typeof__(int) a1;  a0 = (HSD_GObj **) (ifStock_804A1A8C); a1 = (int) ((0 + (int) (rnd() % 17u))); ifStock_ClearGObjSlots(a0, a1); } break;
            case 35: { __typeof__(HSD_GObj **) a0;  a0 = (HSD_GObj **) (ifStock_804A1A8C); ifStock_CreateStockGObjs(a0); } break;
            case 36: {    ifStock_802FAEC4(); } break;
            case 37: {    ifStock_802FB390(); } break;
            case 38: {    ifStock_802FB41C(); } break;
            case 39: {    ifStock_802FB484(); } break;
            case 40: { __typeof__(HSD_MObj *) a0; __typeof__(GXColor *) a1;  a0 = (HSD_MObj *) (pick_HSD_MObj(0)); a1 = (GXColor *) (&GCOL[0]); copyColor(a0, a1); } break;
            case 41: { __typeof__(int) a0; __typeof__(GXColor *) a1;  a0 = (int) (rnd() % 6u); a1 = (GXColor *) (&GCOL[0]); ifStock_802FB4EC(a0, a1); } break;
            case 42: { __typeof__(int) a0;  a0 = (int) (rnd() % 6u); ifStock_802FB650(a0); } break;
            case 43: { __typeof__(int) a0;  a0 = (int) (rnd() % 6u); ifStock_802FB6AC(a0); } break;
            case 44: {    ifStock_GenoReset(); } break;
            case 45: { __typeof__(int) a0; __typeof__(int) a1; __typeof__(HSD_ImageDesc *) a2;  a0 = (int) ((0 + (int) (rnd() % 4u))); a1 = (int) ((0 + (int) (rnd() % 4u))); a2 = (HSD_ImageDesc *) (pick_HSD_ImageDesc(1)); ret = PI((const void*) ifStock_GenoFind(a0, a1, a2)); } break;
            case 46: { __typeof__(HSD_TObj *) a0; __typeof__(int) a1;  a0 = (HSD_TObj *) (pick_HSD_TObj(0)); a1 = (int) (rnd() % 2u); ret = PI((const void*) ifStock_GenoBinding(a0, a1)); } break;
            case 47: { __typeof__(HSD_TObj *) a0; __typeof__(f32) a1;  a0 = (HSD_TObj *) (pick_HSD_TObj(1)); a1 = (f32) (((rnd() % 3u) == 0 ? 1000000.0f + (float) ((rnd() % 6u) * 256u + rnd() % 4u) : rg())); ifStock_ReqAnim(a0, a1); } break;
            case 48: { __typeof__(u8 **) a0; __typeof__(u8) a1; FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u]; a0 = (u8 **) (&FPTR); a1 = (u8) (fv()); ret = B(genoArt_FObjValue(a0, a1)); } break;
            case 49: { __typeof__(u8 **) a0; FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u]; a0 = (u8 **) (&FPTR); ret = (u32) genoArt_FObjPacked(a0); } break;
            case 50: { __typeof__(HSD_TObj *) a0; __typeof__(int) a1; __typeof__(int *) a2;  a0 = (HSD_TObj *) (pick_HSD_TObj(1)); a1 = (int) ((-1 + (int) (rnd() % 8u))); a2 = (int *) (&MAXI); ret = (u32) genoArt_ImageKeyAt(a0, a1, a2); } break;
            case 51: { { const char* a0 = NM[rnd() % 8u]; genoArt_Initials(a0, OUTB); } } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
