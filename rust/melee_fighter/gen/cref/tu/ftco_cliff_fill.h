/* Shared fill, state hash and recording stubs for the ftCo_Cliff*.c TU drivers (included after
 * tu_common.h). Stubs here are callees outside all five Cliff TUs. Mirrored by
 * tests/tu_ftco_cliff/mod.rs. */
static float ANG[2];

static void cliff_fill(void)
{
    ANG[0] = rg() * 2;
    ANG[1] = rg() * 2;
    F.input.pressed_buttons = rnd() & 0xFFFF;
    F.input.lstick[0].x = rg();
    F.input.lstick[0].y = rg();
    F.input.cstick[0].x = rg();
    F.dmg.x1830_percent = rw();
    CD.x488 = (int) (rnd() % 200) - 50;
    CD.x48C = rw();
    CD.x490 = rw();
    CD.x494 = rg();
    CD.x20_radians = rg();
    CD.ledge_cooldown = rnd() % 100;
    CD.x49C = rnd() % 100;
    F.facing_dir = rg();
    F.mv.co.cliff.ledge_id = rnd() % 4;
    F.mv.co.cliff.x4 = rw();
    F.mv.co.cliff.x8 = rnd() & 1;
    F.ground_or_air = rnd() & 1;
    F.x68C_transNPos.x = rg();
    F.x68C_transNPos.y = rg();
    F.x68C_transNPos.z = rg();
    F.self_vel.x = rw();
    F.self_vel.y = rw();
    F.co_attrs.ledge_jump_horizontal_velocity = rg();
    F.co_attrs.ledge_jump_vertical_velocity = rg();
    F.motion_id = (rnd() & 1) ? ftCo_MS_CliffJumpQuick1 : ftCo_MS_CliffJumpSlow1;
    F.cur_pos.x = rw();
    F.cur_pos.y = rw();
    F.coll_data.floor.index = rnd() % 64;
    F.x2064_ledgeCooldown = rnd() % 9;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.cur_pos.x));
    HM(B(F.cur_pos.y));
    HM(F.coll_data.floor.index);
    HM(F.x2064_ledgeCooldown);
    HM(F.x221D_b5);
    HM(F.x221D_b7);
    HM(F.x2227_b1);
    HM(F.mv.co.cliff.ledge_id);
    HM(B(F.mv.co.cliff.x4));
    HM(F.mv.co.cliff.x8);
    HM(B(F.self_vel.x));
    HM(B(F.self_vel.y));
    return H;
}

float ftCo_GetLStickAngle(Fighter* f) { T(1); T(f == &F); return ANG[0]; }
float ftCo_GetCStickAngle(Fighter* f) { T(2); T(f == &F); return ANG[1]; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(3); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(5); T(f == &F); T((u32) (int) v); }
void ftCo_CliffCatch_Phys(Fighter_GObj* g) { T(6); T(g == &GO); }
void ftCo_CliffCatch_Coll(Fighter_GObj* g) { T(7); T(g == &GO); }
void ftCommon_8007D7FC(Fighter* f) { T(8); T(f == &F); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(9); T(g == &GO); }
bool ftCo_800DF6F8(Fighter* f) { T(10); T(f == &F); return SR(2); }
bool ftCo_800DF72C(Fighter* f) { T(11); T(f == &F); return SR(3); }
bool ftCo_800DF79C(Fighter* f) { T(12); T(f == &F); return SR(4); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(13); T(g == &GO); }
bool mpLib_80054ED8(int l) { T(14); T(l); return SR(5); }
void mpLib_80053ECC_Floor(int l, Vec3* v) { T(15); T(l); v->x = rg(); v->y = rg(); v->z = rg(); }
void mpLib_80053DA4_Floor(int l, Vec3* v) { T(16); T(l); v->x = rg(); v->y = rg(); v->z = rg(); }
void ft_80084FA8(Fighter_GObj* g) { T(17); T(g == &GO); }
bool ft_800821DC(Fighter_GObj* g) { T(18); T(g == &GO); return SR(6); }
bool ftCo_8009EF68(Fighter_GObj* g) { T(19); T(g == &GO); return SR(7); }
void ft_80084104(Fighter_GObj* g) { T(20); T(g == &GO); }
ftCo_JumpInput ftCo_Jump_GetInput(Fighter_GObj* g) { T(21); T(g == &GO); return SRV[8] % 3; }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(22); T(g == &GO); return SR(9); }
void ft_80084DB0(Fighter_GObj* g) { T(23); T(g == &GO); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(25); (void) g; (void) l; return 0; }
void ft_80082B1C(Fighter_GObj* g) { T(26); (void) g; }
void ft_800835B0(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(24); T(g == &GO); T(a == ftCo_80096CC8); T(b == ft_80082B1C); }
void ftColl_8007B760(Fighter_GObj* g, int v) { T(27); T(g == &GO); T(v); }
void ftCo_80090780(Fighter_GObj* g) { T(28); T(g == &GO); }

#define CLIFF_MAIN(NOPS)                                                             \
    int main(void)                                                                   \
    {                                                                                \
        int op, k;                                                                   \
        tu_init();                                                                   \
        for (op = 0; op < NOPS; op++) {                                              \
            for (k = 0; k < 2000; k++) {                                             \
                u32 ret;                                                             \
                tu_case(op, k);                                                      \
                cliff_fill();                                                        \
                ret = run(op);                                                       \
                printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);      \
            }                                                                        \
        }                                                                            \
        return 0;                                                                    \
    }
