/* Native test of the skin registry's pure logic (pc/platform/gw_skins_core.h): ordering, the 255 cap, team and
 * visibility rules, partner rows, the netplay wire costume. No game, no disc. */
#include "../platform/gw_skins_core.h"

static int failures;
#define CHECK(c, ...) do { if (!(c)) { printf("FAIL line %d: ", __LINE__); printf(__VA_ARGS__); printf("\n"); ++failures; } } while (0)

static char logbuf[64][300];
static int nlog;
static void sink(const char *m) { if (nlog < 64) snprintf(logbuf[nlog++], 300, "%s", m); }
static int logged(const char *needle) {
    int i;
    for (i = 0; i < nlog; ++i) if (strstr(logbuf[i], needle)) return 1;
    return 0;
}

static int base_fn(int fk) {
    switch (fk) {
    case 0: return 5;    /* Mario */
    case 1: return 5;    /* Fox */
    case 7: return 5;    /* Sheik */
    case 10: return 5;   /* Popo */
    case 11: return 5;   /* Nana */
    case 19: return 5;   /* Zelda */
    case 4: return 6;    /* Kirby */
    case 40: return 7;   /* an m-ex fighter */
    case 41: return 1;   /* a Geno define */
    default: return 0;
    }
}
static const char *name_fn(int fk) { return fk == 0 ? "Mario" : fk == 1 ? "Fox" : NULL; }

static sk_costume mk(int fk, const char *mod, int entry, int order) {
    sk_costume s;
    memset(&s, 0, sizeof s);
    s.fk = fk;
    snprintf(s.mod, sizeof s.mod, "%s", mod);
    s.entry = entry;
    s.order = order;
    snprintf(s.name, sizeof s.name, "%s-%d", mod, entry);
    snprintf(s.file, sizeof s.file, "skins/%s/%d.dat", mod, entry);
    snprintf(s.joint, sizeof s.joint, "J%d", entry);
    s.team = -1;
    s.like = 0;
    s.kirby_hat = -1;
    s.id = sk_identity(mod, entry, "1.0.0");
    return s;
}
static void add(sk_costume s) { if (sk_add(&s)) { printf("add failed\n"); ++failures; } }

static void fresh(void) { sk_reset(); sk_sink = sink; nlog = 0; }

static void test_order_and_indices(void) {
    const sk_costume *r;
    fresh();
    add(mk(0, "zeta", 0, 0));
    add(mk(0, "Alpha", 1, 0));
    add(mk(0, "alpha", 0, 0)); /* the same id folded: exact compare breaks the tie */
    add(mk(0, "mid", 0, -5));  /* order -5 sorts first */
    add(mk(1, "fox-one", 0, 0));
    sk_finalize(base_fn, name_fn);
    CHECK(sk_total(0) == 9, "Mario total %d", sk_total(0));
    CHECK(sk_total(1) == 6, "Fox total %d", sk_total(1));
    CHECK(sk_added(0) == 4, "added %d", sk_added(0));
    r = sk_row(0, 5);
    CHECK(r && !strcmp(r->mod, "mid"), "index 5 is the order -5 skin, got %s", r ? r->mod : "none");
    r = sk_row(0, 6);
    CHECK(r && !strcmp(r->mod, "Alpha"), "6 is Alpha (exact compare breaks the fold tie): %s", r ? r->mod : "none");
    r = sk_row(0, 8);
    CHECK(r && !strcmp(r->mod, "zeta"), "last is zeta, got %s", r ? r->mod : "none");
    CHECK(sk_row(0, 4) == NULL, "an original costume has no row");
    CHECK(sk_row(0, 9) == NULL, "past the end has no row");
    CHECK(sk_total(2) == 0, "a fighter without a list");
    /* two runs, same input, same order (stability) */
    {
        int a = sk_row(0, 6)->index, b;
        sk_reset(); sk_sink = sink;
        add(mk(0, "zeta", 0, 0)); add(mk(0, "alpha", 0, 0)); add(mk(0, "Alpha", 1, 0)); add(mk(0, "mid", 0, -5));
        sk_finalize(base_fn, name_fn);
        b = sk_row(0, 6)->index;
        CHECK(a == b && !strcmp(sk_row(0, 6)->mod, "Alpha"), "insertion order must not matter");
    }
}

