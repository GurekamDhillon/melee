/* netdir_test.c - the online run director (gw_netdir.h): the shared stocks, the verdicts, the stage-end tuple and the Classic plan over the 1P tables.
 * Pure functions, no game: tools/port/build.sh --native-test netdir. */
#include "../platform/gw_netdir.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

#define WIDE 0x01FFFFFFu /* fighters 0..24 */

static void flags_and_state(void) {
    GwNdRun r;
    long f;
    gw_nd_init(&r, GW_ND_KIND_CLASSIC, 8, 9, 1);
    assert(r.pool == 9 && r.pool_start == 9 && r.len == 8 && gw_nd_cont_left(&r) == 1 && !gw_nd_asking(&r));
    f = gw_nd_flags(&r);
    assert((f & 0x7F) == 9 && ((f >> 7) & 3) == 0 && ((f >> 9) & 1) == 0 && ((f >> 10) & 3) == 1 && ((f >> 12) & 63) == 8 && ((f >> 18) & 3) == 1);
    r.pool = 0; r.over = GW_ND_LOST; r.cont_used = 1;
    f = gw_nd_flags(&r);
    assert((f & 0x7F) == 0 && ((f >> 7) & 3) == GW_ND_LOST && ((f >> 9) & 1) == 1);
    assert(f > 0 && f < 2147483647L);
    gw_nd_init(&r, 0, 999, 500, 9);
    assert(r.len == 63 && r.pool == GW_ND_POOL_MAX && r.cont_start == 1);
}

static void shared_stocks(void) {
    GwNdRun r;
    assert(gw_nd_deaths(3, 0, 2) == 4 && gw_nd_deaths(1, 0, 1) == 1 && gw_nd_deaths(4, 4, 4) == 0 && gw_nd_deaths(1, -1, 5) == 1);
    /* a pool that pays, an empty pool that offers the continue, the continue refilling, then the end */
    gw_nd_init(&r, GW_ND_KIND_STAGES, 0, 6, 1);
    assert(gw_nd_after_stage(&r, 2) == GW_ND_LIVE && r.pool == 4 && r.stage == 1 && !gw_nd_asking(&r));
    assert(gw_nd_after_stage(&r, 9) == GW_ND_LIVE && r.pool == 0 && r.stage == 1 && gw_nd_asking(&r)); /* the failed stage is not passed */
    assert(gw_nd_spend_continue(&r) && r.pool == 6 && r.cont_used == 1 && gw_nd_cont_left(&r) == 0 && !gw_nd_asking(&r));
    assert(!gw_nd_spend_continue(&r)); /* one token per run */
    assert(gw_nd_after_stage(&r, 1) == GW_ND_LIVE && r.pool == 5 && r.stage == 2);
    assert(gw_nd_after_stage(&r, 5) == GW_ND_LOST && r.pool == 0 && !gw_nd_asking(&r));
    assert(gw_nd_after_stage(&r, 0) == GW_ND_LOST && r.stage == 2); /* an ended run stays ended */
    /* no token at all: the empty pool ends the run at once */
    gw_nd_init(&r, GW_ND_KIND_STAGES, 0, 2, 0);
    assert(gw_nd_after_stage(&r, 1) == GW_ND_LIVE && gw_nd_after_stage(&r, 1) == GW_ND_LOST);
    /* the last stage clears the run */
    gw_nd_init(&r, GW_ND_KIND_CLASSIC, 3, 20, 1);
    assert(gw_nd_after_stage(&r, 1) == GW_ND_LIVE && gw_nd_after_stage(&r, 1) == GW_ND_LIVE && gw_nd_after_stage(&r, 1) == GW_ND_CLEARED && r.stage == 3);
    assert(strcmp(gw_nd_over_name(GW_ND_LOST), "stocks") == 0 && strcmp(gw_nd_over_name(GW_ND_CLEARED), "cleared") == 0 && gw_nd_over_name(GW_ND_LIVE)[0] == '\0');
}

