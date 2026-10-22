/* State oracle for src/tu/gm_18A1.rs (see tu_gm_180A_ref.c). The C numbers bitfields from the
 * LSB, the shipping PPC build from the MSB: the three result bytes are bit-reversed on the way
 * in and out so both sides hold the same logical flags. */
#include "melee/gm/gm_18A1.c"
#include "oracle.h"

static StartMeleeRules g_rules;
#include "tu_gm_18A1_stubs.inc"

static unsigned char rev8(unsigned char b)
{
    unsigned char r = 0;
    int i;
    for (i = 0; i < 8; i++) if (b & (1 << i)) r |= (unsigned char) (1 << (7 - i));
    return r;
}

int main(void)
{
    int c, d, s, k;
    arena_init();
    SIZE(MatchEnd);
    SIZE(UnkAllstarData);
    SIZE(gmm_x0_528_t);
    SIZE(ClassicStageEntry);
    reg("rules", &g_rules, sizeof(g_rules));
    reg("allstar", &gm_80473A18, sizeof(gm_80473A18));
    DUMP("tbl", gmTraining_ClassicStages);
    state_end();
    for (d = 0; d < 13; d++) {
        for (s = 0; s < 5; s++) {
            int e = s + d * 5;
            if (e > 64) continue;
            printf("row %d %d %d %08x %08x %08x %08x", d, s, gm_8018A160((u8) d, (u8) s), fbits(gm_8018A188((u8) d, (u8) s)),
                   fbits(gm_8018A1D8((u8) d, (u8) s)), fbits(gm_8018A2C4((u8) d, (u8) s)), fbits(gm_8018A314((u8) d, (u8) s)));
            for (k = 0; k < 4; k++) {
                if (k == 3 && e == 64) continue;
                printf(" %d %d %d", gm_8018A228((u8) d, (u8) s, (u8) k), gm_8018A25C((u8) d, (u8) s, (u8) k), gm_8018A290((u8) d, (u8) s, (u8) k));
            }
            printf("\n");
        }
    }
    for (c = 0; c < 3000; c++) {
        MatchEnd* me;
        unsigned char* b;
        CASE(c);
        rseed(40 + c);
        arena_reset();
        rfill(&gm_80473A18, sizeof(gm_80473A18));
        gm_80473A18.x0.x0.cpu_level = (u8) (rnd() % 6);
        gm_80473A18.x0.xC.x20 = rnd() % 0xA000;
        gm_80473A18.x0.xC.x1C = (int) (rnd() % 4);
        gm_80473A18.x0.xC.xE = (u8) (rnd() % 2);
        gm_80473A18.x0.xC.xF = (u8) (rnd() % 2);
        gm_80473A18.x0.xC.xD = (u8) (rnd() % 2);
        g_rules.x4_5 = rnd() & 1;
        me = (MatchEnd*) AZR(sizeof(MatchEnd));
        me->player_standings[0].x44 = rnd() % 4;
        me->player_standings[0].stocks = (s8) ((int) (rnd() % 6) - 1);
        b = (unsigned char*) me->_x448;
        for (k = 0; k < 4; k++) b[k] = rev8((unsigned char) (rnd() >> 7));
        fn_8018A364((int) (unsigned long long) me);
        printf("x448 %02x %02x %02x %02x\n", rev8(b[0]), rev8(b[1]), rev8(b[2]), rev8(b[3]));
        dump("me0", me, (unsigned char*) me->_x448 - (unsigned char*) me);
        dump("me1", me->x44C, sizeof(MatchEnd) - ((unsigned char*) me->x44C - (unsigned char*) me));
        DUMP("allstar", gm_80473A18);
        DUMPARENA();
        state_end();
    }
    return 0;
}
