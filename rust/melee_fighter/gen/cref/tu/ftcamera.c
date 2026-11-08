/* TU: src/melee/ft/ftcamera.c */
/* C oracle for src/tu/ftcamera.rs: TU included verbatim, stage getters and ftLib_GetCameraBonePos are
 * stubs returning values drawn from SRV (mirrored by tests/tu_ftcamera.rs). */
#include TU_SRC
#include "tu_common.h"

static float ZOOM, BT, CT, TC;
float Stage_GetCamFixedZoom(void) { T(1); return ZOOM; }
float Stage_GetBlastZoneTopOffset(void) { T(2); return BT; }
float Stage_GetCamBoundsTopOffset(void) { T(3); return CT; }
void Stage_UnkSetVec3TCam_Offset(Vec3* v) { T(4); v->x = 0; v->y = TC; v->z = 0; }
void ftLib_GetCameraBonePos(HSD_GObj* g, Vec3* v) { T(5); T(g == &GO); v->x = TC; v->y = ZOOM; v->z = BT; }

static CmSubject CB;
static UnkFloat6_Camera IN, OUT;
static ftData FD;

static void hv(Vec3* v) { HM(B(v->x)); HM(B(v->y)); HM(B(v->z)); }
static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(CB.state); HM(CB.on_ledge); hv(&CB.pos); hv(&CB.bone_pos); HM(B(CB.facing_dir));
    HM(B(CB.ext.h.x)); HM(B(CB.ext.h.y)); hv(&CB.ext.v);
    HM(B(CB.target_ext.h.x)); HM(B(CB.target_ext.h.y)); hv(&CB.target_ext.v);
    hv(&OUT.x0); hv(&OUT.xC);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 1000; k++) {
            tu_case(op, k);
            zero(&CB, sizeof CB); zero(&OUT, sizeof OUT); zero(&FD, sizeof FD);
            IN.x0.x = rw(); IN.x0.y = rw(); IN.x0.z = rw();
            IN.xC.x = rw(); IN.xC.y = rw(); IN.xC.z = rw();
            ZOOM = rw(); BT = rw(); CT = rw(); TC = rg();
            F.x34_scale.y = rw();
            F.facing_dir = (rnd() & 1) ? 1.0f : rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rw();
            CB.pos.x = rw(); CB.state = rnd() & 3; CB.on_ledge = rnd() & 1;
            FD.x3C = &IN; F.ft_data = &FD; F.x890_cameraBox = &CB;
            switch (op) {
            case 0: ftCamera_80076018(&IN, &OUT, F.x34_scale.y); break;
            case 1: ftCamera_80076064(&F); break;
            case 2: ftCamera_UpdateCameraBox(&GO); break;
            case 3: ftCamera_800762F4(&GO); break;
            case 4: if (BT - TC != 0.0f) ftCamera_80076320(&GO); break;
            }
            printf("%d %d %08X %08X\n", op, k, state_hash(), TR);
        }
    }
    return 0;
}
