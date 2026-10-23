/* TU: src/melee/if/if_2FF2.c */
/* C oracle for src/tu/if_2FF2.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (memzero is real). The descriptor tables are checked by value through the
 * HSD_CObjLoadDesc / HSD_LObjLoadDesc stub effects. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
void memzero(void* p, int n)
{
    unsigned char* a = p;
    int i;
    for (i = 0; i < n; i++) a[i] = 0;
}
static HSD_GObj G[6], NG[4], HG;
static HSD_Text TX[3];
static Vec3 HPOS[6];
static HSD_LObj LO;
static HSD_CObj CO;
static HSD_GObjProc PROC0;
static MatchEnd ME;
#include "if_2FF2_gen.h"

static void setup(void)
{
    int i;
    zero(G, sizeof G); zero(NG, sizeof NG); zero(&HG, sizeof HG); zero(TX, sizeof TX); zero(&PROC0, sizeof PROC0);
    for (i = 0; i < 6; i++) fill_HSD_GObj(&G[i]);
    for (i = 0; i < 4; i++) fill_HSD_GObj(&NG[i]);
    fill_HSD_GObj(&HG);
    for (i = 0; i < 3; i++) fill_HSD_Text(&TX[i]);
    for (i = 0; i < 6; i++) { HPOS[i].x = rw(); HPOS[i].y = rw(); HPOS[i].z = rw(); }
    zero(&un_804A1F58, sizeof un_804A1F58);
    un_804A1F58.x0 = fv();
    for (i = 0; i < 6; i++) {
        un_804A1F58.x8[i].x0 = (rnd() % 3 == 0) ? 0 : &G[rnd() % 6];
        un_804A1F58.x8[i].x4 = (rnd() % 3 == 0) ? 0 : &TX[rnd() % 3];
        un_804A1F58.x8[i].x8 = fv();
        un_804A1F58.x8[i].xC = fv();
        un_804A1F58.x8[i].x10 = (u8) (rnd() % 3);
    }
    HSD_GObj_CameraKind = (u8) (rnd() % 5);
    HSD_GObj_LightKind = (s8) (rnd() % 5);
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 6; i++) hash_HSD_GObj(&G[i]);
    for (i = 0; i < 4; i++) hash_HSD_GObj(&NG[i]);
    hash_HSD_GObj(&HG);
    for (i = 0; i < 3; i++) hash_HSD_Text(&TX[i]);
    HM(un_804A1F58.x0);
    for (i = 0; i < 6; i++) {
        HM(PI(un_804A1F58.x8[i].x0)); HM(PI(un_804A1F58.x8[i].x4));
        HM(un_804A1F58.x8[i].x8); HM(un_804A1F58.x8[i].xC); HM(un_804A1F58.x8[i].x10);
    }
}

int main(void)
{
    int op, k, i;
    for (i = 0; i < 6; i++) preg(&G[i], sizeof(HSD_GObj));
    for (i = 0; i < 4; i++) preg(&NG[i], sizeof(HSD_GObj));
    preg(&HG, sizeof HG); preg(TX, sizeof TX); preg(HPOS, sizeof HPOS); preg(&LO, sizeof LO); preg(&CO, sizeof CO);
    preg(&PROC0, sizeof PROC0); preg(&ME, sizeof ME);
    freg_all();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: fn_802FF218((rnd() % 4 == 0) ? &NG[0] : &G[rnd() % 6]); break;
            case 1: fn_802FF360(0); break;
            case 2: un_802FF364((int) (rnd() % 6)); break;
            case 3: un_802FF498(); break;
            case 4: un_802FF4FC(); break;
            case 5: un_802FF570(); break;
            case 6: un_802FF620(); break;
            case 7: un_802FF6A0(); break;
            case 8: un_802FF710(); break;
            case 9: ret = PI(un_802FF78C()); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
