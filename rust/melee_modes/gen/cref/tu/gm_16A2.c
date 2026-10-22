/* TU: src/melee/gm/gm_16A2.c */
/* HS: lbl_8046B488_t lbl_8046B668_t VsSceneState StartMeleeRules Vec3 */
/* STUBS: Player_GetFlagsBit1:ret=small:2 Player_GetPlayerCharacter:ret=small:40 Player_GetPlayerSlotType:ret=small:5 */
/* STUBS: Player_GetPlayerState:ret=small:4 gm_IsCKindUnlocked Player_800325C8:ret=small:9 Player_80031DA8 */
/* STUBS: Player_SetFlagsBit1 Player_SetTeam fn_8016B138 Player_SetSlottype Player_UnsetFlagsBit1 fn_8016B738 Player_80036D24 */
/* STUBS: Stage_GetCamBoundsTopOffset Player_80032768 Player_SetPlayerCharacter Player_SetStocks Player_SetCostumeId */
/* STUBS: Player_SetControllerIndex Player_GetCostumeId:ret=small:6 Player_SetMoreFlagsBit6 Player_SetMoreFlagsBit1 */
/* STUBS: Player_SetFacingDirection Player_SetHUDDamage Player_SetPlayerId Player_SetFlagsBit0 Player_SetNametagSlotID */
/* STUBS: Player_SetPlayerAndEntityCpuLevel Player_SetPlayerAndEntityCpuType Player_SetFlagsBit5 Player_SetFlagsAEBit0 */
/* STUBS: Player_SetFlagsBit6 Player_SetModelScale Player_SetAttackRatio Player_SetDefenseRatio Player_SetMoreFlagsBit5 */
/* STUBS: Player_SetUnk4D Player_SetFlagsAEBit1 Player_80031AD0 ifStatus_802F6508 un_802FD28C Player_GetStocks:ret=small:3 */
/* STUBS: Player_80031848 HSD_Randi:manual gm_GetNumCostumesForCKind:manual memzero:manual fn_8016AE60:manual */
/* STUBS: gmVs_GetSceneState:manual Ground_801C2D24:manual Player_SetStructFunc:manual gm_GetStartMeleeRules:manual */
/* C oracle for src/tu/gm_16A2.rs: the whole TU included verbatim (TARGET_PC build); every callee out of it is a
 * recording stub. The oracle is compiled with 4-byte pointers (like the GameCube), so lbl_8046B488_t's byte
 * layout is the game's. One line per case: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "hs_gm_16A2.h"
#include "stubs_gm_16A2.h"

static VsSceneState VSS;
static StartMeleeRules SMR;

s32 HSD_Randi(s32 max) { T(900); T(max); return max > 0 ? (s32) (rnd() % (u32) max) : 0; }
u8 gm_GetNumCostumesForCKind(u8 ck) { u8 r; T(901); T(ck); r = 1 + rnd() % 9; return r; }
void memzero(void* m, ssize_t n) { T(902); T((u32) n); zero(m, (unsigned) n); }
struct lbl_8046B488_t* fn_8016AE60(void) { T(903); return &lbl_8046B488; }
VsSceneState* gmVs_GetSceneState(void) { T(904); return &VSS; }
bool Ground_801C2D24(enum_t i, Vec3* v) { T(905); T(i); v->x = rg(); v->y = rg(); v->z = rg(); return rnd() & 1; }
void Player_SetStructFunc(s32 slot, void* f) { T(906); T(slot); T(f == (void*) fn_8016A488); }
StartMeleeRules* gm_GetStartMeleeRules(void) { T(907); return &SMR; }

static bool cbR(int a) { T(950); T(a); return SR(2); }
static void cbE(s32 slot, u8 rem) { T(951); T(slot); T(rem); }

static struct { u8 p0[512]; s8 a[320]; u8 p1[512]; s8 b[320]; u8 p2[512]; s8 c[16]; u8 p3[512]; } GG;
#define BUFA GG.a
#define BUFB GG.b
#define BUFC GG.c
static int chk(void){ int i; for(i=0;i<512;i++){ if(GG.p0[i]) return 1; if(GG.p1[i]) return 2; if(GG.p2[i]) return 3; if(GG.p3[i]) return 4;} return 0; }

/* gm_8016A404's callback lives in the record's pad_1BC (a 4-byte pointer on the GC); the Rust port keeps it
 * in a static, so the oracle compares "is it set" and keeps the pad out of the record hash */
#define CBSLOT (*(u32*) ((char*) &lbl_8046B488 + 0x1BC))
static u32 state_hash(void)
{
    u32 h = 0, cbs = CBSLOT;
    CBSLOT = 0;
    h = hs_lbl_8046B488_t(&lbl_8046B488, h);
    CBSLOT = cbs;
    h = hs_lbl_8046B668_t(&lbl_8046B668, h);
    h = hs_VsSceneState(&VSS, h);
    h = hs_StartMeleeRules(&SMR, h);
    h = hm_(h, HBUF(BUFA, sizeof BUFA));
    h = hm_(h, HBUF(BUFB, sizeof BUFB));
    h = hm_(h, HBUF(BUFC, sizeof BUFC));
    h = hm_(h, cbs != 0);
    return h;
}

