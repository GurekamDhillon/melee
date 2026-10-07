/* atlas-results: the results card set from a match summary, the per-human confirm and the exit rule, as fn_80178050 and fn_801791E4 (gmresultplayer.c)
 * have them. Invented inputs only. */
#include "atlas_check.h"
#include "../platform/gw_ui_results.h"

int main(void)
{
    AtResults r; AtResInput in; int i;
    AtDataRow rows[4]; const char *names[4] = { "Fox", "Marth", "", "Falco" };

    memset(&in, 0, sizeof in);
    in.outcome = 2; in.is_teams = 0;                                   /* elimination, free for all */
    for (i = 0; i < 4; i++) in.p[i].pkind = AT_PK_NONE;
    in.p[0].pkind = AT_PK_HUMAN; in.p[0].ckind = 1; in.p[0].stocks = 2; in.p[0].kos = 4; in.p[0].falls = 1; in.p[0].percent = 87; in.p[0].winner = 1;
    in.p[1].pkind = AT_PK_CPU; in.p[1].ckind = 2; in.p[1].stocks = 0; in.p[1].kos = 1; in.p[1].falls = 4; in.p[1].percent = 130;
    at_results_build(&r, &in);
    CHECK(r.n == 2 && r.players[0].port == 0 && r.players[1].port == 1);          /* none-ports are skipped */
    CHECK(r.players[0].place == 1 && r.players[1].place == 2 && r.players[0].winner);
    CHECK(!r.canceled && r.humans == 1);
    CHECK(!at_results_done(&r));
    at_results_confirm(&r, 1, AT_RC_START);                                        /* a CPU port cannot confirm */
    CHECK(!at_results_done(&r));
    at_results_confirm(&r, 3, AT_RC_START);                                        /* neither can an empty one, or one that is not there */
    at_results_confirm(&r, -1, AT_RC_START);
    at_results_confirm(&r, 9, AT_RC_START);
    CHECK(!at_results_done(&r));
    at_results_confirm(&r, 0, AT_RC_START);
    CHECK(at_results_done(&r) == 1 && r.exit_frames == 10);                        /* any human present: the 0x0A countdown (fn_80178050) */
    at_results_confirm(&r, 0, AT_RC_START);                                        /* a second press changes nothing */
    CHECK(at_results_done(&r) == 1 && r.exit_frames == 10);

    /* four players, all human: every one must confirm; the countdown is still 0x0A */
    memset(&in, 0, sizeof in); in.outcome = 2;
    for (i = 0; i < 4; i++) { in.p[i].pkind = AT_PK_HUMAN; in.p[i].ckind = i; in.p[i].kos = i; }
    at_results_build(&r, &in);
    CHECK(r.humans == 4);
    at_results_confirm(&r, 0, AT_RC_START); at_results_confirm(&r, 1, AT_RC_START); at_results_confirm(&r, 2, AT_RC_START);
    CHECK(!at_results_done(&r));
    at_results_confirm(&r, 3, AT_RC_ERR);                                          /* a disconnected pad counts as confirmed, as retail does (err != 0) */
    CHECK(at_results_done(&r) == 1 && r.exit_frames == 10);

    /* no human at all (four CPUs): nobody has to confirm, the countdown is 0x14 */
    memset(&in, 0, sizeof in); in.outcome = 2;
    for (i = 0; i < 4; i++) in.p[i].pkind = AT_PK_CPU;
    at_results_build(&r, &in);
    CHECK(r.humans == 0 && at_results_done(&r) == 1 && r.exit_frames == 20);

    /* a cancelled match: any human START leaves at once; a disconnected pad does not */
    memset(&in, 0, sizeof in); in.outcome = 4; in.canceled = 1; in.p[0].pkind = AT_PK_HUMAN; in.p[1].pkind = AT_PK_HUMAN; in.p[2].pkind = AT_PK_NONE; in.p[3].pkind = AT_PK_NONE;
    at_results_build(&r, &in);
    CHECK(r.canceled);
    at_results_confirm(&r, 1, AT_RC_ERR);
    CHECK(!at_results_done(&r));
    at_results_confirm(&r, 1, AT_RC_START);
    CHECK(at_results_done(&r) == 1 && r.exit_frames == 0);

    /* teams: place by team; a tie shares the place */
    memset(&in, 0, sizeof in); in.outcome = 3; in.is_teams = 1;
    for (i = 0; i < 4; i++) { in.p[i].pkind = AT_PK_HUMAN; in.p[i].team = i < 2 ? 0 : 1; in.p[i].winner = i < 2; in.p[i].stocks = i < 2 ? 1 : 0; }
    at_results_build(&r, &in);
    CHECK(r.players[0].place == 1 && r.players[1].place == 1 && r.players[2].place == 2 && r.players[3].place == 2);

    /* free for all: more stocks, then less damage; equal everything shares the place (a draw) */
    memset(&in, 0, sizeof in); in.outcome = 2;
    for (i = 0; i < 4; i++) { in.p[i].pkind = AT_PK_HUMAN; in.p[i].stocks = 1; in.p[i].percent = 50; }
    in.p[2].stocks = 3; in.p[3].percent = 10;
    at_results_build(&r, &in);
    CHECK(r.players[2].place == 1 && r.players[3].place == 2 && r.players[0].place == 3 && r.players[1].place == 3);
    in.p[2].winner = 1; in.p[0].stocks = 9;                                        /* a winner is first whatever the stocks say */
    at_results_build(&r, &in);
    CHECK(r.players[2].place == 1 && r.players[0].place == 2);

    /* the rows: the winner's row first, then by place, each port's name, the numbers in words */
    memset(&in, 0, sizeof in); in.outcome = 2;
    for (i = 0; i < 4; i++) in.p[i].pkind = AT_PK_NONE;
    in.p[0].pkind = AT_PK_HUMAN; in.p[0].stocks = 0; in.p[0].kos = 1; in.p[0].falls = 4; in.p[0].percent = 130;
    in.p[1].pkind = AT_PK_CPU; in.p[1].stocks = 2; in.p[1].kos = 4; in.p[1].falls = 1; in.p[1].percent = 87; in.p[1].winner = 1;
    in.p[3].pkind = AT_PK_HUMAN; in.p[3].stocks = 1; in.p[3].kos = 0; in.p[3].falls = 2; in.p[3].percent = 5;
    at_results_build(&r, &in);
    CHECK(at_results_rows(&r, names, rows, 4) == 3);
    CHECK_STR(rows[0].label, "P2 CPU  Marth"); CHECK_STR(rows[0].value, "WINNER"); CHECK(rows[0].flags & AT_DR_WIN);
    CHECK_STR(rows[0].sub, "4 KOs, 1 fall, 87%");
    CHECK_STR(rows[1].label, "P4  Falco"); CHECK_STR(rows[1].value, "2nd"); CHECK_STR(rows[1].sub, "0 KOs, 2 falls, 5%");
    CHECK_STR(rows[2].label, "P1  Fox"); CHECK_STR(rows[2].value, "3rd"); CHECK_STR(rows[2].sub, "1 KO, 4 falls, 130%");
    in.p[0].percent = 5; in.p[0].stocks = 1; in.p[0].kos = 0; in.p[0].falls = 2;   /* a tie: the same place, "2nd" for both */
    at_results_build(&r, &in);
    CHECK(at_results_rows(&r, names, rows, 4) == 3 && strcmp(rows[1].value, "2nd") == 0 && strcmp(rows[2].value, "2nd") == 0);
    at_results_confirm(&r, 3, AT_RC_START);                                                          /* a confirmed human says so; the others do not */
    CHECK(at_results_rows(&r, names, rows, 4) == 3 && strncmp(rows[2].sub, "READY  ", 7) == 0 && strncmp(rows[1].sub, "READY", 5) != 0);
    CHECK(at_results_rows(&r, names, rows, 2) == 2);                                                 /* never past the caller's room */
    CHECK(at_results_rows(&r, names, rows, 0) == 0);
    CHECK_STR(at_place_word(1), "1st"); CHECK_STR(at_place_word(2), "2nd"); CHECK_STR(at_place_word(3), "3rd"); CHECK_STR(at_place_word(4), "4th"); CHECK_STR(at_place_word(0), "");

    /* out of range inputs are pulled in, never trusted */
    memset(&in, 0, sizeof in); in.outcome = 2; in.p[0].pkind = 77; in.p[1].pkind = AT_PK_HUMAN; in.p[1].percent = 70000; in.p[1].kos = -3;
    in.p[2].pkind = AT_PK_NONE; in.p[3].pkind = AT_PK_NONE;
    at_results_build(&r, &in);
    CHECK(r.n == 1 && r.players[0].port == 1 && r.players[0].kos == 0 && r.players[0].percent == 999);
    ATLAS_DONE("atlas results");
}
