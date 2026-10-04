/* Compile actual native event definition and append function extracted from
 * gw_script.c by _build/tmp/em4f1-context-test.py, then feed production game
 * helper callbacks from a surviving capsule across table replacement. */
#define main state_fixture_main
#include "hit_rules_state_test.c"
#undef main
#include "../gameworld/script_lab.h"
#include "../platform/gw_clank_event.h"
#include "../platform/gw_script_spine_event.h"
#include "em4f1-context-event.inc"
static struct {int want_events,nev;GsEvent ev[GS_MAX_EVENTS];} gs;
static int resim;
static int gw_Snap_Resimulating(void){return resim;}
#include "em4f1-context-append.inc"
int main(void) {
 GsEvent* e=&gs.ev[0];int i;
 test_context_replacement();
 gs.want_events=1;gs.nev=1;e->what=LAB_EV_HIT;e->b=1;
 for(i=0;i<trace_count;++i)gw_Script_HitRuleContext(1,trace[i],4,1,trace_owner[i]);
 printf("helper callbacks=%d, native event IDs=%d, last ID=%d; event size=%u bytes\n",trace_count,e->hit_rule_count,e->hit_rule_ids[e->hit_rule_count-1],(unsigned)sizeof *e);
 assert(e->hit_rule_count==96);
 for(i=0;i<96;++i)assert(e->hit_rule_ids[i]==trace[i] && e->hit_rule_owners[i]==trace_owner[i]);
 /* Duplicates and an extra append after the proven maximum stay bounded. */
 gw_Script_HitRuleContext(1,931,4,1,13);gw_Script_HitRuleContext(1,999,4,1,13);assert(e->hit_rule_count==96);
 memset(e,0,sizeof *e);e->what=LAB_EV_HIT;e->b=1;resim=1;
 gw_Script_HitRuleContext(1,900,4,1,13);assert(!e->hit_rule_count);resim=0;
 gw_Script_HitRuleContext(2,900,4,1,13);assert(!e->hit_rule_count);
 puts("actual native context append retains historical/current/incoming96 PASS");return 0;
}
