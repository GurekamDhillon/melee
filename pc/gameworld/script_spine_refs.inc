/* No host allocator/cache: retail identities and this scene epoch are MEM1
 * snapshot state. Querying identity must not mutate simulation memory. */
#include "script_spine_refs.h"
static unsigned script_spine_epoch=1;
static unsigned script_spine_ground_generation[64];
static int script_spine_ground_address[64];
int ScriptGame_EntityEpoch(void){return (int)script_spine_epoch;}
void ScriptGame_RefsGroundBorn(int id,int address)
{
    if(id<0||id>=64)return;
    script_spine_ground_address[id]=0;
    if(script_spine_ground_generation[id]==0x7fffffffU)return;
    ++script_spine_ground_generation[id];script_spine_ground_address[id]=address;
}
void ScriptGame_RefsSceneEnd(void)
{
    /* Exhaustion disables refs instead of making an obsolete ref valid. */
    memset(script_spine_ground_address,0,sizeof script_spine_ground_address);
    if(!script_spine_epoch)return;
    if(script_spine_epoch<0x7fffffffU)++script_spine_epoch;
    else script_spine_epoch=0;
}
int ScriptGame_EntityField(int kind,int id,int field)
{
    Fighter* fp=NULL;Item* ip=NULL;int e=-1,i,definition=0,generation=0,roles=0;
    if(!script_spine_epoch||field<0||field>5)return 0;
    if(kind==1) {
        if(id<0||id>=12)return 0;
        e=id;fp=ScriptGame_CapsFighter(id/2+1+(id%2)*6);
    } else if(kind==2) {
        for(i=0;i<ScriptGame_ItemCount();++i){Item* candidate=script_item(i);
            if(candidate&&candidate->x1C==id){ip=candidate;break;}}
        if(!ip)return 0;
        definition=ip->kind;generation=ip->x1C;
        if(GENO_ART_IS_KIND(ip->kind))roles|=2;
        for(i=0;i<12;++i){Fighter* owner=ScriptGame_CapsFighter(i/2+1+(i%2)*6);
            if(ip->owner&&owner&&owner->gobj==ip->owner){e=i;break;}}
    } else if(kind==3) {
        if(id<=0)return 0;
        for(i=0;i<12&&e<0;++i){int r;
            for(r=0;r<8;++r)if(ScriptGame_EchoRead(i,r,0)==id){e=i;break;}}
        if(e>=0)fp=ScriptGame_CapsFighter(e/2+1+(e%2)*6);
    } else if(kind==4) {
        if(id<=0)return 0;
        for(i=0;i<SCRIPT_STAGE_MODELS;++i)if(script_stage.model[i].active&&script_stage.model[i].handle==id)definition=2;
        for(i=0;i<SCRIPT_STAGE_LINES;++i)if(script_stage.line[i].active&&script_stage.line[i].handle==id)definition=1;
        for(i=0;i<SCRIPT_STAGE_TARGETS;++i)if(script_stage.target[i].active&&script_stage.target[i].handle==id)definition=3;
        for(i=0;i<SCRIPT_STAGE_ENEMIES;++i)if(script_stage.enemy[i].active&&script_stage.enemy[i].handle==id)definition=4;
        for(i=0;i<SCRIPT_MESH_INSTANCES;++i)if(script_stage.instance[i].handle==id)definition=5;
        if(!definition)return 0;generation=id;
    } else if(kind==5) {
        HSD_GObj* g;int found=0;
        if(id<0||id>=64||!script_spine_ground_address[id]||!HSD_GObjPLinkHead)return 0;
        /* Ground_GetStageGObj creates map actors on process link5. Never
         * dereference a table entry until the live list agrees. */
        for(g=HSD_GObjPLinkHead[5];g;g=g->next)
            if((int)g==script_spine_ground_address[id]&&stage_info.map_gobjs[id]==g){found=1;break;}
        if(!found)return 0;
        generation=(int)script_spine_ground_generation[id];definition=id;
    } else return 0;
    if(kind==1||kind==3){
        if(!fp)return 0;
        generation=fp->x8_spawnNum;definition=fp->kind;
        if(ScriptGame_FighterI(e/2,SCRIPT_I_SLOT_TYPE)==1)roles|=1;
    }
    switch(field){case 0:return 1;case 1:return (int)script_spine_epoch;
        case 2:return generation;case 3:return definition;case 4:return e+1;case 5:return roles;}
    return 0;
}
/* Existing clank telemetry uses a same-frame GObj address. Validate it against
 * live lists in game code; the canonical reference never contains the address. */
int ScriptGame_EntityAddressField(int kind,int address,int field)
{
    int i,id=-1,found=0;if(!address)return 0;
    if(kind==1)for(i=0;i<12;++i){Fighter* fp=ScriptGame_CapsFighter(i/2+1+(i%2)*6);
        if(fp&&(int)fp->gobj==address){id=i;found=1;break;}}
    else if(kind==2)for(i=0;i<ScriptGame_ItemCount();++i){Item* ip=script_item(i);
        if(ip&&(int)ip->entity==address){id=ip->x1C;found=1;break;}}
    if(!found)return 0;
    return field==6?id:ScriptGame_EntityField(kind,id,field);
}
