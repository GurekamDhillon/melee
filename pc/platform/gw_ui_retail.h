/* gw_ui_retail.h - which retail elements Atlas hides, and the retail pause as Atlas sees it. Pure C.
 *
 * The mask has four sources (the environment, the console, one script, the scene policy) and is EMPTY by default. Its effective
 * value is 0 while netplay or rollback is on: nothing retail is ever hidden online. The pause state machine follows the retail
 * pause (gm_DoPauseChecksAndRoutine / gm_DoUnpauseChecksAndRoutine); a takeover is offline only, and an unpause request is a
 * one-shot that never leaks into a later pause. */
#ifndef GW_UI_RETAIL_H
#define GW_UI_RETAIL_H
#include "gw_ui_retail_ids.h"
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_RS_ENV, AT_RS_CONSOLE, AT_RS_SCRIPT, AT_RS_POLICY, AT_RS_COUNT };
typedef struct { unsigned src[AT_RS_COUNT]; int script_owner; } AtRetail;     /* script_owner: script slot + 1, 0 none */

const char *at_retail_name(int id);                   /* "hud.damage" ... "pause.panel"; NULL out of range */
int at_retail_parse(const char *list, unsigned *mask, char *err, int cap); /* "hud.damage,hud.stock" or "all" or "" */
unsigned at_retail_script_allowed(void);              /* what a mod may hide: everything but hud.timer */
void at_retail_set(AtRetail *r, int source, unsigned mask);
unsigned at_retail_effective(const AtRetail *r, int online);   /* the OR of the sources; 0 when online */
int at_retail_hidden(const AtRetail *r, int id, int online);
void at_retail_release_script(AtRetail *r, int owner);         /* the script source clears when its owner goes */

typedef struct { int paused, pauser, takeover, unpause_req; } AtPause;
void at_pause_on(AtPause *p, int pauser, int takeover_wanted, int online);   /* takeover only offline */
void at_pause_off(AtPause *p);                                               /* clears any request too */
int  at_pause_request_unpause(AtPause *p, int online);                       /* 1 accepted: paused, offline, none pending */
int  at_pause_take_unpause(AtPause *p);                                      /* the pauser once, else -1 */

#ifdef __cplusplus
}
#endif
#endif