static void end_tuple(void) {
    GwNdEnd a = { 7, 1423, 1, 0, 1, 2, 5, 0xDEADBEEFu }, b, c;
    char m[80];
    assert(gw_nd_end_format(&a, m, sizeof m) > 0 && m[0] == 'F');
    assert(gw_nd_end_parse(m, &b) && gw_nd_end_diff(&a, &b) == NULL);
    assert(!gw_nd_end_parse("F 1 2 zz 0 0 0 0 0", &c) && !gw_nd_end_parse("G 1 2 00000000 0 0 0 0 0", &c) && !gw_nd_end_parse("F 1 2 00000000 7 0 0 0 0", &c));
    assert(!gw_nd_end_parse("F 7 1423 deadbeef 1 0 1 2 5 junk", &c));
    assert(!gw_nd_end_parse("F 7 1423 deadbeef 1 0 1 2 100", &c));
    c = a; c.epoch++;  assert(strcmp(gw_nd_end_diff(&a, &c), "epoch") == 0);
    c = a; c.stage++;  assert(strcmp(gw_nd_end_diff(&a, &c), "stage") == 0);
    c = a; c.frame++;  assert(strcmp(gw_nd_end_diff(&a, &c), "exit frame") == 0);
    c = a; c.winner = 0; assert(strcmp(gw_nd_end_diff(&a, &c), "winner") == 0);
    c = a; c.s1++;     assert(strcmp(gw_nd_end_diff(&a, &c), "stocks") == 0);
    c = a; c.pool++;   assert(strcmp(gw_nd_end_diff(&a, &c), "shared stocks") == 0);
    c = a; c.hash ^= 1; assert(strcmp(gw_nd_end_diff(&a, &c), "final hash") == 0);
    c = a; c.hash = 0; assert(strcmp(gw_nd_end_diff(&a, &c), "final hash unavailable") == 0);
}

static int plan_ok(const GwNdPlan *p, uint32_t mask, int h0, int h1, int strict) {
    uint32_t used = 0;
    int i, k;
    for (i = 0; i < p->n; ++i) {
        const GwNdStage *g = &p->st[i];
        if (g->flags & GW_ND_ROW_BOSS) { if (g->ck[0] != 30 || g->ck[1] != 26 || g->n_enemy != 2) return 0; continue; }
        if (g->n_enemy < 1) return 0;
        for (k = 0; k < 3; ++k) {
            int c = g->ck[k];
            if (c == GW_ND_ENEMY_NONE) continue;
            if (strict && (!gw_nd_mask_has(mask, c) || c == h0 || c == h1 || gw_nd_mask_has(used, c))) return 0;
        }
        for (k = 0; k < 3; ++k) if (g->ck[k] != GW_ND_ENEMY_NONE) used |= 1u << g->ck[k];
    }
    return 1;
}

