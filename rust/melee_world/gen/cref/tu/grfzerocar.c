/* TU: src/melee/gr/grfzerocar.c */
/* C oracle for src/tu/grfzerocar.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G;
static HSD_JObj JR[2];
static HSD_JObj JP[64];
static int JN;
static UnkArchiveStruct ARCH;
static UnkStageDat USD;
static unsigned char DMB[4 * 0x34 + 64] __attribute__((aligned(16)));
static s16 CDATA[30];
#include "grfzerocar_gen.h"

int main(void)
{
    int op, k, i;
    preg(&G, sizeof G);
    preg(JR, sizeof JR);
    preg(JP, sizeof JP);
    preg(&ARCH, sizeof ARCH);
    preg(&USD, sizeof USD);
    preg(DMB, sizeof DMB);
    preg(CDATA, sizeof CDATA);
    freg_all();
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&G, sizeof G);
            zero(JR, sizeof JR);
            zero(JP, sizeof JP);
            zero(DMB, sizeof DMB);
            fill_HSD_GObj(&G);
            for (i = 0; i < 2; i++) fill_HSD_JObj(&JR[i]);
            for (i = 0; i < 64; i++) fill_HSD_JObj(&JP[i]);
            JN = 0;
            JR[0].child = &JR[1];
            G.hsd_obj = &JR[0];
            for (i = 0; i < 30; i++) CDATA[i] = (s16) (100 + fv() % 100);
            ARCH.unk0 = 0;
            ARCH.unk4 = &USD;
            USD.unk8 = (void*) DMB;
            for (i = 0; i < 4; i += 2) {
                DynamicModelDesc* d = (DynamicModelDesc*) (DMB + i * 0x34);
                d->anims = (fv() & 1) ? (HSD_AnimJoint**) (DMB + 8) : 0;
                d->matanims = (fv() & 1) ? (HSD_MatAnimJoint**) (DMB + 16) : 0;
            }
            *(void**) (DMB + 8) = DMB + 24;
            *(void**) (DMB + 16) = DMB + 32;
            switch (op) {
            case 0: grFZeroCar_801CAFBC(&G, CDATA, (int) ((fv() % 2) * 2), (int) (fv() & 1)); break;
            case 1:
                for (i = 0; i < 30; i++) {
                    HM((u32) grFZeroCar_803E0BD8[i].unk0); HM((u32) grFZeroCar_803E0BD8[i].unk2);
                    HM((u32) grFZeroCar_803E0BD8[i].children[0]); HM((u32) grFZeroCar_803E0BD8[i].children[1]);
                    HM((u32) grFZeroCar_803E0BD8[i].children[2]); HM((u32) grFZeroCar_803E0BD8[i].children[3]);
                }
                HM(B(grFZeroCar_803B7E50.scale.x)); HM(B(grFZeroCar_803B7E50.scale.y)); HM(B(grFZeroCar_803B7E50.scale.z));
                HM(B(grFZeroCar_803B7E50.translate.x)); HM(B(grFZeroCar_803B7E50.translate.y)); HM(B(grFZeroCar_803B7E50.translate.z));
                HM(B(grFZeroCar_803B7E50.rotate.x)); HM(B(grFZeroCar_803B7E50.rotate.y));
                HM(B(grFZeroCar_803B7E50.rotate.z)); HM(B(grFZeroCar_803B7E50.rotate.w));
                HM(RH(grFZeroCar_804D4598, 8));
                break;
            }
            hash_HSD_GObj(&G);
            for (i = 0; i < 2; i++) hash_HSD_JObj(&JR[i]);
            for (i = 0; i < 64; i++) hash_HSD_JObj(&JP[i]);
            HM((u32) JN);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
