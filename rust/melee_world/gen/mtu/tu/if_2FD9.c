/* TU: src/melee/if/if_2FD9.c */
/* C oracle for src/tu/if_2FD9.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G[4], NG[4];
static HSD_JObj RJ, CJ, NJ[4];
static HSD_DObj D[3];
static HSD_MObj M[3];
static HSD_Material MA[3];
static HSD_GObjProc PROC0;
static HSD_Archive* ARCP;
static char ARCH[16];
static DynamicModelDesc DMD;
static DynamicModelDesc* DMDA[1];
#include "if_2FD9_gen.h"

static void setup(void)
{
    int i;
    zero(G, sizeof G); zero(NG, sizeof NG); zero(&RJ, sizeof RJ); zero(&CJ, sizeof CJ); zero(NJ, sizeof NJ);
    zero(D, sizeof D); zero(M, sizeof M); zero(MA, sizeof MA); zero(&PROC0, sizeof PROC0); zero(&DMD, sizeof DMD);
    for (i = 0; i < 4; i++) { fill_HSD_GObj(&G[i]); fill_HSD_GObj(&NG[i]); G[i].hsd_obj = &RJ; G[i].user_data = (void*) &un_804DDBA8[i]; }
    RJ.flags = rnd() & ~(u32) JOBJ_USE_QUATERNION;
    RJ.rotate.x = rw(); RJ.rotate.y = rw(); RJ.rotate.z = rw(); RJ.rotate.w = rw();
    RJ.translate.x = rw(); RJ.translate.y = rw(); RJ.translate.z = rw();
    RJ.child = &CJ;
    CJ.u.dobj = &D[0];
    for (i = 0; i < 3; i++) {
        D[i].next = (i < 2) ? &D[i + 1] : 0;
        D[i].mobj = &M[i];
        M[i].mat = &MA[i];
        MA[i].diffuse.r = (u8) rnd(); MA[i].diffuse.g = (u8) rnd(); MA[i].diffuse.b = (u8) rnd(); MA[i].diffuse.a = (u8) rnd();
    }
    DMDA[0] = &DMD;
    DMD.joint = (HSD_Joint*) ARCH;
    ARCP = (rnd() & 1) ? (HSD_Archive*) ARCH : 0;
    un_804D6D90 = (int) (rnd() & 1);
    zero(&un_804A1F10, sizeof un_804A1F10);
    un_804A1F10.x0 = DMDA;
    for (i = 0; i < 4; i++) {
        un_804A1F10.x4[i] = (rnd() % 3 == 0) ? 0 : &G[rnd() % 4];
        un_804A1F10.x14[i].r = (u8) rnd(); un_804A1F10.x14[i].g = (u8) rnd(); un_804A1F10.x14[i].b = (u8) rnd(); un_804A1F10.x14[i].a = (u8) rnd();
        un_804A1F10.x24[i] = (u8) (rnd() & 1);
        un_804A1F10.x28[i] = (u8) ((rnd() % 3 == 0) ? 0 : (rnd() % 4));
        un_804A1F10.x2C[i] = (u8) (rnd() % 4);
    }
    un_804A1F10.x30 = (u8) (rnd() & 1);
    un_804A1F10.x31 = (s8) ((rnd() % 3 == 0) ? -1 : (int) (rnd() % 4));
    un_804A1F10.x32 = (u8) (rnd() % 51);
    HSD_GObj_JObjKind = (u8) (rnd() % 4);
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 4; i++) { hash_HSD_GObj(&G[i]); hash_HSD_GObj(&NG[i]); }
    HM(RJ.flags); HM(B(RJ.rotate.x)); HM(B(RJ.rotate.y)); HM(B(RJ.rotate.z)); HM(B(RJ.rotate.w));
    HM(B(RJ.translate.x)); HM(B(RJ.translate.y)); HM(B(RJ.translate.z));
    for (i = 0; i < 3; i++) HM(RH(&MA[i].diffuse, 4));
    HM((u32) un_804D6D90);
    HM(PI(un_804A1F10.x0));
    for (i = 0; i < 4; i++) {
        HM(PI(un_804A1F10.x4[i])); HM(RH(&un_804A1F10.x14[i], 4));
        HM(un_804A1F10.x24[i]); HM(un_804A1F10.x28[i]); HM(un_804A1F10.x2C[i]);
    }
    HM(un_804A1F10.x30); HM((u32) (int) un_804A1F10.x31); HM(un_804A1F10.x32);
}

int main(void)
{
    int op, k, i;
    for (i = 0; i < 4; i++) { preg(&G[i], sizeof(HSD_GObj)); preg(&NG[i], sizeof(HSD_GObj)); preg(&NJ[i], sizeof(HSD_JObj)); }
    preg(&RJ, sizeof RJ); preg(&CJ, sizeof CJ); preg(D, sizeof D); preg(M, sizeof M); preg(MA, sizeof MA);
    preg(&PROC0, sizeof PROC0); preg(&ARCP, sizeof ARCP); preg(ARCH, sizeof ARCH); preg(&DMD, sizeof DMD); preg(DMDA, sizeof DMDA);
    preg(un_804DDBA8, sizeof un_804DDBA8); preg(&un_804A1F10, sizeof un_804A1F10);
    freg_all();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: un_802FD910(); break;
            case 1: un_802FD91C(); break;
            case 2: {
                u8 slot = (u8) (rnd() % 6);
                u8 a1 = (u8) rnd();
                GXColor c;
                c.r = (u8) rnd(); c.g = (u8) rnd(); c.b = (u8) rnd(); c.a = (u8) rnd();
                un_802FD928(slot, a1, &c);
                break;
            }
            case 3: un_802FD9D8((u8) (rnd() % 6)); break;
            case 4: { int g = (int) (rnd() % 4); int pass = (int) fv(); fn_802FDA4C(&G[g], pass); break; }
            case 5: fn_802FDA78(&G[rnd() % 4]); break;
            case 6: un_802FE260(); break;
            case 7: un_802FE390(); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
