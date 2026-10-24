/* EXTRACT: src/melee/it/it_26B1.c itGetKind it_8026B294 it_8026B718 it_8026C1E8 it_8026B3C0 it_ResetStageArticleTable it_8026B40C */
/* EXTRACT: src/melee/it/ithitbox.c it_80275414 it_802756D0 it_802756E0 */
/* EXTRACT: src/melee/it/itcoll.c it_802714C0 */
/* EXTRACT: src/melee/it/it_2725.c it_80274C60 */
/* EXTRACT: src/melee/it/kinds/itcoin.c it_802F2014 it_802F2020 */
/* EXTRACT: src/melee/it/kinds/ittincle.c it_802ECC8C it_802ECC98 it_802ECCA4 */
/* EXTRACT: src/melee/it/kinds/itkyasarin.c it_802ECD1C */
/* State oracle for melee_world::itacc: the verbatim C definitions (extract_itacc.h) over a random Item.
 * One line per case: op case hash (the whole Item record, the outputs, the article table). */
#include <melee/it/forward.h>
#include <melee/it/types.h>
#include <melee/it/item.h>
#include <melee/it/inlines.h>
#include <melee/it/itcoll.h>
#include <melee/it/ithitbox.h>
#include <sysdolphin/baselib/gobj.h>
#include "common.h"
extern int printf(const char*, ...);
extern void* calloc(unsigned, unsigned);

Article* it_804A0F60[30];
HSD_GObj** HSD_GObjPLinkHead;
int MEXCALLS;
void Mex_IndexStageItem(void* article, int idx) { (void) article; MEXCALLS += 1 + idx; }
#include "extract_itacc.h"

static u64_ H;
static void hw(unsigned w) { H = fnv(&w, 4, H); }

int main(void)
{
    int op, k, i;
    static HSD_GObj G[6]; static HSD_GObj* heads[64]; static Item I[6]; static Article A[3];
    HSD_GObjPLinkHead = heads;
    printf("sizeof_Item %u\n", (unsigned) sizeof(Item));
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 200; k++) {
            unsigned char* p; unsigned j; Vec pos; int kind = -1; int ret = 0;
            H = FNV0;
            for (i = 0; i < 6; i++) {
                p = (unsigned char*) &I[i];
                for (j = 0; j < sizeof(Item); j++) p[j] = (unsigned char) rnd();
                I[i].grab_victim = (rnd() & 1) ? (HSD_GObj*) 0 : (HSD_GObj*) 0x1000;
                I[i].owner = (rnd() & 1) ? (HSD_GObj*) 0 : (HSD_GObj*) 0x2000;
                I[i].xAC8_hurtboxNum = rnd() % 3;
                /* bitfields are set logically: the native C and the GameCube-ordered Rust records number their bits differently */
                I[i].xDC8_word.flags.x13 = rnd() & 1; I[i].xDC8_word.flags.xC = rnd() & 1;
                I[i].xDCD_flag.b7 = rnd() & 1; I[i].xDAA_flag.b1 = rnd() & 1;
                { int m; for (m = 0; m < 2; m++) I[i].xACC_itemHurtbox[m].skip_update_pos = rnd() & 1; }
                I[i].kind = (ItemKind) (rnd() % 5);
                G[i].user_data = &I[i];
                G[i].next = i < 5 ? &G[i + 1] : 0;
            }
            heads[HSD_GOBJ_PLINK_ITEM] = &G[0];
            for (i = 0; i < 30; i++) it_804A0F60[i] = (rnd() & 1) ? 0 : &A[0];
            pos.x = pos.y = pos.z = 0; MEXCALLS = 0;
            { unsigned a = (unsigned) rnd(); float f = rndf(); int sel = (int) (rnd() % 6); HSD_GObj* g = &G[sel];
            switch (op) {
            case 0: ret = (int) itGetKind(g); break;
            case 1: it_8026B294(g, &pos); break;
            case 2: it_8026B718(g, f); break;
            case 3: ret = it_8026C1E8(g); break;
            case 4: ret = it_8026B3C0((ItemKind) (a % 5)); break;
            case 5: it_ResetStageArticleTable(); break;
            case 6: it_8026B40C(&A[a % 3], (int) (It_Kind_Old_Kuri + (a >> 4) % 30)); break;
            case 7: it_8026B40C(&A[a % 3], 5000 + (int) (a % 7)); break;
            case 8: it_80275414(g); break;
            case 9: it_802756D0(g); break;
            case 10: it_802756E0(g); break;
            case 11: it_802714C0(g); break;
            case 12: it_80274C60(g); break;
            case 13: it_802F2014(g, (int) a); break;
            case 14: ret = it_802F2020(g); break;
            case 15: ret = it_802ECC8C(g); break;
            case 16: it_802ECC98(g, f); break;
            case 17: it_802ECCA4(g, &kind, &pos); break;
            case 18: it_802ECD1C(g, &pos); break;
            case 19: break;
            }
            }
            hw((unsigned) ret); hw((unsigned) kind); hw((unsigned) MEXCALLS);
            { unsigned* w = (unsigned*) &pos; hw(w[0]); hw(w[1]); hw(w[2]); }
            for (i = 0; i < 6; i++) {
                int m;
                hw(I[i].kind); { unsigned* w = (unsigned*) &I[i].pos; hw(w[0]); hw(w[1]); hw(w[2]); }
                hw(*(unsigned*) &I[i].xCBC_hitlagFrames); hw(I[i].grab_victim != 0); hw(I[i].owner != 0);
                hw(I[i].xDC8_word.flags.x13); hw(I[i].xDC8_word.flags.xC); hw(I[i].xDCD_flag.b7); hw(I[i].xDAA_flag.b1);
                hw(I[i].xD0C); hw(I[i].xAC8_hurtboxNum);
                for (m = 0; m < 2; m++) hw(I[i].xACC_itemHurtbox[m].skip_update_pos);
                hw(I[i].x378_itemColl.joint_id_only); hw(I[i].xDD4_itemVar.coin.x0);
                hw(I[i].xDD4_itemVar.tincle.x20); hw(*(unsigned*) &I[i].xDD4_itemVar.tincle.x50); hw(*(unsigned*) &I[i].xDD4_itemVar.tincle.x54);
                hw(I[i].msid);
            }
            for (i = 0; i < 30; i++) hw(it_804A0F60[i] == 0 ? 0 : (unsigned) (it_804A0F60[i] - &A[0]) + 1);
            printf("op%d %d %016llx\n", op, k, H);
        }
    }
    return 0;
}