static void classic_plan(void) {
    GwNdPlan a, b, c;
    uint32_t seed;
    int i, narrow_ok = 0;
    assert(gw_nd_classic_plan(12345u, 0, WIDE, 8, 9, &a) == GW_ND_CLASSIC_STAGES);
    gw_nd_classic_plan(12345u, 0, WIDE, 8, 9, &b);
    assert(memcmp(&a, &b, sizeof a) == 0 && strlen(a.digest) == 16); /* a pure function */
    /* the shape: fight, team of two, fight, metal, fight, trio, giant, boss */
    assert(strcmp(gw_nd_flags_name(a.st[0].flags), "fight") == 0 && strcmp(gw_nd_flags_name(a.st[1].flags), "team of two") == 0 && strcmp(gw_nd_flags_name(a.st[3].flags), "metal") == 0 &&
           strcmp(gw_nd_flags_name(a.st[5].flags), "trio") == 0 && strcmp(gw_nd_flags_name(a.st[6].flags), "giant") == 0 && strcmp(gw_nd_flags_name(a.st[7].flags), "boss") == 0);
    assert(a.st[1].n_enemy == 2 && a.st[3].n_enemy == 1 && a.st[5].n_enemy == 3 && a.st[0].n_enemy == 1);
    for (i = 0; i < GW_ND_CLASSIC_STAGES; ++i) assert(a.st[i].flags != 0x80 && a.st[i].row != 2 && a.st[i].row != 5 && a.st[i].row != 8); /* no bonus rows online */
    /* seeds differ, plans differ; the digest covers the mask, the humans' fighters and the loop */
    gw_nd_classic_plan(12346u, 0, WIDE, 8, 9, &c); assert(strcmp(a.digest, c.digest) != 0);
    gw_nd_classic_plan(12345u, 0, WIDE & ~(1u << 10), 8, 9, &c); assert(strcmp(a.digest, c.digest) != 0);
    gw_nd_classic_plan(12345u, 0, WIDE, 8, 10, &c); assert(strcmp(a.digest, c.digest) != 0);
    gw_nd_classic_plan(12345u, 1, WIDE, 8, 9, &c); assert(strcmp(a.digest, c.digest) != 0);
    /* every rule holds for many seeds: enemies in the mask, never a human's fighter, no fighter twice */
    for (seed = 1; seed < 400; ++seed) {
        gw_nd_classic_plan(seed, 0, WIDE, (int) (seed % 25), (int) ((seed * 7 + 3) % 25), &a);
        assert(plan_ok(&a, WIDE, (int) (seed % 25), (int) ((seed * 7 + 3) % 25), 1));
        for (i = 0; i < a.n; ++i) assert(a.st[i].relaxed == 0);
    }
    /* a narrow agreed mask (the intersection of two saves that unlocked little): only those fighters are ever picked while the pools allow it */
    {
        uint32_t narrow = (1u << 0) | (1u << 1) | (1u << 2) | (1u << 4) | (1u << 5) | (1u << 6) | (1u << 7) | (1u << 8) | (1u << 11) | (1u << 13) | (1u << 15) | (1u << 17) | (1u << 20) | (1u << 21) | (1u << 22);
        for (seed = 1; seed < 200; ++seed) {
            gw_nd_classic_plan(seed, 0, narrow, 8, 0, &a);
            for (i = 0; i < a.n; ++i) {
                int k, in = 1;
                if (a.st[i].flags & GW_ND_ROW_BOSS) continue;
                for (k = 0; k < 3; ++k) if (a.st[i].ck[k] != GW_ND_ENEMY_NONE && !gw_nd_mask_has(narrow, a.st[i].ck[k])) in = 0;
                if (in) narrow_ok++;
                else assert(0 && "opponent outside agreed unlock mask");
            }
        }
        assert(narrow_ok > 200 * 6);
    }
    /* an empty mask still fills every row (deterministically, flagged relaxed) */
    gw_nd_classic_plan(5u, 0, 0, 0, 1, &a); gw_nd_classic_plan(5u, 0, 0, 0, 1, &b);
    assert(memcmp(&a, &b, sizeof a) == 0 && a.n == 0);
}

static void record_carries_state(void) {
    GwNrRecord r, p;
    char text[GW_NR_TEXT_MAX + 1], why[80];
    GwNdRun d;
    gw_nd_init(&d, GW_ND_KIND_CLASSIC, 8, 12, 1);
    gw_nr_clear(&r);
    r.seed = 4242; r.game = 3; r.flags = gw_nd_flags(&d); r.ext = 0x123456; r.score[0] = 1; r.score[1] = 1;
    snprintf(r.x, sizeof r.x, "%s", "0123456789abcdef");
    assert(gw_nr_encode(&r, text, sizeof text) > 0);
    assert(gw_nr_parse(text, &p, why, sizeof why) && p.flags == r.flags && p.ext == 0x123456 && p.stage == 2 && strcmp(p.x, "0123456789abcdef") == 0);
    /* a different pool is a different record */
    d.pool--;
    {
        GwNrRecord q = r;
        char t2[GW_NR_TEXT_MAX + 1];
        q.flags = gw_nd_flags(&d);
        assert(gw_nr_encode(&q, t2, sizeof t2) > 0 && strcmp(gw_nr_digest_of(text), gw_nr_digest_of(t2)) != 0);
    }
}

int main(void) {
    flags_and_state();
    shared_stocks();
    end_tuple();
    classic_plan();
    record_carries_state();
    puts("netdir: all passed");
    return 0;
}