static void test_cap(void) {
    int i;
    char m[32];
    fresh();
    for (i = 0; i < 300; ++i) { snprintf(m, sizeof m, "s%03d", i); add(mk(0, m, 0, 0)); }
    sk_finalize(base_fn, name_fn);
    CHECK(sk_total(0) == 255, "capped total %d", sk_total(0));
    CHECK(sk_added(0) == 250, "added %d", sk_added(0));
    CHECK(sk_row(0, 254) && !strcmp(sk_row(0, 254)->mod, "s249"), "id 254 is the 250th skin");
    CHECK(sk_reg.refused == 50, "refused %d", sk_reg.refused);
    CHECK(logged("is at 255 costumes - refused s250"), "the cap message names the first refused skin");
    /* the refusal is deterministic: the earliest 250 by order win */
    CHECK(sk_from_wire(0, sk_to_wire(0, 254)) == 254, "id 254 round trips");
    CHECK(sk_from_wire(0, SK_WIRE_TAG | (int) (sk_identity("s250", 0, "1.0.0") & SK_WIRE_MASK)) == 0, "a refused skin maps to default");
}

static void test_team_like_kirby(void) {
    sk_costume a, b, c;
    fresh();
    a = mk(0, "a", 0, 0); a.team = 0; a.like = 2;
    b = mk(0, "b", 0, 0); b.team = 0;           /* red claimed twice */
    c = mk(0, "c", 0, 0); c.team = 2; c.like = 99; /* bad like falls back to 0 */
    add(a); add(b); add(c);
    sk_finalize(base_fn, name_fn);
    CHECK(sk_team(0, 0) == 5, "red is the first claimant (5), got %d", sk_team(0, 0));
    CHECK(sk_team(0, 2) == 7, "green is c (7), got %d", sk_team(0, 2));
    CHECK(sk_team(0, 1) == -1, "blue unclaimed");
    CHECK(logged("team red is already costume 5"), "second red claim is logged");
    CHECK(sk_like(0, 5) == 2, "like 2");
    CHECK(sk_like(0, 7) == 0, "bad like -> 0");
    CHECK(sk_like(0, 3) == -1, "original costume: -1");
    CHECK(sk_kirby_row(0, 5) == 2, "kirby row follows like when unset");
}

static void test_kirby_explicit(void) {
    sk_costume k;
    fresh();
    k = mk(4, "kb", 0, 0); k.kirby_hat = 3; k.like = 1;
    add(k);
    sk_finalize(base_fn, name_fn);
    CHECK(sk_kirby_row(4, 6) == 3, "explicit hat");
    CHECK(sk_total(4) == 7, "kirby total");
}

static void test_partner(void) {
    sk_costume p, q, bad;
    const sk_costume *r;
    fresh();
    p = mk(10, "ic-a", 0, 0);
    q = mk(10, "ic-b", 0, 0); q.partner = 1;
    snprintf(q.pfile, sizeof q.pfile, "skins/ic-b/nana.dat"); snprintf(q.pjoint, sizeof q.pjoint, "NJ"); q.pmat[0] = '\0';
    bad = mk(11, "nana-direct", 0, 0);
    add(p); add(q); add(bad);
    sk_finalize(base_fn, name_fn);
    CHECK(sk_total(10) == 7 && sk_total(11) == 7, "popo %d nana %d", sk_total(10), sk_total(11));
    r = sk_row(11, 5);
    CHECK(r && r->partner_row && r->file[0] == '\0', "nana row 5 is the default (no file)");
    r = sk_row(11, 6);
    CHECK(r && r->partner_row && !strcmp(r->file, "skins/ic-b/nana.dat") && !strcmp(r->joint, "NJ"), "nana row 6 is the partner block");
    CHECK(logged("takes its main fighter's skins"), "direct nana skin refused with a message");
    CHECK(sk_from_wire(11, sk_to_wire(10, 6)) == 0, "a partner kind never matches a wire skin");
}

