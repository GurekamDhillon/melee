/* Production producer, queue, final-state reader and Lua payload with small
 * live-object fixtures. No simulation/rendering, disc, or game executable. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "../platform/gw_clank_event.h"
typedef struct { float x,y,z; } Vec3;
typedef struct GObj { struct GObj* next;void* user_data; } HSD_GObj;
typedef struct { int player_id,motion_id;HSD_GObj* gobj;struct {float x195c_hitlag_frames;} dmg; } Fighter;
typedef struct {HSD_GObj* entity;int kind;float xCBC_hitlagFrames;} Item;
typedef struct {float damage;Vec3 hurt_coll_pos;} HitCapsule;
static struct {float x3CC;} common={9};
static const void* unused;
#define p_ftCommonData (&common)
#define ftCo_MS_ReboundStop 237
#define ftCo_MS_Rebound 238
#define HSD_GOBJ_PLINK_ITEM 0
#define HSD_GOBJ_PLINK_FIGHTER 1
static HSD_GObj* object_heads[2];
static HSD_GObj** HSD_GObjPLinkHead=object_heads;
#include "../gameworld/script_clank.inc"
static int gw_ScriptGame_ClankValue(int a,int item,int rebound) { return ScriptGame_ClankValue(a,item,rebound); }
typedef struct {int unused;} lua_State;
typedef struct {int what;GsClank clank;} GsEvent;
#define GS_MAX_EVENTS 4
static struct {lua_State* L;int want_events,nev,ev_dropped;GsEvent ev[GS_MAX_EVENTS];} gs;
static int resim,gs_want_clanks;
static int gw_Snap_Resimulating(void) {return resim;}
static float gs_camera_float(int i) {union {int i;float f;} u;u.i=i;return u.f;}
static struct {char name[24];double value;} fields[32];static int nf;
static void lua_createtable(lua_State* L,int a,int b) {(void)L;(void)a;(void)b;nf=0;}
static void gs_setnum(lua_State* L,const char* k,double v) {(void)L;snprintf(fields[nf].name,24,"%s",k);fields[nf++].value=v;}
static void gs_setint(lua_State* L,const char* k,int v) {gs_setnum(L,k,v);}
static void gs_setbool(lua_State* L,const char* k,int v) {gs_setnum(L,k,v!=0);}
static double field(const char* k) {for(int i=0;i<nf;++i)if(!strcmp(fields[i].name,k))return fields[i].value;return -1;}
#include "../platform/gw_script_clank.inc"
static int Script_ClankWanted(void) {return gw_Script_ClankWanted();}
static void Script_Clank(int a,int b,int item,int kind,int x,int y,int z,int da,int db,int flags,int ea,int eb)
{gw_Script_Clank(a,b,item,kind,x,y,z,da,db,flags,ea,eb);}
#define TARGET_PC
#include "../gameworld/script_clank_observe.inc"
static HSD_GObj ga,gb,gi;
static Fighter a,b;static Item item;static HitCapsule ha,hb;
static void reset(void) {
    HSD_GObjPLinkHead=object_heads;
    memset(&gs,0,sizeof gs);gs.L=(lua_State*)1;gs.want_events=1;gs_want_clanks=1;
    a=(Fighter){0,237,&ga,{8}};b=(Fighter){1,238,&gb,{7}};item=(Item){&gi,48,5};
    ga=(HSD_GObj){&gb,&a};gb=(HSD_GObj){NULL,&b};gi=(HSD_GObj){NULL,&item};
    HSD_GObjPLinkHead[1]=&ga;HSD_GObjPLinkHead[0]=&gi;
    ha=(HitCapsule){12,{2,4,6}};hb=(HitCapsule){13,{4,8,10}};resim=0;
}
int main(void) {
    /* Match the real pointer table, including pre-match initialization. */
    HSD_GObjPLinkHead=NULL;
    assert(ScriptGame_ClankValue(0,0,0)==0);
    assert(ScriptGame_ClankValue(1,0,0)==0);
    assert(ScriptGame_ClankValue(1,1,1)==0);
    reset();ga.user_data=NULL;gi.user_data=NULL;
    assert(ScriptGame_ClankValue((int)&ga,0,0)==0);
    assert(ScriptGame_ClankValue((int)&ga,0,1)==0);
    assert(ScriptGame_ClankValue((int)&gi,1,0)==0);
    assert(ScriptGame_ClankValue((int)&gi,1,1)==0);
    assert(ScriptGame_ClankValue(0,0,0)==0);
    assert(ScriptGame_ClankValue(1,0,0)==0);
    reset();Fighter before=a;Item item_before=item;
    script_clank_observe(&a,&ha,&b,NULL,&hb);
    assert(gs.nev==1 && gs.ev[0].what==GS_EV_CLANK);
    gs_clank_snapshot(&gs.ev[0].clank);
    gs_clank_push(gs.L,&gs.ev[0].clank);
    assert(field("port_a")==1 && field("port_b")==2);
    assert(field("x")==3 && field("y")==6 && field("z")==8);
    assert(field("damage_a")==12 && field("damage_b")==13);
    assert(field("rebound")==1 && field("hitlag_a")==8 && field("hitlag_b")==7);
    assert(field("cancel_a")==1 && field("cancel_b")==1);
    /* Another subscriber's immediate gameplay write must not alter this event. */
    a.motion_id=14;a.dmg.x195c_hitlag_frames=99;
    gs_clanks_snapshot_pending(); /* dispatcher must not recapture post-hook writes */
    gs_clank_push(gs.L,&gs.ev[0].clank);
    assert(field("rebound_a")==1 && field("hitlag_a")==8);
    a=before;
    script_clank_observe(&b,&hb,&a,NULL,&ha);assert(gs.nev==1);
    assert(!memcmp(&a,&before,sizeof a) && !memcmp(&item,&item_before,sizeof item));
    reset();a.motion_id=14;script_clank_observe(&a,&ha,NULL,&item,&hb);
    gs_clank_snapshot(&gs.ev[0].clank);
    gs_clank_push(gs.L,&gs.ev[0].clank);
    assert(field("port_b")==-1 && field("item_kind")==48);
    assert(field("rebound")==0 && field("hitlag_a")==8 && field("hitlag_b")==5);
    HSD_GObjPLinkHead[0]=NULL;gs_clank_push(gs.L,&gs.ev[0].clank);assert(field("hitlag_b")==5);
    reset();script_clank_observe(&a,&ha,NULL,&item,&hb);HSD_GObjPLinkHead[0]=NULL;
    gs_clank_snapshot(&gs.ev[0].clank);gs_clank_push(gs.L,&gs.ev[0].clank);assert(field("hitlag_b")==0);
    reset();gs_want_clanks=0;script_clank_observe(&a,&ha,&b,NULL,&hb);assert(gs.nev==0);
    reset();gs.want_events=0;script_clank_observe(&a,&ha,&b,NULL,&hb);assert(gs.nev==0);
    gs.want_events=1;resim=1;script_clank_observe(&a,&ha,&b,NULL,&hb);assert(gs.nev==0);
    reset();ha.damage=25;script_clank_observe(&a,&ha,&b,NULL,&hb);
    assert(gs.ev[0].clank.cancel_a==0 && gs.ev[0].clank.cancel_b==1);
    reset();gs.nev=GS_MAX_EVENTS;script_clank_observe(&a,&ha,&b,NULL,&hb);assert(gs.ev_dropped==1);
    puts("clank producer/queue/payload: PASS");return 0;
}
