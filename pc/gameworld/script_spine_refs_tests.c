#include <assert.h>
#include <stdio.h>
#include <string.h>
typedef struct Fighter {int x8_spawnNum,kind,player_id,is_sub_fighter;void* gobj;} Fighter;
typedef struct Item {int x1C,kind;void* owner;void* entity;} Item;
typedef struct {int handle,active,kind;} Object;
typedef struct HSD_GObj {struct HSD_GObj* next;} HSD_GObj;
static HSD_GObj* links[6],**HSD_GObjPLinkHead=links;
static struct {HSD_GObj* map_gobjs[64];} stage_info;
#define SCRIPT_STAGE_LINES 2
#define SCRIPT_STAGE_MODELS 2
#define SCRIPT_STAGE_TARGETS 2
#define SCRIPT_STAGE_ENEMIES 2
#define SCRIPT_MESH_INSTANCES 2
static struct {Object line[2],model[2],target[2],enemy[2],instance[2];} script_stage;
static Fighter fighters[12],*live[12];
static Item items[3];static int item_count;
static Fighter* ScriptGame_CapsFighter(int e){return e>=1&&e<=12?live[e-1]:0;}
static Item* script_item(int i){return i>=0&&i<item_count?items+i:0;}
static int ScriptGame_ItemCount(void){return item_count;}
static int echo_handle[12];
static int ScriptGame_EchoRead(int e,int i,int f){return i==0&&f==0?echo_handle[e]:0;}
static int ScriptGame_FighterI(int p,int f){(void)p;(void)f;return 1;}
#define SCRIPT_I_SLOT_TYPE 7
#define GENO_ART_IS_KIND(k) ((k)>=1000&&(k)<2000)
#include "script_spine_refs.inc"
typedef struct {unsigned epoch,generation[64];int address[64];} RefSnapshot;
static void snapshot(RefSnapshot* s){s->epoch=script_spine_epoch;memcpy(s->generation,script_spine_ground_generation,sizeof s->generation);memcpy(s->address,script_spine_ground_address,sizeof s->address);}
static void restore(const RefSnapshot* s){script_spine_epoch=s->epoch;memcpy(script_spine_ground_generation,s->generation,sizeof s->generation);memcpy(script_spine_ground_address,s->address,sizeof s->address);}
static void replay_identity(int address){ScriptGame_RefsSceneEnd();ScriptGame_RefsGroundBorn(2,address);ScriptGame_RefsGroundBorn(2,address);}
int main(void){unsigned saved;int e;
for(e=0;e<12;++e){fighters[e].x8_spawnNum=100+e;fighters[e].kind=10+e;live[e]=&fighters[e];}
assert(ScriptGame_EntityField(1,0,2)==100);
assert(ScriptGame_EntityField(1,1,2)==106); /* interleaved Nana -> capability7 */
fighters[6].gobj=(void*)1000;assert(ScriptGame_EntityAddressField(1,1000,6)==1);
assert(ScriptGame_EntityAddressField(1,1000,2)==106);assert(!ScriptGame_EntityAddressField(1,999,0));
assert(ScriptGame_EntityField(1,10,2)==105); /* primary6 */
assert(ScriptGame_EntityField(1,12,0)==0);
saved=script_spine_epoch;assert(saved);
items[0]=(Item){700,1,0};items[1]=(Item){701,1001,0};item_count=2;
assert(ScriptGame_EntityField(2,700,3)==1);
assert(ScriptGame_EntityField(2,700,4)==0);
assert(ScriptGame_EntityField(2,701,5)&2);
items[0]=items[1];item_count=1;assert(!ScriptGame_EntityField(2,700,0));
assert(ScriptGame_EntityField(2,701,2)==701); /* reordered list */
echo_handle[6]=88;assert(ScriptGame_EntityField(3,88,4)==7);
assert(ScriptGame_EntityField(3,88,2)==103);
fighters[3].x8_spawnNum=999;assert(ScriptGame_EntityField(3,88,2)==999);
fighters[3].kind=42;assert(ScriptGame_EntityField(1,6,3)==42);
live[3]=0;assert(!ScriptGame_EntityField(3,88,0));live[3]=&fighters[3];
script_stage.model[0]=(Object){90,1,0};assert(ScriptGame_EntityField(4,90,3)==2);
script_stage.model[0].active=0;assert(!ScriptGame_EntityField(4,90,0));
script_stage.instance[0].handle=91;assert(ScriptGame_EntityField(4,91,3)==5);
script_stage.instance[0].handle=0;assert(!ScriptGame_EntityField(4,91,0));
script_stage.instance[0].handle=91;assert(ScriptGame_EntityField(4,91,3)==5);
assert(saved==script_spine_epoch); /* reads never allocate/mutate */
ScriptGame_RefsSceneEnd();assert(script_spine_epoch==saved+1);
HSD_GObj ground={0};links[5]=&ground;stage_info.map_gobjs[2]=&ground;
ScriptGame_RefsGroundBorn(2,(int)&ground);assert(ScriptGame_EntityField(5,2,0));
int incarnation=ScriptGame_EntityField(5,2,2);ScriptGame_RefsGroundBorn(2,(int)&ground);
assert(ScriptGame_EntityField(5,2,2)!=incarnation);links[5]=0;assert(!ScriptGame_EntityField(5,2,0));
/* Standalone byte-copy replay of the production reference state, not a LAB
 * snapshot test. Existing retail actor/heap restoration is tested separately. */
RefSnapshot before,first,second;snapshot(&before);assert(sizeof before==516);
replay_identity((int)&ground);snapshot(&first);restore(&before);
replay_identity((int)&ground);snapshot(&second);assert(!memcmp(&first,&second,sizeof first));
restore(&before);puts("spine reference-state standalone snapshot/replay: 516 bytes, zero differences PASS");
script_spine_epoch=saved;assert(script_spine_epoch==saved); /* snapshotted scalar restore */
script_spine_epoch=0x7fffffff;ScriptGame_RefsSceneEnd();assert(!ScriptGame_EntityField(1,0,0));
ScriptGame_RefsSceneEnd();assert(!ScriptGame_EntityField(1,0,0)); /* exhausted never wraps */
puts("spine game identity: primary/Nana/CPU item serial/article echo owner transform scene query purity PASS");return 0;}
