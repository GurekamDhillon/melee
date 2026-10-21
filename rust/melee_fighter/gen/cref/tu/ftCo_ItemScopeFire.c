/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemScopeFire.c */
/* C oracle for src/tu/ftCo_ItemScopeFire.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub (efSync_Spawn is variadic: the stub traces the optional facing pointer
 * through the 4th arg only when the C passes it; the id tells which form). */
#include TU_SRC
#include "tu_common.h"
#include <stdarg.h>

FtMotionId fn_800D769C(Fighter* f, FtMotionId m) { T(1); T(f == &F); T(m); return SR(0) ? m + 8 : m; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(3); }
void ftCommon_8007D7FC(Fighter* f) { T(4); }
void ftCommon_8007D5D4(Fighter* f) { T(5); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(6); T(g == &GO); }
void ftCommon_8007E7E4(Fighter_GObj* g, s32 a) { T(7); T(g == &GO); T(a); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(8); return SR(1); }
void ftCo_800CDE54(Fighter_GObj* g) { T(9); }
void ftCo_800CDE74(Fighter_GObj* g) { T(10); }
void ftCo_800CDE94(Fighter* f, Vec3* in, Vec3* out)
{ T(11); T(f == &F); T(B(in->x)); T(B(in->y)); T(B(in->z)); out->x = in->x * 2.0f; out->y = in->y - 1.0f; out->z = in->z + 3.0f; }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(12); T(cb == fn_800D82EC); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(13); T(cb == fn_800D8268); cb(g); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 v, u8 p) { T(14); T(f == &F); T(id); T(v); T(p); }
void it_80291CCC(Item_GObj* g, Vec3* pos) { T(15); T(g == &IGO); pos->x = rg(); pos->y = rg(); pos->z = rg(); }
void it_80291FA8(Item_GObj* g, Vec3* pos, int n, float facing)
{ T(16); T(g == &IGO); T(B(pos->x)); T(B(pos->y)); T(B(pos->z)); T(n); T(B(facing)); pos->x += 5.0f; pos->z = rg(); }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    va_list ap;
    va_start(ap, g);
    T(17); T(id); T(g == &GO);
    {
        Vec3* p = va_arg(ap, Vec3*);
        T(B(p->x)); T(B(p->y)); T(B(p->z));
        if (id == 0x430) {
            float* fd = va_arg(ap, float*);
            T(fd == &F.facing_dir);
        }
    }
    va_end(ap);
    return NULL;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.accessory4_cb == NULL ? 0 : F.accessory4_cb == fn_800D86B8 ? 1 : 9);
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == fn_800D8378 ? 2 : 9);
    HM(F.mv.co.common.x0);
    HM(F.throw_flags_b0); HM(F.throw_flags_b1); HM(F.throw_flags_b2); HM(F.throw_flags_b3);
    HM(F.throw_flags_b4); HM(F.throw_flags_b5); HM(F.throw_flags_b6); HM(F.throw_flags_b7);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.gobj = &GO;
            F.item_gobj = (rnd() % 8) ? &IGO : NULL;
            F.motion_id = 158 + rnd() % 16;
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw();
            F.facing_dir = rg();
            F.throw_flags = rnd() & 0xFF;
            F.mv.co.common.x0 = (int) (rnd() % 14) - 2;
            switch (op) {
            case 0: fn_800D8140(&GO, (int) (rnd() % 14) - 2); break;
            case 1: fn_800D81D0(&GO, (int) (rnd() % 14) - 2); break;
            case 2: fn_800D8268(&GO); break;
            case 3: fn_800D82EC(&GO); break;
            case 4: fn_800D8378(&GO); break;
            case 5: ftCo_ItemScopeFire_Anim(&GO); break;
            case 6: ftCo_ItemScopeAirFire_Anim(&GO); break;
            case 7: ftCo_ItemScopeFire_IASA(&GO); break;
            case 8: ftCo_ItemScopeAirFire_IASA(&GO); break;
            case 9: ftCo_ItemScopeFire_Phys(&GO); break;
            case 10: ftCo_ItemScopeAirFire_Phys(&GO); break;
            case 11: ftCo_ItemScopeFire_Coll(&GO); break;
            case 12: ftCo_ItemScopeAirFire_Coll(&GO); break;
            case 13: fn_800D84D4(&GO, F.mv.co.common.x0); break;
            case 14: fn_800D86B8(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
