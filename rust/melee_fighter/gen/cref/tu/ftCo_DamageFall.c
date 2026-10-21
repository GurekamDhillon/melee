/* TU: src/melee/ft/kinds/ftCommon/ftCo_DamageFall.c */
/* C oracle for src/tu/ftCo_DamageFall.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Ops cover the functions new in that file (the model-shift leaves are
 * checked by tests/ftdmgfall_ref.rs). Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define S8(i) ((SRV[i] & 7) == 0)

/* callees of the already-verified leaves: never reached by the ops below */
Vec2** Fighter_804D6530;
bool ft_80081DD4(Fighter_GObj* g) { (void) g; for (;;) {} }
float ftCommon_CalcHitlag(int d, FtMotionId m, float x) { (void) d; (void) m; (void) x; for (;;) {} }
void ft_PlaySFX(Fighter* f, enum_t a, u8 b, u8 c) { (void) f; (void) a; (void) b; (void) c; for (;;) {} }
void ft_800889F4(Fighter* f, FtSFXArr* s) { (void) f; (void) s; for (;;) {} }
void ft_80084DB0(Fighter_GObj* g) { (void) g; for (;;) {} }

void ftCommon_8007D5D4(Fighter* f) { T(1); T(f == &F); }
s32 ftGetParasolStatus(Fighter_GObj* g) { T(2); T(g == &GO); return S8(1) ? (s32) (SRV[2] % 3) : -1; }
void ftCo_800CF4DC(Fighter_GObj* g) { T(3); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(5); T(f == &F); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(6); T(f == &F); T(a); T(b); }
int Replay_UcfVersion(int port) { T(7); T(port); return (SRV[3] & 1) ? 84 : 83; }
int ftUcf_PadRaw(int port, int which_y, int back) { T(8); T(port); T(which_y); T(back); return (int) (rnd() % 256) - 128; }
bool ftCo_800C5240(Fighter_GObj* g) { T(9); T(g == &GO); return S8(4); }
bool ftCo_SpecialAir_CheckInput(Fighter_GObj* g) { T(10); T(g == &GO); return S8(5); }
bool ftCo_80095328(Fighter_GObj* g, bool* o) { T(11); T(g == &GO); T(o == NULL); return S8(6); }
bool ftCo_800D7100(Fighter_GObj* g) { T(12); T(g == &GO); return S8(7); }
bool ftCo_800C3B10(Fighter_GObj* g) { T(13); T(g == &GO); return S8(8); }
bool ftCo_AttackAir_CheckItemThrowInput(Fighter_GObj* g) { T(14); T(g == &GO); return S8(9); }
bool ftCo_800D705C(Fighter_GObj* g) { T(15); T(g == &GO); return S8(10); }
bool ftCo_800CB870(Fighter_GObj* g) { T(16); T(g == &GO); return S8(11); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(17); T(g == &GO); }
bool ftCo_800C5DDC(Fighter_GObj* g) { T(18); T(g == &GO); return S8(12); }
bool ftCo_800C5CD4(Fighter_GObj* g) { T(19); T(g == &GO); return S8(13); }
void ftCo_80090984(Fighter_GObj* g) { T(21); (void) g; }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(20); T(g == &GO); T(cb == ftCo_80090984); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    return H;
}

static u32 run(int op)
{
    switch (op) {
    case 0: ftCo_80090780(&GO); break;
    case 1: return ftCo_Ucf084Tumble(&F);
    case 2: ftCo_DamageFall_IASA(&GO); break;
    case 3: ftCo_DamageFall_Coll(&GO); break;
    }
    return 0;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret;
            tu_case(op, k);
            F.ground_or_air = rnd() & 1;
            F.x618_player_id = rnd() % 4;
            F.input.lstick[1].x = rg();
            F.input.lstick[0].x = rg();
            CD.x210 = rg();
            CD.x214 = rnd() % 10;
            F.active_timer.lstick.x = rnd() % 12;
            ret = run(op);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