static void test_wire(void) {
    int i, w;
    sk_costume s;
    fresh();
    add(mk(0, "one", 0, 0)); add(mk(0, "two", 0, 0)); add(mk(40, "one", 0, 0));
    s = mk(41, "geno-skin", 0, 0); add(s);
    sk_finalize(base_fn, name_fn);
    for (i = 0; i < 5; ++i) CHECK(sk_to_wire(0, i) == i && sk_from_wire(0, i) == i, "base %d", i);
    w = sk_to_wire(0, 5);
    CHECK((w & SK_WIRE_TAG) && w > 0, "skin wire is tagged and positive: %d", w);
    CHECK(sk_from_wire(0, w) == 5, "round trip 5");
    CHECK(sk_from_wire(0, sk_to_wire(0, 6)) == 6, "round trip 6");
    /* the same wire value on another fighter is not that fighter's skin (the lookup is per fighter) */
    CHECK(sk_from_wire(40, w) == 7 || sk_from_wire(40, w) == 0, "cross-fighter safe");
    CHECK(sk_from_wire(1, w) == 0, "a fighter with no such skin: default");
    CHECK(sk_from_wire(0, 17) == 0, "an index past the fighter's own costumes on the wire is not a skin: default");
    CHECK(sk_from_wire(0, 254) == 0, "254 on the wire: default");
    CHECK(sk_from_wire(0, -1) == 0, "negative: default");
    CHECK(sk_from_wire(0, SK_WIRE_TAG | 0x12345) == 0, "unknown skin: default");
    CHECK(sk_from_wire(99, 3) == 3, "a fighter the registry does not know passes small values through");
    CHECK(sk_from_wire(41, sk_to_wire(41, 1)) == 1, "geno skin round trip");
    for (i = 5; i < 7; ++i) CHECK(sk_to_wire(0, i) >= SK_WIRE_TAG, "skin wires are >= tag");
}

static void test_refusals(void) {
    fresh();
    add(mk(2, "nolist", 0, 0));   /* base 0 */
    { sk_costume x = mk(999, "x", 0, 0); CHECK(sk_add(&x) != 0, "an out-of-range fighter is refused by add"); }
    sk_finalize(base_fn, name_fn);
    CHECK(sk_total(2) == 0 && logged("has no costume list"), "no list: skipped with a message");
}

static void test_stress(void) {
    int fk, i, total = 0;
    char m[32];
    fresh();
    for (fk = 0; fk < 2; ++fk)
        for (i = 0; i < 250; ++i) { snprintf(m, sizeof m, "k%d-%03d", fk, i); add(mk(fk, m, i % 3, 0)); }
    sk_finalize(base_fn, name_fn);
    for (fk = 0; fk < 2; ++fk) total += sk_added(fk);
    CHECK(sk_added(0) == 250 && sk_added(1) == 250, "added %d %d", sk_added(0), sk_added(1));
    for (i = 5; i < 255; ++i) {
        const sk_costume *r = sk_row(1, i);
        if (!r || r->index != i) { CHECK(0, "row %d", i); break; }
        if (sk_from_wire(1, sk_to_wire(1, i)) != i) { CHECK(0, "wire %d", i); break; }
    }
    (void) total;
}

int main(void) {
    test_order_and_indices();
    test_cap();
    test_team_like_kirby();
    test_kirby_explicit();
    test_partner();
    test_wire();
    test_refusals();
    test_stress();
    if (failures) { printf("skins core: %d FAILED\n", failures); return 1; }
    printf("skins core: all checks passed\n");
    return 0;
}
