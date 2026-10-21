/* TU: src/melee/ft/ftcliffcommon.c */
/* C oracle for src/tu/ftcliffcommon.rs (mirrored by tests/tu_ftcliffcommon.rs): CliffCatch
 * Phys / Coll / Anim with recording stubs for the map and state callees. */
#include TU_SRC
#include "tu_common.h"
#include "ftcliffcommon_traps.h"

static Vec3 LV;
bool mpLib_80054ED8(int id) { T(1); T(id); return SR(0); }
void mpLib_80053ECC_Floor(int id, Vec3* v) { T(2); T(id); *v = LV; }
void mpLib_80053DA4_Floor(int id, Vec3* v) { T(3); T(id); LV.x += 1.0f; *v = LV; }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(4); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(5); return SR(1); }
void ftCo_8009A804(Fighter_GObj* g) { T(6); }
bool ft_800821DC(Fighter_GObj* g) { T(7); return SR(2); }
void ft_80082B1C(Fighter_GObj* g) { T(8); }
bool ftCo_8009EF68(Fighter_GObj* g) { T(9); return SR(3); }

static void hv(Vec3* v) { HM(B(v->x)); HM(B(v->y)); HM(B(v->z)); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 2000; k++) {
            tu_case(op, k);
            LV.x = rw(); LV.y = rw(); LV.z = rw();
            F.x68C_transNPos.x = rw(); F.x68C_transNPos.y = rw(); F.x68C_transNPos.z = rw();
            F.facing_dir = rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rw();
            F.mv.co.cliff.ledge_id = rnd() % 100;
            CD.ledge_cooldown = rnd() % 60;
            F.x2064_ledgeCooldown = rnd() % 60;
            switch (op) {
            case 0: ftCo_CliffCatch_Phys(&GO); break;
            case 1: ftCo_CliffCatch_Coll(&GO); break;
            case 2: ftCo_CliffCatch_Anim(&GO); break;
            case 3: ftCo_CliffCatch_IASA(&GO); break;
            }
            H = 0x811C9DC5u;
            hv(&F.cur_pos); HM(F.x2064_ledgeCooldown);
            printf("%d %d %08X %08X\n", op, k, H, TR);
        }
    }
    return 0;
}
