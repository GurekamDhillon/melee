/* C reference for src/tu/ftkirbyyoshiegg.rs: fn_8010B16C (1 fused site: grab_timer -= percentTemp
 * * f()) cut verbatim from the game by tu_shard329.py. ftKb_SpecialNYs_80109380 (another TU) is
 * the stand-in unk3 / damage_multiplier; ftCo_800C0C88 is a stand-in predicate on its argument. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/types.h"
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 s = 0xE66u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static float UNK3, DMGMUL;
float ftKb_SpecialNYs_80109380(void) { return UNK3 / DMGMUL; }
bool ftCo_800C0C88(int v) { return (v & 1) != 0; }
void Fighter_UpdateModelScale(HSD_GObj* g) { (void) g; }

#include "tu_ftkirbyyoshiegg_extract.inc"

static Fighter F;
static HSD_GObj G;

int main(void)
{
    int i;
    G.user_data = &F;
    for (i = 0; i < 3000; i++) {
        F.grab_timer = rf() * 60.0f;
        F.dmg.x1838_percentTemp = rf() * 30.0f;
        F.dmg.x18CC = (rnd() & 1) ? 3 : (int) (rnd() % 5);
        F.dmg.x18D0 = (int) (rnd() % 8);
        UNK3 = rf();
        DMGMUL = rf() + 1.5f;
        F.x1828 = 0;
        fn_8010B16C(&G);
        printf("%d %08x %d\n", i, b(F.grab_timer), (int) F.x1828);
    }
    return 0;
}
