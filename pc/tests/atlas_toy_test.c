/* atlas-toy: the chrome text of the framed trophy scenes. No game, no disc: every name below is invented. */
#include "atlas_check.h"
#include "../platform/gw_ui_toy.h"

static void gallery(void)
{
    AtToyChrome c;
    char longd[260], longn[200], longs[200];
    int i;
    /* the counter and the authored fallbacks when no retail text could be decoded */
    at_toy_chrome(&c, 6, 293, "", "", "");
    CHECK_STR(c.counter, "7 / 293"); CHECK_STR(c.title, "TROPHY 7"); CHECK(c.what[0] == '\0'); CHECK(c.from[0] == '\0');
    /* decoded text wins, clipped to the explainer's limits */
    at_toy_chrome(&c, 0, 1, "Invented Trophy", "A short invented description.", "Invented series");
    CHECK_STR(c.title, "Invented Trophy"); CHECK_STR(c.what, "A short invented description."); CHECK_STR(c.from, "Invented series"); CHECK_STR(c.counter, "1 / 1");
    for (i = 0; i < 259; i++) longd[i] = (char) ('a' + i % 26);
    longd[259] = '\0';
    at_toy_chrome(&c, 0, 5, "T", longd, ""); CHECK(strlen(c.what) <= 110);
    CHECK(strlen(c.what) == 108 || strlen(c.what) == 110 || strlen(c.what) > 100);          /* cut near the limit, not at a stub */
    CHECK(strcmp(c.what + strlen(c.what) - 3, "\xE2\x80\xA6") == 0);                         /* and it says so */
    for (i = 0; i < 199; i++) { longn[i] = (char) ('A' + i % 26); longs[i] = (char) ('a' + i % 26); }
    longn[199] = longs[199] = '\0';
    at_toy_chrome(&c, 0, 5, longn, "", longs); CHECK(strlen(c.title) <= 63 && strlen(c.from) <= 63);
    /* the edges: nothing owned, the index past the count (an m-ex trophy), a negative index, one trophy */
    at_toy_chrome(&c, 0, 0, "", "", "");     CHECK_STR(c.counter, "");  CHECK_STR(c.title, "NO TROPHIES YET");
    at_toy_chrome(&c, 3, -4, "x", "y", "z"); CHECK_STR(c.counter, "");  CHECK_STR(c.title, "NO TROPHIES YET"); CHECK(c.what[0] == '\0');
    at_toy_chrome(&c, 299, 293, "", "", ""); CHECK_STR(c.counter, "293 / 293");              /* clamped to the last */
    at_toy_chrome(&c, 300, 300, "", "", ""); CHECK_STR(c.counter, "300 / 300");              /* an m-ex gallery past the retail count */
    at_toy_chrome(&c, -3, 293, "", "", "");  CHECK_STR(c.counter, "1 / 293");
    at_toy_chrome(&c, 0, 1, "", "", "");     CHECK_STR(c.counter, "1 / 1");
    /* only the first sentence of a description is the rule line */
    at_toy_chrome(&c, 0, 3, "T", "First sentence. Second sentence here.", ""); CHECK_STR(c.what, "First sentence.");
    at_toy_chrome(&c, 0, 3, "T", "Who? Me! Yes.", ""); CHECK_STR(c.what, "Who?");
    at_toy_chrome(&c, 0, 3, "T", "v1.5 is a version. Next.", ""); CHECK_STR(c.what, "v1.5 is a version.");   /* a dot inside a word is not a sentence end */
    at_toy_chrome(&c, 0, 3, "T", "No full stop here", ""); CHECK_STR(c.what, "No full stop here");
    at_toy_chrome(&c, 0, 3, NULL, NULL, NULL); CHECK_STR(c.title, "TROPHY 1");                 /* NULL is empty */
    /* a clip never splits a UTF-8 character: a run of two-byte characters */
    { char u[260]; for (i = 0; i < 256; i += 2) { u[i] = (char) 0xC3; u[i + 1] = (char) 0xA9; } u[256] = '\0';
      at_toy_chrome(&c, 0, 3, "T", u, "");
      for (i = 0; c.what[i] != '\0'; i++) if (((unsigned char) c.what[i] & 0xC0) == 0x80) CHECK(i > 0 && ((unsigned char) c.what[i - 1] & 0x80) != 0);
      CHECK(strlen(c.what) <= 110 && (strlen(c.what) - 3) % 2 == 0); }
}

static void lottery(void)
{
    AtLotChrome c;
    int coins[] = { 0, 1, 99, 100 }, k;
    at_lot_chrome(&c, 0, 0);
    CHECK_STR(c.title, "NO COINS"); CHECK(strstr(c.what, "coins") != NULL); CHECK_STR(c.counter, "");
    at_lot_chrome(&c, -5, 3); CHECK_STR(c.title, "NO COINS");                               /* a negative balance is no balance */
    at_lot_chrome(&c, 1, 1); CHECK_STR(c.title, "1 COIN"); CHECK_STR(c.counter, "BET 1 / 1");
    at_lot_chrome(&c, 12, 3); CHECK_STR(c.title, "12 COINS"); CHECK_STR(c.counter, "BET 3 / 12");
    at_lot_chrome(&c, 99, 20); CHECK_STR(c.title, "99 COINS"); CHECK_STR(c.counter, "BET 20 / 20");   /* retail's ceiling is 20 */
    at_lot_chrome(&c, 100, 50); CHECK_STR(c.counter, "BET 20 / 20");                         /* a bet past the ceiling is clamped */
    at_lot_chrome(&c, 5, 9); CHECK_STR(c.counter, "BET 5 / 5");                              /* ... and past the balance */
    at_lot_chrome(&c, 5, 0); CHECK_STR(c.counter, "");                                       /* no bet chosen yet: no readout */
    for (k = 0; k < 4; k++) {                                                                /* no invented numbers: the only digits are the balance and the bet */
        int i; at_lot_chrome(&c, coins[k], 1);
        for (i = 0; c.what[i] != '\0'; i++) CHECK(c.what[i] < '0' || c.what[i] > '9');
        CHECK(strlen(c.title) < AT_TOY_TITLE && strlen(c.what) < AT_TOY_WHAT);
    }
}

static void collection(void)
{
    AtCollChrome c;
    at_coll_chrome(&c, 0); CHECK_STR(c.title, "TROPHY ROOM"); CHECK_STR(c.counter, ""); CHECK(c.what[0] != '\0');
    at_coll_chrome(&c, 1); CHECK_STR(c.counter, "1 TROPHY");
    at_coll_chrome(&c, 293); CHECK_STR(c.counter, "293 TROPHIES");
    at_coll_chrome(&c, 300); CHECK_STR(c.counter, "300 TROPHIES");                          /* m-ex adds trophies: the count follows the getter */
    at_coll_chrome(&c, -1); CHECK_STR(c.counter, "");
}

int main(void)
{
    gallery(); lottery(); collection();
    ATLAS_DONE("atlas toy");
}
