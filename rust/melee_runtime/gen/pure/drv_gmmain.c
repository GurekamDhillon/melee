/* EXTRACT: src/melee/gm/gmmain_lib.c bitset_mask bitset_clear gmMainLib_GetCardData GetPersistentFighterData gmMainLib_8015D8B0 gmMainLib_8015CCF0 gmMainLib_8015CDC8 gmMainLib_8015CDD4 gmMainLib_8015CDE0 gmMainLib_8015D06C gmMainLib_8015D48C gmMainLib_8015D4E8 gmMainLib_GetBonusMatchTotal gmMainLib_GetCoinMatchTotal gmMainLib_GetKOTotal gmMainLib_GetStockMatchTotal gmMainLib_GetTimeMatchTotal */
/* State oracle for melee_world::gmmain: the verbatim C accessors over a random gmm_x0 record.
 * One line per case: op case hash (offsets of the returned pointers inside the record, the words they reach). */
#include <melee/gm/types.h>
#include <melee/gm/gmmain_lib.h>
#include "common.h"
extern int printf(const char*, ...);

struct gmm_x0 gmMainLib_8045A6C0;
struct gmm_x0* gmMainLib_804D3EE0 = &gmMainLib_8045A6C0;
#include "extract_gmmain.h"

static u64_ H;
static void hw(unsigned w) { H = fnv(&w, 4, H); }
static void hoff(void* p) { hw(p ? (unsigned) ((char*) p - (char*) &gmMainLib_8045A6C0) : 0xFFFFFFFFu); }

int main(void)
{
    int op, k, i;
    printf("sizeof_gmm_x0 %u\n", (unsigned) sizeof(struct gmm_x0));
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 40; k++) {
            unsigned char* p = (unsigned char*) &gmMainLib_8045A6C0; unsigned j;
            unsigned a = (unsigned) rnd(); int ret = 0; unsigned char sk = (unsigned char) (rnd() % 9);
            H = FNV0;
            for (j = 0; j < sizeof(struct gmm_x0); j++) p[j] = (unsigned char) rnd();
            for (i = 0; i < 9; i++) GetPersistentFighterData((u8) i)->x7C.b0 = rnd() & 1;
            switch (op) {
            case 0: gmMainLib_8015D8B0(a % 96); break;
            case 1: hoff(gmMainLib_8015CCF0()); break;
            case 2: hoff(gmMainLib_8015CDC8()); break;
            case 3: hoff(gmMainLib_8015CDD4()); break;
            case 4: hoff(gmMainLib_8015CDE0()); break;
            case 5: hoff(gmMainLib_8015D06C(sk)); break;
            case 6: ret = gmMainLib_8015D48C(sk); break;
            case 7: gmMainLib_8015D4E8(sk, (int) a); break;
            case 8: hoff(gmMainLib_GetBonusMatchTotal()); break;
            case 9: hoff(gmMainLib_GetCoinMatchTotal()); break;
            case 10: hoff(gmMainLib_GetKOTotal()); break;
            case 11: hoff(gmMainLib_GetStockMatchTotal()); break;
            case 12: hoff(gmMainLib_GetTimeMatchTotal()); break;
            case 13: break;
            case 14: break;
            }
            hw((unsigned) ret);
            { unsigned m;
              GmSaveData* sd = &gmMainLib_804D3EE0->thing.save_data;
              for (m = 0; m < sizeof(sd->x1B40) / 4; m++) hw(sd->x1B40[m]);
              hw(sd->x1A48); hw(sd->bonus_matches); hw(sd->coin_matches); hw(sd->stock_matches); hw(sd->time_matches); hw((unsigned) sd->x1A5C);
              for (i = 0; i < 9; i++) { __typeof__(GetPersistentFighterData(0)) f = GetPersistentFighterData((u8) i); hw(f->x7C.b0); hw((unsigned) f->x7C.x84); }
            }
            printf("op%d %d %016llx\n", op, k, H);
        }
    }
    return 0;
}
