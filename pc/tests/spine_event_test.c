#define main refs_fixture_main
#include "spine_refs_api_test.c"
#undef main
#include "../platform/gw_script_spine_event.h"
typedef struct {int what,a,b,c,d;GsSpineEvent spine;} GsEvent;
static struct {int frame;} gs;
static int gs_in_frame;
static int gw_ScriptGame_EntityEpoch(void){return epoch;}
static int gw_ScriptGame_EntityAddressField(int kind,int address,int field){return address==1000?(field==6?1:gw_ScriptGame_EntityField(kind,1,field)):0;}
#define LAB_EV_ACTION 1
#define LAB_EV_HIT 2
#define LAB_EV_HITLAG 3
#define LAB_EV_LAND 4
#define LAB_HIT_ATTACKER_SUB 256
#define LAB_HIT_VICTIM_SUB 512
#define LAB_HIT_BY_ITEM 1024
#include "../platform/gw_script_spine_events.inc"
int main(void){GsEvent a={1,0,14,20,1},b={16,0,20,1,1},hit={2,0,1,LAB_HIT_ATTACKER_SUB|LAB_HIT_VICTIM_SUB,0};char old[80];
gs.frame=7;gs_spine_event_capture(&a);assert(a.spine.frame==7&&a.spine.sequence==1&&a.spine.phase==1);
gs_spine_address(old,1,1000);GsEntityRef address_ref;assert(gs_ref_parse(old,strlen(old),&address_ref)&&address_ref.id==1);
gs_spine_event_capture(&b);b.spine.cause=a.spine.sequence;assert(b.spine.sequence==2&&b.spine.cause==1);
gs_spine_event_capture(&hit);assert(hit.spine.sequence==3&&hit.spine.phase==2);
GsEntityRef r;assert(!hit.spine.actor[0]);assert(gs_ref_parse(a.spine.actor,strlen(a.spine.actor),&r)&&r.id==1);
assert(gs_ref_parse(hit.spine.target,strlen(hit.spine.target),&r)&&r.id==3);
strcpy(old,a.spine.actor);spawn=999;assert(!strcmp(old,a.spine.actor)); /* captured before retirement */
gs.frame=8;gs_spine_event_capture(&b);assert(b.spine.sequence==1);
gs_in_frame=1;gs_spine_event_capture(&b);assert(b.spine.frame==9);gs_in_frame=0;
gs_spine_post_pending=1;gs_spine_event_capture(&b);assert(b.spine.frame==9&&b.spine.sequence==2);
gs_spine_post_pending=0;gs.frame++;gs_spine_event_capture(&b);assert(b.spine.frame==9&&b.spine.sequence==3);
GsEvent item_hit={2,0,1,LAB_HIT_BY_ITEM|LAB_HIT_VICTIM_SUB,0};
gs_spine_event_capture(&item_hit);assert(!item_hit.spine.actor[0]&&item_hit.spine.target[0]);
/* Thrown-body context cannot establish its credited owner's sibling. */
GsEvent thrown_hit=hit;gs_spine_hit_context_actor(&thrown_hit);assert(!thrown_hit.spine.actor[0]&&thrown_hit.spine.target[0]);
epoch=2;gs_spine_event_capture(&b);assert(b.spine.epoch==2&&b.spine.sequence==1);
lua_State*L=luaL_newstate();gs_spine_event_push(L,&a,"on_action_change");lua_getfield(L,-1,"sequence");assert(lua_tointeger(L,-1)==1);lua_pop(L,1);lua_getfield(L,-1,"actor_ref");assert(!strcmp(lua_tostring(L,-1),old));lua_close(L);
puts("spine queued envelope: captured frame/sequence/phase/cause and retired primary/sub identities PASS");return 0;}
