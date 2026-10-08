/* geno_online_tests.inc - slice 7, game half: the rollback hash covers every word of a define's Geno state, by test, not by review.
 * Included by geno_tests.c after geno_lua_tests.inc (it uses tl_setup and the TL_* fixtures).
 *
 * Every 32-bit word of GenoState and of GenoLuaBlock must belong to a field of the tables below, and every field says whether the
 * digest (GenoDefine_StateDigest, folded into RB_GameHash for a define) covers it. The test perturbs each word in turn:
 *   covered = 1   the digest MUST move (a word that decides what happens next but is not hashed is a silent desync);
 *   covered = 0   the digest must NOT move (so an exclusion cannot rot into a hole), and the reason is written next to it.
 * A field a later slice adds fails the first check until someone decides, on purpose, which list it is in. */

typedef struct GenoCoverField {
    const char* name;
    u32 off, size;
    int covered;
    const char* why;
} GenoCoverField;

#define GC_F(T, f, c, why) { #f, (u32) __builtin_offsetof(T, f), (u32) sizeof(((T*) 0)->f), c, why }
#define GC_S(f, c, why) GC_F(GenoState, f, c, why)

static const GenoCoverField gc_state_fields[] = {
    GC_S(profile, 2, "fixed at spawn: names the kind's profile (a different profile is a different fighter, not a state change); not walked"),
    GC_S(kind, 2, "fixed at spawn: the kind the block was reset for; not walked"),
    GC_S(flags, 0, "GENO_SF_SCRIPT only marks a block a script touched; a define always has a profile, so it decides nothing"),
    GC_S(resets, 0, "diagnostic: spawns since boot"),
    GC_S(la_i, 1, ""), GC_S(ra_i, 1, ""), GC_S(la_f, 1, ""), GC_S(ra_f, 1, ""),
    GC_S(hook_calls, 0, "diagnostic: hooks run since the reset"),
    GC_S(extra_jumps, 1, ""), GC_S(action_time, 1, ""), GC_S(nchecks, 1, ""),
    GC_S(last_check, 1, "found by this audit: a CHGAND after a wait extends the check registered frames earlier"),
    GC_S(checks, 1, ""), GC_S(rehit_period, 1, ""), GC_S(rehit_count, 1, ""), GC_S(link_mode, 1, ""), GC_S(stun_add, 1, ""),
    GC_S(stun_bonus, 1, ""),
    GC_S(stun_bonus_frame, 0, "the frame stun_bonus was set in: stun_bonus is read only in that frame (geno_stun_frame)"),
    GC_S(hb_flags, 1, ""),
    GC_S(react_flags, 0, "set and read inside one frame's hit processing, keyed to react_frame"),
    GC_S(react_frame, 0, "the frame key of react_flags"),
    GC_S(in_coll, 0, "inside the collision callback only; zero at the start of a logic frame, where the hash is read"),
    GC_S(edge_pending, 0, "as in_coll"),
    GC_S(edge_target, 0, "as in_coll"),
    GC_S(changes, 0, "diagnostic: Geno-made action changes since the reset"),
    GC_S(hold_motion, 1, ""), GC_S(hold_frames, 1, ""), GC_S(move_i, 1, ""), GC_S(move_f, 1, ""),
    GC_S(state_entries, 0, "diagnostic: Geno states entered since the reset"),
    GC_S(enter_from, 1, ""), GC_S(hidden, 1, ""), GC_S(hit_count, 1, ""), GC_S(hit_damage, 1, ""), GC_S(hit_port, 1, ""),
    GC_S(hit_counter, 1, ""),
    GC_S(counters, 0, "diagnostic: hits countered since the reset"),
    GC_S(art_spawned, 0, "diagnostic: articles spawned since the reset (the live articles themselves are hashed)"),
    GC_S(ledge, 1, ""), GC_S(motion_started, 1, ""), GC_S(motion_vy, 1, ""),
    GC_S(motion_land, 0, "diagnostic: frames the root motion has been grounded"),
    GC_S(motion_facing, 1, ""), GC_S(motion_gravity, 1, ""), GC_S(enter_keep, 1, ""), GC_S(atk_connected, 1, ""),
    GC_S(atk_connected_prev, 1, ""), GC_S(fall_limit, 1, ""),
};