/* -- helpers shared with the Rust test: list with a -2 terminator at index n */
static void mklist(s8* b, int n, int range, int special)
{
    int i;
    for (i = 0; i < n; i++) b[i] = (rnd() % 4 == 0 && special >= 0) ? (s8) special : (s8) (rnd() % range);
    b[n] = -2;
}

void* __acrt_iob_func(unsigned);
int setvbuf(void*, char*, int, unsigned);
int main(void)
{
    int op, k;
    setvbuf(__acrt_iob_func(1), 0, 4, 0);
    for (op = 1; op <= 41; op++) {
        for (k = 0; k < 700; k++) {
            u32 ret = 0;
            int i;
            tu_case(op, k);
            zero(&lbl_8046B488, sizeof lbl_8046B488); zero(&lbl_8046B668, sizeof lbl_8046B668);
            zero(&VSS, sizeof VSS); zero(&SMR, sizeof SMR);
            zero(BUFA, sizeof BUFA); zero(BUFB, sizeof BUFB); zero(BUFC, sizeof BUFC);
            fl_lbl_8046B488_t(&lbl_8046B488);
            fl_lbl_8046B668_t(&lbl_8046B668);
            fl_VsSceneState(&VSS);
            fl_StartMeleeRules(&SMR);
            CBSLOT = 0;
            VSS.hud_enabled = rnd() & 1;
            /* terminated lists the loops walk: the three ckind slots hold plausible ids */
            lbl_8046B488.x20[rnd() % 100] = -2;
            for (i = 0; i < 6; i++) lbl_8046B488.x1A6[i] = rnd() % 100;
            lbl_8046B488.x7 = rnd() % 6;
            lbl_8046B488.x8 = rnd() % 4;
            lbl_8046B488.x9 = rnd() % 8;
            lbl_8046B488.xB = rnd() & 1;
            lbl_8046B488.xE = rnd() & 1;
            for (i = 0; i < 3; i++) (&lbl_8046B488.x0)[i] = (rnd() % 3 == 0) ? 4 : (rnd() % 5 == 0 ? 0x21 : rnd() % 40);
            lbl_8046B668.arr2[rnd() % 27] = -2;
            for (i = 0; i < 28; i++) lbl_8046B668.arr1[i] = rnd() % 30;
            switch (op) {
            case 1: T(gm_1601_GetUnkData() == &lbl_8046B488); break;
            case 2: { int a = rnd() % 0x1B0; if (a == 0x10) a = 0x0F; /* the bitfield byte is packed per compiler; read through unk_10_b* instead */ ret = (u8) gm_80169370(a); } break;
            case 3: ret = gm_80169384(); break;
            case 4: ret = gm_80169394(); break;
            case 5: ret = fn_801693A8(); break;
            case 6: ret = gm_801693BC(rnd() % 6); break;
            case 7: fn_80169434((rnd() & 1) ? cbR : 0); break;
            case 8: if (rnd() & 1) lbl_8046B488.x1B8 = cbR; ret = fn_80169444(rnd() & 1); break;
            case 9: ret = gm_801694A0(0); break;
            case 10: T(gm_80169520() == lbl_8046B488.x20); T(gm_80169530() == lbl_8046B488.xA2); T(gm_80169540() == lbl_8046B488.x124); break;
            case 11: fn_80169550(rnd() % 6); break;
            case 12: { int n = rnd() % 40; fn_80169574(n, BUFA); } break;
            case 13: {
                u8 a0 = (rnd() % 6 == 0) ? 0x21 : rnd() % 40;
                u8 a1 = (rnd() % 3 == 0) ? a0 : rnd() % 40;
                u8 a2 = rnd() % 8;
                int n = 1 + rnd() % 29;
                for (i = 0; i < n; i++) ((u8*) BUFB)[i] = (rnd() % 2) ? a0 : rnd() % 40;
                for (i = 0; i < n; i++) BUFA[i] = rnd();
                BUFA[n] = -2;
                fn_801695BC(a0, a1, a2, (const u8*) BUFB, BUFA);
            } break;
            case 14: {
                s8 c = (rnd() % 6 == 0) ? 0x21 : rnd() % 40;
                s8 co = (rnd() % 4 == 0) ? (s8) 254 : (s8) (rnd() % 8);
                s8 nc = (rnd() % 3 == 0) ? c : (s8) (rnd() % 40);
                s8 nco = (s8) (rnd() % 8);
                mklist(BUFA, rnd() % 30, 100, -1);
                fn_801697FC(c, co, nc, nco, BUFA);
            } break;
            case 15: {
                u8 a0[3];
                u8 a1 = rnd() % 40, a2 = rnd() % 8;
                int n = 1 + rnd() % 19;
                for (i = 0; i < 3; i++) a0[i] = (rnd() % 6 == 0) ? 0x21 : rnd() % 40;
                for (i = 0; i < n; i++) ((u8*) BUFB)[i] = (rnd() % 2) ? a0[rnd() % 3] : rnd() % 40;
                for (i = 0; i < n; i++) BUFA[i] = rnd();
                BUFA[n] = -2;
                fn_8016989C(a0, a1, a2, (u8*) BUFB, BUFA);
            } break;
            case 16: {
                u8 a0 = rnd() % 4;
                mklist(BUFB, rnd() % 30, 100, -1);
                for (i = 0; i < 40; i++) BUFA[i] = rnd();
                lbl_8046B488.x0 = rnd() % 40; lbl_8046B488.x1 = (rnd() % 4) ? 0x21 : rnd() % 40; lbl_8046B488.x2 = (rnd() % 4) ? 0x21 : rnd() % 40;
                fn_80169900(a0, &lbl_8046B488, BUFA, BUFB);
            } break;
            case 17: {
                u8 a0 = rnd() % 3;
                mklist(BUFB, rnd() % 12, 100, -1);
                for (i = 0; i < 40; i++) BUFA[i] = rnd();
                for (i = 0; i < 27; i++) lbl_8046B488.x1C0[i] = (rnd() % 4 == 0) ? -1 : (s8) (rnd() % 30);
                fn_80169A84(a0, BUFA, BUFB);
            } break;
            case 18: { s8 a0 = (rnd() % 3 == 0) ? 4 : (s8) (rnd() % 40); s8 a1 = rnd(); fn_80169C54(a0, a1); } break;
            case 19: { s8 a0 = (rnd() % 3 == 0) ? 4 : (s8) (rnd() % 40); s8 a1 = rnd(); fn_80169F50(a0, a1); } break;
            case 20: fn_8016A09C(); break;
            case 21: gm_8016A164(); break;
            case 22: ret = fn_8016A1E4(); break;
            case 23: ret = gm_8016A1F8(); break;
            case 24: gm_8016A21C(&SMR); T(SMR.x54 == (void*) &lbl_8046B488); break;
            case 25: {
                s8 k0 = (rnd() % 3 == 0) ? 4 : (s8) (rnd() % 40), k1 = (rnd() % 5 == 0) ? 0x21 : (s8) (rnd() % 40), k2 = (s8) (rnd() % 40);
                u8 a3 = rnd(), a4 = rnd(), a5 = rnd();
                int mode = rnd() % 3, a7 = rnd() % 4;
                u8 color = rnd() & 1, p87 = rnd() % 40, p8b = rnd() % 8;
                int x6 = rnd() % 100, x7 = rnd() % 60, x9 = rnd() % 9, xA = rnd() % 5, flag2 = rnd(), flag1 = rnd();
                f32 f1 = rg(), f2 = rg();
                lbl_8046B488.pad_1DB[0] = -1; /* the deal loop wraps at -1: keeps it inside the record (27 deck bytes + this one) */
                gm_8016A22C(k0, k1, k2, a3, a4, a5, mode, a7, color, p87, p8b, x6, x7, x9, xA, flag2, flag1, f1, f2);
            } break;
            case 26: gm_8016A404((rnd() & 1) ? (s32) (size_t) cbE : 0); break;
            case 27: gm_8016A414(rg()); break;
            case 28: gm_8016A424(rnd()); break;
            case 29: gm_8016A434(); break;
            case 30: fn_8016A450(); break;
            case 31: fn_8016A46C(); break;
            case 32: fn_8016A488(rnd() % 6); break;
            case 33:
                gm_8016A404((rnd() & 1) ? (s32) (size_t) cbE : 0);
                fn_80169434((rnd() & 1) ? cbR : 0);
                lbl_8046B488.unk_10_b1 = (rnd() % 8) != 0;
                lbl_8046B488.xA = rnd() % 4;
                fn_8016A4C8();
                break;
            case 34: gm_8016A92C(&SMR); T(SMR.x58 == &lbl_8046B668); break;
            case 35: SMR.x58 = (rnd() & 1) ? &lbl_8046B668 : 0; ret = gm_8016A944(); break;
            case 36: T(gm_8016A97C() == lbl_8046B668.arr2); break;
            case 37: T(gm_8016A98C() == &lbl_8046B668); break;
            case 38: { s8 a0 = rnd(), a1 = (rnd() % 5 == 0) ? (s8) 254 : (s8) rnd(); ret = gm_8016A998(a0, a1); } break;
            case 39: { u8 a0 = rnd(); s8 a1 = (rnd() % 5 == 0) ? (s8) 254 : (s8) rnd(); ret = gm_8016A9E8(a0, a1); } break;
            case 40: {
                SMR.x58 = (rnd() % 4) ? &lbl_8046B668 : 0;
                { s8 ck = rnd() % 30; s8 co = (rnd() % 5 == 0) ? (s8) 254 : (s8) (rnd() % 8);
                  for (i = 0; i < 28; i++) if (rnd() % 3 == 0) lbl_8046B668.arr2[i] = co;
                  for (i = 0; i < 28; i++) if (rnd() % 3 == 0) lbl_8046B668.arr1[i] = ck;
                  ret = gm_8016AC44(ck, co); }
            } break;
            case 41: ret = gm_PcTestLegacyCostumes(); break;
            }
            if (chk()) { printf("OOB %d %d %d\n", op, k, chk()); return 1; }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
