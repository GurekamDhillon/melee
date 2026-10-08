/* Online Envoy, stage 2: the native BUILD table. Included by script_game.c after script_fighter_interrupt.inc.
 *
 * A match between two humans can carry a "build" for each player: the passive modifier ops of their Envoy build (fighter values, hit rules,
 * capability caps, armour, crit configuration) compiled by the script layer BEFORE the match and applied ONCE, before frame 0, by the native
 * side (gw_script_netbuild.inc) through the same setters a script uses offline. The ops land in the existing snapshotted BSS (script_fighter_mod,
 * script_hit_ports, the caps tables); THIS table records what was applied (the digest of the agreed record, the digest of the applied ops, counts)
 * so that:
 *   - it lives in game BSS, so a snapshot restore (rollback) puts it back exactly like every other piece of match state;
 *   - it contributes ONE word to the rollback hash (ScriptGame_BuildHashWord), mixed from the stored digests AND from the live fighter values
 *     and hit rules, so a peer whose table, values or rules differ is a checksum desync at the first compared frame;
 *   - scripts can read it (gd.netbuild), never write it.
 * The word is 0 when no build is applied, so a match without one hashes exactly as before (the Turbo and Geno-define pattern).
 * Design: _research/envoy-netplay-scoping-2026-10-05.md, architecture (c), stage 2.
 */
typedef struct {
    int active;             /* 1 once a build was applied for this port */
    unsigned rec_hi, rec_lo;/* the 64-bit digest of the agreed build record */
    unsigned ops_word;      /* digest of the compiled ops the native side applied */
    int nops, nrec;         /* operations applied; modifiers in the record */
    int applied_frame_tag;  /* reserved: 0 (applied before frame 0) */
} ScriptBuild;
static ScriptBuild script_build[6];

static unsigned script_build_mix(unsigned h, unsigned v)
{
    h ^= v;
    h *= 0x01000193u;
    h ^= h >> 15;
    return h;
}

/* port 0..5. Returns 1 when stored. Called by the native apply once the ops are in place. */
int ScriptGame_BuildSet(int port, int rec_hi, int rec_lo, int ops_word, int nops, int nrec)
{
    ScriptBuild* b;
    if (port < 0 || port >= 6 || nops < 0 || nops > 4096 || nrec < 0 || nrec > 64) return 0;
    b = &script_build[port];
    b->active = 1;
    b->rec_hi = (unsigned) rec_hi;
    b->rec_lo = (unsigned) rec_lo;
    b->ops_word = (unsigned) ops_word;
    b->nops = nops;
    b->nrec = nrec;
    OSReport("envoy build: port=%d applied nrec=%d nops=%d record=%08x%08x ops=%08x\n", port + 1, nrec, nops,
             (unsigned) rec_hi, (unsigned) rec_lo, (unsigned) ops_word);
    return 1;
}

/* A match ended or a new one is being armed: nothing carries over. */
void ScriptGame_BuildClear(void)
{
    memset(script_build, 0, sizeof script_build);
    ScriptMods_Clear(); /* stage 4: the triggered programs and their state go with the build */
}

/* field: 0 active, 1 record digest high, 2 low, 3 ops word, 4 ops, 5 modifiers, 6 live word (what the hash sees for this port) */
static unsigned script_build_live(int port)
{
    ScriptBuild* b = &script_build[port];
    unsigned h = 0x811C9DC5u;
    int i, j, n;
    h = script_build_mix(h, b->rec_hi);
    h = script_build_mix(h, b->rec_lo);
    h = script_build_mix(h, b->ops_word);
    h = script_build_mix(h, (unsigned) b->nops);
    h = script_build_mix(h, (unsigned) b->nrec);
    /* the values the match actually runs on: fighter overlays and the hit rules of both of the port's entities. A value that was
     * edited after the apply (or never applied on one peer) is not what the digests above say, and the word shows it. */
    for (i = 0; i < 11; ++i) h = script_build_mix(h, (unsigned) ScriptGame_FighterModRead(port, i));
    for (j = 0; j < 2; ++j) {
        int e = port * 2 + j;
        n = ScriptGame_HitRulesCount(e);
        h = script_build_mix(h, (unsigned) n);
        h = script_build_mix(h, (unsigned) ScriptGame_FighterStatus(e));
        for (i = 0; i < n; ++i) {
            int f;
            for (f = 0; f < 16; ++f) {
                if (f == 12) continue; /* the handle: allocation order, not meaning */
                h = script_build_mix(h, (unsigned) ScriptGame_HitRuleRead(e, i, f));
            }
        }
    }
    return h;
}