#define GC_L(f, c, why) GC_F(GenoLuaBlock, f, c, why)
static const GenoCoverField gc_lua_fields[] = {
    GC_L(slot, 1, ""), GC_L(faults, 1, ""), GC_L(last_fault, 1, ""),
};

/* TestFail takes a plain string (no printf in a game TU): "<what>.<field> (byte <n>) <tail>" */
static void gc_fail(const char* what, const char* field, u32 off, const char* tail, const char* more)
{
    char buf[400], num[12];
    int n = 0, k = 0, d;
    const char* parts[4];
    parts[0] = what; parts[1] = ".";
    parts[2] = field != NULL ? field : "?";
    parts[3] = " (byte ";
    for (d = 0; d < 4; d++) {
        const char* s = parts[d];
        while (*s != 0 && n < 360) {
            buf[n++] = *s++;
        }
    }
    if (off == 0) {
        num[k++] = '0';
    }
    while (off != 0 && k < 11) {
        num[k++] = (char) ('0' + off % 10);
        off /= 10;
    }
    while (k > 0) {
        buf[n++] = num[--k];
    }
    buf[n++] = ')';
    buf[n++] = ' ';
    while (*tail != 0 && n < 380) {
        buf[n++] = *tail++;
    }
    while (more != NULL && *more != 0 && n < 399) {
        buf[n++] = *more++;
    }
    buf[n] = 0;
    TestFail(buf);
}

/* Perturb every word of `block` (n bytes) in turn; `digest` is recomputed each time. Returns the failures. */
static int gc_walk(const char* what, u32* block, u32 bytes, const GenoCoverField* f, int nf, Fighter* fp)
{
    u32 w, i, base, rc = 0;
    for (w = 0; w < bytes / 4; w++) {
        u32 off = w * 4, orig = block[w], after;
        const GenoCoverField* hit = NULL;
        for (i = 0; i < (u32) nf; i++) {
            if (off >= f[i].off && off < f[i].off + f[i].size) {
                hit = &f[i];
            }
        }
        if (hit == NULL) {
            gc_fail(what, "?", off, "belongs to no field of the coverage table: a field was added without deciding whether the rollback hash covers it", NULL);
            rc++;
            continue;
        }
        if (hit->covered == 2) {
            continue;
        }
        base = GenoDefine_StateDigest(fp);
        block[w] = orig ^ 0x00400401u;
        after = GenoDefine_StateDigest(fp);
        block[w] = orig;
        if (hit->covered && after == base) {
            gc_fail(what, hit->name, off, "decides the simulation but does not move the rollback hash", NULL);
            rc++;
        } else if (!hit->covered && after != base) {
            gc_fail(what, hit->name, off, "is listed as outside the hash but moves it: ", hit->why);
            rc++;
        }
    }
    return (int) rc;
}

static int test_geno_digest_coverage(void)
{
    TestGenoState* st;
    GenoLuaBlock* lb;
    int rc = 0, i, j;
    if (!tl_setup(TL_HEAD("cover-lua") TL_BLOCK TL_STATES "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    /* every check, and every condition of every check, is inside what the digest reads */
    st->nchecks = GENO_MAX_CHECKS;
    for (i = 0; i < GENO_MAX_CHECKS; i++) {
        st->checks[i].ncond = GENO_CHECK_CONDS;
        for (j = 0; j < GENO_CHECK_CONDS; j++) {
            st->checks[i].cond[j].head = 1;
        }
    }
    /* a field the walk perturbs below changes a LOOP BOUND (nchecks, ncond): restore keeps the walk valid */
    rc += gc_walk("GenoState", (u32*) st, (u32) sizeof(GenoState), gc_state_fields,
                  (int) (sizeof gc_state_fields / sizeof gc_state_fields[0]), &t_fp);
    lb = GenoGame_LuaBlockOf(&t_fp);
    rc += gc_walk("GenoLuaBlock", (u32*) lb, (u32) sizeof(GenoLuaBlock), gc_lua_fields,
                  (int) (sizeof gc_lua_fields / sizeof gc_lua_fields[0]), &t_fp);
    Geno_TestRestore();
    return rc != 0;
}
