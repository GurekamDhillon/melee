/* TU: src/melee/ft/ftcolanim.c */
/* C oracle for src/tu/ftcolanim.rs (mirrored by tests/tu_ftcolanim.rs): every function over
 * random flags with recording stubs for the colanim interpreter (lb_013B), ftAction, Mex and
 * the PlCo tables. Each line: op case hash trace. */
#include TU_SRC
#include "tu_common.h"

Fighter_804D653C_t* Fighter_804D653C;
Fighter_804D653C_t* Fighter_804D6538;
static Fighter_804D653C_t TBL[0x7B], SPY[8];
static int VAN;
static CmSubject CB;
static int LOOPS;

static u32 which(ColorOverlay* co)
{
    if (co == &F.x408) return 1;
    if (co == &F.x488) return 2;
    if (co == &F.x508) return 3;
    return 9;
}
static u32 tbl(Fighter_804D653C_t* t) { return t == TBL ? 1 : t == SPY ? 2 : 9; }

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId msid, MotionFlags fl, f32 a, f32 b, f32 c, Fighter_GObj* o)
{ T(1); T(msid); T(fl); T(B(a)); T(B(b)); T(B(c)); T(g == &GO); T(o == NULL); }
HSD_GObj* Player_GetEntityAtIndex(s32 slot, s32 i) { T(2); T(slot); T(i); return SR(0) ? &GO2 : NULL; }
void ftCo_800D4F24(Fighter_GObj* g, int i) { T(3); T(g == &GO2); T(i); }
void gm_80167320(s32 slot, bool sub) { T(4); T(slot); T(sub); }
void lb_80014498(ColorOverlay* co) { T(5); T(which(co)); }
bool lb_800144C8(ColorOverlay* co, Fighter_804D653C_t* t, int idx, int b)
{ T(6); T(which(co)); T(tbl(t)); T(idx); T(b); return SR(1); }
void ftAction_80071028(Fighter_GObj* g, CommandInfo* c) { T(10); }
void ftAction_800711DC(Fighter_GObj* g, CommandInfo* c) { T(11); }
void ftAction_80071B50(Fighter_GObj* g, CommandInfo* c) { T(12); }
void ftAction_80071CA4(Fighter_GObj* g, CommandInfo* c) { T(13); }
void ftAction_800730B8(Fighter_GObj* g, CommandInfo* c) { T(14); }
void ftAction_80073108(Fighter_GObj* g, CommandInfo* c) { T(15); }
bool ftCo_800C53E4(Fighter* fp) { T(16); T(fp == &F); return SR(2); }
void Mex_OnActionStateChangeDispatch(int kind, void* gobj, void* vanilla)
{ T(17); T(kind); T(gobj == &GO); T(vanilla == (void*) &VAN); }
HSD_GObjEvent ftData_UnkMotionStates4[Ft_Kind_Max];
bool lb_80014258(Fighter_GObj* g, void* co_, FtCmd2 cb)
{
    ColorOverlay* co = co_;
    T(7); T(which(co)); T(cb == ft_800BFF34 ? 1 : cb == ft_800BFF70 ? 2 : 9);
    if (rnd() & 1) cb(g, NULL, 21 + (int) (rnd() % 3));
    if (LOOPS > 0 && (rnd() & 1)) { LOOPS--; return true; }
    return false;
}

int main(void)
{
    int op, k, i;
    tu_init();
    Fighter_804D653C = TBL;
    Fighter_804D6538 = SPY;
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 1000; k++) {
            tu_case(op, k);
            for (i = 0; i < 0x7B; i++) TBL[i].unk5 = rnd() & 1;
            for (i = 0; i < Ft_Kind_Max; i++) ftData_UnkMotionStates4[i] = NULL;
            F.kind = rnd() % Ft_Kind_Max;
            ftData_UnkMotionStates4[F.kind] = (HSD_GObjEvent) (void*) &VAN;
            F.gobj = &GO;
            F.player_id = rnd() % 6;
            F.is_sub_fighter = rnd() & 1;
            F.x2222_b5 = rnd() & 1;
            F.x2226_b4 = rnd() & 1;
            F.x2224_b2 = rnd() & 1;
            F.x221D_b6 = rnd() & 1;
            F.x2221_b0 = rnd() & 1;
            F.dmg.x18F0 = rnd() % 2;
            F.x1990 = rnd() % 2;
            F.x1994 = rnd() % 2;
            F.x408.x0_timer = rnd();
            F.x488.x0_timer = rnd();
            F.x408.x28_colanim.i = rnd() % 2;
            F2.x408.x0_timer = rnd();
            F2.x488.x0_timer = rnd();
            CB.state = 0;
            F.x890_cameraBox = &CB;
            LOOPS = (int) (rnd() % 4);
            switch (op) {
            case 0: ftCo_800BFD04(&GO); break;
            case 1: ftCo_800BFD9C(&GO); break;
            case 2: T(ftCo_800BFFD0(&F, (int) (rnd() % 0x80), rnd() & 1)); break;
            case 3: ft_800C0098(&F); break;
            case 4: ftCo_800C0134(&F); break;
            case 5: ftCo_800C0200(&F, (int) (rnd() % 0x7B)); break;
            case 6: ftCo_800C0358(&F, &F2, (int) (rnd() % 0x7B)); break;
            case 7: ftCo_800C0408(&GO); break;
            case 8: ftCo_800BFFAC(&F); ftCo_800C0074(&F); ftCo_Sleep_Anim(&GO); ftCo_Sleep_IASA(&GO); break;
            }
            H = 0x811C9DC5u;
            HM(F.invisible); HM(F.x221E_b1); HM(F.x221E_b2); HM(F.x2219_b1); HM(F.x221F_b3);
            HM(F.x221F_b1); HM(F.x2221_b3); HM(CB.state); HM(F2.x408.x0_timer); HM(F2.x488.x0_timer);
            printf("%d %d %08X %08X\n", op, k, H, TR);
        }
    }
    return 0;
}