int ScriptGame_BuildRead(int port, int field)
{
    ScriptBuild* b;
    if (port < 0 || port >= 6) return 0;
    b = &script_build[port];
    switch (field) {
    case 0: return b->active;
    case 1: return (int) b->rec_hi;
    case 2: return (int) b->rec_lo;
    case 3: return (int) b->ops_word;
    case 4: return b->nops;
    case 5: return b->nrec;
    case 6: return b->active ? (int) script_build_live(port) : 0;
    }
    return 0;
}

/* RB_GameHash: one word for every applied build; 0 when none is (a match without one hashes as before). */
unsigned ScriptGame_BuildHashWord(void)
{
    unsigned h = 0;
    int p, any = 0;
    for (p = 0; p < 6; ++p) {
        if (!script_build[p].active) continue;
        any = 1;
        h = script_build_mix(h ^ (unsigned) (p + 1) * 0x9E3779B1u, script_build_live(p));
    }
    if (any) h = script_build_mix(h, ScriptMods_HashWord()); /* stage 4: statuses, recents, programs (0 without a triggered program) */
    return any ? (h | 1u) ^ 0x45564F59u : 0;
}

/* Fixture for the native suite: the table, the word's coverage, snapshot exactness (it is plain BSS: a memcpy of the table is all a restore does). */
int ScriptGame_BuildTest(void)
{
    ScriptBuild saved[6], snap[6];
    unsigned w0, w1, w2;
    int rc = 0;
    memcpy(saved, script_build, sizeof saved);
    memset(script_build, 0, sizeof script_build);
    if (ScriptGame_BuildHashWord() != 0) rc |= 1;                        /* no build: no word */
    if (ScriptGame_BuildSet(6, 0, 0, 0, 0, 0) || ScriptGame_BuildSet(-1, 0, 0, 0, 0, 0) ||
        ScriptGame_BuildSet(0, 0, 0, 0, 5000, 0) || ScriptGame_BuildSet(0, 0, 0, 0, 0, 65)) rc |= 2; /* bounds */
    if (!ScriptGame_BuildSet(0, 0x11223344, 0x55667788, 0x0A0B0C0D, 7, 3)) rc |= 4;
    w0 = ScriptGame_BuildHashWord();
    if (w0 == 0 || ScriptGame_BuildRead(0, 0) != 1 || ScriptGame_BuildRead(0, 1) != 0x11223344 ||
        ScriptGame_BuildRead(0, 4) != 7 || ScriptGame_BuildRead(1, 0) != 0) rc |= 8;
    if (ScriptGame_BuildHashWord() != w0) rc |= 16;                      /* a pure function of the table and the live values */
    ScriptGame_BuildSet(0, 0x11223344, 0x55667789, 0x0A0B0C0D, 7, 3);   /* one bit of the record digest */
    w1 = ScriptGame_BuildHashWord();
    ScriptGame_BuildSet(0, 0x11223344, 0x55667788, 0x0A0B0C0E, 7, 3);   /* one bit of the ops word */
    w2 = ScriptGame_BuildHashWord();
    if (w1 == w0 || w2 == w0 || w1 == w2) rc |= 32;                      /* every stored field is in the word */
    ScriptGame_BuildSet(0, 0x11223344, 0x55667788, 0x0A0B0C0D, 7, 3);
    ScriptGame_BuildSet(2, 1, 2, 3, 4, 5);
    if (ScriptGame_BuildHashWord() == w0) rc |= 64;                      /* a second port changes it */
    memcpy(snap, script_build, sizeof snap);
    w1 = ScriptGame_BuildHashWord();
    ScriptGame_BuildSet(1, 9, 9, 9, 9, 9);
    memcpy(script_build, snap, sizeof snap);                             /* the rollback restore */
    if (ScriptGame_BuildHashWord() != w1 || script_build[1].active) rc |= 128;
    ScriptGame_BuildClear();
    if (ScriptGame_BuildHashWord() != 0 || ScriptGame_BuildRead(0, 0) != 0) rc |= 256;
    memcpy(script_build, saved, sizeof saved);
    return rc;
}
