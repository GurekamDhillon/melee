/* gw_ui_results.h - the results card set from a match summary, the per-human confirm and the exit rule (Atlas step 8). Pure C; inputs are plain numbers
 * the game side read from MatchEnd (gm/types.h). The rules are the retail ones:
 *   - a human confirms with START, or with a pad error (HSD_PadCopyStatus.err != 0 counts as confirmed: fn_80178050); a CPU or an empty port never does;
 *   - when every human has confirmed the scene counts 0x0A frames, or 0x14 when no slot is human at all (lbl_804D3FC8 in fn_80178050), then exits;
 *   - a cancelled match leaves at once on any human's START (a pad error does not: fn_801791E4 asks err == 0). */
#ifndef GW_UI_RESULTS_H
#define GW_UI_RESULTS_H
#include "gw_ui_data.h"
#ifdef __cplusplus
extern "C" {
#endif
enum { AT_PK_HUMAN = 0, AT_PK_CPU = 1, AT_PK_OTHER = 2, AT_PK_NONE = 3 };   /* Gm_PKind */
enum { AT_RC_START = 1, AT_RC_ERR = 2 };
typedef struct { int pkind, ckind, stocks, kos, falls, percent, team, winner; } AtResPlayerIn;
typedef struct { int outcome, is_teams, canceled; AtResPlayerIn p[4]; } AtResInput;
typedef struct { int port, pkind, ckind, stocks, kos, falls, percent, team, winner, place, confirmed; } AtResPlayer;
typedef struct { AtResPlayer players[4]; int n, humans, canceled, exit_frames, done; } AtResults;

/* skips empty ports (and any kind that is not a player), clamps what it reads, counts the humans, and sets the places: the winner first, then more
 * stocks, then less damage; a team shares its place (the winning team first, then by stocks and damage); equal standings share a place. */
void at_results_build(AtResults *r, const AtResInput *in);
void at_results_confirm(AtResults *r, int port, int kind);
int  at_results_done(const AtResults *r);
/* the rows by place (ties by port): "P2 CPU  Marth", WINNER or "2nd", "4 KOs, 1 fall, 87%", with "READY  " in front once that human has confirmed. names[port] may be "" or NULL. Returns the count (at most cap). */
int  at_results_rows(const AtResults *r, const char *const names[4], AtDataRow *rows, int cap);
const char *at_place_word(int place);   /* "1st" .. "4th", "" outside */
#ifdef __cplusplus
}
#endif
#endif
