/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemScopeStart.c */
/* C oracle for src/tu/ftCo_ItemScopeStart.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace.
 *
 * The TU reads `fp + 0x2340/0x2344` through overlay structs (GameCube offsets = mv words 0/1).
 * Compiled natively that is byte offset 0x2340 of the native Fighter, so this driver seeds and
 * hashes those native bytes; the Rust side seeds and hashes mv words 0/1 with the same values. */
#include TU_SRC
#include "tu_common.h"

#define OV_F(o) (*(f32*) ((u8*) &F + (o)))
#define OV_I(o) (*(s32*) ((u8*) &F + (o)))

bool it_8026B594(Item_GObj* g) { T(1); T(g == &IGO); return SR(0); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(3); }
void ftCommon_ClampAirDrift(Fighter* f) { T(4); }
void ftCommon_8007E79C(Fighter_GObj* g, s32 a) { T(5); T(g == &GO); T(a); }
void ftCommon_8007E7E4(Fighter_GObj* g, s32 a) { T(6); T(g == &GO); T(a); }
void ftCommon_8007D7FC(Fighter* f) { T(7); }
void ftCommon_8007D5D4(Fighter* f) { T(8); }
int it_80291DAC(Item_GObj* g, int n) { T(9); T(n); return (n & 3) - 1; }
void it_80291F14(Item_GObj* g, int n) { T(10); T(n); }
s32 it_80291CF4(Item_GObj* g, s32 n) { T(11); T(n); return SR(1); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(12); return SR(2); }
void fn_800D7BDC(Fighter_GObj* g) { T(13); }
void fn_800D7C60(Fighter_GObj* g) { T(14); }
void fn_800D8140(Fighter_GObj* g, int n) { T(15); T(n); }
void fn_800D81D0(Fighter_GObj* g, int n) { T(16); T(n); }
void ftCo_800CDE54(Fighter_GObj* g) { T(17); }
void ftCo_800CDE74(Fighter_GObj* g) { T(18); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(19); T(cb == fn_800D78B0); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(20); T(cb == fn_800D7830); cb(g); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == (HSD_GObjEvent) fn_800D7938 ? 1 : 9);
    HM(B(OV_F(0x2340))); HM(OV_I(0x2344));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.gobj = &GO;
            F.item_gobj = (rnd() % 8) ? &IGO : NULL;
            F.motion_id = 158 + rnd() % 16;
            F.input.held_buttons[0] = rnd() & 0x100;
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw();
            OV_F(0x2340) = rg() * 20.0f; OV_I(0x2344) = rnd() & 1;
            CD.x5B8 = rg() * 20.0f;
            switch (op) {
            case 0: { int m = rnd() % 400; ret = fn_800D769C(&F, m); break; }
            case 1: ft_800D76B8(&GO); break;
            case 2: ft_800D7770(&GO); break;
            case 3: fn_800D7830(&GO); break;
            case 4: fn_800D78B0(&GO); break;
            /* fn_800D7938 reads the file-level FighterOverlay, whose native layout puts x2340 at
             * native +0x2344 (8-byte x1A58): mirror the timer into both words for this op. */
            case 5: OV_I(0x2344) = OV_I(0x2340); fn_800D7938(&GO); break;
            case 6: ftCo_ItemScopeStart_Anim(&GO); break;
            case 7: ftCo_ItemScopeAirStart_Anim(&GO); break;
            case 8: ftCo_ItemScopeStart_IASA(&GO); break;
            case 9: ftCo_ItemScopeAirStart_IASA(&GO); break;
            case 10: ftCo_ItemScopeStart_Phys(&GO); break;
            case 11: ftCo_ItemScopeAirStart_Phys(&GO); break;
            case 12: ftCo_ItemScopeStart_Coll(&GO); break;
            case 13: ftCo_ItemScopeAirStart_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
