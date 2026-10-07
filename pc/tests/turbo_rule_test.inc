/* Native fixture of the SAME state core used by script_game.c. No game, disc,
 * timing, sockets or Lua. Included by engine-data because its runner is read-only. */
#include "../platform/gw_intrwin_core.h"
#include "../platform/gw_replay_matchrules.h"

static void turbo_rule_native_test(void)
{
    IntrWin w = {0}, before, snap, expected;
    IntrWinAttack hit = {IW_K_JAB, 44, 7, 3, 0};
    unsigned rules = GW_TURBO_V1;
    int jumps = 2, saved_jumps, expected_jumps, key, kind;

    {
        unsigned char event[1 + GW_RP_RULES_START] = {0};
        unsigned word = 99;
        assert(!gw_rp_rules_read(event, GW_RP_STANDARD_START, &word) && word == 0);
        assert(!gw_rp_rules_read(event, GW_RP_RULES_START, &word) && word == 0);
        gw_rp_rules_write(event, rules | (12u << GW_TURBO_FRAMES_SHIFT));
        assert(gw_rp_rules_read(event, GW_RP_STANDARD_START + 4, &word) == -1);
        assert(!gw_rp_rules_read(event, GW_RP_RULES_START, &word));
        assert(word == (rules | (12u << GW_TURBO_FRAMES_SHIFT)));
        gw_rp_rules_write(event, rules | GW_TURBO_HITLAG);
        assert(gw_rp_rules_read(event, GW_RP_RULES_START, &word) == -1);
        gw_rp_rules_write(event, 0);
        assert(!gw_rp_rules_read(event, GW_RP_RULES_START, &word) && word == 0);
        event[1 + GW_RP_STANDARD_START + 3] = '2';
        assert(gw_rp_rules_read(event, GW_RP_RULES_START, &word) == -1);
        puts("turbo replay: tagged rule roundtrip, legacy OFF, unsupported refusal PASS");
    }

    assert(gw_turbo_valid(0) && gw_turbo_valid(rules));
    assert(gw_turbo_frames(rules) == 30);
    assert(gw_turbo_frames(rules | (12u << GW_TURBO_FRAMES_SHIFT)) == 12);
    assert(!gw_turbo_valid(rules | GW_TURBO_HITLAG));
    assert(!gw_turbo_valid(rules | 0x1000u));
    assert(!gw_turbo_valid(GW_TURBO_SHIELD_HIT));
    assert(!gw_turbo_valid(rules | (121u << GW_TURBO_FRAMES_SHIFT)));

    /* Off must return before touching ANY window byte or the fighter jump count,
     * including stale/snapshotted windows. Every source move and hit kind. */
    memset(&w, 0x5a, sizeof w);
    before = w;
    for (key = IW_K_NONE; key <= IW_K_SPECIAL; ++key) {
        hit.key = key;
        for (kind = IW_OPEN_HIT; kind <= IW_OPEN_ITEM; ++kind) {
            assert(intrwin_open_state(&w, &hit, kind, 0, &jumps) == 0);
            assert(intrwin_open_state(&w, &hit, kind, rules & ~GW_TURBO_ON, &jumps) == 0);
            assert(!memcmp(&w, &before, sizeof w) && jumps == 2);
        }
    }
    puts("turbo OFF: all hit kinds/move keys leave window and jumps byte-identical PASS");

    memset(&w, 0, sizeof w);
    hit.key = IW_K_JAB;
    assert(intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps) == 1);
    assert(w.frames == 30 && w.opened == 1 && w.key == IW_K_JAB);
    before = w;
    assert(!intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps));
    assert(!memcmp(&before, &w, sizeof w)); /* multi-hit does not refresh */
    intrwin_close(&w, 1);
    assert(w.frames == 0 && w.cancels == 1);
    assert(!intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps));
    ++hit.x206C_attack_instance;
    assert(intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps) == 1);

    /* Each strict exclusion and a permitted neighboring action. */
    for (key = IW_K_JAB; key <= IW_K_SPECIAL; ++key)
        assert(!intrwin_allows(rules, key, 1, IW_X_JAB, key));
    assert(intrwin_allows(rules, IW_K_JAB, 1, IW_X_TILT, IW_K_FTILT));
    assert(!intrwin_allows(rules, IW_K_SPECIAL, 0, IW_X_SPECIAL, IW_K_NONE));
    assert(intrwin_allows(rules, IW_K_JAB, 1, IW_X_DASH, IW_K_NONE));
    assert(intrwin_allows(rules, IW_K_DASHATK, 1, IW_X_DASH, IW_K_NONE));
    for (key = IW_K_FTILT; key <= IW_K_SPECIAL; ++key)
        assert(!intrwin_allows(rules, key, 1, IW_X_DASH, IW_K_NONE));
    for (key = IW_K_FSMASH; key <= IW_K_DSMASH; ++key) {
        assert(!intrwin_allows(rules, key, 1, IW_X_CROUCH, IW_K_NONE));
        assert(!intrwin_allows(rules, key, 1, IW_X_JUMP, IW_K_NONE));
        assert(intrwin_allows(rules, key, 1, IW_X_ESCAPE, IW_K_NONE));
    }
    assert(!intrwin_allows(rules, IW_K_JAB, 1, IW_X_SHIELD, IW_K_NONE));
    for (key = IW_K_NAIR; key <= IW_K_SPECIAL; ++key) {
        assert(!intrwin_allows(rules, key, 0, IW_X_AIRDODGE, IW_K_NONE));
        assert(intrwin_allows(rules, key, 0, IW_X_AIRJUMP, IW_K_NONE));
    }
    assert(!intrwin_allows(rules, IW_K_JAB, 1, IW_X_WALK, IW_K_NONE));
    assert(!intrwin_allows(rules, IW_K_JAB, 1, IW_X_TURN, IW_K_NONE));
    assert(intrwin_allows(GW_TURBO_ON, IW_K_JAB, 1, IW_X_JAB, IW_K_JAB));
    assert(intrwin_allows(GW_TURBO_ON, IW_K_FSMASH, 1, IW_X_JUMP, IW_K_NONE));

    memset(&w, 0, sizeof w);
    hit.key = IW_K_NONE; /* game adapter maps grabs/throws here */
    assert(!intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps));
    hit.key = IW_K_FAIR; hit.air = 1;
    assert(!intrwin_open_state(&w, &hit, IW_OPEN_ITEM, rules, &jumps));
    assert(!intrwin_open_state(&w, &hit, IW_OPEN_SHIELD, rules & ~GW_TURBO_SHIELD_HIT, &jumps));
    assert(intrwin_open_state(&w, &hit, IW_OPEN_SHIELD, rules, &jumps) == 1);
    assert(jumps == 1);
    intrwin_close(&w, 0); ++hit.x206C_attack_instance;
    assert(intrwin_open_state(&w, &hit, IW_OPEN_ITEM, rules | GW_TURBO_ITEM_HIT, &jumps) == 1);
    puts("turbo ON: hit/open, consume, multi-hit, exclusions, variants, jumps PASS");

    /* Restore BEFORE the hit, then replay open -> movement cancel -> repeat
     * refusal -> different hit. Compare complete decision-driving state and
     * fighter jumps, not just the bytes immediately after memcpy. */
    memset(&w, 0, sizeof w);
    rules |= GW_TURBO_NO_MOVE_LOOP;
    hit.key = IW_K_JAB; hit.air = 1; jumps = 2;
    snap = w; saved_jumps = jumps;
    for (kind = 0; kind < 2; ++kind) {
        if (kind) { w = snap; jumps = saved_jumps; }
        hit.key = IW_K_JAB; hit.x206C_attack_instance = 30;
        assert(intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps) == 1);
        w.frames = 24; w.taken = IW_X_AIRJUMP;
        intrwin_carry_set(&w); intrwin_close(&w, 1);
        assert(w.carry_frames == 23 && w.carry_key == IW_K_JAB);
        ++hit.x206C_attack_instance;
        assert(intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps) == -1);
        assert(w.carry_frames == 30);
        intrwin_carry_tick(&w);
        hit.key = IW_K_FAIR; ++hit.x206C_attack_instance;
        assert(intrwin_open_state(&w, &hit, IW_OPEN_HIT, rules, &jumps) == 1);
        assert(!w.carry_frames && !w.carry_key);
        if (!kind) { expected = w; expected_jumps = jumps; }
        else assert(!memcmp(&w, &expected, sizeof w) && jumps == expected_jumps);
    }
    /* Restore MID-window and replay consuming it and expiring movement memory. */
    snap = w;
    for (kind = 0; kind < 2; ++kind) {
        if (kind) w = snap;
        w.frames = 3; w.taken = IW_X_AIRJUMP;
        intrwin_carry_set(&w); intrwin_close(&w, 1);
        intrwin_carry_tick(&w); intrwin_carry_tick(&w);
        assert(!w.frames && !w.carry_frames && !w.carry_key);
        if (!kind) expected = w;
        else assert(!memcmp(&w, &expected, sizeof w));
    }
    /* Actual window expiry: replay the remaining 30 free-frame ticks from the
     * same mid-window snapshot. Hitlag is excluded by fighter.c's caller gate. */
    w = snap;
    for (kind = 0; kind < 2; ++kind) {
        if (kind) w = snap;
        for (key = 0; key < 29; ++key) assert(!intrwin_age(&w));
        assert(w.frames == 1 && intrwin_age(&w));
        intrwin_close(&w, 0);
        assert(w.frames == 0);
        if (!kind) expected = w;
        else assert(!memcmp(&w, &expected, sizeof w));
    }
    puts("turbo snapshot: replay across open, consume, repeat refusal and memory expiry PASS");
}
